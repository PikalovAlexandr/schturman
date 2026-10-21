use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub jwt_refresh_secret: String,
    pub jwt_refresh_expiration: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            server_address: env::var("AUTH_SERVICE_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:3001".to_string()),
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .expect("JWT_REFRESH_SECRET must be set"),
            jwt_refresh_expiration: env::var("JWT_REFRESH_EXPIRATION")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()?,
        })
    }
}
