use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod handlers;
mod auth_middleware;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let state = Arc::new(AppState { pool });

    let protected = Router::new()
        .route("/api/documents/upload", post(handlers::upload_document))
        .route("/api/documents", get(handlers::list_documents))
        .route("/api/documents/:id", get(handlers::get_document))
        .route("/api/documents/:id/versions", get(handlers::get_document_versions))
        .route("/api/documents/:id/download", get(handlers::download_document))
        .layer(middleware::from_fn(auth_middleware::auth_middleware));

    let app = Router::new()
        .route("/health", get(health))
        .merge(protected)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::env::var("DOCUMENT_SERVICE_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3003".to_string());
    
    tracing::info!("Document service starting on {}", addr);

    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}
