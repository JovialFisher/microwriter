#!/usr/bin/env bash
set -e

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
cd "$ROOT_DIR"

if [ -x "target/release/microwriter" ]; then
    exec "target/release/microwriter"
elif [ -x "target/debug/microwriter" ]; then
    exec "target/debug/microwriter"
elif command -v cargo >/dev/null 2>&1; then
    exec cargo run --release
else
    printf '%s\n' "Microwriter is not built and Cargo was not found on PATH." >&2
    printf '%s\n' "Install Rust/Cargo or build the app, then run this launcher again." >&2
    exit 1
fi
