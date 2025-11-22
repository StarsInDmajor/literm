pub mod fs;
pub mod hdf5;
pub mod login;

use crate::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(login::router())
        .merge(fs::router())
        .merge(hdf5::router())
}