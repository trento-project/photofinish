use anyhow::{Context, Result, bail};
use std::{
    fs,
    io::{self, Read},
};

static PHOTOFINISH_FILE: &str = ".photofinish.toml";

#[derive(Debug)]
pub struct Scenario {
    pub label: String,
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

fn get_config_from_stdin() -> Result<String> {
    let mut piped_input = String::new();
    let read_size = io::stdin()
        .read_to_string(&mut piped_input)
        .with_context(|| "could not read from stdin the photofinish config file")?;
    if read_size == 0 {
        Ok("".to_owned())
    } else {
        Ok(piped_input)
    }
}

pub fn get_config_file_content() -> Result<String> {    
    match fs::read_to_string(PHOTOFINISH_FILE) {
        Ok(toml_content) => Ok(toml_content),
        Err(err) => {
            let piped_config = get_config_from_stdin()?;

            if piped_config.len() == 0 {
                bail!(".photofinish.toml file is missing, error: {}", err);
            }
            Ok(piped_config)
        }
    }
}

fn extract_array(label: &str, config_table: &toml::Value) -> Result<Vec<String>> {
    let default_array = toml::value::Array::new();
    let default_toml_value = &"array = []".parse::<toml::Value>()?["array"];

    let extracted_values = config_table
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
        .collect();
    Ok(extracted_values)
}

pub fn parse_scenarios(config: String) -> Result<Vec<Scenario>> {
    let toml_config: toml::value::Table = toml::from_str(&config)?;
    let scenarios: Result<Vec<Scenario>> = toml_config
        .into_iter()
        .map(|(key, value)| {
            let scenario_files: Vec<String> = extract_array("files", &value)?;
            let scenario_directories: Vec<String> = extract_array("directories", &value)?;

            Ok(Scenario {
                label: key.to_string(),
                files: scenario_files,
                directories: scenario_directories,
            })
        })
        .collect();
    scenarios
}
