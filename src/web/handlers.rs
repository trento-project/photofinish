use actix_web::{get, post, web, HttpResponse};

use crate::web::{ApiState, ExecutionRequest, ExecutionResponse};

#[get("/api/scenarios")]
pub async fn list_scenarios(data: web::Data<ApiState>) -> HttpResponse {
    let scenarios = data.scenarios.clone();
    HttpResponse::Ok().json(scenarios)
}

#[post("/api/execution")]
pub async fn execute_scenario(execution_info: web::Json<ExecutionRequest>,_data: web::Data<ApiState>) -> HttpResponse {
    let execution_response = ExecutionResponse{
        scenario: execution_info.scenario.clone(),
        execution_time: 20,
        errored: false
    };
    HttpResponse::Ok().json(execution_response)
}