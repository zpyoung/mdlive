#![allow(dead_code)]

use axum_test::TestServer;
use mdlive::{new_router, scan_markdown_files};
use std::fs;
use tempfile::{Builder, NamedTempFile, TempDir};

pub const FILE_WATCH_DELAY_MS: u64 = 100;
pub const WEBSOCKET_TIMEOUT_SECS: u64 = 5;

pub const TEST_FILE_1_CONTENT: &str = "# Test 1\n\nContent of test1";
pub const TEST_FILE_2_CONTENT: &str = "# Test 2\n\nContent of test2";
pub const TEST_FILE_3_CONTENT: &str = "# Test 3\n\nContent of test3";
pub const YAML_FRONTMATTER_CONTENT: &str =
    "---\ntitle: Test Post\nauthor: Name\n---\n\n# Test Post\n";
pub const TOML_FRONTMATTER_CONTENT: &str = "+++\ntitle = \"Test Post\"\n+++\n\n# Test Post\n";

fn create_test_server_impl(content: &str, use_http: bool) -> (TestServer, NamedTempFile) {
    let temp_file = Builder::new()
        .suffix(".md")
        .tempfile()
        .expect("Failed to create temp file");
    fs::write(&temp_file, content).expect("Failed to write temp file");

    let canonical_path = temp_file
        .path()
        .canonicalize()
        .unwrap_or_else(|_| temp_file.path().to_path_buf());

    let base_dir = canonical_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let tracked_files = vec![canonical_path];

    let router = new_router(base_dir, tracked_files, false).expect("Failed to create router");

    let server = if use_http {
        TestServer::builder()
            .http_transport()
            .build(router)
            .expect("Failed to create test server")
    } else {
        TestServer::new(router).expect("Failed to create test server")
    };

    (server, temp_file)
}

pub async fn create_test_server(content: &str) -> (TestServer, NamedTempFile) {
    create_test_server_impl(content, false)
}

pub async fn create_test_server_with_http(content: &str) -> (TestServer, NamedTempFile) {
    create_test_server_impl(content, true)
}

fn create_directory_server_impl(use_http: bool) -> (TestServer, TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("test1.md"), TEST_FILE_1_CONTENT)
        .expect("Failed to write test1.md");
    fs::write(temp_dir.path().join("test2.markdown"), TEST_FILE_2_CONTENT)
        .expect("Failed to write test2.markdown");
    fs::write(temp_dir.path().join("test3.md"), TEST_FILE_3_CONTENT)
        .expect("Failed to write test3.md");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = scan_markdown_files(&base_dir).expect("Failed to scan markdown files");

    let router = new_router(base_dir, tracked_files, true).expect("Failed to create router");

    let server = if use_http {
        TestServer::builder()
            .http_transport()
            .build(router)
            .expect("Failed to create test server")
    } else {
        TestServer::new(router).expect("Failed to create test server")
    };

    (server, temp_dir)
}

pub async fn create_directory_server() -> (TestServer, TempDir) {
    create_directory_server_impl(false)
}

pub async fn create_directory_server_with_http() -> (TestServer, TempDir) {
    create_directory_server_impl(true)
}
