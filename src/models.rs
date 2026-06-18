use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleType {
    Publish,
    Unpublish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub is_published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTask {
    pub id: Uuid,
    pub product_id: Uuid,
    pub schedule_type: ScheduleType,
    pub execute_at: DateTime<Utc>,
    pub is_executed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub product_id: Uuid,
    pub schedule_type: ScheduleType,
    pub execute_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreateScheduleResponse {
    pub task_id: Uuid,
    pub product_id: Uuid,
    pub schedule_type: ScheduleType,
    pub execute_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub tasks: Vec<ScheduleTask>,
}

#[derive(Debug, Serialize)]
pub struct TriggerTaskResponse {
    pub task_id: Uuid,
    pub product_id: Uuid,
    pub schedule_type: ScheduleType,
    pub executed: bool,
    pub message: String,
}
