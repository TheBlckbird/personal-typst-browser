use axum::{response::{IntoResponse, Redirect}, Form};
use serde::Deserialize;
use tower_sessions::Session;

use crate::auth::IS_VERIFIED;

#[derive(Deserialize)]
pub struct LoginForm {
    password: String,
}

/// Handle the login
pub async fn login(session: Session, Form(login_form): Form<LoginForm>) -> impl IntoResponse {
    if login_form.password == "ok" {
        session.insert(IS_VERIFIED, true).await.unwrap();
        Redirect::to("/")
    } else {
        Redirect::to("/login")
    }
}
