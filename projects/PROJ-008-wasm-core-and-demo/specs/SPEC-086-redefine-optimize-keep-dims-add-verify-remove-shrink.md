---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-086
  type: story
  cycle: design
  blocked: false
  priority: high
  complexity: M
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-14
references:
  decisions: [DEC-019, DEC-048, DEC-059, DEC-068, DEC-069]
  constraints: [ergonomic-defaults, every-public-fn-tested, test-before-implementation,
                one-spec-per-pr, no-unwrap-on-recoverable-paths]
  related_specs: [SPEC-084, SPEC-085]
value_link: >
  Finishes the two-tier story: `optimize` is the honest keep-dimensions byte-primitive (fast, never
  bigger, opt-in proof via --verify), and the redundant `shrink` verb — whose downscale-re-encode `web`
  now does better with AVIF — is removed. One intent per verb.
---

# SPEC-086: redefine `optimize` (keep-dims + `--verify`) + remove `shrink`

## Context

SPEC-084 already turned `optimize`'s **engine** into the fast, keep-dimensions, AVIF-aware, never-bigger
byte-primitive (`Mode::Fast`), and gated the SSIMULACRA2 score **off** the default. SPEC-085 ships
`web` (the downscale-and-modernize flagship), which fully absorbs what `shrink` did (resize + re-encode)
and upgrades it to AVIF. This spec finishes the **surface**: give `optimize` an opt-in `--verify` (turn
the score back on for a single run), **remove the now-redundant `shrink` verb**, and clean up the stale
docs the SPEC-084 verify flagged. It is mostly surface + deletion — the engine work is done.

Grounding (probed): `optimize` (`Commands::Optimize`, `src/cli/mod.rs` ~311) already keeps dimensions by
default (`--max` is an explicit opt-in bound) and routes the default through `Mode::Fast`;
`score_winner_once` (`src/quality/mod.rs`) exists but is unused by the default (SPEC-084 set
`winner_score = None`). `shrink` (`Commands::Shrink` ~278, `run_shrink` ~3718) is a separate verb that
shares config helpers with `optimize` (`optimize_auto_config`, etc.).

## Goal

Add **`optimize --verify`** (compute + report the winner's SSIMULACRA2 once via `score_winner_once`,
otherwise the default stays lean — the score is off), **remove the `shrink` verb entirely** (command +
`run_shrink` + help + tests + every doc/reference), and fix the stale `run_optimize` doc-comment. No
engine change; `web`/`convert`/`--profile preserve`/the opt-in searches (`--target`/`--ssim`/`--max-size`)
are unchanged.

## Inputs — files to read

- `src/cli/mod.rs` — `Commands::Optimize` (~311) + `run_optimize` (~4004, the stale `///` at ~4009) +
  `optimize_decide_one` (where `winner_score` is set to `None`, SPEC-084); `Commands::Shrink` (~278) +
  `run_shrink` (~3718) + the dispatch `match` (~740) + `optimize_auto_config` and any shrink-only helpers.
- `src/quality/mod.rs` — `score_winner_once` (the helper `--verify` calls).
- Everything that names `shrink`: `grep -rn shrink` across `src/`, `docs/`, `tests/`, `README`, help
  strings, completions, recipes — the hard-cutover removal must leave no dangling reference.

## Outputs

- **`src/cli/mod.rs`**
  - Add `--verify` to `Commands::Optimize`; when set, call `score_winner_once` on the winner and report
    the score (` · ssim NN.N`, human + `--json`); when unset, the default stays score-free (SPEC-084).
    (`web` scores always — SPEC-085 — so `--verify` is `optimize`-only.)
  - **Remove `Commands::Shrink`** and `run_shrink` and the dispatch arm; drop any shrink-only helpers,
    reworking shared config so `optimize`/`web` keep theirs.
  - Fix the stale `run_optimize` `///` doc-comment (no "perceptual target / visually-lossless by
    default" — the default is fast fixed-quality; the searches are opt-in).
- **Docs/tests/completions:** remove/redirect every `shrink` reference (README, `docs/`, recipe docs,
  shell completions, integration tests). Where a doc taught `shrink`, point to **`web`** (downscale +
  modernize) or **`optimize`** (keep dims) as appropriate.
- **DEC** (small, or fold into DEC-069's follow-through): records `optimize --verify` + the `shrink`
  removal + the migration mapping (shrink → web / optimize).

## Acceptance Criteria

- [ ] `optimize <photo>` (no flags) stays **lean and score-free** (fast keep-dims Mode::Fast, never
      bigger) — unchanged from SPEC-084.
- [ ] `optimize --verify <photo>` reports the winner's **SSIMULACRA2 score** (one `score_winner_once`
      call; ~107 ms/MP added — STAGE-030 notes); the score matches a `diff` of input vs output within
      tolerance; `--verify` works with `--json`.
- [ ] **`crustyimg shrink` no longer exists** (exits as an unknown subcommand); `run_shrink` is gone;
      **no `shrink` reference remains** anywhere in `src/`, `docs/`, `tests/`, `README`, help, or
      completions (`grep -rn shrink` is clean except intentional migration notes).
- [ ] The `optimize` help + the `run_optimize` doc-comment are **honest** (no "visually-lossless by
      default").
- [ ] `--target`/`--ssim`/`--max-size` (opt-in searches), `--profile preserve` (DEC-059), `convert`, and
      `web` are **unchanged**.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`, and
      `cargo build --no-default-features` pass; no test still references `shrink`.

## Failing Tests (written at design)

- **Integration / `src/cli`**
  - `optimize_default_has_no_score` — default `optimize` output/JSON carries no score (SPEC-084 held).
  - `optimize_verify_reports_score` — `optimize --verify` reports a score in `(0,100]` that matches a
    `diff` of input↔output within tolerance.
  - `shrink_subcommand_is_gone` — invoking `shrink` errors as an unknown subcommand (exit for a usage
    error), and the parser has no `Shrink` variant.
  - `no_shrink_references_remain` — a repo-wide check (or a curated assertion) that `shrink` is absent
    from help/`--help` output and the command list.
- **`src/quality/mod.rs`**
  - `score_winner_once_matches_diff` — the helper's score equals `quality::score(input, output)` for the
    same pair (the `--verify` readout is the real metric).

## Implementation Context

### Decisions that apply
- `DEC-069` (SPEC-084) — `Mode::Fast` keep-dims default + the gated-off score; `--verify` is the surface
  that turns the score on. `DEC-068` — the wasm twin (note DEC-069's native(85)/wasm(80) quality
  divergence follow-up here — align it if `src/wasm.rs` is touched, else leave the note).
- `DEC-019` — SSIMULACRA2 is the metric `score_winner_once`/`--verify` reports.
- `DEC-059` — `--profile preserve` stays the engine-off / keep-source-format anchor; unaffected.
- `DEC-048` — the decision engine; unchanged (surface-only spec).

### Constraints
- `ergonomic-defaults` — `optimize` is the lean byte-primitive; the proof is opt-in (`--verify`), the
  flagship (`web`) shows it always. `one-spec-per-pr` — the `shrink` removal + `--verify` are one
  coherent surface change (the two-tier finish); keep it to this PR.

### Out of scope (this spec)
- `web` + bundled recipes (SPEC-085); the `meta` group (SPEC-087); the unified audit report / committed
  bench (SPEC-088); `convert --to` rename + social/archive recipes (SPEC-089).
- Any engine/decision change or wasm change (beyond optionally aligning the wasm AVIF-quality number).

## Notes for the Implementer
- **This is surface + deletion, not engine.** `--verify` just calls the existing `score_winner_once`;
  the removal is a clean cut (no users but the maintainer — no alias, no deprecation).
- **Leave no dangling `shrink`.** `grep -rn shrink` across the repo after the removal must be clean
  (except an intentional migration note); shell completions and `--help` snapshots count.
- **Migration story for the docs:** `shrink --max N` → `web` (downscale + modernize, smaller) or
  `optimize` (keep dims); `shrink --target/--max-size` → `optimize --target/--max-size`.
- **Verify will drive:** `optimize` lean vs `--verify` (score matches `diff`), and prove `shrink` is
  gone from the real binary's help + dispatch.

---

## Build Completion
- **Branch:** · **PR:** · **All acceptance criteria met?** · **New decisions:** · **Deviations:** · **Follow-ups:**
### Build-phase reflection
1. <answer> 2. <answer> 3. <answer>

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
