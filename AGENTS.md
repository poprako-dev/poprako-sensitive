# poprako-sensitive

The root package contains `Sensitive` and reexports. `poprako-sensitive-macro`
contains the derive macros. Keep both packages independently packageable.

- Follow plan → implement → review; preserve unrelated changes.
- Use conventional Rust formatting with the default `cargo fmt` configuration.
- Do not add empty `//` separators or require blank lines between statements.
- Keep meaningful comments and public API documentation in English.
- Do not change lint policy without explicit permission.
- All packages inherit the workspace lint policy.
- After Rust changes, run `just fmt-check`, `just check`, and `just clippy`,
  plus affected tests. Use `just fmt` to apply formatting.
- Every commit must pass `sh scripts/ci-local.sh` through the pre-commit hook;
  never bypass hooks. Install it with `git config core.hooksPath scripts/githooks`.
- CI runs checked-in POSIX shell scripts directly.
- Publishing to crates.io requires explicit user authorization and a successful
  dry run. Publish the macro package before the main package.
