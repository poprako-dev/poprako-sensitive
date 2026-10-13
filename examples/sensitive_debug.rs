//! Default field redaction with a bare sensitive marker.

use poprako_sensitive::SensitiveDebug;

#[derive(SensitiveDebug)]
struct Login<'a> {
    user_id: &'a str,
    #[sensitive]
    password: &'a str,
}

// Show both ordinary and alternate Debug without changing the stored password.
#[allow(
    clippy::print_stdout,
    reason = "This runnable example demonstrates formatter output on stdout"
)]
fn main() {
    let login = Login {
        user_id: "user-42",
        password: "example-password",
    };
    println!("debug_assertions = {}", cfg!(debug_assertions));
    println!("{login:?}");
    println!("{login:#?}");
    assert_eq!(login.password, "example-password");
}
