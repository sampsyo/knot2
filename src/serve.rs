use crate::Context;
use crate::core::Resource;
use axum::{Router, extract::State, http::Uri};
use std::sync::Arc;

#[tokio::main]
pub async fn serve(ctx: Context) {
    tracing_subscriber::fmt::init();

    let shared_ctx = Arc::new(ctx);
    let app = Router::new().fallback(blarg).with_state(shared_ctx);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn blarg(State(ctx): State<Arc<Context>>, uri: Uri) -> Vec<u8> {
    // TODO percent-unescape the path?
    let path = uri.path();
    tracing::info!("request for {:?}", &path);

    match ctx.resolve_resource(path) {
        Some(Resource::Note(src_path)) => {
            let mut buf: Vec<u8> = vec![];
            ctx.render_note(&src_path, &mut buf).unwrap();
            buf
        }
        _ => "not found".into(),
    }
}
