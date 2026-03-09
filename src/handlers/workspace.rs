use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::task::AbortHandle;

use crate::state::SharedMarkdownState;
use crate::util::scan_supported_files;
use crate::watcher::start_watcher;

#[derive(Deserialize)]
pub(crate) struct SwitchRequest {
    path: String,
}

#[derive(Serialize)]
pub(crate) struct WorkspaceResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_count: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct RecentEntry {
    path: String,
    mode: String,
    last_opened: u64,
}

#[derive(Serialize)]
pub(crate) struct RecentResponse {
    success: bool,
    recent: Vec<RecentEntry>,
}

fn expand_path(raw: &str) -> PathBuf {
    let expanded = if let Some(rest) = raw.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            home.join(rest.strip_prefix('/').unwrap_or(rest))
        } else {
            PathBuf::from(raw)
        }
    } else {
        PathBuf::from(raw)
    };
    expanded
        .canonicalize()
        .unwrap_or_else(|_| std::path::absolute(&expanded).unwrap_or(expanded))
}

pub(crate) async fn api_workspace_switch(
    State(state): State<SharedMarkdownState>,
    Json(body): Json<SwitchRequest>,
) -> Result<Json<WorkspaceResponse>, (StatusCode, Json<WorkspaceResponse>)> {
    let target = expand_path(&body.path);

    let (base_dir, files, dir_mode) = if target.is_file() {
        let base = target
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();
        let base = base.canonicalize().unwrap_or(base);
        (base, vec![target], false)
    } else if target.is_dir() {
        let target = target.canonicalize().unwrap_or(target);
        let files = scan_supported_files(&target).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(WorkspaceResponse {
                    success: false,
                    error: Some(format!("failed to scan directory: {e}")),
                    base_dir: None,
                    mode: None,
                    file_count: None,
                }),
            )
        })?;
        if files.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(WorkspaceResponse {
                    success: false,
                    error: Some("no supported files found (.md, .txt, .json)".to_string()),
                    base_dir: None,
                    mode: None,
                    file_count: None,
                }),
            ));
        }
        (target, files, true)
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(WorkspaceResponse {
                success: false,
                error: Some("path does not exist".to_string()),
                base_dir: None,
                mode: None,
                file_count: None,
            }),
        ));
    };

    let file_count = files.len();
    let mode = if dir_mode { "directory" } else { "file" };
    let base_dir_display = base_dir.display().to_string();

    {
        let mut guard = state.lock().await;
        guard
            .switch_workspace(base_dir.clone(), files, dir_mode)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(WorkspaceResponse {
                        success: false,
                        error: Some(format!("failed to switch workspace: {e}")),
                        base_dir: None,
                        mode: None,
                        file_count: None,
                    }),
                )
            })?;
    }

    let abort_handle: AbortHandle = start_watcher(&base_dir, state.clone()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(WorkspaceResponse {
                success: false,
                error: Some(format!("failed to start file watcher: {e}")),
                base_dir: None,
                mode: None,
                file_count: None,
            }),
        )
    })?;

    {
        let mut guard = state.lock().await;
        guard.watcher_abort = Some(abort_handle);
    }

    Ok(Json(WorkspaceResponse {
        success: true,
        error: None,
        base_dir: Some(base_dir_display),
        mode: Some(mode.to_string()),
        file_count: Some(file_count),
    }))
}

pub(crate) async fn api_workspace_current(
    State(state): State<SharedMarkdownState>,
) -> Json<WorkspaceResponse> {
    let guard = state.lock().await;
    if guard.has_workspace() {
        let mode = if guard.is_directory_mode {
            "directory"
        } else {
            "file"
        };
        Json(WorkspaceResponse {
            success: true,
            error: None,
            base_dir: Some(guard.base_dir.display().to_string()),
            mode: Some(mode.to_string()),
            file_count: Some(guard.tracked_files.len()),
        })
    } else {
        Json(WorkspaceResponse {
            success: true,
            error: None,
            base_dir: None,
            mode: None,
            file_count: None,
        })
    }
}

pub(crate) async fn api_workspace_recent(
    State(state): State<SharedMarkdownState>,
) -> Json<RecentResponse> {
    let guard = state.lock().await;
    let recent = guard
        .config
        .as_ref()
        .map(|c| {
            c.recent
                .iter()
                .map(|r| RecentEntry {
                    path: r.path.clone(),
                    mode: r.mode.clone(),
                    last_opened: r.last_opened,
                })
                .collect()
        })
        .unwrap_or_default();

    Json(RecentResponse {
        success: true,
        recent,
    })
}
