use crate::Context;
use crate::core::Resource;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{StatusCode, Uri, header},
    response::IntoResponse,
    response::Response,
};
use axum_extra::body::AsyncReadBody;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

#[tokio::main]
pub async fn serve(ctx: Context) {
    tracing_subscriber::fmt::init();

    let shared_ctx = Arc::new(ctx);
    let app = Router::new().fallback(handle).with_state(shared_ctx);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn send_file(path: &Path) -> Result<Response, (StatusCode, String)> {
    let mime = mime_guess::from_path(path)
        .first_raw()
        .unwrap_or(mime_guess::mime::OCTET_STREAM.as_str());

    let file = fs::File::open(path)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("not found: {e}")))?;

    let headers = [(header::CONTENT_TYPE, mime)];
    let body = AsyncReadBody::new(file);
    Ok((headers, body).into_response())
}

async fn handle(
    State(ctx): State<Arc<Context>>,
    uri: Uri,
) -> Result<Response, (StatusCode, String)> {
    // TODO percent-unescape the path?
    let path = uri.path();
    tracing::info!("request for {:?}", &path);

    match ctx.resolve_resource(path) {
        Some(Resource::Note(src_path)) => {
            let mut buf: Vec<u8> = vec![];
            ctx.render_note(&src_path, &mut buf).unwrap();
            Ok(Body::from(buf).into_response())
        }
        Some(Resource::Static(src_path)) => send_file(&src_path).await,
        Some(Resource::Directory(_)) => Err((
            StatusCode::NOT_IMPLEMENTED,
            "directory listings not implemented".into(),
        )),
        None => Err((StatusCode::NOT_FOUND, "not found".into())),
    }
}
