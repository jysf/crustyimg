# SPEC-143 — BUILD prompt: a watermark that draws nothing is refused

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design gives the exact
messages, every boundary value, every test and a mutation table, so the work is mechanical. The
job is to be thorough across the rows and to prove no mark that draws today moved by one byte.

```
just wt feat/spec-143-refuse-invisible-watermark -- --model sonnet
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own. This spec opens **0.8.2**.

## What you are building
- A watermark that would change no pixel is refused: exit 2 on the CLI (`watermark`), exit 1 in a
  recipe (`apply`, `build`, `--watch`), and a thrown error from wasm `transform`. Nothing is
  written. Six causes: opacity 0, too faint, blank text, too small, colour alpha `00`, transparent
  image.
- One additive predicate, `Watermark::invisible_mark() -> Option<InvisibleMark>`, decided from
  the mark at 16-bit precision through the composite's own `faded_alpha16`. It is called at the
  end of `from_params` and in `run_watermark`.
- Only opacity 0 and colour alpha `00` are flag checks, before any file read. Blank text is a
  mark check: tab and U+3000 draw the font's fallback box.
- `visible_pixels()`, DEC-104's coverage figure, `new_image`/`new_text` and every mark that
  draws today are unchanged.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. Then check all three:
- the spec is at `cycle: build`;
- its design commit `44e3985` is in your history;
- `decisions/DEC-109-a-watermark-that-would-draw-nothing-is-refused-decided-from-the-mark.md`
  exists.

If any of those fails, stop and report.

Other specs are building in parallel (SPEC-141 in PROJ-017). If a PR touching `CHANGELOG.md` or
`docs/` merges before yours, rebase over it and keep both entries.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-143-a-watermark-that-draws-nothing-is-refused.md`.
   Read all of these in full:
   - `## Frame` and its `### Maintainer rulings` (1–3);
   - `## Design`: the RED baseline and Calls 1–8;
   - all 14 ACs;
   - `## Failing Tests` (FT-1 to FT-16) and the mutation table (M1–M10);
   - `## Implementation Context`;
   - `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in
   your report. Do not quietly re-decide it.
2. **DECs:** DEC-109 (new, from the design), DEC-104, DEC-106, DEC-102 and DEC-015. Read them; do
   not trust a summary.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`, "Read first" and "Traps this
   wave paid for" (1–12). They are binding; traps 9, 10 and 12 apply directly. Also
   `/guidance/constraints.yaml`.

## Rulings (settled — do not re-open)
1. **`opacity = 0` is refused.** To turn a watermark off, remove the step.
2. **Decided from the mark:** no pixel with alpha > 0 after opacity → refused. A faint mark that
   changes nothing on one particular image is drawn as asked.
3. **A positive opacity too small to keep any alpha is refused** ("too faint").
4. **`new_image` / `new_text` stay public and infallible.** No new `Refusal` variant. The new API
   is additive only, and `InvisibleMark` is `#[non_exhaustive]`.
5. **CHANGELOG under `### Breaking`**, worded as the spec's Call 7.

## Order (verify checks each step's commit)
1. **Commit 1: the golden, blessed on unmodified `src/`.** Extend `accepted_cases()` in
   `tests/watermark_cli_params.rs` from 3 to 12 cases with the `Base` enum (Call 6). Bless with
   `CRUSTYIMG_BLESS=1 cargo test --test watermark_cli_params bless -- --ignored` and commit the
   golden with **no `src/` change**. ⛔ If you bless after any `src/` edit, the golden proves
   nothing. Verify runs `git show <commit 1> -- src/` and expects it empty.
2. **Commit 2: the integration tests FT-1 to FT-12**, still against unmodified `src/`. The unit
   tests FT-13 to FT-16 call the new API and cannot compile yet, so they land with the fix in
   step 4. Record, test by test:
   - each RED row **for the reason the spec gives**, with `main`'s exit code (0 for most; FT-4's
     `3, 3, 3, 3, 3, 0`; FT-9's 3) and that it wrote output / changed 0 pixels;
   - FT-5, FT-6 and FT-10 GREEN (the over-refusal guards).
3. **The refactor:** write `faded_alpha16` and switch the 16-bit opacity loop
   (`src/operation/mod.rs`, about `:1777-1780`) to call it. **FT-6 must stay green here**; it is
   a pure refactor. Commit it on its own.
4. **The fix:** `max_alpha16`, the `max_alpha` field in all three constructors, `InvisibleMark`,
   `invisible_mark()`, the `from_params` call and its recipe messages, the two CLI flag checks
   and the post-build call in `run_watermark`, and the help lines in `src/cli/mod.rs`. Copy all
   twelve messages from Call 4 character for character. **Push a WIP commit as soon as it
   compiles** (`git commit -s`, then push).
5. **Run the mutation table, M1–M10, one row at a time.** For each one: apply the mutation, check
   `git diff` is non-empty, rebuild **clean** (a fresh `CARGO_TARGET_DIR`), run the named tests,
   and record exactly which go RED, on every path the row names (CLI, recipe, `build`, wasm).
   Then restore, rebuild and show them green. ⚠ Reverting source does not rebuild the binary.
   Also run Call 6's two golden mutations (`.floor()` in `faded_alpha16`; M6) and record FT-6's
   flips.
6. **Docs and record,** worded in the spec (Outputs lists the lines):
   - `recipes/README.md:185` and `:236-237` (the latter promises `opacity = 0` draws; rewrite it);
   - `docs/cli-reference.md`, `docs/api-contract.md` (including the refusal list),
     `docs/data-model.md` (one sentence), `skills/crustyimg/SKILL.md:186`;
   - `CHANGELOG.md` under `## [Unreleased]` → `### Breaking`, Call 7's wording;
   - the one Consequences line in DEC-106. Run `./scripts/decisions-audit.sh` afterwards.
   - No spec or DEC ids in messages, comments or the CHANGELOG.
7. **Clean matrix,** then the PR.

## The things most likely to go wrong
- **The golden must be blessed before any `src/` change** (step 1). That is the one trap the
  design flagged.
- **"Nothing written" is a file check,** not an exit code: no `-o` file, no recipe output, and
  input bytes unchanged.
- **`black16` and `faint16` are controls:** assert in the test that each decodes back as 16-bit
  (and `faint16` with alpha 4). Every boundary row asserts output ≠ input on `black16`. The base
  is 400×300 because a 60×30 mark on a smaller base trips the 5 % cap.
- **Opacity in messages prints as the shortest `f32` form.** Widening to `f64` prints
  `0.0000009999999974752427`. FT-1, FT-2, FT-8 and FT-16 catch it (M10).
- **The CLI order:** existing flag checks → the new opacity-0 and colour-alpha-00 checks → the
  `--font`/`--image` read → `invisible_mark()` → the inputs. `--text " " --font /no/f.ttf` still
  exits **3**; FT-4 pins it.
- **The 8-bit branch is not touched.** `visible_pixels()` and `opacity_does_not_change_coverage`
  are not touched, and that test stays green **without edits**.
- **wasm:** FT-12 runs under `just wasm-test`. A refusal must come back as a thrown error, not a
  panic or abort. Copy `transform_refuses_a_mark_that_does_not_fit` (`tests/wasm_roundtrip.rs`,
  about `:307`).
- **No changes to `Cargo.toml`, `Cargo.lock`, the version or `CACHE_SCHEMA_VERSION`.** No new
  recipe key and no `params()` change.
- **Stay in your files** (the spec's `## Outputs`): `src/operation/mod.rs`, `src/cli/ops.rs`,
  `src/cli/mod.rs`, a re-export in `src/lib.rs` only if `Watermark` is re-exported there, the four
  test files and the golden, the docs above, `CHANGELOG.md` and `decisions/DEC-106-…md`. Anything
  else is a deviation; say why.

## Guardrails
- ⛔ No version bump, no release and no tag. 0.8.2 is cut by release prep; the tag is the
  maintainer's.
- **Every commit uses `git commit -s`.** The DCO check fails any PR with an unsigned commit.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background. When it exits, read a
  direct snapshot at the PR's true head SHA and check every leg, including windows-latest.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no
  `timeout(1)`.
- **The matrix legs run sequentially, each in a fresh `CARGO_TARGET_DIR`:** default,
  `--no-default-features`, `--features webp-lossy`, and `just wasm-test`. Run clippy
  `-D warnings` and `fmt --check` on each native leg. Delete each scratch target dir when you're
  done with it.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids.
- **Budget ~90 exchanges.** If the matrix is green and you are past that, stop and report what
  is left.

## When you finish
1. **Fill `## Build Completion`:**
   - every file from `git diff --name-only origin/main...HEAD`, not from memory;
   - commit 1's SHA, with `git show <commit 1> -- src/` empty;
   - the refactor commit's SHA, with FT-6 green at it;
   - the RED/GREEN baseline, test by test, with `main`'s exit codes;
   - each mutation's flips (M1–M10 and the two golden mutations), path by path;
   - the matrix legs, wasm included;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-143 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines
     that share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
     **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
     output under-counts about 11×.
   - Price each component at the model that actually ran (read `.message.model`), never at a flat
     rate: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and
     cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance and open the PR.**
   - Run `just advance-cycle SPEC-143 verify`.
   - Open the PR **as a draft**: `gh pr create --draft …`. ⛔ A draft cannot be merged. It stays
     a draft until a separate verify session approves it and the orchestrator marks it ready.
     Never mark it ready yourself.
   - Do **not** edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `fix(watermark): refuse a mark that draws nothing (SPEC-143)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-109 (new, from design), DEC-106 (one line), DEC-104, DEC-102,
       DEC-015;
     - Constraints checked;
     - New decisions: none beyond DEC-109;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-143 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short:
- commit 1's SHA, and that it touches no `src/`; the refactor commit, with FT-6 green at it;
- AC completion (14 rows);
- the RED/GREEN baseline confirmed;
- each mutation's flips, path by path;
- the matrix result, wasm included;
- files changed vs the spec's list;
- any design call that turned out wrong;
- any out-of-spec finding;
- your measured cost.
