//! Custom Display redaction and the empty-path default.

use core::fmt::{Display, Formatter, Result};

use poprako_sensitive::SensitiveDisplay;

struct Email {
    local: String,
    domain: String,
}

impl Display for Email {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}@{}", self.local, self.domain)
    }
}

// The callback receives the original Email, not its Display output.
#[cfg(not(debug_assertions))]
fn mask_email(email: &Email, formatter: &mut Formatter<'_>) -> Result {
    write!(formatter, "***@{}", email.domain)
}

#[derive(SensitiveDisplay)]
struct Contact<'a> {
    name: &'a str,
    #[sensitive(display = "mask_email")]
    email: Email,
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
        email: Email {
            local: "reader".into(),
            domain: "example.com".into(),
        },
        recovery_code: "example-recovery-code",
    };

    println!("debug_assertions = {}", cfg!(debug_assertions));
    println!("{contact}");

    assert_eq!(contact.email.local, "reader");
    assert_eq!(contact.email.domain, "example.com");
}
