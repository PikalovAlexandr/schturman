use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting BOM Validator Agent...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://schturman:schturman_password@localhost:5434/schturman".to_string());

    let pool = PgPool::connect(&database_url).await?;

    // Polling loop
    loop {
        match poll_tasks(&pool).await {
            Ok(processed) => {
                if processed > 0 {
                    println!("Processed {} bom_validation tasks", processed);
                }
            }
            Err(e) => eprintln!("Error polling tasks: {}", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

async fn poll_tasks(pool: &PgPool) -> Result<u32, Box<dyn std::error::Error>> {
    let tasks = sqlx::query(
        r#"
        SELECT id, input_data
        FROM ai_tasks
        WHERE task_type = 'bom_validation'
        AND status = 'queued'
        ORDER BY priority DESC, created_at ASC
        LIMIT 5
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut processed = 0;
    for task_row in tasks {
        let task_id: Uuid = task_row.get(0);
        let input_data: serde_json::Value = task_row.get(1);

        // Mark as processing
        sqlx::query(
            r#"
            UPDATE ai_tasks
            SET status = 'processing', updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .execute(pool)
        .await?;

        // Process task
        match process_bom_validation(pool, &input_data).await {
            Ok(result) => {
                sqlx::query(
                    r#"
                    UPDATE ai_tasks
                    SET status = 'completed', result_data = $2, completed_at = now(), updated_at = now()
                    WHERE id = $1
                    "#,
                )
                .bind(task_id)
                .bind(result)
                .execute(pool)
                .await?;
                processed += 1;
            }
            Err(e) => {
                sqlx::query(
                    r#"
                    UPDATE ai_tasks
                    SET status = 'failed', error_message = $2, completed_at = now(), updated_at = now()
                    WHERE id = $1
                    "#,
                )
                .bind(task_id)
                .bind(e.to_string())
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(processed)
}

async fn process_bom_validation(
    pool: &PgPool,
    input_data: &serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let bom_id = input_data["bom_id"]
        .as_str()
        .ok_or("Missing bom_id")?;
    let bom_id = Uuid::parse_str(bom_id)?;

    // 1. Получить информацию о BOM
    let bom = sqlx::query(
        r#"
        SELECT assembly_part_id, version, status
        FROM boms
        WHERE id = $1
        "#,
    )
    .bind(bom_id)
    .fetch_optional(pool)
    .await?
    .ok_or("BOM not found")?;

    let assembly_part_id: Uuid = bom.get(0);
    let version: String = bom.get(1);
    let status: String = bom.get(2);

    // 2. Получить все элементы BOM
    let items = sqlx::query(
        r#"
        SELECT bi.id, bi.part_id, bi.quantity, p.number, p.name, p.status, p.part_type
        FROM bom_items bi
        JOIN parts p ON p.id = bi.part_id
        WHERE bi.bom_id = $1
        "#,
    )
    .bind(bom_id)
    .fetch_all(pool)
    .await?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Статистика
    let mut part_count_map: HashMap<Uuid, i32> = HashMap::new();
    let mut status_counts: HashMap<String, i32> = HashMap::new();
    let mut type_counts: HashMap<String, i32> = HashMap::new();

    for row in &items {
        let item_id: Uuid = row.get(0);
        let part_id: Uuid = row.get(1);
        let quantity: f64 = row.get(2);
        let part_number: String = row.get(3);
        let part_name: String = row.get(4);
        let part_status: String = row.get(5);
        let part_type: Option<String> = row.get(6);

        // Подсчет дубликатов
        *part_count_map.entry(part_id).or_insert(0) += 1;

        // Подсчет по статусам
        *status_counts.entry(part_status.clone()).or_insert(0) += 1;

        // Подсчет по типам
        if let Some(ref t) = part_type {
            *type_counts.entry(t.clone()).or_insert(0) += 1;
        }

        // Проверка 1: Устаревшие детали
        if part_status == "obsolete" {
            errors.push(json!({
                "severity": "error",
                "code": "OBSOLETE_PART",
                "message": format!("Part {} ({}) is obsolete", part_number, part_name),
                "item_id": item_id.to_string(),
                "part_id": part_id.to_string()
            }));
        }

        // Проверка 2: Черновики в производственных BOM
        if part_status == "draft" && status == "released" {
            warnings.push(json!({
                "severity": "warning",
                "code": "DRAFT_IN_RELEASED_BOM",
                "message": format!("Part {} ({}) is still in draft", part_number, part_name),
                "item_id": item_id.to_string(),
                "part_id": part_id.to_string()
            }));
        }

        // Проверка 3: Нулевое количество
        if quantity <= 0.0 {
            errors.push(json!({
                "severity": "error",
                "code": "INVALID_QUANTITY",
                "message": format!("Part {} has invalid quantity: {}", part_number, quantity),
                "item_id": item_id.to_string(),
                "part_id": part_id.to_string()
            }));
        }

        // Проверка 4: Большое количество (возможная ошибка ввода)
        if quantity > 1000.0 {
            warnings.push(json!({
                "severity": "warning",
                "code": "HIGH_QUANTITY",
                "message": format!("Part {} has unusually high quantity: {}", part_number, quantity),
                "item_id": item_id.to_string(),
                "part_id": part_id.to_string()
            }));
        }
    }

    // Проверка 5: Дубликаты
    for (part_id, count) in &part_count_map {
        if *count > 1 {
            // Получить информацию о детали
            let part = sqlx::query(
                "SELECT number, name FROM parts WHERE id = $1",
            )
            .bind(part_id)
            .fetch_one(pool)
            .await?;

            let part_number: String = part.get(0);
            let part_name: String = part.get(1);

            errors.push(json!({
                "severity": "error",
                "code": "DUPLICATE_PART",
                "message": format!("Part {} ({}) appears {} times in BOM", part_number, part_name, count),
                "part_id": part_id.to_string()
            }));
        }
    }

    // Проверка 6: Пустой BOM
    if items.is_empty() {
        warnings.push(json!({
            "severity": "warning",
            "code": "EMPTY_BOM",
            "message": "BOM has no items"
        }));
    }

    // Проверка 7: Поиск стандартных деталей (крепеж)
    // Предложить замену на стандартные из каталога
    for row in &items {
        let part_type: Option<String> = row.get(6);
        let part_number: String = row.get(3);
        let part_name: String = row.get(4);

        if let Some(ref t) = part_type {
            if t == "fastener" || part_name.to_lowercase().contains("bolt") || part_name.to_lowercase().contains("screw") {
                // Поиск похожих стандартных деталей
                let similar = sqlx::query(
                    r#"
                    SELECT number, name
                    FROM parts
                    WHERE part_type = 'standard'
                    AND status = 'released'
                    AND (name ILIKE $1 OR number ILIKE $1)
                    LIMIT 3
                    "#,
                )
                .bind(format!("%{}%", part_number.split('-').next().unwrap_or("")))
                .fetch_all(pool)
                .await?;

                if !similar.is_empty() {
                    let mut alternatives = Vec::new();
                    for s in similar {
                        let std_number: String = s.get(0);
                        let std_name: String = s.get(1);
                        alternatives.push(json!({
                            "number": std_number,
                            "name": std_name
                        }));
                    }

                    suggestions.push(json!({
                        "code": "STANDARD_ALTERNATIVE",
                        "message": format!("Consider using standard part instead of {}", part_number),
                        "part_number": part_number,
                        "alternatives": alternatives
                    }));
                }
            }
        }
    }

    // Проверка 8: Совместимость материалов (простая эвристика)
    // Предупреждаем о смешивании металлов
    let mut has_steel = false;
    let mut has_aluminum = false;
    let mut has_brass = false;

    for row in &items {
        let part_name: String = row.get(4);
        let name_lower = part_name.to_lowercase();

        if name_lower.contains("steel") || name_lower.contains("нержавеющая") {
            has_steel = true;
        }
        if name_lower.contains("aluminum") || name_lower.contains("алюминий") {
            has_aluminum = true;
        }
        if name_lower.contains("brass") || name_lower.contains("латунь") {
            has_brass = true;
        }
    }

    if (has_steel && has_aluminum) || (has_steel && has_brass) || (has_aluminum && has_brass) {
        warnings.push(json!({
            "severity": "warning",
            "code": "MATERIAL_COMPATIBILITY",
            "message": "BOM contains parts with potentially incompatible materials (galvanic corrosion risk)"
        }));
    }

    // Сборка результата
    let is_valid = errors.is_empty();
    let error_count = errors.len();
    let warning_count = warnings.len();

    let result = json!({
        "bom_id": bom_id.to_string(),
        "version": version,
        "status": status,
        "is_valid": is_valid,
        "validation_summary": {
            "total_items": items.len(),
            "error_count": error_count,
            "warning_count": warning_count,
            "suggestion_count": suggestions.len()
        },
        "errors": errors,
        "warnings": warnings,
        "suggestions": suggestions,
        "statistics": {
            "by_status": status_counts,
            "by_type": type_counts,
            "unique_parts": part_count_map.len()
        }
    });

    Ok(result)
}
