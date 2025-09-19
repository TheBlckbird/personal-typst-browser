use std::{cmp::Ordering, path};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};
use log::debug;
use mime_guess::mime;
use tokio::{
    fs::{self, File},
    process::Command,
};
use tokio_util::io::ReaderStream;

use crate::AppState;

pub async fn root_page(State(state): State<AppState>) -> impl IntoResponse {
    render_page("/".to_string(), state).await
}

pub async fn get_path(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    render_page(path, state).await
}

async fn render_page(mut relative_path_raw: String, state: AppState) -> impl IntoResponse {
    let root_dir = state.root_dir;
    let url_prefix = state.url_prefix;

    while let Some(new_relative_path) = relative_path_raw.strip_prefix('/') {
        relative_path_raw = new_relative_path.to_string();
    }

    relative_path_raw.insert(0, '/');

    let path = format!("{root_dir}{relative_path_raw}");
    let path = path::Path::new(&path);
    let relative_path = path::Path::new(&relative_path_raw);

    if is_path_ignored(relative_path, &state.exclude_files) {
        StatusCode::FORBIDDEN.into_response()
    } else if !path.exists() {
        (StatusCode::NOT_FOUND, "file not found".to_string()).into_response()
    } else if path.is_file() {
        let is_typst_file = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".typ");

        let file = if is_typst_file {
            Command::new("typst")
                .arg("compile")
                .arg(path)
                .args(["--root", &root_dir])
                .arg("./out/generated.pdf")
                .output()
                .await
                .unwrap();

            File::open("out/generated.pdf").await.unwrap()
        } else {
            File::open(path).await.unwrap()
        };

        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        let file_name = path.file_name().unwrap().to_str().unwrap();
        let content_disposition = format!("inline; filename=\"{file_name}\"",);

        let headers = [
            (
                header::CONTENT_TYPE,
                if is_typst_file {
                    "application/pdf"
                } else {
                    &mime_guess::from_path(path)
                        .first()
                        .unwrap_or(mime::TEXT_PLAIN)
                        .to_string()
                },
            ),
            (header::CONTENT_DISPOSITION, &content_disposition),
        ];

        (StatusCode::OK, headers, body).into_response()
    } else {
        let mut filesystem_objects = fs::read_dir(path).await.unwrap();

        let mut list = match relative_path.parent() {
            Some(parent) => format!(
                r#"<li><a href="{url_prefix}{}">..</a></li>"#,
                parent.to_str().unwrap()
            ),
            None => String::new(),
        };

        let mut dir_entries = Vec::new();

        while let Ok(Some(dir_entry)) = filesystem_objects.next_entry().await {
            dir_entries.push(dir_entry);
        }

        dir_entries.sort_by(|a, b| match (a.path().is_dir(), b.path().is_dir()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        });

        for dir_entry in dir_entries {
            let file_name = dir_entry.file_name().to_str().unwrap().to_string();

            if is_file_name_ignored(&file_name, &state.exclude_files) {
                continue;
            }

            let class = if dir_entry.path().is_file() {
                "file"
            } else {
                "dir"
            };

            list.push_str(format!(r#"<li class="{class}"><a href="{url_prefix}{relative_path_raw}{}{file_name}">{}{file_name}</a></li>"#, if relative_path_raw == "/" {""} else {"/"}, if class == "dir" {"📁 "} else {""}).as_str());
        }

        (
            StatusCode::OK,
            Html::from(format!(
                r#"
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta charset="utf-8">
    <link rel="stylesheet" href="/main.css">
</head>

<body>
    <h1 class="title">{relative_path_raw}</h1>

    <ul class="dir-list">
        {list}
    </ul>
</body>

</html>"#,
                if relative_path_raw.trim_matches('/').is_empty() {
                    "/"
                } else {
                    path.file_name().unwrap().to_str().unwrap()
                }
            )),
        )
            .into_response()
    }
}

/// Checks if any path segment is included in `excluded_files` or starts with a dot
fn is_path_ignored(file_path: &path::Path, excluded_files: &[String]) -> bool {
    let is_excluded = |name: &String| excluded_files.contains(name) || name.starts_with('.');

    for ancestor in file_path.ancestors() {
        let Some(ancestor_name) = ancestor.file_name() else {
            debug!("whoops");
            continue;
        };
        let ancestor_name = ancestor_name.to_str().unwrap().to_string();

        debug!("{ancestor_name}");

        if is_excluded(&ancestor_name) {
            return true;
        }
    }

    false
}

/// Checks if the file name is included in `excluded_files` or starts with a dot
fn is_file_name_ignored(file_name: &String, excluded_files: &[String]) -> bool {
    excluded_files.contains(file_name) || file_name.starts_with('.')
}
