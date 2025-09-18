use std::path;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};
use tokio::{
    fs::{self, File},
    process::Command,
};
use tokio_util::io::ReaderStream;

use crate::AppState;

pub async fn root_page(State(state): State<AppState>) -> impl IntoResponse {
    render_page("/".to_string(), state).await
}

pub async fn get_path(State(state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    render_page(path, state).await
}

async fn render_page(mut relative_path: String, state: AppState) -> impl IntoResponse {
    let root_dir = state.root_dir;
    let url_prefix = state.url_prefix;

    while let Some(new_relative_path) = relative_path.strip_prefix('/') {
        relative_path = new_relative_path.to_string();
    }

    relative_path.insert(0, '/');

    let path = format!("{root_dir}{relative_path}");
    let path = path::Path::new(&path);

    if !path.exists() {
        (StatusCode::NOT_FOUND, "file not found".to_string()).into_response()
    } else if path.is_file() {
        Command::new("typst")
            .arg("compile")
            .arg(path)
            .args(["--root", &root_dir])
            .arg("./out/generated.pdf")
            .output()
            .await
            .unwrap();

        let file = File::open("out/generated.pdf").await.unwrap();
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        let content_disposition = format!(
            "inline; filename=\"{}\"",
            path.file_name().unwrap().to_str().unwrap()
        );

        let headers = [
            (header::CONTENT_TYPE, "application/pdf"),
            (header::CONTENT_DISPOSITION, &content_disposition),
        ];

        (StatusCode::OK, headers, body).into_response()
    } else {
        let mut filesystem_objects = fs::read_dir(path).await.unwrap();

        let mut list = match path::Path::new(&relative_path).parent() {
            Some(parent) => format!(
                r#"<li><a href="{url_prefix}{}">..</a></li>"#,
                parent.to_str().unwrap()
            ),
            None => String::new(),
        };

        while let Ok(Some(dir_entry)) = filesystem_objects.next_entry().await {
            let file_name = dir_entry.file_name().to_str().unwrap().to_string();

            if state.exclude_files.contains(&file_name) {
                continue;
            }

            let is_file = dir_entry.path().is_file();

            if is_file && !file_name.ends_with(".typ") {
                // TODO: For now, later this should result in a download
                continue;
            }

            let class = if is_file {
                "file"
            } else {
                "dir"
            };

            list.push_str(format!(r#"<li class="{class}"><a href="{url_prefix}{relative_path}{}{file_name}">{}{file_name}</a></li>"#, if &relative_path == "/" {""} else {"/"}, if class == "dir" {"📁 "} else {""}).as_str());
        }

        (
            StatusCode::OK,
            Html::from(format!(
                r#"
<!DOCTYPE html>
<html>
<head>
    <title>ok</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta charset="utf-8">
    <link rel="stylesheet" href="/main.css">
</head>

<body>
    <h1 class="title">{relative_path}</h1>

    <ul class="dir-list">
        {list}
    </ul>
</body>

</html>"#,
            )),
        )
            .into_response()
    }
}
