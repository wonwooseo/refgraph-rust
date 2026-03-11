use axum::Router;
use axum::routing::get;

#[tokio::main]
async fn main() {
    let srv = Router::new()
        .route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    let _ = axum::serve(listener, srv).await;
}
