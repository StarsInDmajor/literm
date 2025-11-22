use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{
    stream::{SplitSink, StreamExt},
    SinkExt,
};
use std::io::Read;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub fn router() -> Router<AppState> {
    Router::new().route("/ws/term", get(ws_handler))
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    tracing::info!("new terminal ws connection");

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // Initialize PTY session immediately with default size (will be resized by client shortly)
    let (active_session, reader) = match state.pty.create_session(24, 80) {
        Ok(pair) => pair,
        Err(err) => {
            tracing::error!("failed to create pty session: {err:?}");
            let _ = sender.lock().await.send(Message::Close(None)).await;
            return;
        }
    };
    
    let session_id = active_session.id();
    let reader_task = spawn_reader_task(reader, sender.clone());

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Binary(bytes) if !bytes.is_empty() => {
                match bytes[0] {
                    // 0x01: Input (Stdin)
                    0x01 => {
                        if bytes.len() > 1 {
                            if let Err(err) = active_session.write(&bytes[1..]).await {
                                tracing::error!("pty write failed: {err:?}");
                            }
                        }
                    }
                    // 0x02: Resize
                    0x02 => {
                        if bytes.len() >= 5 {
                            // Parse u16 big-endian
                            let rows = u16::from_be_bytes([bytes[1], bytes[2]]);
                            let cols = u16::from_be_bytes([bytes[3], bytes[4]]);
                            if let Err(err) = active_session.resize(rows, cols).await {
                                tracing::warn!("pty resize failed: {err:?}");
                            }
                        }
                    }
                    _ => {}
                }
            }
            Message::Close(_) => {
                break;
            }
            Message::Ping(payload) => {
                let _ = sender.lock().await.send(Message::Pong(payload)).await;
            }
            _ => {}
        }
    }

    active_session.shutdown().await;
    reader_task.abort();

    tracing::info!("terminal ws connection ended (session: {})", session_id);
}

fn spawn_reader_task(
    mut reader: Box<dyn Read + Send>,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut buffer = vec![0u8; 4096];
        loop {
            let read_res = tokio::task::block_in_place(|| reader.read(&mut buffer));
            match read_res {
                Ok(0) => {
                    // EOF: Shell exited. Close the WebSocket to notify client.
                    let mut guard = sender.lock().await;
                    let _ = guard.send(Message::Close(None)).await;
                    break;
                }
                Ok(n) => {
                    // Send raw binary data directly
                    let mut guard = sender.lock().await;
                    if guard.send(Message::Binary(buffer[..n].to_vec())).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    tracing::warn!("pty read failed: {err:?}");
                    break;
                }
            }
        }
    })
}

