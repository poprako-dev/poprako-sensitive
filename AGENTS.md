# poprako-sensitive

The root package contains `Sensitive` and reexports. `poprako-sensitive-macro`
contains the derive macros. Keep both packages independently packageable.

- Follow plan → implement → review; preserve unrelated changes.
- Keep Rust files below 600 lines. Do not create `mod.rs` or use `if ... else`.
- Keep comments in English and blank lines between statements.
- Do not change lint policy without explicit permission.
- All packages inherit the workspace lint policy.
- After Rust changes, run `just fmt-check`, `just check`, and `just clippy`,
  plus affected tests. Use `just fmt` to apply formatting.
- Every commit must pass `sh scripts/ci-local.sh` through the pre-commit hook;
  never bypass hooks. Install it with `git config core.hooksPath scripts/githooks`.
- CI runs checked-in POSIX shell scripts directly.
- Publication is paused. Do not publish to crates.io or add automatic publishing
  until the user explicitly authorizes it.
