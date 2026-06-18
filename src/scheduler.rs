use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{Product, ScheduleTask, ScheduleType};

#[derive(Clone)]
pub struct AppState {
    pub scheduler: JobScheduler,
    pub tasks: Arc<Mutex<HashMap<Uuid, ScheduleTask>>>,
    pub products: Arc<Mutex<HashMap<Uuid, Product>>>,
    pub job_ids: Arc<Mutex<HashMap<Uuid, uuid::Uuid>>>,
}

impl AppState {
    pub async fn new() -> AppResult<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| AppError::SchedulerError(e.to_string()))?;

        Ok(Self {
            scheduler,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            products: Arc::new(Mutex::new(HashMap::new())),
            job_ids: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn init_sample_data(&self) {
        let mut products = self.products.lock().await;
        let now = Utc::now();

        let sample_products = vec![
            Product {
                id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
                name: "iPhone 15 Pro".to_string(),
                is_published: false,
                created_at: now,
                updated_at: now,
            },
            Product {
                id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
                name: "MacBook Pro 14".to_string(),
                is_published: true,
                created_at: now,
                updated_at: now,
            },
        ];

        for p in sample_products {
            products.insert(p.id, p);
        }

        info!("Sample products initialized");
    }

    pub async fn start(&self) -> AppResult<()> {
        self.scheduler
            .start()
            .await
            .map_err(|e| AppError::SchedulerError(e.to_string()))?;
        info!("Scheduler started");
        Ok(())
    }

    pub async fn create_schedule_task(
        &self,
        product_id: Uuid,
        schedule_type: ScheduleType,
        execute_at: DateTime<Utc>,
    ) -> AppResult<ScheduleTask> {
        if execute_at <= Utc::now() {
            return Err(AppError::InvalidScheduleTime(
                "Scheduled time must be in the future".to_string(),
            ));
        }

        let products = self.products.lock().await;
        if !products.contains_key(&product_id) {
            return Err(AppError::ProductNotFound(product_id.to_string()));
        }
        drop(products);

        let cancelled = self
            .cancel_pending_tasks(product_id, schedule_type)
            .await?;
        if cancelled > 0 {
            info!(
                "Replaced {} existing pending {:?} task(s) for product {}",
                cancelled, schedule_type, product_id
            );
        }

        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let task = ScheduleTask {
            id: task_id,
            product_id,
            schedule_type,
            execute_at,
            is_executed: false,
            created_at: now,
        };

        let cron_expr = datetime_to_cron(&execute_at);
        info!(
            "Creating schedule task: type={:?}, product_id={}, execute_at={}, cron={}",
            schedule_type, product_id, execute_at, cron_expr
        );

        let state_clone = self.clone();
        let job = Job::new_one_shot_async(&cron_expr, move |_uuid, _l| {
            let state = state_clone.clone();
            Box::pin(async move {
                if let Err(e) = execute_task(&state, task_id).await {
                    error!("Task execution failed: {}", e);
                }
            })
        })
        .map_err(|e| AppError::SchedulerError(format!("Invalid cron expression: {}", e)))?;

        let job_id = self
            .scheduler
            .add(job)
            .await
            .map_err(|e| AppError::SchedulerError(e.to_string()))?;

        let mut tasks = self.tasks.lock().await;
        tasks.insert(task_id, task.clone());

        let mut job_ids = self.job_ids.lock().await;
        job_ids.insert(task_id, job_id);

        Ok(task)
    }

    async fn cancel_pending_tasks(
        &self,
        product_id: Uuid,
        schedule_type: ScheduleType,
    ) -> AppResult<usize> {
        let to_cancel: Vec<Uuid> = {
            let tasks = self.tasks.lock().await;
            tasks
                .values()
                .filter(|t| {
                    t.product_id == product_id
                        && t.schedule_type == schedule_type
                        && !t.is_executed
                })
                .map(|t| t.id)
                .collect()
        };

        if to_cancel.is_empty() {
            return Ok(0);
        }

        let count = to_cancel.len();
        for task_id in &to_cancel {
            if let Some(job_id) = self.job_ids.lock().await.remove(task_id) {
                if let Err(e) = self.scheduler.remove(&job_id).await {
                    error!(
                        "Failed to remove scheduled job {} for task {}: {}",
                        job_id, task_id, e
                    );
                }
            }
            self.tasks.lock().await.remove(task_id);
            info!(
                "Cancelled pending {:?} task {} for product {}",
                schedule_type, task_id, product_id
            );
        }

        Ok(count)
    }

    pub async fn list_tasks(&self) -> Vec<ScheduleTask> {
        let tasks = self.tasks.lock().await;
        tasks.values().cloned().collect()
    }

    pub async fn get_task(&self, task_id: Uuid) -> Option<ScheduleTask> {
        let tasks = self.tasks.lock().await;
        tasks.get(&task_id).cloned()
    }

    pub async fn list_products(&self) -> Vec<Product> {
        let products = self.products.lock().await;
        products.values().cloned().collect()
    }
}

fn datetime_to_cron(dt: &DateTime<Utc>) -> String {
    format!(
        "{} {} {} {} {} *",
        dt.second(),
        dt.minute(),
        dt.hour(),
        dt.day(),
        dt.month()
    )
}

async fn execute_task(state: &AppState, task_id: Uuid) -> AppResult<()> {
    info!("Executing task: {}", task_id);

    let mut tasks = state.tasks.lock().await;
    let task = tasks
        .get_mut(&task_id)
        .ok_or_else(|| AppError::TaskNotFound(task_id.to_string()))?;

    if task.is_executed {
        info!("Task {} already executed, skipping", task_id);
        return Ok(());
    }

    let product_id = task.product_id;
    let schedule_type = task.schedule_type;

    let mut products = state.products.lock().await;
    let product = products
        .get_mut(&product_id)
        .ok_or_else(|| AppError::ProductNotFound(product_id.to_string()))?;

    match schedule_type {
        ScheduleType::Publish => {
            product.is_published = true;
            info!("Product {} published", product_id);
        }
        ScheduleType::Unpublish => {
            product.is_published = false;
            info!("Product {} unpublished", product_id);
        }
    }
    product.updated_at = Utc::now();

    task.is_executed = true;

    info!("Task {} completed successfully", task_id);
    Ok(())
}
