use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct PtyManager {
    system: StdMutex<Box<dyn PtySystem + Send>>,
}

pub struct PtySession {
    id: Uuid,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send>>>,
}

impl PtyManager {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            system: StdMutex::new(native_pty_system()),
        })
    }

    pub fn create_session(
        &self,
        rows: u16,
        cols: u16,
    ) -> anyhow::Result<(PtySession, Box<dyn Read + Send>)> {
        let mut size = PtySize::default();
        size.rows = rows.max(1);
        size.cols = cols.max(1);
        let system = self.system.lock().expect("pty system mutex poisoned");
        let pair = system.openpty(size)?;
        let portable_pty::PtyPair { master, slave } = pair;

        let shell = std::env::var("SHELL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "/bin/bash".into());
        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");
        let child = slave.spawn_command(cmd)?;
        let reader = master.try_clone_reader()?;
        let writer = master.take_writer()?;

        Ok((
            PtySession {
                id: Uuid::new_v4(),
                master: Arc::new(Mutex::new(master)),
                writer: Arc::new(Mutex::new(writer)),
                child: Arc::new(Mutex::new(child)),
            },
            reader,
        ))
    }
}

impl PtySession {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn write(&self, data: &[u8]) -> anyhow::Result<()> {
        let mut writer = self.writer.lock().await;
        let payload = data.to_vec();
        tokio::task::block_in_place(move || {
            writer.write_all(&payload)?;
            writer.flush()?;
            Ok::<_, std::io::Error>(())
        })?;
        Ok(())
    }

    pub async fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
        let master = self.master.lock().await;
        tokio::task::block_in_place(move || {
            master.resize(PtySize {
                rows: rows.max(1),
                cols: cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
        })?;
        Ok(())
    }

    pub async fn shutdown(&self) {
        let mut child = self.child.lock().await;
        let _ = tokio::task::block_in_place(|| child.kill());
    }
}
