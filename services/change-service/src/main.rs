use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

mod handlers;
mod models;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://schturman:schturman_password@localhost:5434/schturman".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Public routes (no auth)
    let public_router = Router::new()
        .route("/health", get(|| async { "OK" }));

    // Protected routes (require auth header x-user-id)
    let protected_router = Router::new()
        .route("/api/changes", post(handlers::create_change_order))
        .route("/api/changes", get(handlers::list_changes))
        .route("/api/changes/:id", get(handlers::get_change_order))
        .route("/api/changes/:id", put(handlers::update_change_order))
        .route("/api/changes/:id", delete(handlers::delete_change_order))
        .route("/api/changes/:id/submit", post(handlers::submit_change_order))
        .route("/api/changes/:id/approve", post(handlers::approve_change_order))
        .route("/api/changes/:id/reject", post(handlers::reject_change_order))
        .route("/api/changes/:id/implement", post(handlers::implement_change_order))
        .route("/api/changes/:id/affected-items", post(handlers::add_affected_item))
        .route("/api/changes/:id/affected-items", get(handlers::list_affected_items))
        .route("/api/changes/:id/affected-items/:item_id", delete(handlers::remove_affected_item))
        .route("/api/changes/:id/approvals", post(handlers::add_approval))
        .route("/api/changes/:id/approvals", get(handlers::list_approvals))
        .route("/api/changes/:id/approvals/:approval_id", put(handlers::update_approval_status))
        .route("/api/changes/:id/comments", post(handlers::add_comment))
        .route("/api/changes/:id/comments", get(handlers::list_comments))
        .route("/api/changes/:id/history", get(handlers::get_change_history));

    let app = Router::new()
        .merge(public_router)
        .merge(protected_router)
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3005));
    tracing::info!("Change service listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
