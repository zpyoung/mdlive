#!/bin/bash
set -euo pipefail

# read current base version from Cargo.toml
BASE=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# increment dev build counter
COUNTER_FILE=".dev-build-number"
if [ -f "$COUNTER_FILE" ]; then
    N=$(cat "$COUNTER_FILE")
    N=$((N + 1))
else
    N=1
fi
echo "$N" > "$COUNTER_FILE"

DEV_VERSION="${BASE}-dev.${N}"
echo "Building dev version: $DEV_VERSION"

# semver pre-release tag for dev builds
sed -i '' "s/\"version\": \".*\"/\"version\": \"$DEV_VERSION\"/" src-tauri/tauri.conf.json

# build CLI then Tauri app
cargo build -p mdlive --release
cargo tauri build 2>&1

# restore tauri.conf.json
sed -i '' "s/\"version\": \".*\"/\"version\": \"$BASE\"/" src-tauri/tauri.conf.json

echo ""
echo "DMG: target/release/bundle/dmg/mdlive_${DEV_VERSION}_aarch64.dmg"
