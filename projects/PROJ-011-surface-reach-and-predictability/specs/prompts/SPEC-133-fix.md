# SPEC-133 — FIX prompt: the verify punch list, on the same draft PR #206

Cycle: **build** (punch-list follow-up). Fresh CLI session, **Sonnet**, on the PR's own branch:

```
just wt feat/spec-133-watermark-damage-rule -- --model sonnet
```

That reuses the existing worktree; close any old session in it first. You did not build or verify
SPEC-133. Verify at `3dd23ff` found the behaviour correct on every surface, with passing marks
byte-identical to `main`. It found four test gaps that mutation testing exposed and three false
doc sentences. Fix exactly the items below on branch `feat/spec-133-watermark-damage-rule`, and
push to the **same draft PR #206**. The maintainer pasted this prompt into a fresh session and
will paste your final message back to the orchestrator, so it must stand on its own.

## Step 0
1. `git fetch origin && git status`. The tree must be clean.
2. `git pull --ff-only`. `git log -1` should show `3dd23ff` or later.
3. `git merge origin/main`. Merge, don't rebase: the PR is shared. `main` has only docs changes
   since.
4. Read the spec's ACs 3–5 and 17, `## Build Completion`, and `Watermark` (`refusal`,
   `mark_size`, the `visible` count) in `src/operation/mod.rs`. Read `tests/watermark_damage.rs`
   too.

## Tests: each one must kill the named mutation
For each item:
- apply the mutation;
- rebuild in a **fresh** `CARGO_TARGET_DIR`, because reverting source does not rebuild the binary;
- show that the new test goes RED;
- restore the source, rebuild, and show it GREEN.

Check that each mutation really applied: `git diff` is non-empty before you build.

1. **AC-5 is hollow (mutation M2: count the visible pixels after `opacity` is applied).**
   - The cause: 255 × 0.01 = 2.55, which composites as alpha 3, so no alpha reaches 0 at
     `opacity = 0.01`, and M2 leaves the suite green.
   - The fix: use an opacity where the composited alpha really is 0. Either `opacity = 0.001`,
     or an overlay alpha of about 100 at 0.01. Derive that from the arithmetic, not by trying
     values until one works. Assert `covered: 501` still.
   - Correct AC-5's wording in the spec to match.
2. **The 16-bit count is untested (M1: count on `to_rgba8()`).**
   - Add one unit row: an `ImageRgba16` overlay with 501 pixels at alpha ≤ 128 out of 65535, on
     a 100×100 base. Assert it is refused with `covered: 501`.
   - Under M1 those alphas truncate to 0, and the mark is accepted.
3. **No test checks that the rule judges the image entering the step (M7: judge the pipeline's
   input instead).**
   - Add two one-input tests. They can be CLI or pipeline-level; your choice:
     - `[resize percent 400, watermark]` on an input too small for the mark: exit 0, and the
       mark is drawn (the output differs from the same recipe without the watermark step);
     - a shrinking resize that makes a fitting mark stop fitting: exit 2, nothing written.
   - Note that M7 may still refuse the shrink case, because `apply`'s own guard backs it up.
     The enlarge row is the one that must flip.
4. **"K counts images, not steps" is untested (M5: count K per skipped step).**
   - Add a batch row: a recipe with **two** watermark steps that both refuse on the small
     input. Assert two warning lines and `watermark skipped on 1 of 2 images`.

## Docs
5. **The `watermark --help` sentence is false.** `--image logo64 --tile` on 256×256 covers 100 %
   and exits 0. Scope the 5 % clause to a fixed-size mark: no `--scale`, no `--tile`. Then check
   that `docs/cli-reference.md` matches, if it repeats the sentence.
6. **The input count is taken after globs and directories resolve.**
   - `watermark 'one/*.png'` and `watermark one/` resolving to one file take the exit-2 path.
     That is consistent with SPEC-134, which counts resolved inputs too. Keep the behaviour and
     document it.
   - Where it says "one explicit input" in `docs/api-contract.md`, `recipes/README.md` and
     DEC-104, state that the count is taken after globs and directories resolve: one resulting
     image is refused with exit 2; two or more are skipped.
   - Add a test row for a directory holding one file → exit 2.
7. **The CHANGELOG Fixed sentence describes behaviour no release had.** "…refused at load too
   (exit 1) instead of covering half the image": 0.7.1 has no recipe watermark step at all.
   Reword it to state only the rule (a recipe `text` step refuses `scale` and `tile = true`),
   with no "instead of".

## Cleanups (orchestrator's call: code quality is a must)
8. Turn `optimize_decide_one`'s 6-tuple into a named struct, the way `DecidedEncode` is. No
   behaviour change.
9. The input-label block is copied into three closures in `run_apply`, and it now runs on success
   too. Move it into one small helper, computed only when a line needs it.
10. SPEC-131's `scale` range rows (`scale = 0`, `nan`, `1e-50`) in
    `tests/recipe_strict_params.rs` sit on **text** steps, which Call 5 now makes invalid. They
    still test the range only because the schema check runs before the mode check.
    - Move them to an image step, so each row fails for its own reason.
    - Show each row's message is the range message, not the mode message.
11. Add `scale = 1` on an **image** step to a recipe corpus file. That way an integer `scale`
    is round-tripped again, which the build's removal of the text-step line lost. Update the
    matching `.golden`.
12. DEC-104: add one line saying that a faint 16-bit mark on an 8-bit base is refused by its
    pixel count even when it changes 0 px. That is conservative, and by design.

Not in scope:
- `--scale inf`'s `4294967295×…` message; it belongs to the open `--size nan` backlog item.
- A `--json` field for skipped steps.
- SPEC-128's missing CHANGELOG Added entry; release prep backfills it.

## Boundaries
- `tests/`, the CHANGELOG, `docs/api-contract.md`, `docs/cli-reference.md`,
  `recipes/README.md`, and DEC-104.
- The `watermark` help text in `src/cli/mod.rs`.
- In `src/cli/optimize.rs`, items 8 and 9 only.
- The spec's AC-5 wording, `## Build Completion` and `cost.sessions`.

No other `src/` change. If a test shows the code is wrong, stop and report: don't fix it here. Do
not edit the spec's timeline; it lives on `main`.

## Finish
- **Out-of-spec findings** (HANDOFF rule 6): any bug you find outside this list goes in Build
  Completion as an "Out-of-spec finding" line, with either a proposed CHANGELOG/DEC line or a
  backlog item.
- **Clean matrix:** default, `--no-default-features` and `--features webp-lossy`.
  - Give each leg a **fresh** `CARGO_TARGET_DIR` and run the legs **one after another**.
  - Each leg runs the tests and `clippy --all-targets -D warnings`.
  - Pass feature flags as separate literal arguments: zsh does not word-split.
  - **Delete each target dir when its leg is done:** an earlier session filled the disk.
  - Then run `fmt --check`, `just wasm-test` and `just demo-build`.
- Sign every commit with `git commit -s` (DCO). Push once the code compiles, before the matrix.
  ⛔ The PR stays a **draft**; never mark it ready.
- **Never poll CI.** Run `gh pr checks 206 --watch` in the background. When it exits, take a
  direct snapshot at the true head SHA and check every leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code. macOS has no `timeout(1)`.
- **Budget ~80 exchanges.**

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Find it by content:
`grep -l "SPEC-133 — FIX prompt" ~/.claude/projects/<slug>/*.jsonl`.

Dedup by `.message.id`: take input, cache_creation and cache_read once per id, and the MAX
output_tokens. Price each component at the model that actually ran, never at a flat rate: Sonnet
$3/$15 per MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

Append a `build` entry to the spec's `cost.sessions` on this branch. Report `tokens_total`, the
breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-133 fix: draft PR #206 at <SHA> — CI <green|red>`. Then:
- items 1–4: each new test, and its RED under the mutation and GREEN after, with the
  fresh-target-dir evidence;
- items 5–7: the doc changes, with the exact new sentences;
- items 8–12: what changed;
- any out-of-spec findings;
- the matrix;
- the files changed;
- your cost.
