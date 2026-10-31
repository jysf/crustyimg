# SPEC-141 — VERIFY prompt: pin the output of every existing recipe (the PROJ-017 guard)

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build SPEC-141.

```
just wt verify/spec-141 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-141-recipe-output-guard:refs/remotes/origin/test/spec-141-recipe-output-guard
git reset --hard origin/test/spec-141-recipe-output-guard
```

Draft PR #233's head was `dfaa28b` when this prompt was written. `886d388` is the last code
commit, and `dfaa28b` changes only the spec. Record the SHA you verify. The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so it
must stand on its own.

Every later PROJ-017 spec proves "no existing output changed" with this guard. A hollow guard
here goes unnoticed for the whole wave, which is why this verify goes deeper than the build's own
mutation run.

## What the build claims
- The spec is
  `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-141-no-existing-recipe-output-changes.md`.
  Read all of it:
  - `## Frame` with its three rulings;
  - `## Design`, Calls 1–7;
  - AC-1 to AC-13;
  - `## Failing Tests` (FT-1 to FT-8) and the mutation table M1–M7;
  - the build's `## Build Completion`.
- **Outcome (ii), per-target goldens.**
  - Checkpoint 1 (run `41134312226`, Mac goldens in the shared layout) was red on the Linux legs
    and on Windows. Linux changed 26 of 48 rows. The Windows rows were not counted.
  - The five Linux legs printed byte-identical dumps.
  - The Linux and Windows goldens were copied from that run's marker blocks.
  - Run `41136772264` at `886d388` is green on every leg.
- 42 `.output` files: 14 recipes × `linux-x86_64`, `macos-aarch64`, `windows-x86_64`.
- **M1–M7 matched the table, with one gap.** Under M4 the stray-golden assert fires first, so the
  `48 ≠ 51` line-count literal was never seen going red on its own.
- **M1–M7 were driven on the shared layout**, before the per-target switch. The build argues the
  switch only changes which file is read.
- **One deviation from the spec.** The "output equals the fixture" check (AC-4 (b)) runs only when
  the dims check passed.
- An empty `src/` diff. The matrix (default 8 tests; lean and webp-lossy 7) is green, each in a
  fresh target dir.

## What the orchestrator found on the branch (check it, don't trust it)
Comparing the committed goldens per target, with comment lines dropped:
- **12 of 14 recipes differ on every pair of targets.**
  - These are every resizer, plus `v1_watermark_image_placement`.
  - That recipe has no `resize` step, but it scales the logo (`scale = 0.25`), and only its
    `rgba16_2600x1730` line differs.
- **Two recipes are identical on all three targets:** `v1_watermark_text_font` and
  `v2_format_quality_optimize`.
- **Linux and Windows are both x86_64, yet they differ.** The resizer is built with
  `fast_image_resize::Resizer::new()` (`src/operation/mod.rs:586`), which picks its SIMD path
  from the CPU at **run time**. Item 5 below is about what that means.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Don't fix what you find; report it. Don't edit
the spec or its timeline.

## Check, in order
1. **Scope and commits.**
   - Compare `git diff --stat origin/main...HEAD` with the spec's Outputs and Build Completion.
   - `git diff --stat origin/main...HEAD -- src/ .github/ scripts/ docs/ Cargo.toml Cargo.lock`
     must be empty.
   - The 13 pre-existing `.golden` files are byte-identical to `main`.
   - Every commit is signed off.
   - `git diff --name-only 886d388 HEAD` lists only the spec.
   - If **SPEC-143** (PROJ-011, the watermark that draws nothing) has merged to `main` since,
     merge `origin/main` into a **local scratch branch** (never pushed) and run the test on it.
     A red row is a SPEC-143 finding, not a SPEC-141 one. Name the row.
2. **The fixtures (AC-2).**
   - Diff the test's generator against Design Call 1's text. It must be verbatim.
   - It must not call any crustyimg decode or encode.
   - Run FT-2 and confirm the three full SHA-256 literals. Their prefixes are `94daa4af`,
     `7df0cef0` and `75dfcd86`.
   - Read the green CI run's log on one Linux leg and on the Windows leg, and confirm FT-2 ran
     there as `ok`. A fixture that differs per target would make every golden for that target
     wrong.
3. **Per-target goldens are what CI produced.**
   - Run `gh run view 41134312226 --log-failed` once. Extract the Linux and Windows marker blocks
     and diff each against the committed `.linux-x86_64.output` and `.windows-x86_64.output`
     files, after normalising CRLF.
   - Report any recipe that has no block, or whose file differs.
   - Independently re-derive the orchestrator's 12 / 2 split above.
4. **The layout, from `ci.yml`, not from the spec.**
   - Read every `cargo test` leg in `.github/workflows/ci.yml` (matrix and extra jobs), and map
     each `runs-on` to the `(os, arch)` the test selects.
   - `TARGETS` must be exactly those three, and FT-6 must require 42 files.
   - **FT-8:** an unknown target is `Err` with Call 5's message, never a fallback to another
     target's file.
   - Read the green run's log on every leg once (`gh run view <run> --log --job <job>`; never
     poll). Confirm `recipe_corpus_output` ran 8 tests (7 on lean and webp-lossy), with
     `bless_recipe_corpus_outputs` the only `ignored`.
5. **Do the x86 goldens pin a runner's CPU, not its OS?** This is the question that most decides
   whether the guard can false-red later.
   - Read `fast_image_resize` 6.0.0's source under `~/.cargo/registry/src/`. For the `F32x4`
     convolution, find:
     - which CPU-extension paths exist (none, SSE4.1, AVX2, NEON, wasm32 simd128);
     - whether `Resizer::new()` selects among them at run time;
     - whether those paths can give different f32 results (a different summation order or FMA,
       for example).
   - **Drive it if you can,** in a scratch crate outside the repo. Resize one fixture-shaped
     buffer in `F32x4` with Lanczos3, once per available `CpuExtensions` value
     (`set_cpu_extensions`, `unsafe`), and hash each result.
     - On aarch64, compare `Neon` with `None`.
     - If `rustup target list --installed` has `x86_64-apple-darwin`, or you can add it, also run
       under Rosetta 2 with `None`, `Sse4_1` and `Avx2`, whichever the CPU reports.
     - Use a fresh target dir and delete it afterwards.
   - Also find the cross-OS seat for the Linux-vs-Windows difference: if the dispatch can't
     explain it, check the f32 `powf` in the linear-light conversion, which uses the platform's
     libm on native targets. State the cause as a **prior**, with the evidence you have.
   - **Verdict, in one line:** either "goldens are stable per (os, arch)", or "goldens depend on
     the runner's CPU features (name them), so a runner change can turn the guard red with no
     code change".
     - The second is a finding to rate, not automatically a rejection.
     - Propose the smallest remedy and where it belongs. It is probably not this PR: a `src/`
       change is out of SPEC-141's scope.
6. **Re-drive the mutations on the per-target layout.** For each one:
   - apply it, found **by content** (SPEC-143 may have moved line numbers), and check `git diff`
     is non-empty;
   - rebuild **clean** (a fresh `CARGO_TARGET_DIR`, or `cargo clean -p crustyimg`);
   - run `cargo test --test recipe_corpus_output --no-fail-fast > out.txt 2>&1; echo $?`;
   - count the mismatch lines with `/usr/bin/grep -c`;
   - restore it, and show it green.

   The mutations:
   - **M1 (required).** 36 lines, `36 of 48 rows changed`, FT-4's `png` row red. `recipe_round_trip`
     stays green.
   - **M3.** Re-bless under the mutation, then run. Expect 33 `dims` lines, 9 `output equals the
     fixture` lines, and one `step … changed nothing` line per non-exempt step.
     - **Derive the expected step-line count yourself** from the 14 recipe files: the non-exempt
       steps × 3 fixtures. Compare it with the build's 54.
     - Restore with `git checkout -- src/ tests/fixtures/`, and show `git status --short` empty.
   - **M5.** Hash the golden directory before and after; it must be identical. Also run the
     control with `CI` unset.
   - **M4, split into two**, closing the build's gap:
     - **M4a, as the spec words it.** Record which assert fires first.
     - **M4b.** Delete `"v1_resize_fit"` from `RECIPES` **and** delete its three `.output` files.
       FT-1 must give `rows checked 45, expected 48`, and FT-6 must go red on the `51` literal
       alone, with no stray-golden line.
     - If FT-6 stays green under M4b, the line-count literal is hollow. Report that.
   - **M2** (watermark opacity default). 9 lines.
7. **The deviation (AC-4 (b) runs only after dims pass).**
   - Judge whether it weakens the guard. On a resizer, a dims change already implies a changed
     output. On the 3 non-resizers, (b) must still run on every fixture.
   - Prove the second point with a mutation: make one non-resizer's step a no-op (for example,
     the watermark's draw returns its input), rebuilt clean. All 3 of its rows must report
     `output equals the fixture`.
8. **Mutations the spec did not list.** Apply each one, rebuild clean, and name the test that
   catches it, or report a gap:
   - **M-a, CRLF.** Convert one Mac golden to CRLF on disk (`perl -pi -e 's/\n/\r\n/'`): the test
     stays green. Then remove the line-ending normalisation in the reader: it goes red. This is
     the only local proxy for the Windows checkout.
   - **M-b, fixture drift.** Change one sample in the RGB8 generator: FT-2 goes red, and every
     `rgb8` row with it.
   - **M-c, logo drift.** Change one byte of the test's logo pixels: only the image-watermark
     rows go red. List them.
   - **M-d, bless gate.** `bless_gate` treats `CI=""` as set: FT-7 goes red.
9. **The guard as a tool for later specs.**
   - Read `tests/fixtures/recipe_corpus/README.md` (AC-11).
   - A later spec adds a recipe. Do the README and Call 7 tell its builder exactly which three
     literals to bump (48, 51, and the dims table), and that its new `.output` lines are not a
     re-bless?
   - A later spec runs on a new CI target. Does the guard fail with a message that says how to
     bless that target from a CI log?
   - Read every failure message against Call 4's wording.
10. **CI history.** On checkpoint 1, the macOS heic leg died on `avif_tile_pin` before it reached
    this test.
    - Check whether that test fails on recent `main` runs:
      `gh run list --branch main --workflow ci.yml --limit 15`, then the failing legs only.
    - Report it as a flake (with run ids) or as a one-off. It isn't a SPEC-141 finding, but it is
      a finding.
11. **Full suite, sequentially.** Run the default leg and the `--no-default-features` leg, each in
    a fresh target dir, with `--no-fail-fast`. Redirect stdout and read `$?`. Then run
    `gh pr checks 233` once at the head you verified.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. In zsh, write
  `"${B}:path"` for a git rev and path. macOS has no `timeout(1)`.
- `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count.
- Reverting source doesn't rebuild the binary, and a target dir shared across worktrees serves a
  stale library. Use a fresh dir per checkout, and never run differently-featured builds in one
  dir at once.
- Delete each scratch target dir, scratch crate and scratch worktree when you're done. Keep
  scratch files in a scratch directory, never in `~` or the repo.
- **Budget ~70 exchanges.** Item 5's Rosetta probe is the optional part. If you're short, read
  the source and give the verdict from the code alone, and say so.

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "VERIFY prompt: pin the output of every existing recipe" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
- Price each component at the model that ran (`.message.model`): Opus $5/$25 per MTok, Sonnet
  $3/$15. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #233 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #233 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–11, one or two lines each, with evidence;
- **item 5's verdict on its own line**, then the cause as a prior, and the remedy and where it
  belongs;
- the mutation table as you ran it: mutation, line count, red set, whether it matches the spec;
  M4a and M4b separately;
- M-a to M-d, each caught (by which test) or a gap;
- findings that belong outside this PR, each with where it should be filed;
- optional cleanups, listed separately from punch items;
- your cost.
