use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use schturman_models::{Claims, UserRole};
use uuid::Uuid;

/// Создание access token
pub fn create_access_token(
    user_id: Uuid,
    username: &str,
    role: UserRole,
    secret: &str,
    expiration: i64,
) -> anyhow::Result<String> {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(expiration);

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        role,
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Создание refresh token
pub fn create_refresh_token(
    user_id: Uuid,
    username: &str,
    role: UserRole,
    secret: &str,
    expiration: i64,
) -> anyhow::Result<String> {
    create_access_token(user_id, username, role, secret, expiration)
}

/// Верификация токена
pub fn verify_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
