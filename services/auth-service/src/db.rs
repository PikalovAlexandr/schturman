// Вспомогательные структуры для работы с БД без макроса query!
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct UserIdRow {
    pub id: Uuid,
}
