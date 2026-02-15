# Changelog

All notable changes to this project will be documented in this file.

## [2.0.0] - 2026-02-15

### Refactoring

- Renamed package and binary from `mdserve` to `mdlive`
- Split monolithic `app.rs` (1082 lines) into focused modules: state, router, handlers, watcher, tree, template, util
- Moved integration tests to `tests/` directory, unit tests inline with modules
- New repo at [bearded-giant/mdlive](https://github.com/bearded-giant/mdlive), independent from the upstream fork

### Features

- Context menu with copy raw markdown, move, delete, new file (from prior work)
- Collapsible directory tree sidebar (from prior work)
- Syntax highlighting via highlight.js (from prior work)
