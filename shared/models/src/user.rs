use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Роль пользователя
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,      // Администратор
    Engineer,   // Инженер
    Viewer,     // Наблюдатель
    Manager,    // Менеджер
}

/// Пользователь
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    
    pub username: String,
    pub email: String,
    
    /// Полное имя
    pub full_name: String,
    
    pub role: UserRole,
    
    /// Активен ли пользователь
    pub is_active: bool,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Запрос на регистрацию пользователя
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 100))]
    pub password: String,
    
    #[validate(length(min = 1, max = 200))]
    pub full_name: String,
}

/// Запрос на вход
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub username: String,
    
    #[validate(length(min = 1))]
    pub password: String,
}

/// Ответ с токеном
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: User,
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub username: String,
    pub role: UserRole,
    pub exp: i64,     // expiration time
    pub iat: i64,     // issued at
}
