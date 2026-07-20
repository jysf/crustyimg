//! SPEC-097 verify-only guard: every item that was `pub` at
//! `crustyimg::cli::*` before the mod.rs split must still resolve at the
//! identical path after it. This file names all 11 originally-public
//! top-level paths; if any were dropped, renamed, or had its visibility
//! narrowed, this test file fails to compile — a mechanical check for a
//! mechanical claim (the re-export contract).
//!
//! The 11 items are the complete set of non-`pub(crate)`/`pub(super)`
//! top-level `pub` items in the pre-split `src/cli/mod.rs` (git main).

#![allow(unused_imports)]

// Types / enums — naming them in a `use` proves the path resolves and is
// visible from outside the crate.
use crustyimg::cli::AutoQuality;
use crustyimg::cli::Cli;
use crustyimg::cli::CliError;
use crustyimg::cli::Commands;
use crustyimg::cli::ExplainFmt;
use crustyimg::cli::GlobalArgs;
use crustyimg::cli::MetaCommand;
use crustyimg::cli::ProfileArg;
use crustyimg::cli::QualityTarget;

#[test]
fn all_prior_public_cli_paths_still_resolve() {
    // Function item: bind it to prove `crustyimg::cli::run` still exists with
    // the same `fn() -> ExitCode` shape.
    let _run: fn() -> std::process::ExitCode = crustyimg::cli::run;

    // Const item: reference it (and pin its value, which is part of the
    // preserved public contract — see SPEC-088/DEC-070).
    const _EDGE: u32 = crustyimg::cli::WEB_DEFAULT_LONG_EDGE;
    assert_eq!(crustyimg::cli::WEB_DEFAULT_LONG_EDGE, 2048);

    // Touch each type in type position so the imports above are load-bearing
    // even under a future `-D unused_imports`; `None::<T>` names the path
    // without needing a constructor.
    let _ = None::<Cli>;
    let _ = None::<GlobalArgs>;
    let _ = None::<QualityTarget>;
    let _ = None::<ProfileArg>;
    let _ = None::<ExplainFmt>;
    let _ = None::<AutoQuality>;
    let _ = None::<Commands>;
    let _ = None::<MetaCommand>;
    let _ = None::<CliError>;
}
