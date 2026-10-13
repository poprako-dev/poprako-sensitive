//! Sensitive formatting composes with tracing fields and instrumented spans.

use poprako_sensitive::{Sensitive, SensitiveDebug};
use tracing::instrument;

#[derive(SensitiveDebug)]
struct Credentials<'a> {
    user_id: &'a str,
    #[sensitive]
    password: &'a str,
}

// Payload size is a separate concern: exclude its bytes in every configuration.
#[instrument(skip(payload), fields(payload_bytes = payload.len()))]
fn submit(credentials: &Credentials<'_>, payload: &[u8]) {
    tracing::info!(
        user_id = credentials.user_id,
        access_token = %Sensitive::new("example-access-token"),
        "accepted example request",
    );
}

// Keep output deterministic so the fields are easy to inspect.
fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_ansi(false)
        .init();

    let credentials = Credentials {
        user_id: "user-42",
        password: "example-password",
    };

    let payload = b"payload-that-must-not-be-logged".repeat(4096);

    tracing::info!(
        debug_assertions = cfg!(debug_assertions),
        credentials = ?credentials,
        "starting tracing example",
    );

    submit(&credentials, &payload);
}
