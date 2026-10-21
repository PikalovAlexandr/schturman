use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;

#[tokio::main]
async fn main() {
    // Инициализация логирования
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_orchestrator=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Подключение к БД
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");

    // Роуты
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/agents", get(handlers::list_agents))
        .route("/api/tasks", post(handlers::create_task))
        .route("/api/tasks", get(handlers::list_tasks))
        .route("/api/tasks/:id", get(handlers::get_task))
        .route("/api/tasks/:id/cancel", delete(handlers::cancel_task))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = "0.0.0.0:3006";
    tracing::info!("AI Orchestrator listening on {}", addr);

    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
