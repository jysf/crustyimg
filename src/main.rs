//! Thin `crustyimg` entrypoint (SPEC-007).
//!
//! Delegates entirely to `crustyimg::cli::run()`. The clap subcommand surface,
//! dispatch, and exit-code mapping live in `src/cli/` (DEC-012, DEC-007).

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    crustyimg::cli::run()
}

/// The wasm build has no CLI: there is no argv and no filesystem, and its artifact
/// is the `cdylib` that `src/wasm.rs` exports — not a binary anyone runs (SPEC-072,
/// DEC-064). Cargo still *builds* the bin target when the whole crate is compiled
/// for wasm32 (`cargo build --tests`, which is what `wasm-pack test` runs), so this
/// target needs a `main` that compiles. An empty one is the honest answer; the
/// alternative — leaving `main` calling the gated-out `cli` — fails the wasm test
/// build with a confusing "cannot find `cli` in `crustyimg`".
#[cfg(target_arch = "wasm32")]
fn main() {}
