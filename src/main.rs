use std::{
    env,
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::exit,
};

use axum::{Router, middleware, routing::get};
use dotenvy::dotenv;
use log::{error, info};
use surrealdb::engine::remote::ws::Client;
use time::Duration;
use tokio::{fs, net::TcpListener};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_surrealdb_store::SurrealSessionStore;

use crate::{
    auth::middleware::authentication, setup_surrealdb::setup_surrealdb, state::AppState, static_files::make_static_router, browser::{get_path, root_page}
};

mod auth;
mod setup_surrealdb;
mod state;
mod browser;
mod static_files;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv()?;

    let root_dir = env("root_dir");
    let url_prefix = env("url_prefix");
    let exclude_files = get_exclude_files();
    let host = env_or(
        "host",
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000),
    );
    let Some(db_address) = env("db_host") else {
        error!("Missing DB_HOST environment variable!");
        exit(1);
    };
    let Some(db_name) = env("db_name") else {
        error!("Missing DB_NAME environment variable!");
        exit(1);
    };
    let Some(db_namespace) = env("db_namespace") else {
        error!("Missing DB_NAMESPACE environment variable!");
        exit(1);
    };
    let Some(db_user) = env("db_user") else {
        error!("Missing DB_USER environment variable!");
        exit(1);
    };
    let Some(db_password) = env("db_pass") else {
        error!("Missing DB_PASS environment variable!");
        exit(1);
    };

    let state = make_state(root_dir, url_prefix, exclude_files);

    let out_dir_name = env_or("OUT_DIR", "out");
    create_out_dir(out_dir_name).await;

    let app = Router::new()
        .route("/", get(root_page))
        .route("/{*path}", get(get_path))
        .layer(middleware::from_fn(authentication))
        .merge(auth::get_router())
        .merge(make_static_router())
        .with_state(state)
        .layer(make_session_layer(db_address, db_user, db_password, db_namespace, db_name).await);

    info!("Starting server on {host}");

    let listener = TcpListener::bind(host).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn make_state(
    root_dir: Option<String>,
    url_prefix: Option<String>,
    exclude_files: Vec<String>,
) -> AppState {
    if let Some(root_dir) = root_dir
        && let Some(url_prefix) = url_prefix
    {
        AppState::new(root_dir, exclude_files, url_prefix)
    } else {
        error!("Missing either URL_PREFIX or ROOT_DIR in .env file!");
        exit(1);
    }
}

/// Tries to find an environment variable with the given name and returns its content as a string
/// or the provided default if it can't be found.
fn env_or(searched_name: &str, default: impl ToString) -> String {
    env::vars()
        .find(|(var_name, _)| var_name.to_lowercase() == searched_name)
        .map(|(_, content)| content)
        .unwrap_or(default.to_string())
}

/// Tries to find an environment variable with the given name and returns its content as a string
/// or None.
fn env(searched_name: &str) -> Option<String> {
    env::vars()
        .find(|(var_name, content)| var_name.to_lowercase() == searched_name && !content.is_empty())
        .map(|(_, content)| content)
}

/// Gets the list of files to exclude, set by the environment variable `EXCLUDE_FILES`
fn get_exclude_files() -> Vec<String> {
    let mut exclude_files = Vec::new();

    for file in env_or("exclude_files", "").split(',') {
        exclude_files.push(file.trim().to_owned())
    }

    exclude_files
}

/// Creates the output directory if it doesn't already exist
async fn create_out_dir(name: String) {
    fs::create_dir_all(name).await.unwrap();
}

/// Creates the session layer for `tower-sessions`
async fn make_session_layer(
    db_address: String,
    db_user: String,
    db_password: String,
    db_namespace: String,
    db_name: String,
) -> SessionManagerLayer<SurrealSessionStore<Client>> {
    let surreal_client = setup_surrealdb(db_address, db_user, db_password, db_namespace, db_name)
        .await
        .unwrap_or_else(|error| {
            error!("An error occurred while trying to connect to the database: {error}");
            exit(1);
        });

    let session_store = SurrealSessionStore::new(surreal_client, "sessions".to_string());
    SessionManagerLayer::new(session_store).with_expiry(Expiry::OnInactivity(Duration::days(365)))
}
