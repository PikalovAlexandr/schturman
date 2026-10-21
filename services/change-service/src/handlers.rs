use crate::models::*;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

fn get_user_id_from_headers(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
    headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::UNAUTHORIZED)
}

async fn add_history(
    pool: &PgPool,
    change_order_id: Uuid,
    user_id: Uuid,
    action: &str,
    details: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO change_history (change_order_id, user_id, action, details)
         VALUES ($1, $2, $3, $4)"
    )
    .bind(change_order_id)
    .bind(user_id)
    .bind(action)
    .bind(details)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_change_order(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(payload): Json<CreateChangeOrderRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());

    let result = sqlx::query_as::<_, ChangeOrder>(
        "INSERT INTO changes (number, title, description, reason, change_type, priority, initiator_id, updated_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
         RETURNING *"
    )
    .bind(&payload.number)
    .bind(&payload.title)
    .bind(&payload.description.unwrap_or_default())
    .bind(&payload.reason)
    .bind(&payload.change_type)
    .bind(&priority)
    .bind(user_id)
    .fetch_one(&pool)
    .await;

    match result {
        Ok(change_order) => {
            let _ = add_history(&pool, change_order.id, user_id, "created", None).await;
            Ok((StatusCode::CREATED, Json(change_order)))
        }
        Err(e) => {
            eprintln!("Failed to create change order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct ListQuery {
    status: Option<String>,
    change_type: Option<String>,
    priority: Option<String>,
    created_by: Option<Uuid>,
}

pub async fn list_changes(
    State(pool): State<PgPool>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let mut query = "SELECT * FROM changes WHERE 1=1".to_string();
    
    if params.status.is_some() {
        query.push_str(" AND status = $1");
    }
    if params.change_type.is_some() {
        query.push_str(" AND change_type = $2");
    }
    if params.priority.is_some() {
        query.push_str(" AND priority = $3");
    }
    if params.created_by.is_some() {
        query.push_str(" AND created_by = $4");
    }
    
    query.push_str(" ORDER BY created_at DESC");

    let mut sql_query = sqlx::query_as::<_, ChangeOrder>(&query);
    
    if let Some(status) = params.status {
        sql_query = sql_query.bind(status);
    }
    if let Some(change_type) = params.change_type {
        sql_query = sql_query.bind(change_type);
    }
    if let Some(priority) = params.priority {
        sql_query = sql_query.bind(priority);
    }
    if let Some(created_by) = params.created_by {
        sql_query = sql_query.bind(created_by);
    }

    match sql_query.fetch_all(&pool).await {
        Ok(changes) => Ok(Json(changes)),
        Err(e) => {
            eprintln!("Failed to list change orders: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_change_order(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, ChangeOrder>("SELECT * FROM changes WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
    {
        Ok(change_order) => Ok(Json(change_order)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get change order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_change_order(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateChangeOrderRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    let mut updates = vec![];
    let mut index = 1;

    if payload.title.is_some() {
        updates.push(format!("title = ${}", index));
        index += 1;
    }
    if payload.description.is_some() {
        updates.push(format!("description = ${}", index));
        index += 1;
    }
    if payload.reason.is_some() {
        updates.push(format!("reason = ${}", index));
        index += 1;
    }
    if payload.change_type.is_some() {
        updates.push(format!("change_type = ${}", index));
        index += 1;
    }
    if payload.priority.is_some() {
        updates.push(format!("priority = ${}", index));
        index += 1;
    }

    if updates.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    updates.push(format!("updated_by = ${}", index));
    index += 1;
    updates.push("updated_at = NOW()".to_string());

    let query = format!(
        "UPDATE changes SET {} WHERE id = ${} RETURNING *",
        updates.join(", "),
        index
    );

    let mut sql_query = sqlx::query_as::<_, ChangeOrder>(&query);

    if let Some(title) = payload.title {
        sql_query = sql_query.bind(title);
    }
    if let Some(description) = payload.description {
        sql_query = sql_query.bind(description);
    }
    if let Some(reason) = payload.reason {
        sql_query = sql_query.bind(reason);
    }
    if let Some(change_type) = payload.change_type {
        sql_query = sql_query.bind(change_type);
    }
    if let Some(priority) = payload.priority {
        sql_query = sql_query.bind(priority);
    }

    sql_query = sql_query.bind(user_id).bind(id);

    match sql_query.fetch_one(&pool).await {
        Ok(change_order) => {
            let _ = add_history(&pool, id, user_id, "updated", None).await;
            Ok(Json(change_order))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update change order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_change_order(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    match sqlx::query("DELETE FROM changes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                let _ = add_history(&pool, id, user_id, "deleted", None).await;
                Ok(StatusCode::NO_CONTENT)
            }
        }
        Err(e) => {
            eprintln!("Failed to delete change order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn submit_change_order(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    match sqlx::query_as::<_, ChangeOrder>(
        "UPDATE changes SET status = 'submitted', updated_by = $1, updated_at = NOW()
         WHERE id = $2 AND status = 'draft' RETURNING *"
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    {
        Ok(change_order) => {
            let _ = add_history(&pool, id, user_id, "submitted", None).await;
            Ok(Json(change_order))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to submit change order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn approve_change_order(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApprovalDecisionRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    // Update approval record
    let approval_result = sqlx::query(
        "UPDATE change_approvals 
         SET status = 'approved', comment = $1, updated_at = NOW()
         WHERE change_order_id = $2 AND approver_id = $3 AND status = 'pending'"
    )
    .bind(&payload.comment)
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await;

    if let Err(e) = approval_result {
        eprintln!("Failed to update approval: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Check if all approvals are complete
    let pending_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM change_approvals WHERE change_order_id = $1 AND status = 'pending'"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or((1,));

    if pending_count.0 == 0 {
        // All approvals complete, mark as approved
        match sqlx::query_as::<_, ChangeOrder>(
            "UPDATE changes SET status = 'approved', updated_by = $1, updated_at = NOW()
             WHERE id = $2 RETURNING *"
        )
        .bind(user_id)
        .bind(id)
        .fetch_one(&pool)
        .await
        {
            Ok(change_order) => {
                let details = serde_json::json!({ "comment": payload.comment });
                let _ = add_history(&pool, id, user_id, "approved", Some(details)).await;
                Ok(Json(change_order))
            }
            Err(e) => {
                eprintln!("Failed to approve change order: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        // Still pending approvals
        match sqlx::query_as::<_, ChangeOrder>(
            "UPDATE changes SET status = 'in_review', updated_by = $1, updated_at = NOW()
             WHERE id = $2 RETURNING *"
        )
        .bind(user_id)
        .bind(id)
        .fetch_one(&pool)
        .await
        {
            Ok(change_order) => Ok(Json(change_order)),
            Err(e) => {
                eprintln!("Failed to update change order status: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn reject_change_order(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApprovalDecisionRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    // Update approval record
    let _ = sqlx::query(
        "UPDATE change_approvals 
         SET status = 'rejected', comment = $1, updated_at = NOW()
         WHERE change_order_id = $2 AND approver_id = $3"
    )
    .bind(&payload.comment)
    .bind(id)
    .bind(user_id)
    .execute(&pool)
    .await;

    match sqlx::query_as::<_, ChangeOrder>(
        "UPDATE changes SET status = 'rejected', updated_by = $1, updated_at = NOW()
         WHERE id = $2 RETURNING *"
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    {
        Ok(change_order) => {
            let details = serde_json::json!({ "comment": payload.comment });
            let _ = add_history(&pool, id, user_id, "rejected", Some(details)).await;
            Ok(Json(change_order))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to reject change order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn implement_change_order(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    match sqlx::query_as::<_, ChangeOrder>(
        "UPDATE changes SET status = 'implemented', updated_by = $1, updated_at = NOW()
         WHERE id = $2 AND status = 'approved' RETURNING *"
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    {
        Ok(change_order) => {
            let _ = add_history(&pool, id, user_id, "implemented", None).await;
            Ok(Json(change_order))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to implement change order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_affected_item(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddAffectedItemRequest>,
) -> impl IntoResponse {
    let _user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    // Support both part_id and document_id
    if let Some(part_id) = payload.part_id {
        match sqlx::query_as::<_, AffectedItem>(
            "INSERT INTO change_affected_parts (change_id, part_id)
             VALUES ($1, $2) RETURNING *"
        )
        .bind(id)
        .bind(part_id)
        .fetch_one(&pool)
        .await
        {
            Ok(item) => return Ok((StatusCode::CREATED, Json(item))),
            Err(e) => {
                eprintln!("Failed to add affected part: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    if let Some(document_id) = payload.document_id {
        // Insert into change_affected_documents
        match sqlx::query(
            "INSERT INTO change_affected_documents (change_id, document_id) VALUES ($1, $2)"
        )
        .bind(id)
        .bind(document_id)
        .execute(&pool)
        .await
        {
            Ok(_) => {
                // Return a mock AffectedItem (document table doesn't match the struct)
                return Err(StatusCode::NOT_IMPLEMENTED);
            }
            Err(e) => {
                eprintln!("Failed to add affected document: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

pub async fn list_affected_items(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, AffectedItem>(
        "SELECT * FROM change_affected_parts WHERE change_id = $1 ORDER BY created_at"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    {
        Ok(items) => Ok(Json(items)),
        Err(e) => {
            eprintln!("Failed to list affected items: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn remove_affected_item(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path((change_id, item_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let _user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    match sqlx::query(
        "DELETE FROM change_affected_parts WHERE change_id = $1 AND part_id = $2"
    )
    .bind(change_id)
    .bind(item_id)
    .execute(&pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                Ok(StatusCode::NO_CONTENT)
            }
        }
        Err(e) => {
            eprintln!("Failed to remove affected item: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_approval(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddApprovalRequest>,
) -> impl IntoResponse {
    let _user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    match sqlx::query_as::<_, Approval>(
        "INSERT INTO change_approvals (id, change_order_id, approver_id, status)
         VALUES (gen_random_uuid(), $1, $2, 'pending') RETURNING *"
    )
    .bind(id)
    .bind(payload.approver_id)
    .fetch_one(&pool)
    .await
    {
        Ok(approval) => Ok((StatusCode::CREATED, Json(approval))),
        Err(e) => {
            eprintln!("Failed to add approval: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_approvals(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, Approval>(
        "SELECT * FROM change_approvals WHERE change_order_id = $1 ORDER BY created_at"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    {
        Ok(approvals) => Ok(Json(approvals)),
        Err(e) => {
            eprintln!("Failed to list approvals: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_approval_status(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path((change_id, approval_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateApprovalRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    // Validate status
    if payload.status != "approved" && payload.status != "rejected" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update approval
    match sqlx::query_as::<_, Approval>(
        "UPDATE change_approvals 
         SET status = $1, comment = $2, updated_at = NOW()
         WHERE id = $3 AND change_order_id = $4 AND approver_id = $5
         RETURNING *"
    )
    .bind(&payload.status)
    .bind(&payload.comment)
    .bind(approval_id)
    .bind(change_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    {
        Ok(approval) => {
            // Add history record
            let details = serde_json::json!({
                "approver_id": approval.approver_id,
                "status": approval.status,
                "comment": approval.comment
            });
            let _ = add_history(&pool, change_id, user_id, &format!("approval_{}", payload.status), Some(details)).await;

            // If approved, check if all approvals are done and update change order status
            if payload.status == "approved" {
                let pending_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM change_approvals 
                     WHERE change_order_id = $1 AND status = 'pending'"
                )
                .bind(change_id)
                .fetch_one(&pool)
                .await
                .unwrap_or(1);

                if pending_count == 0 {
                    // All approvals done, update change order to approved
                    let _ = sqlx::query(
                        "UPDATE changes SET status = 'approved', updated_by = $1, updated_at = NOW()
                         WHERE id = $2"
                    )
                    .bind(user_id)
                    .bind(change_id)
                    .execute(&pool)
                    .await;

                    let _ = add_history(&pool, change_id, user_id, "approved", None).await;
                }
            } else {
                // If rejected, reject the whole change order
                let _ = sqlx::query(
                    "UPDATE changes SET status = 'rejected', updated_by = $1, updated_at = NOW()
                     WHERE id = $2"
                )
                .bind(user_id)
                .bind(change_id)
                .execute(&pool)
                .await;

                let _ = add_history(&pool, change_id, user_id, "rejected", Some(serde_json::json!({ "reason": payload.comment }))).await;
            }

            Ok(Json(approval))
        }
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update approval status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_comment(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddCommentRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&headers) {
        Ok(id) => id,
        Err(status) => return Err(status),
    };

    match sqlx::query_as::<_, ChangeComment>(
        "INSERT INTO change_comments (change_order_id, user_id, comment)
         VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(id)
    .bind(user_id)
    .bind(&payload.comment)
    .fetch_one(&pool)
    .await
    {
        Ok(comment) => Ok((StatusCode::CREATED, Json(comment))),
        Err(e) => {
            eprintln!("Failed to add comment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_comments(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, ChangeComment>(
        "SELECT * FROM change_comments WHERE change_order_id = $1 ORDER BY created_at"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    {
        Ok(comments) => Ok(Json(comments)),
        Err(e) => {
            eprintln!("Failed to list comments: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_change_history(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, ChangeHistory>(
        "SELECT * FROM change_history WHERE change_order_id = $1 ORDER BY created_at DESC"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    {
        Ok(history) => Ok(Json(history)),
        Err(e) => {
            eprintln!("Failed to get change history: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
