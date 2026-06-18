---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-023
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-009
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build cycle (or orchestrator-direct main loop)
  created_at: 2026-06-17

references:
  decisions: [DEC-025, DEC-019, DEC-007, DEC-002]
  constraints:
    - ergonomic-defaults
    - clippy-fmt-clean
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
  related_specs: [SPEC-016, SPEC-022]

value_link: "Delivers STAGE-009's verification surface — a perceptual SSIMULACRA2 comparison with a CI visual-regression gate, reusing the metric the auto-quality engine optimizes against."

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-023: diff — perceptual comparison + CI gate

## Context

STAGE-008 introduced the SSIMULACRA2 perceptual metric (`crate::quality::score`)
as the *engine* behind auto-quality. SPEC-022 (`optimize`) made the engine a
one-button command. This spec makes the metric **directly usable as proof**: a
`diff` command that scores how perceptually similar two images are and, with
`--fail-under <N>`, acts as a **CI visual-regression gate** (exit non-zero when the
score drops below the threshold).

This is the verification half of STAGE-009 (see the stage `## Spec Backlog`) and
the natural sibling of the future EXIF audit-linter: both are "compute a number,
gate CI on it" tools. It is almost pure reuse — `crate::quality::score` already
exists and is tested; this spec adds a thin command + a distinguishable gate exit
code.

The **visual** half of "perceptual + visual" (a highlighted pixel-diff heatmap
image) is **deferred to a follow-up spec** (see Out of scope + DEC-025): it is the
only genuinely new pixel code, it carries open design questions (colormap,
amplification), and the score + CI gate is the high-value, self-contained core.
`diff` v1 ships the number and the gate.

## Goal

Add `crustyimg diff <a> <b>` that prints the SSIMULACRA2 score of `b` relative to
`a` and, with `--fail-under <N>`, exits with a dedicated **check-failed** code
(exit 7) when the score is below `N` — usable as a one-line CI visual-regression
gate. `--json` emits a machine-readable result.

## Inputs

- **Files to read:** `src/quality/mod.rs` — `score(reference, candidate)` (the
  whole comparison; same-dimensions required). `src/cli/mod.rs` — the `Commands`
  enum, `dispatch`, `CliError` + `code()` + the `exit_code_mapping_is_total` test,
  `run_info`/`write_json`/`escape_json` (the hand-rolled-JSON precedent to mirror),
  `QualityError` wiring (`CliError::Quality`).
- **Related code paths:** `src/image/mod.rs` (`Image::load`, `pixels()`,
  `width()`/`height()`).
- **Decisions:** DEC-025 (this spec's command + exit-code DEC — author during
  build), DEC-019 (the metric), DEC-007 (typed errors / exit-code mapping).
- **Tests to mirror:** `tests/cli.rs` (`Command::new(BIN)`, `write_bytes`,
  `stdout_str`/`stderr_str`, exit-code asserts) and `src/quality` tests
  (`score_degraded_is_lower`: a q≈5–8 JPEG round-trip of a detailed image scores
  well below 90). Fixtures: `common::detailed_png`, `common::detailed_jpeg`.

## Outputs

- **Files modified:**
  - `src/cli/mod.rs` — add `Commands::Diff { a, b, fail_under, json }`, a `dispatch`
    arm, `run_diff`, the `diff_passes` helper + a `write_diff_json` helper, a new
    `CliError::CheckFailed` variant mapped to **exit 7**, and unit tests (incl.
    extending `exit_code_mapping_is_total`).
  - `decisions/DEC-025-diff-command-and-check-exit-code.md` — the new decision.
  - `docs/api-contract.md` — add exit code **7** to the table + a `diff` entry.
  - `tests/cli.rs` — `diff_*` integration tests + add `diff` to the two subcommand
    lists.
- **New exports:** none outside `src/cli`.
- **No new dependency. No database changes.**

## Command surface (PINNED)

```
crustyimg diff <a> <b>
    [--fail-under <N>]    # 0..=100; exit 7 if score(b vs a) < N
    [--json]              # machine-readable result on stdout
```

- `a`, `b` are two **file paths** (positional, both required); each is loaded via
  `Image::load`. Glob/stdin/multi-input are NOT supported (out of scope).
- The score is `crate::quality::score(a.pixels(), b.pixels())` — higher = more
  similar, ~100 = visually identical. `b` is the *candidate*, `a` the *reference*.
- **Different dimensions → usage error (exit 2)** with a clear message; SSIMULACRA2
  requires equal dimensions and there is no defined comparison otherwise.
- `--fail-under <N>` out of `0..=100` → usage error (exit 2).

### Output

- **Human (default):** one line to **stdout**: `ssimulacra2: <score:.4>` (e.g.
  `ssimulacra2: 91.2345`). When the gate fails, a diagnostic goes to **stderr**
  (unless `--quiet`): `diff: ssimulacra2 <score:.4> is below --fail-under <N>`.
- **`--json`:** one line to stdout (hand-rolled, mirroring `write_json`, no
  serde_json runtime dep):
  `{"a":"<a>","b":"<b>","score":<score:.4>,"fail_under":<N|null>,"passed":<bool>}`
- **Exit:** `0` when no gate, or score ≥ `N`. **`7`** when score < `N`
  (`CliError::CheckFailed`). The score line is STILL printed to stdout before the
  non-zero exit, so CI can capture both the number and the verdict.

## Acceptance Criteria

- [ ] `diff a.png a.png` (identical) prints `ssimulacra2: <score>` with score ≈ 100
  (≥ 90) and exits 0.
- [ ] `diff a.png b.jpg` where `b` is a heavily-degraded (low-quality) copy of the
  same pixels prints a score **below** the identical score (and < 90) and exits 0.
- [ ] `--fail-under 90` on a below-90 pair exits **7**; on an at/above-90 pair exits
  **0**. The score line is printed to stdout in both cases.
- [ ] `--fail-under 150` (or any value outside 0..=100) exits **2**.
- [ ] Comparing two images of different dimensions exits **2** with a message naming
  both dimensions.
- [ ] `--json` emits a single-line object containing `score`, `fail_under`, and
  `passed`.
- [ ] A missing input path exits **3**.
- [ ] `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test`
  pass; `cargo deny check licenses` stays green (no new dep). The api-contract
  exit-code table lists code 7.

## Failing Tests

Written during **design**, BEFORE build.

### Unit tests — `src/cli/mod.rs` (`#[cfg(test)] mod tests`)

- **`diff_parses_args`** — `Cli::try_parse_from(["crustyimg","diff","a.png","b.png",
  "--fail-under","90"])` parses to `Commands::Diff` with `a == "a.png"`,
  `b == "b.png"`, `fail_under == Some(90.0)`, `json == false`.
- **`diff_passes_gate`** — `diff_passes(95.0, Some(90.0))` is `true`;
  `diff_passes(85.0, Some(90.0))` is `false`; `diff_passes(12.0, None)` is `true`
  (no gate ⇒ always passes).
- **`exit_code_mapping_is_total`** (extend the existing test) — add
  `assert_eq!(CliError::CheckFailed.code(), 7);`.

### Integration tests — `tests/cli.rs`

- **`diff_identical_scores_high`** — write `detailed_png(96,96)` to `a.png`; run
  `diff a.png a.png`; assert exit 0 and stdout starts with `ssimulacra2:` with a
  parsed score ≥ 90.0.
- **`diff_degraded_scores_lower`** — write `detailed_png(96,96)` to `a.png`; encode
  its decoded pixels to a quality-5 JPEG `b.jpg`; run `diff a.png b.jpg`; assert
  exit 0 and the parsed score is < 90.0 (and < the identical score).
- **`diff_fail_under_gate_fails`** — the same degraded pair with `--fail-under 90`
  exits **7**; stdout still contains the `ssimulacra2:` line.
- **`diff_fail_under_gate_passes`** — `diff a.png a.png --fail-under 90` exits **0**.
- **`diff_fail_under_out_of_range_exits_2`** — `diff a.png a.png --fail-under 150`
  exits **2**.
- **`diff_dimension_mismatch_exits_2`** — `diff` of `detailed_png(64,64)` vs
  `detailed_png(32,32)` exits **2**.
- **`diff_json_output`** — `diff a.png a.png --json` exits 0 and stdout contains
  `"score":` and `"passed":true`.
- **`diff_missing_input_exits_3`** — `diff missing.png a.png` exits **3**.

## Implementation Context

*Read this section before starting the build cycle.*

### Decisions that apply

- **DEC-025 (NEW — author it)** — the `diff` command shape + the new **exit code 7
  ("a check/gate was not satisfied")** + dimension-mismatch = exit 2 + the deferral
  of the visual-diff heatmap. Use `decisions/_template.md`; `affected_scope`:
  `src/cli/mod.rs`, `docs/api-contract.md`; confidence ~0.8.
- **DEC-019** — the SSIMULACRA2 metric reused unchanged (`crate::quality::score`).
- **DEC-007** — typed errors + the single exit-code mapping in `CliError::code()`;
  the `exit_code_mapping_is_total` test guards it.
- **DEC-002** — decode-once: load each image once; no re-encode here.

### Constraints that apply

- `ergonomic-defaults` — `diff a b` with no flags just prints the score; the gate is
  one flag.
- `clippy-fmt-clean` — `--all-targets`, warnings as errors; run `cargo fmt` then
  `git add -u` before the final commit (the CI fmt trap).
- `no-unwrap-on-recoverable-paths` — typed `CliError`; `unwrap` only in `#[cfg(test)]`.
- `every-public-fn-tested` — covered by the unit + integration tests above.

### Prior related work

- `SPEC-016` (shipped) — introduced `crate::quality::score` (DEC-019), the exact
  function `diff` calls.
- `SPEC-022` (shipped) — the prior STAGE-009 command; mirror its structure (clap
  variant + thin handler + helper + tests).
- `run_info`/`write_json`/`escape_json` in `src/cli/mod.rs` — the hand-rolled-JSON
  pattern to mirror for `--json` (no serde_json runtime dep).

### Out of scope (for this spec specifically)

- **The visual-diff heatmap image** (a highlighted pixel-diff written to a file).
  Deferred to a follow-up spec — it is new pixel code with open design questions
  (colormap, amplification, output format). DEC-025 records the deferral. `diff` v1
  is score + gate + json only.
- **Glob / directory / stdin inputs and multi-pair batch.** `diff` takes exactly two
  file paths in v1.
- **Auto-resizing mismatched images to compare them.** Different dimensions is a
  usage error (exit 2), not an implicit resize.
- **Other metrics** (PSNR/SSIM/butteraugli). SSIMULACRA2 only.

## Notes for the Implementer

- **`run_diff` shape:**
  ```rust
  fn run_diff(
      a: &str,
      b: &str,
      fail_under: Option<f64>,
      json: bool,
      global: &GlobalArgs,
  ) -> Result<(), CliError> {
      if let Some(t) = fail_under {
          if !(0.0..=100.0).contains(&t) {
              return Err(CliError::Usage(format!(
                  "--fail-under must be a score in 0..=100, got {t}"
              )));
          }
      }
      let img_a = Image::load(a)?;           // ImageError → exit 3/1/4
      let img_b = Image::load(b)?;
      if img_a.width() != img_b.width() || img_a.height() != img_b.height() {
          return Err(CliError::Usage(format!(
              "cannot compare images of different dimensions ({}x{} vs {}x{})",
              img_a.width(), img_a.height(), img_b.width(), img_b.height()
          )));
      }
      let score = quality::score(img_a.pixels(), img_b.pixels())?;  // QualityError → exit 1
      let passed = diff_passes(score, fail_under);

      let mut out = std::io::stdout().lock();
      if json {
          write_diff_json(&mut out, a, b, score, fail_under, passed)
              .map_err(crate::sink::SinkError::Io)?;
      } else {
          writeln!(out, "ssimulacra2: {score:.4}").map_err(crate::sink::SinkError::Io)?;
      }
      if !passed {
          if !global.quiet {
              eprintln!(
                  "diff: ssimulacra2 {score:.4} is below --fail-under {}",
                  fail_under.unwrap_or(0.0)
              );
          }
          return Err(CliError::CheckFailed);
      }
      Ok(())
  }
  ```
  (Add `use std::io::Write;` if not already imploned via the module's imports — the
  module already writes to `out` elsewhere; reuse whatever is in scope.)
- **`diff_passes`:** `fn diff_passes(score: f64, fail_under: Option<f64>) -> bool {
  fail_under.is_none_or(|t| score >= t) }` (or `map_or(true, …)` if the MSRV lacks
  `is_none_or`).
- **`write_diff_json`** — mirror `write_json`: hand-roll the object, escape `a`/`b`
  with the existing `escape_json`, format `score` as `{:.4}`, emit `fail_under` as
  the number `{:.4}` or the literal `null`, and `passed` as a bare bool.
- **`CliError::CheckFailed`** — a unit variant (no payload); `#[error("check not
  satisfied")]`; `code()` returns `7`. Extend `exit_code_mapping_is_total`.
- **clap variant:**
  ```rust
  /// Perceptual comparison: SSIMULACRA2 score of <b> vs <a> (STAGE-009, DEC-025).
  /// `--fail-under <N>` exits 7 when the score is below N — a CI visual-regression gate.
  Diff {
      a: String,
      b: String,
      #[arg(long, value_name = "N")]
      fail_under: Option<f64>,
      #[arg(long)]
      json: bool,
  },
  ```
  Add the `dispatch` arm: `run_diff(a, b, *fail_under, *json, &cli.global)`.
- **api-contract.md** — add a row `| 7 | A check/gate was not satisfied (e.g. \`diff
  --fail-under\`). |` to the Exit Codes table, and a short `diff` command entry near
  the other read commands.
- **Degraded-fixture test helper:** to make `b.jpg`, decode `detailed_png(96,96)`
  and re-encode at quality 5 via `image::codecs::jpeg::JpegEncoder::new_with_quality`
  (same pattern as SPEC-022's q100 baseline). Keep both images 96×96 (≥ the
  SSIMULACRA2 floor); the dimension-mismatch test never scores, so 64/32 are fine.
- **Confirm every named failing test exists** before claiming green.
- **Cost:** append a build session to `cost.sessions` (real `tokens_total`, or a
  labeled estimate if main-loop), per AGENTS.md §4.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-025` — diff command + exit code 7 (check/gate not satisfied) + heatmap deferral
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>
3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
