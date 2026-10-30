# SPEC-141 — BUILD prompt: pin the output of every existing recipe (the PROJ-017 guard)

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design
gives the fixture generator, the golden line format, the failure messages, and a seven-row
mutation table already partly driven, so the work is mechanical. Be thorough across all 48 rows,
and let CI decide the one open question.

```
just wt test/spec-141-recipe-output-guard -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that
message must stand on its own. This spec is line 1 of PROJ-017's build order. Every later
geometry spec uses this guard to show it changed no existing output.

## What you are building
- `tests/recipe_corpus_output.rs`. It runs all 14 corpus and bundled recipes on 3 fixtures it
  builds itself, and pins what each produced: width, height, colour type, bit depth and a SHA-256
  of the pixels before encoding (42 rows). It also pins the encoded bytes for PNG and lossless WebP
  (6 rows), for **48 rows** in total, asserted as a literal.
- Checks that turn a guard that ran nothing red, a seam check through the real binary, a format
  check for the bundled `optimize` choice, a completeness check on the golden files, and a bless
  switch that refuses in CI.
- One new corpus recipe in the ddphotos page's shape (`v2_ddphotos_cli_shape.toml`) with its
  `.golden`.
- `tests/fixtures/recipe_corpus/README.md` (new), stating the rules.
- **No `src/` change** (AC-12). Mutations touch `src/` temporarily and are never committed.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `6cfdbf9` is in your history;
- `### Maintainer rulings (2026-10-30)` lists three rulings.

If any check fails, stop and report.

⚠ **SPEC-143** (PROJ-011, "a watermark that draws nothing is refused") is in flight and touches
the watermark op. If it lands on `main` before you open your PR, rebase over it.
- The mutation table's line numbers (`src/operation/mod.rs:584` and `:1629`) may shift. Find each
  mutation **by its content**, and record the line you actually used.
- If SPEC-143 changes any corpus row's output, that's a finding for SPEC-143. Report it and don't
  re-bless around it.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-141-no-existing-recipe-output-changes.md`.
   Read in full: `## Frame` and its rulings, AC-1 to AC-13, the Failing Tests (FT-1 to FT-7, plus
   FT-8 under outcome (ii)), the mutation table M1–M7, `## Implementation Context` and
   `## Notes for the Implementer`. **The design calls are settled.** If one turns out wrong, say so
   in Build Completion and in your report; don't quietly re-decide it.
2. **DECs:** DEC-058, DEC-059, DEC-095, DEC-102. Read them yourself; don't trust a summary.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 11 and 12 especially). These bind PROJ-017. Also read
   `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
1. If the CI legs give different pixels, pin **exactly per OS and architecture**. Never use a
   tolerance.
2. No byte pins for AVIF or the `optimize` search (DEC-059 Option A).
3. The ddphotos-shaped recipe joins the corpus now.

## Order
1. **The corpus recipe first.** Add `v2_ddphotos_cli_shape.toml` exactly as AC-1 gives it, and
   bump the corpus count in `tests/recipe_round_trip.rs` from 13 to 14. Then run the existing
   round-trip bless (`CRUSTYIMG_BLESS`, about `:374`). **`git status` must show only the one new
   `.golden`.** If any existing `.golden` changed, stop: something else moved.
2. **The test, then the Mac goldens.** Write `recipe_corpus_output.rs` as the spec words it, with
   the fixture generator copied verbatim. Confirm the three fixture SHA-256 prefixes match AC-2
   (`94daa4af` / `7df0cef0` / `75dfcd86`) **before** blessing anything: a wrong fixture makes every
   golden wrong. Bless, then run the test green.
3. **Checkpoint 1: a draft PR, early.** Commit with `git commit -s`, push, and open the PR **as a
   draft**. Then start `gh pr checks <PR> --watch` **in the background**. Never poll. When it
   exits, read a direct snapshot at the PR's true head SHA, and read every leg.
   - **(i) Every leg green:** one `.output` per recipe stands (14 files). Go on.
   - **(ii) Some legs red on a pixel or byte row:** switch to the per-target layout the spec
     specifies (`<stem>.<os>-<arch>.output`, 42 files, FT-8). Copy each target's goldens from the
     failing log's marker block (`gh run view <run> --log-failed`). They count only once a
     **second** CI run is green on every leg.
   - **Two legs on the same target disagree** (for example two Linux legs with different
     features): stop and report BLOCKED. Per-feature goldens weren't designed.

   Record each leg's result and this test's wall time on it.
4. **Mutations M1–M7, one at a time** (HANDOFF trap 12). For each one:
   - apply it at the location the spec names, and check `git diff` is non-empty;
   - rebuild **clean** (a fresh `CARGO_TARGET_DIR`, or `cargo clean -p crustyimg`), because a
     reverted source does not rebuild the binary;
   - run the tests, and record exactly which rows go red against the spec's red/green lists;
   - restore, rebuild, and show it green.

   **M3 re-blesses.** Afterwards, restore the goldens with `git checkout -- tests/fixtures/` and
   show `git status` clean. **M5** must show the golden directory's hashes identical before and
   after the refused bless. Paste each mutation's red output into Build Completion.
5. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`. Run them one
   after another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and
   `fmt --check` on each. AC-7's format rows compile only with `avif` on and `webp-lossy` off.
   Show that the lean leg compiles them out rather than failing them.
6. **Prove no `src/` change:** `git diff --stat origin/main...HEAD -- src/` must be empty. Paste
   it.

## The things most likely to go wrong
- **A golden blessed from a broken fixture passes forever.** Check the fixture hashes (AC-2)
  before the first bless, and re-check them on CI.
- **A missing golden must fail, never skip** (trap 11). This covers a new target under (ii) and a
  recipe without an `.output`. libtest hides stderr for passing tests, so a printed skip is
  invisible.
- **CRLF on Windows.** Goldens check out with CRLF there. AC-8 normalises line endings on read.
  Don't "fix" this with `.gitattributes` instead.
- **The seam row (AC-6) runs the real binary.** Use the binary Cargo builds for this test
  (`CARGO_BIN_EXE_crustyimg`), never one on `PATH`, because Homebrew's 0.8.x binary reports the
  same version.
- **Format rows are slow** (about 32 s for the 3 rows in the dev profile). Don't add rows to
  them.
- **Stay in your files:** `tests/recipe_corpus_output.rs`, `tests/recipe_round_trip.rs`,
  `tests/fixtures/recipe_corpus/` (the new `.toml`, `.golden` and `.output` files and the README),
  and possibly `tests/common/mod.rs`. Anything else is a deviation, so say why.

## Guardrails
- ⛔ No `src/` change committed. No version bump, no release, no tag. No change to `Cargo.toml` or
  `Cargo.lock`.
- **Every commit uses `git commit -s`.** The DCO check fails a PR with an unsigned commit.
- **Never poll CI.** Run `gh pr checks --watch` in the background only.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`.
  zsh does not word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no
  `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Delete each scratch target dir when you're done with it.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids.
- **Budget ~90 exchanges.** Outcome (ii) costs a second CI round. If you're past budget with the
  matrix green, stop and report what's left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory;
   - the empty `src/` diff;
   - the outcome, (i) or (ii), with each leg's result and wall time;
   - M1–M7, each with its applied line, its rows that went red, and its rows that stayed green,
     against the spec's table;
   - the matrix legs;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-141 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-141 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `test(recipes): pin the output of every existing recipe (SPEC-141)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-058, DEC-059, DEC-095, DEC-102;
     - Constraints checked;
     - New decisions: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-141 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.
Then, briefly:
- the outcome, (i) or (ii), and each leg's result and wall time;
- AC completion (13 rows);
- the fixture hashes as checked on CI;
- each mutation's flips against the table, and any row that didn't match;
- the matrix result, and the empty `src/` diff;
- files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost.
