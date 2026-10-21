use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    headers::{authorization::Bearer, Authorization, HeaderMap},
    TypedHeader,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::AppState;

// Модели данных
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bom {
    pub id: Uuid,
    pub assembly_part_id: Uuid,
    pub version: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BomItem {
    pub id: Uuid,
    pub bom_id: Uuid,
    pub part_id: Uuid,
    pub position: String,
    pub quantity: f64,
    pub unit: String,
    pub note: Option<String>,
    pub level: i32,
    pub parent_item_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBomRequest {
    pub assembly_part_id: Uuid,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBomRequest {
    pub version: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddBomItemRequest {
    pub part_id: Uuid,
    pub position: String,
    pub quantity: f64,
    pub unit: String,
    pub note: Option<String>,
    pub level: Option<i32>,
    pub parent_item_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListBomsQuery {
    pub assembly_part_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// Создание BOM
pub async fn create_bom(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateBomRequest>,
) -> Result<Json<Bom>, StatusCode> {
    let user_id = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let id = Uuid::new_v4();
    let version = req.version.unwrap_or_else(|| "1.0".to_string());
    let now = Utc::now();

    let bom = sqlx::query_as!(
        Bom,
        r#"
        INSERT INTO boms (id, assembly_part_id, version, status, created_at, created_by, updated_at, updated_by)
        VALUES ($1, $2, $3, 'draft', $4, $5, $6, $7)
        RETURNING id, assembly_part_id, version, status, created_at, created_by, updated_at, updated_by
        "#,
        id,
        req.assembly_part_id,
        version,
        now,
        user_id,
        now,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create BOM: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(bom))
}

// Получение списка BOM
pub async fn list_boms(
    Extension(state): Extension<AppState>,
    Query(query): Query<ListBomsQuery>,
) -> Result<Json<Vec<Bom>>, StatusCode> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let mut sql = "SELECT id, assembly_part_id, version, status, created_at, created_by, updated_at, updated_by FROM boms WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];

    if let Some(assembly_part_id) = query.assembly_part_id {
        params.push(format!(" AND assembly_part_id = '{}'", assembly_part_id));
    }

    if let Some(ref status) = query.status {
        params.push(format!(" AND status = '{}'", status));
    }

    for param in params {
        sql.push_str(&param);
    }

    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", per_page, offset));

    let boms = sqlx::query_as::<_, Bom>(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list BOMs: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(boms))
}

// Получение BOM по ID
pub async fn get_bom(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Bom>, StatusCode> {
    let bom = sqlx::query_as!(
        Bom,
        "SELECT id, assembly_part_id, version, status, created_at, created_by, updated_at, updated_by FROM boms WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(bom))
}

// Обновление BOM
pub async fn update_bom(
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBomRequest>,
) -> Result<Json<Bom>, StatusCode> {
    let user_id = headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let now = Utc::now();

    let mut updates = vec!["updated_at = $1".to_string(), "updated_by = $2".to_string()];
    let mut next_param = 3;

    if req.version.is_some() {
        updates.push(format!("version = ${}", next_param));
        next_param += 1;
    }

    if req.status.is_some() {
        updates.push(format!("status = ${}", next_param));
    }

    let sql = format!(
        "UPDATE boms SET {} WHERE id = ${} RETURNING id, assembly_part_id, version, status, created_at, created_by, updated_at, updated_by",
        updates.join(", "),
        next_param + 1
    );

    let mut query = sqlx::query_as::<_, Bom>(&sql)
        .bind(now)
        .bind(user_id);

    if let Some(version) = req.version {
        query = query.bind(version);
    }

    if let Some(status) = req.status {
        query = query.bind(status);
    }

    query = query.bind(id);

    let bom = query
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(bom))
}

// Удаление BOM
pub async fn delete_bom(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!("DELETE FROM boms WHERE id = $1", id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

// Добавление элемента в BOM
pub async fn add_bom_item(
    Extension(state): Extension<AppState>,
    Path(bom_id): Path<Uuid>,
    Json(req): Json<AddBomItemRequest>,
) -> Result<Json<BomItem>, StatusCode> {
    let id = Uuid::new_v4();
    let level = req.level.unwrap_or(0);
    let now = Utc::now();

    let item = sqlx::query_as!(
        BomItem,
        r#"
        INSERT INTO bom_items (id, bom_id, part_id, position, quantity, unit, note, level, parent_item_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, bom_id, part_id, position, quantity, unit, note, level, parent_item_id, created_at, updated_at
        "#,
        id,
        bom_id,
        req.part_id,
        req.position,
        req.quantity,
        req.unit,
        req.note,
        level,
        req.parent_item_id,
        now,
        now
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to add BOM item: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(item))
}

// Получение списка элементов BOM
pub async fn list_bom_items(
    Extension(state): Extension<AppState>,
    Path(bom_id): Path<Uuid>,
) -> Result<Json<Vec<BomItem>>, StatusCode> {
    let items = sqlx::query_as!(
        BomItem,
        "SELECT id, bom_id, part_id, position, quantity, unit, note, level, parent_item_id, created_at, updated_at FROM bom_items WHERE bom_id = $1 ORDER BY position",
        bom_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to list BOM items: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(items))
}

// Удаление элемента из BOM
pub async fn delete_bom_item(
    Extension(state): Extension<AppState>,
    Path(item_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!("DELETE FROM bom_items WHERE id = $1", item_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

// Where-used запрос
pub async fn where_used(
    Extension(state): Extension<AppState>,
    Path(part_id): Path<Uuid>,
) -> Result<Json<Vec<WhereUsedResult>>, StatusCode> {
    let results = sqlx::query_as!(
        WhereUsedResult,
        r#"
        SELECT 
            b.id as bom_id,
            b.assembly_part_id,
            b.version as bom_version,
            bi.position,
            bi.quantity,
            bi.unit
        FROM bom_items bi
        JOIN boms b ON bi.bom_id = b.id
        WHERE bi.part_id = $1
        ORDER BY b.assembly_part_id, b.version
        "#,
        part_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query where-used: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(results))
}

#[derive(Debug, Serialize)]
pub struct WhereUsedResult {
    pub bom_id: Uuid,
    pub assembly_part_id: Uuid,
    pub bom_version: String,
    pub position: String,
    pub quantity: f64,
    pub unit: String,
}
