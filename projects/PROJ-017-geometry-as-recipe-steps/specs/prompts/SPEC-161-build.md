# SPEC-161 — BUILD prompt: crop on the ddphotos edit page, the recipe model and the 4× replay

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**.

There's no Rust in this build. It is JavaScript and the parity harness:
- **`recipe.js`** gains the crop model: `aspectSize`, `followOrientation`, `presetCrop`, `swapCrop`,
  `cropBox` and `loadPhoto`, `planEdit` with a crop, and the crop step in the recipes.
- **`worker.js`'s `load`** becomes one line.
- **The harness** gains 35 crop rows, 17 replay rows on Pillow-made originals with a PSNR limb,
  model, placement and golden checks, B11–B19, and the controls N4 and N5. It also takes the P5
  relabel and drops the rows 15–16 transpose.
- **DEC-118** is a new record, and you write it from the spec's text.

The design drove every number and every mutation row on a prototype. Most of the work is the rows and
the harness plumbing. Be thorough across every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-161-page-crop-model -- --model sonnet
```

You didn't design this; the spec carries the context (AGENTS §15). The maintainer pastes this prompt
into a fresh session and pastes your final message back to the orchestrator, so that message must
stand on its own. **Write it as plain lists, not box-drawn tables.**

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `9a626fa` is in your history;
- `decisions/` has no `DEC-118` file yet (you write it);
- the spec has `### Maintainer rulings on the design (2026-11-26)`.

If any check fails, stop and report.

⚠ **Overlap.** The spec's map lists only parked PR #237, with no file in common. After every rebase,
run `gh pr list --state open --json number,title,files` and compare. Name any new overlap in Build
Completion. If a rebase conflicts, stop and report. Don't touch #237.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-161-crop-on-the-ddphotos-edit-page-the-recipe-model-and-the-4x-replay.md`.
   Read it in full:
   - `## Frame`, and both sets of rulings: 2026-11-22 at the frame's end, and 2026-11-26 under
     `## Design`;
   - `## Design`: the corrections, "What was driven", the model, `worker.js`/`edit.js`, the replay
     mode, the rows, DEC-118's text;
   - the Outputs, every AC and every Failing Test;
   - the mutation table and `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-110 (native against wasm on the same input), DEC-112 (with rule 8), DEC-113 and
   DEC-114 rule 5. Read DEC-117 for the front-matter shape DEC-118 copies.
3. **The page code in full:** `integrations/ddphotos/edit/recipe.js`, `worker.js` and `edit.js`
   (`edit.js` must not change).
4. **The harness:** `tests/ddphotos_parity.mjs`, `tests/ddphotos_decode.py`, and the
   `ddphotos-parity` recipe in the `justfile`.
5. **The arithmetic you port**, read only, at the spec's `file:line`: `rect_edges`, `aspect_size`,
   `resize` mode `max` with its `.max(1)`, `follow_orientation` and `placement`. Find each by content
   if it has moved.
6. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for" (traps
   7, 10, 12, 13, 14 and 17). These bind PROJ-017.
7. SPEC-163 in `specs/done/`: its parity rows and N3, which you must leave untouched, and its verify
   and ship notes.

## Maintainer rulings (settled — do not re-open)
1. Split: this spec is the recipe model and the 4× replay. There's no crop control on the page yet.
2. An untouched preset emits `aspect`; a dragged box emits a box.
3. A preset on a non-square photo emits `orientation = "match"`.
4. **A preset on a square photo, and `1:1` anywhere, writes no key.**
5. **A swapped preset writes the aspect as shown, with no key.**
6. The replay DEC is DEC-118.
7. No Rust change. No message, comment or DEC body names a spec or stage id beyond the DEC's own
   front matter.

## ⚠ The harness's temp folders — set this up before your first run
The harness writes each run's outputs, and in this spec the 4× originals, into
`mkdtempSync(join(tmpdir(), "ddphotos-…"))`. It deletes the folder only on a green run, so **every red
run leaves one behind.** The design's red mutant runs left 48 of them, 9.34 GiB, in the system temp
folder.
- **Point the harness at your scratchpad.** For every harness run, set `TMPDIR` to a folder inside
  your session's scratchpad, for example `TMPDIR=<scratchpad>/harness-tmp`. Create that folder first.
  Node's `os.tmpdir()` reads `TMPDIR`, so the folders land there and not in the system temp folder.
  - Do this for `just ddphotos-parity` and for any direct `node tests/ddphotos_parity.mjs` run.
  - Check it took effect: after your first run, the new `ddphotos-*` folder must be under your folder.
    If it isn't, stop and work out why before running any mutant.
- **Empty that folder after each red run you have finished reading,** and certainly between mutation
  rows. A folder kept for debugging must be named in Build Completion.
- **Never delete `ddphotos-*` folders in the system temp folder.** Some there are from earlier
  sessions, and you can't tell which are whose. Before your first run, list them with their sizes
  (`node -p "require('os').tmpdir()"` gives the folder). List them again at the end and say whether
  the count grew. If it grew, your `TMPDIR` didn't take; say which runs leaked.
- **Don't change the harness's keep-on-failure behaviour.** It's how a red row is debugged, and the
  spec doesn't change it.
- **At the end**, measure and then delete your harness folder, your scratch target dirs and the
  mutation worktree. Report the space freed, per item.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. These are the spec's checkpoints.
1. **The model, the worker and the module check.**
   - First run the new checks on `main`'s code and quote the RED baseline: the module check fails on
     the missing exports and the worker line. Say if anything else is red.
   - Then `recipe.js` (the model, `loadPhoto`, `planEdit` with `Math.max(1, …)`), the one-line
     `worker.js` `load`, the module check, and the transpose deleted.
   - TABLE stays green with rows 15–16 at 803×1200 and 723×1080.
   - Open the PR **as a draft** with this commit.
2. **CROP_TABLE (C1–C35)**, the goldens, the `aspectSize` and orientation literal tables, the
   fraction readback, and placement. Every expected size is the spec's literal, **never** a value
   from `recipe.js` or crustyimg. Write each square-photo preset row in both orders, as the spec
   does: a row already in the order a mutant produces can't catch it.
3. **The replay.** `tests/ddphotos_upscale.py`, REPLAY_TABLE (R1–R17), `judgeReplay`, `psnr` and
   B11–B19.
   - R's original keeps R's EXIF block (`im.getexif().tobytes()` into `save(..., exif=...)`), or R7,
     R12 and R15 replay sideways.
   - Build each row's recipe **inside that row's `try`**, so a throw fails that row, not the run. M7
     depends on it.
   - Quote every replay row's PSNR against its form's floor: 53.5 dB for aspect, 59 dB for box.
4. **The controls and the summary.**
   - N1 and N2 append their C rows and keep their TABLE entries in order. Leave N3 untouched.
   - N4: `width = 0.5\n` → `width = 0.500625\n`, red on exactly C26, C31, C32, C33 and C34.
   - N5: content shifted **left** by `round(factor)` px, size kept, red on all 17 replay rows on
     `psnr`.
   - The summary line counts all five families on a green run, and the three P5 checks are
     relabelled `P5-pixel`, `P5-cli` and `P5-preview`.
5. **DEC-118**, written from the spec's text, with `affected_scope` as a block list (HANDOFF trap 7).
   Run `just decisions-audit` and quote its line.
6. **The mutation table, re-driven** (HANDOFF traps 12 and 14).
   - Run it in a scratch `git worktree` of your pushed head, never in the build worktree, with its own
     `CARGO_TARGET_DIR` and the harness `TMPDIR` above.
   - The mutants are JS-only, so the binary and wasm don't need rebuilding per row. Build them once
     and reuse them. Confirm each mutant edit is live: `git diff` is non-empty, and the run reads the
     mutated file.
   - For each row M1–M11, N4, N5 and the trap-14 rows:
     - apply **the spec's quoted edit**, not your own version (if the quoted line doesn't exist, make
       the closest edit, quote it, and say why);
     - run the full harness, with output redirected and `$?` read;
     - read the red set **off that run**, and compare it with the table. Call out every row the table
       doesn't list.
   - **M1** is invisible on page paths by design: only the orientation table check goes red. Say
     whether that still holds.
   - **M5** (crop after resize) leaves the rows where crop and resize commute green. Confirm the CLI
     golden still catches it.
   - Restore after each row and show it green. Then empty the harness folder.

   Paste each red set into Build Completion, trimmed to the lines that name rows. Where a cell is
   wrong, correct it in Build Completion and say so; don't edit the design's table.
7. **The rest of the matrix.** `src/` is untouched, but run `cargo fmt --check` and
   `cargo clippy --all-targets -D warnings` once on default, plus `just wasm-check`, to show nothing
   moved. Read the slim wasm size from the ddphotos-parity build and compare it with the spec's
   brotli 833,619 B (0 B growth).

## The things most likely to go wrong
- **Expected values from the code under test.** A `recipe.js` mutant changes the text both native and
  wasm run, so they still agree with each other. Only a literal size catches it.
- **The stored size leaking back in.** `planEdit` must get `loadPhoto`'s upright size. R-refit (375×1600,
  mark 37, capped) is the row that tells.
- **A square photo following.** It must write no key (M9; the square golden).
- **Fractions.** Write the shortest text that reads back (`String(n / side)`), never `toFixed`.
- **The crop's place.** It goes right after `auto-orient` and before `resize`, in all three recipes.
- **`info()` objects.** They are wasm-side; call `.free()`.
- **`"pub"` rows** pass `size: String(Infinity)`, so there's no resize.
- **The P5 relabel** changes only the three failure strings, and the transpose deletion only code and
  a comment. TABLE's literals and `GOLDEN_*` texts must be byte-for-byte unchanged.

## Guardrails
- **What must not change:** anything under `src/` or `tests/fixtures/`, `edit.js`, the `justfile`,
  `CHANGELOG.md`, `integrations/ddphotos/README.md`, and the recipe corpus. Paste `git diff --stat
  origin/main...HEAD -- src tests/fixtures integrations/ddphotos/edit/edit.js justfile CHANGELOG.md
  recipes`; it must be empty.
- No version bump, no release, no tag.
- Don't edit the spec's timeline or STAGE-060: the orchestrator and ship keep them.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI, and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, and take one snapshot
  when it exits. If it exits with legs still pending, say so; don't call that green. The
  ddphotos-parity job is advisory: read its log and quote the summary line with all five counts.
- **Rebase onto `main` before each push**, and redo the overlap check after each rebase.
- **If a permission check refuses a push, stop and report.** Name the commits and the exact command
  for the maintainer to run. Don't say it's pushed until `git ls-remote` shows it.
- **Shell traps:**
  - A piped command reports the **pipe's** exit code, so redirect and read `$?`.
  - zsh doesn't word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
    (`${R}:`).
  - macOS has no `timeout(1)`.
  - `rtk` can truncate `grep`, `ls` and `git log`, so use `/usr/bin/grep`, `/bin/ls` and
    `/usr/bin/git` for anything you count.
- **Shared state:** never share a `CARGO_TARGET_DIR` with another session. Write scratch files only
  in your session's scratchpad, never `/tmp` or the system temp folder. Don't leave PR-body files
  behind.
- **Never drive the maintainer's browser.**
- **Budget ~110 exchanges.** If you're past budget with the harness green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase
     on what changed. Give the count, and say whether it includes the spec;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not and why;
   - the harness summary line, both modes, and each replay row's PSNR against its floor;
   - every mutation row: the edit applied, and the rows red and green against the table, with every
     mismatch called out;
   - the fmt/clippy/wasm-check results, the slim size, and the CI legs;
   - the overlap check after your last rebase;
   - the temp-folder accounting: the system temp folder's `ddphotos-*` count before and after, and
     the space you freed, per item;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost at your last turn, after the last push.** The last two builds measured
   early and under-counted.
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - **Identify it by content, not recency:**
     `/usr/bin/grep -l "SPEC-161 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - **Dedup by `.message.id`.** Take the input and both cache counts once per id, and the **MAX**
     `output_tokens`. Include any `subagents/` transcripts.
   - **Price** at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation
     is ×1.25 the input rate, and cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-161 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks
     it ready. Never mark it ready yourself.
   - Title: `feat(ddphotos): the edit page's crop model, and a 4x replay of its recipe (SPEC-161)`.
   - Body per AGENTS §13:
     - a summary, including that EXIF-rotated photos are now planned from their upright size (a
       long watermark that used to be refused now fits), and that no crop control appears yet;
     - the spec metadata;
     - decisions referenced: DEC-110, DEC-112, DEC-113, DEC-114;
     - constraints checked;
     - new decisions: DEC-118;
     - breaking: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-161 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`. Then,
briefly, as plain lists:
- the RED baseline on `main`;
- AC completion, as a count, plus any row not met;
- the harness summary line in both modes, and the lowest replay PSNR per form against its floor;
- N1, N2, N4 and N5 against their expected lists;
- each mutation's red set against the table, any edit that differed from the quoted one, and every
  cell that turned out wrong;
- fmt, clippy, wasm-check, the slim size, and the CI legs, including ddphotos-parity's summary line;
- the overlap check after your last rebase;
- the files changed, against the spec's `## Outputs`, and the empty `git diff --stat` for the paths
  that must not change;
- the temp folders: the system temp folder's `ddphotos-*` count before and after, and the space freed
  per item;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
