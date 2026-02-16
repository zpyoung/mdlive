use notify::Event;
use std::path::Path;

use crate::state::{ServerMessage, SharedMarkdownState};
use crate::util::{is_image_file, is_markdown_file};

fn is_mdlive_path(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == ".mdlive")
}

pub(crate) async fn handle_file_event(event: Event, state: &SharedMarkdownState) {
    if event.paths.iter().any(|p| is_mdlive_path(p)) {
        return;
    }
    match event.kind {
        notify::EventKind::Modify(notify::event::ModifyKind::Name(rename_mode)) => {
            use notify::event::RenameMode;
            match rename_mode {
                RenameMode::Both => {
                    if event.paths.len() == 2 {
                        let new_path = &event.paths[1];
                        handle_markdown_file_change(new_path, state).await;
                    }
                }
                RenameMode::From => {}
                RenameMode::To => {
                    if let Some(path) = event.paths.first() {
                        handle_markdown_file_change(path, state).await;
                    }
                }
                RenameMode::Any => {
                    if let Some(path) = event.paths.first() {
                        if path.exists() {
                            handle_markdown_file_change(path, state).await;
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {
            for path in &event.paths {
                if is_markdown_file(path) {
                    match event.kind {
                        notify::EventKind::Create(_)
                        | notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                            handle_markdown_file_change(path, state).await;
                        }
                        notify::EventKind::Remove(_) => {
                            // don't remove files from tracking -- editors like neovim save by
                            // renaming to a backup then creating a new file. removing here
                            // would cause transient 404s.
                        }
                        _ => {}
                    }
                } else if path.is_file() && is_image_file(path.to_str().unwrap_or("")) {
                    match event.kind {
                        notify::EventKind::Modify(_)
                        | notify::EventKind::Create(_)
                        | notify::EventKind::Remove(_) => {
                            let state_guard = state.lock().await;
                            let _ = state_guard.change_tx.send(ServerMessage::Reload);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

async fn handle_markdown_file_change(path: &Path, state: &SharedMarkdownState) {
    if !is_markdown_file(path) {
        return;
    }

    let mut state_guard = state.lock().await;

    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let key = canonical
        .strip_prefix(&state_guard.base_dir)
        .unwrap_or(&canonical)
        .to_string_lossy()
        .to_string();

    if state_guard.tracked_files.contains_key(&key) {
        if state_guard.refresh_file(&key).is_ok() {
            let _ = state_guard.change_tx.send(ServerMessage::Reload);
        }
    } else if state_guard.is_directory_mode && state_guard.add_tracked_file(canonical).is_ok() {
        let _ = state_guard.change_tx.send(ServerMessage::Reload);
    }
}
