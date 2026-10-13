#!/bin/sh
set -eu

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_root"

cargo fmt --all --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo doc --workspace --all-features --no-deps --locked
