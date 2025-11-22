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
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::select;
use tokio::sync::{mpsc, Mutex};

pub fn router() -> Router<AppState> {
    Router::new().route("/ws/system", get(ws_handler))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
enum SystemClientMessage {
    #[serde(rename = "watch")]
    Watch { path: String },
    #[serde(rename = "unwatch")]
    Unwatch { path: String },
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    tracing::info!("new system ws connection");

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let watch_enabled = state.config.features.enable_watch;
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let mut watcher = if watch_enabled {
        match RecommendedWatcher::new(
            move |res| {
                let _ = event_tx.send(res);
            },
            Config::default(),
        ) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                tracing::error!("failed to initialize watcher: {err:?}");
                send_error(&sender, "file watching unavailable").await;
                None
            }
        }
    } else {
        None
    };

    let mut tracked: HashMap<PathBuf, String> = HashMap::new();

    loop {
        select! {
            ws_msg = receiver.next() => {
                match ws_msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_client_message(&state, &sender, &mut watcher, &mut tracked, text, watch_enabled).await;
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = sender.lock().await.send(Message::Pong(payload)).await;
                    }
                    Some(Ok(Message::Close(_))) => {
                        break;
                    }
                    Some(Ok(Message::Binary(_))) => {}
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Err(err)) => {
                        tracing::warn!("system ws recv error: {err:?}");
                        break;
                    }
                    None => break,
                }
            }
            event = event_rx.recv(), if watch_enabled && watcher.is_some() => {
                match event {
                    Some(Ok(ev)) => {
                        forward_event(&state, &sender, &tracked, ev).await;
                    }
                    Some(Err(err)) => {
                        tracing::warn!("watcher error: {err:?}");
                        send_error(&sender, "watcher error").await;
                    }
                    None => break,
                }
            }
        }
    }

    tracing::info!("system ws connection ended");
}

async fn handle_client_message(
    state: &AppState,
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    watcher: &mut Option<RecommendedWatcher>,
    tracked: &mut HashMap<PathBuf, String>,
    payload: String,
    watch_enabled: bool,
) {
    let msg = match serde_json::from_str::<SystemClientMessage>(&payload) {
        Ok(m) => m,
        Err(err) => {
            tracing::warn!("invalid system payload: {err}");
            send_error(sender, "invalid payload").await;
            return;
        }
    };

    match msg {
        SystemClientMessage::Watch { path } => {
            if !watch_enabled {
                send_error(sender, "file watching disabled").await;
                return;
            }
            let resolved = match state.fs.resolve_path(&path) {
                Ok(p) => p,
                Err(err) => {
                    tracing::warn!("watch path rejected: {err}");
                    send_error(sender, "invalid path").await;
                    return;
                }
            };
            let Some(watcher_ref) = watcher.as_mut() else {
                send_error(sender, "watcher unavailable").await;
                return;
            };
            if let Err(err) = watcher_ref.watch(&resolved, RecursiveMode::NonRecursive) {
                tracing::error!("failed to watch {resolved:?}: {err:?}");
                send_error(sender, "watch failed").await;
                return;
            }
            tracked.insert(resolved, path.clone());
            send_json(sender, json!({"event":"watching","path":path})).await;
        }
        SystemClientMessage::Unwatch { path } => {
            let resolved = match state.fs.resolve_path(&path) {
                Ok(p) => p,
                Err(_) => return,
            };
            if let Some(watcher_ref) = watcher.as_mut() {
                if tracked.remove(&resolved).is_some() {
                    let _ = watcher_ref.unwatch(&resolved);
                }
            } else {
                send_error(sender, "watcher unavailable").await;
                return;
            }
            send_json(sender, json!({"event":"unwatched","path":path})).await;
        }
    }
}

async fn forward_event(
    state: &AppState,
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    tracked: &HashMap<PathBuf, String>,
    event: Event,
) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    for path in event.paths {
        let canonical = path.canonicalize().ok();
        let rel = canonical
            .as_ref()
            .and_then(|p| tracked.get(p).cloned())
            .or_else(|| canonical.as_ref().and_then(|p| state.fs.to_relative(p)))
            .or_else(|| tracked.get(&path).cloned())
            .or_else(|| state.fs.to_relative(&path));
        if let Some(path_str) = rel {
            send_json(
                sender,
                json!({"event":"change","path":path_str,"timestamp":timestamp}),
            )
            .await;
        }
    }
}

async fn send_error(sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>, message: &str) {
    send_json(sender, json!({"event":"error","message":message})).await;
}

async fn send_json(sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>, value: serde_json::Value) {
    let mut guard = sender.lock().await;
    let _ = guard.send(Message::Text(value.to_string())).await;
}
