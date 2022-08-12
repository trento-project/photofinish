use crate::runner::config::Scenario;

use serde::{Serialize,Deserialize};

pub mod handlers;

pub struct ApiState {
    pub scenarios: Vec<Scenario>
}

#[derive(Deserialize)]
pub struct ExecutionRequest {
    pub scenario: String,
    pub api_key: String
}

#[derive(Serialize)]

pub struct ExecutionResponse {
    pub scenario: String,
    pub execution_time: u64,
    pub errored: bool,
}