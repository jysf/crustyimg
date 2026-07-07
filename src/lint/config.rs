//! `.crustyimg-lint.toml` config: discovery, parsing, and the effective merge
//! (SPEC-051, DEC-050).
//!
//! `lint` is zero-config by default; this module lets a project *tune* it with
//! conventions developers already know:
//!
//! - **ruff-style** `select`/`ignore` (rule-id prefixes) + `per_file_ignores`
//!   (a glob → rule ids suppressed on matching files).
//! - **eslint-style** per-rule severity overrides (`error`/`warn`/`info`/`off`).
//! - per-glob `[[budget]]` (`max_bytes`, `max_intended_width`) — the format-aware
//!   `--maxkb`, first consumed by SPEC-053's `size/oversized-bytes`.
//! - a **savings-threshold** gate (`min_bytes`/`min_percent`, default 4096/10 —
//!   Lighthouse's own 4 KiB floor) exposed to the engine-backed rules (STAGE-014).
//!
//! Discovery walks up from the first input (or cwd) to the filesystem root; the
//! nearest `.crustyimg-lint.toml` wins. `--config PATH` forces one; `--no-config`
//! ignores discovery. Precedence: **CLI flags > file > built-in defaults**. A
//! malformed config is a typed usage error (exit 2), never a panic. No new
//! dependency — reuses `serde`/`toml` (DEC-005) and the shipped `glob`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{known_rule_ids, Severity};

/// The config filename auto-discovered by walking up the tree.
pub const CONFIG_FILENAME: &str = ".crustyimg-lint.toml";

/// Reject a config file larger than this before parsing (untrusted-input
/// hardening, mirroring the recipe cap in DEC-036). 256 KiB is far above any
/// real lint config.
const CONFIG_MAX_BYTES: usize = 256 * 1024;

/// The savings-threshold defaults (DEC-050): Lighthouse's own 4 KiB floor + 10%.
pub const DEFAULT_MIN_BYTES: u64 = 4096;
pub const DEFAULT_MIN_PERCENT: u32 = 10;

// ── Runtime config ──────────────────────────────────────────────────────────

/// A per-rule severity override (eslint-style). `Off` disables the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityOverride {
    Error,
    Warn,
    Info,
    Off,
}

impl SeverityOverride {
    /// The overriding [`Severity`], or `None` when the rule is turned `off`.
    pub fn severity(self) -> Option<Severity> {
        match self {
            SeverityOverride::Error => Some(Severity::Error),
            SeverityOverride::Warn => Some(Severity::Warn),
            SeverityOverride::Info => Some(Severity::Info),
            SeverityOverride::Off => None,
        }
    }

    fn parse(s: &str) -> Option<SeverityOverride> {
        match s {
            "error" => Some(SeverityOverride::Error),
            "warn" => Some(SeverityOverride::Warn),
            "info" => Some(SeverityOverride::Info),
            "off" => Some(SeverityOverride::Off),
            _ => None,
        }
    }
}

/// A per-glob byte / intended-width budget (`[[budget]]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budget {
    pub glob: String,
    pub max_bytes: Option<u64>,
    pub max_intended_width: Option<u32>,
}

/// A `per_file_ignores` entry: suppress `rules` on files matching `glob`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerFileIgnore {
    pub glob: String,
    pub rules: Vec<String>,
}

/// The savings-threshold gate for "could be smaller" rules (STAGE-014).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavingsThreshold {
    pub min_bytes: u64,
    pub min_percent: u32,
}

impl Default for SavingsThreshold {
    fn default() -> Self {
        SavingsThreshold {
            min_bytes: DEFAULT_MIN_BYTES,
            min_percent: DEFAULT_MIN_PERCENT,
        }
    }
}

/// The effective lint config the runner applies (after merging CLI > file >
/// defaults). Zero-config produces [`LintConfig::default`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintConfig {
    /// Rule-id prefixes to include (empty ⇒ all default rules).
    pub select: Vec<String>,
    /// Rule-id prefixes to exclude.
    pub ignore: Vec<String>,
    /// Per-rule severity overrides (exact rule id → override).
    pub per_rule_severity: BTreeMap<String, SeverityOverride>,
    /// Per-glob budgets (later entries win on overlap).
    pub budgets: Vec<Budget>,
    /// Per-file rule suppressions.
    pub per_file_ignores: Vec<PerFileIgnore>,
    /// The savings-threshold gate.
    pub savings_threshold: SavingsThreshold,
    /// A globally-declared intended width (opt-in; source-file analogue of a
    /// rendered width). Feeds SPEC-053's `dims/oversized-dimensions`.
    pub max_intended_width: Option<u32>,
}

impl LintConfig {
    /// Whether `rule_id` is active under `select`/`ignore` and not turned `off`.
    ///
    /// `select` (when non-empty) restricts to matching prefixes; `ignore`
    /// removes matching prefixes; an explicit `off` severity disables the rule.
    pub fn is_rule_active(&self, rule_id: &str) -> bool {
        if matches!(
            self.per_rule_severity.get(rule_id),
            Some(SeverityOverride::Off)
        ) {
            return false;
        }
        if !self.select.is_empty() && !self.select.iter().any(|p| prefix_matches(p, rule_id)) {
            return false;
        }
        if self.ignore.iter().any(|p| prefix_matches(p, rule_id)) {
            return false;
        }
        true
    }

    /// The effective severity for `rule_id`, or `default` when not overridden.
    /// (An `off` rule never produces a finding, so its severity is moot.)
    pub fn severity_for(&self, rule_id: &str, default: Severity) -> Severity {
        self.per_rule_severity
            .get(rule_id)
            .and_then(|o| o.severity())
            .unwrap_or(default)
    }

    /// Whether `rule_id` is suppressed on `path` by a `per_file_ignores` entry.
    pub fn is_ignored_for_path(&self, rule_id: &str, path: &Path) -> bool {
        self.per_file_ignores
            .iter()
            .any(|pfi| pfi.rules.iter().any(|r| r == rule_id) && glob_matches(&pfi.glob, path))
    }

    /// The byte budget applying to `path` (last matching `[[budget]]` wins).
    pub fn byte_budget_for(&self, path: &Path) -> Option<u64> {
        self.budgets
            .iter()
            .rev()
            .find(|b| b.max_bytes.is_some() && glob_matches(&b.glob, path))
            .and_then(|b| b.max_bytes)
    }

    /// The intended width applying to `path`: a matching budget's, else the
    /// global `max_intended_width`.
    pub fn intended_width_for(&self, path: &Path) -> Option<u32> {
        self.budgets
            .iter()
            .rev()
            .find(|b| b.max_intended_width.is_some() && glob_matches(&b.glob, path))
            .and_then(|b| b.max_intended_width)
            .or(self.max_intended_width)
    }
}

/// Whether rule-id `id` matches ruff-style prefix `p`: an exact match or a
/// namespace prefix (`privacy` matches `privacy/gps-metadata-leak`).
fn prefix_matches(p: &str, id: &str) -> bool {
    id == p || id.starts_with(&format!("{p}/"))
}

/// Whether `path` matches shell-glob `pattern` (reuses the shipped `glob`
/// crate's `Pattern` — no new dependency).
fn glob_matches(pattern: &str, path: &Path) -> bool {
    glob::Pattern::new(pattern)
        .map(|p| p.matches_path(path))
        .unwrap_or(false)
}

// ── File schema (serde) ───────────────────────────────────────────────────────

/// The on-disk `.crustyimg-lint.toml` shape. Kept separate from [`LintConfig`]
/// so the runtime type stays serde-free and the string→enum parsing is explicit.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    select: Vec<String>,
    #[serde(default)]
    ignore: Vec<String>,
    #[serde(default)]
    max_intended_width: Option<u32>,
    #[serde(default)]
    severity: BTreeMap<String, String>,
    #[serde(default)]
    savings_threshold: Option<RawSavings>,
    #[serde(default, rename = "budget")]
    budgets: Vec<RawBudget>,
    #[serde(default)]
    per_file_ignores: Vec<RawPerFileIgnore>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSavings {
    #[serde(default = "default_min_bytes")]
    min_bytes: u64,
    #[serde(default = "default_min_percent")]
    min_percent: u32,
}

fn default_min_bytes() -> u64 {
    DEFAULT_MIN_BYTES
}
fn default_min_percent() -> u32 {
    DEFAULT_MIN_PERCENT
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBudget {
    glob: String,
    #[serde(default)]
    max_bytes: Option<u64>,
    #[serde(default)]
    max_intended_width: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPerFileIgnore {
    glob: String,
    #[serde(default)]
    rules: Vec<String>,
}

// ── Discovery ─────────────────────────────────────────────────────────────────

/// Walk up from `start` (a file or directory) to the filesystem root, returning
/// the nearest `.crustyimg-lint.toml`. `None` when none is found.
pub fn discover_config(start: &Path) -> Option<PathBuf> {
    // Begin at `start` if it is a directory, else its parent.
    let mut dir: Option<&Path> = if start.is_dir() {
        Some(start)
    } else {
        start.parent()
    };
    while let Some(d) = dir {
        let candidate = d.join(CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

// ── Parse + merge ─────────────────────────────────────────────────────────────

/// A parse/validation failure. Mapped to a usage error (exit 2) at the CLI.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file could not be read.
    #[error("could not read lint config '{path}': {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    /// The config exceeds [`CONFIG_MAX_BYTES`].
    #[error("lint config '{path}' is too large ({size} bytes; max {max})")]
    TooLarge {
        path: String,
        size: usize,
        max: usize,
    },
    /// The TOML was malformed.
    #[error("malformed lint config '{path}': {reason}")]
    Parse { path: String, reason: String },
    /// A `[severity]` value was not `error`/`warn`/`info`/`off`.
    #[error("invalid severity '{value}' for rule '{rule}' (expected error|warn|info|off)")]
    BadSeverity { rule: String, value: String },
    /// A `select`/`ignore`/`severity` entry named an unknown rule id.
    #[error("unknown lint rule id or prefix '{0}'")]
    UnknownRule(String),
}

/// The CLI overrides merged over the file (precedence: these win).
#[derive(Debug, Default, Clone)]
pub struct CliOverrides {
    pub select: Vec<String>,
    pub ignore: Vec<String>,
    pub max_intended_width: Option<u32>,
    pub savings_threshold: Option<SavingsThreshold>,
}

/// Load and validate a config file into a [`LintConfig`] (no CLI merge yet).
fn load_file(path: &Path) -> Result<LintConfig, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    if text.len() > CONFIG_MAX_BYTES {
        return Err(ConfigError::TooLarge {
            path: path.display().to_string(),
            size: text.len(),
            max: CONFIG_MAX_BYTES,
        });
    }
    let raw: RawConfig = toml::from_str(&text).map_err(|e| ConfigError::Parse {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    raw_into_config(raw)
}

/// Convert a parsed [`RawConfig`] into a runtime [`LintConfig`], parsing
/// severity strings and defaulting the savings threshold.
fn raw_into_config(raw: RawConfig) -> Result<LintConfig, ConfigError> {
    let mut per_rule_severity = BTreeMap::new();
    for (rule, value) in raw.severity {
        let ov = SeverityOverride::parse(&value).ok_or_else(|| ConfigError::BadSeverity {
            rule: rule.clone(),
            value: value.clone(),
        })?;
        per_rule_severity.insert(rule, ov);
    }
    let savings_threshold = raw
        .savings_threshold
        .map(|s| SavingsThreshold {
            min_bytes: s.min_bytes,
            min_percent: s.min_percent,
        })
        .unwrap_or_default();
    Ok(LintConfig {
        select: raw.select,
        ignore: raw.ignore,
        per_rule_severity,
        budgets: raw
            .budgets
            .into_iter()
            .map(|b| Budget {
                glob: b.glob,
                max_bytes: b.max_bytes,
                max_intended_width: b.max_intended_width,
            })
            .collect(),
        per_file_ignores: raw
            .per_file_ignores
            .into_iter()
            .map(|p| PerFileIgnore {
                glob: p.glob,
                rules: p.rules,
            })
            .collect(),
        savings_threshold,
        max_intended_width: raw.max_intended_width,
    })
}

/// Resolve the effective config: load the file (forced `--config`, else
/// discovered — unless `no_config`), merge the CLI overrides over it, and
/// validate rule ids.
///
/// Precedence: CLI overrides > file > built-in defaults. Zero-config (no file,
/// no flags) yields [`LintConfig::default`].
pub fn effective_config(
    cli: &CliOverrides,
    forced: Option<&Path>,
    discovered: Option<&Path>,
    no_config: bool,
) -> Result<LintConfig, ConfigError> {
    let mut config = if no_config {
        LintConfig::default()
    } else if let Some(path) = forced {
        load_file(path)?
    } else if let Some(path) = discovered {
        load_file(path)?
    } else {
        LintConfig::default()
    };

    // CLI overrides: select/ignore EXTEND the file's lists; scalars replace.
    config.select.extend(cli.select.iter().cloned());
    config.ignore.extend(cli.ignore.iter().cloned());
    if let Some(w) = cli.max_intended_width {
        config.max_intended_width = Some(w);
    }
    if let Some(t) = cli.savings_threshold {
        config.savings_threshold = t;
    }

    validate_rule_ids(&config)?;
    Ok(config)
}

/// Validate every `select`/`ignore` prefix and `severity` key against the known
/// rule-id catalog — an unknown id is a usage error (exit 2).
fn validate_rule_ids(config: &LintConfig) -> Result<(), ConfigError> {
    let known = known_rule_ids();
    let known_prefix = |p: &str| known.iter().any(|id| prefix_matches(p, id));
    for p in config.select.iter().chain(config.ignore.iter()) {
        if !known_prefix(p) {
            return Err(ConfigError::UnknownRule(p.clone()));
        }
    }
    for rule in config.per_rule_severity.keys() {
        if !known.iter().any(|id| id == rule) {
            return Err(ConfigError::UnknownRule(rule.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, body).unwrap();
        p
    }

    #[test]
    fn discovery_walks_up_and_picks_the_nearest_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Config at an ancestor, a nested start dir below it.
        write(root, CONFIG_FILENAME, "select = [\"privacy\"]\n");
        let nested = root.join("a").join("b");
        std::fs::create_dir_all(&nested).unwrap();

        let found = discover_config(&nested).expect("should find ancestor config");
        assert_eq!(found, root.join(CONFIG_FILENAME));

        // A nearer config wins.
        let nearer = write(&root.join("a"), CONFIG_FILENAME, "ignore = [\"color\"]\n");
        let found2 = discover_config(&nested).expect("should find nearer config");
        assert_eq!(found2, nearer);
    }

    #[test]
    fn select_ignore_filter_rules_and_unknown_id_is_usage_error() {
        // A known prefix is accepted; select restricts, ignore removes.
        let cfg = effective_config(
            &CliOverrides {
                select: vec!["privacy".into()],
                ..Default::default()
            },
            None,
            None,
            false,
        )
        .unwrap();
        assert!(cfg.is_rule_active("privacy/gps-metadata-leak"));
        assert!(!cfg.is_rule_active("size/truncated-or-corrupt"));

        // An unknown id → error.
        let err = effective_config(
            &CliOverrides {
                ignore: vec!["bogus/nope".into()],
                ..Default::default()
            },
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::UnknownRule(_)));
    }

    #[test]
    fn per_rule_severity_override_changes_severity_and_off_disables() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write(
            tmp.path(),
            CONFIG_FILENAME,
            "[severity]\n\"privacy/gps-metadata-leak\" = \"warn\"\n\"size/truncated-or-corrupt\" = \"off\"\n",
        );
        let cfg = effective_config(&CliOverrides::default(), Some(&path), None, false).unwrap();
        assert_eq!(
            cfg.severity_for("privacy/gps-metadata-leak", Severity::Error),
            Severity::Warn
        );
        assert!(
            !cfg.is_rule_active("size/truncated-or-corrupt"),
            "off disables"
        );
        assert!(cfg.is_rule_active("privacy/gps-metadata-leak"));
    }

    #[test]
    fn per_file_ignores_suppress_a_rule_for_a_matching_glob() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write(
            tmp.path(),
            CONFIG_FILENAME,
            "[[per_file_ignores]]\nglob = \"vendor/**\"\nrules = [\"privacy/gps-metadata-leak\"]\n",
        );
        let cfg = effective_config(&CliOverrides::default(), Some(&path), None, false).unwrap();
        assert!(cfg.is_ignored_for_path("privacy/gps-metadata-leak", Path::new("vendor/lib/x.jpg")));
        assert!(!cfg.is_ignored_for_path("privacy/gps-metadata-leak", Path::new("src/x.jpg")));
        // A different rule on the same path is not suppressed.
        assert!(
            !cfg.is_ignored_for_path("size/truncated-or-corrupt", Path::new("vendor/lib/x.jpg"))
        );
    }

    #[test]
    fn per_glob_byte_budget_resolves_by_path() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write(
            tmp.path(),
            CONFIG_FILENAME,
            "[[budget]]\nglob = \"assets/**\"\nmax_bytes = 100000\nmax_intended_width = 1600\n",
        );
        let cfg = effective_config(&CliOverrides::default(), Some(&path), None, false).unwrap();
        assert_eq!(
            cfg.byte_budget_for(Path::new("assets/hero.jpg")),
            Some(100000)
        );
        assert_eq!(cfg.byte_budget_for(Path::new("other/hero.jpg")), None);
        assert_eq!(
            cfg.intended_width_for(Path::new("assets/hero.jpg")),
            Some(1600)
        );
    }

    #[test]
    fn malformed_toml_is_a_config_error_not_a_panic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write(tmp.path(), CONFIG_FILENAME, "this is = = not toml\n");
        let err = effective_config(&CliOverrides::default(), Some(&path), None, false).unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn cli_overrides_extend_file_and_scalars_replace() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write(
            tmp.path(),
            CONFIG_FILENAME,
            "ignore = [\"size\"]\nmax_intended_width = 800\n",
        );
        let cfg = effective_config(
            &CliOverrides {
                ignore: vec!["privacy".into()],
                max_intended_width: Some(1920),
                ..Default::default()
            },
            Some(&path),
            None,
            false,
        )
        .unwrap();
        // ignore extended (both present); scalar replaced.
        assert!(cfg.ignore.contains(&"size".to_string()));
        assert!(cfg.ignore.contains(&"privacy".to_string()));
        assert_eq!(cfg.max_intended_width, Some(1920));
    }

    #[test]
    fn no_config_yields_defaults() {
        let cfg = LintConfig::default();
        assert_eq!(cfg.savings_threshold.min_bytes, DEFAULT_MIN_BYTES);
        assert_eq!(cfg.savings_threshold.min_percent, DEFAULT_MIN_PERCENT);
        assert!(cfg.is_rule_active("privacy/gps-metadata-leak"));
    }
}
