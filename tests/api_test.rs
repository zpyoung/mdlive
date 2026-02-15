mod common;

use common::*;
use std::fs;

#[tokio::test]
async fn test_api_raw_content() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server.get("/api/raw_content?path=test1.md").await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.header("content-type"), "text/plain; charset=utf-8");
    let body = response.text();
    assert_eq!(body, TEST_FILE_1_CONTENT);
}

#[tokio::test]
async fn test_api_raw_content_not_found() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server.get("/api/raw_content?path=nonexistent.md").await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_api_raw_content_traversal_blocked() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server
        .get("/api/raw_content?path=../../../etc/passwd")
        .await;
    assert_eq!(response.status_code(), 403);
}

#[tokio::test]
async fn test_api_delete_file() {
    let (server, temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/delete_file")
        .json(&serde_json::json!({"path": "test1.md"}))
        .await;
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);

    assert!(!temp_dir.path().join("test1.md").exists());

    let response = server.get("/test1.md").await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_api_delete_file_not_found() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/delete_file")
        .json(&serde_json::json!({"path": "nonexistent.md"}))
        .await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_api_delete_file_traversal_blocked() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/delete_file")
        .json(&serde_json::json!({"path": "../../../etc/passwd"}))
        .await;
    assert_eq!(response.status_code(), 403);
}

#[tokio::test]
async fn test_api_move_file() {
    let (server, temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/move_file")
        .json(&serde_json::json!({"path": "test1.md", "target": "renamed.md"}))
        .await;
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["path"], "renamed.md");

    assert!(!temp_dir.path().join("test1.md").exists());
    assert!(temp_dir.path().join("renamed.md").exists());

    let response = server.get("/renamed.md").await;
    assert_eq!(response.status_code(), 200);
    let html = response.text();
    assert!(html.contains("Test 1"));
}

#[tokio::test]
async fn test_api_move_file_to_subdirectory() {
    let (server, temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/move_file")
        .json(&serde_json::json!({"path": "test1.md", "target": "archive/test1.md"}))
        .await;
    assert_eq!(response.status_code(), 200);

    assert!(temp_dir.path().join("archive/test1.md").exists());
}

#[tokio::test]
async fn test_api_move_file_target_exists() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/move_file")
        .json(&serde_json::json!({"path": "test1.md", "target": "test3.md"}))
        .await;
    assert_eq!(response.status_code(), 409);
}

#[tokio::test]
async fn test_api_move_file_non_md_target() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/move_file")
        .json(&serde_json::json!({"path": "test1.md", "target": "test1.txt"}))
        .await;
    assert_eq!(response.status_code(), 400);
}

#[tokio::test]
async fn test_api_create_file() {
    let (server, temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/create_file")
        .json(&serde_json::json!({"path": "new_file.md"}))
        .await;
    assert_eq!(response.status_code(), 201);
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["path"], "new_file.md");

    let content = fs::read_to_string(temp_dir.path().join("new_file.md")).unwrap();
    assert!(content.contains("# new_file"));

    let response = server.get("/new_file.md").await;
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_api_create_file_with_content() {
    let (server, temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/create_file")
        .json(&serde_json::json!({"path": "custom.md", "content": "# Custom\n\nHello world"}))
        .await;
    assert_eq!(response.status_code(), 201);

    let content = fs::read_to_string(temp_dir.path().join("custom.md")).unwrap();
    assert_eq!(content, "# Custom\n\nHello world");
}

#[tokio::test]
async fn test_api_create_file_in_new_subdirectory() {
    let (server, temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/create_file")
        .json(&serde_json::json!({"path": "deep/nested/dir/new.md"}))
        .await;
    assert_eq!(response.status_code(), 201);

    assert!(temp_dir.path().join("deep/nested/dir/new.md").exists());
}

#[tokio::test]
async fn test_api_create_file_already_exists() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/create_file")
        .json(&serde_json::json!({"path": "test1.md"}))
        .await;
    assert_eq!(response.status_code(), 409);
}

#[tokio::test]
async fn test_api_create_file_non_md_extension() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server
        .post("/api/create_file")
        .json(&serde_json::json!({"path": "script.sh"}))
        .await;
    assert_eq!(response.status_code(), 400);
}
