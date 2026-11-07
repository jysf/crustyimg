# SPEC-152 — BUILD prompt: `trim` as a recipe step

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design
gives the op in code terms, a 36-row match-rule table, every refusal's exact message, 65 ACs and a
21-row mutation table, every row driven at design on a prototype. The op is one file of about 200
lines; most of the work is sweeps across colour types, refusals, match rows and doc surfaces. Be
thorough across every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-152-trim -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that
message must stand on its own. Keep it plain text with short lists; a wide box-drawn table can
garble when copied out of the terminal. This spec is line 6 of PROJ-017's build order; the `edit`
flags spec (line 8) follows it.

## What you are building
- **`src/operation/trim.rs`:** the `trim` op. Keys `color` (optional, any alpha) and `tolerance`
  (an integer 0..=255, default 0). With no `color`, the reference is the top-left pixel in the
  image's own samples. Each side is searched on its own; the crop is `crop_imm` of the input, kept
  through `with_pixels` (DEC-113). A whole match fails at apply; nothing to trim returns the input
  unchanged. It never uses the refusal hook.
- **`ParamKind::UInt` gains `max: Option<u32>`** in `src/operation/registry.rs`; the three existing
  `UInt` rows in `src/operation/mod.rs` get `max: None` with their messages unchanged. Registration
  and doc lists in `registry.rs`.
- **The `parse_color` fix** in `src/text/mod.rs`: a non-ASCII colour is refused with the existing
  message instead of panicking (exit 101 today in `pad`, `watermark` and `watermark --color`).
- **Tests:** unit tests in `trim.rs`, a new `tests/trim.rs`, registry and `text` tests, the
  fold-in asserts, two wasm rows in `tests/wasm_roundtrip.rs`, and two corpus recipes
  (`v2_trim_corner`, `v2_trim_tolerance`) with their literal bumps.
- **`scripts/wasm-ci-check.sh`:** both constants and the self-test fixtures.
- **The fold-in:** resize's cap message text pinned (AC-54).
- **The docs surfaces** the spec lists, and `CHANGELOG.md` `## [Unreleased]` entries under
  `### Added`, `### Fixed` and `### Breaking`.

The spec's `## Outputs` is the complete file list. Anything outside it is a deviation, so say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `934c920` is in your history;
- `ls decisions/` shows no DEC written for this spec (none is expected).

If any check fails, stop and report.

⚠ **SPEC-150 (PROJ-011, PR #257) is in flight at the same time.** It is a draft at its punch-list
cycle and may merge while you build. The spec's "Files shared with SPEC-150" lists the overlap
(`docs/api-contract.md`, `CHANGELOG.md`). **One overlap the design missed:** PR #257 also edits
`src/cli/ops.rs`, which reads the operation param tables. After every rebase, run
`/usr/bin/grep -rn 'UInt {' src tests` and confirm every `UInt` literal has a `max`; if a SPEC-150
line builds `UInt { min }`, add `max: None` and say so in Build Completion. If a rebase conflicts in
a file outside the spec's list, stop and report.

**Re-read `FULL_COUNT` and `SLIM_COUNT` on `main` after your last rebase.** The spec expects
53/51 → 55/53. If `main` has moved, add 2 to whatever it says and say so in Build Completion.

## Read in order, before any code
1. **The spec:** `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-152-trim-as-a-recipe-step.md`.
   Read it in full: `## Frame` (for the why), `## Design` (especially "Design corrections to the
   frame", "The op, in code terms", "`ParamKind::UInt` gains `max`", "The match rule, as a table a
   test can check" and "Exact messages"), AC-1 to AC-65, the Failing Tests, the mutation table,
   `## Implementation Context` and `## Notes for the Implementer`.
   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-003, DEC-095, DEC-102, DEC-110, DEC-112 and DEC-113. Read them yourself.
3. `src/operation/pad.rs` and `tests/pad.rs`: the shape to mirror (table, `from_params`, the colour
   message, the no-op, the test helpers and the metadata row).
4. `tests/fixtures/recipe_corpus/README.md`: the bless procedure. Follow it exactly.
5. `scripts/wasm-ci-check.sh`, the constants **and** `self_test`.
6. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 9, 10, 12, 13 and 14). These bind PROJ-017. Also read
   `/guidance/constraints.yaml`.
7. SPEC-151 in `specs/done/`, the last op added this way: its Build Completion and ship reflection.

## Maintainer rulings (settled — do not re-open)
1. **No `color` means the top-left pixel**, read in the image's own samples, and all four sides
   are trimmed against it.
2. **`tolerance` is an integer in 8-bit steps,** checked per channel, inclusive (`|a − b| <= t`):
   ×257 at 16 bits; a float sample is read as its nearest 16-bit value (`round(s × 65535)`,
   clamped to 0..=65535; NaN never matches). `8.0` is refused.
3. **Alpha 0 matches alpha 0** whatever colour is hidden; otherwise all four channels are
   compared. Grey `L` compares as `(L, L, L)`.
4. **One colour; each side searched on its own** to the bounding box.
5. **A whole match fails at apply** (exit 1, or exit 6 in a batch, nothing written for that
   input). Nothing to trim is a no-op.
6. **`color` is optional and may be translucent.** No `transparent` keyword.
7. **The `parse_color` fix stays in this spec** (the design's correction 6). AC-12, M13 and T23
   stay.
8. No message or doc line names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. These are the spec's six checkpoints, plus the mutations and the matrix.
1. **Red tests first.** Write the integration, registry, `text` and fold-in tests the spec lists.
   (The unit tests in `trim.rs` can't compile before the module exists; they land with stage 2.)
   Run them against `main`'s code and quote the baseline: the `tests/trim.rs` rows fail with
   `unknown operation 'trim'`, and the non-ASCII rows panic. Open the PR **as a draft** with this
   commit.
2. **The op, green locally.**
   - Implement `trim`, the `UInt` `max` and the registration as the spec words them.
   - Every message is **byte-for-byte** the spec's text: copy it, never retype it. That includes
     W1 and W2 (no hex in the default's message) and the `internal:` fallback, which must differ
     from the table's text (M15).
   - Expected boxes, levels and pixels come from the spec's literals (the match-rule table, the
     levels like 2056/2057 and 65535), **never** from the matcher or the scan, which are the code
     under test.
   - The float path is exactly `f64::from(s) × 65535.0` → `.round()` → `.clamp(0.0, 65535.0)` →
     `as i64`, with NaN checked first. Port the design's 2,357,504-case search as AC-3's test.
   - No `unwrap`/`expect` in `trim.rs` outside tests; use `unwrap_or` for the side searches.
   - Check the metadata test's control B before you trust its trim rows.
3. **The corpus, for your own target.**
   - Add the two `.toml` files and write their `.golden` files by the README.
   - Raise every literal in one commit, as the spec gives them: `RECIPES` 22, `EXPECTED_DIMS` 22,
     `ROWS` 72, `GOLDEN_LINES` 75, the `60` → 66, `CORPUS` 19, `pairs.len()` 22, and the doc
     comments that state the counts.
   - Expect dims `2652x1782` / `1782x2652` / `2652x1782` for both recipes, and identical pixel lines
     between the two recipes (correction 3).
   - Bless your own target only.
   - ⛔ **`git status` must show only added files plus the literal lines.** If any existing
     `.output` or `.golden` is modified, stop: that's AC-49 failing, and it means something else
     moved. Report it; don't re-bless around it.
   - Push. Start `gh pr checks <PR> --watch` **in the background**, once. When it exits, copy the
     other two targets' `.output` files from the failing log's `----- BEGIN … -----` blocks
     (`gh run view <run> --log-failed`). They count only once a **second** CI run is green on every
     leg. Then `cmp` the three per-target files for each recipe, and quote the result (AC-48).
4. **wasm and the script, in one commit.**
   - Add the two wasm tests (AC-51, AC-52).
   - Set the two constants (55/53, or main's numbers + 2), **and** move the `--self-test` fixtures
     to match in the same commit (AC-53). Each row keeps its meaning.
   - Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` (expect 18/18) and
     `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh workflow` (expect 16/0), and
     `./scripts/validate-frontmatter.sh`. Quote each result line.
   - Run `just wasm-check`, `just wasm-test` and `just wasm-test-slim` locally if you can, and quote
     each result line. Then read CI's two wasm count steps.
5. **The `parse_color` fix and the resize fold-in.** AC-12 (trim, pad, watermark and the binary:
   `invalid color 'aéaaa'`, exit 2 on the CLI flag) and AC-54, each from the spec.
6. **The docs.** Make each docs AC (AC-55 to AC-62) true at the `file:line` the spec gives. Line
   numbers may have shifted, so find each place **by content**. `tests/docs_ops.rs` must resolve
   every new `op = "trim"` example. The CHANGELOG `### Breaking` line names the `UInt` change.
7. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. Every row was driven
   at design on a prototype, so a mismatch on the real code is a finding. For each row:
   - apply it at the location the spec names, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected, `$?` read; record exactly which
     tests go red against the spec's red/green lists, including the tests the spec marks
     *reasoned* (now that they exist, they are driven);
   - restore, rebuild, and show it green.

   **M10** edits a corpus recipe in the scratch copy only. **M12** restores `main`'s
   `with_builtins` verbatim, and every `tests/trim.rs` integration row must go red. **M9** runs on
   the script, not the suite. Paste each mutation's red output, trimmed to the lines that name
   tests, into Build Completion. Where a cell is wrong, correct the table in Build Completion and
   say so; don't edit the design's table.
8. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`. Run them one
   after another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check`
   on each. `trim` has no feature gate, so the lean leg must run its tests, not skip them. Also
   record CI's wasm bundle-size delta.

## The things most likely to go wrong
- **`<< 8` instead of `× 257`.** `#ff` is 65535 at 16 bits, not 65280, and `t = 8` is 2056 levels,
  not 2048 (M1, M1b). Only the `rgba16` corpus row catches M1, by dims.
- **Float maths in `/ 255`.** It misses `f32(1/255)` at `t = 1` (M3). The 8-bit-level matrix does
  **not** catch this; the match-rule table does. Don't drop its float rows.
- **NaN read as 0, or floats left unclamped.** An `as` cast turns NaN into 0 (black); `1.5` must
  read as white (M17, M18).
- **A softer alpha rule.** Alpha 3 with a bright hidden colour is **kept** at `#00000000`,
  `tolerance = 8`. That's ruling 3 as written; the softer rule is out of scope.
- **A hex colour in the default's whole-match message.** W1 names "its top-left pixel"; W2 prints
  the colour as written, case kept.
- **The top-left read from the wrong corner** (M5), or one side's search reusing another's (M6).
  Each side is its own search.
- **A no-op that isn't one.** Nothing to trim returns the same `Image`, type and bytes.
- **The wasm counts without the self-test.** Both move in one commit (stage 4).
- **A `UInt { min }` literal from SPEC-150** after a rebase (Step 0).
- **A green leg on one OS isn't the matrix.** Read every CI leg, including the lean one, the wasm
  job and the bundle-size gate.
- **Stay in the spec's files.** The PROJ-011 orchestrator is keeping clear of them for you.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock`. Paste `git diff --stat origin/main...HEAD` for those paths (AC-64).
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks --watch` once in the background, or stop and report.
- **Rebase onto `main` before each push** (the spec's merge-order rule with SPEC-150), and redo
  Step 0's `UInt` grep and count re-read after each rebase. Don't chase `main` beyond that.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each
  commit on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no `timeout(1)`.
  `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done.
- Comments and user-facing text read behaviour-first, with no spec, stage or DEC ids (AC-63;
  SPEC-150's literal scanner checks this once it lands).
- **Budget ~110 exchanges.** The second CI round for the corpus files is expected. If you're past
  budget with the matrix green, stop and report what's left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed; say whether the count includes the spec file;
   - AC-1 to AC-65, one line each: met, or not and why;
   - every mutation row, with its applied line, the tests that went red and the tests that stayed
     green, against the spec's table, with every mismatch called out and every reasoned cell marked
     confirmed or corrected;
   - the corpus: each CI leg's result, and the `cmp` of the three per-target files;
   - the wasm script's three local checks, the local and CI wasm counts, and the bundle-size delta;
   - the matrix legs;
   - whether SPEC-150 had merged by your last rebase, and the `UInt` grep result;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-152 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Measure at your last turn, and append a `build` entry to `cost.sessions` with `tokens_total`, a
     `tokens_breakdown`, `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-152 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `feat(recipes): trim a uniform border, as a recipe step (SPEC-152)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-003, DEC-095, DEC-102, DEC-110, DEC-112, DEC-113;
     - Constraints checked;
     - New decisions: none;
     - Breaking: `ParamKind::UInt` gains `max`;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-152 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.
Then, briefly, as plain lists:
- AC completion (65 rows, as a count, plus any row not met);
- each mutation's flips against the table, and every cell that turned out wrong;
- the corpus: CI legs and the three-file `cmp`; and confirmation that no existing `.output`
  changed;
- the wasm counts (local and CI), whether main's base had moved, and the self-test result;
- whether SPEC-150 had merged, and the `UInt` grep;
- the matrix result and the bundle-size delta;
- files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost.
