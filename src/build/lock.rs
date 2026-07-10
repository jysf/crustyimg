//! The reproducibility lockfile: `crustyimg.build.lock` (SPEC-066, DEC-059).
//!
//! A **lockfile** is a committed, versioned record of what a `crustyimg build`
//! produced — one `[[output]]` per written file, plus the `[env]` the build ran
//! under:
//!
//! ```toml
//! version = 1
//!
//! [env]
//! crustyimg_version = "0.4.0"
//! target = "aarch64-macos"
//! features = ""
//!
//! [[output]]
//! path = "dist/a.png"
//! source = "src/a.png"
//! recipe = "recipes/web.toml"
//! key = "9f2c…"     # the pinned DEC-058 cache key (an identity of the INPUTS)
//! hash = "4ab1…"    # the observed output bytes
//! bytes = 1234
//! ```
//!
//! ## Pin the robust, record the fragile
//!
//! `key` is the shipped cache key (DEC-058): a domain-separated digest of tool
//! version + features + canonical recipe + quality + input extension + input
//! content. It is a function of the **inputs only**, so it reproduces on any
//! machine — which is why key equality *is* input equality, and a key change is
//! an unambiguous, cross-machine drift signal.
//!
//! `hash` is the SHA-256 of the bytes that were actually written. Encoders are
//! byte-identical run-to-run **within a machine** (STAGE-021's determinism
//! experiment) but not across arch/OS/codec versions — so the output hash is
//! *recorded as observed under `[env]`*, never promised. [`diff`] therefore
//! treats an output-hash change under the **same** `env.target` as a real
//! regression and under a **different** one as informational, unless `strict`.
//!
//! For the review-grade "did the image actually change?" question, the answer is
//! perceptual and already shipped: `crustyimg diff` (SSIMULACRA2, DEC-025). This
//! module compares digests, not pixels.
//!
//! ## Layering
//!
//! Like its sibling [`super`], this module is the library half: it parses,
//! serializes, and diffs. It never touches the filesystem — the executor
//! (`run_build` in `crate::cli`) reads and writes the file and owns the
//! `--check` / `--frozen` / `--locked` / `--strict` wiring.
//!
//! ## Validation (untrusted-input-hardening)
//!
//! The lockfile is committed config, so it is parsed with the same discipline as
//! a manifest (DEC-057) or a recipe (DEC-005): a [`LOCK_MAX_BYTES`] size guard
//! checked **before** parsing, `deny_unknown_fields`, and a `version` gate.
//! Keys, hashes, and paths are compared as opaque strings — nothing read out of
//! a lockfile is ever used to build a filesystem path.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ─── Constants ──────────────────────────────────────────────────────────────

/// The lockfile name, resolved against the process working directory — the same
/// convention the manifest and the cache root follow (DEC-057).
pub const DEFAULT_LOCK_FILE: &str = "crustyimg.build.lock";

/// The only lockfile schema version this build understands.
pub const SUPPORTED_LOCK_VERSION: u32 = 1;

/// Maximum allowed byte length of a lockfile (4 MiB).
///
/// Mirrors [`crate::recipe::RECIPE_MAX_BYTES`] and
/// [`super::BUILD_MANIFEST_MAX_BYTES`] in kind, not in size: a lockfile is
/// *generated*, one `[[output]]` (~250 bytes) per written file, so 4 MiB leaves
/// room for roughly sixteen thousand outputs. [`BuildLock::from_toml`] checks
/// `s.len()` before parsing; the CLI checks the on-disk size via `fs::metadata`
/// before reading (DEC-036). Reject only on `>`; equality is accepted.
pub const LOCK_MAX_BYTES: usize = 4 * 1024 * 1024;

// ─── LockError ──────────────────────────────────────────────────────────────

/// Errors parsing, validating, or serializing a [`BuildLock`] (DEC-007).
///
/// Every variant is a lockfile *content* error, so the CLI maps them all to exit
/// 2 (usage) — a malformed lockfile is a broken contract, not a failed check.
/// Reading the file is the caller's concern.
#[derive(Debug, Error)]
pub enum LockError {
    /// The lockfile TOML could not be parsed: malformed syntax, an unknown key
    /// (`deny_unknown_fields`), or a missing required key.
    #[error("could not parse build lockfile: {0}")]
    Parse(String),

    /// The lockfile's `version` is not supported by this build.
    #[error("unsupported build lockfile version {found} (supported: {supported})")]
    UnsupportedVersion {
        /// The `version` value found in the file.
        found: u32,
        /// The only version this binary understands.
        supported: u32,
    },

    /// The lockfile text exceeds [`LOCK_MAX_BYTES`] (checked before parsing).
    #[error("build lockfile is too large ({size} bytes; max {max})")]
    TooLarge {
        /// The actual byte length of the oversized text.
        size: usize,
        /// The cap that was exceeded.
        max: usize,
    },

    /// A [`BuildLock`] could not be rendered back to TOML. Unreachable for the
    /// shipped schema (scalars, then one table, then an array of scalar tables),
    /// but returned rather than panicked: library code does not `unwrap`.
    #[error("could not serialize build lockfile: {0}")]
    Serialize(String),
}

// ─── Schema ─────────────────────────────────────────────────────────────────

/// The environment a build's output hashes were **observed under**.
///
/// Not part of any output's identity — the pinned `key` already carries the tool
/// version and feature signature. `[env]` exists so [`diff`] can tell "the
/// encoder produced different bytes on this same machine" (a regression) from
/// "these bytes were recorded on another arch" (expected variance).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockEnv {
    /// The crustyimg version that observed these hashes.
    pub crustyimg_version: String,
    /// `"{ARCH}-{OS}"` from `std::env::consts` (e.g. `"aarch64-macos"`).
    pub target: String,
    /// The compiled-in encode-affecting cargo features (see
    /// [`super::cache::feature_signature`]); `""` for a default build.
    pub features: String,
}

/// One built output: where it was written, what produced it, and what it was.
///
/// `path` is the primary key. That it *can* be — that no two inputs of a build
/// map to one output path — is exactly what SPEC-065's injectivity guarantee
/// buys, and the lockfile is its second line of defense (the executor rejects a
/// duplicate `path` here even when the pre-decode check could not see it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockOutput {
    /// The output file, relative to the build's working directory, always
    /// `/`-separated so a lockfile committed on one OS reads on another.
    pub path: String,
    /// The source file this output was built from (provenance, for review).
    pub source: String,
    /// The recipe applied, as spelled in the manifest (provenance, for review).
    pub recipe: String,
    /// The **pinned** DEC-058 cache key, hex. A function of the inputs alone.
    pub key: String,
    /// The **recorded** SHA-256 of the written bytes, hex. Observed under `[env]`.
    pub hash: String,
    /// The written output's size in bytes (provenance, and a readable diff).
    pub bytes: u64,
}

/// A parsed `crustyimg.build.lock`.
///
/// Outputs are kept sorted by `path` at every entry point ([`BuildLock::new`],
/// [`BuildLock::from_toml`]), so serialization is deterministic: two clean builds
/// on one machine write byte-identical lockfiles, and a review diff shows only
/// what changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildLock {
    /// Schema version. Only [`SUPPORTED_LOCK_VERSION`] is accepted.
    pub version: u32,
    /// The environment the `hash` fields were observed under.
    pub env: LockEnv,
    /// One entry per written output, sorted by `path`. A build with no targets
    /// locks an empty output list.
    #[serde(default)]
    pub output: Vec<LockOutput>,
}

/// `"{ARCH}-{OS}"` for the running binary — the `[env].target` a build records.
///
/// From `std::env::consts`, so no dependency and no build script. This is a
/// coarse fingerprint on purpose: it separates the machines whose encoders can
/// legitimately disagree on bytes, and nothing finer is load-bearing (the tool
/// version and feature set live in the pinned key).
pub fn current_target() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

/// The [`LockEnv`] of the running binary.
pub fn current_env() -> LockEnv {
    LockEnv {
        crustyimg_version: crate::version().to_owned(),
        target: current_target(),
        features: super::cache::feature_signature(),
    }
}

impl BuildLock {
    /// A lockfile for the current schema version, with `output` sorted by `path`.
    pub fn new(env: LockEnv, mut output: Vec<LockOutput>) -> BuildLock {
        output.sort_by(|a, b| a.path.cmp(&b.path));
        BuildLock {
            version: SUPPORTED_LOCK_VERSION,
            env,
            output,
        }
    }

    /// Parse a lockfile TOML string and validate it end to end.
    ///
    /// Checks, in order: size cap → TOML parse (`deny_unknown_fields`) → version
    /// gate. Outputs are sorted by `path` on the way out, so a hand-edited
    /// lockfile diffs the same as a generated one. Nothing here touches the
    /// filesystem.
    pub fn from_toml(s: &str) -> Result<BuildLock, LockError> {
        // Size check BEFORE parsing (parse-time DoS prevention, DEC-036).
        if s.len() > LOCK_MAX_BYTES {
            return Err(LockError::TooLarge {
                size: s.len(),
                max: LOCK_MAX_BYTES,
            });
        }

        let mut lock: BuildLock = toml::from_str(s).map_err(|e| LockError::Parse(e.to_string()))?;

        // Version before anything semantic, so a future lockfile reports the
        // version rather than a cascade of field errors.
        if lock.version != SUPPORTED_LOCK_VERSION {
            return Err(LockError::UnsupportedVersion {
                found: lock.version,
                supported: SUPPORTED_LOCK_VERSION,
            });
        }

        // `key` and `hash` are compared and rendered as opaque digests, but they
        // arrive from a committed, hand-editable file (untrusted-input-hardening).
        // Validate that each is a non-empty hex string HERE, at the boundary, so a
        // hostile lockfile is a typed error (exit 2) — not a byte-slice panic
        // downstream on a multi-byte char, and not a silently-compared garbage
        // string. This is the same discipline as `deny_unknown_fields` + the
        // version gate: the lockfile is validated, not merely deserialized.
        for out in &lock.output {
            for (field, value) in [("key", &out.key), ("hash", &out.hash)] {
                if value.is_empty() || !value.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(LockError::Parse(format!(
                        "output \"{}\": {field} is not a hex digest",
                        out.path
                    )));
                }
            }
        }

        lock.output.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(lock)
    }

    /// Render this lockfile to TOML, outputs sorted by `path`.
    ///
    /// Deterministic: the same build on the same machine renders byte-identical
    /// text, which is what makes `--check` a diff and not a coin flip.
    pub fn to_toml(&self) -> Result<String, LockError> {
        let sorted = BuildLock::new(self.env.clone(), self.output.clone());
        toml::to_string(&sorted).map_err(|e| LockError::Serialize(e.to_string()))
    }

    /// Index this lockfile's outputs by `path`.
    fn by_path(&self) -> BTreeMap<&str, &LockOutput> {
        self.output
            .iter()
            .map(|o| (o.path.as_str(), o))
            .collect::<BTreeMap<_, _>>()
    }
}

// ─── The env-aware diff (DEC-059) ───────────────────────────────────────────

/// What changed about one output between the committed and the current lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockChangeKind {
    /// The current build produces an output the lockfile does not pin.
    Added,
    /// The lockfile pins an output the current build no longer produces.
    Removed,
    /// The pinned cache key changed: a source, recipe, quality, or the tool
    /// version differs. Input drift — unambiguous, and env-independent.
    KeyChanged {
        /// The key the lockfile pinned.
        committed: String,
        /// The key this build resolved.
        current: String,
    },
    /// Same inputs (same key), different output bytes, **same** `env.target`:
    /// this machine's encoder produced something else than it did before.
    HashChangedSameEnv {
        /// The output hash the lockfile recorded.
        committed: String,
        /// The output hash this build observed.
        current: String,
    },
    /// Same inputs, different output bytes, **different** `env.target`: expected
    /// cross-arch/OS encoder variance. Informational unless `strict`.
    HashChangedCrossEnv {
        /// The output hash the lockfile recorded.
        committed: String,
        /// The output hash this build observed.
        current: String,
        /// The `env.target` the lockfile recorded.
        committed_target: String,
        /// The `env.target` this build ran under.
        current_target: String,
    },
}

/// One output's difference, and whether it counts as drift under the active mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockChange {
    /// The output path this change concerns.
    pub path: String,
    /// What changed.
    pub kind: LockChangeKind,
    /// Whether this change fails a `--check`. Only a cross-env hash change can
    /// be `false`, and only when `strict` is off.
    pub drift: bool,
}

impl fmt::Display for LockChange {
    /// A one-line, reviewable rendering. Paths are quoted literally rather than
    /// with `{:?}`, which would double-escape a Windows separator (SPEC-065).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = if self.drift { "drift" } else { "note" };
        match &self.kind {
            LockChangeKind::Added => {
                write!(f, "{tag}: \"{}\": not in the lockfile", self.path)
            }
            LockChangeKind::Removed => {
                write!(f, "{tag}: \"{}\": in the lockfile, not built", self.path)
            }
            LockChangeKind::KeyChanged { committed, current } => write!(
                f,
                "{tag}: \"{}\": inputs changed (key {} → {})",
                self.path,
                short(committed),
                short(current)
            ),
            LockChangeKind::HashChangedSameEnv { committed, current } => write!(
                f,
                "{tag}: \"{}\": same inputs, different output bytes on this machine ({} → {})",
                self.path,
                short(committed),
                short(current)
            ),
            LockChangeKind::HashChangedCrossEnv {
                committed,
                current,
                committed_target,
                current_target,
            } => {
                // The hint has to know which side of `--strict` we are on: once
                // strict is set, "use --strict" is advice the user already took.
                let why = if self.drift {
                    "failing under --strict"
                } else {
                    "expected encoder variance — use --strict to fail on it"
                };
                write!(
                    f,
                    "{tag}: \"{}\": output bytes differ across environments \
                     ({committed_target} {} → {current_target} {}); {why}",
                    self.path,
                    short(committed),
                    short(current)
                )
            }
        }
    }
}

/// The first 12 hex characters of a digest — enough to identify, short enough to read.
///
/// `from_toml` validates every `key`/`hash` as hex, so in practice this only ever
/// sees single-byte ASCII. It still slices with `get(..n)` rather than `&hex[..n]`
/// so a byte index that isn't a char boundary can never panic — a free
/// belt-and-suspenders against any future caller that hasn't been through
/// validation (no-unwrap-on-recoverable-paths).
fn short(hex: &str) -> &str {
    let n = hex.len().min(12);
    hex.get(..n).unwrap_or(hex)
}

/// The result of comparing a committed lockfile against the current build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockDiff {
    /// True when at least one change is drift — the `--check` failure signal.
    pub drifted: bool,
    /// Every difference found, in `path` order. Includes informational
    /// (non-drift) changes, so `--check` can report cross-env variance while
    /// still exiting 0.
    pub changes: Vec<LockChange>,
}

impl LockDiff {
    /// True when the current build matches the lockfile exactly.
    pub fn is_clean(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Compare a committed lockfile against the current build, env-aware (DEC-059).
///
/// Keyed on output `path` (a valid primary key thanks to SPEC-065):
///
/// - a `path` in only one lock → **drift** (added / removed);
/// - same `path`, different `key` → **drift**, always: the key is a function of
///   the inputs alone, so it reproduces everywhere and its change is unambiguous;
/// - same `path` and `key`, different `hash`:
///   - same `env.target` → **drift** (a real output regression on this machine);
///   - different `env.target` → **informational**, because encoder bytes are not
///     promised across arch/OS — unless `strict`, which promotes it to drift.
///
/// `env` differences alone are never drift: a version or feature change already
/// moves every `key`, and a bare arch change is what the cross-env rule exists
/// to tolerate.
pub fn diff(committed: &BuildLock, current: &BuildLock, strict: bool) -> LockDiff {
    let same_env = committed.env.target == current.env.target;
    let (old, new) = (committed.by_path(), current.by_path());

    let mut changes: Vec<LockChange> = Vec::new();

    // `BTreeMap` iteration is sorted, so the union walk below is path-ordered.
    for (path, new_out) in &new {
        let Some(old_out) = old.get(path) else {
            changes.push(LockChange {
                path: (*path).to_owned(),
                kind: LockChangeKind::Added,
                drift: true,
            });
            continue;
        };

        if old_out.key != new_out.key {
            changes.push(LockChange {
                path: (*path).to_owned(),
                kind: LockChangeKind::KeyChanged {
                    committed: old_out.key.clone(),
                    current: new_out.key.clone(),
                },
                drift: true,
            });
            continue;
        }

        if old_out.hash != new_out.hash {
            let (kind, drift) = if same_env {
                (
                    LockChangeKind::HashChangedSameEnv {
                        committed: old_out.hash.clone(),
                        current: new_out.hash.clone(),
                    },
                    true,
                )
            } else {
                (
                    LockChangeKind::HashChangedCrossEnv {
                        committed: old_out.hash.clone(),
                        current: new_out.hash.clone(),
                        committed_target: committed.env.target.clone(),
                        current_target: current.env.target.clone(),
                    },
                    strict,
                )
            };
            changes.push(LockChange {
                path: (*path).to_owned(),
                kind,
                drift,
            });
        }
    }

    for path in old.keys() {
        if !new.contains_key(path) {
            changes.push(LockChange {
                path: (*path).to_owned(),
                kind: LockChangeKind::Removed,
                drift: true,
            });
        }
    }

    changes.sort_by(|a, b| a.path.cmp(&b.path));
    LockDiff {
        drifted: changes.iter().any(|c| c.drift),
        changes,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
version = 1

[env]
crustyimg_version = "0.4.0"
target = "aarch64-macos"
features = ""

[[output]]
path = "dist/b.png"
source = "src/b.png"
recipe = "r.toml"
key = "bbbb0000"
hash = "22220000"
bytes = 200

[[output]]
path = "dist/a.png"
source = "src/a.png"
recipe = "r.toml"
key = "aaaa0000"
hash = "11110000"
bytes = 100
"#;

    /// One output, with every field overridable by the caller.
    fn out(path: &str, key: &str, hash: &str) -> LockOutput {
        LockOutput {
            path: path.to_owned(),
            source: format!("src/{path}"),
            recipe: "r.toml".to_owned(),
            key: key.to_owned(),
            hash: hash.to_owned(),
            bytes: 100,
        }
    }

    fn env(target: &str) -> LockEnv {
        LockEnv {
            crustyimg_version: "0.4.0".to_owned(),
            target: target.to_owned(),
            features: String::new(),
        }
    }

    #[test]
    fn parses_valid_lock() {
        let lock = BuildLock::from_toml(VALID).expect("valid lockfile should parse");
        assert_eq!(lock.version, SUPPORTED_LOCK_VERSION);
        assert_eq!(lock.env.crustyimg_version, "0.4.0");
        assert_eq!(lock.env.target, "aarch64-macos");
        assert_eq!(lock.env.features, "");
        assert_eq!(lock.output.len(), 2);

        // Parsed in file order `b`, `a` — sorted by path on the way out.
        assert_eq!(lock.output[0].path, "dist/a.png");
        assert_eq!(lock.output[0].key, "aaaa0000");
        assert_eq!(lock.output[0].hash, "11110000");
        assert_eq!(lock.output[0].bytes, 100);
        assert_eq!(lock.output[0].source, "src/a.png");
        assert_eq!(lock.output[0].recipe, "r.toml");
        assert_eq!(lock.output[1].path, "dist/b.png");
    }

    #[test]
    fn empty_output_list_is_valid() {
        let toml_str =
            "version = 1\n[env]\ncrustyimg_version = \"0\"\ntarget = \"t\"\nfeatures = \"\"\n";
        let lock = BuildLock::from_toml(toml_str).expect("a no-target build locks no outputs");
        assert!(lock.output.is_empty());
    }

    #[test]
    fn rejects_unknown_field() {
        // A typo'd key in an output is rejected (deny_unknown_fields), not ignored.
        let toml_str = format!("{VALID}bogus = 1\n");
        let err = BuildLock::from_toml(&toml_str).expect_err("unknown field must be rejected");
        assert!(
            matches!(&err, LockError::Parse(msg) if msg.contains("bogus")),
            "expected Parse naming the unknown field, got {err:?}"
        );

        // ... and so is a typo'd key at the top level.
        let err = BuildLock::from_toml("version = 1\nversoin = 2\n")
            .expect_err("top-level unknown field must be rejected");
        assert!(matches!(err, LockError::Parse(_)), "got {err:?}");
    }

    #[test]
    fn rejects_unsupported_version() {
        let toml_str = VALID.replace("version = 1", "version = 999");
        let err = BuildLock::from_toml(&toml_str).expect_err("version must be gated");
        assert!(
            matches!(
                err,
                LockError::UnsupportedVersion {
                    found: 999,
                    supported: SUPPORTED_LOCK_VERSION
                }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn rejects_non_hex_digest() {
        // A committed lockfile is hand-editable (untrusted-input-hardening). A
        // non-hex `key`/`hash` must be a typed error (exit 2) — never a downstream
        // byte-slice panic on a multi-byte char, and never a silently-compared
        // garbage string. '€' is the multi-byte case that panicked `short` before.
        for bad in ["a\u{20ac}\u{20ac}\u{20ac}\u{20ac}", "not-hex!", "", "  "] {
            let toml_str = VALID.replacen("bbbb0000", bad, 1);
            let err = BuildLock::from_toml(&toml_str).expect_err("a non-hex key must be rejected");
            assert!(
                matches!(&err, LockError::Parse(msg) if msg.contains("key") && msg.contains("hex")),
                "expected Parse naming the non-hex key, got {err:?}"
            );
        }
        // The same guard applies to `hash`.
        let toml_str = VALID.replacen("22220000", "a\u{20ac}\u{20ac}\u{20ac}\u{20ac}", 1);
        let err = BuildLock::from_toml(&toml_str).expect_err("a non-hex hash must be rejected");
        assert!(
            matches!(&err, LockError::Parse(msg) if msg.contains("hash") && msg.contains("hex")),
            "got {err:?}"
        );
    }

    #[test]
    fn short_never_panics_on_multibyte() {
        // Defence in depth: even a digest that slipped validation must not panic
        // the 12-char slice. '€' is 3 bytes, so byte 12 of "a€€€€" (13 bytes) lands
        // mid-char — a naive `&s[..12]` would panic; `get(..12)` yields the whole
        // string instead.
        assert_eq!(
            short("a\u{20ac}\u{20ac}\u{20ac}\u{20ac}"),
            "a\u{20ac}\u{20ac}\u{20ac}\u{20ac}"
        );
        // ASCII digests slice normally.
        assert_eq!(short("abcdef0123456789"), "abcdef012345");
        assert_eq!(short("abc"), "abc");
    }

    #[test]
    fn rejects_oversize_lock() {
        // '#' is a TOML comment, so a successful parse would prove the size check
        // did NOT fire before parsing.
        let oversized = "#".repeat(LOCK_MAX_BYTES + 1);
        let err = BuildLock::from_toml(&oversized).expect_err("oversize must be rejected");
        assert!(
            matches!(err, LockError::TooLarge { size, max }
                if size == LOCK_MAX_BYTES + 1 && max == LOCK_MAX_BYTES),
            "got {err:?}"
        );
    }

    #[test]
    fn missing_required_field_is_error() {
        // No `[env]` at all, and an output missing `hash`.
        for toml_str in [
            "version = 1\n",
            &format!("{}\n[[output]]\npath = \"p\"\nsource = \"s\"\nrecipe = \"r\"\nkey = \"k\"\nbytes = 1\n",
                "version = 1\n[env]\ncrustyimg_version = \"0\"\ntarget = \"t\"\nfeatures = \"\""),
        ] {
            let err = BuildLock::from_toml(toml_str)
                .expect_err("a missing required field must be a typed error");
            assert!(matches!(err, LockError::Parse(_)), "got {err:?}");
        }
    }

    #[test]
    fn to_toml_from_toml_roundtrips() {
        // Constructed out of order: `new` and `to_toml` both sort by path.
        // Digests are hex (from_toml validates them), distinct per output.
        let lock = BuildLock::new(
            env("x86_64-linux"),
            vec![
                out("dist/z.png", "0c0c", "0d0d"),
                out("dist/a.png", "0a0a", "0b0b"),
                out("dist/m.png", "0e0e", "0f0f"),
            ],
        );
        let text = lock.to_toml().expect("shipped schema must serialize");
        let back = BuildLock::from_toml(&text).expect("own output must parse");

        assert_eq!(back, lock, "to_toml → from_toml must round-trip");
        let paths: Vec<&str> = back.output.iter().map(|o| o.path.as_str()).collect();
        assert_eq!(paths, ["dist/a.png", "dist/m.png", "dist/z.png"]);

        // Deterministic: re-rendering the parsed lock reproduces the same bytes.
        assert_eq!(back.to_toml().expect("re-render"), text);
    }

    #[test]
    fn current_env_is_populated() {
        let e = current_env();
        assert_eq!(e.crustyimg_version, crate::version());
        assert_eq!(e.target, current_target());
        assert!(e.target.contains('-'), "target is ARCH-OS: {}", e.target);
        assert_eq!(e.features, super::super::cache::feature_signature());
    }

    #[test]
    fn diff_identical_is_clean() {
        let lock = BuildLock::new(
            env("aarch64-macos"),
            vec![out("dist/a.png", "ka", "ha"), out("dist/b.png", "kb", "hb")],
        );
        let d = diff(&lock, &lock, false);
        assert!(!d.drifted);
        assert!(d.is_clean(), "no changes expected, got {:?}", d.changes);

        // Strict cannot invent drift out of an identical pair.
        assert!(!diff(&lock, &lock, true).drifted);
    }

    #[test]
    fn diff_added_or_removed_output_is_drift() {
        let one = BuildLock::new(env("t"), vec![out("dist/a.png", "ka", "ha")]);
        let two = BuildLock::new(
            env("t"),
            vec![out("dist/a.png", "ka", "ha"), out("dist/b.png", "kb", "hb")],
        );

        // Current has an output the lockfile doesn't pin.
        let d = diff(&one, &two, false);
        assert!(d.drifted);
        assert_eq!(d.changes.len(), 1);
        assert_eq!(d.changes[0].path, "dist/b.png");
        assert_eq!(d.changes[0].kind, LockChangeKind::Added);

        // The lockfile pins an output the build no longer produces.
        let d = diff(&two, &one, false);
        assert!(d.drifted);
        assert_eq!(d.changes.len(), 1);
        assert_eq!(d.changes[0].path, "dist/b.png");
        assert_eq!(d.changes[0].kind, LockChangeKind::Removed);
    }

    #[test]
    fn diff_key_change_is_drift() {
        let committed = BuildLock::new(env("aarch64-macos"), vec![out("dist/a.png", "ka", "ha")]);
        // A different key — the inputs changed. Even the same output bytes don't
        // excuse it, and it holds in BOTH env branches: key drift is env-blind.
        let current = BuildLock::new(env("aarch64-macos"), vec![out("dist/a.png", "KB", "ha")]);
        let d = diff(&committed, &current, false);
        assert!(d.drifted);
        assert!(matches!(
            d.changes[0].kind,
            LockChangeKind::KeyChanged { .. }
        ));

        let cross = BuildLock::new(env("x86_64-linux"), vec![out("dist/a.png", "KB", "ha")]);
        let d = diff(&committed, &cross, false);
        assert!(d.drifted, "key drift is a failure across environments too");
        assert!(matches!(
            d.changes[0].kind,
            LockChangeKind::KeyChanged { .. }
        ));
    }

    #[test]
    fn diff_hash_change_same_env_is_drift() {
        let committed = BuildLock::new(env("aarch64-macos"), vec![out("dist/a.png", "ka", "ha")]);
        let current = BuildLock::new(env("aarch64-macos"), vec![out("dist/a.png", "ka", "HB")]);
        let d = diff(&committed, &current, false);
        assert!(d.drifted, "same inputs, different bytes, same machine");
        assert!(matches!(
            d.changes[0].kind,
            LockChangeKind::HashChangedSameEnv { .. }
        ));
        assert!(d.changes[0].drift);
    }

    #[test]
    fn diff_hash_change_cross_env_is_informational() {
        // The cross-env branch can only be exercised here: `current_target()` is
        // fixed for a test binary, exactly as `CARGO_PKG_VERSION` is fixed for the
        // cache's version-invalidation test (SPEC-064). So the two locks are
        // constructed with differing `[env].target` directly.
        let committed = BuildLock::new(env("aarch64-macos"), vec![out("dist/a.png", "ka", "ha")]);
        let current = BuildLock::new(env("x86_64-linux"), vec![out("dist/a.png", "ka", "HB")]);

        let d = diff(&committed, &current, false);
        assert!(!d.drifted, "cross-env byte variance is expected, not drift");
        assert_eq!(d.changes.len(), 1, "but it is still reported");
        assert!(!d.changes[0].drift);
        assert!(matches!(
            d.changes[0].kind,
            LockChangeKind::HashChangedCrossEnv { .. }
        ));
        assert!(!d.is_clean());

        // `--strict` promotes exactly this change to a failure.
        let d = diff(&committed, &current, true);
        assert!(d.drifted);
        assert!(d.changes[0].drift);
    }

    #[test]
    fn diff_reports_changes_in_path_order() {
        let committed = BuildLock::new(
            env("t"),
            vec![out("dist/z.png", "kz", "hz"), out("dist/a.png", "ka", "ha")],
        );
        let current = BuildLock::new(
            env("t"),
            vec![out("dist/m.png", "km", "hm"), out("dist/a.png", "KA", "ha")],
        );
        let d = diff(&committed, &current, false);
        let paths: Vec<&str> = d.changes.iter().map(|c| c.path.as_str()).collect();
        // a: key changed, m: added, z: removed — all drift, sorted by path.
        assert_eq!(paths, ["dist/a.png", "dist/m.png", "dist/z.png"]);
        assert!(d.drifted);
    }

    #[test]
    fn change_display_names_the_path_and_reason() {
        let committed = BuildLock::new(env("aarch64-macos"), vec![out("dist/a.png", "ka", "ha")]);
        let current = BuildLock::new(
            env("aarch64-macos"),
            vec![out("dist/a.png", "kb1234567890ff", "ha")],
        );
        let rendered = diff(&committed, &current, false).changes[0].to_string();
        assert!(rendered.starts_with("drift: \"dist/a.png\""), "{rendered}");
        assert!(rendered.contains("inputs changed"), "{rendered}");
        // Digests are abbreviated, and never with `{:?}`.
        assert!(rendered.contains("kb1234567890"), "{rendered}");
        assert!(!rendered.contains("kb1234567890ff"), "{rendered}");

        // An informational change is tagged `note`, not `drift`.
        let cross = BuildLock::new(env("x86_64-linux"), vec![out("dist/a.png", "ka", "hb")]);
        let rendered = diff(&committed, &cross, false).changes[0].to_string();
        assert!(rendered.starts_with("note: "), "{rendered}");
        assert!(
            rendered.contains("use --strict to fail on it"),
            "{rendered}"
        );

        // Under `--strict` the same change is drift, and the hint stops telling
        // the user to pass a flag they already passed.
        let rendered = diff(&committed, &cross, true).changes[0].to_string();
        assert!(rendered.starts_with("drift: "), "{rendered}");
        assert!(rendered.contains("failing under --strict"), "{rendered}");
        assert!(!rendered.contains("use --strict"), "{rendered}");

        // Added / removed render their own reasons.
        let empty = BuildLock::new(env("aarch64-macos"), vec![]);
        assert!(diff(&empty, &committed, false).changes[0]
            .to_string()
            .contains("not in the lockfile"));
        assert!(diff(&committed, &empty, false).changes[0]
            .to_string()
            .contains("not built"));
    }
}
