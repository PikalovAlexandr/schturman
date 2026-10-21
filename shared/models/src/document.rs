use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Формат документа
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    Step,       // STEP CAD
    Iges,       // IGES CAD
    Stl,        // STL 3D model
    Pdf,        // PDF документ
    Dwg,        // AutoCAD DWG
    Dxf,        // AutoCAD DXF
    Image,      // Изображение
    Other,      // Прочее
}

/// Статус документа
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentStatus {
    Draft,      // Черновик
    CheckedOut, // На редактировании
    CheckedIn,  // Зарегистрирован
    Released,   // Выпущен
    Obsolete,   // Устарел
}

/// Документ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    
    pub name: String,
    pub description: Option<String>,
    pub format: DocumentFormat,
    pub status: DocumentStatus,
    
    /// Версия документа
    pub version: String,
    
    /// Размер файла в байтах
    pub file_size: i64,
    
    /// Хеш файла (для content-addressable storage)
    pub file_hash: String,
    
    /// Путь в S3/MinIO
    pub storage_path: String,
    
    /// URL миниатюры (если есть)
    pub thumbnail_url: Option<String>,
    
    /// ID компонента, к которому привязан документ
    pub part_id: Option<Uuid>,
    
    /// Кем выгружен (checked out)
    pub checked_out_by: Option<Uuid>,
    
    /// Время выгрузки
    pub checked_out_at: Option<DateTime<Utc>>,
    
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
}

/// Запрос на загрузку документа
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UploadDocumentRequest {
    #[validate(length(min = 1, max = 500))]
    pub name: String,
    
    pub description: Option<String>,
    pub format: DocumentFormat,
    pub part_id: Option<Uuid>,
}

/// Запрос на checkout документа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckOutRequest {
    pub document_id: Uuid,
}

/// Запрос на checkin документа
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckInRequest {
    pub document_id: Uuid,
    
    #[validate(length(min = 1, max = 1000))]
    pub comment: Option<String>,
}

/// История версий документа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version: String,
    pub file_hash: String,
    pub storage_path: String,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}
