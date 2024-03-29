extern crate clap;
extern crate exitcode;

use clap::{App, Arg};

mod config;
mod list;
mod run;

#[tokio::main]
async fn main() {
    let options = App::new("photofinish")
        .version(clap::crate_version!())
        .subcommand(App::new("list").about("list available event sets"))
        .subcommand(
            App::new("run")
                .about("injects a specific set of events")
                .arg(
                    Arg::new("url")
                        .short('u')
                        .long("url")
                        .default_value("http://localhost:8081/api/collect"),
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
        let scenario_label = run_options.value_of("SET").unwrap();
        let endpoint_url = run_options.value_of("url").unwrap();
        let api_key = run_options.value_of("API_KEY").unwrap();
        match run::run(endpoint_url, api_key, scenario_label.to_string(), scenarios).await {
            Ok(()) => std::process::exit(exitcode::OK),
            Err(()) => std::process::exit(1)
        }
    }

    println!("Subcommand not provided. Available subcommands: list|run");
    std::process::exit(exitcode::USAGE)
}
