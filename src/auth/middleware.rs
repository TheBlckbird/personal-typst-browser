use axum::{extract::Request, middleware::Next, response::{IntoResponse, Redirect, Response}};
use tower_sessions::Session;

use crate::auth::IS_VERIFIED;

/// Middleware to check authentication
pub async fn authentication(session: Session, request: Request, next: Next) -> Response {
    if let Some(is_verified) = session.get(IS_VERIFIED).await.unwrap()
        && is_verified
    {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}
