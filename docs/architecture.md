# mdlive Architecture

## Overview

mdlive is a markdown workspace server for AI coding agents with live reload.
It supports single-file and directory modes through a unified architecture.

**Core principle**: Always work with a base directory and a list of tracked files (1 or more).

```mermaid
graph LR
    A[File System] -->|notify events| B[watcher.rs]
    B -->|update state| C[state.rs]
    B -->|broadcast| D[websocket.rs]
    E[HTTP Request] -->|router.rs| F[handlers/]
    F -->|lookup| C
    C -->|render| G[template.rs]
    G -->|HTML| H[Browser]
    D -->|reload signal| H
```

## Modes

### Single-File Mode
```bash
mdlive README.md
```
- Watches parent directory
- Tracks single file
- No navigation sidebar

### Directory Mode
```bash
mdlive ./docs/
```
- Watches specified directory recursively
- Tracks all `.md` and `.markdown` files
- Shows collapsible tree sidebar

## Module Structure

```
src/
  main.rs           -- CLI parsing (clap), calls lib entry point
  lib.rs            -- public API: serve_markdown(), scan_markdown_files()
  state.rs          -- MarkdownState, TrackedFile, SharedMarkdownState
  router.rs         -- new_router(), route registration, watcher setup
  handlers/
    pages.rs        -- serve_html_root, serve_file, render_markdown
    api.rs          -- CRUD endpoints (raw content, delete, move, create)
    static_files.rs -- embedded JS serving with ETag caching
    websocket.rs    -- WebSocket handler for live reload
  watcher.rs        -- file event handling (notify crate)
  tree.rs           -- build_file_tree (pure data transform)
  util.rs           -- is_markdown_file, is_image_file, scan helpers
  template.rs       -- MiniJinja setup, embedded JS constants
```

## State Management

Central state stores:
- Base directory path
- HashMap of tracked files (filename -> metadata + pre-rendered HTML)
- Directory mode flag (determines UI)
- WebSocket broadcast channel

```mermaid
classDiagram
    class MarkdownState {
        +PathBuf base_dir
        +HashMap~String,TrackedFile~ tracked_files
        +bool is_directory_mode
        +Sender~ServerMessage~ change_tx
    }

    class TrackedFile {
        +PathBuf path
        +SystemTime last_modified
        +String html
    }

    MarkdownState "1" --> "*" TrackedFile : contains
```

Mode is determined by user intent, not file count:
- `mdlive /docs/` with 1 file shows sidebar
- `mdlive single.md` never shows sidebar

## Live Reload

Uses [notify](https://github.com/notify-rs/notify) crate to watch base directory recursively.

File changes flow:
1. File system event detected by `notify`
2. `handle_file_event` (watcher.rs) processes the event
3. Markdown re-rendered to HTML in `MarkdownState`
4. `ServerMessage::Reload` broadcast via channel
5. WebSocket clients receive reload message
6. Clients execute `window.location.reload()`

Events handled:
- Create/modify: refresh file, add if new (directory mode only)
- Delete: ignored (editors like neovim save via rename-to-backup then create-new)
- Rename: track new path
- Image changes: trigger reload without tracking

## Routing

Single unified router (router.rs) handles both modes:
- `GET /` -> first file alphabetically
- `GET /:filename` -> markdown files or images
- `GET /ws` -> WebSocket connection
- `GET /mermaid.min.js` -> bundled Mermaid library
- `GET /highlight.min.js` -> bundled highlight.js
- `GET /api/raw_content` -> raw markdown content
- `POST /api/delete_file` -> delete a tracked file
- `POST /api/move_file` -> rename/move a tracked file
- `POST /api/create_file` -> create a new markdown file

Directory traversal blocked by `canonicalize` + `starts_with(base_dir)` validation.

## Rendering

Uses [MiniJinja](https://github.com/mitsuhiko/minijinja) with templates embedded
at compile time via `minijinja_embed`. Single template (`main.html`) handles both
modes via conditional blocks.

Template variables:
- `content`: pre-rendered markdown HTML
- `mermaid_enabled`: conditionally includes Mermaid.js
- `show_navigation`: controls sidebar visibility
- `tree`: nested tree of files and directories
- `current_file`: active file's relative path

## Design Decisions

**Unified architecture**: single code path handles both modes. Mode determined by user intent, not file count.

**Pre-rendered caching**: all tracked files rendered to HTML in memory on startup and on change. Serving always from memory, never from disk.

**Recursive directory tree**: subdirectories scanned and watched recursively. Sidebar renders a collapsible tree using `<details>/<summary>` elements.

**No file removal on delete**: editors save via rename-to-backup then create-new. Removing on delete would cause transient 404s.

**Server-side logic**: most logic lives server-side. Client JS minimal (theme, reload, Mermaid rendering).
