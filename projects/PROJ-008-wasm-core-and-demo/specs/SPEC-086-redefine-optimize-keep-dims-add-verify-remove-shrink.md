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

- [x] `optimize <photo>` (no flags) stays **lean and score-free** (fast keep-dims Mode::Fast, never
      bigger) — unchanged from SPEC-084. (`optimize_default_has_no_score`; driven end-to-end.)
- [x] `optimize --verify <photo>` reports the winner's **SSIMULACRA2 score** (one `score_winner_once`
      call; ~107 ms/MP added — STAGE-030 notes); the score matches a `diff` of input vs output within
      tolerance; `--verify` works with `--json` (the `--explain=json` channel gains an `"ssim"` field).
      (`optimize_verify_reports_score`; driven: ` · ssim 84.3` + `"ssim":84.3`.)
- [x] **`crustyimg shrink` no longer exists** (exits 2 as an unknown subcommand); `run_shrink` is gone;
      **no `shrink` reference remains** on the live surface — `src/`, `tests/`, `README`, and every
      current-facing `docs/` file are clean; help + completions are clap-generated so `shrink` is absent
      (`grep -rn shrink` clean except intentional migration notes + dated historical records).
      (`shrink_subcommand_is_gone`, `no_shrink_references_remain`.)
- [x] The `optimize` help + the `run_optimize` doc-comment are **honest** (no "visually-lossless by
      default").
- [x] `--target`/`--ssim`/`--max-size` (opt-in searches), `--profile preserve` (DEC-059), `convert`, and
      `web` are **unchanged**.
- [x] `cargo test` (default **and** `--features avif`), `cargo clippy` (both), `cargo fmt --check`, and
      `cargo build --no-default-features` pass; no test references `shrink` (except the two removal
      tests + one historical note).

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
- **Branch:** `spec-086-optimize-verify-remove-shrink` · **PR:** #TBD · **All acceptance criteria met?**
  Yes — all six acceptance boxes checked; the five design Failing Tests exist and pass (plus a migrated
  `optimize_unreachable_target_warns_best_effort` and a `explain_json_includes_ssim_only_when_verified`
  unit test); driven end-to-end on a real photo (`optimize` no-score vs `--verify` `· ssim 84.3` vs
  `web` always-scores; `shrink` exits 2). Gates green: `cargo test` (default 717 / avif 730), `cargo
  clippy` (both, clean), `cargo fmt --check`, `cargo build --no-default-features`.
- **New decisions:** DEC-071 (`optimize --verify` opt-in score + the JSON `"ssim"` extension + the
  `shrink` removal + the migration mapping).
- **Deviations:**
  1. **`--verify` also extends the JSON explain**, not just the human summary. The spec said "report the
     score (human + `--json`)"; the pinned `crustyimg.optimize.explain/v1` had no score field, so I added
     a trailing `"ssim":NN.N` emitted **only** when a winner is scored (`web`/`--verify`) — a non-verify
     run's JSON stays byte-identical. Recorded in DEC-071.
  2. **Scope of the `shrink` grep-clean.** Cleaned the live surface (`src/`, `tests/`, `README`, and all
     current-facing docs: cli-reference/recipes/USAGE/api-contract/architecture/feature-exploration/
     backlog/territory + justfile). **Left dated historical records untouched** — decision ADRs
     (`decisions/`), session/blog/research/review docs (`docs/sessions|blog|research|reviews`), and prior
     work-tracking specs/stages (`projects/`) — because rewriting them would falsify the project's history
     (the honest read of "except intentional migration notes"). The `no_shrink_references_remain` test
     targets the clap help/command list, which is fully clean.
  3. **Migrated, not deleted, two feature tests + the unreachable-target test** to `optimize` (identical
     search machinery regardless of verb) to preserve WebP/AVIF-target + best-effort coverage; deleted the
     rest of the `shrink_*` integration tests (optimize/convert already cover the same paths).
- **Follow-ups:** DEC-069's native(85)/wasm(80) AVIF-quality divergence still open (no wasm change here).
### Build-phase reflection
1. **The pinned-format path silently skips `--verify`.** `optimize -o x.avif` bypasses the auto-decision
   (and therefore the score) — by design (a pin is an explicit override), and consistent with how
   `--explain` is ignored there. Worth remembering: `--verify` only reports on the auto-decide path
   (`--out-dir` or format-inferred), which is where the score is meaningful.
2. **A near-name-collision that wasn't.** The global `--check` flag's help text begins with the word
   "Verify", which made a naive grep of `optimize --help` look like a `--verify` collision; the real
   global build-assert flag is `--check`/`--frozen`/`--locked`, so `--verify` was free. Read the flag
   NAME, not a description that happens to contain the word.
3. **Deletion is a coverage question, not just a grep.** Removing ~50 `shrink` test references risked
   silently dropping edge coverage (unreachable-target best-effort, perceptual-drives-WebP); I checked
   optimize/convert's existing tests first, then migrated the few unique ones instead of deleting blindly.

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
