//! Thin `crustyimg` entrypoint (SPEC-001 scaffold).
//!
//! No clap yet (that is SPEC-007) — a hand-rolled argv match keeps the
//! dependency set empty. Diagnostics go to stderr; the binary never panics
//! on a recoverable path (DEC-007): unknown args produce a friendly stderr
//! message and a non-zero exit via `ExitCode`, not a panic.

use std::process::ExitCode;

/// Usage exit code, matching the api-contract's "usage error" convention.
const EXIT_USAGE: u8 = 2;

const USAGE: &str = "Usage: crustyimg [--version|-V] [--help|-h]";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("--version") | Some("-V") => {
            println!("{}", crustyimg::version());
            ExitCode::SUCCESS
        }
        Some("--help") | Some("-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        other => {
            match other {
                Some(arg) => eprintln!("crustyimg: unrecognized argument '{arg}'"),
                None => eprintln!("crustyimg: no command given"),
            }
            eprintln!("{USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}
