---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-087
  type: story
  cycle: design
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
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

- [ ] `meta strip <inputs>`, `meta clean [--gps] <inputs>`, `meta copy --from A --to B` produce
      **byte-identical** results to today's `strip`/`clean`/`copy-metadata` (same ops, just moved).
- [ ] The **top-level** `strip`/`clean`/`copy-metadata` verbs **no longer exist** (unknown-subcommand
      exit); `meta` with no subcommand prints help listing the three.
- [ ] `auto-orient` remains **top-level** (unchanged).
- [ ] No `strip`/`clean`/`copy-metadata` **top-level** reference remains on the live surface (help,
      completions, README, user-facing docs); historical records untouched.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, and
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
- **Branch:** · **PR:** · **All acceptance criteria met?** · **New decisions:** · **Deviations:** · **Follow-ups:**
### Build-phase reflection
1. <answer> 2. <answer> 3. <answer>

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
