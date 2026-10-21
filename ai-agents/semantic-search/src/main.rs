use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct SearchInput {
    query: String,
    limit: Option<i64>,
    search_parts: Option<bool>,
    search_documents: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct PartSearchResult {
    id: Uuid,
    number: String,
    name: String,
    description: Option<String>,
    rank: f32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct DocumentSearchResult {
    id: Uuid,
    name: String,
    format: String,
    description: Option<String>,
    rank: f32,
}

#[derive(Debug, Serialize)]
struct SearchResult {
    parts: Vec<PartSearchResult>,
    documents: Vec<DocumentSearchResult>,
}

async fn search_parts(
    pool: &sqlx::PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<PartSearchResult>, sqlx::Error> {
    // Full-text search по деталям
    let results = sqlx::query_as::<_, PartSearchResult>(
        r#"
        SELECT 
            id,
            number,
            name,
            description,
            ts_rank(search_vector, websearch_to_tsquery('russian', $1)) as rank
        FROM parts
        WHERE search_vector @@ websearch_to_tsquery('russian', $1)
        ORDER BY rank DESC, created_at DESC
        LIMIT $2
        "#,
    )
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

async fn search_documents(
    pool: &sqlx::PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<DocumentSearchResult>, sqlx::Error> {
    // Поиск по документам (по name и description)
    let results = sqlx::query_as::<_, DocumentSearchResult>(
        r#"
        SELECT 
            id,
            name,
            format,
            description,
            ts_rank(
                to_tsvector('russian', COALESCE(name, '') || ' ' || COALESCE(description, '')),
                websearch_to_tsquery('russian', $1)
            ) as rank
        FROM documents
        WHERE to_tsvector('russian', COALESCE(name, '') || ' ' || COALESCE(description, ''))
              @@ websearch_to_tsquery('russian', $1)
        ORDER BY rank DESC, created_at DESC
        LIMIT $2
        "#,
    )
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(results)
}

async fn process_search_task(
    pool: &sqlx::PgPool,
    task_id: Uuid,
    input: SearchInput,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Processing search task {}: query={}", task_id, input.query);

    // Обновить статус на processing
    sqlx::query(
        "UPDATE ai_tasks SET status = 'processing', started_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(task_id)
    .execute(pool)
    .await?;

    let limit = input.limit.unwrap_or(10).clamp(1, 50);
    let should_search_parts = input.search_parts.unwrap_or(true);
    let should_search_documents = input.search_documents.unwrap_or(true);

    // Выполнить поиск
    let parts = if should_search_parts {
        search_parts(pool, &input.query, limit).await?
    } else {
        vec![]
    };

    let documents = if should_search_documents {
        search_documents(pool, &input.query, limit).await?
    } else {
        vec![]
    };

    let result = SearchResult { parts, documents };

    tracing::info!(
        "Search completed: {} parts, {} documents",
        result.parts.len(),
        result.documents.len()
    );

    // Сохранить результат
    sqlx::query(
        r#"
        UPDATE ai_tasks 
        SET status = 'completed', 
            result_data = $1,
            completed_at = NOW(),
            updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(serde_json::to_value(&result)?)
    .bind(task_id)
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "semantic_search=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Semantic Search Agent started, connected to database");

    // TODO: В будущем здесь будет NATS subscriber для получения задач из очереди
    // Пока что будем polling pending задач semantic_search

    loop {
        // Получить pending задачу semantic_search
        let task: Option<(Uuid, serde_json::Value)> = sqlx::query_as(
            r#"
            SELECT id, input_data
            FROM ai_tasks
            WHERE task_type = 'semantic_search' AND status = 'pending'
            ORDER BY priority DESC, created_at
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .fetch_optional(&pool)
        .await?;

        if let Some((task_id, input_data)) = task {
            match serde_json::from_value::<SearchInput>(input_data) {
                Ok(input) => {
                    if let Err(e) = process_search_task(&pool, task_id, input).await {
                        tracing::error!("Failed to process task {}: {}", task_id, e);
                        // Пометить как failed
                        sqlx::query(
                            r#"
                            UPDATE ai_tasks 
                            SET status = 'failed', 
                                error_message = $1,
                                completed_at = NOW(),
                                updated_at = NOW()
                            WHERE id = $2
                            "#,
                        )
                        .bind(e.to_string())
                        .bind(task_id)
                        .execute(&pool)
                        .await?;
                    }
                }
                Err(e) => {
                    tracing::error!("Invalid input data for task {}: {}", task_id, e);
                    sqlx::query(
                        "UPDATE ai_tasks SET status = 'failed', error_message = $1, updated_at = NOW() WHERE id = $2",
                    )
                    .bind(format!("Invalid input: {}", e))
                    .bind(task_id)
                    .execute(&pool)
                    .await?;
                }
            }
        } else {
            // Нет задач, подождать
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
}
