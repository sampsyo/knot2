use axum::{Router, http::Uri};

pub fn serve() {
    serve_inner()
}

#[tokio::main]
async fn serve_inner() {
    tracing_subscriber::fmt::init();

    let app = Router::new().fallback(blarg);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn blarg(uri: Uri) -> &'static str {
    tracing::info!("request for {}", uri);
    "Hello, World!"
}
