use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Элемент спецификации (BOM item)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomItem {
    pub id: Uuid,
    
    /// ID родительской спецификации
    pub bom_id: Uuid,
    
    /// ID компонента
    pub part_id: Uuid,
    
    /// Позиция в спецификации
    pub position: String,
    
    /// Количество
    pub quantity: f64,
    
    /// Единица измерения
    pub unit: String,
    
    /// Примечание
    pub note: Option<String>,
    
    /// Уровень вложенности (0 - верхний)
    pub level: i32,
    
    /// ID родительского элемента (для иерархии)
    pub parent_item_id: Option<Uuid>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Спецификация (Bill of Materials)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bom {
    pub id: Uuid,
    
    /// ID компонента-сборки
    pub assembly_part_id: Uuid,
    
    /// Версия спецификации
    pub version: String,
    
    /// Статус
    pub status: BomStatus,
    
    /// Список элементов
    pub items: Vec<BomItem>,
    
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

/// Статус спецификации
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BomStatus {
    Draft,
    Review,
    Released,
    Obsolete,
}

/// Запрос на создание спецификации
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBomRequest {
    pub assembly_part_id: Uuid,
    
    #[validate(length(min = 1, max = 50))]
    pub version: String,
}

/// Запрос на добавление элемента в спецификацию
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddBomItemRequest {
    pub bom_id: Uuid,
    pub part_id: Uuid,
    
    #[validate(length(min = 1, max = 50))]
    pub position: String,
    
    #[validate(range(min = 0.001))]
    pub quantity: f64,
    
    #[validate(length(min = 1, max = 20))]
    pub unit: String,
    
    pub note: Option<String>,
    pub parent_item_id: Option<Uuid>,
}

/// Where-Used запрос (где используется компонент)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereUsedQuery {
    pub part_id: Uuid,
}

/// Результат Where-Used запроса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereUsedResult {
    pub part_id: Uuid,
    pub used_in: Vec<WhereUsedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereUsedItem {
    pub bom_id: Uuid,
    pub assembly_part_id: Uuid,
    pub assembly_name: String,
    pub position: String,
    pub quantity: f64,
}

/// Результат сравнения двух спецификаций
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomDiff {
    pub added_items: Vec<BomItem>,
    pub removed_items: Vec<BomItem>,
    pub modified_items: Vec<BomItemDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BomItemDiff {
    pub item_id: Uuid,
    pub old_quantity: f64,
    pub new_quantity: f64,
    pub old_note: Option<String>,
    pub new_note: Option<String>,
}
