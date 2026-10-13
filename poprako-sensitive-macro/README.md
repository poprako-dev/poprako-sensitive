# poprako-sensitive-macro

Procedural derive macros for
[poprako-sensitive](https://github.com/poprako-dev/poprako-sensitive).
Prefer the `SensitiveDebug` and `SensitiveDisplay` reexports from the main crate.

Both derives support named, tuple, unit, and generic structs. Mark a field with
`#[sensitive]` for default redaction, or use
`#[sensitive(debug = "module::debug_mask", display = "module::display_mask")]`
for independent formatting callbacks. Empty or omitted paths select the default.

Callbacks have the signature
`fn(&FieldType, &mut core::fmt::Formatter<'_>) -> core::fmt::Result` and run only
when the consuming crate has debug assertions disabled. With debug assertions
enabled, the original value is formatted. The generated implementations select
these configurations with `#[cfg]`.

Rust 1.95 or newer is required. Publication is pending; this package is not yet
released to crates.io. Licensed under MIT; see [LICENSE](LICENSE).
