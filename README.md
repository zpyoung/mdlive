![mdlive](static/img/md-hero.png)

# mdlive

A live markdown viewer I built to sit next to AI coding agents. When an agent writes markdown -- plans, architecture docs, research -- I'd rather read it rendered in a browser than squint at raw text in a terminal.

This started as a fork of Jose Fernandez's [mdserve](https://github.com/jfernandez/mdserve). The original idea and core implementation are his. I've since taken it in a different direction: modular architecture, file operations via context menu, recursive directory trees, syntax highlighting. Different enough to warrant its own repo.

## The idea

Point it at a file or directory, get a live-reloading preview in the browser. That's it. No config files, no setup, no flags required. The agent writes, you read.

```bash
mdlive README.md         # single file
mdlive docs/             # whole directory with sidebar
```

It watches for changes and reloads instantly via WebSocket. New files in directory mode get picked up automatically.

## What it does

GFM rendering (tables, task lists, fenced code blocks), Mermaid diagrams, syntax highlighting with highlight.js, five themes including Catppuccin variants, and a right-click context menu for file operations -- copy raw markdown, move, rename, delete, create new files. Directory mode gives you a collapsible tree sidebar that handles nested subdirectories.

## What it doesn't do

This isn't a documentation site generator or a static server you deploy somewhere. It's not trying to be mdBook or Docusaurus. mdlive is ephemeral -- start it when you're working, kill it when you're done.

## Install

```bash
cargo install mdlive
```

Or build from source:

```bash
git clone https://github.com/bearded-giant/mdlive
cd mdlive
cargo build --release
```

## Usage

```bash
mdlive file.md                # serve on port 3000
mdlive docs/                  # directory mode with sidebar
mdlive docs/ -p 8080          # custom port
mdlive file.md --open         # open browser automatically
mdlive file.md -H 0.0.0.0    # bind to all interfaces
```

Single-file mode gives you a clean focused view. Directory mode recursively finds all `.md` and `.markdown` files and builds a navigable tree.

## Development

Rust 1.82+. Templates are embedded at compile time via `minijinja-embed`, so changes to `templates/` need a rebuild.

```bash
cargo test                        # everything
cargo test --test pages_test      # specific test file
cargo test test_server_starts     # by name
```

See [docs/architecture.md](docs/architecture.md) for internals.

## License

MIT. See [LICENSE](LICENSE).

## Attribution

Based on [mdserve](https://github.com/jfernandez/mdserve) by Jose Fernandez, MIT licensed. Original contributors are in the git history.
