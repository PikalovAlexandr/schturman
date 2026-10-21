use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Статус жизненного цикла компонента
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PartStatus {
    Draft,      // Черновик
    Review,     // На проверке
    Released,   // Выпущен
    Obsolete,   // Устарел
}

/// Тип компонента
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PartType {
    Part,         // Деталь
    Assembly,     // Сборка
    Purchased,    // Покупное изделие
    Material,     // Материал
    Standard,     // Стандартное изделие
}

/// Компонент
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Part {
    pub id: Uuid,
    
    #[validate(length(min = 1, max = 100))]
    pub number: String,
    
    #[validate(length(min = 1, max = 500))]
    pub name: String,
    
    pub description: Option<String>,
    pub part_type: PartType,
    pub status: PartStatus,
    
    /// Версия компонента
    pub version: String,
    
    /// Ревизия
    pub revision: i32,
    
    /// Пользовательские атрибуты (JSONB в БД)
    pub attributes: serde_json::Value,
    
    /// Владелец компонента
    pub owner_id: Uuid,
    
    /// Дата создания
    pub created_at: DateTime<Utc>,
    
    /// Дата последнего изменения
    pub updated_at: DateTime<Utc>,
    
    /// Кем изменён
    pub updated_by: Uuid,
}

/// Запрос на создание компонента
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePartRequest {
    #[validate(length(min = 1, max = 100))]
    pub number: String,
    
    #[validate(length(min = 1, max = 500))]
    pub name: String,
    
    pub description: Option<String>,
    pub part_type: PartType,
    pub attributes: Option<serde_json::Value>,
}

/// Запрос на обновление компонента
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePartRequest {
    #[validate(length(min = 1, max = 500))]
    pub name: Option<String>,
    
    pub description: Option<String>,
    pub status: Option<PartStatus>,
    pub attributes: Option<serde_json::Value>,
}

/// Фильтры поиска компонентов
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartFilter {
    pub part_type: Option<PartType>,
    pub status: Option<PartStatus>,
    pub owner_id: Option<Uuid>,
    pub search_query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Результат поиска компонентов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartListResponse {
    pub items: Vec<Part>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
