# SPEC-156 — PUNCH LIST prompt: pin the APP14 marker check

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. One **assert** item. The
behaviour is already right, so no re-verify follows. You did not design, build or verify SPEC-156.

```
just wt fix/spec-156-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-156-strip-keeps-app14:refs/remotes/origin/feat/spec-156-strip-keeps-app14
git checkout --detach origin/feat/spec-156-strip-keeps-app14
```
Push to the PR branch with `git push origin HEAD:feat/spec-156-strip-keeps-app14`. Draft PR #269's
head was `b0b870d` when this was written. If it has moved, stop and report.

The maintainer pastes this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify rebuilt the fixture byte for byte with its own `cjpeg`. It drove nine more JPEG shapes through
`djpeg` and ImageMagick: every Adobe shape renders the same after the PR's strip, and `main`'s
strip changes them. It re-drove M1, M3, M3b, M5a, M5b and T14, and every row matched.

It found one gap. `is_adobe_colour_transform` checks `marker == JPEG_ADOBE_MARKER`, and nothing tests
that clause: a mutant that makes it always true (MK) leaves all 1,357 tests green. The risk is real.
Photoshop writes an APP13 that starts with `Adobe_CM` and is at least 12 bytes long. Without the
marker check, strip would keep it.

## The fix (settled, do not re-decide)
1. Add a unit test in `src/metadata/mod.rs`'s test module, next to FT-5 and FT-6, named
   `strip_all_removes_an_adobe_cm_app13` (FT-10):
   - input: `with_segments(jpeg_with_exif(), &[(0xED, b"Adobe_CM\x00\x01\x00\x00\x0c\x00\x00\x00")])`.
     Use the helper's real signature;
   - assert one APP13 before strip, as a precondition, and none after;
   - also assert the APP13 payload is at least `ADOBE_APP14_MIN_LEN` bytes and starts with `Adobe`.
     That way the test can only pass because of the marker check, not the length or the signature.
2. In the spec:
   - add FT-10 to Failing Tests;
   - add it to AC-3's checks, and widen AC-3's wording to "an APP14 that is not Adobe's, too short to
     hold the transform, **or an `Adobe`-prefixed segment under another marker**, is removed";
   - add row **MK** to the mutation table: `marker == JPEG_ADOBE_MARKER` →
     `(marker == JPEG_ADOBE_MARKER || true)`. Give it the red set you drive.

   Mark each change `(added at punch list)`.
3. No spec, stage or DEC id in any changed line outside the spec.

Stay in `src/metadata/mod.rs` (the test module only) and the SPEC-156 spec. `git diff --stat b0b870d`
must show nothing else. Non-test code in `src/metadata/mod.rs` must not change: show
`git diff b0b870d -- src/metadata/mod.rs` touches only lines inside `mod tests`.

**Not yours:**
- verify's out-of-spec findings, which the orchestrator has filed on `main`;
- #240's conflicts;
- the timeline;
- the DEC text for ship.

## Check it
- **Drive MK first**, against the new test, in a scratch worktree with its own `CARGO_TARGET_DIR`:
  `cargo clean -p crustyimg`, then a full `cargo test --no-fail-fast` with stdout redirected. FT-10
  must be red with its assertion message, and nothing else. Quote the counts.
- Then the unmutated head: a full test run gives 1,358 passed and 0 failed.
- Run clippy `--all-targets -D warnings` and `fmt --check` on the default and the
  `--no-default-features` builds, each in a fresh `CARGO_TARGET_DIR`.
- CI runs on push.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- If the PR is behind `main`, rebase onto `main` before you push. Never force-push over someone
  else's commit: if the PR's head isn't `b0b870d` when you start, stop.
- **Never poll CI.** Run `gh pr checks 269 --watch` once in the background after your push, and don't
  write your own wait loop. Don't use `pgrep -f`. When it exits, take one snapshot at the PR's head
  SHA, every leg.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not word-split
  unquoted parameters, and it reads `$B:r…` as a modifier, so brace it (`${B}:`). `rtk` can truncate
  `grep`, so use `/usr/bin/grep` for anything you count.
- Write scratch files in your session's scratchpad, never `/tmp` or the system temp dir.
- **Budget ~20 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`. It holds:
  - FT-10, with MK's red run;
  - the unmutated count;
  - clippy and fmt;
  - the CI snapshot.
- Leave `cycle: verify` as it is. Ship advances it.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-156 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is
    ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors. Also record verify's cost there as a
    `verify` entry, from these numbers: input 62, cache_creation 105,395, cache_read 3,160,954,
    output 20,822, total 3,287,233, $2.76, Opus.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-156 punch list: PR #269 at <head SHA> — CI <green | red: legs>`.

Then, briefly:
- `git diff --stat b0b870d`, and that only test code changed in `src/metadata/mod.rs`;
- FT-10's code, and MK's red set with its counts;
- the unmutated count, clippy and fmt;
- AC-3 as it now reads;
- the CI snapshot;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
