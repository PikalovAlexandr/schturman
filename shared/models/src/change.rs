use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Тип изменения
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Eco,  // Engineering Change Order
    Ecr,  // Engineering Change Request
    Mcr,  // Manufacturing Change Request
}

/// Статус изменения
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    Draft,      // Черновик
    Submitted,  // Подан
    InReview,   // На рассмотрении
    Approved,   // Одобрен
    Rejected,   // Отклонён
    Implemented,// Реализован
    Cancelled,  // Отменён
}

/// Приоритет изменения
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Изменение (ECO/ECR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: Uuid,
    
    /// Номер изменения
    pub number: String,
    
    /// Заголовок
    pub title: String,
    
    /// Описание
    pub description: String,
    
    /// Обоснование
    pub reason: Option<String>,
    
    pub change_type: ChangeType,
    pub status: ChangeStatus,
    pub priority: ChangePriority,
    
    /// Инициатор изменения
    pub initiator_id: Uuid,
    
    /// Ответственный за реализацию
    pub assignee_id: Option<Uuid>,
    
    /// Срок реализации
    pub due_date: Option<DateTime<Utc>>,
    
    /// Затронутые компоненты
    pub affected_parts: Vec<Uuid>,
    
    /// Затронутые документы
    pub affected_documents: Vec<Uuid>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

/// Запрос на создание изменения
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateChangeRequest {
    #[validate(length(min = 1, max = 100))]
    pub number: String,
    
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    
    #[validate(length(min = 1, max = 5000))]
    pub description: String,
    
    pub reason: Option<String>,
    pub change_type: ChangeType,
    pub priority: ChangePriority,
    pub affected_parts: Vec<Uuid>,
    pub affected_documents: Vec<Uuid>,
}

/// Согласование изменения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeApproval {
    pub id: Uuid,
    pub change_id: Uuid,
    
    /// Согласующий
    pub approver_id: Uuid,
    
    /// Статус согласования
    pub status: ApprovalStatus,
    
    /// Комментарий
    pub comment: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,   // Ожидает
    Approved,  // Одобрено
    Rejected,  // Отклонено
}

/// Запрос на отправку изменения на согласование
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitForApprovalRequest {
    pub change_id: Uuid,
    pub approvers: Vec<Uuid>,
}

/// Запрос на согласование/отклонение изменения
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApproveChangeRequest {
    pub change_id: Uuid,
    pub approved: bool,
    
    #[validate(length(max = 1000))]
    pub comment: Option<String>,
}
