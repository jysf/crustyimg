# SPEC-145 — BUILD prompt: `crop` as a recipe step, in fractions of the image

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design
gives the op's maths in code terms, every refusal's exact message, 65 ACs and a 13-row mutation
table, so the work is mechanical but long. Be thorough across every row: Sonnet's known weak spot
is sweep thoroughness, and this spec is mostly sweeps.

```
just wt feat/spec-145-crop -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that
message must stand on its own. This spec is line 2 of PROJ-017's build order. `pad`, `trim`, the
page's crop UI and the `edit` flags all wait on it.

## What you are building
- **`src/operation/crop.rs`:** one `crop` op with two key groups that can't be mixed, rectangle
  (`x`, `y`, `width`, `height`) or aspect (`aspect`, `gravity`). All lengths are fractions 0–1
  of the image the step receives, per **DEC-112**.
- **Registration** in `src/operation/registry.rs`, and `get_f64` in `src/operation/mod.rs`.
- **Tests:** `tests/crop.rs` (the 32 load refusals R1–R32, the 4 apply refusals A1–A4, the CLI
  rows and DEC-095 exactness), unit and property tests in `crop.rs`, and three rows in
  `tests/wasm_roundtrip.rs`.
- **Two corpus recipes** (`v2_crop_rect`, `v2_crop_aspect`), with per-target `.output` files, and
  the literal bumps in the corpus tests.
- **The docs surfaces** the spec lists (AC-55 to AC-63), and an `### Added` entry under
  `CHANGELOG.md`'s `## [Unreleased]`.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation, so say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `b00089f` is in your history;
- `decisions/DEC-112-*.md` exists.

If any check fails, stop and report.

⚠ **SPEC-146** (PROJ-011: CI runs the wasm round-trip tests) may be in flight. It touches CI and
possibly `tests/wasm_roundtrip.rs`.
- If it lands on `main` before your PR is ready, rebase over it. CI then runs your wasm rows, so
  read that job.
- If the rebase conflicts in `tests/wasm_roundtrip.rs`, keep both sides' rows. If it conflicts in
  any file outside the spec's list, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-145-crop-as-a-recipe-step-in-units-that-survive-a-resize.md`.
   Read it in full: `## Frame`, `## Design` (especially "The op, in code terms" and "Exact
   messages"), AC-1 to AC-65, the Failing Tests, the mutation table M1–M12,
   `## Implementation Context` and `## Notes for the Implementer`. **The design calls are
   settled.** If one turns out wrong, say so in Build Completion and in your report; don't quietly
   re-decide it.
2. **DECs:** DEC-112 (the unit rule, new), DEC-091, DEC-095, DEC-102, DEC-109 and DEC-110. Read
   them yourself; don't trust a summary.
3. `tests/fixtures/recipe_corpus/README.md`: the bless procedure. Follow it exactly.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 11, 12, 14 and 15 especially). These bind PROJ-017. Also read
   `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
1. **Fractions only** in a recipe. No pixel units; pixels arrive later through `edit --crop`.
2. **`aspect` takes whole numbers only** (`"239:100"`). A decimal is refused with a message
   that shows the whole-number spelling.
3. **A crop that rounds to 0 px is refused at apply, never clamped.** The check lives in
   `Crop::apply`, **not** the refusal hook. Native pipelines skip a refused step and still write
   the image, so the hook would exit 0 with the uncropped image. The message names `crop`, the
   rounded size and the image size. It carries no step index: no op's apply-time error does
   today, and that gap is filed separately. The maintainer accepted this reading on 2026-11-05.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push
after each numbered stage.
1. **Red tests first.** Write every failing test the spec lists. Run them against `main`'s code
   and quote the baseline: the integration rows fail with `unknown operation 'crop'`. Open the PR
   **as a draft** with this commit.
2. **The op, green locally.** Implement `crop.rs` and the registration as the spec words them.
   Every refusal message is **byte-for-byte** the spec's text: copy it, never retype it. The
   refusal tests run every row, collect every mismatch, and assert the row count, so a wrong
   message shows up in full.
3. **The corpus, for your own target.**
   - Add the two `.toml` files and write their `.golden` files by the README.
   - Raise every literal in one commit: `GOLDEN_LINES` 51 → 57, `ROWS` 48 → 54, the 42 → 48, and
     `pairs.len()` 14 → 16. `EXPECTED_DIMS` gets length 16.
   - Bless your own target only.
   - ⛔ **`git status` must show only added files plus the literal lines.** If any existing
     `.output` or `.golden` is modified, stop: that's AC-52 failing, and it means something else
     moved. Report it; don't re-bless around it.
   - Push. Start `gh pr checks <PR> --watch` **in the background**. Never poll. When it exits,
     copy the other two targets' `.output` files from the failing log's `----- BEGIN … -----`
     blocks (`gh run view <run> --log-failed`). They count only once a **second** CI run is green
     on every leg. Then `cmp` the three per-target files for each recipe (AC-51), and quote the
     result.
4. **The docs.** Make each docs AC (AC-55 to AC-63) true at the `file:line` the spec gives.
   Line numbers may have shifted, so find each place **by content**. `tests/docs_ops.rs` must
   resolve the new `op = "crop"` example.
5. **Mutations M1–M12, one at a time** (HANDOFF traps 12 and 14). For each one:
   - apply it at the location the spec names, and check `git diff` is non-empty;
   - rebuild **clean** (a fresh `CARGO_TARGET_DIR`, or `cargo clean -p crustyimg`), because a
     reverted source doesn't rebuild the binary;
   - run the tests, and record exactly which rows go red against the spec's red/green lists;
   - restore, rebuild, and show it green.

   **M11** edits a corpus recipe on a scratch copy only. Afterwards, show `git status` clean.
   **M12** restores `main`'s registry verbatim, and every crop integration row must go red. Paste
   each mutation's red output, trimmed to the lines that name rows, into Build Completion.
6. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`. Run them one
   after another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check`
   on each. Crop has no feature gate, so the lean leg must run its tests, not skip them.
7. **Wasm, locally:** `just wasm-check` and `just wasm-test`. Quote each command's result line. CI
   doesn't run `wasm-test` until SPEC-146 lands, so this run is the evidence. Also read CI's wasm
   bundle-size gate leg (`just check` doesn't run it), and record the size delta.

## The things most likely to go wrong
- **A message retyped instead of copied.** Thirty-six refusal messages leave many chances for one
  stray character. Copy each one from "Exact messages", and let the collect-every-row test show
  any mismatch.
- **f32 anywhere on the fraction path.** M8 shows `x = 0.0009375` gives left edge 1 instead of 2
  under f32. Read every key as `f64` and do the maths in `f64`. The aspect maths is `u64`
  integers.
- **`crop_imm` clamps silently.** It never errors on an out-of-bounds rectangle. That's why the
  load check and the apply-side guard both exist (M3, M3b). Don't remove either as redundant.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including the lean one and the
  bundle-size gate.
- **Stay in the spec's files.** The PROJ-011 orchestrator is keeping clear of them for you. If
  you need a file outside the list, say why in Build Completion.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. AC-64 asserts this; paste `git diff --stat origin/main...HEAD`
  for those paths.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI.** Run `gh pr checks --watch` in the background only.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`.
  zsh does not word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no
  `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir when
  you're done with it.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids. That includes every
  refusal message and every doc line.
- **Budget ~120 exchanges.** The second CI round for the corpus files is expected. If you're past
  budget with the matrix green, stop and report what's left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a
     phrase on what changed;
   - AC-1 to AC-65, one line each: met, or not and why;
   - M1–M12, each with its applied line, the rows that went red and the rows that stayed green,
     against the spec's table, with every mismatch called out;
   - the corpus: each CI leg's result, and the `cmp` of the three per-target files;
   - the matrix legs, the local `wasm-test` result line, and the bundle-size delta;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-145 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-145 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `feat(recipes): a crop step in fractions of the image (SPEC-145)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-091, DEC-095, DEC-102, DEC-109, DEC-110;
     - Constraints checked;
     - New decisions: DEC-112 (written at design, on `main`);
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-145 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.
Then, briefly:
- AC completion (65 rows, as a count, plus any row not met);
- each mutation's flips against the table, and any row that didn't match;
- the corpus: CI legs and the three-file `cmp`; and confirmation that no existing `.output`
  changed;
- the matrix result, the local `wasm-test` line, and the bundle-size delta;
- files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost.
