//! Compiles the wrapper source with debug assertions both enabled and disabled.

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Removes temporary compiler fixtures even when an assertion fails.
struct FixtureDirectory(PathBuf);

impl Drop for FixtureDirectory {
    // Cleanup failure does not obscure the original compiler/test failure.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn wrapper_modes_preserve_and_redact_values() -> Result<(), Box<dyn Error>> {
    //
    let directory = FixtureDirectory(
        std::env::temp_dir()
            .join(format!("poprako-sensitive-wrapper-{}", std::process::id())),
    );

    fs::create_dir_all(&directory.0)?;

    let wrapper =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/sensitive.rs");

    let source = format!(
        "#[path = {wrapper:?}] mod sensitive;\n{}",
        include_str!("wrapper.txt"),
    );

    let source_path = directory.0.join("fixture.rs");

    fs::write(&source_path, source)?;

    for assertions in ["yes", "no"] {
        //
        let executable = directory.0.join(format!("fixture-{assertions}"));

        let compiled = Command::new("rustc")
            .arg("--edition=2024")
            .arg("--test")
            .arg("-Dwarnings")
            .arg("-C")
            .arg(format!("debug-assertions={assertions}"))
            .arg(&source_path)
            .arg("-o")
            .arg(&executable)
            .output()?;

        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );

        let tested = Command::new(executable).output()?;

        assert!(
            tested.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&tested.stdout),
            String::from_utf8_lossy(&tested.stderr)
        );
    }

    Ok(())
}
