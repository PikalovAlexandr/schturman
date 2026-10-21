use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisInput {
    document_id: Uuid,
    storage_path: String,
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalysisResult {
    dimensions: Option<Dimensions>,
    thumbnail_url: Option<String>,
    part_type: Option<String>,
    material: Option<String>,
    mass: Option<f64>,
    surface_area: Option<f64>,
    volume: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Dimensions {
    length: f64,
    width: f64,
    height: f64,
    unit: String,
}

#[derive(Debug, sqlx::FromRow)]
struct AiTask {
    id: Uuid,
    task_type: String,
    status: String,
    input_data: serde_json::Value,
    priority: i32,
    created_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://schturman:schturman_password@localhost:5434/schturman".to_string());

    let pool = PgPool::connect(&database_url).await?;
    info!("CAD Parser Agent started, connected to database");

    // Основной цикл: проверяем новые задачи каждые 2 секунды
    loop {
        match fetch_pending_task(&pool).await {
            Ok(Some(task)) => {
                info!("Processing CAD analysis task {}", task.id);
                
                // Парсим input_data
                let input: AnalysisInput = match serde_json::from_value(task.input_data.clone()) {
                    Ok(i) => i,
                    Err(e) => {
                        error!("Failed to parse input_data: {}", e);
                        mark_task_failed(&pool, task.id, &format!("Invalid input: {}", e)).await?;
                        continue;
                    }
                };

                // Выполняем анализ
                match analyze_cad_file(&input).await {
                    Ok(result) => {
                        let result_json = serde_json::to_value(result)?;
                        mark_task_completed(&pool, task.id, result_json).await?;
                        info!("Task {} completed successfully", task.id);
                    }
                    Err(e) => {
                        error!("Failed to analyze CAD file: {}", e);
                        mark_task_failed(&pool, task.id, &e.to_string()).await?;
                    }
                }
            }
            Ok(None) => {
                // Нет задач, ждем
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                error!("Failed to fetch pending task: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn fetch_pending_task(pool: &PgPool) -> Result<Option<AiTask>, sqlx::Error> {
    // Получаем первую задачу со статусом queued и task_type = cad_analysis
    let task = sqlx::query_as::<_, AiTask>(
        r#"
        UPDATE ai_tasks
        SET status = 'processing', updated_at = NOW()
        WHERE id = (
            SELECT id FROM ai_tasks
            WHERE status = 'queued' AND task_type = 'cad_analysis'
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, task_type, status, input_data, priority, created_at
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

async fn mark_task_completed(
    pool: &PgPool,
    task_id: Uuid,
    result_data: serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE ai_tasks
        SET status = 'completed', result_data = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(result_data)
    .bind(task_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn mark_task_failed(
    pool: &PgPool,
    task_id: Uuid,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    let error_json = serde_json::json!({ "error": error_message });
    sqlx::query(
        r#"
        UPDATE ai_tasks
        SET status = 'failed', result_data = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(error_json)
    .bind(task_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn analyze_cad_file(input: &AnalysisInput) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    info!("Analyzing CAD file: {} (format: {})", input.storage_path, input.format);

    // Проверяем существование файла
    let file_path = format!("/tmp/schturman-documents/{}", input.storage_path);
    if !tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
        return Err(format!("File not found: {}", file_path).into());
    }

    // Mock-анализ в зависимости от формата
    let result = match input.format.to_lowercase().as_str() {
        "step" | "stp" => {
            // Для STEP-файлов извлекаем больше данных
            AnalysisResult {
                dimensions: Some(Dimensions {
                    length: 100.0,
                    width: 50.0,
                    height: 30.0,
                    unit: "mm".to_string(),
                }),
                thumbnail_url: Some(format!("thumbnails/{}.png", input.document_id)),
                part_type: Some("mechanical_part".to_string()),
                material: Some("steel".to_string()),
                mass: Some(0.245), // кг
                surface_area: Some(16000.0), // мм²
                volume: Some(150000.0), // мм³
            }
        }
        "iges" | "igs" => {
            // Для IGES ограниченные данные
            AnalysisResult {
                dimensions: Some(Dimensions {
                    length: 80.0,
                    width: 40.0,
                    height: 25.0,
                    unit: "mm".to_string(),
                }),
                thumbnail_url: Some(format!("thumbnails/{}.png", input.document_id)),
                part_type: None,
                material: None,
                mass: None,
                surface_area: None,
                volume: None,
            }
        }
        _ => {
            return Err(format!("Unsupported CAD format: {}", input.format).into());
        }
    };

    // Генерация thumbnail
    generate_thumbnail(&input.document_id).await?;

    Ok(result)
}

async fn generate_thumbnail(document_id: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
    // Создаем директорию для thumbnails
    tokio::fs::create_dir_all("/tmp/schturman-documents/thumbnails").await?;

    // Генерируем простое placeholder-изображение 256x256
    let img = image::RgbaImage::from_fn(256, 256, |x, y| {
        // Простой градиент
        let intensity = ((x + y) / 2) as u8;
        image::Rgba([intensity, intensity, intensity, 255])
    });

    let thumbnail_path = format!("/tmp/schturman-documents/thumbnails/{}.png", document_id);
    img.save(&thumbnail_path)?;

    info!("Thumbnail generated: {}", thumbnail_path);
    Ok(())
}
