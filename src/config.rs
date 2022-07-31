use std::{fs, io::{self, Read}};

#[derive(Debug)]
pub struct Scenario {
    pub label: String,
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

fn get_config_from_stdin() -> String {
    let mut piped_input = String::new();
    match io::stdin().read_to_string(&mut piped_input) {
        Ok(len) => {
            match len {
                0 => String::new(),
                _ => piped_input
            }
        },
        Err(error) => {
            println!("Error! could not read from stdin the photofinish config file\n: {}", error);
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
                    println!(
                        "Error! Probably .photofinish.toml is missing\n{}",
                        err
                    );
                    String::new()
                },
                _ => piped_config
            }
        }
    }
}

fn extract_array(label: &str, config_table: &toml::Value) -> Vec<String> {
    let default_array = toml::value::Array::new();
    let default_toml_value = &"array = []".parse::<toml::Value>().unwrap()["array"];

    config_table
        .as_table()
        .unwrap()
        .iter()
        .find(|(key, _)| key == &label)
        .and_then(|(_, value)| Some(value))
        .unwrap_or_else(|| &default_toml_value)
        .as_array()
        .unwrap_or_else(|| &default_array)
        .iter()
        .map(|file_path| file_path.as_str().unwrap_or_else(|| "").to_string())
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
