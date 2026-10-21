use axum::{
    extract::Json,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

mod config;
mod db;
mod handlers;
mod jwt;
mod password;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Загрузка конфигурации
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    info!("Starting auth-service on {}", config.server_address);

    // Подключение к базе данных
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    info!("Connected to database");

    // Выполнение миграций
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    info!("Migrations completed");

    // Создание состояния приложения
    let state = Arc::new(AppState {
        pool,
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiration: config.jwt_expiration,
        jwt_refresh_secret: config.jwt_refresh_secret.clone(),
        jwt_refresh_expiration: config.jwt_refresh_expiration,
    });

    // Настройка маршрутов
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/refresh", post(handlers::refresh_token))
        .route("/api/auth/me", get(handlers::get_current_user))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Запуск сервера (Axum 0.6 API)
    info!("Auth service listening on {}", config.server_address);
    
    axum::Server::bind(&config.server_address.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "auth-service"
    }))
}
