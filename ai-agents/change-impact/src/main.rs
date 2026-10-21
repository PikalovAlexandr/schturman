use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://schturman:schturman_password@localhost:5434/schturman".to_string()
        });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("[change-impact] Connected to database");
    println!("[change-impact] Polling for change_impact tasks...");

    loop {
        match process_next_task(&pool).await {
            Ok(true) => {
                println!("[change-impact] Task processed");
            }
            Ok(false) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => {
                eprintln!("[change-impact] Error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_next_task(pool: &PgPool) -> Result<bool, Box<dyn Error>> {
    let task = sqlx::query(
        r#"
        UPDATE ai_tasks
        SET status = 'processing'
        WHERE id = (
            SELECT id FROM ai_tasks
            WHERE task_type = 'change_impact'
            AND status = 'queued'
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, input_data
        "#,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = task {
        let task_id: Uuid = row.get(0);
        let input_data: Value = row.get(1);

        println!("[change-impact] Processing task {}", task_id);

        let change_id = input_data["change_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or("Missing or invalid change_id")?;

        match analyze_change_impact(pool, change_id).await {
            Ok(result) => {
                sqlx::query(
                    r#"
                    UPDATE ai_tasks
                    SET status = 'completed', result_data = $1, completed_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(&result)
                .bind(task_id)
                .execute(pool)
                .await?;

                println!("[change-impact] Task {} completed", task_id);
                Ok(true)
            }
            Err(e) => {
                sqlx::query(
                    r#"
                    UPDATE ai_tasks
                    SET status = 'failed', error_message = $1, completed_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(e.to_string())
                .bind(task_id)
                .execute(pool)
                .await?;

                eprintln!("[change-impact] Task {} failed: {}", task_id, e);
                Ok(true)
            }
        }
    } else {
        Ok(false)
    }
}

/// Анализ влияния изменения
async fn analyze_change_impact(
    pool: &PgPool,
    change_id: Uuid,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // 1. Получить информацию об изменении
    let change = sqlx::query(
        r#"
        SELECT title, description, change_type, status, priority
        FROM changes
        WHERE id = $1
        "#,
    )
    .bind(change_id)
    .fetch_optional(pool)
    .await?
    .ok_or("Change not found")?;

    let title: String = change.get(0);
    let description: Option<String> = change.get(1);
    let change_type: String = change.get(2);
    let status: String = change.get(3);
    let priority: String = change.get(4);

    // 2. Получить затронутые части
    let affected_parts = sqlx::query(
        r#"
        SELECT p.id, p.number, p.name, p.status
        FROM change_affected_parts cap
        JOIN parts p ON p.id = cap.part_id
        WHERE cap.change_id = $1
        "#,
    )
    .bind(change_id)
    .fetch_all(pool)
    .await?;

    let mut impacted_parts = Vec::new();
    let mut part_ids = Vec::new();

    for row in &affected_parts {
        let part_id: Uuid = row.get(0);
        let part_number: String = row.get(1);
        let part_name: String = row.get(2);
        let part_status: String = row.get(3);

        part_ids.push(part_id);
        impacted_parts.push(json!({
            "part_id": part_id.to_string(),
            "part_number": part_number,
            "part_name": part_name,
            "status": part_status
        }));
    }

    // 3. Найти все BOM, где используются затронутые части (where-used)
    let where_used = if !part_ids.is_empty() {
        sqlx::query(
            r#"
            SELECT DISTINCT b.id, b.assembly_part_id, p.number, p.name, b.version
            FROM bom_items bi
            JOIN boms b ON b.id = bi.bom_id
            JOIN parts p ON p.id = b.assembly_part_id
            WHERE bi.part_id = ANY($1)
            "#,
        )
        .bind(&part_ids)
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };

    let mut affected_assemblies = Vec::new();
    let mut assembly_ids: HashSet<Uuid> = HashSet::new();

    for row in &where_used {
        let bom_id: Uuid = row.get(0);
        let assembly_id: Uuid = row.get(1);
        let assembly_number: String = row.get(2);
        let assembly_name: String = row.get(3);
        let version: String = row.get(4);

        assembly_ids.insert(assembly_id);
        affected_assemblies.push(json!({
            "bom_id": bom_id.to_string(),
            "assembly_id": assembly_id.to_string(),
            "assembly_number": assembly_number,
            "assembly_name": assembly_name,
            "version": version
        }));
    }

    // 4. Получить затронутые документы
    let affected_documents = sqlx::query(
        r#"
        SELECT d.id, d.name, d.version, d.status
        FROM change_affected_documents cad
        JOIN documents d ON d.id = cad.document_id
        WHERE cad.change_id = $1
        "#,
    )
    .bind(change_id)
    .fetch_all(pool)
    .await?;

    let mut impacted_documents = Vec::new();
    for row in &affected_documents {
        let doc_id: Uuid = row.get(0);
        let doc_name: String = row.get(1);
        let doc_version: String = row.get(2);
        let doc_status: String = row.get(3);

        impacted_documents.push(json!({
            "document_id": doc_id.to_string(),
            "name": doc_name,
            "version": doc_version,
            "status": doc_status
        }));
    }

    // 5. Найти связанные изменения (через те же части)
    let related_changes = if !part_ids.is_empty() {
        sqlx::query(
            r#"
            SELECT DISTINCT c.id, c.title, c.status, c.created_at
            FROM change_affected_parts cap
            JOIN changes c ON c.id = cap.change_id
            WHERE cap.part_id = ANY($1)
            AND c.id != $2
            ORDER BY c.created_at DESC
            LIMIT 5
            "#,
        )
        .bind(&part_ids)
        .bind(change_id)
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };

    let mut related = Vec::new();
    for row in &related_changes {
        let rel_id: Uuid = row.get(0);
        let rel_title: String = row.get(1);
        let rel_status: String = row.get(2);

        related.push(json!({
            "change_id": rel_id.to_string(),
            "title": rel_title,
            "status": rel_status
        }));
    }

    // 6. Расчет метрик риска
    let risk_score = calculate_risk_score(
        &change_type,
        &priority,
        affected_parts.len(),
        affected_assemblies.len(),
        affected_documents.len(),
    );

    // 7. Генерация рекомендаций
    let recommendations = generate_recommendations(
        &change_type,
        &priority,
        affected_parts.len(),
        affected_assemblies.len(),
    );

    Ok(json!({
        "change_id": change_id.to_string(),
        "title": title,
        "description": description,
        "change_type": change_type,
        "status": status,
        "priority": priority,
        "impact_summary": {
            "total_parts": affected_parts.len(),
            "total_assemblies": affected_assemblies.len(),
            "total_documents": affected_documents.len(),
            "related_changes": related_changes.len(),
            "risk_score": risk_score,
            "risk_level": get_risk_level(risk_score)
        },
        "impacted_parts": impacted_parts,
        "affected_assemblies": affected_assemblies,
        "impacted_documents": impacted_documents,
        "related_changes": related,
        "recommendations": recommendations
    }))
}

/// Расчет риск-балла (0-100)
fn calculate_risk_score(
    change_type: &str,
    priority: &str,
    parts_count: usize,
    assemblies_count: usize,
    documents_count: usize,
) -> i32 {
    let mut score = 0;

    // Тип изменения
    score += match change_type {
        "major" => 30,
        "minor" => 15,
        "patch" => 5,
        _ => 10,
    };

    // Приоритет
    score += match priority {
        "critical" => 25,
        "high" => 15,
        "medium" => 10,
        "low" => 5,
        _ => 10,
    };

    // Количество затронутых элементов
    score += (parts_count as i32).min(20);
    score += (assemblies_count as i32 * 2).min(15);
    score += (documents_count as i32).min(10);

    score.min(100)
}

fn get_risk_level(score: i32) -> &'static str {
    match score {
        0..=25 => "low",
        26..=50 => "medium",
        51..=75 => "high",
        _ => "critical",
    }
}

fn generate_recommendations(
    change_type: &str,
    priority: &str,
    parts_count: usize,
    assemblies_count: usize,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if change_type == "major" || priority == "critical" {
        recommendations.push("Requires senior management approval".to_string());
        recommendations.push("Schedule impact review meeting with all stakeholders".to_string());
    }

    if assemblies_count > 3 {
        recommendations
            .push("High assembly impact - coordinate with manufacturing team".to_string());
        recommendations.push("Update production schedules and inventory planning".to_string());
    }

    if parts_count > 5 {
        recommendations.push("Multiple parts affected - consider phased implementation".to_string());
        recommendations.push("Validate BOM integrity after change implementation".to_string());
    }

    if priority == "low" && parts_count == 0 {
        recommendations.push("Low impact change - can be fast-tracked".to_string());
    }

    if assemblies_count > 0 {
        recommendations.push("Update assembly documentation and work instructions".to_string());
    }

    if recommendations.is_empty() {
        recommendations.push("Standard approval process recommended".to_string());
    }

    recommendations
}
