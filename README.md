![mdlive](static/img/md-hero.png)

# mdlive

A lightweight markdown editor and live preview server. Point it at a file or directory, get instant rendered output in the browser with live reload. Edit, create, rename, delete -- all from the browser. No config, no setup.

I built this to sit next to AI coding agents. When an agent writes markdown -- plans, architecture docs, research -- I'd rather read it rendered than squint at raw text in a terminal. But it works just as well as a standalone markdown workspace.

This started as a fork of Jose Fernandez's [mdserve](https://github.com/jfernandez/mdserve). The original idea and core implementation are his. I've since taken it in a different direction: in-browser editing, file CRUD, version history, keyboard shortcuts, theming. Different enough to warrant its own repo.

https://github.com/bearded-giant/mdlive/raw/main/docs/images/mdlive.mp4

## Install

```bash
git clone https://github.com/bearded-giant/mdlive.git
cd mdlive
cargo build --release
# binary is at target/release/mdlive
```

Single binary, no runtime dependencies. Everything (templates, JS libraries, images) is embedded at compile time.

### Claude Code plugin

mdlive is available as a [Claude Code](https://docs.anthropic.com/en/docs/claude-code) plugin. Once installed, Claude will automatically use mdlive to preview markdown when it makes sense -- plans, architecture docs, tables, diagrams, and anything that reads better rendered than raw.

```
/plugin install mdlive@bearded-giant/mdlive
```

## Quick start

```bash
mdlive README.md         # single file, opens browser
mdlive docs/             # directory mode with sidebar
```

#### Single file view
![Single file view](docs/images/single-view.png)

#### Directory view
![Directory view](docs/images/dir-view.png)

It watches for changes and reloads instantly via WebSocket. New files in directory mode get picked up automatically. The browser opens on launch by default (pass `--no-open` to suppress).

### Running alongside an AI agent

Point mdlive at whatever directory the agent writes to. As the agent creates or updates markdown files, the browser reflects changes in real time.

```bash
mdlive scratch/ &        # background it, work alongside your agent
```

This is particularly useful for reviewing agent-generated plans, design docs, and research as they're being written.

## Features

### Viewing

GFM rendering with tables, task lists, strikethrough, and fenced code blocks. Syntax highlighting via highlight.js. Mermaid diagram rendering. YAML and TOML frontmatter is stripped before rendering. Images referenced in markdown are served inline (png, jpg, gif, svg, webp). Plain text and JSON files are also viewable and editable with syntax highlighting.

### Themes

Five built-in themes: Catppuccin Mocha (default), Catppuccin Macchiato, Catppuccin Latte, Light, and Dark. Switch via the palette icon in the bottom-right corner. Selection is persisted to localStorage.

### Editing

Every markdown file has a built-in editor accessible via the edit icon or the `e` shortcut. The editor is a split-pane view: raw markdown on the left, live preview on the right. The divider is draggable (persisted to localStorage). Scroll position syncs between panes. `Ctrl+S` saves, `Esc` closes. An unsaved-changes dot and `beforeunload` guard prevent accidental data loss.

### File operations

Full CRUD from the browser. Create new files, rename/move existing ones, delete with confirmation. All operations available from both the right-click context menu (on sidebar items) and from buttons in the editor header. In single-file mode, right-click the content area for the context menu.

<img src="docs/images/context-menu.png" alt="Context menu" width="425">

### Revisions

Every save creates a timestamped snapshot in a `.mdlive/` directory next to your files. Open the revisions panel (`h` or `r` shortcut, or the Revisions button) to browse previous versions and restore any of them. You'll probably want to add `.mdlive` to your `.gitignore` unless you want revision history in your repo -- which can actually be handy for shared design docs or architecture decisions.

<img src="docs/images/revision-sidebar.jpg" alt="Revision history" width="365"> <img src="docs/images/revision-detail.png" alt="Revision detail with diff" width="500">

### Directory mode

Pass a directory and mdlive recursively finds all `.md`, `.markdown`, `.txt`, and `.json` files, builds a collapsible tree sidebar with document tabs for quick switching between open files. Right-click folders to create files in specific subdirectories. The sidebar is resizable (drag the edge) and collapsible (`k` shortcut or the toggle button). Search the tree by pressing `s`.

### Keyboard shortcuts

Press `/` to see all available shortcuts. Shortcuts are suppressed when typing in inputs, when modifier keys are held, and when dialogs are open.


| Key | Action |
|-----|--------|
| `/` | Show shortcuts |
| `e` | Edit file |
| `n` | New file |
| `d` | Delete file |
| `k` | Toggle sidebar |
| `s` | Focus search |
| `Ctrl+S` | Save |
| `h` / `r` | Toggle revisions |
| `w` | Close tab |
| `?` | About / attribution |
| `Esc` | Close / cancel |

## Usage

```bash
mdlive file.md                # serve a single file (port 3000)
mdlive docs/                  # directory mode with sidebar
mdlive docs/ -p 8080          # custom port
mdlive file.md --no-open      # don't auto-open the browser
mdlive file.md -H 0.0.0.0    # bind to all interfaces
```

## Development

Rust 1.82+, 2021 edition. Templates and static assets are embedded at compile time (`minijinja-embed`, `include_str!`, `include_bytes!`), so changes to `templates/` or `static/` need a rebuild.

```bash
cargo build --release
cargo test                        # all tests (74 unit + integration)
cargo test --test pages_test      # specific test file
cargo test test_server_starts     # by name
```

See [docs/architecture.md](docs/architecture.md) for internals.

## License

MIT. See [LICENSE](LICENSE).

## Attribution

Based on [mdserve](https://github.com/jfernandez/mdserve) by Jose Fernandez, MIT licensed. Original contributors are in the git history.
