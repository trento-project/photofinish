use crate::config::Scenario;
use reqwest::StatusCode;
use std::fs;

#[derive(Debug)]
enum FixtureResult {
    Success,
    Retryable { file: String },
    Skippable,
}

struct Errored {
    file: String,
    reason: String,
}

async fn post_fixture(
    remote_endpoint: &str,
    file: &str,
    http_client: &reqwest::Client,
) -> Result<FixtureResult, Errored> {
    let canonical_path = fs::canonicalize(file).unwrap_or_default();
    let processed_fixture = file.to_string();
    match fs::read_to_string(canonical_path) {
        Ok(file_content) => {
            let response = http_client
                .post(remote_endpoint)
                .body(file_content)
                .header("Content-Type", "application/json")
                .send()
                .await;
            match response {
                Ok(res) => match res.status() {
                    StatusCode::ACCEPTED => {
                        println!("Successfully POSTed file: {}", file);
                        Ok(FixtureResult::Success)
                    }
                    StatusCode::BAD_REQUEST => Ok(FixtureResult::Retryable {
                        file: processed_fixture,
                    }),
                    _ => Ok(FixtureResult::Skippable),
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
                true => e.path().to_str().and_then(|s| Some(s.to_string())),
                false => None,
            })
        })
        .collect();
    Ok(files_list)
}

pub async fn run(
    remote_endpoint: &str,
    scenario_label: String,
    scenarios: Vec<Scenario>,
    http_client: reqwest::Client,
) -> () {
    let selected_scenario = scenarios
        .iter()
        .find(|current_scenario| current_scenario.label == scenario_label);

    match selected_scenario {
        None => println!("Non-existing scenario!"),
        Some(scenario) => {
            let fixtures_in_directories: Vec<String> = scenario
                .directories
                .iter()
                .filter_map(extract_fixtures_from_directory)
                .flatten()
                .collect();

            let full_scenario = [&scenario.files[..], &fixtures_in_directories[..]].concat();

            let mut retryable: Vec<FixtureResult> = vec![];

            for file in full_scenario.iter() {
                let execution_result = post_fixture(remote_endpoint, file, &http_client).await;
                match execution_result {
                    Ok(FixtureResult::Retryable { file }) => {
                        retryable.push(FixtureResult::Retryable { file })
                    }
                    Ok(FixtureResult::Skippable | FixtureResult::Success) => (),
                    Err(Errored { file, reason }) => {
                        println!("An error occurred in loading fixture {}: {}", file, reason)
                    }
                }
            }

            for to_retry in retryable.iter() {
                match to_retry {
                    FixtureResult::Retryable { file } => {
                        println!("Retrying: {}", file);
                        _ = post_fixture(remote_endpoint, file, &http_client).await;
                    }
                    _ => (),
                }
            }
        }
    }
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
