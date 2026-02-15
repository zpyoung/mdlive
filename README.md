# mdlive

Markdown workspace server for AI coding agents.

Follow along as your AI agent writes markdown, rendered live in the browser
instead of raw text in the terminal.

Originally forked from [mdserve](https://github.com/jfernandez/mdserve)
by Jose Fernandez. This project has diverged significantly -- adding file
operations, a modular architecture, and a different direction -- but the
core concept and initial implementation are his.

## Features

**Zero config.** `mdserve file.md` just works. No config files, no flags
required, no setup steps.

**Single binary.** One statically-compiled executable. Install it and forget
about it. No runtime dependencies to manage.

**Instant live reload.** File changes appear in the browser immediately via
WebSocket. This is the core interaction: an agent writes, a human reads.

**Directory mode.** Point it at a directory and get a collapsible tree sidebar
with recursive file discovery. New files are picked up automatically.

**Agent-friendly content.** Full GFM support (tables, task lists, code blocks),
Mermaid diagrams, syntax highlighting via highlight.js, and right-click
context menu for file operations (copy raw markdown, move, delete, new file).

**5 themes.** Light, dark, and Catppuccin variants (Latte, Frappe, Macchiato).
Your choice persists across sessions.

## What mdlive is not

It is not a documentation site generator, a static site server you deploy to
production, or a general-purpose markdown authoring tool with heavy
customization. Use mdBook, Docusaurus, or MkDocs for those things.

mdlive is ephemeral. Start it during a coding session, kill it when you're done.

## Installation

### Using Cargo

```bash
cargo install mdlive
```

### From Source

```bash
git clone https://github.com/bearded-giant/mdlive
cd mdlive
cargo build --release
cp target/release/mdserve <folder in your PATH>
```

The binary is called `mdserve` regardless of the package name.

## Usage

```bash
mdserve README.md              # single file on default port (3000)
mdserve docs/                  # directory with sidebar
mdserve docs/ -p 8080          # custom port
mdserve README.md --open       # open browser automatically
mdserve README.md -H 0.0.0.0  # custom hostname
```

**Single-file mode** serves one markdown file with a clean, focused view.
Watches the parent directory for changes.

**Directory mode** recursively scans and serves all `.md` and `.markdown`
files, displays a collapsible tree sidebar, watches for new files added
anywhere in the directory tree, and serves images from subdirectories.

## Architecture

For details on internal architecture, module structure, and design decisions,
see [docs/architecture.md](docs/architecture.md).

## Development

Rust 1.82+, 2021 edition. Templates are embedded at compile time via
`minijinja-embed` -- changes to `templates/` require a rebuild.

```bash
cargo build --release
cargo test                        # all tests (unit + integration)
cargo test --test pages_test      # run a specific test file
cargo test test_server_starts     # run a single test by name
```

## License

MIT. See [LICENSE](LICENSE).

## Attribution

This project is based on [mdserve](https://github.com/jfernandez/mdserve)
by Jose Fernandez, licensed under MIT. The original contributors are listed
in the git history.
