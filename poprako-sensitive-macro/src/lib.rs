//! Derive support for source-declared sensitive struct fields.

// Generate the consumer's Debug implementations.
mod debug;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

/// Implements Debug for a struct, replacing `#[sensitive]` fields with
/// `[REDACTED]` when the consuming crate has debug assertions disabled.
///
/// Supports named, tuple, and unit structs, including generics. Do not also
/// derive Debug. Field types need Debug only in configurations that show them.
/// Optional `debug = "path"` selects a redaction callback with signature
/// `fn(&FieldType, &mut core::fmt::Formatter<'_>) -> core::fmt::Result`.
/// Empty or omitted paths use default redaction. Callbacks run only without
/// debug assertions; `display` metadata is reserved for `SensitiveDisplay`.
#[proc_macro_derive(SensitiveDebug, attributes(sensitive))]
pub fn sensitive_debug(input: TokenStream) -> TokenStream {
    //
    debug::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Implements Display using the struct's named or tuple layout.
///
/// Visible fields use Display. Sensitive fields use `[REDACTED]` without
/// debug assertions, or the optional `display = "path"` callback with signature
/// `fn(&FieldType, &mut core::fmt::Formatter<'_>) -> core::fmt::Result`.
/// Empty or omitted paths use default redaction; `debug` is independent.
#[proc_macro_derive(SensitiveDisplay, attributes(sensitive))]
pub fn sensitive_display(input: TokenStream) -> TokenStream {
    debug::expand_format(input.into(), true)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
