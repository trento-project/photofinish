use actix_web::{get, web, HttpResponse};

use crate::web::ApiState;

#[get("/api/scenarios")]
pub async fn list_scenarios(data: web::Data<ApiState>) -> HttpResponse {
    let scenarios = data.scenarios.clone();
    HttpResponse::Ok().json(scenarios)
}
