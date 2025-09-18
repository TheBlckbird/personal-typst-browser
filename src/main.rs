use std::{env, error::Error, process::exit};

use axum::{Router, http::header, response::IntoResponse, routing::get};
use dotenvy::dotenv;
use log::{error, info};
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
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut root_dir = None;
    let mut url_prefix = None;

    dotenv()?;

    for (key, value) in env::vars() {
        match key.to_lowercase().as_str() {
            "root_dir" => root_dir = Some(value),
            "url_prefix" => url_prefix = Some(value),
            _ => continue,
        }

        if root_dir.is_some() && url_prefix.is_some() {
            break;
        }
    }

    let mut exclude_files = Vec::new();

    for file in env_or("exclude_files", "").split(',') {
        exclude_files.push(file.trim().to_owned())
    }

    let state = if let Some(root_dir) = root_dir
        && let Some(url_prefix) = url_prefix
    {
        AppState::new(root_dir, exclude_files, url_prefix)
    } else {
        error!("Missing either URL_PREFIX or ROOT_DIR in .env file!");
        exit(1);
    };

    let out_dir_name = env_or("OUT_DIR", "out");
    create_out_dir(out_dir_name).await;

    let app = Router::new()
        .route("/", get(root_page))
        .route("/main.css", get(main_css))
        .route("/{*path}", get(get_path))
        .with_state(state);

    let host = env_or("host", "0.0.0.0:3000");

    info!("Starting server on {host}");

    let listener = TcpListener::bind(host).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Tries to find an environment variable with the given name and returns its content as a string
/// or the provided default if it can't be found.
fn env_or(searched_name: &str, default: &str) -> String {
    env::vars()
        .find(|(var_name, _)| var_name.to_lowercase() == searched_name)
        .map(|(_, content)| content)
        .unwrap_or(default.to_string())
}

async fn main_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("./main.css"),
    )
}

/// Creates the output directory if it doesn't already exist
async fn create_out_dir(name: String) {
    fs::create_dir_all(name).await.unwrap();
}
