use crate::{error::AppError, state::AppState};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue},
    routing::get,
    Json, Router,
};
use mime_guess::MimeGuess;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio_util::io::ReaderStream;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/fs/list", get(list_handler))
        .route("/api/fs/content", get(content_handler))
        .route("/api/fs/raw", get(raw_handler))
}

#[derive(Debug, Deserialize)]
pub struct FsListQuery {
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FsEntry {
    pub name: String,
    pub entry_type: String,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Serialize)]
pub struct FsListResponse {
    pub ok: bool,
    pub path: String,
    pub entries: Vec<FsEntry>,
}

pub async fn list_handler(
    State(state): State<AppState>,
    Query(query): Query<FsListQuery>,
) -> Result<Json<FsListResponse>, AppError> {
    let rel = query.path.unwrap_or_default();
    let resolved = state.fs.resolve_path(&rel)?;

    let mut entries_res = Vec::new();
    let mut dir = fs::read_dir(&resolved).await?;
    while let Some(entry) = dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        let file_type = if metadata.is_dir() { "dir" } else { "file" };
        let size = metadata.len();
        let mtime = metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let name = entry.file_name().to_string_lossy().to_string();

        entries_res.push(FsEntry {
            name,
            entry_type: file_type.to_string(),
            size,
            mtime,
        });
    }

    Ok(Json(FsListResponse {
        ok: true,
        path: rel,
        entries: entries_res,
    }))
}

#[derive(Debug, Deserialize)]
pub struct FsContentQuery {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct FsContentResponse {
    pub ok: bool,
    pub path: String,
    pub content: String,
}

pub async fn content_handler(
    State(state): State<AppState>,
    Query(query): Query<FsContentQuery>,
) -> Result<Json<FsContentResponse>, AppError> {
    let resolved = state.fs.resolve_path(&query.path)?;
    let content = fs::read_to_string(&resolved).await?;

    Ok(Json(FsContentResponse {
        ok: true,
        path: query.path,
        content,
    }))
}

pub async fn raw_handler(
    State(state): State<AppState>,
    Query(query): Query<FsContentQuery>,
) -> Result<(HeaderMap, Body), AppError> {
    let resolved = state.fs.resolve_path(&query.path)?;
    let file = fs::File::open(&resolved).await?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mime = MimeGuess::from_path(&resolved).first_or_octet_stream();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .map_err(|_| AppError::Internal("invalid content type".into()))?,
    );

    Ok((headers, body))
}
