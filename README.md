# poprako-sensitive

Compile-time formatting policy for sensitive Rust values, with optional custom
redaction callbacks. The workspace contains `poprako-sensitive` and its derive
implementation, `poprako-sensitive-macro`.

## Installation

The crates.io release is pending. Use the Git repository for now, preferably
with a pinned commit:

```toml
[dependencies]
poprako-sensitive = { git = "https://github.com/poprako-dev/poprako-sensitive" }
```

Rust 1.95 or newer is required. Both packages use Rust edition 2024.

## Field redaction

```rust
use poprako_sensitive::SensitiveDebug;

#[derive(SensitiveDebug)]
struct Login {
    user_id: String,
    #[sensitive]
    password: String,
}
```

`SensitiveDebug` supports named, tuple, unit, and generic structs. Ordinary
fields retain their usual Debug output. Sensitive fields follow the consuming
crate's `debug_assertions` configuration:

| Configuration | Sensitive field output |
| --- | --- |
| `debug_assertions` enabled | Original value |
| `debug_assertions` disabled | `[REDACTED]`, or a custom callback |

The generated implementations use `#[cfg]`; no runtime mode selection is needed.
The setting is `debug_assertions`, not the profile name: a custom profile can
change this behavior. Debug builds expose sensitive values by design.

## Custom formatting

```rust
use core::fmt::{Formatter, Result};
use poprako_sensitive::{SensitiveDebug, SensitiveDisplay};

fn mask_email(value: &str, formatter: &mut Formatter<'_>) -> Result {
    let domain = value.split_once('@').map_or("hidden", |(_, domain)| domain);

    write!(formatter, "***@{domain}")
}

#[derive(SensitiveDebug, SensitiveDisplay)]
struct Account {
    #[sensitive(debug = "mask_email", display = "mask_email")]
    email: String,
}
```

Callbacks take a borrowed field and a mutable formatter, returning
`core::fmt::Result`. Paths may be module-qualified. Borrow coercions work as in
ordinary Rust calls; the example accepts `&str` for a `String` field.

- `debug` is used by `SensitiveDebug`; `display` by `SensitiveDisplay`.
- Either key may be omitted or set to `""` to use default redaction.
- `#[sensitive]` and `#[sensitive()]` use default redaction for both traits.
- Callbacks run only with debug assertions disabled. They decide what to reveal.
- Formatter flags and callback errors are passed through.
- Hidden or custom-formatted fields do not need the corresponding native trait
  in that configuration. Visible fields still need Debug or Display.

`SensitiveDisplay` preserves the struct's named or tuple layout and uses Display
for visible fields. Only derive the formatting traits you actually use.

## Values without a field declaration

```rust
use poprako_sensitive::Sensitive;

let token = String::from("private-token");
let wrapped = Sensitive::new(&token);

let debug_output = format!("{wrapped:?}");
let display_output = format!("{wrapped}");
```

`Sensitive<T>` accepts owned or borrowed values and implements Debug and Display
when `T` implements the corresponding trait. Its policy follows the wrapper
crate's debug-assertion configuration. Without debug assertions it never calls
the wrapped value's formatter. Custom callbacks belong to the derive metadata.

This library changes formatting only. It does not encrypt, erase, or modify
stored values or serialization. Large payloads should be excluded explicitly at
logging call sites; size-based log exclusion is separate from redaction.

## Runnable examples

```sh
cargo run --example sensitive_debug
cargo run --example sensitive_display
cargo run --example tracing
```

- `sensitive_debug`: bare `#[sensitive]`, compact and pretty Debug, and unchanged
  stored values.
- `sensitive_display`: a custom email formatter plus an empty `display = ""`
  that selects default redaction.
- `tracing`: `?` fields, automatically captured instrument arguments, `%` with
  `Sensitive`, and explicit exclusion of a large payload in every configuration.

The default development profile exposes the example secrets. To see redaction,
run the same examples with `--release` (assuming the default release profile,
which disables debug assertions):

```sh
cargo run --release --example sensitive_debug
cargo run --release --example sensitive_display
cargo run --release --example tracing
```

The custom Display example then prints:

```text
Contact { name: Reader, email: ***@example.com, recovery_code: [REDACTED] }
```

The Debug example replaces the password with `"[REDACTED]"`. The tracing example
retains `user_id` and `payload_bytes`, redacts the password and access token,
and never logs the payload body in either configuration. All values in these
examples are synthetic.

## Development

```sh
git config core.hooksPath scripts/githooks
just ci
```

`sh scripts/ci-local.sh` runs formatting, compilation, Clippy, tests, and docs.
Tests compile fixtures with debug assertions both enabled and disabled.
GitHub Actions runs the same script. No publishing workflow is installed.

Both manifests currently inherit `publish = false`. After API standardization
and explicit release approval, enable publishing, publish the macro package
first, then publish the main package. Its macro dependency already has a version
requirement alongside the local development path.

## License

MIT. See [LICENSE](LICENSE).
