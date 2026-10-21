use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Row;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Document Extractor Agent started, waiting for tasks...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://schturman:schturman_password@localhost:5434/schturman".to_string());

    let pool = PgPool::connect(&database_url).await?;

    loop {
        match process_next_task(&pool).await {
            Ok(Some(task_id)) => {
                tracing::info!("Processing document extraction task {}", task_id);
                if let Err(e) = execute_document_extraction(task_id, &pool).await {
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
            WHERE task_type = 'document_extraction' AND status = 'queued'
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

async fn execute_document_extraction(
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
    let document_id = input["document_id"]
        .as_str()
        .ok_or("Missing document_id")?;
    let document_id = Uuid::parse_str(document_id)?;

    // 2. Получить документ
    let doc = sqlx::query(
        r#"
        SELECT d.name, d.format, d.storage_path, d.version, d.status, p.number as part_number, p.name as part_name
        FROM documents d
        LEFT JOIN parts p ON p.id = d.part_id
        WHERE d.id = $1
        "#,
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await?
    .ok_or("Document not found")?;

    let doc_name: String = doc.get(0);
    let format: String = doc.get(1);
    let storage_path: String = doc.get(2);
    let version: String = doc.get(3);
    let status: String = doc.get(4);
    let part_number: Option<String> = doc.get(5);
    let part_name: Option<String> = doc.get(6);

    tracing::info!(
        "Extracting data from document {} (format: {}, version: {})",
        doc_name,
        format,
        version
    );

    // 3. Эмулируем извлечение данных (в реальности: OCR, парсинг PDF, извлечение текста)
    let extracted_data = extract_document_data(&format, &storage_path).await?;

    // 4. Формируем результат
    let result = json!({
        "document_id": document_id.to_string(),
        "document_name": doc_name,
        "format": format,
        "version": version,
        "status": status,
        "part_number": part_number,
        "part_name": part_name,
        "extracted_data": extracted_data,
        "extraction_timestamp": chrono::Utc::now().to_rfc3339()
    });

    // 5. Сохранить результат
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

async fn extract_document_data(
    format: &str,
    _storage_path: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Mock-извлечение данных в зависимости от формата
    match format.to_lowercase().as_str() {
        "pdf" => Ok(json!({
            "text_length": 1234,
            "page_count": 5,
            "extracted_text": "This is a sample extracted text from PDF...",
            "metadata": {
                "author": "John Doe",
                "creation_date": "2024-01-15",
                "title": "Technical Drawing"
            },
            "tables": [
                {
                    "page": 2,
                    "rows": 10,
                    "columns": 4,
                    "data": [["Part", "Qty", "Material", "Notes"], ["M6 Bolt", "10", "SS304", "Standard"]]
                }
            ]
        })),
        "step" | "iges" | "stl" => Ok(json!({
            "cad_metadata": {
                "assembly_name": "Main Assembly",
                "part_count": 15,
                "features": ["Holes", "Fillets", "Extrusions"],
                "bounding_box": {
                    "min": [0.0, 0.0, 0.0],
                    "max": [100.0, 50.0, 30.0]
                }
            },
            "extracted_properties": {
                "material": "Stainless Steel",
                "finish": "Polished",
                "tolerance": "±0.1mm"
            }
        })),
        "docx" | "doc" => Ok(json!({
            "text_length": 5678,
            "paragraph_count": 45,
            "extracted_text": "This is a sample extracted text from Word document...",
            "metadata": {
                "author": "Jane Smith",
                "last_modified": "2024-03-20",
                "title": "Design Requirements"
            },
            "sections": ["Introduction", "Requirements", "Specifications", "Appendix"]
        })),
        "xlsx" | "xls" => Ok(json!({
            "sheet_count": 3,
            "total_rows": 150,
            "sheets": [
                {
                    "name": "BOM",
                    "rows": 50,
                    "columns": 6,
                    "sample_data": [["Part Number", "Description", "Qty"], ["BLT-M6-20", "Bolt M6x20", "10"]]
                },
                {
                    "name": "Costs",
                    "rows": 50,
                    "columns": 4
                }
            ]
        })),
        "png" | "jpg" | "jpeg" => Ok(json!({
            "image_metadata": {
                "width": 1920,
                "height": 1080,
                "format": format,
                "color_space": "RGB"
            },
            "ocr_text": "Sample OCR extracted text: Part Number: M6-BOLT-SS",
            "detected_objects": ["bolt", "washer", "nut"],
            "confidence": 0.85
        })),
        _ => Ok(json!({
            "format": format,
            "extraction_method": "generic",
            "file_size_bytes": 12345,
            "note": "Format not specifically supported, using generic extraction"
        })),
    }
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
