---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-086
  type: story
  cycle: ship
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

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop (un-metered, §4); mostly surface + deletion
        on top of the SPEC-084 engine (add `--verify`, remove `shrink`, fix the doc-comment).
    - cycle: build
      interface: claude-code
      tokens_total: 180000
      estimated_usd: 2.00
      recorded_at: 2026-07-15
      note: >
        ~35 min, own worktree. `optimize --verify` (reuse `score_winner_once`; JSON gains an `"ssim"`
        field gated on measured, non-verify byte-identical); hard-cut `shrink` (Commands::Shrink,
        run_shrink, shrink_auto_config, DEFAULT_SHRINK_MAX; neutral helper renames); stale doc-comment
        fixed. Drove the binary end-to-end (self-check, not the independent verify). Emitted DEC-071.
    - cycle: verify
      interface: claude-code
      tokens_total: 120000
      estimated_usd: 1.30
      recorded_at: 2026-07-15
      note: >
        independent adversarial pass, ~15 min, fresh worktree — CLEAN. Proved non-verify JSON
        byte-identical to main + the pinned output byte-identical; the `--verify` score matches
        `crustyimg diff`; `shrink` gone from the live surface (historical records correctly kept);
        renamed-helper regressions + gates green. One non-defect: pinned `--verify -o x.avif` silently
        ignores `--verify` (by design, mirrors the pre-existing silent `--explain`-on-pin).
    - cycle: ship
      interface: claude-code
      tokens_total: null
      recorded_at: 2026-07-15
      note: >
        ship bookkeeping in the orchestrator main loop (un-metered, §4). Clean squash-merge (PR #90,
        f54aac9) — mergeable/CLEAN, no rebase. Follow-up logged: pinned `optimize --verify` could
        surface a hint rather than silently ignore the flag (ergonomics nit).
  totals:
    tokens_total: 300000
    estimated_usd: 3.30
    session_count: 4
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
- **Branch:** `spec-086-optimize-verify-remove-shrink` · **PR:** #90 · **All acceptance criteria met?**
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
1. **What would I do differently next time?** — The build session self-verified *and* opened the PR,
   which isn't the independent check — I ran a separate adversarial verify anyway, and it was worth it
   for the confidence even though it came back CLEAN (unlike SPEC-084/085, which the independent pass
   caught defects in). For a surface+deletion spec the risk was low, but "the build verified its own
   work" isn't a substitute for the separate pass. Also (third time): remember the `cost:` block when
   Writing a spec over its scaffold.
2. **Does any template, constraint, or decision need updating?** — DEC-071 records `--verify`, the
   `shrink` removal + migration mapping, and the JSON `"ssim"` deviation (additive, gated on measured,
   `v1` unbumped). The hard-cutover grep-clean discipline worked: live surface clean, dated historical
   records (ADRs, session logs, prior specs) intentionally preserved — that distinction is the right
   default for a repo whose history is a deliverable.
3. **Is there a follow-up spec I should write now before I forget?** — A small ergonomics nit: pinned
   `optimize --verify -o x.avif` silently ignores `--verify` (mirrors the pre-existing silent
   `--explain`-on-pin) — worth a hint someday, not a spec. STAGE-030 continues: SPEC-087 (`meta` group),
   SPEC-088 (unified audit + committed bench), SPEC-089 (`convert --to`, optional). DEC-069/071 also
   carry the native(85)/wasm(80) AVIF-quality alignment for when `src/wasm.rs` is next touched.
