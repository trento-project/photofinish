use std::{fs, io::{self, Read}};

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

