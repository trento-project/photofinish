// SPDX-FileCopyrightText: SUSE LLC
// SPDX-License-Identifier: Apache-2.0

extern crate clap;
extern crate exitcode;

use clap::{Arg, Command};

mod config;
mod list;
mod run;

#[tokio::main]
async fn main() {
    let options = Command::new("photofinish")
        .version(clap::crate_version!())
        .subcommand(Command::new("list").about("list available event sets"))
        .subcommand(
            Command::new("run")
                .about("injects a specific set of events")
                .arg(
                    Arg::new("url")
                        .short('u')
                        .long("url")
                        .default_value("http://localhost:8081/api/collect"),
                )
                .arg(
                    Arg::new("insecure")
                        .help("Skip SSL verification")
                        .short('k')
                        .default_missing_value("true")
                        .long("insecure")
                        .default_value("false")
                        .num_args(0..=1)
                        .required(false)
                )
                .arg(
                    Arg::new("SET")
                        .help("name of the events set")
                        .required(true),
                )
                .arg(
                    Arg::new("API_KEY")
                        .help("API key for the remote endpoint")
                        .default_value("")
                        .required(false),
                )
                .arg(
                    Arg::new("wait")
                        .short('w')
                        .help("Wait interval between http requests, in milliseconds")
                        .default_value("0")
                        .required(false),
                ),
        )
        .get_matches();

    let config = config::get_config_file_content();

    let scenarios = config::parse_scenarios(config);

    if options.subcommand_matches("list").is_some() {
        list::show_list(&scenarios);
        std::process::exit(exitcode::OK)
    }

    if let Some(run_options) = options.subcommand_matches("run") {
        let scenario_label = run_options.get_one::<String>("SET").unwrap();
        let endpoint_url = run_options.get_one::<String>("url").unwrap();
        let insecure = run_options.get_one::<String>("insecure").unwrap();
        let wait = run_options.get_one::<String>("wait").unwrap();
        let api_key = run_options.get_one::<String>("API_KEY").unwrap();

        match run::run(
            endpoint_url,
            insecure.parse::<bool>().unwrap(),
            api_key,
            scenario_label.to_string(),
            scenarios,
            wait.parse::<u64>().unwrap(),
        )
        .await
        {
            Ok(()) => std::process::exit(exitcode::OK),
            Err(()) => std::process::exit(1),
        }
    }

    println!("Subcommand not provided. Available subcommands: list|run");
    std::process::exit(exitcode::USAGE)
}
