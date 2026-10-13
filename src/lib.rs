//! Declarative Debug redaction and explicit formatting of sensitive values.
//!
//! Values are visible with debug assertions enabled and replaced by
//! `[REDACTED]` otherwise, unless a field declares a custom redaction callback.
//! Serialization and stored values are unchanged.
//!
//! Custom callbacks write directly to the formatter and only run when debug
//! assertions are disabled. Both keys are optional; `""` selects the default.
//!
//! ```
//! use core::fmt::{Formatter, Result};
//! use poprako_sensitive::{SensitiveDebug, SensitiveDisplay};
//!
//! fn mask_email(value: &str, formatter: &mut Formatter<'_>) -> Result {
//!     let domain = value.split_once('@').map_or("hidden", |(_, domain)| domain);
//!
//!     write!(formatter, "***@{domain}")
//! }
//!
//! #[derive(SensitiveDebug, SensitiveDisplay)]
//! struct Account {
//!     #[sensitive(debug = "mask_email", display = "mask_email")]
//!     email: String,
//! }
//! ```

// Explicit formatting for values without a source-declared sensitive field.
mod sensitive;

pub use poprako_sensitive_macro::{SensitiveDebug, SensitiveDisplay};

pub use crate::sensitive::Sensitive;
