# SPEC-142 — RE-VERIFY prompt: the punch list on the ddphotos parity harness

Cycle: **verify (re-verify)**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not
design, build or fix SPEC-142. Scope is narrow: did the punch list close its two items without
breaking what verify already passed?

```
just wt verify/spec-142-re -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-142-ddphotos-parity:refs/remotes/origin/test/spec-142-ddphotos-parity
git reset --hard origin/test/spec-142-ddphotos-parity
```

Draft PR #234's head was `474c9fc` when this prompt was written. `e71acbd` is the last code
commit, and CI run `41154144464` there is green (17 success, 6 skipped). Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

## What verify found (at `1b1c010`) and what the punch list claims
Verify's punch items:
1. A crashing `scripts/ci-code-paths.sh` made the `changes` step classify the PR as docs-only, so
   every heavy job would skip and show green.
2. No golden recipe text pinned the preview recipe. A `previewRecipe` that dropped
   `quality = 88` stayed green.

The punch list (`prompts/SPEC-142-punchlist.md` on `main`) claims:
- **The `changes` step** now assigns `code=$(…)` first, requires exactly `true` or `false`, then
  branches. Driven: the crash exits 2 and writes no `code=`, `maybe` exits 1 with an `::error::`,
  and the old body under the same crash exits 0 with `code=false`.
- **P5 pins row 12.** The literal is built in the harness as
  `GOLDEN_PIXEL_BOTH.replace('version = "2"\n', 'version = "2"\nquality = 88\n')`, and the spec
  carries the text pasted from a real run. The preview mutation turns P5 red, although the row
  table still reads `16 rows, 0 failed`.
- **The row line** prints both `wasm-sha256` and `native-sha256`, and a summary line reads
  `largest differing-pixel count on a resampling row: 1 (K = 1)`.
- **DEC-110's Validation section** gains three bullets: the second CPU, K guarded by review only,
  and the measured `libm::powf` remedy.
- **The mutation-table notes** cover B-edge/B2 and B-dims/N2.
- The CI run landed on a **third** x86 CPU (an Intel Xeon Platinum 8370C), with the same result.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Don't fix what you find; report it. Don't
edit the test bed (a separate session fixes it).

## Check, in order
1. **Scope.**
   - `git diff --stat 1b1c010 HEAD` lists only `ci.yml`, DEC-110, the spec and
     `tests/ddphotos_parity.mjs`.
   - In `ci.yml`, only the `changes` step's `run:` changed.
   - DEC-110 changed only inside Validation.
   - `git diff 1b1c010 HEAD -- src/ Cargo.toml Cargo.lock` is empty, and every commit is signed
     off.
   - `./scripts/decisions-audit.sh` reports no structural error.
2. **Punch item 1, driven by you.** Copy the `changes` step's `run:` body **from `ci.yml` at the
   head**, not from the punch list's scratch copy, into a scratch script, with `files` and
   `GITHUB_OUTPUT` set by hand. Run every case under `bash -e {0}`, the way Actions runs it.
   - The classifier crashes in its classify path after the self-test passes: non-zero exit, no
     `code=`.
   - The classifier prints an empty string: non-zero exit, with the value named.
   - The self-test itself fails: non-zero exit.
   - `src/lib.rs` gives `code=true`; `docs/x.md` gives `code=false`.
   - The old body (from `1b1c010`) under the same crash: exits 0 with `code=false`, as the
     control.
   - Read the rest of the step for any other `$(…)` whose failure is swallowed. Look especially
     at the `files=$(git diff …) || echo "src/"` fallback, and say whether it fails safe (it
     should mean "run everything").
3. **Punch item 2.**
   - Make `previewRecipe` return its input unchanged. P5 must go red on row 12, and the run must
     **exit non-zero**. Read the exit code, since the table line read `0 failed`.
   - **Judge the derived literal.** It's a fixed `.replace` on the row-4 literal, not a copy of
     `withQuality`.
     - Does it pin where `quality` goes? Move the `quality` line elsewhere in `withQuality`'s
       output, as a mutation in `recipe.js`. Does P5 go red?
     - Does the derived string equal the spec's pasted text, byte for byte?
     - If either answer is no, it's a punch item. If both are yes, a verbatim literal is still
       an optional cleanup, so say so.
4. **The new output lines.**
   - Run `just ddphotos-parity` from clean (`rm -rf target/ddphotos-parity`).
   - On every row, `native-sha256` equals `wasm-sha256` exactly where the line says
     `bytes same`.
   - The summary line reads `1 (K = 1)`.
   - Read CI run `41154144464`'s parity log once, and compare its 16 native hashes with yours.
     The 12 byte-equal rows must match across the Mac and the Xeon. Report the 4 resampling rows
     as they are, matching or not. Either is consistent with DEC-110, so this is evidence, not a
     pass/fail.
5. **Nothing verify passed has regressed.**
   - Run `--self-test`: B1–B10, N1 and N2 give their listed verdicts.
   - Re-drive **N1** and **B-edge** once each, and restore.
   - `gh pr checks 234` once at the head: the parity job and the matrix are green, and the
     parity job is still not a required check.
6. **The text.** The mutation-table notes and DEC-110's three bullets say what verify found.
   Check the CPU names against the run logs.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep`, so use `/usr/bin/grep` for anything you count.
- Never poll CI; read each log once. Never drive the maintainer's browser.
- Delete scratch scripts and `target/ddphotos-parity` when you're done, and leave the tree clean.
- **Budget ~30 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "RE-VERIFY prompt: the punch list on the ddphotos parity harness" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10.
- Report the total and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #234 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #234 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–6, one or two lines each, with evidence;
- **the classifier cases as a table:** case, exit code, `code=` written;
- the derived-literal verdict;
- the native hashes across the Mac and the CI CPU;
- anything the punch list broke;
- optional cleanups, listed separately;
- your cost.
