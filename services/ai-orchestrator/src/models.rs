use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Типы AI-задач
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum TaskType {
    #[serde(rename = "semantic_search")]
    SemanticSearch,
    #[serde(rename = "cad_analysis")]
    CadAnalysis,
    #[serde(rename = "bom_validation")]
    BomValidation,
    #[serde(rename = "change_impact")]
    ChangeImpact,
    #[serde(rename = "document_extraction")]
    DocumentExtraction,
    #[serde(rename = "llm_summary")]
    LlmSummary,
}

/// Статус задачи
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

/// AI-задача
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiTask {
    pub id: Uuid,
    pub task_type: String,
    pub status: String,
    pub input_data: serde_json::Value,
    pub result_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub priority: i32,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Запрос на создание задачи
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub task_type: TaskType,
    pub input_data: serde_json::Value,
    pub priority: Option<i32>,
}

/// Ответ со статусом задачи
#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub task_type: String,
    pub status: String,
    pub result_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Список доступных AI-агентов
#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub enabled: bool,
}

impl From<AiTask> for TaskResponse {
    fn from(task: AiTask) -> Self {
        Self {
            id: task.id,
            task_type: task.task_type,
            status: task.status,
            result_data: task.result_data,
            error_message: task.error_message,
            created_at: task.created_at,
            completed_at: task.completed_at,
        }
    }
}
