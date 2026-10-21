use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChangeOrder {
    pub id: Uuid,
    pub number: String,
    pub title: String,
    pub description: String,
    pub reason: Option<String>,
    pub change_type: String,
    pub priority: String,
    pub status: String,
    pub initiator_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateChangeOrderRequest {
    pub number: String,
    pub title: String,
    pub description: Option<String>,
    pub reason: Option<String>,
    pub change_type: String,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChangeOrderRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub reason: Option<String>,
    pub change_type: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AffectedItem {
    pub change_id: Uuid,
    pub part_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddAffectedItemRequest {
    pub part_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Approval {
    pub id: Uuid,
    pub change_order_id: Uuid,
    pub approver_id: Uuid,
    pub status: String,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddApprovalRequest {
    pub approver_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalDecisionRequest {
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChangeComment {
    pub id: Uuid,
    pub change_order_id: Uuid,
    pub user_id: Uuid,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub comment: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApprovalRequest {
    pub status: String,  // 'approved' or 'rejected'
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChangeHistory {
    pub id: Uuid,
    pub change_order_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
