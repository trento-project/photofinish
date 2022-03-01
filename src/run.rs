use crate::config::Scenario;
use std::fs;

async fn post_fixture(remote_endpoint: &str, file: &str, http_client: &reqwest::Client) -> () {
    let canonical_path = fs::canonicalize(file).unwrap_or_default();
    match fs::read_to_string(canonical_path) {
        Ok(file_content) => {
            let response = http_client
                .post(remote_endpoint)
                .body(file_content)
                .header("Content-Type", "application/json")
                .send()
                .await;
            match response {
                Ok(_) => {
                    println!("Successfully POSTed file: {}", file);
                }
                Err(_) => println!("Error while POSTing fixture: {}", file),
            }
        }
        Err(_) => println!("Couldn't read file: {}", file),
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
            for file in scenario.files.iter() {
                post_fixture(remote_endpoint, file, &http_client).await
            }
            for directory in scenario.directories.iter() {
                match scan_directory(directory) {
                    Ok(directory_files) => {
                        for file in directory_files.iter() {
                            post_fixture(remote_endpoint, file, &http_client).await
                        }
                    }
                    Err(_) => println!("Couldn't read directory: {}", directory),
                }
            }
        }
    }
}
