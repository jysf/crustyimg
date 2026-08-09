//! STAGE-039 D-3: every operation name `docs/data-model.md` shows in a recipe must
//! actually resolve, so the worked example cannot drift back into advertising ops the
//! registry does not have.
//!
//! It had drifted: the example named `unsharp`, `watermark` and `clean-gps`, none of
//! which exist — `unsharp` and `clean-gps` are unimplemented and `watermark` is
//! **deliberately** unregistered (`src/operation/mod.rs`, `src/cli/ops.rs`) because it
//! is a CLI verb rather than a recipe step. A reader following the example got
//! `unknown operation` on three of five steps.
//!
//! The doc carried a hedge — *"Operation names are illustrative"* — which is exactly
//! the shape this repo keeps getting caught by: prose that reads like a caveat while
//! standing in for a check nobody runs ([[a-citation-looks-like-prose-not-a-claim]]).
//! A human re-reading the doc is the wrong instrument; a grep is
//! ([[mechanical-sweeps-need-a-mechanical-check]]).
//!
//! Sits beside `tests/adoption_glue.rs` and `tests/demo_copy.rs`, which police other
//! documentation claims the same way.

use crustyimg::operation::registry::{OperationRegistry, RegistryError};
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// The reserved terminal marker (SPEC-085 / SPEC-111). It legitimately appears as
/// `op = "optimize"` in a recipe and is legitimately **absent** from the registry —
/// it yields bytes plus a format choice, not a transformed `Image`, so the apply and
/// build paths strip it before `build_pipeline`. Exempted **by name**, not by
/// skipping unknown names generally: a genuinely bogus op must still fail this test.
const RESERVED_NON_REGISTRY_OPS: &[&str] = &["optimize"];

/// Is `name` absent from the registry?
///
/// Keys on [`RegistryError::Unknown`] specifically, **not** on "`build` returned an
/// error". The distinction is load-bearing and this test caught itself getting it
/// wrong: `resize` is registered, but building it with empty params yields
/// `InvalidParams`, so an any-error check reported a registered op as missing. The
/// question here is whether the doc names something the registry has never heard of —
/// not whether the doc's illustrative params happen to be complete.
fn is_unknown_op(registry: &OperationRegistry, name: &str) -> bool {
    matches!(
        registry.build(name, &Default::default()),
        Err(RegistryError::Unknown { .. })
    )
}

/// Every `op = "..."` value in a doc, in file order, with duplicates kept so the
/// failure message can point at how often a bad name appears.
fn op_names(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix("op")?;
            let rest = rest.trim_start().strip_prefix('=')?;
            let rest = rest.trim_start().strip_prefix('"')?;
            let end = rest.find('"')?;
            Some(rest[..end].to_string())
        })
        .collect()
}

/// D-3. Every op the worked example names resolves against the real registry.
///
/// Asserts against `with_builtins()` rather than a hand-copied list, so adding an
/// operation cannot make this test stale in the permissive direction.
#[test]
fn every_op_named_in_data_model_resolves() {
    let doc = std::fs::read_to_string(Path::new(ROOT).join("docs/data-model.md"))
        .expect("read docs/data-model.md");
    let names = op_names(&doc);

    // Positive control: the parser must actually be finding the names we know are
    // there. A silently-empty sweep would make this test vacuous and green
    // ([[a-harness-that-exercises-nothing-reports-green]]).
    //
    // Asserts the specific expected names rather than a count. A count is the weaker
    // control and this test proved it: the threshold was first written as `>= 3`
    // against the old five-step example, and correcting the doc to two steps tripped
    // it — a false alarm that says nothing about whether the parser works. Naming the
    // steps ties the control to content that only changes when the example does.
    for expected in ["auto-orient", "resize"] {
        assert!(
            names.iter().any(|n| n == expected),
            "parser did not find `op = \"{expected}\"` in the worked example — the \
             parser is broken, or the example no longer contains it. Parsed: {names:?}"
        );
    }

    let registry = OperationRegistry::with_builtins();
    let unresolved: Vec<&String> = names
        .iter()
        .filter(|n| !RESERVED_NON_REGISTRY_OPS.contains(&n.as_str()))
        .filter(|n| is_unknown_op(&registry, n))
        .collect();

    assert!(
        unresolved.is_empty(),
        "docs/data-model.md names operations that do not resolve: {unresolved:?}. \
         Either the doc is advertising something the registry lacks, or a genuinely \
         reserved non-registry marker needs adding to RESERVED_NON_REGISTRY_OPS."
    );
}

/// The reserved marker is exempted **by name**, and the exemption is narrow: a name
/// that is neither registered nor reserved must still be caught. Without this, the
/// test above could be satisfied by widening the exemption list rather than fixing
/// the doc ([[test-a-carve-out-additively-not-just-by-replacement]]).
#[test]
fn the_reserved_exemption_does_not_hide_a_bogus_op() {
    let registry = OperationRegistry::with_builtins();

    // The marker really is absent from the registry — that is why it needs exempting.
    assert!(
        is_unknown_op(&registry, "optimize"),
        "`optimize` is expected to be a reserved NON-registry marker; if it became a \
         real operation, drop it from RESERVED_NON_REGISTRY_OPS"
    );

    // And a made-up name is neither registered nor reserved, so the filter above
    // would surface it.
    let bogus = "definitely-not-an-operation";
    assert!(!RESERVED_NON_REGISTRY_OPS.contains(&bogus));
    assert!(is_unknown_op(&registry, bogus));

    // And the discrimination the helper exists for: a REGISTERED op whose params are
    // incomplete is not "unknown". Without this, an any-error check would report
    // `resize` as missing from the registry — which it is not.
    assert!(
        !is_unknown_op(&registry, "resize"),
        "`resize` is registered; empty params make it InvalidParams, not Unknown"
    );
}

/// The three ops the worked example used to advertise are gone from the doc.
///
/// Named individually rather than checked as a set, so a failure says which one came
/// back. `watermark` is the subtle one: it is a real CLI verb, so it may legitimately
/// appear in prose about the `watermark` command — this asserts only that it is not
/// used as a recipe `op`, which is what `every_op_named_in_data_model_resolves`
/// already enforces, and additionally that the dead `--unsharp` flag is gone.
#[test]
fn the_removed_flags_are_not_advertised() {
    let doc = std::fs::read_to_string(Path::new(ROOT).join("docs/data-model.md"))
        .expect("read docs/data-model.md");

    for flag in ["--unsharp", "--watermark"] {
        assert!(
            !doc.contains(flag),
            "docs/data-model.md still advertises the `{flag}` CLI flag, which does not exist"
        );
    }
}
