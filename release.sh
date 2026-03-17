#!/bin/bash
set -euo pipefail
V=$1
sed -i '' "s/^version = \".*\"/version = \"$V\"/" Cargo.toml src-tauri/Cargo.toml
sed -i '' "s/\"version\": \".*\"/\"version\": \"$V\"/" src-tauri/tauri.conf.json
cargo generate-lockfile
git add -A && git commit -m "bump to v$V" && git tag "v$V" && git push origin main "v$V"
