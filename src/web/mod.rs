use crate::runner::config::Scenario;

pub mod handlers;

pub struct ApiState {
    pub scenarios: Vec<Scenario>
}