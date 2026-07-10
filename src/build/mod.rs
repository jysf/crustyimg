//! The declared build manifest: `crustyimg.build.toml` (SPEC-063, DEC-057).
//!
//! A **build manifest** is a versioned list of `[[target]]`s, each binding a set
//! of sources (a glob / dir / path, or a list of them) to a recipe file and an
//! output directory + name template:
//!
//! ```toml
//! version = 1
//!
//! [[target]]
//! source = "assets/**/*.png"     # or ["a/*.png", "b/"]
//! recipe = "recipes/web.toml"    # a recipe TOML (see `crate::recipe`)
//! out    = "dist/img"            # output directory (auto-created)
//! name   = "{stem}_web.{ext}"    # optional; default "{stem}.{ext}"
//! ```
//!
//! This module is the **library half** of `crustyimg build`: it parses and
//! validates the manifest and nothing else. It does not read images, resolve
//! sources, or write outputs — the executor (`run_build` in `crate::cli`) does
//! that by looping the shipped per-input `apply` worker over these targets.
//!
//! Layering: this module depends on `serde`/`toml`/`thiserror` only. No `clap`,
//! no pixel crate, no filesystem. Its sibling [`cache`] (SPEC-064, DEC-058) owns
//! the build's content-addressed cache — the key composition and the on-disk
//! store — and does touch the filesystem; its sibling [`lock`] (SPEC-066,
//! DEC-059) owns the `crustyimg.build.lock` format and its env-aware diff, and
//! does not. The executor still owns all the wiring.
//!
//! ## Relationship to recipes (DEC-005 / DEC-057)
//!
//! A recipe is *portable and input-agnostic* — an ordered list of operations. A
//! build manifest is the *sibling contract* that BINDS recipes to concrete
//! source→output mappings. They are deliberately separate files: a recipe stays
//! reusable across projects, a manifest is project-specific.
//!
//! ## Validation (untrusted-input-hardening)
//!
//! - Text longer than [`BUILD_MANIFEST_MAX_BYTES`] → [`BuildError::TooLarge`]
//!   (checked **before** `toml::from_str`, so an oversized manifest is never parsed).
//! - Malformed TOML, an unknown key, or a missing required key →
//!   [`BuildError::Parse`] (`deny_unknown_fields` catches typos).
//! - Any `version` other than [`SUPPORTED_VERSION`] → [`BuildError::UnsupportedVersion`],
//!   checked before target validation.
//! - More than [`BUILD_MANIFEST_MAX_TARGETS`] targets → [`BuildError::TooManyTargets`].
//! - A target with an empty `source`/`recipe`/`out`, or a `-` (stdin) source →
//!   [`BuildError::InvalidTarget`]. A build reads declared files, never stdin.
//!
//! All of these fire before the executor touches a single input.

pub mod cache;
pub mod lock;

use serde::Deserialize;
use thiserror::Error;

// ─── Constants ──────────────────────────────────────────────────────────────

/// The default manifest file name, discovered when `crustyimg build` is run
/// with no `FILE` argument.
pub const DEFAULT_MANIFEST_FILE: &str = "crustyimg.build.toml";

/// The only manifest schema version this build understands.
pub const SUPPORTED_VERSION: u32 = 1;

/// The name template used when a target omits `name`.
pub const DEFAULT_NAME_TEMPLATE: &str = "{stem}.{ext}";

/// Maximum allowed byte length of a manifest TOML string (64 KiB).
///
/// Mirrors [`crate::recipe::RECIPE_MAX_BYTES`]: [`BuildManifest::from_toml`]
/// checks `s.len()` **before** parsing, and the CLI checks the on-disk size via
/// `std::fs::metadata` before reading the file into memory (DEC-036). Reject
/// only on `>`; equality is accepted.
pub const BUILD_MANIFEST_MAX_BYTES: usize = 64 * 1024;

/// Maximum allowed number of targets in one manifest (1024).
///
/// Checked after the version check, so an over-version manifest is still
/// `UnsupportedVersion`. Reject only on `>`; equality is accepted.
pub const BUILD_MANIFEST_MAX_TARGETS: usize = 1024;

// ─── BuildError ─────────────────────────────────────────────────────────────

/// Errors parsing or validating a [`BuildManifest`] (DEC-007).
///
/// The `RecipeError` analog. Every variant is a *manifest content* error, so the
/// CLI maps them all to exit 2 (usage); the I/O of reading the manifest file is
/// the caller's concern (exit 3).
#[derive(Debug, Error)]
pub enum BuildError {
    /// The manifest TOML could not be parsed: malformed syntax, an unknown key
    /// (`deny_unknown_fields`), or a missing required key.
    #[error("could not parse build manifest: {0}")]
    Parse(String),

    /// The manifest's `version` is not supported by this build.
    #[error("unsupported build manifest version {found} (supported: {supported})")]
    UnsupportedVersion {
        /// The `version` value found in the file.
        found: u32,
        /// The only version this binary understands.
        supported: u32,
    },

    /// The manifest text exceeds [`BUILD_MANIFEST_MAX_BYTES`] (checked before parsing).
    #[error("build manifest is too large ({size} bytes; max {max})")]
    TooLarge {
        /// The actual byte length of the oversized text.
        size: usize,
        /// The cap that was exceeded.
        max: usize,
    },

    /// The manifest declares more than [`BUILD_MANIFEST_MAX_TARGETS`] targets.
    #[error("build manifest has too many targets ({count}; max {max})")]
    TooManyTargets {
        /// The actual number of targets.
        count: usize,
        /// The cap that was exceeded.
        max: usize,
    },

    /// A target parsed but is not usable (empty `source` list, blank field, or a
    /// `-` stdin source). `index` is the target's 0-based position.
    #[error("build manifest target #{index} is invalid: {reason}")]
    InvalidTarget {
        /// The 0-based position of the offending `[[target]]`.
        index: usize,
        /// Why the target was rejected.
        reason: String,
    },
}

// ─── Schema ─────────────────────────────────────────────────────────────────

/// A target's `source`: one glob/dir/path, or a list of them.
///
/// Untagged, so both TOML spellings deserialize into the same type:
/// `source = "a/*.png"` and `source = ["a/*.png", "b/"]`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum SourceSpec {
    /// A single source argument.
    One(String),
    /// An ordered list of source arguments.
    Many(Vec<String>),
}

impl SourceSpec {
    /// The source arguments as a slice, whichever spelling was used. The
    /// executor calls `source::resolve` on each and flattens the results.
    pub fn as_slice(&self) -> &[String] {
        match self {
            SourceSpec::One(s) => std::slice::from_ref(s),
            SourceSpec::Many(v) => v.as_slice(),
        }
    }
}

/// One declared build target: sources × a recipe → an output dir + name template.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// The input source(s): a glob, a directory, or a file path — or a list.
    pub source: SourceSpec,
    /// Path to the recipe TOML applied to every resolved input.
    pub recipe: String,
    /// The output directory. Created if missing; outputs never escape it.
    pub out: String,
    /// Optional output name template (`{stem}`/`{ext}`/`{name}`/`{parent}`).
    /// Defaults to [`DEFAULT_NAME_TEMPLATE`].
    pub name: Option<String>,
}

impl Target {
    /// This target's name template, or [`DEFAULT_NAME_TEMPLATE`] when unset.
    pub fn template(&self) -> &str {
        self.name.as_deref().unwrap_or(DEFAULT_NAME_TEMPLATE)
    }
}

/// A parsed, validated `crustyimg.build.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildManifest {
    /// Schema version. Only [`SUPPORTED_VERSION`] is accepted.
    pub version: u32,
    /// The declared targets, in file order. An empty manifest is valid (a no-op build).
    #[serde(default)]
    pub target: Vec<Target>,
}

impl BuildManifest {
    /// Parse a manifest TOML string and validate it end to end.
    ///
    /// Checks, in order: size cap → TOML parse (`deny_unknown_fields`) → version
    /// gate → target-count cap → per-target validation. Every failure is a typed
    /// [`BuildError`]; nothing here touches the filesystem.
    pub fn from_toml(s: &str) -> Result<BuildManifest, BuildError> {
        // Size check BEFORE parsing (parse-time DoS prevention, DEC-036).
        if s.len() > BUILD_MANIFEST_MAX_BYTES {
            return Err(BuildError::TooLarge {
                size: s.len(),
                max: BUILD_MANIFEST_MAX_BYTES,
            });
        }

        let manifest: BuildManifest =
            toml::from_str(s).map_err(|e| BuildError::Parse(e.to_string()))?;

        // Version before anything semantic, so a future manifest reports the
        // version rather than a cascade of "invalid target" errors.
        if manifest.version != SUPPORTED_VERSION {
            return Err(BuildError::UnsupportedVersion {
                found: manifest.version,
                supported: SUPPORTED_VERSION,
            });
        }

        if manifest.target.len() > BUILD_MANIFEST_MAX_TARGETS {
            return Err(BuildError::TooManyTargets {
                count: manifest.target.len(),
                max: BUILD_MANIFEST_MAX_TARGETS,
            });
        }

        for (index, target) in manifest.target.iter().enumerate() {
            target
                .validate()
                .map_err(|reason| BuildError::InvalidTarget { index, reason })?;
        }

        Ok(manifest)
    }
}

impl Target {
    /// Reject targets that parse but cannot be run. Returns the human reason.
    fn validate(&self) -> Result<(), String> {
        let sources = self.source.as_slice();
        if sources.is_empty() {
            return Err("`source` is an empty list".to_owned());
        }
        for src in sources {
            if src.trim().is_empty() {
                return Err("`source` contains an empty entry".to_owned());
            }
            // A build reads declared files; stdin is a one-shot stream that
            // cannot feed N targets (and would deadlock the rayon fan-out).
            if src == "-" {
                return Err(
                    "`source` may not be `-` (stdin); a build reads declared files".to_owned(),
                );
            }
        }
        if self.recipe.trim().is_empty() {
            return Err("`recipe` is empty".to_owned());
        }
        if self.out.trim().is_empty() {
            return Err("`out` is empty".to_owned());
        }
        if self.name.as_deref().is_some_and(|n| n.trim().is_empty()) {
            return Err("`name` is empty".to_owned());
        }
        Ok(())
    }
}

// ─── Injective source→output (SPEC-065, DEC-057) ────────────────────────────

/// Two inputs of a build that would be written to the same output path.
///
/// A build's `source → output` mapping must be a **function**: with
/// `Overwrite::Allow` and the rayon fan-out, two inputs sharing an output path
/// race — the winner is nondeterministic and the summary over-counts — and a
/// lockfile (STAGE-022) cannot pin an output path two inputs fight over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputCollision {
    /// The shared output path (or the collision key that stands for it).
    pub output: String,
    /// The earlier of the two colliding sources.
    pub first: String,
    /// The later of the two colliding sources.
    pub second: String,
}

/// The first pair of `entries` sharing a collision key, or `None` if every key
/// is distinct.
///
/// `entries` are `(collision_key, source_label)` pairs in build order. Pure,
/// deterministic, and order-preserving: the reported `first` is the source that
/// claimed the key, `second` the one that duplicated it, and scanning stops at
/// the earliest duplicate. The caller composes the key (the executor joins each
/// target's `out` dir to its expanded name template); this fn only detects the
/// duplicate, so it stays filesystem-free and unit-testable.
pub fn find_output_collision(entries: &[(String, String)]) -> Option<OutputCollision> {
    let mut seen: std::collections::HashMap<&str, &str> =
        std::collections::HashMap::with_capacity(entries.len());
    for (key, label) in entries {
        match seen.get(key.as_str()) {
            Some(first) => {
                return Some(OutputCollision {
                    output: key.clone(),
                    first: (*first).to_owned(),
                    second: label.clone(),
                });
            }
            None => {
                seen.insert(key.as_str(), label.as_str());
            }
        }
    }
    None
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
version = 1

[[target]]
source = "assets/**/*.png"
recipe = "recipes/web.toml"
out = "dist/img"

[[target]]
source = ["a/*.png", "b/"]
recipe = "recipes/thumb.toml"
out = "dist/thumb"
name = "{stem}_thumb.{ext}"
"#;

    #[test]
    fn parses_valid_manifest() {
        let m = BuildManifest::from_toml(VALID).expect("valid manifest should parse");
        assert_eq!(m.version, SUPPORTED_VERSION);
        assert_eq!(m.target.len(), 2);

        // A `source` string and a `source` list both resolve to the expected list.
        assert_eq!(m.target[0].source.as_slice(), ["assets/**/*.png"]);
        assert_eq!(m.target[1].source.as_slice(), ["a/*.png", "b/"]);

        // `name` defaults when absent, and is honored when present.
        assert_eq!(m.target[0].template(), DEFAULT_NAME_TEMPLATE);
        assert_eq!(m.target[1].template(), "{stem}_thumb.{ext}");

        assert_eq!(m.target[0].recipe, "recipes/web.toml");
        assert_eq!(m.target[0].out, "dist/img");
    }

    #[test]
    fn empty_target_list_is_valid() {
        let m = BuildManifest::from_toml("version = 1\n").expect("an empty build is a no-op");
        assert!(m.target.is_empty());
    }

    #[test]
    fn rejects_unknown_field() {
        // A typo'd key in a target is rejected (deny_unknown_fields), not ignored.
        let toml_str = r#"
version = 1

[[target]]
source = "a/*.png"
recipe = "r.toml"
out = "dist"
bogus = 1
"#;
        let err = BuildManifest::from_toml(toml_str).expect_err("unknown field must be rejected");
        assert!(
            matches!(&err, BuildError::Parse(msg) if msg.contains("bogus")),
            "expected Parse naming the unknown field, got {err:?}"
        );

        // ... and so is a typo'd key at the top level.
        let err = BuildManifest::from_toml("version = 1\nversoin = 2\n")
            .expect_err("top-level unknown field must be rejected");
        assert!(matches!(err, BuildError::Parse(_)), "got {err:?}");
    }

    #[test]
    fn rejects_unsupported_version() {
        let err = BuildManifest::from_toml("version = 999\n").expect_err("version must be gated");
        assert!(
            matches!(
                err,
                BuildError::UnsupportedVersion {
                    found: 999,
                    supported: SUPPORTED_VERSION
                }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn rejects_oversize_manifest() {
        // '#' is a TOML comment, so a successful parse would prove the size check
        // did NOT fire before parsing.
        let oversized = "#".repeat(BUILD_MANIFEST_MAX_BYTES + 1);
        let err = BuildManifest::from_toml(&oversized).expect_err("oversize must be rejected");
        assert!(
            matches!(err, BuildError::TooLarge { size, max }
                if size == BUILD_MANIFEST_MAX_BYTES + 1 && max == BUILD_MANIFEST_MAX_BYTES),
            "got {err:?}"
        );
    }

    #[test]
    fn accepts_manifest_at_size_cap() {
        // The cap is inclusive: only `>` is rejected.
        let base = "version = 1\n";
        let at_cap = format!(
            "{base}{}",
            "#".repeat(BUILD_MANIFEST_MAX_BYTES - base.len())
        );
        assert_eq!(at_cap.len(), BUILD_MANIFEST_MAX_BYTES);
        assert!(BuildManifest::from_toml(&at_cap).is_ok());
    }

    #[test]
    fn rejects_too_many_targets() {
        let block = "[[target]]\nsource = \"a.png\"\nrecipe = \"r.toml\"\nout = \"d\"\n";
        let n = BUILD_MANIFEST_MAX_TARGETS + 1;
        let toml_str = format!("version = 1\n{}", block.repeat(n));
        assert!(
            toml_str.len() <= BUILD_MANIFEST_MAX_BYTES,
            "fixture must exercise the target cap, not the byte cap"
        );
        let err = BuildManifest::from_toml(&toml_str).expect_err("target cap must fire");
        assert!(
            matches!(err, BuildError::TooManyTargets { count, max }
                if count == n && max == BUILD_MANIFEST_MAX_TARGETS),
            "got {err:?}"
        );
    }

    #[test]
    fn missing_required_field_is_error() {
        for toml_str in [
            // no `recipe`
            "version = 1\n[[target]]\nsource = \"a.png\"\nout = \"d\"\n",
            // no `out`
            "version = 1\n[[target]]\nsource = \"a.png\"\nrecipe = \"r.toml\"\n",
            // no `source`
            "version = 1\n[[target]]\nrecipe = \"r.toml\"\nout = \"d\"\n",
            // no `version`
            "[[target]]\nsource = \"a.png\"\nrecipe = \"r.toml\"\nout = \"d\"\n",
        ] {
            let err = BuildManifest::from_toml(toml_str)
                .expect_err("a missing required field must be a typed error");
            assert!(matches!(err, BuildError::Parse(_)), "got {err:?}");
        }
    }

    #[test]
    fn rejects_unusable_target() {
        // An empty source list, a blank entry, and a stdin source are all rejected
        // with the offending target's index.
        for (toml_str, needle) in [
            (
                "version = 1\n[[target]]\nsource = []\nrecipe = \"r.toml\"\nout = \"d\"\n",
                "empty list",
            ),
            (
                "version = 1\n[[target]]\nsource = \"  \"\nrecipe = \"r.toml\"\nout = \"d\"\n",
                "empty entry",
            ),
            (
                "version = 1\n[[target]]\nsource = \"-\"\nrecipe = \"r.toml\"\nout = \"d\"\n",
                "stdin",
            ),
            (
                "version = 1\n[[target]]\nsource = \"a.png\"\nrecipe = \"\"\nout = \"d\"\n",
                "`recipe` is empty",
            ),
            (
                "version = 1\n[[target]]\nsource = \"a.png\"\nrecipe = \"r.toml\"\nout = \"\"\n",
                "`out` is empty",
            ),
            (
                "version = 1\n[[target]]\nsource = \"a.png\"\nrecipe = \"r.toml\"\nout = \"d\"\nname = \"\"\n",
                "`name` is empty",
            ),
        ] {
            let err = BuildManifest::from_toml(toml_str).expect_err("unusable target must fail");
            match &err {
                BuildError::InvalidTarget { index, reason } => {
                    assert_eq!(*index, 0);
                    assert!(reason.contains(needle), "expected {needle:?} in {reason:?}");
                }
                other => panic!("expected InvalidTarget, got {other:?}"),
            }
        }
    }

    #[test]
    fn source_spec_as_slice_covers_both_spellings() {
        assert_eq!(SourceSpec::One("a".into()).as_slice(), ["a"]);
        assert_eq!(
            SourceSpec::Many(vec!["a".into(), "b".into()]).as_slice(),
            ["a", "b"]
        );
    }

    // ── find_output_collision (SPEC-065) ──────────────────────────────────────

    /// `(key, label)` pairs from `&str` literals — the shape `run_build` builds.
    fn entries(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, l)| ((*k).to_owned(), (*l).to_owned()))
            .collect()
    }

    #[test]
    fn detects_first_duplicate_collision_key() {
        let e = entries(&[
            ("dist/a.{ext}", "src/a.png"),
            ("dist/logo.{ext}", "a/logo.png"),
            ("dist/logo.{ext}", "b/logo.png"),
            ("dist/z.{ext}", "src/z.png"),
        ]);
        let c = find_output_collision(&e).expect("repeated key must collide");
        assert_eq!(c.output, "dist/logo.{ext}");
        assert_eq!(c.first, "a/logo.png");
        assert_eq!(c.second, "b/logo.png");
    }

    #[test]
    fn no_collision_when_all_keys_distinct() {
        let e = entries(&[
            ("dist/a.{ext}", "src/a.png"),
            ("dist/b.{ext}", "src/b.png"),
            // Same stem, different out dir — a different output path.
            ("other/a.{ext}", "src2/a.png"),
        ]);
        assert_eq!(find_output_collision(&e), None);
        // The empty build is vacuously injective.
        assert_eq!(find_output_collision(&[]), None);
    }

    #[test]
    fn collision_is_order_preserving() {
        let e = entries(&[
            ("dist/x.{ext}", "first.png"),
            ("dist/x.{ext}", "second.png"),
            ("dist/x.{ext}", "third.png"),
        ]);
        let c = find_output_collision(&e).expect("must collide");
        // The earliest offending pair, and `first` is the earlier source.
        assert_eq!(
            (c.first.as_str(), c.second.as_str()),
            ("first.png", "second.png")
        );

        // Reversing the inputs reverses the reported pair — the fn reads order,
        // not the labels.
        let rev = entries(&[
            ("dist/x.{ext}", "second.png"),
            ("dist/x.{ext}", "first.png"),
        ]);
        let c = find_output_collision(&rev).expect("must collide");
        assert_eq!(
            (c.first.as_str(), c.second.as_str()),
            ("second.png", "first.png")
        );
    }
}
