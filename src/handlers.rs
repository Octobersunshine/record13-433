use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::errors::AppResult;
use crate::models::{
    CreateScheduleRequest, CreateScheduleResponse, Product, ScheduleTask, TaskListResponse,
    TriggerTaskResponse,
};
use crate::scheduler::AppState;

pub async fn create_schedule(
    State(state): State<AppState>,
    Json(req): Json<CreateScheduleRequest>,
) -> AppResult<(StatusCode, Json<CreateScheduleResponse>)> {
    let task = state
        .create_schedule_task(req.product_id, req.schedule_type, req.execute_at)
        .await?;

    let response = CreateScheduleResponse {
        task_id: task.id,
        product_id: task.product_id,
        schedule_type: task.schedule_type,
        execute_at: task.execute_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_tasks(
    State(state): State<AppState>,
) -> AppResult<Json<TaskListResponse>> {
    let tasks = state.list_tasks().await;
    Ok(Json(TaskListResponse { tasks }))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<ScheduleTask>> {
    let task = state
        .get_task(task_id)
        .await
        .ok_or_else(|| crate::errors::AppError::TaskNotFound(task_id.to_string()))?;
    Ok(Json(task))
}

pub async fn list_products(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<Product>>> {
    let products = state.list_products().await;
    Ok(Json(products))
}

pub async fn trigger_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<TriggerTaskResponse>> {
    let task = state.execute_task_now(task_id).await?;

    let response = TriggerTaskResponse {
        task_id: task.id,
        product_id: task.product_id,
        schedule_type: task.schedule_type,
        executed: task.is_executed,
        message: format!(
            "Task {:?} for product {} executed successfully",
            task.schedule_type, task.product_id
        ),
    };

    Ok(Json(response))
}

pub async fn health_check() -> &'static str {
    "OK"
}
