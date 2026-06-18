use axum::{routing, Router};

use crate::handlers::*;
use crate::scheduler::AppState;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", routing::get(health_check))
        .route("/api/schedules", routing::post(create_schedule))
        .route("/api/schedules", routing::get(list_tasks))
        .route("/api/schedules/:task_id", routing::get(get_task))
        .route("/api/products", routing::get(list_products))
        .with_state(state)
}
