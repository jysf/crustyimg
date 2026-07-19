---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-097
  type: chore
  cycle: build
  blocked: false
  priority: medium
  complexity: M

project:
  id: PROJ-008
  stage: STAGE-031
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # model choice is the maintainer's call at build dispatch (see Notes)
  created_at: 2026-07-19

references:
  decisions: [DEC-007, DEC-012]
  constraints: []
  related_specs: [SPEC-063, SPEC-067, SPEC-088]

value_link: >
  Makes the CLI legible and safe to change post-launch — one 6.5k-line file becomes a submodule tree
  and a duplicated JSON escaper becomes one source of truth — with byte-identical behavior.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 1200000
      duration_minutes: null
      estimated_usd: 6.48
      note: >
        Main-loop build session (no metered subagent) — ORDER-OF-MAGNITUDE
        ESTIMATE, not a real usage-object reading. Derived from session scope
        (full 6,483-line read of cli/mod.rs across many chunks, 5 new files
        totaling ~5,260 lines written, ~15 cargo test/clippy/build cycles
        across native+lean+wasm, multiple golden-harness runs) at Sonnet
        list rate ($3/$15 per MTok, ~80/20 input/output, no cache discount).
        Ship cycle should replace with the real subagent_tokens/duration_ms
        if this ran as a dispatched subagent, or /cost if run interactively.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
    # Computed at ship (AGENTS.md §4) — not build's job. See cost.sessions
    # above for the build cycle's own (estimated) entry.
---

# SPEC-097: decompose cli/mod.rs into a submodule tree and dedup escape_json

## Context

`src/cli/mod.rs` is **6,483 lines / ~250 KB, 117 top-level items** — the one unambiguous structural
defect flagged by the pre-launch Rust audit (STAGE-031's origin) and by an independent repo review. One
file holds the clap surface, ~24 `run_*` handlers, build/lock-file I/O, JSON serializers, and a
hand-rolled `escape_json`. It accreted; it was not designed this way. As the codebase a Show-HN spike
will draw contributors into, it should be legible and safe to change.

Two concrete problems:
1. **The mega-module** — no seam, hard to review, hard to navigate.
2. **`escape_json` is DUPLICATED** — `src/cli/mod.rs:2323` **and** `src/lint/report.rs:124` are two
   independent hand-rolled JSON escapers that can silently diverge (one could be fixed for a control-char
   edge case and the other not). This spec dedups them into one shared helper.

This is a **pure mechanical refactor**: no behavior change, no new features, no signature changes to the
CLI surface. The **entire external contract is `crustyimg::cli::run()`** (`src/main.rs:8`) — the
integration tests drive the built binary (assert_cmd), not `crustyimg::cli::` type paths (grep of
`src/main.rs` + `tests/` finds **2** `cli::` references, both the `run()` call/doc in main.rs). So the
re-export burden is small; the risk lives entirely in **proving the moved code behaves identically.**

**Frame only — this spec is not to be executed until the maintainer reviews the approach + the
verification gate.** A 6k-line move is safe only if behavior is proven identical.

## Goal

Split `src/cli/mod.rs` into a `src/cli/` submodule tree (thin `mod.rs` front door + dispatch, plus
cohesive submodules), and replace the two hand-rolled `escape_json` implementations with one shared
helper — **with byte-identical CLI output/exit codes and a green native/lean/wasm matrix throughout.**

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — the file being split; inventory its real item-by-item dependencies before
    assigning each to a submodule.
  - `src/main.rs` — the sole external consumer (`crustyimg::cli::run()`); the re-export contract.
  - `src/lint/report.rs` (`escape_json` at line 124) — the second copy to dedup against `cli/mod.rs:2323`.
  - `src/lib.rs` — confirm how `cli` is declared/exported (`pub mod cli`) so re-exports preserve paths.
  - `tests/` — confirm nothing references `crustyimg::cli::` types directly (grep; expected: none beyond
    the binary-driven tests).
- **Related code paths:** `src/build/`, `src/lint/`, `src/quality/`, `src/sink/` — called by the handlers
  being moved (imports shift, call sites do not).

## Outputs

- **Files created (candidate tree — confirm against the file, these are starting boundaries not gospel):**
  - `src/cli/mod.rs` (slimmed) — **keeps** the thin public front door + dispatch: `Cli`, `GlobalArgs`,
    `Commands`, `MetaCommand`, the arg enums (`QualityTarget`/`ProfileArg`/`ExplainFmt`/`AutoQuality`),
    `CliError`, `run()` (`:763`), `dispatch()` (`:780`), and **re-exports** so every prior
    `crustyimg::cli::*` path still resolves. Everything else moves out.
  - `src/cli/args.rs` — the clap derive structs/enums, IF pulling them out of `mod.rs` reduces bulk
    without churn; otherwise leave them in `mod.rs`. Implementer's judgment.
  - `src/cli/build.rs` — the build/lock cluster: `PreparedTarget`, `Built`, `BuildCtx`, `BuildOutcome`,
    `load_manifest`, `prepare_target`, `output_collision_key`, `check_output_injective`, `cache_key_for`,
    `lock_output_path`/`record`, `build_one`, `load_lock`, `write_lock`, `run_build`,
    `run_build_watching`, `make_watcher`, `register_roots`, `watch_impl` (~lines 1377–2133).
  - `src/cli/report.rs` — info/diff reporting: `InfoReport`, `ExifTag`, format/color labels,
    `read_exif_tags`, `write_json`, `print_human`, `run_info`, `diff_passes`, `write_diff_json`,
    `run_diff`. **The shared `escape_json` lands here (or in `common.rs`)**; `lint/report.rs` re-uses it.
  - `src/cli/optimize.rs` — `run_optimize`, `run_web`, `run_optimize_autodecide`, `run_responsive`,
    `run_convert`, `EncodePlan`, `resolve_effective_quality`, and the quality-plumbing helpers they own.
  - `src/cli/ops.rs` — pixel/geometry/metadata handlers: `run_resize`, `run_pixel_op`, `run_watermark`,
    `run_thumbnail`, `run_edit`, `run_auto_orient`, `run_metadata_lane`, `run_strip`/`clean`/`set`/
    `copy_metadata`, `ResizeModes`, `resize_params`, `parse_wxh`, etc.
  - `src/cli/common.rs` — helpers genuinely shared across ≥2 of the above: `encode_one`, `apply_one`,
    `write_encoded`, `build_sink`, `resolve_format`, `load_recipe`, `fmt_bytes`, `plural`,
    `require_out_dir_for_batch`.
- **Files modified:** `src/lint/report.rs` — drop its local `escape_json`, use the shared one.
- **New exports:** none new to the outside world. Internal items shared across the new submodules become
  `pub(crate)` / `pub(super)` — **not** `pub`. The public API at `crustyimg::cli::*` is unchanged
  (preserved via `pub use` in `mod.rs`).

## Acceptance Criteria

- [ ] **Byte-identical CLI output.** For a representative command set (see the Verification Gate), the
      captured stdout + stderr + exit code is identical before the split and after every extraction — proven
      by a script, not eyeballed.
- [ ] **Public paths preserved.** Every `crustyimg::cli::` path that resolved before still resolves; a grep
      of `src/main.rs` + `tests/` for `cli::` is cited with its hit count and each hit confirmed to resolve
      (a mechanical sweep gets a mechanical check).
- [ ] **One JSON escaper.** `escape_json` exists in exactly one place; `src/cli/mod.rs:2323` and
      `src/lint/report.rs:124` both become uses of it. Their pre-merge behavior is proven identical on
      adversarial input (quotes, backslashes, control chars, unicode) before the merge.
- [ ] **No widened visibility.** No item is made more public than the split requires (`pub(crate)`/
      `pub(super)`, not `pub`).
- [ ] **No signature/param changes** anywhere on the CLI surface or the moved handlers (the
      `run_optimize` arg-bundling is explicitly deferred — see Out of scope).
- [ ] **Green matrix.** `cargo test`, `cargo build --no-default-features` (lean/CI-lean), the wasm build,
      `cargo fmt --check`, and `cargo clippy` (with the repo's lint level) all pass. The full test set —
      including the large `#[cfg(test)]` block currently in `cli/mod.rs` — still runs (no dropped module).

## Failing Tests

This is a behavior-preserving refactor, so the "failing test" is a **differential gate**, not a new unit
test of new behavior:

- **A golden-output diff harness** (script under `scripts/` or `tests/`, e.g. `scripts/cli-golden.sh`)
  - Captures stdout+stderr+exit-code for a representative command set on a fixed fixture **from the
    pre-split binary** (the oracle), then asserts the post-split binary reproduces each **byte-for-byte**.
    Command set must cover every submodule: `info --json`, `diff --json`, `lint --format json`,
    `lint --format sarif`, `optimize`/`web` with `--explain`/`--timing`/`--json`, `build`, `resize`,
    `watermark`, `meta strip`/`clean`/`set`/`copy`. This harness is the whole point — write it first.
- **`escape_json` equivalence** (a unit test, before the merge)
  - `"escape_json_impls_are_equivalent"` — feed both the `cli` and `lint` implementations the same
    adversarial strings (`"`, `\`, `\n`/`\t`/`\0` and other control chars, multi-byte unicode, a lone
    surrogate-ish sequence) and assert identical output. Only after this passes may they be merged. Then a
    post-merge test asserts the single helper still produces those outputs.

## Implementation Context

### Decisions that apply
- `DEC-012` — the `main` → `cli::run()` → dispatch → exit-code-mapping structure; `main.rs` delegates
  entirely to `crustyimg::cli::run()`. Preserve that boundary exactly.
- `DEC-007` — thiserror internal errors + exit-code mapping at the CLI boundary, no panics on expected
  failures. `CliError` and the mapping stay in `mod.rs`; moved handlers keep returning the same typed errors.

### Constraints that apply
- None from `guidance/constraints.yaml` specifically gate this move; the governing constraint is the
  spec's own **byte-identity gate** — the refactor is valid only if output is provably unchanged.

### Prior related work
- `SPEC-063` / `SPEC-067` — the `build`/`--watch` cluster whose code moves to `cli/build.rs`.
- `SPEC-088` — the unified audit `--json`/`--timing` surface (the JSON serializers + `escape_json` in
  scope here); its `optimize.explain/v1` schema must remain byte-identical.
- The pre-launch **Rust audit** (`docs/research/proj-008-rust-directives-audit.md`) — the source of this
  spec; treat its evidence (line counts, duplication, seams) as given, but **confirm each seam's real
  dependencies against the file before assigning it.**

### Out of scope (for this spec specifically)
- **Any signature/param change.** The `run_optimize` 11-argument signature is a real smell, but bundling
  it into a params struct is a **separate, optional, cosmetic follow-up** — mixing a behavior-preserving
  move with a signature change destroys the "output is byte-identical" gate. Note it in the deferred list;
  do not do it here.
- No new dependencies; no `Cargo.toml` / `constraints.yaml` edits; no logic changes; no performance work.
- No change to the CLI surface, flags, help text, or output.

### Deferred (capture, don't do)
- `run_optimize` argument-struct bundling (own spec, if ever — cosmetic).
- Any further module splits beyond what makes the seams cohesive.

## Notes for the Implementer

- **The gate is the deliverable.** Write the golden-output harness FIRST, capture the oracle from `main`
  before touching anything, then extract **one module per commit** and re-run the diff after each — so a
  regression is caught at the commit that caused it and `git bisect` stays useful.
- **Confirm seams, don't trust the inventory.** The candidate module lists above are starting boundaries;
  grep each item's real callers/callees before assigning it. Put a helper in `common.rs` only if ≥2
  submodules use it.
- **The `#[cfg(test)]` block is a trap.** `cli/mod.rs` has a large test module (exit-code + arg-parse
  tests). Decide explicitly whether each test moves with its code or stays as a cli integration test — and
  confirm the **full test count is unchanged** (a silently dropped `#[cfg(test)] mod tests` is the classic
  regression here; count tests before and after).
- **Prove `escape_json` equivalence BEFORE merging** the two copies (the equivalence test above), so the
  dedup can't quietly change JSON output.
- **Don't widen visibility.** Reach for `pub(crate)`/`pub(super)`; `pub` only for what was already public.
- **`cargo fmt` + `git add -u` before each commit** — a later fmt reformats earlier commits and breaks the
  CI fmt check even though local `--check` passed (a known repo footgun).
- **Confirm the lean + wasm legs compile.** The split touches only native `cli/`, but `cargo build
  --no-default-features` and the wasm build must still pass — run them, don't assume.
- **Model choice is the maintainer's call at build dispatch.** This is mechanical (favors Sonnet, extends
  the model experiment) but large and regression-prone (favors Opus care); the byte-identity gate is the
  safety net either way. Verify on Opus regardless.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-097-cli-decomposition`
- **PR (if applicable):** none yet — not opened per instructions (build only)
- **All acceptance criteria met?** yes
  - Byte-identical CLI output: `scripts/cli-golden.py` (21 cases spanning every
    submodule) captured an oracle from the pre-split binary and re-verified
    21/21 byte-identical after every single commit, including the final
    state. A negative control (deliberately mutated one output string,
    rebuilt, re-ran `check`) proved the harness actually detects a real diff
    (exit 1, correct case flagged) before being trusted as a green gate.
  - Public paths preserved: `grep -rn "cli::" src/main.rs tests/` → 2 hits,
    both in `src/main.rs` (the doc comment + the `crustyimg::cli::run()`
    call) — unchanged from the pre-split count. Every item that was `pub` in
    the original `cli/mod.rs` (`Cli`, `GlobalArgs`, `QualityTarget`,
    `ProfileArg`, `ExplainFmt`, `AutoQuality`, `Commands`, `MetaCommand`,
    `CliError`, `run()`, `WEB_DEFAULT_LONG_EDGE`) is still `pub` at the exact
    same `crustyimg::cli::<name>` path — `WEB_DEFAULT_LONG_EDGE` moved into
    `optimize.rs` and is re-exported via `pub use optimize::
    WEB_DEFAULT_LONG_EDGE;` in `mod.rs`.
  - One JSON escaper: `escape_json` now lives once, in `cli::report`
    (`pub(crate)`, since `lint::report` reuses it across the `cli`/`lint`
    module boundary — `cli::mod`'s `mod report;` became `pub(crate) mod
    report;` to allow that). `lint::report`'s independent copy is deleted.
    Equivalence was proven BEFORE the merge (`escape_json_impls_are_equivalent`,
    committed separately) with adversarial input (quotes, backslashes,
    control chars incl. the 0x20 boundary and DEL, multi-byte unicode, the
    code points flanking the surrogate range, U+FFFD); after the merge that
    test was replaced by three permanent regression tests asserting the
    single surviving helper still produces the same proven outputs.
  - No widened visibility: every new cross-submodule item is `pub(super)`;
    only `escape_json` and the `report` module declaration are `pub(crate)`
    (required for the `cli`/`lint` boundary); `WEB_DEFAULT_LONG_EDGE` stays
    `pub` exactly as it was before the split (not a widening — a relocation).
  - No signature/param changes: every extracted function's signature was
    copied verbatim; `run_optimize`'s 11-argument signature is untouched
    (bundling deferred, per Out of scope).
  - Green matrix: `cargo test` 769/769 (see test-count reconciliation
    below), `cargo build --no-default-features` clean, `cargo clippy
    --all-targets -- -D warnings` clean on both feature sets, `cargo fmt
    --check` clean, `just wasm-check` clean (the `cli` module is
    `#[cfg(not(target_arch = "wasm32"))]`-gated in `lib.rs` — it was never
    part of the wasm build surface, so this confirms the split didn't leak
    a stray reference into a wasm-reachable module, not that `cli/` itself
    compiles for wasm).
- **New decisions emitted:** none
- **Deviations from spec:**
  - Extracted `common.rs` together with `build.rs` in the first commit
    (rather than build → report → optimize → ops → common in that literal
    order) — `build.rs`'s `run_build`/`run_build_watching` depend on
    `common.rs`'s `load_recipe`/`encode_one`/`write_encoded`/
    `BATCH_PROGRESS_TEMPLATE`, and `common.rs` is a leaf dependency of every
    other submodule, so extracting it first kept every commit compiling
    standalone instead of carrying temporary duplicate definitions.
  - `plural()` (and its test) went to `build.rs`, not `common.rs`, despite
    being in the spec's common.rs candidate list — after the move it has
    exactly one real caller (`run_build`), which fails the spec's own
    "`common.rs` only if ≥2 submodules use it" rule; confirmed by grep before
    assigning it. This is exactly the kind of inventory-vs-file mismatch the
    spec warned to check for.
  - `run_apply` (with `split_terminal_optimize`/`OPTIMIZE_STEP_OP`) is not
    named in any candidate module list; assigned to `optimize.rs` (implementer's
    judgment) since its non-trivial branch dispatches straight into the
    auto-decide engine (`run_optimize_autodecide`), and its plain-recipe
    fallback uses only `common.rs` helpers already centralized there.
  - `run_view` is likewise unlisted; assigned to `ops.rs` as a single-image
    pixel/display handler, cohesive with resize/thumbnail/watermark.
  - `lint`'s handler cluster (`LintFlags`, `parse_savings_threshold`,
    `build_lint_config`, `run_lint`, `LintReportFormat`, `lint_report_format`)
    is not named in any candidate module list; folded into `report.rs`
    (read-only reporting family, adjacent in the original file) rather than
    a separate module — no unit tests existed for this cluster in the
    original file (it's covered by `tests/lint.rs`), so nothing needed to move.
  - `args.rs` was NOT created: the spec explicitly allows leaving the clap
    derive types in `mod.rs` "if pulling them out... reduces bulk without
    churn; otherwise leave them in mod.rs" — moving `Cli`/`GlobalArgs`/
    `Commands`/`MetaCommand`/the arg enums out would have meant re-exporting
    9 additional `pub` items instead of the 1 (`WEB_DEFAULT_LONG_EDGE`) that
    actually needed it, for no reduction in mod.rs's conceptual weight (the
    clap surface + `CliError` + `dispatch()` is the front door's actual job).
    `mod.rs` is 1,426 lines post-split — mostly the `Commands` enum's
    extensive doc comments, `CliError`'s exit-code mapping + its ~250-line
    total-mapping test, and `dispatch()`.
- **Follow-up work identified:**
  - None new. The two items the spec explicitly deferred (the `run_optimize`
    11-arg → params-struct bundling; any further module splits beyond these
    5) remain deferred, unstarted, as instructed.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Several real functions/tests had no home in any candidate module list
   (`run_apply`, `run_view`, the whole `lint` handler cluster, `plural()`'s
   correct home). The spec anticipated this ("candidate boundaries... not
   gospel... implementer's judgment") but resolving each one still meant
   reading the full file and tracing every call site before assigning it —
   the single biggest time cost in the build, ahead of the mechanical
   extraction itself.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraint; the spec's own Notes (write the harness first,
   one module per commit, prove `escape_json` equivalence before merging,
   don't trust the inventory, watch for a silently dropped test module) were
   exactly the checklist this build needed and nothing else was missing.

3. **If you did this task again, what would you do differently?**
   — Extract `common.rs` first explicitly in the plan rather than discovering
   the forward-reference problem mid-build.rs-extraction — it's the obvious
   leaf dependency in hindsight (every other submodule needs at least one of
   its helpers) and planning the commit order around the dependency graph
   up front (common → build/report/optimize/ops in any order) would have
   avoided the one mid-commit detour.

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
