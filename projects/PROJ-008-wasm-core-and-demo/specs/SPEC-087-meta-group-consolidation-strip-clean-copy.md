---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-087
  type: story
  cycle: verify
  blocked: false
  priority: medium
  complexity: S
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-15
references:
  decisions: [DEC-017]
  constraints: [ergonomic-defaults, every-public-fn-tested, test-before-implementation, one-spec-per-pr]
  related_specs: [SPEC-086]
value_link: >
  One intent per verb-group: fold the scattered metadata verbs (strip / clean / copy-metadata) into a
  single `meta` group with subcommands, so the top-level surface reads as distinct *jobs* (web,
  optimize, convert, resize, meta, …) instead of a flat list where metadata ops sit next to encoders.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop (un-metered, §4). A pure surface
        move within the STAGE-030 freeze; the design grounded the clap shape (nested subcommand)
        and confirmed byte-identity as the whole value.
    - cycle: build
      interface: claude-code
      tokens_total: 210000
      estimated_usd: 2.30
      recorded_at: 2026-07-15
      note: >
        metered subagent, own worktree — ESTIMATE (orchestrator finalizes subagent_tokens at
        ship, §4). Introduced the `Meta` group + `MetaCommand` enum (strip/clean/copy),
        removed the 3 top-level verbs, kept run_* handlers unchanged; grep-cleaned the live
        surface (README/docs/cli-reference/api-contract/recipes/data-model/architecture/moat/
        feature-exploration + the lint fix fragments + constraints.yaml rule) and the tests.
        Gates green (test default+avif, clippy, fmt, no-default-features, validate). Flagged a
        design-grounding error: a top-level `set` verb DOES exist (left top-level per spec scope).
    - cycle: verify
      interface: claude-code
      tokens_total: 190000
      estimated_usd: 2.10
      recorded_at: 2026-07-15
      note: >
        metered subagent, own detached worktree — ESTIMATE (orchestrator finalizes subagent_tokens
        at ship, §4). Adversarial verify: built the binary here AND the parent-commit (80c71c1) old
        binary, drove both on one EXIF+GPS JPEG fixture — `meta strip`/`meta clean --gps`/`meta copy`
        are byte-identical to the OLD top-level `strip`/`clean`/`copy-metadata` output (independent of
        the build's library-op test). Confirmed top-level verbs exit 2, bare `meta` prints help, and
        the lint fix fragments (`crustyimg meta clean --gps <file>`) run post-cutover. Re-ran all gates
        green. Verdict CLEAN.
---

# SPEC-087: `meta` group consolidation (strip / clean / copy)

## Context

STAGE-030's taxonomy freeze wants ~14 one-intent verbs. Today the metadata operations are three
**top-level** verbs — `strip`, `clean`, `copy-metadata` (`src/cli/mod.rs`: `Commands::Strip` ~396,
`Commands::Clean` ~399, `Commands::CopyMetadata` ~417; dispatched at ~851/852/865) — flattened next to
the encoders. This spec groups them under a single **`meta`** command with subcommands, so the surface
reads by job. It's a pure surface move (a hard cutover, no aliases); the underlying `run_strip` /
`run_clean` / `run_copy_metadata` behavior is unchanged.

Grounding (probed): there is **no `set` verb today** — the strategy brief's `meta {strip,clean,set,copy}`
listed a `set` that doesn't exist. Writing a metadata value is a *new capability*, not a move, so it's
out of scope here (see below). `auto-orient` is an image operation (it bakes orientation into pixels),
not a metadata verb — it **stays top-level** (DEC-017).

## Goal

Introduce a **`meta` command group** — `meta strip`, `meta clean`, `meta copy` — that wraps today's
`strip` / `clean` / `copy-metadata` behavior unchanged, and **remove the three top-level verbs** (hard
cutover). Update every reference. No behavior change, no new capability.

## Inputs — files to read

- `src/cli/mod.rs` — `Commands::Strip`/`Clean`/`CopyMetadata` (~396–420), their dispatch arms
  (~851–865), and `run_strip`/`run_clean`/`run_copy_metadata`. The `Web`/`Apply` nested-args pattern +
  any existing subcommand-group example for the clap shape.
- `src/metadata/` — the ops behind the verbs (unchanged).
- Everything naming the old verbs: `grep -rn 'strip\|clean\|copy-metadata'` across `src/`, `docs/`,
  `tests/`, `README`, completions.

## Outputs

- **`src/cli/mod.rs`** — a `Meta` command with a subcommand enum (`Strip`/`Clean`/`Copy`), each carrying
  the same args as the current top-level verb; dispatch each to the existing `run_*` (no logic change).
  **Remove** `Commands::Strip`, `Commands::Clean`, `Commands::CopyMetadata` and their top-level dispatch.
  `meta copy` keeps `--from`/`--to`; `meta clean` keeps `--gps`.
- **Docs/tests/completions:** rewrite every `strip`/`clean`/`copy-metadata` usage to `meta strip` /
  `meta clean` / `meta copy` on the live surface (README, `docs/`, help examples, integration tests,
  shell completions). Dated historical records stay intact (the SPEC-086 grep-clean discipline).
- No DEC needed (a surface move within the STAGE-030 freeze); note it in the STAGE-030 stage doc.

## Acceptance Criteria

- [x] `meta strip <inputs>`, `meta clean [--gps] <inputs>`, `meta copy --from A --to B` produce
      **byte-identical** results to today's `strip`/`clean`/`copy-metadata` (same ops, just moved).
      (`meta_subcommands_match_old_verbs` asserts the CLI bytes equal the underlying container-lane op.)
- [x] The **top-level** `strip`/`clean`/`copy-metadata` verbs **no longer exist** (unknown-subcommand
      exit 2); `meta` with no subcommand prints help listing the three.
- [x] `auto-orient` remains **top-level** (unchanged); it is NOT a `meta` subcommand.
- [x] No `strip`/`clean`/`copy-metadata` **top-level** reference remains on the live surface (help,
      completions, README, user-facing docs); historical records untouched.
- [x] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, and
      `cargo build --no-default-features` pass.

## Failing Tests (written at design)

- **Integration / `src/cli`**
  - `meta_subcommands_match_old_verbs` — `meta strip`/`clean`/`copy` output byte-identical to the old
    top-level verbs on the same inputs (capture pre-move golden bytes in the test).
  - `top_level_metadata_verbs_are_gone` — `strip`/`clean`/`copy-metadata` at top level error as unknown
    subcommands; the parser has no such variants.
  - `meta_bare_prints_subcommand_help` — `meta` alone lists strip/clean/copy.
  - `auto_orient_still_top_level` — `auto-orient` unchanged.

## Implementation Context

### Decisions that apply
- `DEC-017` — `auto-orient` bakes orientation into pixels + drops the metadata bundle; it is an image
  op, not a metadata verb → stays top-level (do not fold it into `meta`).

### Constraints
- `ergonomic-defaults` — the grouped surface should read by job; `one-spec-per-pr` — this is one
  coherent surface move.

### Out of scope (this spec)
- **A `meta set` capability** (writing a metadata value) — that's a *new feature*, not a consolidation;
  frame its own spec if the maintainer wants it. This spec groups only the three existing verbs.
- The unified audit report / `--json`/`--timing` / committed bench (SPEC-088); `convert --to` (SPEC-089).
- Any change to metadata op behavior.

## Notes for the Implementer
- **Pure move, prove byte-identity.** Capture golden output from the old verbs before deleting them, and
  assert the `meta` subcommands reproduce it — the value of a consolidation is that nothing changed but
  the path.
- **Hard cutover, live-surface grep-clean** (SPEC-086 precedent): rewrite user-facing docs/completions;
  leave dated historical records.
- Mirror the existing nested-args clap pattern (`web`/`apply`); don't invent a new one.

---

## Build Completion
- **Branch:** `spec-087-meta-group` · **PR:** #91 · **All acceptance criteria met?**
  Yes — all five acceptance boxes checked; the four design Failing Tests exist and pass
  (`meta_subcommands_match_old_verbs`, `top_level_metadata_verbs_are_gone`,
  `meta_bare_prints_subcommand_help`, `auto_orient_still_top_level`) plus a new
  `meta_subcommand_help_parses`. Driven end-to-end on a real JPEG: `meta strip`/`meta clean --gps`/
  `meta copy` produce bytes byte-identical to the container-lane ops; top-level `strip`/`clean`/
  `copy-metadata` now exit 2; bare `meta` prints subcommand help. Gates green: `cargo test` (default
  723 / avif), `cargo clippy` (clean), `cargo fmt --check`, `cargo build --no-default-features`,
  `just validate`.
- **New decisions:** None. A pure surface move within the STAGE-030 freeze (as framed) — no DEC needed.
- **Deviations:**
  1. **`set` verb discrepancy (design-grounding error — FLAGGED, left in place).** The spec's Context +
     the STAGE-030 backlog both assert "there is **no `set` verb today**." That is **wrong**: a top-level
     `Commands::Set` (`set --artist/--copyright/--description`, SPEC-027) exists and is documented. Per the
     spec's explicit scope (move exactly `strip`/`clean`/`copy-metadata`; a `meta set` *write* capability
     is out of scope), I left `set` **top-level, unchanged**. This leaves one metadata verb (`set`)
     stranded outside the `meta` group — a taxonomy inconsistency the maintainer should rule on (fold
     `set` → `meta set` in a follow-up, or keep it top-level deliberately). Not fixed here because it
     exceeds the spec's enumerated scope and has no failing test.
  2. **`run_clean` usage-error string kept verbatim.** `meta clean` without `--gps` still errors
     `"clean requires --gps"` (not `"meta clean requires --gps"`) — the handler was moved *unchanged*
     per the spec directive. Success-path bytes (the whole point) are byte-identical; the message is a
     cosmetic nit a follow-up could polish.
  3. **Lint fix fragments updated to the new path.** The lint rules emit runnable fixes
     (`crustyimg <fix> <file>`); their fragments `strip`/`clean --gps` were rewritten to `meta strip`/
     `meta clean --gps` so the suggested command still runs after the cutover (rules.rs, mod.rs, report.rs
     goldens + the lint integration/unit tests).
- **Follow-ups:**
  - Maintainer call on `set`: fold into `meta set` (own spec — and it would make the group whole), or
    keep top-level by design. Correct the "no `set` verb exists" note in the SPEC-087 Context + STAGE-030
    backlog either way.
  - Optional: `meta clean` usage message wording (`clean` → `meta clean`).
### Build-phase reflection
1. **The spec's own grounding was falsifiable — and false.** "There is no `set` verb today" read like a
   settled probe, but the code has `Commands::Set`. Grounding claims in a spec are hypotheses to re-check
   against the tree at build time, not givens; a `grep` of the `Commands` enum caught it in seconds.
2. **Byte-identity is best proven against the op, not a captured blob.** "Capture golden bytes from the
   OLD verbs before deleting them" is impossible once the verb is gone and would need a committed binary
   fixture (against convention). The container-lane fns are `pub`, so the test compares the `meta` CLI's
   stdout to `metadata::strip_all/clean_gps/copy_metadata` directly — the exact code both the old and new
   paths dispatch to. Self-contained, no fixtures, and a *stronger* invariant than a frozen hash.
3. **A "pure surface move" still touched fifteen files.** The move itself is ~30 lines of clap; the value
   ("nothing changed but the path") only holds if every *rendered* path changes too — including the lint
   fixes users copy-paste (`crustyimg clean --gps …` would 404 after the cutover). The grep-clean, not the
   enum edit, was the real work.

---

## Verify — ✅ APPROVED (CLEAN)

Independent verify session (fresh, own detached worktree at `origin/spec-087-meta-group`).
Verdict: **CLEAN — no defects.** Every acceptance criterion has a completion-table row and was
independently re-verified (not just re-run from the build's own tests).

**Acceptance criteria, row-by-row:**
1. **Byte-identity (`meta strip`/`clean --gps`/`copy`).** Proven adversarially *against the old code
   path*, not only the library op the build's test chose: built the parent-commit (`80c71c1`) OLD
   binary and this branch's NEW binary, drove both on one EXIF+GPS+copyright JPEG fixture. All three
   ops `cmp`-identical — strip 688 B, clean 876 B, copy 1002 B. Semantics correct (strip removes all;
   `clean --gps` drops GPS, keeps Orientation + Copyright). ✅
2. **Top-level verbs gone / bare `meta` help.** `strip`, `clean`, `copy-metadata` at top level all
   exit **2** (unknown subcommand; `copy-metadata` even suggests `meta`); bare `meta` prints the group
   help listing strip/clean/copy and exits 2 (`arg_required_else_help`). ✅
3. **`auto-orient` top-level, unchanged.** Confirmed top-level; `set` (SPEC-027) also confirmed still
   top-level, untouched (the flagged out-of-scope deviation). ✅
4. **No stale top-level ref on the live surface.** Grepped `src/`/`docs/`/`README`/`guidance`. Remaining
   hits are all legitimate: dated historical records (sessions/research/reviews/blog), library fn names
   (`strip_all`/`clean_gps`) in `metadata/` unit tests, a `"clean"` *test fixture stem*, and the
   crate-rustdoc per-SPEC changelog line for SPEC-026 (the module capability, not a CLI example).
   The **lint fix fragments** are correctly `meta strip` / `meta clean --gps`; drove `crustyimg lint`
   on a real image and confirmed the emitted `crustyimg meta clean --gps <file>` **parses and runs**
   post-cutover (it strips GPS; the earlier exit-5 was a self-inflicted `-o /dev/null`, an overwrite
   guard, not a parse failure). Shell completions are generated dynamically from the clap enum — the
   generated zsh/bash output shows top-level `meta`/`auto-orient`/`set` with strip/clean/copy nested
   under `meta`, no stale top-level entries. ✅
5. **Gates.** Re-ran all myself: `cargo test` **723** (default) / **736** (`--features avif`),
   `cargo clippy --all-targets` clean (default + avif), `cargo fmt --check` clean,
   `cargo build --no-default-features` builds, `just validate` passes. The five named design tests
   (`meta_subcommands_match_old_verbs`, `top_level_metadata_verbs_are_gone`,
   `meta_bare_prints_subcommand_help`, `auto_orient_still_top_level`, `meta_subcommand_help_parses`)
   exist and pass. ✅

**Pure-move confirmation:** the `src/` diff to `run_strip`/`run_clean`/`run_copy_metadata`/
`run_metadata_lane` is doc-comment-only; behavior routes through the new `MetaCommand` match to the
identical handlers. No metadata-op logic changed. No decision drift (`just decisions-audit --changed`
clean; build correctly declared no new DEC). Prior cycles have `cost.sessions` entries.

**On the `set` deviation:** confirmed SPEC-087 did its scoped move correctly and left the top-level
`set` verb untouched. The "no `set` verb exists" grounding error and the fold-`set`→`meta set`
question are out of SPEC-087's scope and owned by the orchestrator's follow-up spec — **not** folded
here. Not a defect against this spec.

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
