use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;
use uuid::Uuid;

mod config;
mod handlers;
mod middleware;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

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

    info!("Starting part-service on {}", config.server_address);

    // Подключение к базе данных
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    info!("Connected to database");

    // Создание состояния приложения
    let state = Arc::new(AppState { pool });

    // Настройка маршрутов
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/parts", post(handlers::create_part))
        .route("/api/parts", get(handlers::list_parts))
        .route("/api/parts/:id", get(handlers::get_part))
        .route("/api/parts/:id", put(handlers::update_part))
        .route("/api/parts/:id", delete(handlers::delete_part))
        .route("/api/parts/search", get(handlers::search_parts))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Запуск сервера (Axum 0.6 API)
    info!("Part service listening on {}", config.server_address);
    
    axum::Server::bind(&config.server_address.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "part-service"
    }))
}
