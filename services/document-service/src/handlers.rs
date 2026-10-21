use axum::{
    body::Bytes,
    extract::{Path, Query, Multipart, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sha2::Digest;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadDocumentRequest {
    pub name: String,
    pub description: Option<String>,
    pub format: String,
    pub part_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListDocumentsQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub part_id: Option<Uuid>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub format: String,
    pub status: String,
    pub version: String,
    pub file_size: i64,
    pub file_hash: String,
    pub storage_path: String,
    pub thumbnail_url: Option<String>,
    pub part_id: Option<Uuid>,
    pub checked_out_by: Option<Uuid>,
    pub checked_out_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

fn extract_user_id(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()))
}

pub async fn upload_document(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = extract_user_id(&headers)?;

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut format: Option<String> = None;
    let mut part_id: Option<Uuid> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        
        match field_name.as_str() {
            "name" => {
                let data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Read error: {}", e))
                })?;
                name = Some(String::from_utf8_lossy(&data).to_string());
            },
            "description" => {
                let data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Read error: {}", e))
                })?;
                description = Some(String::from_utf8_lossy(&data).to_string());
            },
            "format" => {
                let data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Read error: {}", e))
                })?;
                format = Some(String::from_utf8_lossy(&data).to_string());
            },
            "part_id" => {
                let data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("Read error: {}", e))
                })?;
                let s = String::from_utf8_lossy(&data).to_string();
                if !s.is_empty() {
                    part_id = Uuid::parse_str(&s).ok();
                }
            },
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, format!("File read error: {}", e))
                })?.to_vec());
            },
            _ => {}
        }
    }

    let name = name.ok_or((StatusCode::BAD_REQUEST, "Missing name".to_string()))?;
    let format = format.ok_or((StatusCode::BAD_REQUEST, "Missing format".to_string()))?;
    let file_data = file_data.ok_or((StatusCode::BAD_REQUEST, "Missing file".to_string()))?;

    // Compute file hash
    let file_hash = format!("{:x}", sha2::Sha256::digest(&file_data));
    let file_size = file_data.len() as i64;

    // Save file to storage
    let doc_id = Uuid::new_v4();
    let storage_path = format!("documents/{}/{}", doc_id, filename.unwrap_or_else(|| "file".to_string()));
    
    // For now, just save to local filesystem (in production would use MinIO)
    let storage_dir = std::env::var("DOCUMENT_STORAGE_PATH")
        .unwrap_or_else(|_| "/tmp/schturman-documents".to_string());
    
    std::fs::create_dir_all(format!("{}/documents/{}", storage_dir, doc_id))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage error: {}", e)))?;
    
    std::fs::write(format!("{}/{}", storage_dir, storage_path), &file_data)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("File write error: {}", e)))?;

    // Insert into database
    let doc = sqlx::query_as::<_, DocumentRow>(
        "INSERT INTO documents (id, name, description, format, status, version, file_size, file_hash, storage_path, part_id, created_by, updated_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
         RETURNING *"
    )
    .bind(doc_id)
    .bind(&name)
    .bind(&description)
    .bind(&format)
    .bind("draft")
    .bind("1.0")
    .bind(file_size)
    .bind(&file_hash)
    .bind(&storage_path)
    .bind(part_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    Ok(Json(json!(doc)))
}

pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let mut sql = "SELECT * FROM documents WHERE 1=1".to_string();
    let mut bindings = Vec::new();
    let mut param_count = 1;

    if let Some(part_id) = query.part_id {
        sql.push_str(&format!(" AND part_id = ${}", param_count));
        bindings.push(part_id.to_string());
        param_count += 1;
    }

    if let Some(format) = query.format {
        sql.push_str(&format!(" AND format = ${}", param_count));
        bindings.push(format);
        param_count += 1;
    }

    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_count, param_count + 1));

    let mut query_builder = sqlx::query_as::<_, DocumentRow>(&sql);
    for binding in bindings {
        if let Ok(uuid) = Uuid::parse_str(&binding) {
            query_builder = query_builder.bind(uuid);
        } else {
            query_builder = query_builder.bind(binding);
        }
    }
    query_builder = query_builder.bind(page_size).bind(offset);

    let documents = query_builder
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    Ok(Json(json!({
        "documents": documents,
        "page": page,
        "page_size": page_size
    })))
}

pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentRow>, (StatusCode, String)> {
    let doc = sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Document not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)),
        })?;

    Ok(Json(doc))
}

pub async fn get_document_versions(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Simplified - in full version would query document_versions table
    Ok(Json(json!({"versions": []})))
}

pub async fn download_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let doc = sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Document not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)),
        })?;

    let storage_dir = std::env::var("DOCUMENT_STORAGE_PATH")
        .unwrap_or_else(|_| "/tmp/schturman-documents".to_string());
    
    let file_path = format!("{}/{}", storage_dir, doc.storage_path);
    
    let file_data = std::fs::read(&file_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("File read error: {}", e)))?;

    let filename = format!("attachment; filename=\"{}\"", doc.name);
    let headers = [
        ("Content-Type", "application/octet-stream".to_string()),
        ("Content-Disposition", filename),
    ];

    Ok((headers, file_data))
}
