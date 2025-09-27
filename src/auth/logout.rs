use axum::response::{IntoResponse, Redirect};
use tower_sessions::Session;

use crate::auth::IS_VERIFIED;

pub async fn logout(session: Session) -> impl IntoResponse {
    session.remove::<bool>(IS_VERIFIED).await.unwrap();
    Redirect::to("/")
}
