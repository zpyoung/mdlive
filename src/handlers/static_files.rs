use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::fs;

use crate::state::SharedMarkdownState;
use crate::template::{
    FAVICON_PNG, HIGHLIGHT_JS, MARKED_JS, MDLIVE_LOGO_PNG, MD_ICON_PNG, MERMAID_JS, STATIC_ETAG,
};
use crate::util::guess_image_content_type;

pub(crate) async fn serve_mermaid_js(headers: HeaderMap) -> impl IntoResponse {
    serve_embedded_js(&headers, MERMAID_JS)
}

pub(crate) async fn serve_highlight_js(headers: HeaderMap) -> impl IntoResponse {
    serve_embedded_js(&headers, HIGHLIGHT_JS)
}

pub(crate) async fn serve_marked_js(headers: HeaderMap) -> impl IntoResponse {
    serve_embedded_js(&headers, MARKED_JS)
}

fn serve_embedded_js(headers: &HeaderMap, content: &'static str) -> Response {
    let is_match = check_etag(headers);

    let response_headers = [
        (header::CONTENT_TYPE, "application/javascript"),
        (header::ETAG, STATIC_ETAG),
        (header::CACHE_CONTROL, "public, no-cache"),
    ];

    if is_match {
        (StatusCode::NOT_MODIFIED, response_headers).into_response()
    } else {
        (StatusCode::OK, response_headers, content).into_response()
    }
}

pub(crate) async fn serve_md_icon(headers: HeaderMap) -> impl IntoResponse {
    serve_embedded_image(&headers, MD_ICON_PNG)
}

pub(crate) async fn serve_favicon(headers: HeaderMap) -> impl IntoResponse {
    serve_embedded_image(&headers, FAVICON_PNG)
}

pub(crate) async fn serve_mdlive_logo(headers: HeaderMap) -> impl IntoResponse {
    serve_embedded_image(&headers, MDLIVE_LOGO_PNG)
}

fn serve_embedded_image(headers: &HeaderMap, content: &'static [u8]) -> Response {
    let is_match = check_etag(headers);

    let response_headers = [
        (header::CONTENT_TYPE, "image/png"),
        (header::ETAG, STATIC_ETAG),
        (header::CACHE_CONTROL, "public, no-cache"),
    ];

    if is_match {
        (StatusCode::NOT_MODIFIED, response_headers).into_response()
    } else {
        (StatusCode::OK, response_headers, content).into_response()
    }
}

fn check_etag(headers: &HeaderMap) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|etags| etags.split(',').any(|tag| tag.trim() == STATIC_ETAG))
}

pub(crate) async fn serve_static_file_inner(
    filename: String,
    state: SharedMarkdownState,
) -> axum::response::Response {
    let state = state.lock().await;

    let full_path = state.base_dir.join(&filename);

    match full_path.canonicalize() {
        Ok(canonical_path) => {
            if !canonical_path.starts_with(&state.base_dir) {
                return (
                    StatusCode::FORBIDDEN,
                    [(header::CONTENT_TYPE, "text/plain")],
                    "Access denied".to_string(),
                )
                    .into_response();
            }

            match fs::read(&canonical_path) {
                Ok(contents) => {
                    let content_type = guess_image_content_type(&filename);
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, content_type.as_str())],
                        contents,
                    )
                        .into_response()
                }
                Err(_) => (
                    StatusCode::NOT_FOUND,
                    [(header::CONTENT_TYPE, "text/plain")],
                    "File not found".to_string(),
                )
                    .into_response(),
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            "File not found".to_string(),
        )
            .into_response(),
    }
}
