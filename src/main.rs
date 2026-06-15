//! Thin `crustyimg` entrypoint (SPEC-007).
//!
//! Delegates entirely to `crustyimg::cli::run()`. The clap subcommand surface,
//! dispatch, and exit-code mapping live in `src/cli/` (DEC-012, DEC-007).

use std::process::ExitCode;

fn main() -> ExitCode {
    crustyimg::cli::run()
}
