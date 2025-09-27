use axum::{Router, extract::Path, http::header, response::IntoResponse, routing::get};
use tokio::fs;

pub fn make_static_router<T>() -> Router<T>
where
    T: Sync + Send + Clone + 'static,
{
    Router::new().route("/style/{file_name}", get(style))
}

async fn style(Path(file_name): Path<String>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        fs::read_to_string(format!("styles/{file_name}"))
            .await
            .unwrap(),
    )
}
