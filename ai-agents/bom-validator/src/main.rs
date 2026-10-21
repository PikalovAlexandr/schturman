use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("bom_validator=debug,info")
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://schturman:schturman_password@localhost:5434/schturman".to_string());

    let pool = PgPool::connect(&database_url).await?;
    tracing::info!("BOM Validator started, waiting for tasks...");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Найти задачи типа bom_validation со статусом queued
        let task_opt = sqlx::query(
            r#"
            SELECT id, input_data
            FROM ai_tasks
            WHERE task_type = 'bom_validation' AND status = 'queued'
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .fetch_optional(&pool)
        .await?;

        let Some(task) = task_opt else {
            continue;
        };

        let task_id: Uuid = task.get(0);
        let input_data: serde_json::Value = task.get(1);

        tracing::info!("Processing BOM validation task {}", task_id);

        // Обновить статус на processing
        sqlx::query(
            r#"
            UPDATE ai_tasks
            SET status = 'processing', started_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(task_id)
        .execute(&pool)
        .await?;

        // Извлечь bom_id
        let bom_id = input_data["bom_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok());

        let result = match bom_id {
            Some(bom_id) => validate_bom(&pool, bom_id).await,
            None => Err("Missing bom_id in input_data".into()),
        };

        match result {
            Ok(report) => {
                sqlx::query(
                    r#"
                    UPDATE ai_tasks
                    SET status = 'completed', result_data = $1, completed_at = $2
                    WHERE id = $3
                    "#,
                )
                .bind(report)
                .bind(Utc::now())
                .bind(task_id)
                .execute(&pool)
                .await?;
                tracing::info!("Task {} completed successfully", task_id);
            }
            Err(e) => {
                sqlx::query(
                    r#"
                    UPDATE ai_tasks
                    SET status = 'failed', error_message = $1, completed_at = $2
                    WHERE id = $3
                    "#,
                )
                .bind(e.to_string())
                .bind(Utc::now())
                .bind(task_id)
                .execute(&pool)
                .await?;
                tracing::error!("Task {} failed: {}", task_id, e);
            }
        }
    }
}

/// Валидация BOM
async fn validate_bom(
    pool: &PgPool,
    bom_id: Uuid,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Получить все элементы BOM
    let items = sqlx::query(
        r#"
        SELECT part_id, quantity, position
        FROM bom_items
        WHERE bom_id = $1
        "#,
    )
    .bind(bom_id)
    .fetch_all(pool)
    .await?;

    let mut issues = Vec::new();
    let mut part_count: HashMap<Uuid, i32> = HashMap::new();
    let mut positions: HashSet<String> = HashSet::new();
    let mut duplicate_positions = Vec::new();

    for item in &items {
        let part_id: Uuid = item.get(0);
        let quantity: f64 = item.get(1);
        let position: String = item.get(2);

        // Проверка 1: количество должно быть > 0 (уже есть CHECK constraint, но проверим на всякий случай)
        if quantity <= 0.0 {
            issues.push(json!({
                "type": "invalid_quantity",
                "part_id": part_id.to_string(),
                "quantity": quantity,
                "position": position,
                "message": "Quantity must be greater than 0"
            }));
        }

        // Проверка 2: дубликаты position (должен быть UNIQUE constraint, но проверим)
        if !positions.insert(position.clone()) {
            duplicate_positions.push(position.clone());
        }

        // Подсчет деталей
        *part_count.entry(part_id).or_insert(0) += 1;
    }

    // Проверка 3: дублирующиеся детали
    for (part_id, count) in part_count {
        if count > 1 {
            issues.push(json!({
                "type": "duplicate_part",
                "part_id": part_id.to_string(),
                "occurrences": count,
                "message": format!("Part appears {} times in BOM", count)
            }));
        }
    }

    // Проверка 4: дублирующиеся positions
    for pos in duplicate_positions {
        issues.push(json!({
            "type": "duplicate_position",
            "position": pos,
            "message": "Position used multiple times"
        }));
    }

    // Проверка 5: проверить существование деталей (FK уже есть, но проверим для отчета)
    let part_ids: Vec<Uuid> = items.iter().map(|r| r.get(0)).collect();
    let existing_parts = sqlx::query(
        r#"
        SELECT id FROM parts WHERE id = ANY($1)
        "#,
    )
    .bind(&part_ids)
    .fetch_all(pool)
    .await?;

    let existing_ids: HashSet<Uuid> = existing_parts.iter().map(|r| r.get(0)).collect();
    for part_id in part_ids {
        if !existing_ids.contains(&part_id) {
            issues.push(json!({
                "type": "missing_part",
                "part_id": part_id.to_string(),
                "message": "Referenced part does not exist"
            }));
        }
    }

    let total_items = items.len();
    let issues_count = issues.len();

    Ok(json!({
        "bom_id": bom_id.to_string(),
        "total_items": total_items,
        "issues_found": issues_count,
        "valid": issues.is_empty(),
        "issues": issues
    }))
}
