use axum::{
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use minijinja::context;
use minijinja::value::Value;
use serde::Deserialize;
use std::fs;

use crate::state::{MarkdownState, SharedMarkdownState};
use crate::template::{template_env, TEMPLATE_NAME};
use crate::tree::build_file_tree;
use crate::util::is_image_file;

use super::static_files::serve_static_file_inner;

#[derive(Deserialize)]
pub(crate) struct NewFileQuery {
    #[serde(default)]
    dir: String,
}

pub(crate) async fn serve_html_root(State(state): State<SharedMarkdownState>) -> impl IntoResponse {
    let mut state = state.lock().await;

    let filename = match state.get_sorted_filenames().into_iter().next() {
        Some(name) => name,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("No files available to serve".to_string()),
            );
        }
    };

    let _ = state.refresh_file(&filename);

    render_markdown(&state, &filename).await
}

pub(crate) async fn serve_file(
    AxumPath(filepath): AxumPath<String>,
    State(state): State<SharedMarkdownState>,
) -> axum::response::Response {
    if filepath.ends_with(".md") || filepath.ends_with(".markdown") {
        let mut state = state.lock().await;

        if !state.tracked_files.contains_key(&filepath) {
            return (StatusCode::NOT_FOUND, Html("File not found".to_string())).into_response();
        }

        let _ = state.refresh_file(&filepath);

        let (status, html) = render_markdown(&state, &filepath).await;
        (status, html).into_response()
    } else if is_image_file(&filepath) {
        serve_static_file_inner(filepath, state).await
    } else {
        (StatusCode::NOT_FOUND, Html("File not found".to_string())).into_response()
    }
}

pub(crate) async fn render_markdown(
    state: &MarkdownState,
    current_file: &str,
) -> (StatusCode, Html<String>) {
    let env = template_env();
    let template = match env.get_template(TEMPLATE_NAME) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Template error: {e}")),
            );
        }
    };

    let (content, has_mermaid) = if let Some(tracked) = state.tracked_files.get(current_file) {
        let html = &tracked.html;
        let mermaid = html.contains(r#"class="language-mermaid""#);
        (Value::from_safe_string(html.clone()), mermaid)
    } else {
        return (StatusCode::NOT_FOUND, Html("File not found".to_string()));
    };

    let has_history = state.mdlive_dir.is_some();

    let rendered = if state.show_navigation() {
        let filenames = state.get_sorted_filenames();
        let tree = build_file_tree(&filenames);

        match template.render(context! {
            content => content,
            mermaid_enabled => has_mermaid,
            show_navigation => true,
            has_history => has_history,
            tree => tree,
            current_file => current_file,
        }) {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!("Rendering error: {e}")),
                );
            }
        }
    } else {
        match template.render(context! {
            content => content,
            mermaid_enabled => has_mermaid,
            show_navigation => false,
            has_history => has_history,
            current_file => current_file,
        }) {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!("Rendering error: {e}")),
                );
            }
        }
    };

    (StatusCode::OK, Html(rendered))
}

pub(crate) async fn serve_editor(
    AxumPath(filepath): AxumPath<String>,
    State(state): State<SharedMarkdownState>,
) -> impl IntoResponse {
    let state = state.lock().await;

    if !state.tracked_files.contains_key(&filepath) {
        return (StatusCode::NOT_FOUND, Html("File not found".to_string()));
    }

    let tracked = &state.tracked_files[&filepath];
    let raw_content = fs::read_to_string(&tracked.path).unwrap_or_default();

    render_editor(&state, &filepath, &raw_content, false)
}

pub(crate) async fn serve_new_file_editor(
    Query(params): Query<NewFileQuery>,
    State(state): State<SharedMarkdownState>,
) -> impl IntoResponse {
    let state = state.lock().await;

    if !state.is_directory_mode {
        return (
            StatusCode::BAD_REQUEST,
            Html("New file only available in directory mode".to_string()),
        );
    }

    let default_name = if params.dir.is_empty() {
        "new.md".to_string()
    } else {
        format!("{}/new.md", params.dir)
    };

    render_editor(&state, &default_name, "", true)
}

fn render_editor(
    state: &MarkdownState,
    current_file: &str,
    raw_content: &str,
    new_file_mode: bool,
) -> (StatusCode, Html<String>) {
    let env = template_env();
    let template = match env.get_template(TEMPLATE_NAME) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("Template error: {e}")),
            );
        }
    };

    let has_history = state.mdlive_dir.is_some() && !new_file_mode;

    let rendered = if state.show_navigation() {
        let filenames = state.get_sorted_filenames();
        let tree = build_file_tree(&filenames);

        match template.render(context! {
            editor_mode => true,
            new_file_mode => new_file_mode,
            raw_content => raw_content,
            current_file => current_file,
            has_history => has_history,
            show_navigation => true,
            tree => tree,
        }) {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!("Rendering error: {e}")),
                );
            }
        }
    } else {
        match template.render(context! {
            editor_mode => true,
            new_file_mode => new_file_mode,
            raw_content => raw_content,
            current_file => current_file,
            has_history => has_history,
            show_navigation => false,
        }) {
            Ok(r) => r,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!("Rendering error: {e}")),
                );
            }
        }
    };

    (StatusCode::OK, Html(rendered))
}
