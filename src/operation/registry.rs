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

use super::{
    AutoOrient, Identity, Invert, Operation, OperationParams, Resize, Watermark, RESIZE_PARAMS,
    WATERMARK_PARAMS,
};

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

// ─── Param schema ───────────────────────────────────────────────────────────

/// One key an operation accepts in a recipe step, and what its value must be.
///
/// An op publishes its whole accepted-key table when it registers
/// ([`OperationRegistry::register_with_params`]). The registry checks every
/// step against that table before the op's constructor runs, so a key the op
/// does not take, a value of the wrong TOML type, or a value out of range is
/// refused by name instead of being ignored or replaced by a default.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamSpec {
    /// The step key, as written in the recipe.
    pub key: &'static str,
    /// What the key's value must be.
    pub kind: ParamKind,
}

/// The value a [`ParamSpec`] key takes. Every rule here depends only on the
/// value itself; rules that depend on other keys (which keys a mode needs or
/// ignores) stay in the op's constructor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind {
    /// Any string.
    Text,
    /// A string from a fixed list.
    OneOf(&'static [&'static str]),
    /// A string naming a file. The registry never reads it: the caller resolves
    /// it into bytes ([`OperationParams::set_resolved_bytes`]) before the
    /// constructor runs.
    Asset,
    /// A TOML integer in `min..=u32::MAX`. A float is refused, not truncated.
    UInt {
        /// The smallest accepted value.
        min: u32,
    },
    /// A TOML integer or float, always finite (and finite as an `f32`, the
    /// width ops read it at), above `min` (or at it, when `min_inclusive`) and
    /// at most `max`.
    Number {
        /// The lower bound.
        min: f64,
        /// Whether `min` itself is accepted.
        min_inclusive: bool,
        /// The inclusive upper bound, if any.
        max: Option<f64>,
    },
    /// `true` or `false`.
    Bool,
}

impl ParamKind {
    /// Whether `value` satisfies this kind (the check `check_params` runs per
    /// key), for a caller that holds one value rather than a params map.
    pub fn accepts(&self, value: &toml::Value) -> bool {
        self.check(value).is_ok()
    }

    /// The phrase after "must be" in an error, e.g. `a number > 0 and <= 1024`.
    pub fn expectation(&self) -> String {
        match self {
            ParamKind::Text => "a string".to_owned(),
            ParamKind::OneOf(values) => format!("one of: {}", values.join(", ")),
            ParamKind::Asset => "a string naming a file".to_owned(),
            ParamKind::UInt { min } => format!("an integer >= {min}"),
            ParamKind::Number {
                min,
                min_inclusive,
                max,
            } => match (min_inclusive, max) {
                (true, Some(max)) => format!("a number in {min:?}..={max:?}"),
                (false, Some(max)) => format!("a number > {min} and <= {max}"),
                (true, None) => format!("a finite number >= {min}"),
                (false, None) => format!("a finite number > {min}"),
            },
            ParamKind::Bool => "true or false".to_owned(),
        }
    }

    /// `Ok` when `value` satisfies this kind, else the `got …` description of
    /// what was found.
    fn check(&self, value: &toml::Value) -> Result<(), String> {
        use toml::Value;
        match (self, value) {
            (ParamKind::Text | ParamKind::Asset, Value::String(_)) => Ok(()),
            (ParamKind::OneOf(values), Value::String(s)) => {
                if values.contains(&s.as_str()) {
                    Ok(())
                } else {
                    Err(format!("{s:?}"))
                }
            }
            (ParamKind::UInt { min }, Value::Integer(i)) => {
                // TOML integers are 64-bit: refuse negatives and anything past
                // u32::MAX rather than wrapping or dropping them.
                if *i >= i64::from(*min) && *i <= i64::from(u32::MAX) {
                    Ok(())
                } else {
                    Err(i.to_string())
                }
            }
            (
                ParamKind::Number {
                    min,
                    min_inclusive,
                    max,
                },
                Value::Integer(_) | Value::Float(_),
            ) => {
                let (n, shown) = match value {
                    Value::Integer(i) => (*i as f64, i.to_string()),
                    Value::Float(f) => (*f, float_text(*f)),
                    _ => unreachable!("matched Integer | Float above"),
                };
                // The ops read the value as an f32, so the range applies to what
                // they will see: `1e-50` is above 0 as an f64 but reads as 0.0.
                let read = f64::from(n as f32);
                let in_range = |v: f64| {
                    let above_min = if *min_inclusive { v >= *min } else { v > *min };
                    above_min && max.is_none_or(|m| v <= m)
                };
                // `n as f32` overflows to inf for a finite f64 past f32::MAX.
                if n.is_finite() && read.is_finite() && in_range(n) && in_range(read) {
                    Ok(())
                } else {
                    Err(shown)
                }
            }
            (ParamKind::Bool, Value::Boolean(_)) => Ok(()),
            (_, other) => Err(describe_value(other)),
        }
    }
}

/// A float as a recipe author would write it: `nan`, `inf`, `1.5`, `100.0`.
fn float_text(f: f64) -> String {
    if f.is_nan() {
        "nan".to_owned()
    } else if f.is_infinite() {
        if f > 0.0 { "inf" } else { "-inf" }.to_owned()
    } else {
        format!("{f:?}")
    }
}

/// A value of the wrong type, named by its TOML type.
fn describe_value(value: &toml::Value) -> String {
    use toml::Value;
    match value {
        Value::String(s) => format!("the string {s:?}"),
        Value::Integer(i) => format!("the integer {i}"),
        Value::Float(f) => format!("the float {}", float_text(*f)),
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(_) => "a date-time".to_owned(),
        Value::Array(_) => "an array".to_owned(),
        Value::Table(_) => "a table".to_owned(),
    }
}

/// The row for `key` in `specs`, if there is one.
pub fn param_spec<'a>(specs: &'a [ParamSpec], key: &str) -> Option<&'a ParamSpec> {
    specs.iter().find(|s| s.key == key)
}

/// Check every key in `params` against `specs`, in the map's (sorted) order,
/// and return the first failure as a one-line reason naming the key.
///
/// For each key: unknown, then type, then range. `op` names the op in the
/// "takes no keys" wording. [`OperationRegistry::build`] runs this before
/// every constructor; the built-in constructors also run it themselves, so a
/// direct caller gets the same checks.
pub fn check_params(op: &str, specs: &[ParamSpec], params: &OperationParams) -> Result<(), String> {
    for (key, value) in params.iter() {
        let Some(spec) = specs.iter().find(|s| s.key == key) else {
            return Err(if specs.is_empty() {
                format!("unknown key '{key}'; '{op}' takes no keys")
            } else {
                let keys: Vec<&str> = specs.iter().map(|s| s.key).collect();
                format!("unknown key '{key}'; expected one of: {}", keys.join(", "))
            });
        };
        if let Err(got) = spec.kind.check(value) {
            let sep = if matches!(spec.kind, ParamKind::OneOf(_)) {
                ";"
            } else {
                ","
            };
            return Err(format!(
                "'{key}' must be {}{sep} got {got}",
                spec.kind.expectation()
            ));
        }
    }
    Ok(())
}

// ─── OperationRegistry ──────────────────────────────────────────────────────

/// One registered op: its constructor and its accepted-key table.
struct Entry {
    ctor: Constructor,
    params: Vec<ParamSpec>,
    /// The `Asset` keys of `params`, in table order.
    asset_keys: Vec<&'static str>,
}

/// Maps operation names to constructor functions (DEC-005).
///
/// Constructed via [`OperationRegistry::new`] (empty) or
/// [`OperationRegistry::with_builtins`] (pre-populated with `identity`,
/// `invert`, `resize`, `auto-orient`, and `watermark`). New operations
/// register themselves without touching the recipe parser (the whole point of
/// the registry seam), and each publishes the step keys it accepts:
///
/// - [`OperationRegistry::register`] — an op that takes no keys;
/// - [`OperationRegistry::register_with_params`] — an op with a key table;
/// - [`OperationRegistry::register_with_assets`] — shorthand for a table whose
///   every key names a file.
///
/// The registry itself never gains IO. It only records which keys name a
/// file ([`OperationRegistry::asset_keys`], derived from the table), so a
/// native caller can resolve them before construction and `wasm::transform`
/// can refuse a recipe that needs them, both through the same query.
pub struct OperationRegistry {
    map: HashMap<&'static str, Entry>,
}

impl OperationRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        OperationRegistry {
            map: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with the built-in operations:
    /// `"identity"`, `"invert"`, `"resize"`, `"auto-orient"` and
    /// `"watermark"`.
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        reg.register("identity", |_params| Ok(Box::new(Identity)));
        reg.register("invert", |_params| Ok(Box::new(Invert)));
        reg.register_with_params(
            "resize",
            |p| Ok(Box::new(Resize::from_params(p)?)),
            RESIZE_PARAMS,
        );
        reg.register("auto-orient", |_params| Ok(Box::new(AutoOrient)));
        reg.register_with_params(
            "watermark",
            |p| Watermark::from_params(p).map(|w| Box::new(w) as Box<dyn Operation>),
            WATERMARK_PARAMS,
        );
        reg
    }

    /// Register a constructor for an op that takes **no** step keys: any key
    /// on its step is refused.
    ///
    /// Overwrites any previous registration for the same name. Names are
    /// `'static` str references — typically string literals — matching the
    /// `Operation::name()` contract.
    pub fn register(&mut self, name: &'static str, ctor: Constructor) {
        self.register_with_params(name, ctor, &[]);
    }

    /// Register an op with its accepted-key table. A step key not in `params`,
    /// or a value that does not satisfy its [`ParamKind`], is refused before
    /// `ctor` runs. Every [`ParamKind::Asset`] key is listed by
    /// [`OperationRegistry::asset_keys`].
    pub fn register_with_params(
        &mut self,
        name: &'static str,
        ctor: Constructor,
        params: &'static [ParamSpec],
    ) {
        self.insert(name, ctor, params.to_vec());
    }

    /// Register an operation whose step keys all name FILES — e.g. an op that
    /// reads one asset and nothing else. Shorthand for a
    /// [`OperationRegistry::register_with_params`] table of those keys as
    /// [`ParamKind::Asset`].
    ///
    /// Each key, when a step sets it, names a file that must be resolved (read
    /// into bytes, attached via [`OperationParams::set_resolved_bytes`]) at the
    /// recipe IO boundary **before** `ctor` runs — never inside this registry,
    /// which stays pure so it keeps compiling for `wasm32` (DEC-064). `ctor`
    /// reads the bytes back via [`OperationParams::resolved_bytes`]. A key
    /// absent from a given step is simply nothing to resolve.
    pub fn register_with_assets(
        &mut self,
        name: &'static str,
        ctor: Constructor,
        asset_keys: &'static [&'static str],
    ) {
        let params = asset_keys
            .iter()
            .map(|&key| ParamSpec {
                key,
                kind: ParamKind::Asset,
            })
            .collect();
        self.insert(name, ctor, params);
    }

    fn insert(&mut self, name: &'static str, ctor: Constructor, params: Vec<ParamSpec>) {
        let asset_keys = params
            .iter()
            .filter(|s| s.kind == ParamKind::Asset)
            .map(|s| s.key)
            .collect();
        self.map.insert(
            name,
            Entry {
                ctor,
                params,
                asset_keys,
            },
        );
    }

    /// The step keys `name` accepts (empty for an op that takes none), or
    /// `None` for an unregistered name.
    pub fn param_specs(&self, name: &str) -> Option<&[ParamSpec]> {
        self.map.get(name).map(|e| e.params.as_slice())
    }

    /// The param keys that name a file for `name`'s constructor: every
    /// [`ParamKind::Asset`] key of its table, in table order.
    ///
    /// Empty for an operation with no asset params, **including an unknown
    /// name** — never panics. This is the ONE query both the native resolver
    /// (which files to read before `build_pipeline`) and `wasm::transform`'s
    /// refusal (a non-empty result for any step means that recipe cannot run
    /// on the wasm surface, which has no filesystem) share, so the two can
    /// never independently decide an op is/isn't asset-bearing.
    pub fn asset_keys(&self, name: &str) -> &[&'static str] {
        self.map
            .get(name)
            .map(|e| e.asset_keys.as_slice())
            .unwrap_or(&[])
    }

    /// Whether `name` has a registered constructor.
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Construct a `Box<dyn Operation>` for `name` using the registered
    /// constructor and the supplied `params`.
    ///
    /// Returns [`RegistryError::Unknown`] if `name` is not registered.
    /// Returns [`RegistryError::InvalidParams`] if a key fails the op's
    /// published table ([`check_params`]) or the constructor rejects the
    /// params. Never panics on a missing name.
    pub fn build(
        &self,
        name: &str,
        params: &OperationParams,
    ) -> Result<Box<dyn Operation>, RegistryError> {
        let (&op, entry) = self
            .map
            .get_key_value(name)
            .ok_or_else(|| RegistryError::Unknown {
                name: name.to_owned(),
            })?;
        check_params(op, &entry.params, params)
            .map_err(|reason| RegistryError::InvalidParams { op, reason })?;
        (entry.ctor)(params)
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
    fn per_key_check_agrees_with_check_params() {
        use crate::operation::WATERMARK_PARAMS;
        let values = [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            0.0,
            -1.0,
            1e-50,
            0.5,
            1.0,
            1.5,
            1024.0,
            1024.00001,
            1025.0,
            4_294_967_296.0,
            1e39,
        ];
        let keys = ["size", "opacity", "scale"];
        let mut rows = 0;
        for key in keys {
            let spec = param_spec(WATERMARK_PARAMS, key).expect("a table row");
            assert!(matches!(spec.kind, ParamKind::Number { .. }), "{key}");
            for v in values {
                rows += 1;
                let value = toml::Value::Float(v);
                let params =
                    OperationParams::from_map(BTreeMap::from([(key.to_owned(), value.clone())]));
                assert_eq!(
                    spec.kind.accepts(&value),
                    check_params("watermark", WATERMARK_PARAMS, &params).is_ok(),
                    "{key} = {v}"
                );
            }
        }
        assert_eq!(rows, 42, "three keys × fourteen values");
        assert!(param_spec(WATERMARK_PARAMS, "nope").is_none());
    }

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
    fn every_builtin_publishes_a_param_schema() {
        let reg = OperationRegistry::with_builtins();
        let keys = |name: &str| -> Vec<&'static str> {
            reg.param_specs(name)
                .unwrap_or_else(|| panic!("'{name}' must publish a schema"))
                .iter()
                .map(|s| s.key)
                .collect()
        };
        assert_eq!(keys("resize"), ["mode", "width", "height", "percent"]);
        assert_eq!(
            keys("watermark"),
            [
                "image", "text", "font", "size", "color", "gravity", "opacity", "scale", "margin",
                "tile"
            ]
        );
        for paramless in ["identity", "invert", "auto-orient"] {
            assert!(keys(paramless).is_empty(), "'{paramless}' takes no keys");
        }
        assert!(reg.param_specs("bogus").is_none());
    }

    #[test]
    fn asset_keys_are_derived_from_the_schema() {
        let reg = OperationRegistry::with_builtins();
        let from_schema: Vec<&str> = reg
            .param_specs("watermark")
            .expect("watermark schema")
            .iter()
            .filter(|s| s.kind == ParamKind::Asset)
            .map(|s| s.key)
            .collect();
        assert_eq!(from_schema, ["image", "font"]);
        assert_eq!(reg.asset_keys("watermark"), from_schema.as_slice());

        // The old entry point records its keys as assets in the schema too.
        let mut custom = OperationRegistry::new();
        custom.register_with_assets(
            "fixture",
            |_p| Ok(Box::new(crate::operation::Identity)),
            &["a", "b"],
        );
        let specs = custom.param_specs("fixture").expect("schema");
        assert!(specs.iter().all(|s| s.kind == ParamKind::Asset));
        assert_eq!(custom.asset_keys("fixture"), &["a", "b"]);
    }

    /// Every value a `OneOf` key publishes is one its constructor accepts, so
    /// the published list cannot drift from the parser (e.g. `Gravity::from_str`).
    #[test]
    fn one_of_values_are_accepted_by_their_constructors() {
        let reg = OperationRegistry::with_builtins();
        let mut checked = 0;
        for op in ["resize", "watermark"] {
            for spec in reg.param_specs(op).expect("schema") {
                let ParamKind::OneOf(values) = spec.kind else {
                    continue;
                };
                for value in values {
                    let mut m = BTreeMap::new();
                    m.insert(
                        spec.key.to_owned(),
                        toml::Value::String((*value).to_owned()),
                    );
                    match (op, *value) {
                        ("resize", "max") => {
                            m.insert("width".to_owned(), toml::Value::Integer(8));
                        }
                        ("resize", "percent") => {
                            m.insert("percent".to_owned(), toml::Value::Integer(50));
                        }
                        ("resize", _) => {
                            m.insert("width".to_owned(), toml::Value::Integer(8));
                            m.insert("height".to_owned(), toml::Value::Integer(8));
                        }
                        _ => {
                            m.insert("text".to_owned(), toml::Value::String("hi".into()));
                        }
                    }
                    reg.build(op, &OperationParams::from_map(m))
                        .unwrap_or_else(|e| {
                            panic!("{op}: published {}='{value}' was refused: {e}", spec.key)
                        });
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 6 + 9, "6 resize modes + 9 gravities");
    }

    #[test]
    fn register_without_params_refuses_any_key() {
        let mut reg = OperationRegistry::new();
        reg.register("sentinel", |_p| Ok(Box::new(crate::operation::Identity)));
        let params = OperationParams::from_map({
            let mut m = BTreeMap::new();
            m.insert("bogus".to_owned(), toml::Value::Integer(1));
            m
        });
        match reg.build("sentinel", &params) {
            Err(RegistryError::InvalidParams { op, reason }) => {
                assert_eq!(op, "sentinel");
                assert!(
                    reason.contains("unknown key 'bogus'") && reason.contains("takes no keys"),
                    "got: {reason}"
                );
            }
            Err(other) => panic!("expected InvalidParams, got {other:?}"),
            Ok(_) => panic!("an op registered without params must refuse any key"),
        }
        // No keys at all still builds.
        assert!(reg.build("sentinel", &OperationParams::empty()).is_ok());
    }

    #[test]
    fn unknown_op_has_no_asset_keys_and_does_not_panic() {
        let reg = OperationRegistry::with_builtins();
        assert!(reg.asset_keys("bogus").is_empty());
    }
}
