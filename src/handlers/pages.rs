use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use minijinja::context;
use minijinja::value::Value;

use crate::state::{MarkdownState, SharedMarkdownState};
use crate::template::{template_env, TEMPLATE_NAME};
use crate::tree::build_file_tree;
use crate::util::is_image_file;

use super::static_files::serve_static_file_inner;

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

    let rendered = if state.show_navigation() {
        let filenames = state.get_sorted_filenames();
        let tree = build_file_tree(&filenames);

        match template.render(context! {
            content => content,
            mermaid_enabled => has_mermaid,
            show_navigation => true,
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
