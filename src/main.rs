use axum::Router;
use axum::routing::{get, post};
use axum::Extension;
use sqlx::PgPool;

mod handlers;
mod models;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let srv = Router::new()
        .route("/api/ref/{code}", get(handlers::get_ref))
        .route("/api/ref", post(handlers::create_ref))
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    let _ = axum::serve(listener, srv).await;
}
