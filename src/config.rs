// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs,
    io::{self, Read},
};

#[derive(Debug)]
pub struct Scenario {
    pub label: String,
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

fn get_config_from_stdin() -> String {
    let mut piped_input = String::new();
    match io::stdin().read_to_string(&mut piped_input) {
        Ok(len) => match len {
            0 => String::new(),
            _ => piped_input,
        },
        Err(error) => {
            println!(
                "Error! could not read from stdin the photofinish config file\n: {}",
                error
            );
            String::new()
        }
    }
}

pub fn get_config_file_content() -> String {
    match fs::read_to_string(".photofinish.toml") {
        Ok(toml_content) => toml_content,
        Err(err) => {
            let piped_config = get_config_from_stdin();

            match piped_config.as_str() {
                "" => {
                    println!("Error! Probably .photofinish.toml is missing\n{}", err);
                    String::new()
                }
                _ => piped_config,
            }
        }
    }
}

fn extract_array(label: &str, config_table: &toml::Value) -> Vec<String> {
    let default_array = toml::value::Array::new();
    let default_toml_value = toml::Value::Array(default_array.clone());

    config_table
        .as_table()
        .unwrap()
        .iter()
        .find(|(key, _)| key == &label)
        .map(|(_, value)| value)
        .unwrap_or(&default_toml_value)
        .as_array()
        .unwrap_or(&default_array)
        .iter()
        .map(|file_path| file_path.as_str().unwrap_or("").to_string())
        .collect()
}

pub fn parse_scenarios(config: String) -> Vec<Scenario> {
    let toml_config: toml::value::Table = toml::from_str(&config).unwrap();
    toml_config
        .iter()
        .map(|(key, value)| {
            let scenario_files: Vec<String> = extract_array("files", value);
            let scenario_directories: Vec<String> = extract_array("directories", value);

            Scenario {
                label: key.to_string(),
                files: scenario_files,
                directories: scenario_directories,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scenario_with_both_files_and_directories() {
        let scenarios = parse_scenarios(
            r#"
            [my-scenario]
            files = ["a.json", "b.json"]
            directories = ["some/dir"]
            "#
            .to_string(),
        );

        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].label, "my-scenario");
        assert_eq!(scenarios[0].files, vec!["a.json", "b.json"]);
        assert_eq!(scenarios[0].directories, vec!["some/dir"]);
    }

    #[test]
    fn defaults_missing_files_key_to_empty_vec() {
        let scenarios = parse_scenarios(
            r#"
            [fixtures]
            directories = ["./fixtures/"]
            "#
            .to_string(),
        );

        assert_eq!(scenarios.len(), 1);
        assert!(scenarios[0].files.is_empty());
        assert_eq!(scenarios[0].directories, vec!["./fixtures/"]);
    }

    #[test]
    fn defaults_missing_directories_key_to_empty_vec() {
        let scenarios = parse_scenarios(
            r#"
            [second-test-scenario]
            files = ["third file", "fourth-file"]
            "#
            .to_string(),
        );

        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].files, vec!["third file", "fourth-file"]);
        assert!(scenarios[0].directories.is_empty());
    }

    #[test]
    fn extract_array_returns_the_matching_array() {
        let table: toml::Value = toml::from_str(r#"files = ["a", "b"]"#).unwrap();
        assert_eq!(extract_array("files", &table), vec!["a", "b"]);
    }

    #[test]
    fn extract_array_defaults_to_empty_when_label_missing() {
        let table: toml::Value = toml::from_str(r#"directories = ["some/dir"]"#).unwrap();
        assert!(extract_array("files", &table).is_empty());
    }

    #[test]
    fn extract_array_defaults_to_empty_when_value_is_not_an_array() {
        let table: toml::Value = toml::from_str(r#"files = "not-an-array""#).unwrap();
        assert!(extract_array("files", &table).is_empty());
    }

    #[test]
    fn extract_array_maps_non_string_entries_to_empty_string() {
        let table: toml::Value = toml::from_str(r#"files = [1, "b"]"#).unwrap();
        assert_eq!(extract_array("files", &table), vec!["", "b"]);
    }

    #[test]
    fn get_config_file_content_reads_the_photofinish_toml_in_the_cwd() {
        let expected = fs::read_to_string(".photofinish.toml").unwrap();
        assert_eq!(get_config_file_content(), expected);
    }
}
