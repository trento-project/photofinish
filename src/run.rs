// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

use crate::config::Scenario;
use reqwest::{ClientBuilder, StatusCode};
use std::fs;
use tokio::time::sleep;

#[derive(Debug)]
enum FixtureResult {
    Success,
    Retryable { file: String },
    Skippable,
    Unauthorized,
}

struct Errored {
    file: String,
    reason: String,
}

async fn post_fixture(
    remote_endpoint: &str,
    api_key: &str,
    file: &str,
    insecure: bool,
) -> Result<FixtureResult, Errored> {
    let http_client = ClientBuilder::new().danger_accept_invalid_certs(insecure).build().unwrap();
    let canonical_path = fs::canonicalize(file).unwrap_or_default();
    let processed_fixture = file.to_string();

    match fs::read_to_string(canonical_path) {
        Ok(file_content) => {
            let response = http_client
                .post(remote_endpoint)
                .body(file_content)
                .header("x-trento-apikey", api_key)
                .header("Content-Type", "application/json")
                .send()
                .await;
            match response {
                Ok(res) => match res.status() {
                    StatusCode::ACCEPTED => {
                        println!("Successfully POSTed file: {}", file);
                        Ok(FixtureResult::Success)
                    }
                    StatusCode::UNAUTHORIZED => {
                        println!("POST request unauthorized. Set the correct API_KEY as argument");
                        Ok(FixtureResult::Unauthorized)
                    }
                    StatusCode::BAD_REQUEST
                    | StatusCode::UNPROCESSABLE_ENTITY
                    | StatusCode::NOT_FOUND => Ok(FixtureResult::Retryable {
                        file: processed_fixture,
                    }),
                    status_code => {
                        println!(
                            "Unexpected status code {} while POSTing fixture: {}",
                            status_code, file
                        );
                        Ok(FixtureResult::Skippable)
                    }
                },
                Err(err) => {
                    println!("Error while POSTing fixture: {}", file);
                    Err(Errored {
                        file: processed_fixture,
                        reason: err.to_string(),
                    })
                }
            }
        }
        Err(_) => {
            println!("Couldn't read file: {}", file);
            Err(Errored {
                file: processed_fixture,
                reason: "Couldn't read file".to_string(),
            })
        }
    }
}

fn scan_directory(directory: &str) -> Result<Vec<String>, std::io::Error> {
    let files_list = fs::read_dir(directory)?
        .filter_map(|file| {
            file.ok().and_then(|e| match e.path().is_file() {
                true => e.path().to_str().map(|s| s.to_string()),
                false => None,
            })
        })
        .collect();
    Ok(files_list)
}

pub async fn run(
    remote_endpoint: &str,
    insecure: bool,
    api_key: &str,
    scenario_label: String,
    scenarios: Vec<Scenario>,
    wait: u64,
) -> Result<(), ()> {
    let selected_scenario = scenarios
        .iter()
        .find(|current_scenario| current_scenario.label == scenario_label);

    match selected_scenario {
        None => {
            println!("Non-existing scenario!");
            return Err(());
        }
        Some(scenario) => {
            let mut fixtures_in_directories: Vec<String> = scenario
                .directories
                .iter()
                .filter_map(extract_fixtures_from_directory)
                .flatten()
                .collect();
            fixtures_in_directories.sort();

            let full_scenario = [&scenario.files[..], &fixtures_in_directories[..]].concat();

            let mut retryable: Vec<FixtureResult> = vec![];

            for file in full_scenario.iter() {
                let execution_result = post_fixture(remote_endpoint, api_key, file, insecure).await;
                match execution_result {
                    Ok(FixtureResult::Retryable { file }) => {
                        retryable.push(FixtureResult::Retryable { file })
                    }
                    Ok(FixtureResult::Skippable | FixtureResult::Success) => (),
                    Ok(FixtureResult::Unauthorized) => return Err(()),
                    Err(Errored { file, reason }) => {
                        println!("An error occurred in loading fixture {}: {}", file, reason)
                    }
                }

                sleep(std::time::Duration::from_millis(wait)).await
            }

            for to_retry in retryable.iter() {
                if let FixtureResult::Retryable { file } = to_retry {
                    println!("Retrying: {}", file);
                    _ = post_fixture(remote_endpoint, api_key, file, insecure).await;
                }
            }
        }
    }

    Ok(())
}

fn extract_fixtures_from_directory(directory: &String) -> Option<Vec<String>> {
    match scan_directory(directory) {
        Ok(directory_files) => Some(directory_files),
        Err(_) => {
            println!("Couldn't read directory: {}", directory);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::time::timeout;

    static TEST_DIR_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_test_dir(name: &str) -> std::path::PathBuf {
        let id = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "photofinish_test_{}_{}_{}",
            std::process::id(),
            name,
            id
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    async fn spawn_mock_server(response: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        });
        format!("http://{}", addr)
    }

    // Accepts one connection per entry in `responses`, in order, and counts how many
    // connections were actually served — used to assert how many requests `run()` made.
    async fn spawn_multi_response_mock_server(
        responses: Vec<&'static str>,
    ) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        tokio::spawn(async move {
            for response in responses {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = [0u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        (format!("http://{}", addr), counter)
    }

    // scan_directory

    #[test]
    fn scan_directory_lists_only_files_not_subdirectories() {
        let dir = unique_test_dir("scan_files");
        fs::write(dir.join("a.json"), "{}").unwrap();
        fs::write(dir.join("b.json"), "{}").unwrap();
        fs::create_dir(dir.join("nested")).unwrap();

        let mut files = scan_directory(dir.to_str().unwrap()).unwrap();
        files.sort();

        assert_eq!(files.len(), 2);
        assert!(files[0].ends_with("a.json"));
        assert!(files[1].ends_with("b.json"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn scan_directory_errors_on_missing_directory() {
        let dir = std::env::temp_dir().join("photofinish_test_scan_directory_missing");
        assert!(scan_directory(dir.to_str().unwrap()).is_err());
    }

    // extract_fixtures_from_directory

    #[test]
    fn extract_fixtures_from_directory_returns_files_on_success() {
        let dir = unique_test_dir("extract_files");
        fs::write(dir.join("a.json"), "{}").unwrap();

        let files = extract_fixtures_from_directory(&dir.to_str().unwrap().to_string()).unwrap();
        assert_eq!(files.len(), 1);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn extract_fixtures_from_directory_returns_none_for_missing_directory() {
        let missing = std::env::temp_dir()
            .join("photofinish_test_extract_missing")
            .to_str()
            .unwrap()
            .to_string();
        assert!(extract_fixtures_from_directory(&missing).is_none());
    }

    // post_fixture

    #[tokio::test]
    async fn post_fixture_returns_success_on_202() {
        let dir = unique_test_dir("post_202");
        let file = dir.join("fixture.json");
        fs::write(&file, "{}").unwrap();
        let endpoint = spawn_mock_server(
            "HTTP/1.1 202 Accepted\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        )
        .await;

        let result = post_fixture(&endpoint, "api-key", file.to_str().unwrap(), false).await;

        match result {
            Ok(FixtureResult::Success) => (),
            Ok(other) => panic!("expected Success, got {:?}", other),
            Err(_) => panic!("expected Success, got an error"),
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn post_fixture_returns_unauthorized_on_401() {
        let dir = unique_test_dir("post_401");
        let file = dir.join("fixture.json");
        fs::write(&file, "{}").unwrap();
        let endpoint = spawn_mock_server(
            "HTTP/1.1 401 Unauthorized\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        )
        .await;

        let result = post_fixture(&endpoint, "api-key", file.to_str().unwrap(), false).await;

        match result {
            Ok(FixtureResult::Unauthorized) => (),
            Ok(other) => panic!("expected Unauthorized, got {:?}", other),
            Err(_) => panic!("expected Unauthorized, got an error"),
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn post_fixture_returns_retryable_on_400() {
        let dir = unique_test_dir("post_400");
        let file = dir.join("fixture.json");
        fs::write(&file, "{}").unwrap();
        let endpoint = spawn_mock_server(
            "HTTP/1.1 400 Bad Request\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        )
        .await;

        let result = post_fixture(&endpoint, "api-key", file.to_str().unwrap(), false).await;

        match result {
            Ok(FixtureResult::Retryable { file: retried_file }) => {
                assert_eq!(retried_file, file.to_str().unwrap())
            }
            Ok(other) => panic!("expected Retryable, got {:?}", other),
            Err(_) => panic!("expected Retryable, got an error"),
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn post_fixture_returns_skippable_on_unexpected_status() {
        let dir = unique_test_dir("post_500");
        let file = dir.join("fixture.json");
        fs::write(&file, "{}").unwrap();
        let endpoint = spawn_mock_server(
            "HTTP/1.1 500 Internal Server Error\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        )
        .await;

        let result = post_fixture(&endpoint, "api-key", file.to_str().unwrap(), false).await;

        match result {
            Ok(FixtureResult::Skippable) => (),
            Ok(other) => panic!("expected Skippable, got {:?}", other),
            Err(_) => panic!("expected Skippable, got an error"),
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn post_fixture_errors_when_file_is_missing() {
        let missing_file = std::env::temp_dir()
            .join("photofinish_test_post_fixture_missing_file.json")
            .to_str()
            .unwrap()
            .to_string();

        let result = post_fixture("http://127.0.0.1:1", "api-key", &missing_file, false).await;

        match result {
            Err(Errored { file, reason }) => {
                assert_eq!(file, missing_file);
                assert_eq!(reason, "Couldn't read file");
            }
            Ok(_) => panic!("expected an error for a missing file"),
        }
    }

    #[tokio::test]
    async fn post_fixture_errors_when_the_server_is_unreachable() {
        let dir = unique_test_dir("post_unreachable");
        let file = dir.join("fixture.json");
        fs::write(&file, "{}").unwrap();

        // Bind then immediately drop to obtain a port nothing is listening on.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let result = post_fixture(
            &format!("http://{}", addr),
            "api-key",
            file.to_str().unwrap(),
            false,
        )
        .await;

        match result {
            Err(Errored { file: errored_file, .. }) => {
                assert_eq!(errored_file, file.to_str().unwrap())
            }
            Ok(_) => panic!("expected an error for an unreachable server"),
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    // run

    #[tokio::test]
    async fn run_returns_err_for_non_existing_scenario() {
        let result = timeout(
            Duration::from_secs(5),
            run(
                "http://127.0.0.1:1",
                false,
                "api-key",
                "missing-scenario".to_string(),
                vec![],
                0,
            ),
        )
        .await
        .expect("run() timed out");

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn run_posts_all_files_successfully() {
        let dir = unique_test_dir("run_success");
        let file_a = dir.join("a.json");
        let file_b = dir.join("b.json");
        fs::write(&file_a, "{}").unwrap();
        fs::write(&file_b, "{}").unwrap();

        let ok_response = "HTTP/1.1 202 Accepted\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
        let (endpoint, counter) =
            spawn_multi_response_mock_server(vec![ok_response, ok_response]).await;

        let scenario = Scenario {
            label: "my-scenario".to_string(),
            files: vec![
                file_a.to_str().unwrap().to_string(),
                file_b.to_str().unwrap().to_string(),
            ],
            directories: vec![],
        };

        let result = timeout(
            Duration::from_secs(5),
            run(
                &endpoint,
                false,
                "api-key",
                "my-scenario".to_string(),
                vec![scenario],
                0,
            ),
        )
        .await
        .expect("run() timed out");

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn run_stops_early_on_unauthorized() {
        let dir = unique_test_dir("run_unauthorized");
        let file_a = dir.join("a.json");
        let file_b = dir.join("b.json");
        fs::write(&file_a, "{}").unwrap();
        fs::write(&file_b, "{}").unwrap();

        let (endpoint, counter) = spawn_multi_response_mock_server(vec![
            "HTTP/1.1 401 Unauthorized\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        ])
        .await;

        let scenario = Scenario {
            label: "my-scenario".to_string(),
            files: vec![
                file_a.to_str().unwrap().to_string(),
                file_b.to_str().unwrap().to_string(),
            ],
            directories: vec![],
        };

        let result = timeout(
            Duration::from_secs(5),
            run(
                &endpoint,
                false,
                "api-key",
                "my-scenario".to_string(),
                vec![scenario],
                0,
            ),
        )
        .await
        .expect("run() timed out");

        assert!(result.is_err());
        // Only the first file should have been sent — run() must bail out on
        // the first Unauthorized response instead of posting the rest.
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn run_retries_retryable_fixtures() {
        let dir = unique_test_dir("run_retry");
        let file_a = dir.join("a.json");
        fs::write(&file_a, "{}").unwrap();

        let (endpoint, counter) = spawn_multi_response_mock_server(vec![
            "HTTP/1.1 400 Bad Request\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
            "HTTP/1.1 202 Accepted\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        ])
        .await;

        let scenario = Scenario {
            label: "my-scenario".to_string(),
            files: vec![file_a.to_str().unwrap().to_string()],
            directories: vec![],
        };

        let result = timeout(
            Duration::from_secs(5),
            run(
                &endpoint,
                false,
                "api-key",
                "my-scenario".to_string(),
                vec![scenario],
                0,
            ),
        )
        .await
        .expect("run() timed out");

        assert!(result.is_ok());
        // First attempt gets 400 (Retryable), then the retry pass posts it again.
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        fs::remove_dir_all(&dir).unwrap();
    }
}
