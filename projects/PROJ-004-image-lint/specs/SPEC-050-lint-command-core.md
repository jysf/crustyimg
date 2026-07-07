---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-050
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L

project:
  id: PROJ-004
  stage: STAGE-013
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-06

references:
  decisions: [DEC-050, DEC-025, DEC-007]
  constraints: [untrusted-input-hardening, no-new-top-level-deps-without-decision, ergonomic-defaults, test-before-implementation]
  related_specs: [SPEC-023, SPEC-026]

value_link: >
  Delivers STAGE-013's foundation — the `lint` command + the Rule/Finding/Severity
  framework + exit-7, proven end-to-end by two high-value rules. The scaffold every
  other lint rule plugs into.

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-050: the `lint` command core — framework + two foundational rules

## Context

PROJ-004 turns crustyimg into a *checking* tool: `crustyimg lint [PATHS]…` walks an image asset
tree and reports problems with a runnable fix and a CI-native exit code. This spec builds the
**framework** — the command, the source-resolution fan-out, the `Rule`/`Finding`/`Severity` model,
the human report, and the exit-code mapping — and proves it end-to-end with **two foundational
rules**: `privacy/gps-metadata-leak` (the privacy moat) and `size/truncated-or-corrupt` (decode
failure as a finding, not an abort). Config (SPEC-051), the JSON report (SPEC-052), and the rest of
the shipped-capability rules (SPEC-053) layer onto this.

The command shape, rule-id catalog, severity model, exit-7 reuse, and config/savings conventions
are fixed in **DEC-050**.

## Goal

Add a read-only `crustyimg lint [PATHS]…` command: resolve inputs via the shipped `source::resolve`
(glob/dir/file, non-images skipped), run a registry of rules per file, print findings grouped by
file with a runnable fix each, and map the outcome to exit codes reusing `CliError::CheckFailed`
(exit 7, DEC-025) — with two rules registered to prove the framework, adding no dependency.

## Inputs

- **Files to read:**
  - `docs/research/proj-002-design-lint.md` — the rule catalog, the Lighthouse-parity map, the
    config/exit/severity design.
  - `src/source/mod.rs` — `resolve(arg, reader) -> Vec<Input>` (`:138`) and `Input` (`:37`); the
    glob/dir/file fan-out `lint` reuses. Non-image files must be skipped (a lint tree is mixed).
  - `src/cli/mod.rs` — `CliError::CheckFailed` → exit 7 (`:469`, `:519`, DEC-025); the subcommand
    dispatch + arg-struct pattern; `escape_json` (`:1130`) for later; `run_info`/`InfoReport`
    (`:1055`) — the shape rules read.
  - `src/image/mod.rs` — `Image::load`/`from_bytes` (decode → the `truncated-or-corrupt` signal),
    `ImageInfo` (`has_exif`), the `MetadataBundle` (raw EXIF bytes captured at load).
  - `src/metadata/` — how EXIF is read today (the GPS-presence check reuses the read side, not a
    new parser).
- **Related code paths:** `src/cli/mod.rs:428` — the `CheckFailed` comment already names "the
  future EXIF audit-linter"; this is it.

## Outputs

- **Files created:** `src/lint/mod.rs` — `Severity`, `Finding`, the `Rule` trait + a rule registry,
  the lint runner, the exit-code summarizer, and the human report; the two foundational rules.
- **Files modified:** `src/lib.rs` (register `pub mod lint;`); `src/cli/mod.rs` (add the `Lint`
  subcommand + dispatch → the runner; map its result to exit codes).
- **New exports:**
  - `pub enum Severity { Error, Warn, Info }`.
  - `pub struct Finding { file: PathBuf, rule: &'static str, severity: Severity, message: String,
    fix: Option<String> }` (accessors; `rule` is a stable id).
  - `pub trait Rule { fn id(&self) -> &'static str; fn default_severity(&self) -> Severity;
    fn check(&self, target: &LintTarget) -> Option<Finding>; }` (+ a `LintTarget` carrying the
    path, raw bytes, the decode `Result`, and lazily-derived `ImageInfo`/EXIF).
  - `pub fn run_lint(paths, options) -> LintOutcome` (findings + counts) and a
    `pub fn exit_code(&LintOutcome, max_warnings) -> i32`-style summarizer.
  - The two rules: `privacy/gps-metadata-leak` (Error, fix `clean --gps`), `size/truncated-or-corrupt`
    (Error, fix "(re-export a valid image)").
- **Database changes:** none.

## Acceptance Criteria

- [ ] `crustyimg lint <dir>` resolves inputs via `source::resolve`, **skips non-image files**
  (a `.txt`/`.md` in the tree is not a finding), runs the registered rules per image, and prints
  findings **grouped by file**, each with its rule id, severity, message, and a runnable
  `crustyimg` fix.
- [ ] Exit-code mapping (reusing `CheckFailed`, DEC-025): `0` clean · `7` ≥1 `Error` · `2`
  usage/bad args · `3` no inputs resolved. `Info`-severity findings NEVER change the exit code
  (`Warn`/`--max-warnings` is wired in SPEC-051; here `Warn` alone does not fail).
- [ ] `privacy/gps-metadata-leak` fires **only** when the image carries GPS EXIF (reusing the
  shipped EXIF read, not a new parser), with fix `clean --gps`; a photo with no GPS is clean.
- [ ] `size/truncated-or-corrupt` turns a **decode failure into a finding** (Error, exit 7) — the
  runner MUST NOT abort on a bad file (the one deliberate divergence, DEC-050); other files in the
  tree still lint.
- [ ] **Read-only:** `lint` never writes or modifies an image (assert by construction — no
  `Sink`/write path in `src/lint/`).
- [ ] Deterministic: same tree ⇒ same findings in the same order (sorted by path, then severity,
  then rule id); no network/mtime/wall-clock in a finding.
- [ ] No panic on any input (a 0-byte file, a `.png` that's actually text, a huge file — the decode
  cap applies); `just deny` green (**no new dependency**); every existing test stays green.

## Failing Tests

Written during **design**, BEFORE build. Fixtures are generated in-test (reuse the `tests/common`
helpers: `solid_png`, `jpeg_with_exif`, and a GPS-tagged JPEG helper — add one if absent, mirroring
`jpeg_with_orientation`).

- **`src/lint/mod.rs` (unit tests)**
  - `"severity → exit code: any Error ⇒ 7, only Info/Warn ⇒ 0 (no max-warnings)"` — a synthetic
    finding set through the summarizer.
  - `"findings sort deterministically by (path, severity, rule)"`.
  - `"gps rule fires on GPS EXIF, clean on none"` — construct an `Image`/target with and without a
    GPS block; assert the finding + its `clean --gps` fix, and no finding when absent.
  - `"corrupt-decode rule yields a finding, and the runner keeps going"` — a target whose decode is
    `Err`; assert a `truncated-or-corrupt` Error finding (not a panic/abort).
- **`tests/lint.rs` (integration — real CLI)**
  - `"lint on a clean dir exits 0 with no findings"`.
  - `"lint on a dir with a GPS-tagged jpeg exits 7 and prints the finding + fix"`.
  - `"lint on a dir with a truncated file exits 7; a sibling clean file is still linted"`.
  - `"non-image files in the tree are skipped (no finding, exit 0)"`.
  - `"lint with no resolvable inputs exits 3"`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-050` (this project) — the command shape, stable rule ids, the 3-severity model, exit-7
  reuse, read-only, decode-failure-is-a-finding.
- `DEC-025` — the exit-7 `CheckFailed` this reuses (its comment names the audit-linter).
- `DEC-007` — typed errors; `lint` maps usage/no-input to exit 2/3 and gate-failure to 7.

### Constraints that apply
- `untrusted-input-hardening` — a lint tree is untrusted: a corrupt/oversized/mislabeled file is a
  *finding* or a skip, never a panic or abort. The decode cap (DEC-034) already bounds loads.
- `no-new-top-level-deps-without-decision` — the GPS check reuses the shipped EXIF read; output is
  hand-rolled; no new crate.
- `ergonomic-defaults` — zero-config: `lint <dir>` runs the default rule set at default severities.
- `test-before-implementation` — the Failing Tests above are the contract.

### Prior related work
- `SPEC-023` (shipped) — `diff --fail-under` established the exit-7 `CheckFailed` gate `lint`
  reuses. `SPEC-026` (shipped) — the metadata lane whose `clean --gps` is the privacy fix.

### Out of scope (for this spec specifically)
- Config discovery / `select`/`ignore` / per-rule severity / `--max-warnings` — SPEC-051.
- The JSON report — SPEC-052 (human only here).
- All other rules — SPEC-053 (shipped-capability) and STAGE-014 (engine-backed).

## Notes for the Implementer

- Keep `src/lint/` free of any write/`Sink` path — read-only by construction is an acceptance
  criterion. The command reads; the *fix* is a string the user runs.
- The `Rule` trait should make a rule a small, pure `fn(&LintTarget) -> Option<Finding>` so the
  registry is trivial to extend in SPEC-053 and STAGE-014. `LintTarget` should expose the decode
  `Result` (so `truncated-or-corrupt` sees the error) and lazily-derived info/EXIF (so cheap rules
  don't force work they don't need).
- Sort findings deterministically before printing; the golden CI output depends on it.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-050-lint-command-core`
- **PR (if applicable):** (opened after green local gates)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — DEC-050 already fixed the contract; the build followed it verbatim.
- **Deviations from spec:**
  - `size/truncated-or-corrupt` stores `fix: None` (the "re-export a valid image"
    guidance lives in the finding *message*) rather than a `fix` string. Rationale:
    `Finding.fix` is a *runnable `crustyimg` subcommand fragment* (rendered as
    `crustyimg <fix> <file>`); re-export is not a crustyimg command, so surfacing it
    as a fake command would be wrong. Every present `fix` is therefore genuinely
    runnable — which is the spec's intent. Findings still carry the guidance.
  - `lint` with no PATHS defaults to the current directory (an ergonomic-defaults
    read of the `[PATHS]…` signature); an explicit unresolvable path still exits 3.
- **Follow-up work identified:**
  - SPEC-052 must resolve how `lint --format human|json` coexists with the global
    `--format` flag (encode-format). Options: reuse the global flag's string, or a
    lint-local `--report`/renamed flag. Flagged so SPEC-052 doesn't hit a clap
    duplicate-arg conflict.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Only the `fix` field's exact contract for a non-command finding
   (`truncated-or-corrupt`). Resolved by making `fix` mean "a runnable fragment" and
   pushing re-export guidance into the message (see deviations).
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `kamadak-exif`'s `Tag::context() == Context::Gps` gave a clean, parser-reuse
   GPS check exactly as the spec anticipated (no new parser).
3. **If you did this task again, what would you do differently?**
   — Nothing structural. The `LintTarget` (raw bytes + decode `Result` + lazy
   info/GPS via `OnceCell`) made both rules trivial and leaves SPEC-053's rules a
   clean seam.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
