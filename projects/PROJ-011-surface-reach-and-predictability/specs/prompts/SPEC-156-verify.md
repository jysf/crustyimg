# SPEC-156 — VERIFY prompt: `meta strip` keeps the Adobe APP14 colour-transform marker

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-156. Your job is to find what the build's own evidence can't show. Six ticked
ACs and a matching mutation table are claims until you drive them. The tests decode with the same
`image`/`zune-jpeg` stack the code ships with, and they cover the four APP14 shapes someone thought
to build.

```
just wt verify/spec-156 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-156-strip-keeps-app14:refs/remotes/origin/feat/spec-156-strip-keeps-app14
git checkout --detach origin/feat/spec-156-strip-keeps-app14
```

When this was written, draft PR #269's head was `b0b870d`. The orchestrator read its check-runs: 18
passed, 8 skipped (release-only), none failed or pending. Record the SHA you verify. The maintainer
will paste your final message back to the orchestrator, so it must stand on its own. **Write it as
plain lists, not box-drawn tables.**

## What the build claims
- **Fixture:** all three sha256 hashes equal the spec's, made with libjpeg-turbo 3.2.0 `cjpeg`.
  djpeg reads (8,8) as `[128,128,191]` before and after `meta strip`, and the strip output is
  byte-identical to the input.
- **Baseline:** with the helpers added and `strip_all` still `main`'s loop (the M1 shape), FT-1 to
  FT-4 and FT-7 to FT-9 were red, each with the spec's message. FT-5 and FT-6 passed on `main`, as
  predicted.
- **ACs:** 6 met. AC-6, the docs, is left to you.
- **Mutations:** every row matches the table, with no wrong cell. M3b and M8, which the design drove
  only on the lib target, are red on FT-4 and FT-7 alone across every target. M6's run finished,
  doc-tests included. Pass counts are one higher than the design's because FT-4 is in the baseline.
- **Matrix:** default 1357/0, `--no-default-features` 1293/0, `--features webp-lossy` 1361/0. wasm32
  `--lib` builds with only the existing `MANIFEST_DROPPED_WARNING` warning.
- **Deviation:** the `with_segments` test helper uses `enumerate`, because clippy flagged
  `explicit_counter_loop`.
- **DECs:** neither DEC-003 nor DEC-029 was edited. Their text is applied at ship.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no new PR. Report; don't fix. Any mutation you
drive happens in a scratch worktree with its own `CARGO_TARGET_DIR` under your scratchpad, and both
are gone before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-156-meta-strip-keeps-the-adobe-app14-colour-transform-marker.md`.
   Read these sections:
   - `## Frame` with the maintainer rulings;
   - `## Design`: the five corrections and "Driven on `main` and on the prototype";
   - the ACs, Failing Tests and the mutation table;
   - `## Implementation Context`: the code, the fixture recipe, the docs, the CHANGELOG, "What does
     not change", the DEC text for ship, out of scope;
   - Build Completion.
2. The PR's diff of `src/metadata/mod.rs`, `tests/metadata.rs` and the fixture README.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for", especially traps 12, 13 and 14.
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. Only Adobe's APP14 is kept; any other APP14 is still stripped. Design refined "starts with
   `Adobe\0`" to "starts with `Adobe` and is at least 12 bytes", and the orchestrator accepted that.
2. The orientation and ICC warning in strip's docs is a separate STAGE-063 entry, not this spec. The
   docs say only what strip now keeps.
3. No message, comment or doc line names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - Compare `git diff --name-only origin/main...HEAD` with `## Outputs`: 7 files plus the spec.
   - Confirm that `Cargo.*` and `decisions/` didn't change.
   - Check that every commit is signed off and that the PR merges cleanly onto current `main`.
   - Run `git merge-tree` against SPEC-155's PR #268 (`feat/spec-155-atomic-writes`) and against
     PR #240. Say whether each conflicts, and in which hunks.
2. **The code against the spec.** Extract the spec's two Rust blocks yourself and compare them with
   the PR, ignoring rustfmt's wrapping. Then:
   - confirm `run_strip`, `has_manifest`, the PNG arm and the other `meta` verbs are untouched;
   - check FT-8's reconstruction calls `strip_keeps_jpeg_segment`, not a copy of it.
3. **The fixture, rebuilt independently.** Follow `tests/fixtures/metadata/README.md` step by step
   with your own `cjpeg`, in your scratchpad:
   - compare the three hashes;
   - read the six id bytes with `xxd`;
   - decode with `djpeg` before and after a PR-head `meta strip`, and quote pixel (8,8).

   If your `cjpeg` isn't 3.2.0 and a hash differs, say so, and check the offsets instead.
4. **Drive more JPEGs than the tests have.** Build `main` and the PR head `--release`, each in its own
   fresh `CARGO_TARGET_DIR`. Check every shape below with an independent decoder (`djpeg`, and
   ImageMagick's `compare -metric AE`), not crustyimg's decode:
   - **every fixture JPEG with an APP14.** Find them with your own header walk over
     `tests/fixtures/**/*.jpg`. Say what each holds, and whether the PR's strip renders the same as
     the input while `main`'s doesn't;
   - **two APP14s in one file**, an Adobe one and a `Ducky` one, in both orders;
   - **an Adobe APP14 after DQT** rather than right after SOI;
   - **a progressive Adobe JPEG**, from `cjpeg -progressive` with CMYK or RGB input;
   - **an Adobe APP14 whose transform byte is out of range** (say `0x05`), 12 bytes long: what djpeg
     does with it, and whether keeping it is right.

   For each, give the header markers before and after on both binaries, and the AE between the input
   and each output.
5. **The marker check is load-bearing, and nothing tests it.** `is_adobe_colour_transform` checks
   `marker == 0xEE`. Photoshop writes an APP13 that starts with `Adobe_CM`, which is at least 12
   bytes.
   - Drive a mutant without the marker check, and say which test, if any, turns red.
   - Then build a JPEG with an `Adobe_CM…` APP13, and say what the PR head's strip does with it.

   A mutant no row sees is a finding. Classify it **assert**.
6. **Re-drive the mutations the claims rest on.** Each alone, in a scratch worktree, with a clean
   rebuild, a full `cargo test --no-fail-fast` and the red set read off the run:
   - M1, the negative control;
   - M3 and M3b, the signature rules;
   - M5a and M5b, the length floor;
   - T14 (`main`'s `src/metadata/mod.rs` restored). Confirm FT-9 is the only survivor.

   Compare each with the table and Build Completion.
7. **Callers.** Run `/usr/bin/grep -rn "strip_all" src/ tests/`.
   - Confirm `meta strip` is the only production caller.
   - Confirm no pixel-lane, recipe or wasm path depends on the old "every APP14 goes" behaviour.
   - Run `lint` on a folder of stripped outputs from both binaries, and give the findings.
8. **Docs, CHANGELOG and comments.**
   - `diff` each against the spec's exact text.
   - Drive each sentence that states behaviour, for example "keeps JFIF and the Adobe
     colour-transform segment" and "the image decodes the same". Quote step 4's rows as evidence.
   - Check that the CHANGELOG's size claim (+16 bytes on the common form) matches an `info` reading.
   - Run `/usr/bin/grep -nE '(SPEC|STAGE|DEC|PROJ)-[0-9]'` over the changed lines of every non-spec
     file, and classify each hit.
9. **CI.** Run `gh pr checks 269` once at the head, and say whether every leg is green, including
   Windows and lean.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or spec line that misstates something the code already does;
- **assert:** a missing or loose test row, where the behaviour is already right;
- **claim:** a fix that would change what a doc or test claims is true, or what the code does.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec, say whether it was driven, by whom, and over
what range. Examples:
- the 12-byte floor;
- "libjpeg ignores a shorter one";
- "no other fixture's version byte is non-zero";
- "no caller strips after a re-encode";
- "+16 bytes on the common form".

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. zsh also reads `$B:r…` as a
  modifier, so brace it (`${B}:`). macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Redirect `cargo test` stdout to a file. An interactive terminal fails
  `display_sink_refuses_non_tty`.
- Every build uses a fresh `CARGO_TARGET_DIR` under your scratchpad, one per binary or mutant, never
  shared with another session. Run `cargo clean -p crustyimg` between mutants.
- For wasm32, use the rustup `cargo`; a bare `cargo` finds Homebrew's `rustc`.
- Never poll CI and never write your own wait loop (AGENTS §16 habit 6).
- Write scratch files only in your session's scratchpad, never `/tmp` or the system temp dir.
  Leave the tree clean.
- **Budget ~40 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-156 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session writes
  them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #269 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #269 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–9, one or two lines each, with evidence;
- the punch items, numbered. Give each its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- out-of-spec findings, each with where it belongs (STAGE-063 or `guidance/ideas.yaml`), and
  whether each one is pre-existing;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
