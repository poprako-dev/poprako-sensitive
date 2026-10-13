# poprako-sensitive

Declarative redaction for Rust's `Debug` and `Display`.
With `debug_assertions` enabled, values remain visible; otherwise, sensitive
fields use `[REDACTED]` or a custom formatter. Stored values are unchanged.

## Declarative field redaction

```rust
use poprako_sensitive::{SensitiveDebug, SensitiveDisplay};

#[derive(SensitiveDebug, SensitiveDisplay)]
struct Login {
    user_id: String,
    #[sensitive]
    password: String,
}
```

## Custom redaction

Callbacks receive the original borrowed field and write to a formatter.

```rust
use core::fmt::{Formatter, Result};
use poprako_sensitive::SensitiveDebug;

#[derive(Debug)]
struct Email {
    local: String,
    domain: String,
}

fn mask_email(email: &Email, f: &mut Formatter<'_>) -> Result {
    write!(f, "***@{}", email.domain)
}

#[derive(SensitiveDebug)]
struct Account {
    #[sensitive(debug = "mask_email")]
    email: Email,
}
```

Use `debug = "path"` for `SensitiveDebug` and `display = "path"` for
`SensitiveDisplay`. Both are optional; omitted or empty paths use default
redaction. Callbacks run when debug assertions are disabled.

## Manual wrapping

```rust
use poprako_sensitive::Sensitive;

let token = String::from("private-token");
let wrapped = Sensitive::new(&token);

println!("{wrapped:?}"); // Debug
println!("{wrapped}");  // Display
```

See [examples](examples) for runnable demos, including tracing integration.
