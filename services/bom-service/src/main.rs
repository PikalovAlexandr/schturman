use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;
use uuid::Uuid;

mod auth_middleware;
mod handlers;

use auth_middleware::auth_middleware;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://schturman:schturman_password@localhost:5434/schturman".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/boms", post(handlers::create_bom))
        .route("/api/boms", get(handlers::list_boms))
        .route("/api/boms/:id", get(handlers::get_bom))
        .route("/api/boms/:id", put(handlers::update_bom))
        .route("/api/boms/:id", delete(handlers::delete_bom))
        .route("/api/boms/:id/items", post(handlers::add_bom_item))
        .route("/api/boms/:id/items", get(handlers::list_bom_items))
        .route("/api/bom-items/:id", delete(handlers::delete_bom_item))
        .route("/api/parts/:id/where-used", get(handlers::where_used))
        .layer(CorsLayer::permissive())
        .layer(Extension(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3004));
    tracing::info!("BOM service listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "service": "bom-service"}))
}
