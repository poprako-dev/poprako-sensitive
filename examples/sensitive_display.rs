//! Custom Display redaction and the empty-path default.

#[cfg(not(debug_assertions))]
use core::fmt::{Formatter, Result};

use poprako_sensitive::SensitiveDisplay;

// Only the redacted implementation references this callback.
#[cfg(not(debug_assertions))]
fn mask_email(email: &&str, formatter: &mut Formatter<'_>) -> Result {
    let domain = email.split_once('@').map_or("hidden", |(_, domain)| domain);
    write!(formatter, "***@{domain}")
}

#[derive(SensitiveDisplay)]
struct Contact<'a> {
    name: &'a str,
    #[sensitive(display = "mask_email")]
    email: &'a str,
    #[sensitive(display = "")]
    recovery_code: &'a str,
}

// Custom and default redaction can be combined in the same value.
#[allow(
    clippy::print_stdout,
    reason = "This runnable example demonstrates formatter output on stdout"
)]
fn main() {
    let contact = Contact {
        name: "Reader",
        email: "reader@example.com",
        recovery_code: "example-recovery-code",
    };
    println!("debug_assertions = {}", cfg!(debug_assertions));
    println!("{contact}");
    assert_eq!(contact.email, "reader@example.com");
}
