//! The `lint` command core — the rule framework + two foundational rules
//! (SPEC-050, DEC-050).
//!
//! `crustyimg lint [PATHS]…` is a **read-only, advisory** linter for an image
//! asset tree — `clippy`/`eslint`/`ruff` for images. This module owns the
//! framework:
//!
//! - [`Severity`] (`Error` > `Warn` > `Info`) and [`Finding`] (a problem on one
//!   file, carrying a stable rule id and a runnable fix fragment).
//! - The [`Rule`] trait: a rule is a small, pure `fn(&LintTarget) -> Option<Finding>`.
//! - [`LintTarget`]: the per-file input a rule reads — the path, the raw bytes,
//!   the decode `Result`, and lazily-derived `ImageInfo`/EXIF.
//! - [`run_lint`] (the runner), [`exit_code`] (the CI gate), and [`render_human`]
//!   (the grouped-by-file report).
//! - Two foundational rules proving the framework end-to-end:
//!   [`GpsMetadataLeak`] (the privacy moat) and [`TruncatedOrCorrupt`] (a decode
//!   failure is a *finding*, not an abort — DEC-050).
//!
//! ## Read-only by construction
//!
//! This module NEVER writes or modifies an image: there is no `Sink`/write path
//! here. A finding *names* a fix (`meta clean --gps`, `auto-orient`, …); running it
//! is the user's choice. Config, the JSON report, and the rest of the rule
//! catalog layer on in SPEC-051/052/053.

use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use crate::error::ImageError;
use crate::image::{Image, ImageInfo};
use crate::source::Input;

pub mod config;
mod report;
mod rules;

pub use report::{render_human, write_json, write_sarif};

use config::{LintConfig, SavingsThreshold};

// ── Severity ──────────────────────────────────────────────────────────────────

/// A finding's severity (DEC-050's 3-severity model, clippy/eslint/ruff-aligned).
///
/// The declaration order is also the sort/precedence order: `Error` sorts
/// before `Warn` before `Info`, and only `Error` (plus `Warn` over
/// `--max-warnings`, wired in SPEC-051) affects the exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Wrong or leaking (GPS, a hard budget breach, a corrupt asset) — fails CI.
    Error,
    /// A measured saving or correctness risk — fails only under `--max-warnings`.
    Warn,
    /// Opt-in polish — never affects the exit code.
    Info,
}

impl Severity {
    /// A stable lowercase label for the report / JSON (`"error"`/`"warn"`/`"info"`).
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Info => "info",
        }
    }
}

// ── Finding ───────────────────────────────────────────────────────────────────

/// One problem a rule found on one file.
///
/// `rule` is a stable, namespaced id (`privacy/gps-metadata-leak`) — a
/// compatibility surface (DEC-050). `fix` is a runnable `crustyimg` subcommand
/// *fragment* (e.g. `meta clean --gps`); the report renders it as
/// `crustyimg <fix> <file> -o <file> -y` (see [`Finding::fix_command`]), which
/// rewrites the file in place. `None` when there is no command to run (the fix
/// guidance is then in `message`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    file: PathBuf,
    rule: &'static str,
    severity: Severity,
    message: String,
    fix: Option<String>,
    /// Estimated bytes this finding's fix would save. `None` for the
    /// shipped-capability rules (SPEC-050/051/053); the engine-backed
    /// "could be smaller" rules populate it (STAGE-014).
    bytes_saved: Option<u64>,
    /// The finding came from standard input, whose "path" is only a stem: no
    /// printable command can name it.
    from_stdin: bool,
}

/// Which shell's quoting a printed fix uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shell {
    Posix,
    PowerShell,
}

/// The shell a user on this platform most likely pastes into: PowerShell on
/// Windows (the default terminal shell and the GitHub Actions default),
/// sh/bash/zsh everywhere else.
const HOST_SHELL: Shell = if cfg!(windows) {
    Shell::PowerShell
} else {
    Shell::Posix
};

/// Quote one path so `shell` reads it as exactly one argument. `None` when the
/// path is not valid UTF-8 (printing it lossily would name a different file).
fn shell_arg(path: &Path, shell: Shell) -> Option<String> {
    let raw = path.to_str()?;
    // Quoting does not stop clap reading `-x.jpg` as a flag; `./-x.jpg` names
    // the same file on every OS (Rust accepts `/` on Windows).
    let s = if raw.starts_with('-') {
        format!("./{raw}")
    } else {
        raw.to_string()
    };
    // The safe set is deliberately small: `,` makes an array in PowerShell, and
    // `=`, `~`, `@`, `$` and spaces mean something in one shell or another.
    let safe = |c: char| {
        c.is_ascii_alphanumeric()
            || matches!(c, '_' | '-' | '.' | '/')
            || (shell == Shell::PowerShell && matches!(c, '\\' | ':'))
    };
    if !s.is_empty() && s.chars().all(safe) {
        return Some(s);
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        match shell {
            Shell::Posix if c == '\'' => out.push_str("'\\''"),
            // PowerShell also treats U+2018..U+201B as single quotes.
            Shell::PowerShell if c == '\'' || ('\u{2018}'..='\u{201B}').contains(&c) => {
                out.push(c);
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out.push('\'');
    Some(out)
}

impl Finding {
    /// Build a finding. `fix` is a `crustyimg` subcommand fragment without the
    /// binary name, the file or `-o <file> -y` (all added when rendered), or `None`.
    ///
    /// `bytes_saved` defaults to `None`; the engine-backed rules set it via
    /// [`Finding::with_bytes_saved`].
    pub fn new(
        file: impl Into<PathBuf>,
        rule: &'static str,
        severity: Severity,
        message: impl Into<String>,
        fix: Option<String>,
    ) -> Finding {
        Finding {
            file: file.into(),
            rule,
            severity,
            message: message.into(),
            fix,
            bytes_saved: None,
            from_stdin: false,
        }
    }

    /// Mark this finding as coming from standard input (it then has no fix command).
    fn on_stdin(mut self) -> Finding {
        self.from_stdin = true;
        self
    }

    /// The file this finding is about.
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The stable rule id (`area/name`).
    pub fn rule(&self) -> &'static str {
        self.rule
    }

    /// The finding's (config-resolved) severity.
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The human message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The runnable `crustyimg` subcommand fragment, if any.
    pub fn fix(&self) -> Option<&str> {
        self.fix.as_deref()
    }

    /// Estimated bytes the fix would save (engine-backed rules only, STAGE-014).
    pub fn bytes_saved(&self) -> Option<u64> {
        self.bytes_saved
    }

    /// Return this finding with an estimated byte saving attached (STAGE-014).
    pub fn with_bytes_saved(mut self, bytes: u64) -> Finding {
        self.bytes_saved = Some(bytes);
        self
    }

    /// The full runnable fix: `crustyimg <fragment> <file> -o <file> -y`, with the
    /// file quoted for this platform's shell. Run verbatim it rewrites the file in
    /// place (without `-o` it would write the image to stdout and leave the file
    /// unchanged). `None` when the rule names no command, the input was stdin, or
    /// the path cannot be printed.
    pub fn fix_command(&self) -> Option<String> {
        self.fix_command_for(HOST_SHELL)
    }

    fn fix_command_for(&self, shell: Shell) -> Option<String> {
        if self.from_stdin {
            return None;
        }
        let fragment = self.fix.as_deref()?;
        let file = shell_arg(&self.file, shell)?;
        Some(format!("crustyimg {fragment} {file} -o {file} -y"))
    }

    /// Deterministic sort key: `(path, severity, rule)` (DEC-050 determinism).
    fn sort_key(&self) -> (&Path, Severity, &'static str) {
        (&self.file, self.severity, self.rule)
    }

    /// Return this finding with its severity replaced (the config per-rule
    /// severity override the runner applies, SPEC-051).
    pub fn with_severity(mut self, severity: Severity) -> Finding {
        self.severity = severity;
        self
    }
}

// ── LintTarget ────────────────────────────────────────────────────────────────

/// The per-file input a [`Rule`] reads.
///
/// Carries the path, the raw file bytes, the decode `Result` (so
/// `truncated-or-corrupt` sees the error), and lazily-derived `ImageInfo`/EXIF
/// signals (so cheap rules don't force work they don't need). Read-only: a
/// target is inspected, never written.
pub struct LintTarget {
    path: PathBuf,
    bytes: Vec<u8>,
    decoded: std::result::Result<Image, ImageError>,
    info: OnceCell<Option<ImageInfo>>,
    exif: OnceCell<ExifFacts>,
    // Config-resolved per-file settings (SPEC-051), consumed by the
    // budget/dimension/engine rules (SPEC-053 / STAGE-014).
    byte_budget: Option<u64>,
    intended_width: Option<u32>,
    savings_threshold: SavingsThreshold,
}

impl LintTarget {
    /// Build a target from raw bytes and a display path, decoding once, with
    /// **default** (unconfigured) per-file settings.
    ///
    /// Empty or undecodable bytes produce a decode `Err` (which
    /// [`TruncatedOrCorrupt`] turns into a finding) — never a panic.
    pub fn from_bytes(path: impl Into<PathBuf>, bytes: Vec<u8>) -> LintTarget {
        LintTarget::build(path.into(), bytes, None, None, SavingsThreshold::default())
    }

    /// The shared constructor: decode once and store the resolved settings.
    fn build(
        path: PathBuf,
        bytes: Vec<u8>,
        byte_budget: Option<u64>,
        intended_width: Option<u32>,
        savings_threshold: SavingsThreshold,
    ) -> LintTarget {
        // Route by PATH, not by bytes: a RAW container is byte-ambiguous with a
        // plain TIFF, so it is extension-routed (SPEC-061/DEC-055) and
        // `Image::decode_path` is the one seam that owns that decision — its
        // contract is that every command decoding a `Path` goes through it.
        // `lint` did not, so it decoded RAW with the generic byte decoder and
        // called every valid `.nef`/`.cr2` "truncated or corrupt … re-export a
        // valid image" — failing CI on a directory of RAW files (SPEC-071 fix 1;
        // the [IMAGE_EXTENSIONS-exposes-every-decode-caller] lesson again).
        //
        // For a stdin target the path is only a stem, which has no RAW extension,
        // so this degrades to exactly the previous byte-sniffed decode.
        let decoded = Image::decode_path(&path, &bytes);
        LintTarget {
            path,
            bytes,
            decoded,
            info: OnceCell::new(),
            exif: OnceCell::new(),
            byte_budget,
            intended_width,
            savings_threshold,
        }
    }

    /// Build a target from a resolved [`Input`], reading the file once and
    /// resolving `config`'s per-file settings (budget / intended width /
    /// savings threshold) for this path.
    ///
    /// An unreadable path yields empty bytes → a decode `Err` (handled as a
    /// finding), so a resolution/read race never aborts the run.
    fn from_input(input: &Input, config: &LintConfig) -> LintTarget {
        let (path, bytes) = match input {
            Input::Path(p) => (p.clone(), std::fs::read(p).unwrap_or_default()),
            Input::Stdin { bytes, stem } => (PathBuf::from(stem), bytes.clone()),
        };
        let byte_budget = config.byte_budget_for(&path);
        let intended_width = config.intended_width_for(&path);
        LintTarget::build(
            path,
            bytes,
            byte_budget,
            intended_width,
            config.savings_threshold,
        )
    }

    /// The file path (the input path as given).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The raw file bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The decode result: `Ok` with the decoded image, or the typed decode error.
    pub fn decoded(&self) -> std::result::Result<&Image, &ImageError> {
        self.decoded.as_ref()
    }

    /// Lazily-derived inspection info, or `None` when the file did not decode.
    pub fn info(&self) -> Option<&ImageInfo> {
        self.info
            .get_or_init(|| self.decoded.as_ref().ok().map(|img| img.info()))
            .as_ref()
    }

    /// The EXIF facts, parsed once (lazily) via the shipped `kamadak-exif` read
    /// side (DEC-013, DEC-003) — no new parser. "No EXIF" / malformed EXIF ⇒ the
    /// default (empty) facts, never an error.
    fn exif_facts(&self) -> &ExifFacts {
        self.exif.get_or_init(|| scan_exif(&self.bytes))
    }

    /// Whether the file's EXIF carries a GPS location (a coordinate or a place name).
    pub fn has_gps_exif(&self) -> bool {
        self.exif_facts().has_gps
    }

    /// Whether the file's EXIF carries identifying (non-GPS) camera metadata —
    /// Make/Model/serial/lens/owner/original-timestamp. Distinct from GPS.
    pub fn has_camera_metadata(&self) -> bool {
        self.exif_facts().has_camera
    }

    /// The EXIF Orientation value (1–8), if present.
    pub fn exif_orientation(&self) -> Option<u16> {
        self.exif_facts().orientation
    }

    /// Whether the file carries a raw ICC profile (captured at load).
    pub fn has_icc(&self) -> bool {
        self.info().map(|i| i.has_icc).unwrap_or(false)
    }

    /// The `meta` fix for a finding, or `None` when the metadata lane cannot
    /// rewrite this file. The lane handles JPEG and PNG only, judged from the
    /// sniffed container (the same `guess_format` the lane uses), never the
    /// file extension; a command it would refuse must not be printed.
    fn meta_fix(&self, fragment: &str) -> Option<String> {
        matches!(
            ::image::guess_format(&self.bytes),
            Ok(::image::ImageFormat::Jpeg | ::image::ImageFormat::Png)
        )
        .then(|| fragment.to_string())
    }

    /// `message` for a finding whose fix is a `meta` verb: unchanged when the
    /// lane can write the file, else it says so and names the format.
    fn meta_message(&self, message: &str) -> String {
        if self.meta_fix("").is_some() {
            return message.to_string();
        }
        let format = match ::image::guess_format(&self.bytes) {
            Ok(::image::ImageFormat::Tiff) => "TIFF".to_string(),
            Ok(::image::ImageFormat::Avif) => "AVIF".to_string(),
            Ok(f) => format!("{f:?}"),
            Err(_) => "this format".to_string(),
        };
        format!(
            "{message}; crustyimg's metadata lane cannot rewrite {format} yet, so remove it \
             with another tool"
        )
    }

    /// The captured ICC profile's byte length, if any.
    pub fn icc_len(&self) -> Option<usize> {
        self.decoded
            .as_ref()
            .ok()
            .and_then(|img| img.metadata())
            .and_then(|m| m.icc.as_ref())
            .map(|v| v.len())
    }

    /// The byte budget applying to this file (from config `[[budget]]`), if any.
    /// Consumed by SPEC-053's `size/oversized-bytes`.
    pub fn byte_budget(&self) -> Option<u64> {
        self.byte_budget
    }

    /// The declared intended width applying to this file, if any (opt-in).
    /// Consumed by SPEC-053's `dims/oversized-dimensions`.
    pub fn intended_width(&self) -> Option<u32> {
        self.intended_width
    }

    /// The savings-threshold gate applying to this file (default 4096/10).
    /// Consumed by the engine-backed "could be smaller" rules (STAGE-014).
    pub fn savings_threshold(&self) -> SavingsThreshold {
        self.savings_threshold
    }
}

/// Read-only EXIF facts a rule keys off, extracted in one parse pass.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ExifFacts {
    /// The GPS block carries a location: a coordinate or a place name.
    has_gps: bool,
    /// Any identifying non-GPS camera tag is present (Make/Model/serial/…).
    has_camera: bool,
    /// The EXIF Orientation value (1–8), if present.
    orientation: Option<u16>,
}

/// Parse a container's EXIF once (read-only). "No EXIF" / malformed EXIF ⇒ the
/// default (empty) facts, never an error.
fn scan_exif(bytes: &[u8]) -> ExifFacts {
    use exif::{In, Tag};
    let exif = match exif::Reader::new().read_from_container(&mut std::io::Cursor::new(bytes)) {
        Ok(e) => e,
        Err(_) => return ExifFacts::default(),
    };

    // A location is a coordinate (where the photo was taken, or the destination
    // it names) or a free-text place name. A block holding only GPSVersionID,
    // Ref tags, altitude, timestamps or map datum says nothing about where (some
    // cameras write one into every JPEG). Any IFD counts: a location in a
    // thumbnail's GPS IFD is still a leak.
    const GPS_LOCATION_TAGS: &[Tag] = &[
        Tag::GPSLatitude,
        Tag::GPSLongitude,
        Tag::GPSDestLatitude,
        Tag::GPSDestLongitude,
        Tag::GPSAreaInformation,
    ];
    let has_gps = exif.fields().any(|f| GPS_LOCATION_TAGS.contains(&f.tag));

    // Identifying, device-level tags (NOT copyright/artist, which are preserved).
    const CAMERA_TAGS: &[Tag] = &[
        Tag::Make,
        Tag::Model,
        Tag::BodySerialNumber,
        Tag::LensModel,
        Tag::LensMake,
        Tag::DateTimeOriginal,
    ];
    let has_camera = exif
        .fields()
        .any(|f| f.ifd_num == In::PRIMARY && CAMERA_TAGS.contains(&f.tag));

    let orientation = exif
        .get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .map(|v| v as u16);

    ExifFacts {
        has_gps,
        has_camera,
        orientation,
    }
}

// ── Rule trait + registry ─────────────────────────────────────────────────────

/// A single lint rule: a small, pure check over one [`LintTarget`].
///
/// A rule never writes; it returns `Some(Finding)` when it fires, `None` when
/// the target is clean. `id` is the stable, namespaced rule id (DEC-050).
pub trait Rule {
    /// The stable rule id (`area/name`).
    fn id(&self) -> &'static str;

    /// The default severity when no config overrides it.
    fn default_severity(&self) -> Severity;

    /// Whether the rule runs by default (DEC-050 catalog). Opt-in rules return
    /// `false` — they run only when the config explicitly enables them (a
    /// `select` prefix or a per-rule severity entry). Defaults to `true`.
    fn default_enabled(&self) -> bool {
        true
    }

    /// Check the target; `Some` when the rule fires, `None` when clean.
    fn check(&self, target: &LintTarget) -> Option<Finding>;
}

/// The default rule set (DEC-050). SPEC-050 registered the two foundational
/// rules; SPEC-053 adds the shipped-capability catalog; STAGE-014 appends the
/// engine-backed rules.
pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // Foundational (SPEC-050).
        Box::new(GpsMetadataLeak),
        Box::new(TruncatedOrCorrupt),
        // Shipped-capability (SPEC-053).
        Box::new(rules::CameraMetadata),
        Box::new(rules::OrientationNotBaked),
        Box::new(rules::OversizedBytes),
        Box::new(rules::OversizedDimensions),
        Box::new(rules::WrongColorspace),
        Box::new(rules::MissingIcc),
        Box::new(rules::UnexpectedIcc),
        Box::new(rules::AnimatedGif),
        // SPEC-119: the sibling covering APNG and animated WebP — see
        // `rules::AnimatedInput`'s doc comment for why this is a separate
        // rule id rather than a broadened `format/animated-gif`.
        Box::new(rules::AnimatedInput),
    ]
}

/// Every known rule id (the DEC-050 stability surface) — the catalog `select`/
/// `ignore`/`severity` config entries validate against (SPEC-051).
pub fn known_rule_ids() -> Vec<&'static str> {
    default_rules().iter().map(|r| r.id()).collect()
}

// ── Runner + outcome ──────────────────────────────────────────────────────────

/// The result of a lint run: every finding (deterministically sorted) plus the
/// number of files scanned.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LintOutcome {
    /// Findings, sorted by `(path, severity, rule)`.
    pub findings: Vec<Finding>,
    /// How many image files were scanned.
    pub files_scanned: usize,
}

impl LintOutcome {
    /// The number of findings at `Error` severity.
    pub fn error_count(&self) -> usize {
        self.count(Severity::Error)
    }

    /// The number of findings at `Warn` severity.
    pub fn warn_count(&self) -> usize {
        self.count(Severity::Warn)
    }

    /// The number of findings at `Info` severity.
    pub fn info_count(&self) -> usize {
        self.count(Severity::Info)
    }

    fn count(&self, sev: Severity) -> usize {
        self.findings.iter().filter(|f| f.severity == sev).count()
    }

    /// Total estimated bytes saveable across all findings (0 until the
    /// engine-backed rules land, STAGE-014).
    pub fn potential_bytes_saved(&self) -> u64 {
        self.findings.iter().filter_map(|f| f.bytes_saved).sum()
    }
}

/// Run the active rules over every resolved input and collect a sorted outcome,
/// applying `config` (SPEC-051).
///
/// For each rule the config decides: whether it is active (`select`/`ignore`/
/// `off`), whether a finding is suppressed on this file (`per_file_ignores`),
/// and the finding's effective severity (per-rule override). Read-only: builds a
/// [`LintTarget`] per input (decoding once), and NEVER aborts on a bad file — a
/// decode failure is a finding via [`TruncatedOrCorrupt`] (DEC-050). Findings are
/// sorted deterministically by `(path, severity, rule)`.
pub fn run_lint(inputs: &[Input], rules: &[Box<dyn Rule>], config: &LintConfig) -> LintOutcome {
    let active_ids: std::collections::HashSet<&str> = rules
        .iter()
        .filter(|r| config.is_rule_enabled(r.id(), r.default_enabled()))
        .map(|r| r.id())
        .collect();

    let mut findings = Vec::new();
    let mut files_scanned = 0usize;
    for input in inputs {
        let target = LintTarget::from_input(input, config);
        files_scanned += 1;
        for rule in rules {
            if !active_ids.contains(rule.id()) {
                continue;
            }
            let Some(finding) = rule.check(&target) else {
                continue;
            };
            // Per-file suppression (per_file_ignores).
            if config.is_ignored_for_path(rule.id(), target.path()) {
                continue;
            }
            // Per-rule severity override.
            let severity = config.severity_for(rule.id(), finding.severity());
            let finding = finding.with_severity(severity);
            // The input variant decides, not the name: a real file called
            // `stdin` still gets its fix.
            findings.push(if matches!(input, Input::Stdin { .. }) {
                finding.on_stdin()
            } else {
                finding
            });
        }
    }
    findings.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    LintOutcome {
        findings,
        files_scanned,
    }
}

/// Map an outcome to a CI exit code (DEC-050, reusing exit 7 / `CheckFailed`).
///
/// `7` when there is ≥1 `Error`, or (when `max_warnings` is set) the `Warn`
/// count exceeds it; `0` otherwise. `Info` never fails. SPEC-050 passes
/// `max_warnings = None`, so `Warn` alone does not fail; SPEC-051 wires the flag.
pub fn exit_code(outcome: &LintOutcome, max_warnings: Option<usize>) -> i32 {
    if outcome.error_count() > 0 {
        return 7;
    }
    if let Some(max) = max_warnings {
        if outcome.warn_count() > max {
            return 7;
        }
    }
    0
}

// ── Foundational rules ────────────────────────────────────────────────────────

/// `privacy/gps-metadata-leak` (Error): the image carries a GPS location (a
/// coordinate or place name, not just an empty GPS block) — a location
/// leak in a public asset (the privacy moat). Fix: `meta clean --gps`.
pub struct GpsMetadataLeak;

impl Rule for GpsMetadataLeak {
    fn id(&self) -> &'static str {
        "privacy/gps-metadata-leak"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, target: &LintTarget) -> Option<Finding> {
        if target.has_gps_exif() {
            Some(Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                target.meta_message(
                    "image carries GPS location metadata (a privacy leak in a public asset)",
                ),
                target.meta_fix("meta clean --gps"),
            ))
        } else {
            None
        }
    }
}

/// `size/truncated-or-corrupt` (Error): the file failed to decode. A linter
/// reports a broken asset — it does NOT abort (the one deliberate divergence,
/// DEC-050). No fix command (re-export a valid image).
pub struct TruncatedOrCorrupt;

impl Rule for TruncatedOrCorrupt {
    fn id(&self) -> &'static str {
        "size/truncated-or-corrupt"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, target: &LintTarget) -> Option<Finding> {
        match target.decoded() {
            Ok(_) => None,
            // A feature-gated decoder that is simply not compiled in (a `.heic` in
            // the default build, SPEC-062/DEC-052) says NOTHING about the file. It
            // is valid; we just cannot read it. Reporting "truncated or corrupt …
            // re-export a valid image" would be a false diagnosis with a
            // destructive remedy, and would fail CI on any directory of iPhone
            // photos. Stay silent instead. (Follow-up: a `meta/not-inspected` Info
            // finding would be the fully honest answer — this rule cannot carry it,
            // since its severity is fixed at Error.)
            Err(ImageError::CodecNotBuilt { .. }) => None,
            // Same shape, same reasoning: an image whose declared dimensions are
            // outside our decode budget (SPEC-070/DEC-063 — a 100 MP medium-format
            // frame, a big stitched panorama) is a VALID image we decline to
            // decode. Calling it "truncated or corrupt" is a false diagnosis, and
            // "re-export a valid image" is a remedy for a problem it does not have.
            // (Follow-up: a dedicated `size/over-decode-budget` rule could report
            // this honestly; this rule's severity is fixed at Error, so it cannot.)
            Err(ImageError::LimitsExceeded(_)) => None,
            Err(_) => Some(Finding::new(
                target.path().to_path_buf(),
                self.id(),
                self.default_severity(),
                "image is truncated or corrupt (failed to decode); re-export a valid image",
                None,
            )),
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2×2 white PNG (valid, decodable, no EXIF).
    fn clean_jpeg() -> Vec<u8> {
        use image::{DynamicImage, ImageFormat, RgbImage};
        let img = RgbImage::from_pixel(2, 2, image::Rgb([255, 255, 255]));
        let mut buf = std::io::Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Jpeg)
            .unwrap();
        buf.into_inner()
    }

    /// One hand-built GPS IFD entry: (tag, TIFF type, count, value bytes).
    type GpsEntry = (u16, u16, u32, Vec<u8>);

    const BYTE: u16 = 1;
    const ASCII: u16 = 2;
    const RATIONAL: u16 = 5;
    const UNDEFINED: u16 = 7;

    /// A JPEG whose EXIF GPS IFD holds exactly `entries` (hand-built
    /// little-endian TIFF, not written by crustyimg's own metadata writer).
    /// Values over 4 bytes go in a data area after the IFD, with computed offsets.
    fn gps_jpeg_with(entries: &[GpsEntry]) -> Vec<u8> {
        let base = clean_jpeg();
        assert_eq!(&base[0..2], &[0xFF, 0xD8], "expected a JPEG (SOI)");

        let mut tiff: Vec<u8> = Vec::new();
        tiff.extend_from_slice(&[0x49, 0x49, 0x2A, 0x00]); // "II", 42
        tiff.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD0 offset = 8
        tiff.extend_from_slice(&[0x01, 0x00]); // IFD0: 1 entry
        tiff.extend_from_slice(&[0x25, 0x88]); // tag 0x8825 (GPSInfo pointer)
        tiff.extend_from_slice(&[0x04, 0x00]); // type LONG
        tiff.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // count 1
        tiff.extend_from_slice(&26u32.to_le_bytes()); // GPS IFD at offset 26
        tiff.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next IFD = 0
        assert_eq!(tiff.len(), 26);

        let ifd_len = 2 + 12 * entries.len() + 4;
        let mut data_offset = 26 + ifd_len;
        let mut ifd: Vec<u8> = Vec::new();
        let mut data: Vec<u8> = Vec::new();
        ifd.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        for (tag, ty, count, value) in entries {
            ifd.extend_from_slice(&tag.to_le_bytes());
            ifd.extend_from_slice(&ty.to_le_bytes());
            ifd.extend_from_slice(&count.to_le_bytes());
            if value.len() <= 4 {
                let mut inline = value.clone();
                inline.resize(4, 0);
                ifd.extend_from_slice(&inline);
            } else {
                ifd.extend_from_slice(&(data_offset as u32).to_le_bytes());
                data.extend_from_slice(value);
                if value.len() % 2 == 1 {
                    data.push(0);
                }
                data_offset = 26 + ifd_len + data.len();
            }
        }
        ifd.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // next IFD = 0
        tiff.extend_from_slice(&ifd);
        tiff.extend_from_slice(&data);

        let mut payload: Vec<u8> = Vec::new();
        payload.extend_from_slice(b"Exif\0\0");
        payload.extend_from_slice(&tiff);

        let seg_len = (payload.len() + 2) as u16;
        let mut out = Vec::with_capacity(base.len() + payload.len() + 4);
        out.extend_from_slice(&base[0..2]); // SOI
        out.push(0xFF);
        out.push(0xE1); // APP1
        out.extend_from_slice(&seg_len.to_be_bytes());
        out.extend_from_slice(&payload);
        out.extend_from_slice(&base[2..]);
        out
    }

    fn rationals(values: &[(u32, u32)]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|(n, d)| n.to_le_bytes().into_iter().chain(d.to_le_bytes()))
            .collect()
    }

    fn lat_ref() -> GpsEntry {
        (0x0001, ASCII, 2, b"N\0".to_vec())
    }
    fn lat() -> GpsEntry {
        (
            0x0002,
            RATIONAL,
            3,
            rationals(&[(50, 1), (29, 1), (1113, 25)]),
        )
    }
    fn lon_ref() -> GpsEntry {
        (0x0003, ASCII, 2, b"E\0".to_vec())
    }
    fn lon() -> GpsEntry {
        (0x0004, RATIONAL, 3, rationals(&[(30, 1), (31, 1), (0, 1)]))
    }

    /// A JPEG with a full position: lat + lon and their refs.
    fn gps_jpeg() -> Vec<u8> {
        gps_jpeg_with(&[lat_ref(), lat(), lon_ref(), lon()])
    }

    fn finding(path: &str, rule: &'static str, sev: Severity) -> Finding {
        Finding::new(PathBuf::from(path), rule, sev, "msg", None)
    }

    #[test]
    fn severity_to_exit_code_any_error_is_7_only_info_warn_is_0() {
        // Any Error ⇒ 7.
        let with_error = LintOutcome {
            findings: vec![finding(
                "a.png",
                "size/truncated-or-corrupt",
                Severity::Error,
            )],
            files_scanned: 1,
        };
        assert_eq!(exit_code(&with_error, None), 7);

        // Only Info/Warn and no max-warnings ⇒ 0.
        let info_warn = LintOutcome {
            findings: vec![
                finding("a.png", "x/warn", Severity::Warn),
                finding("b.png", "x/info", Severity::Info),
            ],
            files_scanned: 2,
        };
        assert_eq!(exit_code(&info_warn, None), 0);
    }

    #[test]
    fn findings_sort_by_path_then_severity_then_rule() {
        let mut outcome = LintOutcome {
            findings: vec![
                finding("b.png", "z/rule", Severity::Info),
                finding("a.png", "m/rule", Severity::Warn),
                finding("a.png", "a/rule", Severity::Error),
                finding("a.png", "b/rule", Severity::Error),
            ],
            files_scanned: 2,
        };
        outcome
            .findings
            .sort_by(|x, y| x.sort_key().cmp(&y.sort_key()));
        let order: Vec<_> = outcome
            .findings
            .iter()
            .map(|f| (f.file().display().to_string(), f.rule()))
            .collect();
        assert_eq!(
            order,
            vec![
                ("a.png".to_string(), "a/rule"),
                ("a.png".to_string(), "b/rule"),
                ("a.png".to_string(), "m/rule"),
                ("b.png".to_string(), "z/rule"),
            ]
        );
    }

    /// FT-10: the rule keys on a location tag, not on any tag in the GPS IFD.
    #[test]
    fn gps_predicate_keys_on_a_location_tag() {
        let leaky = LintTarget::from_bytes("photo.jpg", gps_jpeg());
        let f = GpsMetadataLeak.check(&leaky).expect("GPS finding expected");
        assert_eq!(f.rule(), "privacy/gps-metadata-leak");
        assert_eq!(f.severity(), Severity::Error);
        assert_eq!(f.fix(), Some("meta clean --gps"));

        let clean = LintTarget::from_bytes("clean.jpg", clean_jpeg());
        assert!(GpsMetadataLeak.check(&clean).is_none());

        let dest_lat = (0x0014, RATIONAL, 3, rationals(&[(50, 1), (0, 1), (0, 1)]));
        let dest_lon = (0x0016, RATIONAL, 3, rationals(&[(30, 1), (0, 1), (0, 1)]));
        let area = (0x001C, UNDEFINED, 12, b"ASCII\0\0\0Kyiv".to_vec());
        let must_fire: Vec<(&str, Vec<GpsEntry>)> = vec![
            ("latitude only", vec![lat()]),
            ("longitude only", vec![lon()]),
            ("dest latitude only", vec![dest_lat]),
            ("dest longitude only", vec![dest_lon]),
            ("area information only", vec![area]),
        ];
        for (name, entries) in &must_fire {
            let t = LintTarget::from_bytes("p.jpg", gps_jpeg_with(entries));
            assert!(
                GpsMetadataLeak.check(&t).is_some(),
                "{name}: a location must fire the rule"
            );
        }

        let alt = (0x0006, RATIONAL, 1, rationals(&[(120, 1)]));
        let alt_ref = (0x0005, BYTE, 1, vec![0]);
        let time = (0x0007, RATIONAL, 3, rationals(&[(12, 1), (0, 1), (0, 1)]));
        let date = (0x001D, ASCII, 11, b"2026:11:05\0".to_vec());
        let datum = (0x0012, ASCII, 6, b"WGS-84".to_vec());
        let must_not_fire: Vec<(&str, Vec<GpsEntry>)> = vec![
            (
                "version only (the P1100 shape)",
                vec![(0x0000, BYTE, 4, vec![2, 3, 0, 0])],
            ),
            ("latitude ref only", vec![lat_ref()]),
            (
                "altitude + time + date + datum",
                vec![alt_ref, alt, time, date, datum],
            ),
        ];
        for (name, entries) in &must_not_fire {
            let t = LintTarget::from_bytes("p.jpg", gps_jpeg_with(entries));
            assert!(
                GpsMetadataLeak.check(&t).is_none(),
                "{name}: no location, so no finding"
            );
        }
    }

    /// FT-3: one shape, `crustyimg <fragment> <file> -o <file> -y`.
    #[test]
    fn fix_command_shape_is_fragment_file_dash_o_file_dash_y() {
        let f = Finding::new(
            "img/leak.jpg",
            "privacy/gps-metadata-leak",
            Severity::Error,
            "m",
            Some("meta clean --gps".to_string()),
        );
        let want = "crustyimg meta clean --gps img/leak.jpg -o img/leak.jpg -y";
        assert_eq!(f.fix_command_for(Shell::Posix).as_deref(), Some(want));
        assert_eq!(f.fix_command_for(Shell::PowerShell).as_deref(), Some(want));

        let none = Finding::new("img/a.jpg", "x/y", Severity::Info, "m", None);
        assert_eq!(none.fix_command_for(Shell::Posix), None);
        assert_eq!(none.fix_command_for(Shell::PowerShell), None);
    }

    /// FT-4: exact quoting for each shell, on every OS. The `'` and `’` rows are
    /// where naive and correct quoting differ.
    #[test]
    fn shell_arg_quotes_for_each_shell() {
        let cases: &[(&str, &str, &str)] = &[
            // (path, POSIX, PowerShell)
            ("a.jpg", "a.jpg", "a.jpg"),
            ("img/a.jpg", "img/a.jpg", "img/a.jpg"),
            (r"img\a.jpg", r"'img\a.jpg'", r"img\a.jpg"),
            (r"C:\x\a.jpg", r"'C:\x\a.jpg'", r"C:\x\a.jpg"),
            ("my photo.jpg", "'my photo.jpg'", "'my photo.jpg'"),
            ("it's.jpg", r"'it'\''s.jpg'", "'it''s.jpg'"),
            (
                "it\u{2019}s.jpg",
                "'it\u{2019}s.jpg'",
                "'it\u{2019}\u{2019}s.jpg'",
            ),
            ("a,b.jpg", "'a,b.jpg'", "'a,b.jpg'"),
            ("$x.jpg", "'$x.jpg'", "'$x.jpg'"),
            ("-x.jpg", "./-x.jpg", "./-x.jpg"),
        ];
        for (path, posix, ps) in cases {
            assert_eq!(
                shell_arg(Path::new(path), Shell::Posix).as_deref(),
                Some(*posix),
                "POSIX quoting of {path:?}"
            );
            assert_eq!(
                shell_arg(Path::new(path), Shell::PowerShell).as_deref(),
                Some(*ps),
                "PowerShell quoting of {path:?}"
            );
        }
    }

    /// FT-5: no command for stdin or a path that is not UTF-8; a real file named
    /// `stdin` still gets its fix.
    #[test]
    fn fix_command_none_for_stdin_and_non_utf8() {
        let mk = |p: &str| {
            Finding::new(
                p,
                "privacy/gps-metadata-leak",
                Severity::Error,
                "m",
                Some("meta clean --gps".to_string()),
            )
        };
        assert_eq!(mk("stdin").on_stdin().fix_command_for(Shell::Posix), None);
        assert_eq!(
            mk("stdin").fix_command_for(Shell::Posix).as_deref(),
            Some("crustyimg meta clean --gps stdin -o stdin -y")
        );
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let path = PathBuf::from(std::ffi::OsStr::from_bytes(b"\xff.jpg"));
            let f = Finding::new(
                path,
                "privacy/gps-metadata-leak",
                Severity::Error,
                "m",
                Some("meta clean --gps".to_string()),
            );
            assert_eq!(f.fix_command_for(Shell::Posix), None);
        }
    }

    #[test]
    fn corrupt_decode_rule_yields_a_finding_without_panicking() {
        // Bytes that look like a PNG but are truncated garbage → decode Err.
        let bad = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01];
        let target = LintTarget::from_bytes("broken.png", bad);
        assert!(target.decoded().is_err(), "decode should fail");
        let f = TruncatedOrCorrupt
            .check(&target)
            .expect("corrupt finding expected");
        assert_eq!(f.rule(), "size/truncated-or-corrupt");
        assert_eq!(f.severity(), Severity::Error);

        // A valid image does NOT fire the rule.
        let good = LintTarget::from_bytes("ok.jpg", clean_jpeg());
        assert!(TruncatedOrCorrupt.check(&good).is_none());
    }

    /// A `.heic` in the DEFAULT build fails to decode with `CodecNotBuilt`, but the
    /// file is perfectly valid — the rule must NOT call it corrupt (SPEC-062). A
    /// directory of iPhone photos would otherwise fail `lint` with exit 7 and tell
    /// the user to "re-export a valid image".
    #[cfg(not(feature = "heic"))]
    #[test]
    fn codec_not_built_is_not_reported_as_corrupt() {
        let heic = include_bytes!("../../tests/fixtures/heic/solid_64x48.heic").to_vec();
        let target = LintTarget::from_bytes("photo.heic", heic);
        assert!(
            matches!(target.decoded(), Err(ImageError::CodecNotBuilt { .. })),
            "fixture should surface CodecNotBuilt in the default build"
        );
        assert!(
            TruncatedOrCorrupt.check(&target).is_none(),
            "a missing codec must not be reported as a corrupt file"
        );
    }

    /// An image whose DECLARED dimensions exceed the decode pixel budget
    /// (SPEC-070/DEC-063) fails to decode with `LimitsExceeded` — but, like a
    /// missing codec, that says nothing about the file's integrity. A 100 MP
    /// medium-format frame is not "truncated or corrupt", and "re-export a valid
    /// image" is a remedy for a problem it does not have (SPEC-071 fix 1).
    ///
    /// The fixture is the F-RAW-1 bomb shape: a JPEG whose SOF0 declares
    /// 16384×9776 (160 Mpix) while the entropy data is only an 8×8 image's, so
    /// the rejection provably comes from the pre-decode header peek.
    #[test]
    fn lint_does_not_call_a_too_large_image_corrupt() {
        let mut jpg = clean_jpeg();
        let sof = jpg
            .windows(2)
            .position(|m| m == [0xFF, 0xC0])
            .expect("encoded JPEG carries an SOF0 marker");
        // SOF0: `FF C0 [len:2] [precision:1] [height:2] [width:2] …`
        jpg[sof + 5..sof + 7].copy_from_slice(&9776u16.to_be_bytes());
        jpg[sof + 7..sof + 9].copy_from_slice(&16384u16.to_be_bytes());

        let target = LintTarget::from_bytes("medium-format.jpg", jpg);
        assert!(
            matches!(target.decoded(), Err(ImageError::LimitsExceeded(_))),
            "fixture should surface LimitsExceeded, got {:?}",
            target.decoded()
        );
        assert!(
            TruncatedOrCorrupt.check(&target).is_none(),
            "an image outside the decode budget must not be reported as corrupt"
        );

        // The catch-all is intact: a genuinely corrupt file still fires the rule.
        let bad = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01];
        assert!(
            TruncatedOrCorrupt
                .check(&LintTarget::from_bytes("broken.png", bad))
                .is_some(),
            "a truly corrupt image must still fire the rule"
        );
    }

    #[test]
    fn run_lint_keeps_going_after_a_corrupt_file() {
        // A corrupt file and a clean file: the run yields one finding, scans both.
        let inputs = vec![
            Input::Stdin {
                bytes: vec![0xFF, 0xD8, 0x00],
                stem: "broken".into(),
            },
            Input::Stdin {
                bytes: clean_jpeg(),
                stem: "clean".into(),
            },
        ];
        let outcome = run_lint(&inputs, &default_rules(), &LintConfig::default());
        assert_eq!(outcome.files_scanned, 2);
        assert_eq!(outcome.error_count(), 1);
        assert_eq!(outcome.findings[0].rule(), "size/truncated-or-corrupt");
    }
}
