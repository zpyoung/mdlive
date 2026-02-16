use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::{ServerMessage, SharedMarkdownState};
use crate::util::is_markdown_file;

#[derive(Deserialize)]
pub(crate) struct RawContentQuery {
    path: String,
}

#[derive(Deserialize)]
pub(crate) struct DeleteFileRequest {
    path: String,
}

#[derive(Deserialize)]
pub(crate) struct MoveFileRequest {
    path: String,
    target: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateFileRequest {
    path: String,
    content: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SaveFileRequest {
    path: String,
    content: String,
}

#[derive(Deserialize)]
pub(crate) struct FileHistoryQuery {
    path: String,
}

#[derive(Deserialize)]
pub(crate) struct RestoreVersionRequest {
    path: String,
    timestamp: String,
}

#[derive(Serialize)]
pub(crate) struct HistoryEntry {
    timestamp: String,
    preview: String,
}

#[derive(Serialize)]
pub(crate) struct HistoryResponse {
    success: bool,
    entries: Vec<HistoryEntry>,
}

#[derive(Serialize)]
pub(crate) struct RestoreResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ApiResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

fn validate_existing_path(
    base_dir: &Path,
    relative: &str,
) -> Result<PathBuf, (StatusCode, Json<ApiResponse>)> {
    if relative.contains("..") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                error: Some("path traversal not allowed".to_string()),
                path: None,
            }),
        ));
    }
    let full = base_dir.join(relative);
    match full.canonicalize() {
        Ok(canonical) => {
            if !canonical.starts_with(base_dir) {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse {
                        success: false,
                        error: Some("path outside base directory".to_string()),
                        path: None,
                    }),
                ))
            } else {
                Ok(canonical)
            }
        }
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                error: Some("file not found".to_string()),
                path: None,
            }),
        )),
    }
}

fn validate_new_path(
    base_dir: &Path,
    relative: &str,
) -> Result<PathBuf, (StatusCode, Json<ApiResponse>)> {
    if relative.contains("..") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                error: Some("path traversal not allowed".to_string()),
                path: None,
            }),
        ));
    }
    let full = base_dir.join(relative);
    if !is_markdown_file(&full) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                error: Some("only .md and .markdown extensions allowed".to_string()),
                path: None,
            }),
        ));
    }
    Ok(full)
}

pub(crate) async fn api_raw_content(
    Query(params): Query<RawContentQuery>,
    State(state): State<SharedMarkdownState>,
) -> Result<Response, (StatusCode, Json<ApiResponse>)> {
    let state = state.lock().await;
    validate_existing_path(&state.base_dir, &params.path)?;

    match state.tracked_files.get(&params.path) {
        Some(tracked) => {
            let content = fs::read_to_string(&tracked.path).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        error: Some("failed to read file".to_string()),
                        path: None,
                    }),
                )
            })?;
            Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                content,
            )
                .into_response())
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                error: Some("file not tracked".to_string()),
                path: None,
            }),
        )),
    }
}

pub(crate) async fn api_delete_file(
    State(state): State<SharedMarkdownState>,
    Json(body): Json<DeleteFileRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    let mut state = state.lock().await;
    let canonical = validate_existing_path(&state.base_dir, &body.path)?;

    if !state.tracked_files.contains_key(&body.path) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                error: Some("file not tracked".to_string()),
                path: None,
            }),
        ));
    }

    fs::remove_file(&canonical).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                error: Some(format!("failed to delete: {e}")),
                path: None,
            }),
        )
    })?;

    state.remove_tracked_file(&body.path);
    let _ = state.change_tx.send(ServerMessage::Reload);

    Ok(Json(ApiResponse {
        success: true,
        error: None,
        path: None,
    }))
}

pub(crate) async fn api_move_file(
    State(state): State<SharedMarkdownState>,
    Json(body): Json<MoveFileRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    let mut state = state.lock().await;
    let source_canonical = validate_existing_path(&state.base_dir, &body.path)?;
    let target_full = validate_new_path(&state.base_dir, &body.target)?;

    if !state.tracked_files.contains_key(&body.path) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                error: Some("file not tracked".to_string()),
                path: None,
            }),
        ));
    }

    if target_full.exists() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse {
                success: false,
                error: Some("target already exists".to_string()),
                path: None,
            }),
        ));
    }

    if let Some(parent) = target_full.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    error: Some(format!("failed to create directories: {e}")),
                    path: None,
                }),
            )
        })?;
    }

    fs::rename(&source_canonical, &target_full).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                error: Some(format!("failed to move: {e}")),
                path: None,
            }),
        )
    })?;

    state.remove_tracked_file(&body.path);
    let canonical_target = target_full.canonicalize().unwrap_or(target_full);
    let _ = state.add_tracked_file(canonical_target);
    let _ = state.change_tx.send(ServerMessage::Reload);

    Ok(Json(ApiResponse {
        success: true,
        error: None,
        path: Some(body.target),
    }))
}

pub(crate) async fn api_create_file(
    State(state): State<SharedMarkdownState>,
    Json(body): Json<CreateFileRequest>,
) -> Result<(StatusCode, Json<ApiResponse>), (StatusCode, Json<ApiResponse>)> {
    let mut state = state.lock().await;
    let target_full = validate_new_path(&state.base_dir, &body.path)?;

    if target_full.exists() {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiResponse {
                success: false,
                error: Some("file already exists".to_string()),
                path: None,
            }),
        ));
    }

    if let Some(parent) = target_full.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    error: Some(format!("failed to create directories: {e}")),
                    path: None,
                }),
            )
        })?;
    }

    let filename_stem = target_full
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("new");
    let content = body
        .content
        .unwrap_or_else(|| format!("# {}\n", filename_stem));

    fs::write(&target_full, &content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                error: Some(format!("failed to create file: {e}")),
                path: None,
            }),
        )
    })?;

    let canonical_target = target_full.canonicalize().unwrap_or(target_full);
    let _ = state.add_tracked_file(canonical_target);
    let _ = state.change_tx.send(ServerMessage::Reload);

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            success: true,
            error: None,
            path: Some(body.path),
        }),
    ))
}

pub(crate) async fn api_save_file(
    State(state): State<SharedMarkdownState>,
    Json(body): Json<SaveFileRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    let mut state = state.lock().await;
    let canonical = validate_existing_path(&state.base_dir, &body.path)?;

    if !state.tracked_files.contains_key(&body.path) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                error: Some("file not tracked".to_string()),
                path: None,
            }),
        ));
    }

    // snapshot current content before overwriting
    if let Some(mdlive_dir) = &state.mdlive_dir {
        snapshot_file(&canonical, &body.path, mdlive_dir);
    }

    fs::write(&canonical, &body.content).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                error: Some(format!("failed to write: {e}")),
                path: None,
            }),
        )
    })?;

    let _ = state.refresh_file(&body.path);
    let _ = state.change_tx.send(ServerMessage::Reload);

    Ok(Json(ApiResponse {
        success: true,
        error: None,
        path: Some(body.path),
    }))
}

fn snapshot_file(canonical: &Path, relative: &str, mdlive_dir: &Path) {
    let Ok(old_content) = fs::read_to_string(canonical) else {
        return;
    };

    let history_dir = mdlive_dir.join("history").join(relative);
    if fs::create_dir_all(&history_dir).is_err() {
        return;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let snapshot_path = history_dir.join(format!("{timestamp}.md"));
    let _ = fs::write(snapshot_path, old_content);

    // prune to 20 snapshots
    if let Ok(entries) = fs::read_dir(&history_dir) {
        let mut files: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
            .collect();
        if files.len() > 20 {
            files.sort();
            for old in &files[..files.len() - 20] {
                let _ = fs::remove_file(old);
            }
        }
    }
}

pub(crate) async fn api_file_history(
    Query(params): Query<FileHistoryQuery>,
    State(state): State<SharedMarkdownState>,
) -> Result<Json<HistoryResponse>, (StatusCode, Json<ApiResponse>)> {
    let state = state.lock().await;
    validate_existing_path(&state.base_dir, &params.path)?;

    let mdlive_dir = match &state.mdlive_dir {
        Some(d) => d,
        None => {
            return Ok(Json(HistoryResponse {
                success: true,
                entries: vec![],
            }));
        }
    };

    let history_dir = mdlive_dir.join("history").join(&params.path);
    let mut entries = Vec::new();

    if let Ok(dir_entries) = fs::read_dir(&history_dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            let timestamp = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let preview = fs::read_to_string(&path)
                .unwrap_or_default()
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(80)
                .collect();
            entries.push(HistoryEntry { timestamp, preview });
        }
    }

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(Json(HistoryResponse {
        success: true,
        entries,
    }))
}

pub(crate) async fn api_restore_version(
    State(state): State<SharedMarkdownState>,
    Json(body): Json<RestoreVersionRequest>,
) -> Result<Json<RestoreResponse>, (StatusCode, Json<ApiResponse>)> {
    let state = state.lock().await;
    validate_existing_path(&state.base_dir, &body.path)?;

    let mdlive_dir = match &state.mdlive_dir {
        Some(d) => d,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    error: Some("history not available".to_string()),
                    path: None,
                }),
            ));
        }
    };

    let snapshot_path = mdlive_dir
        .join("history")
        .join(&body.path)
        .join(format!("{}.md", body.timestamp));

    let content = fs::read_to_string(&snapshot_path).map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                error: Some("version not found".to_string()),
                path: None,
            }),
        )
    })?;

    Ok(Json(RestoreResponse {
        success: true,
        content: Some(content),
        error: None,
    }))
}
