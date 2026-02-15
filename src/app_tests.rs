use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_is_markdown_file() {
    assert!(is_markdown_file(Path::new("test.md")));
    assert!(is_markdown_file(Path::new("/path/to/file.md")));

    assert!(is_markdown_file(Path::new("test.markdown")));
    assert!(is_markdown_file(Path::new("/path/to/file.markdown")));

    assert!(is_markdown_file(Path::new("test.MD")));
    assert!(is_markdown_file(Path::new("test.Md")));
    assert!(is_markdown_file(Path::new("test.MARKDOWN")));
    assert!(is_markdown_file(Path::new("test.MarkDown")));

    assert!(!is_markdown_file(Path::new("test.txt")));
    assert!(!is_markdown_file(Path::new("test.rs")));
    assert!(!is_markdown_file(Path::new("test.html")));
    assert!(!is_markdown_file(Path::new("test")));
    assert!(!is_markdown_file(Path::new("README")));
}

#[test]
fn test_is_image_file() {
    assert!(is_image_file("test.png"));
    assert!(is_image_file("test.jpg"));
    assert!(is_image_file("test.jpeg"));
    assert!(is_image_file("test.gif"));
    assert!(is_image_file("test.svg"));
    assert!(is_image_file("test.webp"));
    assert!(is_image_file("test.bmp"));
    assert!(is_image_file("test.ico"));

    assert!(is_image_file("test.PNG"));
    assert!(is_image_file("test.JPG"));
    assert!(is_image_file("test.JPEG"));

    assert!(is_image_file("/path/to/image.png"));
    assert!(is_image_file("./images/photo.jpg"));

    assert!(!is_image_file("test.txt"));
    assert!(!is_image_file("test.md"));
    assert!(!is_image_file("test.rs"));
    assert!(!is_image_file("test"));
}

#[test]
fn test_guess_image_content_type() {
    assert_eq!(guess_image_content_type("test.png"), "image/png");
    assert_eq!(guess_image_content_type("test.jpg"), "image/jpeg");
    assert_eq!(guess_image_content_type("test.jpeg"), "image/jpeg");
    assert_eq!(guess_image_content_type("test.gif"), "image/gif");
    assert_eq!(guess_image_content_type("test.svg"), "image/svg+xml");
    assert_eq!(guess_image_content_type("test.webp"), "image/webp");
    assert_eq!(guess_image_content_type("test.bmp"), "image/bmp");
    assert_eq!(guess_image_content_type("test.ico"), "image/x-icon");

    assert_eq!(guess_image_content_type("test.PNG"), "image/png");
    assert_eq!(guess_image_content_type("test.JPG"), "image/jpeg");

    assert_eq!(
        guess_image_content_type("test.xyz"),
        "application/octet-stream"
    );
    assert_eq!(guess_image_content_type("test"), "application/octet-stream");
}

#[test]
fn test_scan_markdown_files_empty_directory() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    let result = scan_markdown_files(temp_dir.path()).expect("Failed to scan");
    assert_eq!(result.len(), 0);
}

#[test]
fn test_scan_markdown_files_with_markdown_files() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("test1.md"), "# Test 1").expect("Failed to write");
    fs::write(temp_dir.path().join("test2.markdown"), "# Test 2").expect("Failed to write");
    fs::write(temp_dir.path().join("test3.md"), "# Test 3").expect("Failed to write");

    fs::write(temp_dir.path().join("test.txt"), "text").expect("Failed to write");
    fs::write(temp_dir.path().join("README"), "readme").expect("Failed to write");

    let result = scan_markdown_files(temp_dir.path()).expect("Failed to scan");

    assert_eq!(result.len(), 3);

    let filenames: Vec<_> = result
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert_eq!(filenames, vec!["test1.md", "test2.markdown", "test3.md"]);
}

#[test]
fn test_scan_markdown_files_includes_subdirectories() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("root.md"), "# Root").expect("Failed to write");

    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).expect("Failed to create subdir");
    fs::write(sub_dir.join("nested.md"), "# Nested").expect("Failed to write");

    let result = scan_markdown_files(temp_dir.path()).expect("Failed to scan");

    assert_eq!(result.len(), 2);
    let filenames: Vec<_> = result
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(filenames.contains(&"root.md"));
    assert!(filenames.contains(&"nested.md"));
}

#[test]
fn test_scan_markdown_files_case_insensitive() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("test1.md"), "# Test 1").expect("Failed to write");
    fs::write(temp_dir.path().join("test2.MD"), "# Test 2").expect("Failed to write");
    fs::write(temp_dir.path().join("test3.Md"), "# Test 3").expect("Failed to write");
    fs::write(temp_dir.path().join("test4.MARKDOWN"), "# Test 4").expect("Failed to write");

    let result = scan_markdown_files(temp_dir.path()).expect("Failed to scan");

    assert_eq!(result.len(), 4);
}

#[test]
fn test_format_host() {
    assert_eq!(format_host("127.0.0.1", 3000), "127.0.0.1:3000");
    assert_eq!(format_host("192.168.1.1", 8080), "192.168.1.1:8080");

    assert_eq!(format_host("localhost", 3000), "localhost:3000");
    assert_eq!(format_host("example.com", 80), "example.com:80");

    assert_eq!(format_host("::1", 3000), "[::1]:3000");
    assert_eq!(format_host("2001:db8::1", 8080), "[2001:db8::1]:8080");
}

#[test]
fn test_browsable_host() {
    assert_eq!(browsable_host("0.0.0.0"), "127.0.0.1");
    assert_eq!(browsable_host("::"), "::1");
    assert_eq!(browsable_host("127.0.0.1"), "127.0.0.1");
    assert_eq!(browsable_host("::1"), "::1");
    assert_eq!(browsable_host("192.168.1.1"), "192.168.1.1");
    assert_eq!(browsable_host("localhost"), "localhost");
    assert_eq!(browsable_host("example.com"), "example.com");
}

use axum_test::TestServer;
use std::time::Duration;
use tempfile::{Builder, NamedTempFile, TempDir};

const FILE_WATCH_DELAY_MS: u64 = 100;
const WEBSOCKET_TIMEOUT_SECS: u64 = 5;

const TEST_FILE_1_CONTENT: &str = "# Test 1\n\nContent of test1";
const TEST_FILE_2_CONTENT: &str = "# Test 2\n\nContent of test2";
const TEST_FILE_3_CONTENT: &str = "# Test 3\n\nContent of test3";
const YAML_FRONTMATTER_CONTENT: &str =
    "---\ntitle: Test Post\nauthor: Name\n---\n\n# Test Post\n";
const TOML_FRONTMATTER_CONTENT: &str = "+++\ntitle = \"Test Post\"\n+++\n\n# Test Post\n";

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
    let is_directory_mode = false;

    let router = new_router(base_dir, tracked_files, is_directory_mode)
        .expect("Failed to create router");

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

async fn create_test_server(content: &str) -> (TestServer, NamedTempFile) {
    create_test_server_impl(content, false)
}

async fn create_test_server_with_http(content: &str) -> (TestServer, NamedTempFile) {
    create_test_server_impl(content, true)
}

fn create_directory_server_impl(use_http: bool) -> (TestServer, TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("test1.md"), TEST_FILE_1_CONTENT)
        .expect("Failed to write test1.md");
    fs::write(temp_dir.path().join("test2.markdown"), TEST_FILE_2_CONTENT)
        .expect("Failed to write test2.markdown");
    fs::write(temp_dir.path().join("test3.md"), TEST_FILE_3_CONTENT)
        .expect("Failed to write test3.md");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = scan_markdown_files(&base_dir).expect("Failed to scan markdown files");
    let is_directory_mode = true;

    let router = new_router(base_dir, tracked_files, is_directory_mode)
        .expect("Failed to create router");

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

async fn create_directory_server() -> (TestServer, TempDir) {
    create_directory_server_impl(false)
}

async fn create_directory_server_with_http() -> (TestServer, TempDir) {
    create_directory_server_impl(true)
}

#[tokio::test]
async fn test_server_starts_and_serves_basic_markdown() {
    let (server, _temp_file) =
        create_test_server("# Hello World\n\nThis is **bold** text.").await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(body.contains("<h1>Hello World</h1>"));
    assert!(body.contains("<strong>bold</strong>"));
    assert!(body.contains("theme-toggle"));
    assert!(body.contains("openThemeModal"));
    assert!(body.contains("--bg-color"));
    assert!(body.contains("data-theme=\"dark\""));
}

#[tokio::test]
async fn test_websocket_connection() {
    let (server, _temp_file) = create_test_server_with_http("# WebSocket Test").await;

    let response = server.get_websocket("/ws").await;
    response.assert_status_switching_protocols();
}

#[tokio::test]
async fn test_file_modification_updates_via_websocket() {
    let (server, temp_file) = create_test_server_with_http("# Original Content").await;

    let mut websocket = server.get_websocket("/ws").await.into_websocket().await;

    fs::write(&temp_file, "# Modified Content").expect("Failed to modify file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let update_result = tokio::time::timeout(
        Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
        websocket.receive_json::<ServerMessage>(),
    )
    .await;

    match update_result {
        Ok(update_message) => {
            if let ServerMessage::Reload = update_message {
                // Success
            } else {
                panic!("Expected Reload message after file modification");
            }
        }
        Err(_) => {
            panic!("Timeout waiting for WebSocket update after file modification");
        }
    }
}

#[tokio::test]
async fn test_server_handles_gfm_features() {
    let markdown_content = r#"# GFM Test

## Table
| Name | Age |
|------|-----|
| John | 30  |
| Jane | 25  |

## Strikethrough
~~deleted text~~

## Code block
```rust
fn main() {
    println!("Hello!");
}
```
"#;

    let (server, _temp_file) = create_test_server(markdown_content).await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(body.contains("<table>"));
    assert!(body.contains("<th>Name</th>"));
    assert!(body.contains("<td>John</td>"));
    assert!(body.contains("<del>deleted text</del>"));
    assert!(body.contains("<pre>"));
    assert!(body.contains("fn main()"));
}

#[tokio::test]
async fn test_404_for_unknown_routes() {
    let (server, _temp_file) = create_test_server("# 404 Test").await;

    let response = server.get("/unknown-route").await;

    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_image_serving() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    let md_content =
        "# Test with Image\n\n![Test Image](test.png)\n\nThis markdown references an image.";
    let md_path = temp_dir.path().join("test.md");
    fs::write(&md_path, md_content).expect("Failed to write markdown file");

    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
        0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
        0xD7, 0x63, 0xF8, 0x0F, 0x00, 0x00, 0x01, 0x00, 0x01, 0x5C, 0xDD, 0x8D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let img_path = temp_dir.path().join("test.png");
    fs::write(&img_path, png_data).expect("Failed to write image file");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = vec![md_path];
    let is_directory_mode = false;
    let router = new_router(base_dir, tracked_files, is_directory_mode)
        .expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    let response = server.get("/").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();
    assert!(body.contains("<img src=\"test.png\" alt=\"Test Image\""));

    let img_response = server.get("/test.png").await;
    assert_eq!(img_response.status_code(), 200);
    assert_eq!(img_response.header("content-type"), "image/png");
    assert!(!img_response.as_bytes().is_empty());
}

#[tokio::test]
async fn test_non_image_files_not_served() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    let md_content = "# Test";
    let md_path = temp_dir.path().join("test.md");
    fs::write(&md_path, md_content).expect("Failed to write markdown file");

    let txt_path = temp_dir.path().join("secret.txt");
    fs::write(&txt_path, "secret content").expect("Failed to write txt file");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = vec![md_path];
    let is_directory_mode = false;
    let router = new_router(base_dir, tracked_files, is_directory_mode)
        .expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    let response = server.get("/secret.txt").await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_html_tags_in_markdown_are_rendered() {
    let markdown_content = r#"# HTML Test

This markdown contains HTML tags:

<div class="highlight">
    <p>This should be rendered as HTML, not escaped</p>
    <span style="color: red;">Red text</span>
</div>

Regular **markdown** still works.
"#;

    let (server, _temp_file) = create_test_server(markdown_content).await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(body.contains(r#"<div class="highlight">"#));
    assert!(body.contains(r#"<span style="color: red;">"#));
    assert!(body.contains("<p>This should be rendered as HTML, not escaped</p>"));
    assert!(!body.contains("&lt;div"));
    assert!(!body.contains("&gt;"));
    assert!(body.contains("<strong>markdown</strong>"));
}

#[tokio::test]
async fn test_mermaid_diagram_detection_and_script_injection() {
    let markdown_content = r#"# Mermaid Test

Regular content here.

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[End]
    B -->|No| D[Continue]
```

More regular content.

```javascript
// This is a regular code block, not mermaid
console.log("Hello World");
```
"#;

    let (server, _temp_file) = create_test_server(markdown_content).await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(body.contains(r#"class="language-mermaid""#));
    assert!(body.contains("graph TD"));

    let has_raw_content = body.contains("A[Start] --> B{Decision}");
    let has_encoded_content = body.contains("A[Start] --&gt; B{Decision}");
    assert!(
        has_raw_content || has_encoded_content,
        "Expected mermaid content not found in body"
    );

    assert!(body.contains(r#"<script src="/mermaid.min.js"></script>"#));
    assert!(body.contains("function initMermaid()"));
    assert!(body.contains("function transformMermaidCodeBlocks()"));
    assert!(body.contains("function getMermaidTheme()"));
    assert!(body.contains(r#"class="language-javascript""#));
    assert!(body.contains("console.log"));
}

#[tokio::test]
async fn test_no_mermaid_script_injection_without_mermaid_blocks() {
    let markdown_content = r#"# No Mermaid Test

This content has no mermaid diagrams.

```javascript
console.log("Hello World");
```

```bash
echo "Regular code block"
```

Just regular markdown content.
"#;

    let (server, _temp_file) = create_test_server(markdown_content).await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(!body.contains(r#"<script src="https://cdn.jsdelivr.net/npm/mermaid@11.12.0/dist/mermaid.min.js"></script>"#));
    assert!(body.contains("function initMermaid()"));
    assert!(body.contains(r#"class="language-javascript""#));
    assert!(body.contains(r#"class="language-bash""#));
}

#[tokio::test]
async fn test_multiple_mermaid_diagrams() {
    let markdown_content = r#"# Multiple Mermaid Diagrams

## Flowchart
```mermaid
graph LR
    A --> B
```

## Sequence Diagram
```mermaid
sequenceDiagram
    Alice->>Bob: Hello
    Bob-->>Alice: Hi
```

## Class Diagram
```mermaid
classDiagram
    Animal <|-- Duck
```
"#;

    let (server, _temp_file) = create_test_server(markdown_content).await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    let mermaid_occurrences = body.matches(r#"class="language-mermaid""#).count();
    assert_eq!(mermaid_occurrences, 3);

    assert!(body.contains("graph LR"));
    assert!(body.contains("sequenceDiagram"));
    assert!(body.contains("classDiagram"));

    assert!(body.contains("A --&gt; B") || body.contains("A --> B"));
    assert!(body.contains("Alice-&gt;&gt;Bob") || body.contains("Alice->>Bob"));
    assert!(body.contains("Animal &lt;|-- Duck") || body.contains("Animal <|-- Duck"));

    let script_occurrences = body
        .matches(r#"<script src="/mermaid.min.js"></script>"#)
        .count();
    assert_eq!(script_occurrences, 1);
}

#[tokio::test]
async fn test_mermaid_js_etag_caching() {
    let (server, _temp_file) = create_test_server("# Test").await;

    let response = server.get("/mermaid.min.js").await;
    assert_eq!(response.status_code(), 200);

    let etag = response.header("etag");
    assert!(!etag.is_empty(), "ETag header should be present");

    let cache_control = response.header("cache-control");
    let cache_control_str = cache_control.to_str().unwrap();
    assert!(cache_control_str.contains("public"));
    assert!(cache_control_str.contains("no-cache"));

    let content_type = response.header("content-type");
    assert_eq!(content_type, "application/javascript");

    assert!(!response.as_bytes().is_empty());

    let response_304 = server
        .get("/mermaid.min.js")
        .add_header(
            axum::http::header::IF_NONE_MATCH,
            axum::http::HeaderValue::from_str(etag.to_str().unwrap()).unwrap(),
        )
        .await;

    assert_eq!(response_304.status_code(), 304);
    assert_eq!(response_304.header("etag"), etag);
    assert!(response_304.as_bytes().is_empty());

    let response_200 = server
        .get("/mermaid.min.js")
        .add_header(
            axum::http::header::IF_NONE_MATCH,
            axum::http::HeaderValue::from_static("\"different-etag\""),
        )
        .await;

    assert_eq!(response_200.status_code(), 200);
    assert!(!response_200.as_bytes().is_empty());
}

#[tokio::test]
async fn test_directory_mode_serves_multiple_files() {
    let (server, _temp_dir) = create_directory_server().await;

    let response1 = server.get("/test1.md").await;
    assert_eq!(response1.status_code(), 200);
    let body1 = response1.text();
    assert!(body1.contains("<h1>Test 1</h1>"));
    assert!(body1.contains("Content of test1"));

    let response2 = server.get("/test2.markdown").await;
    assert_eq!(response2.status_code(), 200);
    let body2 = response2.text();
    assert!(body2.contains("<h1>Test 2</h1>"));
    assert!(body2.contains("Content of test2"));

    let response3 = server.get("/test3.md").await;
    assert_eq!(response3.status_code(), 200);
    let body3 = response3.text();
    assert!(body3.contains("<h1>Test 3</h1>"));
    assert!(body3.contains("Content of test3"));
}

#[tokio::test]
async fn test_directory_mode_file_not_found() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server.get("/nonexistent.md").await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_directory_mode_has_navigation_sidebar() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server.get("/test1.md").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(body.contains(r#"<nav class="sidebar">"#));
    assert!(body.contains(r#"<ul class="file-tree">"#));
    assert!(body.contains("test1.md"));
    assert!(body.contains("test2.markdown"));
    assert!(body.contains("test3.md"));
}

#[tokio::test]
async fn test_single_file_mode_no_navigation_sidebar() {
    let (server, _temp_file) = create_test_server("# Single File Test").await;

    let response = server.get("/").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(!body.contains(r#"<nav class="sidebar">"#));
    assert!(!body.contains("<h3>Files</h3>"));
    assert!(!body.contains(r#"<ul class="file-tree">"#));
}

#[tokio::test]
async fn test_directory_mode_active_file_highlighting() {
    let (server, _temp_dir) = create_directory_server().await;

    let response1 = server.get("/test1.md").await;
    assert_eq!(response1.status_code(), 200);
    let body1 = response1.text();

    assert!(
        body1.contains(r#"href="/test1.md" class="active""#),
        "test1.md link should have href and class on same line"
    );

    let active_link_count = body1.matches(r#"class="active""#).count();
    assert_eq!(active_link_count, 1, "Should have exactly one active link");

    let response2 = server.get("/test2.markdown").await;
    assert_eq!(response2.status_code(), 200);
    let body2 = response2.text();

    assert!(
        body2.contains(r#"href="/test2.markdown" class="active""#),
        "test2.markdown link should have href and class on same line"
    );
}

#[tokio::test]
async fn test_directory_mode_file_order() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server.get("/test1.md").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();

    let test1_pos = body.find("test1.md").expect("test1.md not found");
    let test2_pos = body
        .find("test2.markdown")
        .expect("test2.markdown not found");
    let test3_pos = body.find("test3.md").expect("test3.md not found");

    assert!(
        test1_pos < test2_pos,
        "test1.md should appear before test2.markdown"
    );
    assert!(
        test2_pos < test3_pos,
        "test2.markdown should appear before test3.md"
    );
}

#[tokio::test]
async fn test_directory_mode_websocket_file_modification() {
    let (server, temp_dir) = create_directory_server_with_http().await;

    let mut websocket = server.get_websocket("/ws").await.into_websocket().await;

    let test_file = temp_dir.path().join("test1.md");
    fs::write(&test_file, "# Modified Test 1\n\nContent has changed")
        .expect("Failed to modify file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let update_result = tokio::time::timeout(
        Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
        websocket.receive_json::<ServerMessage>(),
    )
    .await;

    match update_result {
        Ok(update_message) => {
            if let ServerMessage::Reload = update_message {
                // Success
            } else {
                panic!("Expected Reload message after file modification");
            }
        }
        Err(_) => {
            panic!("Timeout waiting for WebSocket update after file modification");
        }
    }
}

#[tokio::test]
async fn test_directory_mode_new_file_triggers_reload() {
    let (server, temp_dir) = create_directory_server_with_http().await;

    let mut websocket = server.get_websocket("/ws").await.into_websocket().await;

    let new_file = temp_dir.path().join("test4.md");
    fs::write(&new_file, "# Test 4\n\nThis is a new file").expect("Failed to create new file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let update_result = tokio::time::timeout(
        Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
        websocket.receive_json::<ServerMessage>(),
    )
    .await;

    match update_result {
        Ok(update_message) => {
            if let ServerMessage::Reload = update_message {
                // Success
            } else {
                panic!("Expected Reload message after new file creation");
            }
        }
        Err(_) => {
            panic!("Timeout waiting for WebSocket update after new file creation");
        }
    }

    let response = server.get("/test1.md").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(
        body.contains("test4.md"),
        "New file should appear in navigation"
    );

    let new_file_response = server.get("/test4.md").await;
    assert_eq!(new_file_response.status_code(), 200);
    let new_file_body = new_file_response.text();
    assert!(new_file_body.contains("<h1>Test 4</h1>"));
    assert!(new_file_body.contains("This is a new file"));
}

#[tokio::test]
async fn test_editor_save_simulation_single_file_mode() {
    let (server, temp_file) =
        create_test_server_with_http("# Original\n\nOriginal content").await;

    let file_path = temp_file.path().to_path_buf();
    let backup_path = file_path.with_extension("md~");

    let initial_response = server.get("/").await;
    assert_eq!(initial_response.status_code(), 200);
    assert!(initial_response.text().contains("Original content"));

    fs::rename(&file_path, &backup_path).expect("Failed to rename to backup");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let during_save_response = server.get("/").await;
    assert_eq!(
        during_save_response.status_code(),
        200,
        "File should not return 404 during editor save"
    );

    fs::write(&file_path, "# Updated\n\nUpdated content").expect("Failed to write new file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let final_response = server.get("/").await;
    assert_eq!(final_response.status_code(), 200);
    let final_body = final_response.text();
    assert!(
        final_body.contains("Updated content"),
        "Should serve updated content after save"
    );
    assert!(
        !final_body.contains("Original content"),
        "Should not serve old content"
    );

    let _ = fs::remove_file(&backup_path);
}

#[tokio::test]
async fn test_editor_save_simulation_directory_mode() {
    let (server, temp_dir) = create_directory_server_with_http().await;

    let file_path = temp_dir.path().join("test1.md");
    let backup_path = temp_dir.path().join("test1.md~");

    let initial_response = server.get("/test1.md").await;
    assert_eq!(initial_response.status_code(), 200);
    assert!(initial_response.text().contains("Content of test1"));

    fs::rename(&file_path, &backup_path).expect("Failed to rename to backup");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let during_save_response = server.get("/test1.md").await;
    assert_eq!(
        during_save_response.status_code(),
        200,
        "File should not return 404 during editor save in directory mode"
    );

    fs::write(&file_path, "# Test 1 Updated\n\nUpdated content")
        .expect("Failed to write new file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let final_response = server.get("/test1.md").await;
    assert_eq!(final_response.status_code(), 200);
    let final_body = final_response.text();
    assert!(
        final_body.contains("Updated content"),
        "Should serve updated content after save"
    );

    let _ = fs::remove_file(&backup_path);
}

#[tokio::test]
async fn test_no_404_during_editor_save_sequence() {
    let (server, temp_dir) = create_directory_server_with_http().await;
    let mut websocket = server.get_websocket("/ws").await.into_websocket().await;

    let file_path = temp_dir.path().join("test1.md");
    let backup_path = temp_dir.path().join("test1.md~");

    fs::rename(&file_path, &backup_path).expect("Failed to rename to backup");
    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let response_after_rename = server.get("/test1.md").await;
    assert_eq!(
        response_after_rename.status_code(),
        200,
        "Should not get 404 after rename to backup"
    );

    fs::write(&file_path, "# Test 1 Updated\n\nNew content").expect("Failed to write new file");
    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let response_after_create = server.get("/test1.md").await;
    assert_eq!(
        response_after_create.status_code(),
        200,
        "Should successfully serve after new file created"
    );
    assert!(response_after_create.text().contains("New content"));

    let update_result = tokio::time::timeout(
        Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
        websocket.receive_json::<ServerMessage>(),
    )
    .await;

    assert!(update_result.is_ok(), "Should receive reload after save");

    let _ = fs::remove_file(&backup_path);
}

#[tokio::test]
async fn test_yaml_frontmatter_is_stripped() {
    let (server, _temp_file) = create_test_server(YAML_FRONTMATTER_CONTENT).await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(!body.contains("title: Test Post"));
    assert!(!body.contains("author: Name"));
    assert!(body.contains("<h1>Test Post</h1>"));
}

#[tokio::test]
async fn test_toml_frontmatter_is_stripped() {
    let (server, _temp_file) = create_test_server(TOML_FRONTMATTER_CONTENT).await;

    let response = server.get("/").await;

    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(!body.contains("title = \"Test Post\""));
    assert!(body.contains("<h1>Test Post</h1>"));
}

#[tokio::test]
async fn test_temp_file_rename_triggers_reload_single_file_mode() {
    let (server, temp_file) =
        create_test_server_with_http("# Original\n\nOriginal content").await;

    let mut websocket = server.get_websocket("/ws").await.into_websocket().await;

    let file_path = temp_file.path().to_path_buf();
    let temp_write_path = file_path.with_extension("md.tmp.12345");

    let initial_response = server.get("/").await;
    assert_eq!(initial_response.status_code(), 200);
    assert!(
        initial_response.text().contains("Original content"),
        "File should be tracked and serving content before edit"
    );

    fs::write(
        &temp_write_path,
        "# Updated\n\nUpdated content via temp file",
    )
    .expect("Failed to write temp file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    fs::rename(&temp_write_path, &file_path).expect("Failed to rename temp file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let update_result = tokio::time::timeout(
        Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
        websocket.receive_json::<ServerMessage>(),
    )
    .await;

    match update_result {
        Ok(update_message) => {
            if let ServerMessage::Reload = update_message {
                // Success
            } else {
                panic!("Expected Reload message after temp file rename");
            }
        }
        Err(_) => {
            panic!("Timeout waiting for WebSocket update after temp file rename");
        }
    }

    let final_response = server.get("/").await;
    assert_eq!(final_response.status_code(), 200);
    let final_body = final_response.text();
    assert!(
        final_body.contains("Updated content via temp file"),
        "Should serve updated content after temp file rename"
    );
    assert!(
        !final_body.contains("Original content"),
        "Should not serve old content"
    );
}

#[tokio::test]
async fn test_temp_file_rename_triggers_reload_directory_mode() {
    let (server, temp_dir) = create_directory_server_with_http().await;

    let mut websocket = server.get_websocket("/ws").await.into_websocket().await;

    let file_path = temp_dir.path().join("test1.md");
    let temp_write_path = temp_dir.path().join("test1.md.tmp.67890");

    let initial_response = server.get("/test1.md").await;
    assert_eq!(initial_response.status_code(), 200);
    assert!(
        initial_response.text().contains("Content of test1"),
        "File should be tracked and serving content before edit"
    );

    fs::write(
        &temp_write_path,
        "# Test 1 Updated\n\nUpdated via temp file rename",
    )
    .expect("Failed to write temp file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    fs::rename(&temp_write_path, &file_path).expect("Failed to rename temp file");

    tokio::time::sleep(Duration::from_millis(FILE_WATCH_DELAY_MS)).await;

    let update_result = tokio::time::timeout(
        Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
        websocket.receive_json::<ServerMessage>(),
    )
    .await;

    match update_result {
        Ok(update_message) => {
            if let ServerMessage::Reload = update_message {
                // Success
            } else {
                panic!("Expected Reload message after temp file rename in directory mode");
            }
        }
        Err(_) => {
            panic!(
                "Timeout waiting for WebSocket update after temp file rename in directory mode"
            );
        }
    }

    let final_response = server.get("/test1.md").await;
    assert_eq!(final_response.status_code(), 200);
    let final_body = final_response.text();
    assert!(
        final_body.contains("Updated via temp file rename"),
        "Should serve updated content after temp file rename"
    );
    assert!(
        !final_body.contains("Content of test1"),
        "Should not serve old content"
    );
}

#[tokio::test]
async fn test_bind_with_port_increment_finds_free_port() {
    // occupy a port, then verify bind_with_port_increment skips it
    let blocker = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let blocked_port = blocker.local_addr().unwrap().port();

    let (listener, actual_port) =
        bind_with_port_increment("127.0.0.1", blocked_port).await.unwrap();

    assert!(actual_port > blocked_port, "should have incremented past blocked port");
    assert_eq!(listener.local_addr().unwrap().port(), actual_port);
}

#[tokio::test]
async fn test_bind_with_port_increment_uses_requested_port_when_free() {
    // bind to 0 to get an OS-assigned port, then drop it so it's free
    let tmp = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let free_port = tmp.local_addr().unwrap().port();
    drop(tmp);

    let (listener, actual_port) =
        bind_with_port_increment("127.0.0.1", free_port).await.unwrap();

    assert_eq!(actual_port, free_port);
    assert_eq!(listener.local_addr().unwrap().port(), free_port);
}

#[tokio::test]
async fn test_bind_with_port_increment_skips_multiple_occupied_ports() {
    // find three consecutive free ports by binding to 0 and checking adjacency
    let mut blockers = Vec::new();
    let mut base_port = None;

    // try a few times to find three consecutive bindable ports
    for _ in 0..20 {
        let l1 = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let p = l1.local_addr().unwrap().port();
        if let (Ok(l2), Ok(l3)) = (
            TcpListener::bind(("127.0.0.1", p + 1)).await,
            TcpListener::bind(("127.0.0.1", p + 2)).await,
        ) {
            blockers.extend([l1, l2, l3]);
            base_port = Some(p);
            break;
        }
    }

    let base_port = base_port.expect("could not find three consecutive free ports");

    let (listener, actual_port) =
        bind_with_port_increment("127.0.0.1", base_port).await.unwrap();

    assert!(actual_port >= base_port + 3, "should skip all three blocked ports");
    assert_eq!(listener.local_addr().unwrap().port(), actual_port);

    drop(blockers);
}

#[tokio::test]
async fn test_subdirectory_image_serving() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    let md_content = "# Test\n\n![Sub Image](subdir/test.png)\n![Deep Image](a/b/deep.png)";
    let md_path = temp_dir.path().join("test.md");
    fs::write(&md_path, md_content).expect("Failed to write markdown file");

    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
        0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
        0xD7, 0x63, 0xF8, 0x0F, 0x00, 0x00, 0x01, 0x00, 0x01, 0x5C, 0xDD, 0x8D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    // image in subdirectory
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).expect("Failed to create subdir");
    fs::write(sub_dir.join("test.png"), &png_data).expect("Failed to write subdir image");

    // image in deeply nested subdirectory
    let deep_dir = temp_dir.path().join("a").join("b");
    fs::create_dir_all(&deep_dir).expect("Failed to create deep dir");
    fs::write(deep_dir.join("deep.png"), &png_data).expect("Failed to write deep image");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = vec![md_path];
    let router = new_router(base_dir, tracked_files, false).expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    // subdirectory image
    let response = server.get("/subdir/test.png").await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.header("content-type"), "image/png");

    // deeply nested image
    let response = server.get("/a/b/deep.png").await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.header("content-type"), "image/png");
}

#[tokio::test]
async fn test_directory_traversal_blocked() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    let md_path = temp_dir.path().join("test.md");
    fs::write(&md_path, "# Test").expect("Failed to write markdown file");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = vec![md_path];
    let router = new_router(base_dir, tracked_files, false).expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    let response = server.get("/../../../etc/passwd").await;
    assert_ne!(response.status_code(), 200);
}

#[tokio::test]
async fn test_same_dir_image_still_works_with_wildcard_route() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    let md_path = temp_dir.path().join("test.md");
    fs::write(&md_path, "# Test\n\n![img](photo.jpg)").expect("Failed to write md");

    let jpg_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // minimal JPEG header
    fs::write(temp_dir.path().join("photo.jpg"), &jpg_data).expect("Failed to write jpg");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = vec![md_path];
    let router = new_router(base_dir, tracked_files, false).expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    let response = server.get("/photo.jpg").await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(response.header("content-type"), "image/jpeg");
}

#[test]
fn test_scan_markdown_files_deep_nesting() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("root.md"), "# Root").expect("Failed to write");

    let level1 = temp_dir.path().join("level1");
    fs::create_dir(&level1).expect("Failed to create level1");
    fs::write(level1.join("l1.md"), "# Level 1").expect("Failed to write");

    let level2 = level1.join("level2");
    fs::create_dir(&level2).expect("Failed to create level2");
    fs::write(level2.join("l2.md"), "# Level 2").expect("Failed to write");

    let level3 = level2.join("level3");
    fs::create_dir(&level3).expect("Failed to create level3");
    fs::write(level3.join("l3.md"), "# Level 3").expect("Failed to write");

    let result = scan_markdown_files(temp_dir.path()).expect("Failed to scan");

    assert_eq!(result.len(), 4);
}

#[tokio::test]
async fn test_directory_mode_serves_nested_files() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("root.md"), "# Root File").expect("Failed to write");

    let docs_dir = temp_dir.path().join("docs");
    fs::create_dir(&docs_dir).expect("Failed to create docs dir");
    fs::write(docs_dir.join("guide.md"), "# Guide\n\nGuide content").expect("Failed to write");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = scan_markdown_files(&base_dir).expect("Failed to scan");
    let router = new_router(base_dir, tracked_files, true).expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    let response = server.get("/docs/guide.md").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();
    assert!(body.contains("<h1>Guide</h1>"));
    assert!(body.contains("Guide content"));
}

#[tokio::test]
async fn test_directory_mode_tree_sidebar() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("readme.md"), "# Readme").expect("Failed to write");

    let docs_dir = temp_dir.path().join("docs");
    fs::create_dir(&docs_dir).expect("Failed to create docs dir");
    fs::write(docs_dir.join("guide.md"), "# Guide").expect("Failed to write");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = scan_markdown_files(&base_dir).expect("Failed to scan");
    let router = new_router(base_dir, tracked_files, true).expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    let response = server.get("/readme.md").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(body.contains(r#"<ul class="file-tree">"#));
    assert!(body.contains("<details"));
    assert!(body.contains("<summary>"));
    assert!(body.contains("docs"));
    assert!(body.contains("guide.md"));
}

#[tokio::test]
async fn test_nested_file_active_highlighting() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    fs::write(temp_dir.path().join("root.md"), "# Root").expect("Failed to write");

    let docs_dir = temp_dir.path().join("docs");
    fs::create_dir(&docs_dir).expect("Failed to create docs dir");
    fs::write(docs_dir.join("guide.md"), "# Guide").expect("Failed to write");

    let base_dir = temp_dir.path().to_path_buf();
    let tracked_files = scan_markdown_files(&base_dir).expect("Failed to scan");
    let router = new_router(base_dir, tracked_files, true).expect("Failed to create router");
    let server = TestServer::new(router).expect("Failed to create test server");

    let response = server.get("/docs/guide.md").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(
        body.contains(r#"href="/docs/guide.md" class="active""#),
        "nested file should have active class"
    );

    let active_count = body.matches(r#"class="active""#).count();
    assert_eq!(active_count, 1, "Should have exactly one active link");
}

// --- API endpoint tests ---

#[tokio::test]
async fn test_api_raw_content() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server.get("/api/raw_content?path=test1.md").await;
    assert_eq!(response.status_code(), 200);
    assert_eq!(
        response.header("content-type"),
        "text/plain; charset=utf-8"
    );
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

    // file should be gone from disk
    assert!(!temp_dir.path().join("test1.md").exists());

    // file should no longer be served
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

    // old path gone, new path exists on disk
    assert!(!temp_dir.path().join("test1.md").exists());
    assert!(temp_dir.path().join("renamed.md").exists());

    // new file should be servable
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

    // parent dir created automatically
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

    // file on disk with default content
    let content = fs::read_to_string(temp_dir.path().join("new_file.md")).unwrap();
    assert!(content.contains("# new_file"));

    // servable
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

#[tokio::test]
async fn test_context_menu_elements_in_directory_mode() {
    let (server, _temp_dir) = create_directory_server().await;

    let response = server.get("/test1.md").await;
    assert_eq!(response.status_code(), 200);
    let body = response.text();

    assert!(body.contains(r#"id="contextMenu""#));
    assert!(body.contains(r#"id="fileDialog""#));
    assert!(body.contains("initContextMenu"));
    assert!(body.contains("var currentFile"));
    assert!(body.contains("data-dir-path"));
}

#[tokio::test]
async fn test_context_menu_not_in_single_file_mode() {
    let (server, _temp_file) = create_test_server("# Single").await;

    let response = server.get("/").await;
    let body = response.text();

    assert!(!body.contains(r#"id="contextMenu""#));
    assert!(!body.contains(r#"id="fileDialog""#));
}
