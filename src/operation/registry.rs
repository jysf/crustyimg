//! Operation registry — maps operation names to constructor functions (DEC-005).
//!
//! The registry is the single seam new operations register at. Both the CLI
//! (future, SPEC-007) and the recipe loader (`src/recipe/`) construct
//! `Box<dyn Operation>` through it — which is exactly what makes recipes
//! round-trip. New ops register here; nothing else changes (DEC-005).
//!
//! Layering: this module depends only on `super` (the `Operation` trait and
//! `OperationParams`) and `std`. It must NOT depend on `recipe`, `clap`,
//! `source`, `sink`, files, or terminals.

use std::collections::HashMap;

use thiserror::Error;

use super::{AutoOrient, Identity, Invert, Operation, OperationParams, Resize, Watermark};

// ─── Constructor type alias ─────────────────────────────────────────────────

/// A plain function-pointer constructor: given params, produce a boxed op.
///
/// A `fn` pointer is sufficient for all built-in ops. Later-stage ops that
/// need params read them from `&OperationParams` inside their constructor.
/// The uniform signature keeps the registry type simple and avoids dynamic
/// dispatch on closures.
pub type Constructor = fn(&OperationParams) -> Result<Box<dyn Operation>, RegistryError>;

// ─── RegistryError ──────────────────────────────────────────────────────────

/// Errors that can occur when resolving an operation name via the registry.
///
/// Typed and matchable; mirrors the `SinkError` / `ImageError` pattern
/// (DEC-007). The binary maps these to `RecipeError` variants at the recipe
/// layer boundary.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// No constructor registered under this name.
    #[error("unknown operation '{name}'")]
    Unknown {
        /// The name that was not found in the registry.
        name: String,
    },

    /// A constructor rejected its params (wrong/missing/out-of-range).
    #[error("invalid params for operation '{op}': {reason}")]
    InvalidParams {
        /// The stable registry key for the operation.
        op: &'static str,
        /// Human-readable reason the params were rejected.
        reason: String,
    },
}

// ─── OperationRegistry ──────────────────────────────────────────────────────

/// Maps operation names to constructor functions (DEC-005).
///
/// Constructed via [`OperationRegistry::new`] (empty) or
/// [`OperationRegistry::with_builtins`] (pre-populated with `identity`,
/// `invert`, `resize`, `auto-orient`, and `watermark`). New operations call
/// [`OperationRegistry::register`] to add themselves — without touching the
/// recipe parser (the whole point of the registry seam). An operation whose
/// params name a FILE (an "asset-bearing" op — `watermark`'s `image`/`font`,
/// and the `.cube` LUT op STAGE-050 anticipates) registers via
/// [`OperationRegistry::register_with_assets`] instead (SPEC-128, Call 1):
/// the registry itself never gains IO — it only records WHICH param keys
/// name a file, so a native caller can resolve them before construction, and
/// `wasm::transform` can refuse a recipe that needs them (Call 3), both
/// through the same query, [`OperationRegistry::asset_keys`].
pub struct OperationRegistry {
    map: HashMap<&'static str, Constructor>,
    asset_keys: HashMap<&'static str, &'static [&'static str]>,
}

impl OperationRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        OperationRegistry {
            map: HashMap::new(),
            asset_keys: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with the built-in operations:
    /// `"identity"`, `"invert"` (SPEC-003), `"resize"` (SPEC-010),
    /// `"auto-orient"` (SPEC-015), and `"watermark"` (SPEC-128 — registered
    /// only now; DEC-031 explains why it was not registrable before).
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        reg.register("identity", |_params| Ok(Box::new(Identity)));
        reg.register("invert", |_params| Ok(Box::new(Invert)));
        reg.register("resize", |p| Ok(Box::new(Resize::from_params(p)?)));
        reg.register("auto-orient", |_params| Ok(Box::new(AutoOrient)));
        reg.register_with_assets(
            "watermark",
            |p| Watermark::from_params(p).map(|w| Box::new(w) as Box<dyn Operation>),
            &["image", "font"],
        );
        reg
    }

    /// Register a constructor under `name`.
    ///
    /// Overwrites any previous registration for the same name (including any
    /// asset-key declaration — use [`OperationRegistry::register_with_assets`]
    /// if this op needs one). Names are `'static` str references — typically
    /// string literals — matching the `Operation::name()` contract.
    pub fn register(&mut self, name: &'static str, ctor: Constructor) {
        self.map.insert(name, ctor);
        self.asset_keys.remove(name);
    }

    /// Register an operation whose params name one or more FILES (SPEC-128,
    /// Call 1) — e.g. watermark's `image`/`font`.
    ///
    /// `asset_keys` lists the param keys that, when a step sets one to a
    /// string, name a file that must be resolved (read into bytes, attached
    /// via [`OperationParams::set_resolved_bytes`]) at the recipe IO boundary
    /// **before** `ctor` runs — never inside this registry, which stays pure
    /// so it keeps compiling for `wasm32` (DEC-064). `ctor` reads the
    /// resolved bytes back via [`OperationParams::resolved_bytes`] (see
    /// `Watermark::from_params` for the pattern). A key absent from a given
    /// step (e.g. `font`, whose default is the bundled font) is simply
    /// nothing to resolve — this is not a validation rule, just a resolution
    /// hint.
    pub fn register_with_assets(
        &mut self,
        name: &'static str,
        ctor: Constructor,
        asset_keys: &'static [&'static str],
    ) {
        self.map.insert(name, ctor);
        self.asset_keys.insert(name, asset_keys);
    }

    /// The param keys that name a file for `name`'s constructor.
    ///
    /// Empty for an operation with no asset params, **including an unknown
    /// name** — never panics. This is the ONE query both the native resolver
    /// (which files to read before `build_pipeline`) and `wasm::transform`'s
    /// refusal (Call 3 — a non-empty result for any step means that recipe
    /// cannot run on the wasm surface, which has no filesystem) share, so the
    /// two can never independently decide an op is/isn't asset-bearing.
    pub fn asset_keys(&self, name: &str) -> &'static [&'static str] {
        self.asset_keys.get(name).copied().unwrap_or(&[])
    }

    /// Whether `name` has a registered constructor.
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Construct a `Box<dyn Operation>` for `name` using the registered
    /// constructor and the supplied `params`.
    ///
    /// Returns [`RegistryError::Unknown`] if `name` is not registered.
    /// Returns [`RegistryError::InvalidParams`] if the constructor rejects
    /// the params. Never panics on a missing name.
    pub fn build(
        &self,
        name: &str,
        params: &OperationParams,
    ) -> Result<Box<dyn Operation>, RegistryError> {
        let ctor = self.map.get(name).ok_or_else(|| RegistryError::Unknown {
            name: name.to_owned(),
        })?;
        ctor(params)
    }
}

impl Default for OperationRegistry {
    /// `Default` delegates to [`OperationRegistry::new`] (empty registry).
    ///
    /// Required to satisfy `clippy::new_without_default`.
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::operation::OperationParams;

    #[test]
    fn with_builtins_contains_identity_and_invert() {
        let reg = OperationRegistry::with_builtins();
        assert!(
            reg.contains("identity"),
            "expected 'identity' to be registered"
        );
        assert!(reg.contains("invert"), "expected 'invert' to be registered");
        assert!(!reg.contains("bogus"), "'bogus' should not be registered");
    }

    #[test]
    fn with_builtins_contains_resize() {
        let reg = OperationRegistry::with_builtins();
        assert!(reg.contains("resize"), "expected 'resize' to be registered");
    }

    #[test]
    fn build_unknown_returns_typed_error() {
        let reg = OperationRegistry::with_builtins();
        let result = reg.build("bogus", &OperationParams::empty());
        assert!(
            matches!(result, Err(RegistryError::Unknown { ref name }) if name == "bogus"),
            "expected RegistryError::Unknown {{ name: \"bogus\" }}"
        );
    }

    #[test]
    fn register_then_build_custom_op() {
        // A tiny test-only operation: returns a fixed name.
        struct Sentinel;

        impl crate::operation::Operation for Sentinel {
            fn name(&self) -> &'static str {
                "sentinel"
            }

            fn params(&self) -> OperationParams {
                OperationParams::empty()
            }

            fn apply(
                &self,
                img: crate::image::Image,
            ) -> Result<crate::image::Image, crate::operation::OperationError> {
                Ok(img)
            }
        }

        let mut reg = OperationRegistry::new();
        reg.register("sentinel", |_p| Ok(Box::new(Sentinel)));

        let op = reg
            .build("sentinel", &OperationParams::empty())
            .expect("sentinel should build successfully");
        assert_eq!(op.name(), "sentinel");
    }

    #[test]
    fn build_resize_with_valid_params() {
        let reg = OperationRegistry::with_builtins();
        let params = OperationParams::from_map({
            let mut m = BTreeMap::new();
            m.insert("mode".to_owned(), toml::Value::String("max".into()));
            m.insert("width".to_owned(), toml::Value::Integer(64));
            m
        });
        let op = reg
            .build("resize", &params)
            .expect("build('resize', {mode='max',width=64}) should succeed");
        assert_eq!(op.name(), "resize");
    }

    #[test]
    fn build_resize_invalid_params_is_typed() {
        // empty params → missing mode → InvalidParams
        let reg = OperationRegistry::with_builtins();
        let result = reg.build("resize", &OperationParams::empty());
        assert!(
            matches!(
                result,
                Err(RegistryError::InvalidParams { op: "resize", .. })
            ),
            "expected InvalidParams for resize with empty params"
        );
    }

    // ── SPEC-015 registry tests ───────────────────────────────────────────────

    #[test]
    fn with_builtins_contains_auto_orient() {
        let reg = OperationRegistry::with_builtins();
        assert!(
            reg.contains("auto-orient"),
            "expected 'auto-orient' to be registered in with_builtins"
        );
    }

    #[test]
    fn build_auto_orient() {
        let reg = OperationRegistry::with_builtins();
        let op = reg
            .build("auto-orient", &OperationParams::empty())
            .expect("build('auto-orient', empty) should succeed");
        assert_eq!(op.name(), "auto-orient");
    }

    // ── SPEC-128 registry tests ───────────────────────────────────────────────

    #[test]
    fn with_builtins_contains_watermark() {
        let reg = OperationRegistry::with_builtins();
        assert!(
            reg.contains("watermark"),
            "expected 'watermark' to be registered (SPEC-128)"
        );
    }

    #[test]
    fn watermark_declares_image_and_font_as_asset_keys() {
        let reg = OperationRegistry::with_builtins();
        assert_eq!(reg.asset_keys("watermark"), &["image", "font"]);
    }

    #[test]
    fn an_ordinary_op_has_no_asset_keys() {
        let reg = OperationRegistry::with_builtins();
        assert!(
            reg.asset_keys("resize").is_empty(),
            "resize has no file-shaped params"
        );
    }

    #[test]
    fn unknown_op_has_no_asset_keys_and_does_not_panic() {
        let reg = OperationRegistry::with_builtins();
        assert!(reg.asset_keys("bogus").is_empty());
    }
}
