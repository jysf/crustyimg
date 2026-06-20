//! Integration tests for the `crustyimg completions` subcommand (SPEC-040).
//!
//! Mirrors the `tests/cli.rs` harness: all tests drive the real compiled binary
//! via `env!("CARGO_BIN_EXE_crustyimg")` and `std::process::Command`.
//! No `assert_cmd` dep — pure stdlib.

use std::process::Command;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Trim stdout bytes to a `String`, stripping leading/trailing whitespace.
fn stdout_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// `completions bash` exits 0, stdout is non-empty, and contains both
/// `crustyimg` and `_crustyimg` (the generated bash completion function name).
#[test]
fn completions_bash_emits_script() {
    let output = Command::new(BIN)
        .args(["completions", "bash"])
        .output()
        .expect("failed to run crustyimg completions bash");

    assert_eq!(
        output.status.code(),
        Some(0),
        "completions bash should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );

    let stdout = stdout_str(&output);
    assert!(
        !stdout.is_empty(),
        "completions bash stdout must be non-empty"
    );
    assert!(
        stdout.contains("crustyimg"),
        "bash script should reference the binary name 'crustyimg'; got:\n{stdout}"
    );
    assert!(
        stdout.contains("_crustyimg"),
        "bash script should contain completion function '_crustyimg'; got:\n{stdout}"
    );
}

/// `completions <shell>` for each of the five supported shells exits 0 and
/// produces non-empty stdout.
#[test]
fn completions_all_shells_succeed() {
    let shells = ["bash", "zsh", "fish", "powershell", "elvish"];
    for shell in shells {
        let output = Command::new(BIN)
            .args(["completions", shell])
            .output()
            .unwrap_or_else(|e| panic!("failed to run crustyimg completions {shell}: {e}"));

        assert_eq!(
            output.status.code(),
            Some(0),
            "completions {shell} should exit 0; stderr: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );

        let stdout = stdout_str(&output);
        assert!(
            !stdout.is_empty(),
            "completions {shell} stdout must be non-empty"
        );
    }
}

/// `completions klingon` exits 2 (clap usage error — not a valid `Shell`
/// variant) and writes nothing useful to stdout.
#[test]
fn completions_rejects_unknown_shell() {
    let output = Command::new(BIN)
        .args(["completions", "klingon"])
        .output()
        .expect("failed to run crustyimg completions klingon");

    assert_eq!(
        output.status.code(),
        Some(2),
        "completions with unknown shell should exit 2 (clap usage error)"
    );
}

/// `completions zsh` succeeds with NO positional path argument (it needs no
/// input file) and produces no output *file* — it writes purely to stdout.
#[test]
fn completions_needs_no_input_path() {
    let output = Command::new(BIN)
        .args(["completions", "zsh"])
        .output()
        .expect("failed to run crustyimg completions zsh");

    assert_eq!(
        output.status.code(),
        Some(0),
        "completions zsh with no path arg should exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );

    let stdout = stdout_str(&output);
    assert!(
        !stdout.is_empty(),
        "completions zsh stdout must be non-empty"
    );

    // Confirm no file was written to the current directory by the command.
    // (We check that no file named after crustyimg shells was unexpectedly
    // created in the working directory — stdout-only contract.)
    assert!(
        !std::path::Path::new("_crustyimg").exists(),
        "completions zsh must NOT write a static file to the filesystem"
    );
}
