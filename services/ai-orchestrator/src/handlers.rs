use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AgentInfo, AiTask, CreateTaskRequest, TaskResponse, TaskType};

/// Извлечь user_id из заголовка
pub fn extract_user_id(
    headers: &axum::http::HeaderMap,
) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    let user_id_str = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing x-user-id header"})),
            )
        })?;

    Uuid::parse_str(user_id_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid user_id format"})),
        )
    })
}

/// Создать новую AI-задачу
pub async fn create_task(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id(&headers)?;
    let priority = payload.priority.unwrap_or(5);

    let task_type_str = match payload.task_type {
        TaskType::SemanticSearch => "semantic_search",
        TaskType::CadAnalysis => "cad_analysis",
        TaskType::BomValidation => "bom_validation",
        TaskType::ChangeImpact => "change_impact",
        TaskType::DocumentExtraction => "document_extraction",
        TaskType::LlmSummary => "llm_summary",
    };

    let task = sqlx::query_as::<_, AiTask>(
        r#"
        INSERT INTO ai_tasks (task_type, status, input_data, priority, user_id)
        VALUES ($1, 'pending', $2, $3, $4)
        RETURNING id, task_type, status, input_data, result_data, error_message, 
                  priority, user_id, created_at, started_at, completed_at, updated_at
        "#,
    )
    .bind(task_type_str)
    .bind(&payload.input_data)
    .bind(priority)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to create task: {}", e)})),
        )
    })?;

    // Отправить задачу в NATS queue (заглушка на будущее)
    tracing::info!("Task {} created and queued for processing", task.id);

    Ok((StatusCode::CREATED, Json(TaskResponse::from(task))))
}

/// Получить статус задачи
pub async fn get_task(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id(&headers)?;

    let task = sqlx::query_as::<_, AiTask>(
        r#"
        SELECT id, task_type, status, input_data, result_data, error_message,
               priority, user_id, created_at, started_at, completed_at, updated_at
        FROM ai_tasks
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(task_id)
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Task not found"})),
        )
    })?;

    Ok(Json(TaskResponse::from(task)))
}

/// Параметры фильтрации задач
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<String>,
    pub task_type: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Список задач пользователя
pub async fn list_tasks(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Query(params): Query<ListTasksQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id(&headers)?;
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT id, task_type, status, input_data, result_data, error_message,
               priority, user_id, created_at, started_at, completed_at, updated_at
        FROM ai_tasks
        WHERE user_id = $1
        "#,
    );

    let mut param_count = 1;

    if params.status.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND status = ${}", param_count));
    }

    if params.task_type.is_some() {
        param_count += 1;
        query.push_str(&format!(" AND task_type = ${}", param_count));
    }

    query.push_str(" ORDER BY created_at DESC LIMIT $");
    param_count += 1;
    query.push_str(&param_count.to_string());
    query.push_str(" OFFSET $");
    param_count += 1;
    query.push_str(&param_count.to_string());

    let mut query_builder = sqlx::query_as::<_, AiTask>(&query).bind(user_id);

    if let Some(status) = &params.status {
        query_builder = query_builder.bind(status);
    }

    if let Some(task_type) = &params.task_type {
        query_builder = query_builder.bind(task_type);
    }

    let tasks = query_builder
        .bind(per_page)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ai_tasks WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    Ok(Json(json!({
        "items": tasks.into_iter().map(TaskResponse::from).collect::<Vec<_>>(),
        "total": total.0,
        "page": page,
        "per_page": per_page
    })))
}

/// Отменить задачу
pub async fn cancel_task(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
    Path(task_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = extract_user_id(&headers)?;

    let result = sqlx::query(
        r#"
        UPDATE ai_tasks
        SET status = 'cancelled', updated_at = NOW()
        WHERE id = $1 AND user_id = $2 AND status IN ('pending', 'processing')
        "#,
    )
    .bind(task_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Task not found or cannot be cancelled"})),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Список доступных AI-агентов
pub async fn list_agents() -> impl IntoResponse {
    let agents = vec![
        AgentInfo {
            name: "Semantic Search".to_string(),
            description: "Поиск деталей и документов по смыслу через embeddings".to_string(),
            task_type: TaskType::SemanticSearch,
            enabled: false, // TODO: включить после реализации
        },
        AgentInfo {
            name: "CAD Analysis".to_string(),
            description: "Анализ CAD-файлов: извлечение метаданных, размеров, thumbnail"
                .to_string(),
            task_type: TaskType::CadAnalysis,
            enabled: false,
        },
        AgentInfo {
            name: "BOM Validation".to_string(),
            description: "Проверка BOM на дубликаты, совместимость материалов".to_string(),
            task_type: TaskType::BomValidation,
            enabled: false,
        },
        AgentInfo {
            name: "Change Impact Analysis".to_string(),
            description: "Анализ влияния ECO на BOM, документы, процессы".to_string(),
            task_type: TaskType::ChangeImpact,
            enabled: false,
        },
        AgentInfo {
            name: "Document Extraction".to_string(),
            description: "Извлечение текста из PDF, OCR, метаданные документов".to_string(),
            task_type: TaskType::DocumentExtraction,
            enabled: false,
        },
        AgentInfo {
            name: "LLM Summary".to_string(),
            description: "Генерация описаний, саммари, переводы через LLM".to_string(),
            task_type: TaskType::LlmSummary,
            enabled: false,
        },
    ];

    Json(agents)
}

/// Health check
pub async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "ok", "service": "ai-orchestrator"}))
}
