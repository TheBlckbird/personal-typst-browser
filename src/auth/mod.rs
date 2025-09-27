use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use tokio::fs;

use crate::{
    auth::{login::login, logout::logout},
};

const IS_VERIFIED: &str = "is_verified";

mod login;
mod logout;
pub mod middleware;

/// Creates and returns the router for the authentication pages
pub fn get_router<T>() -> Router<T>
where
    T: Send + Sync + Clone + 'static,
{
    Router::new()
        .route(
            "/login",
            get(|| async { Html::from(fs::read_to_string("pages/login.html").await.unwrap()) }),
        )
        .route("/login", post(login))
        .route("/logout", post(logout))
}
