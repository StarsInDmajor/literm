pub mod fs;
pub mod login;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new().merge(login::router()).merge(fs::router())
}
