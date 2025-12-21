use crate::Context;
use crate::core::Resource;
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
    response::Response,
};
use axum_extra::response::file_stream::FileStream;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

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
        Some(Resource::Static(src_path)) => {
            let file = File::open(src_path).await.unwrap();
            let stream = ReaderStream::new(file);
            let resp = FileStream::new(stream).file_name("hi");
            Ok(resp.into_response())
        }
        Some(Resource::Directory(_)) => Ok("directory listings not implemented".into_response()),
        None => Ok("not found".into_response()),
    }
}
