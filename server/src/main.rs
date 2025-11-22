mod config;
mod error;
mod fs;
mod http;
mod pty;
mod session;
mod state;
mod ws;

use crate::config::Config;
use crate::state::AppState;
use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "literm_server=debug,tower_http=debug,axum=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config_path =
        std::env::var("LITETERM_CONFIG").unwrap_or_else(|_| "config/config.toml".to_string());
    let config = Config::from_file(config_path)?;
    let app_state = AppState::new(config)?;

    let app = Router::new()
        .merge(http::router())
        .merge(ws::router())
        .with_state(app_state.clone())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from((
        app_state
            .config
            .server
            .bind_addr
            .parse::<std::net::IpAddr>()?,
        app_state.config.server.port,
    ));

    tracing::info!("Listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}
