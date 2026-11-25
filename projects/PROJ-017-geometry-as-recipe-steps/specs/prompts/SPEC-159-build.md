# SPEC-159 — BUILD prompt: `pad` with a transparent fill, and the no-alpha output rule

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the change in code terms (`alpha.rs`, `Operation::introduced_alpha`, `Pipeline::unstorable_alpha`,
`parse_fill_color`, `CliError::UnstorableAlpha`, seven call sites), every message verbatim, a byte
pin recorded from `main`, 22 ACs, 21 failing tests (90 unit cells, 32 integration rows) and an
18-row mutation table, almost all driven at design on a prototype. The code is small; most of the
work is the tables, the four surfaces and the doc sentences. Be thorough across every row:
Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-159-pad-transparent -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. Keep it plain text with short lists; a wide box-drawn table can garble when
copied out of the terminal. `edit --pad`/`--trim` (SPEC-160) follows this spec and closes STAGE-058.

## What you are building
- **`src/operation/alpha.rs`** (new): `AlphaCapacity`, `alpha_capacity`, `stores`,
  `IntroducedAlpha`, `UnstorableAlpha` with `Display` (the messages, verbatim).
- **`Operation::introduced_alpha`** (default `None`) and **`parse_fill_color`** in
  `src/operation/mod.rs`; `transparent` (lower case only) → `[0,0,0,0]`, else `parse_color`.
  ⛔ **Don't touch `parse_color`.** SPEC-160 fixes its sign bug.
- **`pad`**: a 4-channel fill, the alpha widening arms, the fill's alpha written (`a`, `wide(a)`,
  `unit(a)`), the load refusal removed. **Never blend**: copy the fill value.
- **`trim`**: reads its colour through `parse_fill_color`, and prints the keyword as written.
- **`Pipeline::introduces_alpha` and `Pipeline::unstorable_alpha`** (`src/pipeline/mod.rs`).
- **`CliError::UnstorableAlpha`**, exit 4.
- **The seven call sites** as the spec's "The design" numbers them. Two matter most:
  - **site 1** moves `output_format_for` before `pipeline.run` in single-input `apply` (a bad
    `--format` plus a 0-px border goes from exit 1 to exit 4: AC-17);
  - **site 7** is the design's correction to the frame: the auto-decided format read only the
    source's alpha, so a JPEG photo padded with `transparent` came out a **JPEG with a black border,
    exit 0**. `has_alpha` ORs `pipeline.introduces_alpha()`.
- **DEC-115 is already on `main`** (written at design). Don't edit it; if the code can't follow it,
  stop and report.
- **Tests and docs** as the spec lists them.

The spec's "Build file list" (in Implementation Context) is the complete file list. Anything outside
it is a deviation, so say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `748283c` is in your history;
- `decisions/DEC-115-a-step-whose-alpha-the-output-format-cannot-store-is-refused-before-the-pipeline-runs.md`
  exists.

If any check fails, stop and report.

⚠ **Other open work touches nearby lines.** The spec's "Overlap with other open work" lists it from
the PRs' file lists: #240 (`src/cli/ops.rs`), #237 (`CHANGELOG.md`), #226 (`src/cli/mod.rs`, near the
`CliError` arms). PROJ-011's SPEC-154 to SPEC-158 may open PRs on `src/cli/build.rs`,
`src/cli/common.rs` and `src/cli/ops.rs` while you build. After every rebase, run
`gh pr list --state open --json number,title,files` and compare; name any new overlap in Build
Completion. If a rebase conflicts in a file outside the spec's list, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-159-pad-with-a-transparent-fill-and-the-no-alpha-output-rule.md`.
   Read it in full: `## Frame` (the why, the format table, the maintainer rulings), `## Design`
   (especially "Design corrections to the frame" and "The design"), every AC, the Failing Tests,
   `## Implementation Context` (overlap, build file list, docs, the mutation table) and
   `## Notes for the Implementer`.
   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-115 (this spec's rule), DEC-095, DEC-102, DEC-110, DEC-015. Read them yourself.
3. The code the spec cites at `file:line`, before you change it.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 5, 10, 12, 13 and 14). These bind PROJ-017. Also `/guidance/constraints.yaml`.
5. SPEC-153 in `specs/done/`: its Build Completion, verify findings and ship reflection. Its only
   defects were two doc sentences the build wrote and never drove.

## Maintainer rulings (settled — do not re-open)
1. A fill alpha the output format can't store is **refused per input, exit 4**, naming the step and
   the format. Flattening is ruled out.
2. **GIF:** alpha `00` and `ff` allowed; `01`–`fe` refused.
3. **`trim` accepts `transparent`** through the shared helper.
4. **Source alpha is out:** an RGBA source written to JPEG still drops alpha silently (exit 0), and
   the docs say so.
5. The check is **static** (recipe × format), never a scan of the output pixels.
6. No message, help line or comment names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. Stages 2–6 are the spec's five checkpoints.
1. **The byte pin first, on unchanged code.** Write FT-6 and run it on `main`'s code **before any
   source edit**: its four literals must pass there. Quote that run. Then write the rest of the
   integration tests and run them on `main`; quote the baseline (the translucent rows fail at load
   with "not supported yet"; quote it once). Open the PR **as a draft** with this commit.
2. **`alpha.rs`, `introduced_alpha`, `parse_fill_color`**, FT-1 to FT-3 green.
3. **`pad.rs` and `trim.rs`**, FT-4 to FT-7, FT-18 and FT-19 green. FT-6 stays green.
4. **The seven call sites and `CliError::UnstorableAlpha`**, FT-8 to FT-17 and FT-20 green.
   - Expected messages, alphas and hashes come from the spec's literals, **never** from
     `UnstorableAlpha`'s `Display` or the run, which are the code under test.
   - Every refusal test asserts the message text and that nothing was written, not only the exit
     code.
   - **`build` tests use a fresh project directory each**: a stale `.crustyimg` cache replays old
     bytes (design hit this).
5. **wasm**, FT-21 (`alphaRows()` in `tests/ddphotos_parity.mjs`). Run `just wasm-build` and
   `node tests/ddphotos_parity.mjs` locally: `just check` covers neither (trap 5). Record the brotli
   size against `WASM_BROTLI_BASELINE`; design measured +0.19%, inside the band.
6. **Docs**, each AC-22 row at the `file:line` the spec gives. Line numbers may have shifted, so find
   each place **by content**. **Drive every doc sentence that states behaviour** on your release
   build (a format refused or allowed, a colour type after padding, the GIF threshold, the source-alpha
   line, the AVIF "approximately"), and quote each drive in Build Completion. CHANGELOG lines go
   where the overlap section says for #237.
7. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. The spec quotes each
   mutant's exact edit: **apply that edit, not your own version of it**. If the quoted line doesn't
   exist in your code, make the closest edit, quote what you applied, and say why. For each row:
   - apply it, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected, `$?` read; record which tests go
     red against the spec's red/green lists. The design excluded one prototype artefact (trim's row
     count after T19 was deleted): your real tests have no such artefact, so record what actually
     reds;
   - the cells marked *reasoned* are driven now: **M3's FT-11**, **M9's single-input row** and
     **M11** (wasm: run the parity harness against a wasm build of the mutant);
   - **M18** is "all green" by design (no `run_pixel_op` caller can carry a pad without a forced
     format): drive it and say whether that still holds;
   - restore, rebuild, and show it green.

   Paste each mutation's red output, trimmed to the lines that name tests, into Build Completion.
   Where a cell is wrong, correct it in Build Completion and say so; don't edit the design's table.
8. **Clean matrix:** default, `--no-default-features`, and `--features webp-lossy`, one after
   another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and `fmt --check` on each.
   The lean build must compile `alpha.rs` (it uses only `image::ImageFormat`).

## The things most likely to go wrong
- **Missing a surface.** There are six checks plus the decided-path line. M6 to M11 each red only
  their own surface's rows, so a missing call site shows up only if its test exists and asserts the
  message.
- **The batch pre-check falling through to the per-input path.** With the pre-check removed, the
  batch still fails, with exit 6 (M8). FT-10 must assert exit 4 **and** that no out-dir exists.
- **A pixel check instead of a static one.** It compiles and passes everything except the
  already-at-aspect row (M12).
- **`× 256` for 16-bit alpha** (M13): `0x80` must be 32896, not 32768.
- **Grey widening:** a grey translucent fill on L8 gives LA8, not RGBA8 (M14).
- **Blending the fill** instead of copying it, as `watermark` does.
- **The decided path** (site 7): without it, `apply` and `build` with `optimize` write a JPEG with a
  black border and exit 0 (M17; FT-13 only).
- **Retyped messages.** Copy every message from the spec byte for byte; alpha prints in hex (`80`).
- **A green leg on one OS isn't the matrix.** Read every CI leg, including lean and wasm.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile format,
  `Cargo.toml` or `Cargo.lock` (AC-22 says why). Paste `git diff --stat origin/main...HEAD` for those
  paths.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, or stop and report.
- **Rebase onto `main` before each push** and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each
  commit on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no `timeout(1)`.
  `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done. Write scratch files in your session's scratchpad, never
  `/tmp`, and don't leave PR-body files behind.
- **Budget ~110 exchanges.** If you're past budget with the matrix green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed; say whether the count includes the spec file;
   - FT-6 on `main` and after, both runs quoted;
   - every AC, one line each: met, or not and why;
   - each doc sentence's drive, quoted;
   - every mutation row, with the edit you applied, the tests that went red and those that stayed
     green, against the spec's table, with every mismatch called out and every reasoned cell
     marked confirmed or corrected;
   - the wasm size, the matrix legs and the CI legs;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-159 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any
     `subagents/` transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Measure at your last turn, and append a `build` entry to `cost.sessions` with `tokens_total`, a
     `tokens_breakdown`, `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-159 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `feat(pad): transparent and translucent fills, refused where the output can't store them (SPEC-159)`.
   - Body per AGENTS §13:
     - Summary, including that a JPEG source through `[pad transparent, optimize]` now writes WebP
       with alpha instead of a JPEG with a black border;
     - Spec metadata;
     - Decisions referenced: DEC-015, DEC-095, DEC-102, DEC-110;
     - Constraints checked;
     - New decisions: DEC-115 (on `main` since design);
     - Breaking: none; recipes that loaded before produce the same bytes;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-159 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.
Then, briefly, as plain lists:
- FT-6 on `main` and after;
- AC completion (as a count, plus any row not met);
- each doc sentence's drive, one line each;
- each mutation's flips against the table, the edit you applied where it differs from the quoted
  one, and every cell that turned out wrong;
- the wasm size, the matrix result and the CI legs, including lean and wasm;
- the overlap check after your last rebase;
- files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on
  its own line.
