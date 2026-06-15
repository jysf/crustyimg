---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-008
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6, fresh session
  created_at: 2026-06-14

references:
  decisions: [DEC-011, DEC-012, DEC-007]
  constraints:
    - ergonomic-defaults
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-005, SPEC-007]

# One sentence on what this spec contributes to its stage's
# value_contribution.
value_link: "Delivers STAGE-002's `view` capability: the first real command that renders an image through the display Sink, proving the source → load → sink path end to end."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 40
      recorded_at: 2026-06-14
      notes: "design cycle, Opus subagent"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-008: view command terminal display

## Context

This spec makes the `view` subcommand **real**. Today `view` is a stub:
`src/cli/mod.rs` declares `Commands::View { input, width, height }`, and
`dispatch()` routes it to `Err(CliError::NotImplemented("view"))` (exit 1).
SPEC-007 (shipped, completed STAGE-001) built the clap surface, the
`CliError` enum + exit-code mapping, and the real `apply` path that this
spec mirrors. SPEC-005 (shipped) built the `Sink`, including the
`Sink::Display` variant whose `write()` arm does the non-tty refusal
(`SinkError::NotATty`) first and the viuer render behind
`#[cfg(feature = "display")]`.

This is the FIRST of STAGE-002's two backlog items (the other is `info`).
STAGE-002 ("view and info") turns the STAGE-001 skeleton into commands a
user actually runs — the first real, read-only path through
source → load → sink, without mutating pixels. `view` is the lowest-risk
such command: no encode, no pixel mutation, no metadata writes.

DEC-011 governs the display Sink and its off-by-default `display` cargo
feature: the `Sink::Display` variant and its `NotATty` refusal compile
**unconditionally** (so the default build type-checks the path and the
refusal test always runs), while the viuer render call is feature-gated.
DEC-011 explicitly names this as the `view` command that consumes the sink
and needs `--features display` to actually render. DEC-012 governs the clap
CLI; DEC-007 governs the typed-error → exit-code mapping.

## Goal

Make `crustyimg view <INPUT>` display an image in the terminal via the
viuer-backed display Sink — fit-to-terminal by default, with optional
`--width`/`--height` sizing — and refuse on a non-tty with a clear,
terminal-required error (exit 5). Replace the `NotImplemented("view")` stub
with a real `run_view` handler; no other command changes.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — the clap surface, `CliError` + `code()`, `run_apply`
    (the structural template), `build_sink`/`resolve_format` helpers,
    and the `exit_code_mapping_is_total` unit test.
  - `src/sink/mod.rs` — `Sink::Display` (variant + `write()` arm), the
    `SinkError::{NotATty, Display}` variants, `SinkInput`, `Overwrite`.
  - `src/source/mod.rs` — `source::resolve`, `Input::{Path, Stdin}`,
    `Input::stem()`, `Input::path()`, `SourceError`.
  - `src/image/mod.rs` — `Image::load`, `Image::from_bytes`, `img.pixels()`.
  - `tests/cli.rs` + `tests/common/mod.rs` — integration-test conventions.
  - `tests/sink.rs` — `display_sink_refuses_non_tty` (a `Sink::Display`
    construction site that MUST be updated when the variant gains fields).
- **External APIs:** `viuer` 0.11.0 (`viuer::Config { width, height, .. }`,
  `viuer::print`) — already a `[dependencies]` entry behind `display`
  (DEC-011). NO new dependency.
- **Related code paths:** `src/cli/`, `src/sink/`.

## Outputs

- **Files created:** none required. (Integration tests for `view` go in
  the existing `tests/cli.rs` — see Failing Tests. Do NOT create a new
  test file.)
- **Files modified:**
  - `src/sink/mod.rs` — add fields to the `Sink::Display` variant:
    `Display { width: Option<u32>, height: Option<u32> }`. In the `write()`
    `Sink::Display { width, height }` arm, after the unchanged non-tty check,
    thread `width`/`height` into the viuer config:
    `viuer::Config { width: *width, height: *height, use_kitty: true,
    use_iterm: true, ..Default::default() }`. The non-tty refusal and the
    `#[cfg(not(feature = "display"))]` arm are otherwise unchanged.
  - `src/cli/mod.rs` — replace `Commands::View { .. } => Err(NotImplemented("view"))`
    in `dispatch()` with a call to a new `run_view(input, width, height, global)`;
    add the `run_view` function (private — covered by integration tests, see
    Notes). No change to `CliError` or `code()`.
  - `tests/sink.rs` — update the `display_sink_refuses_non_tty` construction
    `Sink::Display` → `Sink::Display { width: None, height: None }` so it
    still compiles and still asserts `NotATty`.
  - `docs/api-contract.md` — (done in DESIGN, not build) one-line note on
    the `view` entry recording the non-tty exit code = 5.
- **New / changed public signatures:**
  - `Sink::Display { width: Option<u32>, height: Option<u32> }` (changed
    public enum variant in `crate::sink`).
  - `run_view` is **private** to `src/cli/mod.rs` (not `pub`); it is covered
    by the binary-driven integration tests, mirroring how `run_apply` (also
    private) is covered. No new public function is added, so the
    `every-public-fn-tested` constraint is satisfied by the updated
    `display_sink_refuses_non_tty` test exercising the changed variant.
- **Database changes:** none.

## Acceptance Criteria

Each maps to a test in Failing Tests.

- [ ] `view <png>` with a piped (non-tty) stdout exits **5** and prints a
      terminal/tty-required message to stderr; no image bytes on stdout.
      → `view_non_tty_refuses_exit_5`
- [ ] `view <missing-file>` exits **3** (input not found).
      → `view_missing_input_exits_3`
- [ ] `view <dir>` (a directory input resolving to ≥1 image) does NOT panic
      and exits **5** under the test harness's non-tty stdout — i.e. it
      resolves the first image and reaches the display Sink's non-tty refusal
      (confirming the MVP "first resolved input" semantics, not a usage error).
      → `view_directory_uses_first_input`
- [ ] `view --width 80 <png>` parses and, under non-tty, still exits **5**
      (the width is threaded into the Sink but the tty check fires first).
      → `view_width_flag_still_refuses_non_tty`
- [ ] The existing `each_subcommand_help_parses` and
      `help_lists_all_subcommands` integration tests still pass (view already
      parses; this spec does not change its arg surface).
- [ ] `Sink::Display { width: None, height: None }` constructed under
      `cargo test` (piped stdout) returns `SinkError::NotATty`.
      → `display_sink_refuses_non_tty` (updated existing test)
- [ ] The `exit_code_mapping_is_total` unit test still passes unchanged
      (NotATty maps via `CliError::Sink(_) => 5`; no mapping edit).

## Failing Tests

Written during **design**, made to pass during **build**. Mirror
`tests/cli.rs` conventions: drive the real binary via
`env!("CARGO_BIN_EXE_crustyimg")` + `std::process::Command`, native
in-memory PNG fixtures (reuse the `write_test_png` helper already in
`tests/cli.rs`), `tempfile::tempdir()`, trim stdout, assert exit codes via
`output.status.code()`. Under `cargo test` the child's stdout is a pipe
(non-tty), so `view` always reaches the `NotATty` refusal — that is the
testable headline behavior and it runs in DEFAULT CI (no `display` feature).

- **`tests/cli.rs`** (add these tests to the existing file)
  - `"view_non_tty_refuses_exit_5"` — `view <png>` (piped stdout) → asserts:
    `status.code() == Some(5)`; stderr (lowercased) contains `"tty"` OR
    `"terminal"`; **stdout is empty** (no image bytes leaked). Use
    `write_test_png` for the fixture.
  - `"view_missing_input_exits_3"` — `view <path-that-does-not-exist>` →
    asserts: `status.code() == Some(3)`.
  - `"view_directory_uses_first_input"` — create a tempdir, write one PNG
    into it via `write_test_png`, run `view <dir>` → asserts:
    `status.code() == Some(5)` (resolved the first image, hit the non-tty
    refusal; did NOT panic, did NOT exit 2/usage). This pins the MVP
    "display the first resolved input" decision.
  - `"view_width_flag_still_refuses_non_tty"` — `view --width 80 <png>`
    (piped stdout) → asserts: `status.code() == Some(5)` and stderr mentions
    a terminal/tty requirement. Proves `--width` parses and is wired without
    changing the refusal behavior on a non-tty.

- **`tests/sink.rs`** (UPDATE the existing test, do not add a new one)
  - `"display_sink_refuses_non_tty"` — change the construction to
    `Sink::Display { width: None, height: None }`; the assertion
    (`matches!(err, SinkError::NotATty)`) is unchanged. This both keeps the
    suite compiling after the variant gains fields AND covers the changed
    public surface (`every-public-fn-tested`).

- **`src/cli/mod.rs` `#[cfg(test)] mod tests`** (no new unit test required;
  confirm the EXISTING `exit_code_mapping_is_total` still passes verbatim —
  the NotATty → 5 mapping is via `CliError::Sink(_) => 5` and is NOT edited).

## Implementation Context

*Read this section (and the files it points to) before starting the build
cycle. It is the equivalent of a handoff document, folded into the spec.*

### Decisions that apply

- `DEC-011` — viuer-backed display Sink behind the off-by-default `display`
  feature; `Sink::Display` + `NotATty` refusal compile unconditionally, the
  viuer `print` call is `#[cfg(feature = "display")]`. DEC-011 names `view`
  as the consumer that needs `--features display` to actually render. The
  build MUST keep BOTH the default build and the `--features display` build
  green (DEC-011 wants the feature build from bit-rotting).
- `DEC-012` — clap (derive) is the CLI framework; `view`'s arg surface
  (`input: String`, `--width Option<u32>`, `--height Option<u32>`) is
  already declared and must not change. The pixel core stays clap-free; all
  clap stays in `src/cli/`.
- `DEC-007` — typed `thiserror` errors in the library; the binary boundary
  (`src/cli/`) maps them to exit codes. `SinkError::NotATty` already maps to
  exit 5 via `CliError::Sink(_) => 5`. NO new error variant, NO mapping
  change.

### Constraints that apply

- `ergonomic-defaults` — `view <input>` is one short command; `--width`/
  `--height` are optional; both `None` = fit-to-terminal (viuer default).
- `no-unwrap-on-recoverable-paths` — `run_view` must use `?` with the typed
  errors; NO `unwrap`/`expect`/`panic!` on any recoverable path. The empty
  `source::resolve` result must surface as a typed `CliError`, not a panic
  (mirror `run_apply`'s `.ok_or(CliError::Source(SourceError::NotFound(..)))`).
- `every-public-fn-tested` — the only changed public surface is the
  `Sink::Display` variant; the updated `display_sink_refuses_non_tty`
  covers it. `run_view` is private (integration-covered), like `run_apply`.
- `clippy-fmt-clean` — must pass `cargo clippy -- -D warnings`,
  `cargo fmt --check`, AND `cargo clippy --features display -- -D warnings`.
- `test-before-implementation` — these Failing Tests are written first, then
  made to pass.
- `untrusted-input-hardening` — display is read-only and does not write
  files, so no new traversal/overwrite surface is introduced. Image decode
  hardening (image::Limits) already lives in `Image::load` (out of scope
  here; do not add it in `view`).

### Prior related work

- `SPEC-007` (shipped, PR #7) — built the clap surface, `CliError` +
  exit-code mapping, and `run_apply` (the structural template for
  `run_view`). Completed STAGE-001.
- `SPEC-005` (shipped) — built the `Sink`, including `Sink::Display` (no
  fields yet) with the tty-check-first `write()` arm and the
  `display_sink_refuses_non_tty` test this spec updates.

### Out of scope (for this spec specifically)

If any of these feel necessary during build, create a new spec rather than
expanding this one.

- The `info` command (the other STAGE-002 backlog item — its own spec).
- Actually asserting rendered terminal output / pixel fidelity (CI has no
  tty; the render path is feature-gated and cannot be integration-tested
  here — do NOT attempt to fake a tty).
- `view -` (stdin): allowed to parse (it is just `input == "-"`), but its
  rendering is non-deterministic and out of scope to assert. `source::resolve`
  already handles `-`; if a build test exercises it at all, it would still
  hit the non-tty refusal (exit 5). Do not add stdin-specific logic.
- Multi-input fan-out / displaying more than one image, batch, `--out-dir`.
  `view` is single-image; resolve the FIRST input only (MVP decision below).
- Any new error variant or exit-code change; any new dependency; any DEC.

## Notes for the Implementer

- **`run_view` mirrors `run_apply`, minus recipe/pipeline.** Resolve the
  single input, load the image, build `Sink::Display { width, height }`,
  call `sink.write(&img, &sink_input, Overwrite::Forbid,
  &mut std::io::stdout().lock())`. Shape:

  ```text
  fn run_view(input: &str, width: Option<u32>, height: Option<u32>,
              _global: &GlobalArgs) -> Result<(), CliError> {
      let resolved = source::resolve(input, &mut std::io::stdin().lock())?;
      let first = resolved.into_iter().next()
          .ok_or(CliError::Source(SourceError::NotFound(input.to_owned())))?;
      let img = match &first {
          crate::source::Input::Path(p) => Image::load(p)?,
          crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
      };
      let sink = Sink::Display { width, height };
      let sink_input = SinkInput { stem: first.stem(), path: first.path() };
      sink.write(&img, &sink_input, Overwrite::Forbid,
                 &mut std::io::stdout().lock())?;
      Ok(())
  }
  ```

  `_global` is currently unused by `view` (no `-o`, no `--format` for a
  display sink); take it for signature symmetry with `run_apply` and prefix
  with `_` to keep clippy quiet, OR omit it entirely — either is fine. Keep
  it simple.

- **Dispatch wiring:** replace
  `Commands::View { .. } => Err(CliError::NotImplemented("view"))` with
  `Commands::View { input, width, height } => run_view(input, *width, *height, &cli.global)`.

- **MULTI-INPUT DECISION (#3):** `view` takes a single positional `input`,
  but `source::resolve` can yield many (a directory or glob). The MVP
  behavior is to **display the FIRST resolved input** (`.into_iter().next()`),
  exactly as `run_apply` does. Do NOT refuse on multi-resolve and do NOT
  loop over all of them. This is consistent, simple, and documented in the
  `view_directory_uses_first_input` test.

- **EXIT-CODE DECISION (#4):** the non-tty refusal stays
  `SinkError::NotATty → CliError::Sink(_) → exit 5` (api-contract code 5 =
  "output write failed / refused"; the contract already says "`view` to a
  non-tty refuses"). Do NOT add a distinct code, do NOT touch `code()`, and
  do NOT touch `exit_code_mapping_is_total`. DESIGN added a one-line note to
  `docs/api-contract.md`'s `view` entry recording exit 5.

- **The tty check runs FIRST, before any feature gate.** In default CI
  (no `display` feature) the viuer render path is compiled out, but the
  non-tty check still fires and returns `NotATty` — so the integration tests
  above run and pass in default CI. You CANNOT integration-test real
  rendering; don't try to.

- **Keep BOTH builds green.** The four standard gates PLUS
  `cargo build --features display` and
  `cargo clippy --features display -- -D warnings`. When the feature is on,
  the `Sink::Display { width, height }` arm's viuer block is the live branch —
  make sure `width`/`height` are correctly referenced there (`width: *width,
  height: *height` since the match binds `&Option<u32>`).

- **Reuse `write_test_png`** already defined in `tests/cli.rs`; do not
  duplicate a fixture helper.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
