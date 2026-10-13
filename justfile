fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

check:
    cargo check --workspace --all-targets --all-features

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

ci:
    sh scripts/ci-local.sh
