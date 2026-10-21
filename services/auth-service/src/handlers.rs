use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

use schturman_models::{LoginRequest, LoginResponse, RegisterUserRequest, User, UserRole};

use crate::{db::{UserIdRow, UserRow}, jwt, password, state::AppState};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// Регистрация нового пользователя
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Валидация входных данных
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        )
    })?;

    // Проверка, существует ли пользователь
    let existing_user = sqlx::query_as::<_, UserIdRow>(
        "SELECT id FROM users WHERE username = $1 OR email = $2"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
            }),
        )
    })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "User already exists".to_string(),
            }),
        ));
    }

    // Хеширование пароля
    let password_hash = password::hash_password(&payload.password).map_err(|e| {
        error!("Password hashing error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to hash password".to_string(),
            }),
        )
    })?;

    // Создание пользователя
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, full_name, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind("engineer")
    .bind(true)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("Failed to create user: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create user".to_string(),
            }),
        )
    })?;

    info!("User registered: {}", payload.username);

    // Создание токенов
    let access_token = jwt::create_access_token(
        user_id,
        &payload.username,
        UserRole::Engineer,
        &state.jwt_secret,
        state.jwt_expiration,
    )
    .map_err(|e| {
        error!("Failed to create access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token".to_string(),
            }),
        )
    })?;

    let refresh_token = jwt::create_refresh_token(
        user_id,
        &payload.username,
        UserRole::Engineer,
        &state.jwt_refresh_secret,
        state.jwt_refresh_expiration,
    )
    .map_err(|e| {
        error!("Failed to create refresh token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token".to_string(),
            }),
        )
    })?;

    let user = User {
        id: user_id,
        username: payload.username,
        email: payload.email,
        full_name: payload.full_name,
        role: UserRole::Engineer,
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    let response = LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiration,
        user,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Вход пользователя
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Валидация
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
            }),
        )
    })?;

    // Поиск пользователя
    let user = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, email, password_hash, full_name, role, is_active, created_at, updated_at
        FROM users
        WHERE username = $1
        "#
    )
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
            }),
        )
    })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        )
    })?;

    // Проверка пароля
    let password_valid = password::verify_password(&payload.password, &user.password_hash)
        .map_err(|e| {
            error!("Password verification error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication error".to_string(),
                }),
            )
        })?;

    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        ));
    }

    if !user.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "User is inactive".to_string(),
            }),
        ));
    }

    // Парсинг роли
    let role = match user.role.as_str() {
        "admin" => UserRole::Admin,
        "engineer" => UserRole::Engineer,
        "viewer" => UserRole::Viewer,
        "manager" => UserRole::Manager,
        _ => UserRole::Viewer,
    };

    // Создание токенов
    let access_token = jwt::create_access_token(
        user.id,
        &user.username,
        role,
        &state.jwt_secret,
        state.jwt_expiration,
    )
    .map_err(|e| {
        error!("Failed to create access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token".to_string(),
            }),
        )
    })?;

    let refresh_token = jwt::create_refresh_token(
        user.id,
        &user.username,
        role,
        &state.jwt_refresh_secret,
        state.jwt_refresh_expiration,
    )
    .map_err(|e| {
        error!("Failed to create refresh token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token".to_string(),
            }),
        )
    })?;

    info!("User logged in: {}", user.username);

    let response = LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiration,
        user: User {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            role,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        },
    };

    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    refresh_token: String,
}

#[derive(Debug, sqlx::FromRow)]
struct UserWithoutPassword {
    id: Uuid,
    username: String,
    email: String,
    full_name: String,
    role: String,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Обновление access token
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Верификация refresh token
    let claims = jwt::verify_token(&payload.refresh_token, &state.jwt_refresh_secret).map_err(
        |e| {
            error!("Invalid refresh token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid refresh token".to_string(),
                }),
            )
        },
    )?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID in token: {}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid token".to_string(),
            }),
        )
    })?;

    // Проверка, существует ли пользователь
    let user = sqlx::query_as::<_, UserWithoutPassword>(
        r#"
        SELECT id, username, email, full_name, role, is_active, created_at, updated_at
        FROM users
        WHERE id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
            }),
        )
    })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "User not found".to_string(),
            }),
        )
    })?;

    if !user.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "User is inactive".to_string(),
            }),
        ));
    }

    // Создание нового access token
    let access_token = jwt::create_access_token(
        user.id,
        &user.username,
        claims.role,
        &state.jwt_secret,
        state.jwt_expiration,
    )
    .map_err(|e| {
        error!("Failed to create access token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Failed to create token".to_string(),
            }),
        )
    })?;

    #[derive(Serialize)]
    struct RefreshResponse {
        access_token: String,
        token_type: String,
        expires_in: i64,
    }

    Ok(Json(RefreshResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiration,
    }))
}

/// Получение текущего пользователя
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // Извлечение токена из заголовка Authorization
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing or invalid Authorization header".to_string(),
                }),
            )
        })?;

    // Верификация токена
    let claims = jwt::verify_token(token, &state.jwt_secret).map_err(|e| {
        error!("Invalid token: {}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid token".to_string(),
            }),
        )
    })?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
        error!("Invalid user ID in token: {}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid token".to_string(),
            }),
        )
    })?;

    // Получение пользователя
    let user = sqlx::query_as::<_, UserWithoutPassword>(
        r#"
        SELECT id, username, email, full_name, role, is_active, created_at, updated_at
        FROM users
        WHERE id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
            }),
        )
    })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "User not found".to_string(),
            }),
        )
    })?;

    let role = match user.role.as_str() {
        "admin" => UserRole::Admin,
        "engineer" => UserRole::Engineer,
        "viewer" => UserRole::Viewer,
        "manager" => UserRole::Manager,
        _ => UserRole::Viewer,
    };

    Ok(Json(User {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        role,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}
