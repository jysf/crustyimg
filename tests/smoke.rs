//! Integration smoke tests for the `crustyimg` binary (SPEC-001).
//!
//! The binary is located the Cargo-native way via `CARGO_BIN_EXE_crustyimg`
//! (no `assert_cmd`/`escargot` dependency). Output is trimmed so a trailing
//! `\n` / Windows `\r\n` does not break assertions.

use std::process::Command;

/// Path to the compiled `crustyimg` binary, provided by Cargo to integration
/// tests.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// True if `s` begins with a `MAJOR.MINOR.PATCH` semver core (each numeric),
/// without pulling in a regex crate.
fn starts_with_semver(s: &str) -> bool {
    let mut parts = s.split('.');
    let major = parts.next();
    let minor = parts.next();
    let patch = parts.next();
    match (major, minor, patch) {
        (Some(a), Some(b), Some(c)) => {
            // The patch segment may carry a pre-release/build suffix
            // (e.g. "0-rc1"); only the leading numeric run must parse.
            let patch_num: String = c.chars().take_while(|ch| ch.is_ascii_digit()).collect();
            !a.is_empty()
                && a.chars().all(|ch| ch.is_ascii_digit())
                && !b.is_empty()
                && b.chars().all(|ch| ch.is_ascii_digit())
                && !patch_num.is_empty()
        }
        _ => false,
    }
}

#[test]
fn version_flag_prints_semver() {
    let output = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("failed to run crustyimg --version");
    assert!(output.status.success(), "expected exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    assert!(
        starts_with_semver(trimmed),
        "stdout {trimmed:?} is not a MAJOR.MINOR.PATCH semver"
    );
}

#[test]
fn version_short_flag_matches_long() {
    let long = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("failed to run crustyimg --version");
    let short = Command::new(BIN)
        .arg("-V")
        .output()
        .expect("failed to run crustyimg -V");
    assert!(short.status.success(), "expected -V to exit 0");
    assert_eq!(
        String::from_utf8_lossy(&long.stdout).trim(),
        String::from_utf8_lossy(&short.stdout).trim(),
        "-V output should match --version output"
    );
}

#[test]
fn version_matches_cargo_pkg_version() {
    let output = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("failed to run crustyimg --version");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        env!("CARGO_PKG_VERSION"),
        "printed version should equal the package version"
    );
}

#[test]
fn help_flag_exits_zero_and_names_binary() {
    let output = Command::new(BIN)
        .arg("--help")
        .output()
        .expect("failed to run crustyimg --help");
    assert!(output.status.success(), "expected exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("crustyimg"),
        "help output should name the binary, got {stdout:?}"
    );
}

#[test]
fn unknown_invocation_exits_nonzero_on_stderr() {
    let output = Command::new(BIN)
        .arg("bogus-subcommand")
        .output()
        .expect("failed to run crustyimg bogus-subcommand");
    assert!(
        !output.status.success(),
        "unknown invocation should exit non-zero"
    );
    assert!(
        output.stdout.is_empty(),
        "stdout should stay clean on error"
    );
    assert!(!output.stderr.is_empty(), "diagnostics should go to stderr");
}
