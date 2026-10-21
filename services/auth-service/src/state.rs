use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub jwt_refresh_secret: String,
    pub jwt_refresh_expiration: i64,
}
