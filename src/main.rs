extern crate clap;

use std::process::exit;

use clap::{App, Arg};
use actix_web::{App as ActixApp, HttpServer, web as ActixWeb};
use runner::config::Scenario;
use crate::runner::config as RunnerConfig;

mod config;
mod web;
mod list;
mod runner;

async fn create_server(scenarios: &Vec<Scenario>) -> std::io::Result<()> {
    let app_scenarios = scenarios.clone();
    HttpServer::new(move|| {
        ActixApp::new()
            .app_data(ActixWeb::Data::new(
                web::ApiState{
                    scenarios: app_scenarios.to_vec(),
                }
            ))
            .service(web::handlers::list_scenarios)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[tokio::main]
async fn main() {
    let options = App::new("photofinish")
        .version(clap::crate_version!())
        .subcommand(App::new("server")).about("start photofinish api")
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

    let scenarios = RunnerConfig::parse_scenarios(config);

    if options.subcommand_matches("server").is_some() {
        match create_server(&scenarios).await {
            Ok(_) => {
                println!("Server, shutdown")
            },
            Err(reason) => {
                println!("error during server execution {}", reason);

                exit(1);
            }
        }
    }

    if options.subcommand_matches("list").is_some() {
        list::show_list(&scenarios);
    }

    if let Some(run_options) = options.subcommand_matches("run") {
        let scenario_label = run_options.value_of("SET").unwrap();
        let endpoint_url = run_options.value_of("url").unwrap();
        let api_key = run_options.value_of("API_KEY").unwrap();
        runner::run(endpoint_url, api_key, scenario_label.to_string(), scenarios).await;
    }
}
