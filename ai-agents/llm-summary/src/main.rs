use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Row;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("LLM Summary Agent started, waiting for tasks...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://schturman:schturman_password@localhost:5434/schturman".to_string());

    let pool = PgPool::connect(&database_url).await?;

    loop {
        match process_next_task(&pool).await {
            Ok(Some(task_id)) => {
                tracing::info!("Processing LLM summary task {}", task_id);
                if let Err(e) = execute_llm_summary(task_id, &pool).await {
                    tracing::error!("Task {} failed: {}", task_id, e);
                    mark_task_failed(task_id, &pool, &e.to_string()).await?;
                }
            }
            Ok(None) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
            Err(e) => {
                tracing::error!("Error polling tasks: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_next_task(pool: &PgPool) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
    let result = sqlx::query(
        r#"
        UPDATE ai_tasks
        SET status = 'processing', updated_at = NOW()
        WHERE id = (
            SELECT id FROM ai_tasks
            WHERE task_type = 'llm_summary' AND status = 'queued'
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|row| row.get(0)))
}

async fn execute_llm_summary(
    task_id: Uuid,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Получить входные данные задачи
    let task = sqlx::query(
        r#"
        SELECT input_data
        FROM ai_tasks
        WHERE id = $1
        "#,
    )
    .bind(task_id)
    .fetch_one(pool)
    .await?;

    let input: serde_json::Value = task.get(0);
    let task_type = input["task_type"]
        .as_str()
        .ok_or("Missing task_type")?;
    let context = input["context"]
        .as_object()
        .ok_or("Missing context")?;

    tracing::info!("Generating LLM summary for task type: {}", task_type);

    // 2. Генерируем summary в зависимости от типа задачи
    let summary = match task_type {
        "eco_summary" => generate_eco_summary(context, pool).await?,
        "bom_comparison" => generate_bom_comparison_summary(context, pool).await?,
        "part_description" => generate_part_description(context, pool).await?,
        "change_recommendation" => generate_change_recommendation(context, pool).await?,
        _ => generate_generic_summary(context).await?,
    };

    // 3. Сохранить результат
    let result = json!({
        "task_type": task_type,
        "summary": summary,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "model": "mock-llm-v1",
        "token_count": summary.len() / 4
    });

    sqlx::query(
        r#"
        UPDATE ai_tasks
        SET status = 'completed', result_data = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(&result)
    .bind(task_id)
    .execute(pool)
    .await?;

    tracing::info!("Task {} completed successfully", task_id);
    Ok(())
}

async fn generate_eco_summary(
    context: &serde_json::Map<String, serde_json::Value>,
    pool: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let eco_id = context["eco_id"]
        .as_str()
        .ok_or("Missing eco_id")?;
    let eco_id = Uuid::parse_str(eco_id)?;

    let eco = sqlx::query(
        r#"
        SELECT number, title, description, reason, status, created_at
        FROM changes
        WHERE id = $1
        "#,
    )
    .bind(eco_id)
    .fetch_one(pool)
    .await?;

    let number: String = eco.get(0);
    let title: String = eco.get(1);
    let description: String = eco.get(2);
    let reason: String = eco.get(3);
    let status: String = eco.get(4);

    // Получить затронутые элементы
    let affected_parts_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM change_affected_parts WHERE change_id = $1
        "#,
    )
    .bind(eco_id)
    .fetch_one(pool)
    .await?;

    let affected_docs_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM change_affected_documents WHERE change_id = $1
        "#,
    )
    .bind(eco_id)
    .fetch_one(pool)
    .await?;

    let affected_count = affected_parts_count + affected_docs_count;

    Ok(format!(
        "ECO {} ({}). Status: {}. Reason: {}. Affects {} items. Description: {}",
        number, title, status, reason, affected_count, description
    ))
}

async fn generate_bom_comparison_summary(
    context: &serde_json::Map<String, serde_json::Value>,
    _pool: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let bom1_id = context["bom1_id"].as_str().ok_or("Missing bom1_id")?;
    let bom2_id = context["bom2_id"].as_str().ok_or("Missing bom2_id")?;
    let added = context["added"].as_array().map(|a| a.len()).unwrap_or(0);
    let removed = context["removed"].as_array().map(|a| a.len()).unwrap_or(0);
    let changed = context["changed"].as_array().map(|a| a.len()).unwrap_or(0);

    Ok(format!(
        "BOM comparison between {} and {}: {} items added, {} items removed, {} items changed. ",
        bom1_id, bom2_id, added, removed, changed
    ))
}

async fn generate_part_description(
    context: &serde_json::Map<String, serde_json::Value>,
    pool: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let part_id = context["part_id"]
        .as_str()
        .ok_or("Missing part_id")?;
    let part_id = Uuid::parse_str(part_id)?;

    let part = sqlx::query(
        r#"
        SELECT number, name, description, part_type, material, status
        FROM parts
        WHERE id = $1
        "#,
    )
    .bind(part_id)
    .fetch_one(pool)
    .await?;

    let number: String = part.get(0);
    let name: String = part.get(1);
    let description: Option<String> = part.get(2);
    let part_type: String = part.get(3);
    let material: Option<String> = part.get(4);
    let status: String = part.get(5);

    Ok(format!(
        "Part {} ({}) is a {} with status {}. {}. Material: {}.",
        number,
        name,
        part_type,
        status,
        description.unwrap_or_else(|| "No description".to_string()),
        material.unwrap_or_else(|| "Not specified".to_string())
    ))
}

async fn generate_change_recommendation(
    context: &serde_json::Map<String, serde_json::Value>,
    _pool: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let change_type = context["change_type"]
        .as_str()
        .unwrap_or("general");
    let risk_level = context["risk_level"]
        .as_str()
        .unwrap_or("medium");

    Ok(format!(
        "Recommended approach for {} change with {} risk: 1. Create ECO with detailed description. 2. Identify all affected assemblies. 3. Obtain approvals from engineering and manufacturing. 4. Update documentation. 5. Notify stakeholders.",
        change_type, risk_level
    ))
}

async fn generate_generic_summary(
    context: &serde_json::Map<String, serde_json::Value>,
) -> Result<String, Box<dyn std::error::Error>> {
    let text = context["text"]
        .as_str()
        .unwrap_or("No input text provided");

    Ok(format!(
        "Summary: {}. Key points: extracted from input text. Word count: {}.",
        &text[..text.len().min(100)],
        text.split_whitespace().count()
    ))
}

async fn mark_task_failed(
    task_id: Uuid,
    pool: &PgPool,
    error: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        r#"
        UPDATE ai_tasks
        SET status = 'failed', result_data = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(json!({"error": error}))
    .bind(task_id)
    .execute(pool)
    .await?;
    Ok(())
}
