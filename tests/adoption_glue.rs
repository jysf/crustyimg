//! Verifies the in-repo CI adoption glue (SPEC-057): the `.pre-commit-hooks.yaml`
//! hook, the `just lint-images` recipe, and the README Continuous-integration
//! section. Behavior of the linter the recipe wraps is confirmed by driving the
//! real binary over fixture trees (the same exit codes `just lint-images` yields).
//!
//! The two composite Actions (`setup-crustyimg`, `crustyimg-action`) are separate
//! repos validated by their own 3-OS self-test workflows; only the in-repo glue is
//! testable here.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

mod common;
use common::{jpeg_with_gps, solid_png};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read(rel: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn pre_commit_hooks_defines_the_crustyimg_lint_hook() {
    let y = read(".pre-commit-hooks.yaml");
    assert!(y.contains("id: crustyimg-lint"), "hook id");
    assert!(y.contains("entry: crustyimg lint"), "runs `crustyimg lint`");
    assert!(y.contains("language: rust"), "builds via cargo");
    assert!(y.contains("types: [image]"), "runs on image files");
}

#[test]
fn justfile_has_a_lint_images_recipe() {
    let j = read("justfile");
    // The recipe is declared with a `paths` parameter and wraps `crustyimg lint`.
    assert!(
        j.contains("lint-images *paths"),
        "lint-images recipe with a paths parameter"
    );
    assert!(
        j.contains("-- lint {{paths}}"),
        "lint-images must invoke `crustyimg lint`"
    );
}

#[test]
fn readme_documents_the_ci_on_ramp() {
    let r = read("README.md");
    assert!(r.contains("## Continuous integration"), "CI section");
    assert!(r.contains("jysf/setup-crustyimg"), "setup Action");
    assert!(r.contains("jysf/crustyimg-action"), "lint Action");
    assert!(r.contains("crustyimg lint"), "a lint invocation");
    assert!(r.contains(".pre-commit-hooks.yaml"), "pre-commit reference");
}

/// The exit-code contract `just lint-images` relies on: `0` on a clean tree, `7`
/// on an error finding (a GPS leak). Driven through the real binary — the recipe
/// is a thin `cargo run -- lint {{paths}}` wrapper over exactly this.
#[test]
fn lint_gate_exit_codes_over_fixture_trees() {
    let clean = TempDir::new().unwrap();
    std::fs::write(clean.path().join("a.png"), solid_png(4, 4, [1, 2, 3])).unwrap();
    let clean_code = Command::new(BIN)
        .arg("lint")
        .arg(clean.path())
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert_eq!(clean_code, 0, "clean tree → exit 0");

    let leaky = TempDir::new().unwrap();
    std::fs::write(leaky.path().join("leak.jpg"), jpeg_with_gps(16, 16)).unwrap();
    let leaky_code = Command::new(BIN)
        .arg("lint")
        .arg(leaky.path())
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert_eq!(leaky_code, 7, "GPS-leaking tree → exit 7");
}
