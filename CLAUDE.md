# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

mdlive is a markdown workspace server built as a companion for AI coding agents.
It renders markdown to HTML with live reload via WebSocket, supporting both
single-file and directory modes.

## Build and test

```bash
cargo build --release
cargo test                            # all tests (unit + integration)
cargo test util::tests                # run tests for a specific module
cargo test --test api_test            # run a specific integration test file
cargo test test_server_starts         # run a single test by name substring
```

Rust 1.82+, 2021 edition. Templates are embedded at compile time via
`minijinja-embed` -- changes to `templates/` require a rebuild. The Mermaid JS
bundle in `static/js/` is also embedded via `include_str!` in `template.rs`.

## Architecture

```
src/
  main.rs          -- CLI parsing (clap derive), calls lib entry point
  lib.rs           -- public API: serve_markdown(), scan_markdown_files(), module declarations
  state.rs         -- MarkdownState, TrackedFile, SharedMarkdownState, message enums
  router.rs        -- new_router(), route registration, watcher setup
  handlers/
    mod.rs         -- re-exports
    pages.rs       -- serve_html_root, serve_file, render_markdown
    api.rs         -- api_raw_content, api_delete_file, api_move_file, api_create_file
    static_files.rs -- serve_mermaid_js, serve_highlight_js, serve_embedded_js
    websocket.rs   -- websocket_handler, handle_websocket
  watcher.rs       -- file event handling (notify crate integration)
  tree.rs          -- build_file_tree, build_tree_level (pure data transform)
  util.rs          -- is_markdown_file, is_image_file, guess_image_content_type, scan_markdown_files
  template.rs      -- template_env(), MiniJinja setup, embedded JS constants
templates/
  main.html        -- single MiniJinja template for both modes
tests/
  common/mod.rs    -- test helpers: create_test_server, create_directory_server
  pages_test.rs    -- page rendering, directory mode, mermaid, sidebar tests
  api_test.rs      -- CRUD API endpoint tests
  websocket_test.rs -- WebSocket, file watcher, editor save simulation tests
```

Key types:
- `MarkdownState` (state.rs) -- central state: base_dir, tracked files HashMap,
  directory mode flag, broadcast channel for reload signals
- `TrackedFile` (state.rs) -- per-file: path, last_modified timestamp, pre-rendered HTML
- `SharedMarkdownState` = `Arc<Mutex<MarkdownState>>`

Data flow: file system events (notify crate, recursive) -> mpsc channel ->
`handle_file_event` (watcher.rs) -> state update + broadcast `ServerMessage::Reload` ->
WebSocket clients (websocket.rs) -> `window.location.reload()`

## Design constraints

- **Agent-companion scope.** Not a documentation platform or configurable server.
- **Zero config.** `mdlive file.md` must work with no flags or config files.
- **Recursive directory tree.** Directory mode recursively scans and watches
  subdirectories, rendering a collapsible tree sidebar.
- **Pre-rendered in memory.** All tracked files rendered to HTML on startup and
  on change. Serving is always from memory, never from disk.
- **Minimal client-side JS.** Client JS handles theme selection, sidebar toggle,
  Mermaid rendering, and WebSocket reload only.
- **No file removal on delete events.** Editors like neovim save via
  rename-to-backup then create-new. Removing on delete would cause transient
  404s. Files stay tracked even after `Remove` events.

## Changelog

Generated with [git-cliff](https://git-cliff.org/) using `cliff.toml`:

```bash
git cliff -o CHANGELOG.md
```

## Commits

Use conventional commits: `type: lowercase description` (e.g. `feat:`, `fix:`,
`chore:`, `docs:`, `refactor:`, `test:`). No scopes, no emojis. Subject line
max 72 chars, imperative mood. Body optional, wrap at 72 chars, explain why not
what.
