pub mod system;
pub mod terminal;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(terminal::router())
        .merge(system::router())
}
