# SPEC-142 — VERIFY prompt: the ddphotos page's recipes give the same image natively and in wasm

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build SPEC-142.

```
just wt verify/spec-142 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-142-ddphotos-parity:refs/remotes/origin/test/spec-142-ddphotos-parity
git reset --hard origin/test/spec-142-ddphotos-parity
```

Draft PR #234's head was `1b1c010` when this prompt was written. `f204d43` is the last code commit,
and `1b1c010` changes only the spec. Record the SHA you verify. The maintainer pasted this prompt
into a fresh session and will paste your final message back to the orchestrator, so it must stand
on its own.

Every later STAGE-060 spec (crop on the page, then straighten) adds rows to this harness. A hollow
comparator here would pass all of them.

## What the build claims
- The spec is
  `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-ddphotos-page-native-vs-wasm-parity-harness.md`.
  Read all of it:
  - `## Frame` and both rulings blocks;
  - `## Design finding`;
  - AC-1 to AC-13;
  - FT-1 to FT-13, the row table, the golden texts and the mutation table;
  - the build's `## Build Completion`.

  Also read **DEC-110** in full, including the K the build wrote into its Validation section.
- **Outcome (i), K = 1.**
  - Rows 2, 6, 15 and 16 differ on 1 pixel by 1/255. Every other row has `px=0 max=0`.
  - The aarch64 Mac and the x86_64 runner give the same numbers.
  - The runner was an AMD EPYC 9V45, with avx2, avx512f and sse4_1.
  - JPEG bytes are equal on rows 9 and 11, so they're asserted (`JPEG_BYTES_ASSERTED = true`).
- **All 16 wasm output hashes are identical** on the Mac and on the runner.
- **The job takes 3m22s cold** (run `41144102452`) and **1m45s warm** (run `41147305687`).
- **Every mutation row matched the table, with three exceptions:**
  - **B-edge** also turns **B2** red at K = 1, because B2 is then the same input as B4. The table
    lists B2 as green.
  - **S4** also trips the no-resample-set assert.
  - **B8–B10** were added for the PNG and JPEG byte limbs.
- **FT-11:** all four cases exit 1 and name their cause.
- **FT-13:** the browser checks passed 40/0 on each of the dev, static and apache servers.
- **The test bed's automated stage then crashed:** `check.py:247: RuntimeError: edit.js has no
  GO_ADVANCE / GO_WIDEST`. It reads the font table out of `edit.js`, and the table now lives in
  `recipe.js`.
- **Two deviations:**
  - the venv is recreated on every run, because the cache restored a venv with no pip;
  - the plain run and `--self-test` share one CI step.
- An empty `src/`, `Cargo.toml` and `Cargo.lock` diff. `cargo test` gave 1115 passed.

## Context from SPEC-141 (PR #233, the guard over native output)
- SPEC-141's verify found that **native resampled pixels differ across Linux, macOS and Windows**,
  mostly on the 16-bit rows.
  - The seat is the platform `powf` (libm) in the sRGB transfer functions, stated as a prior.
  - CPU dispatch was ruled out: `F32x4` has no NEON path, and SSE4.1 and AVX2 share one form.
  - The two `powf` implementations disagree on 23 of the 256 8-bit table entries.
- wasm uses the `libm` crate's `powf`. So the 1 px difference here is probably that same
  disagreement, surviving rounding to u8 on 4 rows.
- **Check that this is consistent (item 4).** It matters for the follow-up the orchestrator is
  weighing: a deterministic `powf` on native would make K = 0, and SPEC-141's goldens shared.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Don't fix what you find; report it. Don't edit
the spec, its timeline, DEC-110, or the test bed.

## Check, in order
1. **Scope and commits.**
   - Compare `git diff --stat origin/main...HEAD` with the spec's Outputs and Build Completion.
   - `git diff --stat origin/main...HEAD -- src/ Cargo.toml Cargo.lock` must be empty.
   - Every commit is signed off.
   - `git diff --name-only f204d43 HEAD` lists only the spec.
   - DEC-110's diff touches **only** its Validation section, never the rule or the cap. Run
     `./scripts/decisions-audit.sh`, and report any structural error.
2. **AC-1: one copy of the recipe code.**
   - Use `/usr/bin/grep` on `edit.js`, `worker.js` and `tests/ddphotos_parity.mjs` for every name
     AC-1 lists, plus `planEdit`.
   - Each must be defined only in `recipe.js`. `recipe.js` has no `document`, `$(` or `window`.
   - Then read the harness for any size, margin or recipe text it computes itself. A harness that
     re-derives `outW`/`outH` instead of reading `planEdit`'s output moves with a `recipe.js` bug
     and stays green.
3. **Run it yourself.**
   - In a fresh checkout, run `just ddphotos-parity` from clean (`rm -rf target/ddphotos-parity`
     first). Record the exit code and every row line.
   - The numbers must equal the build's: `px=1 max=1` on rows 2, 6, 15 and 16, zero elsewhere.
   - Run it a second time, warm, and confirm it is identical.
   - **Prove the warm run used fresh builds.** CI restores `target/` from a cache, and that cache
     once served a broken venv. Check that the wasm package and the native binary are rebuilt or
     re-validated on each run, not reused stale:
     - touch a `src/` file (no content change), and confirm cargo rebuilds;
     - compare the `pkg-slim` `.wasm` hash before and after a source change that alters output
       (for example, the M1-style filter swap), rebuilt.
     - If a stale package could survive a source change, that's a finding.
4. **Consistency with SPEC-141's evidence.** Read the CI log of the green parity run once
   (`gh run view <run> --log --job <job>`).
   - Do the native-side outputs on the runner equal the native-side outputs on your Mac, row for
     row? Compare hashes if the harness prints them. If it doesn't, say so, and compute the Mac
     side yourself.
   - The same 4 rows differing by 1 pixel on both targets fits a `powf` table disagreement
     surviving u8 rounding. Say whether the evidence supports that, or contradicts it.
   - **Drive it if you can,** in a scratch crate outside the repo. Run the crate's
     `srgb_to_linear` / `linear_to_srgb` (or a copy) once with `f32::powf` and once with
     `libm::powf`. Then check whether the 8-bit pixels that differ on rows 2, 6, 15 and 16 are
     ones whose computation passes through one of the 23 disagreeing table entries. If that's out
     of reach, say so, and give the prior.
5. **The comparator, read whole.**
   - It's one pure function, and `K` and the cap `78` are named constants. The harness asserts
     `K <= 78` at start.
   - **Dims come first, and the buffers are never touched on a dims mismatch** (B7).
   - "Resamples" is derived from the recipe text, and the `{1, 3, 9, 11}` assert is live.
   - The B-edge exception: B2 and B4 are the same input when K = 1, so B2 going red under
     B-edge is the table's slip, not a comparator bug. Confirm that by reading the cases. Then
     say whether the table needs a note for the next spec that raises K.
6. **Re-drive the mutation table yourself.** Apply each where the table says, run, record the red
   set, then restore.
   - **Required:** N1 and N2 (against the real binary, as the plain run's numbers), B-dims,
     B-derive, B-edge (show the plain-run line on the target where K was seen), S1, S3, C1, H4.
   - Then any row whose recorded result in Build Completion looks thin.
   - Rebuild the native binary **clean** for any row that touches it.
7. **Mutations the spec did not list.** Apply each, and name the test that catches it, or report
   a gap:
   - **M-a:** the decoder returns the *native* file's pixels for both sides, so a harness bug
     compares a file with itself.
   - **M-b:** `K` set to 78 (the cap). Do the plain run and `--self-test` both stay green? What
     then protects against a regression from 1 pixel to 78? Say whether that's acceptable under
     DEC-110, or a gap for its Validation section.
   - **M-c:** the harness drops row 14 (AVIF) silently, with the row count adjusted to 15
     elsewhere. Which literal catches it?
   - **M-d:** `previewRecipe` returns its input unchanged (no quality). Which row goes red?
8. **The CI wiring (AC-10, AC-11).**
   - **The classifier, failing.** The `changes` step calls `scripts/ci-code-paths.sh` inside
     `$(…)` and compares the result with `"true"`. A script that **crashes** there prints
     nothing, so the result is `code=false` and every heavy job skips green.
     - Drive it locally. Make the script exit 1 after its self-test passes (for example, a bad
       `grep` flag only in the classify path), and run the step's shell under `bash -e`, the
       way Actions runs it.
     - Report whether the step fails, or classifies as docs-only. If it classifies as docs-only,
       that's a punch item, because that failure skips CI silently.
   - **The classifier, working.** On the PR's own runs, confirm that the code commit `f204d43`
     ran the matrix legs and that the spec-only commit `1b1c010` skipped them. Read
     `gh pr checks 234` once at the head.
   - **The parity job:**
     - it's advisory, not a required check (`gh api repos/{owner}/{repo}/branches/main/protection`,
       or say why you can't read it);
     - it runs on `ubuntu-latest`, with `needs: changes` and the `code == 'true'` guard;
     - the self-test lines are visible in the log.
     - Judge the one-step deviation: if `--self-test` fails, does the job fail, and does the log
       make clear which half failed?
9. **The page, and the test-bed break.**
   - Read `edit.js` and `worker.js` as changed. The page is still a static file set: no build
     step, and the imports work as relative ES-module imports. `node --check` both, via `.mjs`
     copies.
   - **The test bed.** Find what `check.py` reads from `edit.js` (`GO_ADVANCE` and `GO_WIDEST`)
     and where those names live now.
     - Is the page's behaviour unchanged, with only the test bed's source scraping broken?
     - Name the smallest test-bed-side fix: read `recipe.js` too, or read whichever file
       defines the names.
     - **This is a merge-order question for the maintainer, not a punch item:** once #234 merges,
       the test bed's automated stage goes red. Say whether anything in this repo should change
       instead. For example, would keeping the font table in `edit.js` be right? It probably
       wouldn't, because AC-1 moves it. Argue it either way.
     - Never edit the test bed.
   - The README's "How it was tested" line and the re-measured page size: drive the size.
10. **Fixtures (AC-12).**
    - `tests/fixtures/ddphotos/README.md` gives the exact commands, the tool versions and the
      provenance.
    - Check the committed files' sizes and dimensions, and that `photo_rot.jpg` carries
      Orientation 6 (`exiftool -n -Orientation`).
    - Confirm nothing in CI regenerates them.
11. **Full suite.** Run the default `cargo test` leg once, in a fresh target dir, with
    `--no-fail-fast`. Redirect stdout and read `$?`.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no
  `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Reverting source doesn't rebuild the binary, and a target dir shared across worktrees serves a
  stale library. Use a fresh dir per checkout.
- Run Python that reads test outputs with `python -I`.
- Never poll CI; read each log once. **Never drive the maintainer's browser.** If you re-run the
  test bed, use its headless `just retest`.
- Delete each scratch target dir, scratch crate and scratch worktree when you're done. Keep
  scratch files in a scratch directory, never in `~` or the repo.
- **Budget ~70 exchanges.** Item 4's driven probe is the optional part. If you're short, give the
  verdict from the evidence, and say so.

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "VERIFY prompt: the ddphotos page's recipes" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
- Price each component at the model that ran (`.message.model`): Opus $5/$25 per MTok, Sonnet
  $3/$15. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #234 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #234 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–11, one or two lines each, with evidence;
- **item 4's verdict on its own line:** does a deterministic `powf` on native plausibly take K to
  0? Give the evidence;
- **item 8's classifier-crash result on its own line;**
- the test-bed break: what it reads, the smallest fix on its side, and the merge-order advice;
- the mutation table as you ran it, and M-a to M-d;
- findings that belong outside this PR, each with where it should be filed;
- optional cleanups, listed separately from punch items;
- your cost.
