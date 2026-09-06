//! AC-7 (SPEC-128): the registry seam widened for `watermark` takes a SECOND
//! asset-bearing operation with NO further change to `src/operation/registry.rs`
//! or the resolve signature. Registers a fixture op (not the real `.cube` LUT
//! op — that stays out of scope per Call 4) through the exact same public
//! primitives `watermark` uses: `OperationRegistry::register_with_assets`,
//! `OperationRegistry::asset_keys`, and `OperationParams::set_resolved_bytes`/
//! `resolved_bytes`. If this test needed to touch `registry.rs` to pass, the
//! seam did not generalize.

use std::collections::BTreeMap;

use tempfile::TempDir;

use crustyimg::image::Image;
use crustyimg::operation::{Operation, OperationError, OperationParams, OperationRegistry, RegistryError};

/// A minimal op whose only param, `asset`, names a file. `apply` is
/// irrelevant to this test (identity) — what's under test is CONSTRUCTION via
/// the resolve path, not pixel behavior. `params()` echoes the resolved bytes
/// back as a string ONLY so this test can observe that they made it through
/// the constructor — a real op must never do this (bytes must never reach
/// `params()`/`to_toml`, SPEC-128's own highest-consequence guard); this is a
/// test-local fixture, not a pattern to copy.
struct FixtureAssetOp {
    resolved_as_text: String,
}

impl Operation for FixtureAssetOp {
    fn name(&self) -> &'static str {
        "fixture-asset"
    }

    fn params(&self) -> OperationParams {
        OperationParams::from_map({
            let mut m = BTreeMap::new();
            m.insert(
                "resolved_as_text".to_owned(),
                toml::Value::String(self.resolved_as_text.clone()),
            );
            m
        })
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        Ok(img)
    }
}

#[test]
fn a_second_asset_op_registers_without_seam_change() {
    let mut registry = OperationRegistry::new();
    registry.register_with_assets(
        "fixture-asset",
        |params| {
            let bytes = params.resolved_bytes("asset").ok_or_else(|| {
                RegistryError::InvalidParams {
                    op: "fixture-asset",
                    reason: "'asset' was not resolved before construction".to_owned(),
                }
            })?;
            Ok(Box::new(FixtureAssetOp {
                resolved_as_text: String::from_utf8_lossy(bytes).into_owned(),
            }))
        },
        &["asset"],
    );

    // The seam's query: this op declares exactly one asset key, and an
    // ordinary (non-asset) op declares none — additive, not disruptive.
    assert_eq!(registry.asset_keys("fixture-asset"), &["asset"]);
    assert!(registry.asset_keys("identity").is_empty());

    // Simulate EXACTLY what the CLI's recipe-IO-boundary resolver does
    // (`cli::common::resolve_recipe_assets`): read the file the step's param
    // names, and attach the bytes via the SAME public method watermark's
    // resolver uses.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("asset.bin");
    std::fs::write(&path, b"fixture-asset-bytes").unwrap();

    let mut params = OperationParams::from_map({
        let mut m = BTreeMap::new();
        m.insert(
            "asset".to_owned(),
            toml::Value::String(path.to_string_lossy().into_owned()),
        );
        m
    });
    for &key in registry.asset_keys("fixture-asset") {
        if let Some(p) = params.get_str(key).map(str::to_owned) {
            let bytes = std::fs::read(&p).expect("fixture file must be readable");
            params.set_resolved_bytes(key, bytes);
        }
    }

    let op = registry
        .build("fixture-asset", &params)
        .expect("fixture op must build via the SAME resolve path watermark uses");
    assert_eq!(op.name(), "fixture-asset");

    // Confirm the resolved bytes actually reached the constructor (not just
    // that SOME op was returned) — see the struct's doc for why `params()`
    // echoing them back is a test-only pattern, not one to copy.
    assert_eq!(
        op.params().get_str("resolved_as_text"),
        Some("fixture-asset-bytes")
    );
}
