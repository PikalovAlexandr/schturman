use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePartRequest {
    pub number: String,
    pub name: String,
    pub description: Option<String>,
    pub part_type: String,
    pub status: String,
    pub version: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePartRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub part_type: Option<String>,
    pub status: Option<String>,
    pub version: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPartsQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub status: Option<String>,
    pub part_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchPartsQuery {
    pub query: String,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PartRow {
    pub id: Uuid,
    pub number: String,
    pub name: String,
    pub description: Option<String>,
    pub part_type: String,
    pub status: String,
    pub version: String,
    pub revision: i32,
    pub attributes: serde_json::Value,
    pub owner_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Uuid,
}

/// Создание новой детали
pub async fn create_part(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreatePartRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = extract_user_id_from_headers(&headers)
        .ok_or((StatusCode::UNAUTHORIZED, "Missing or invalid authorization".to_string()))?;

    let part_id = Uuid::new_v4();

    let result = sqlx::query_as::<_, PartRow>(
        r#"
        INSERT INTO parts (
            id, number, name, description, part_type, status, version,
            attributes, owner_id, updated_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(part_id)
    .bind(&req.number)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.part_type)
    .bind(&req.status)
    .bind(req.version.as_ref().unwrap_or(&"1.0".to_string()))
    .bind(req.attributes.as_ref().unwrap_or(&serde_json::json!({})))
    .bind(user_id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    Ok((StatusCode::CREATED, Json(result)))
}

/// Получение детали по ID
pub async fn get_part(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let part = sqlx::query_as::<_, PartRow>(
        "SELECT * FROM parts WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    match part {
        Some(p) => Ok(Json(p)),
        None => Err((StatusCode::NOT_FOUND, "Part not found".to_string())),
    }
}

/// Обновление детали
pub async fn update_part(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<UpdatePartRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = extract_user_id_from_headers(&headers)
        .ok_or((StatusCode::UNAUTHORIZED, "Missing or invalid authorization".to_string()))?;

    // Динамический UPDATE
    let mut updates = Vec::new();
    let mut bind_idx = 1;

    if req.name.is_some() { updates.push(format!("name = ${}", bind_idx)); bind_idx += 1; }
    if req.description.is_some() { updates.push(format!("description = ${}", bind_idx)); bind_idx += 1; }
    if req.part_type.is_some() { updates.push(format!("part_type = ${}", bind_idx)); bind_idx += 1; }
    if req.status.is_some() { updates.push(format!("status = ${}", bind_idx)); bind_idx += 1; }
    if req.version.is_some() { updates.push(format!("version = ${}", bind_idx)); bind_idx += 1; }
    if req.attributes.is_some() { updates.push(format!("attributes = ${}", bind_idx)); bind_idx += 1; }

    if updates.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No fields to update".to_string()));
    }

    updates.push(format!("updated_by = ${}", bind_idx));
    bind_idx += 1;
    updates.push("updated_at = NOW()".to_string());

    let query_str = format!(
        "UPDATE parts SET {} WHERE id = ${} RETURNING *",
        updates.join(", "),
        bind_idx
    );

    let mut query = sqlx::query_as::<_, PartRow>(&query_str);

    if let Some(v) = &req.name { query = query.bind(v); }
    if let Some(v) = &req.description { query = query.bind(v); }
    if let Some(v) = &req.part_type { query = query.bind(v); }
    if let Some(v) = &req.status { query = query.bind(v); }
    if let Some(v) = &req.version { query = query.bind(v); }
    if let Some(v) = &req.attributes { query = query.bind(v); }
    query = query.bind(user_id);
    query = query.bind(id);

    let result = query.fetch_optional(&state.pool).await.map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    match result {
        Some(p) => Ok(Json(p)),
        None => Err((StatusCode::NOT_FOUND, "Part not found".to_string())),
    }
}

/// Удаление детали
pub async fn delete_part(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM parts WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Part not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Список деталей с фильтрацией и пагинацией
pub async fn list_parts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListPartsQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    let mut where_clause = String::from("WHERE 1=1");
    let mut binds: Vec<String> = Vec::new();
    let mut bind_idx = 1;

    if let Some(status) = &params.status {
        where_clause.push_str(&format!(" AND status = ${}", bind_idx));
        bind_idx += 1;
        binds.push(status.clone());
    }
    if let Some(part_type) = &params.part_type {
        where_clause.push_str(&format!(" AND part_type = ${}", bind_idx));
        bind_idx += 1;
        binds.push(part_type.clone());
    }

    let query_str = format!(
        "SELECT * FROM parts {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause, bind_idx, bind_idx + 1
    );

    let mut query = sqlx::query_as::<_, PartRow>(&query_str);
    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(page_size).bind(offset);

    let parts = query.fetch_all(&state.pool).await.map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let count_query_str = format!("SELECT COUNT(*) as count FROM parts {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);
    for b in &binds {
        count_query = count_query.bind(b);
    }
    let total = count_query.fetch_one(&state.pool).await.map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    Ok(Json(json!({
        "parts": parts,
        "page": page,
        "page_size": page_size,
        "total": total,
    })))
}

/// Поиск деталей (полнотекстовый поиск)
pub async fn search_parts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchPartsQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    let parts = sqlx::query_as::<_, PartRow>(
        r#"
        SELECT * FROM parts
        WHERE to_tsvector('russian', coalesce(name, '') || ' ' || coalesce(description, '') || ' ' || coalesce(number, ''))
              @@ plainto_tsquery('russian', $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(&params.query)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    Ok(Json(json!({
        "parts": parts,
        "query": params.query,
        "page": page,
        "page_size": page_size,
    })))
}

/// Извлечение user_id из JWT токена в заголовке Authorization
fn extract_user_id_from_headers(headers: &HeaderMap) -> Option<Uuid> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;
    let token = auth_header.strip_prefix("Bearer ")?;

    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

    #[derive(Debug, serde::Deserialize)]
    struct Claims {
        sub: String,
    }

    let secret = std::env::var("JWT_SECRET").ok()?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ).ok()?;

    Uuid::parse_str(&token_data.claims.sub).ok()
}
