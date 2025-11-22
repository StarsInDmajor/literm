use crate::{config::Config, fs::FsService, pty::PtyManager, session::SessionStore};
use anyhow::Context;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub fs: Arc<FsService>,
    pub pty: Arc<PtyManager>,
    pub sessions: SessionStore,
}

impl AppState {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let fs =
            FsService::new(&config.server.root_dir).context("failed to init filesystem service")?;
        let pty = PtyManager::new().context("failed to initialize PTY manager")?;
        let sessions = SessionStore::new(config.server.session_timeout_minutes);

        Ok(Self {
            config: Arc::new(config),
            fs: Arc::new(fs),
            pty: Arc::new(pty),
            sessions,
        })
    }
}
