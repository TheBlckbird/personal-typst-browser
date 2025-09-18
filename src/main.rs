use std::{env, process::exit};

use axum::{Router, http::header, response::IntoResponse, routing::get};
use dotenvy::dotenv;
use tokio::{fs, net::TcpListener};

use crate::typst::{get_path, root_page};

mod typst;

#[allow(unused)]
#[derive(Clone)]
struct AppState {
    pub root_dir: String,
    pub exclude_files: Vec<String>,
    pub url_prefix: String,
}

impl AppState {
    fn new(root_dir: String, exclude_files: Vec<String>, url_prefix: String) -> Self {
        Self {
            root_dir,
            exclude_files,
            url_prefix,
        }
    }
}

#[tokio::main]
async fn main() {
    let mut root_dir = None;
    let mut exclude_files_raw = None;
    let mut url_prefix = None;

    dotenv().unwrap();

    for (key, value) in env::vars() {
        match key.to_lowercase().as_str() {
            "root_dir" => root_dir = Some(value),
            "url_prefix" => url_prefix = Some(value),
            "exclude_files" => exclude_files_raw = Some(value),
            _ => continue,
        }
    }

    let mut exclude_files = Vec::new();

    if let Some(value) = exclude_files_raw {
        value
            .split(',')
            .for_each(|item| exclude_files.push(item.trim().to_owned()))
    }

    let state = if let Some(root_dir) = root_dir
        && let Some(url_prefix) = url_prefix
    {
        AppState::new(root_dir, exclude_files, url_prefix)
    } else {
        eprintln!("Missing either URL_PREFIX or ROOT_DIR in .env file!");
        exit(1);
    };

    let app = Router::new()
        .route("/", get(root_page))
        .route("/main.css", get(main_css))
        .route("/{*path}", get(get_path))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn main_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        fs::read_to_string("static/main.css").await.unwrap(),
    )
}
