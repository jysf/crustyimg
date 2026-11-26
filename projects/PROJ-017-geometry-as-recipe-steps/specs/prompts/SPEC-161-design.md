# SPEC-161 — DESIGN prompt: crop on the ddphotos edit page, the recipe model and the 4× replay

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-161-design -- --model opus
```

This session starts from the stub the frame pushed to `main` on 2026-11-22. Spec file:
`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-161-crop-on-the-ddphotos-edit-page-the-recipe-model-and-the-4x-replay.md`.
**Read its `## Frame` section first, then "Maintainer rulings (2026-11-22)" at its end.** The rulings
reverse the frame on presets: a preset emits `aspect`, not a box. Every paragraph above the rulings
that says "a preset emits a box" is superseded. The frame is otherwise your binding scope:
- the split;
- the oriented-base fix for EXIF-rotated photos;
- the size bound between the copy and the original;
- the watermark refit (W-refit and R-refit);
- `cliRecipe` on the original;
- the parity and replay rows, and the negative controls;
- the wasm size;
- the fold;
- what is testable where.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
From the stub's "Maintainer rulings (2026-11-22)":
1. **Split.** This spec is the recipe model and the 4× replay, everything Node can test. The crop mode
   (the drag box, preset buttons, keyboard and touch, the CLI panel's wording) is STAGE-060's next
   entry.
2. **The interaction** is a crop mode on the oriented base, with a DOM box and one render on Done. The
   second half builds it, but your model must serve it.
3. **An untouched preset emits `aspect`; a dragged or resized box emits a box.** The recipe must be
   readable and reusable: a preset copied from the page says `aspect = "3:2"` and works on any photo.
   So the page has two crop forms, and the rows cover both.
   - The page's predicted size must equal what wasm makes from the copy, for `aspect` too.
   - At the 4× replay, `aspect` rows lose "exact at an integer multiple". That cost is accepted.
     Design drives the worst case, and the DEC states the driven bound per form. **Don't assume
     1 px.**
4. **A preset that follows the photo emits `orientation = "match"`.** The key's spec ran first.
5. **The replay DEC: yes.** Its number is allocated at design (see below).

Settled elsewhere:
- this spec changes no Rust (the slim wasm already has `crop`);
- no save flow, sidecars, flip/pad/trim on the page, or ddphotos code (STAGE-060
  `explicitly_does_not`);
- comments and user-facing text name no spec, stage or DEC id.

## What changed on `main` since the frame (check each before you build on it)
The frame was written on `7d0e7fd`. Since then:
1. **SPEC-163 shipped** (PR #272 as `3de9f64`, archived in `specs/done/`): `orientation = "match"` on
   an aspect `crop` or `pad`, and DEC-112 has a rule 8.
   - Read its `### The page port (for SPEC-161's design)`. It gives `followOrientation(a, b, w, h)`
     (integer comparisons, run on the **oriented** base) and four driven sizes for 3:2: 1600×1071 →
     1600×1067, 1071×1600 → 1067×1600, 1071×1071 → 1071×714, 1071×1072 → 715×1072.
   - Use it as written, or say exactly why not.
   - `recipe.js` has **no** `aspect_size` port today (read it). Write one, line for line from
     `crop.rs`.
2. **The fixtures the frame planned exist.** SPEC-163 added `photo_portrait.webp` (P, 1071×1600),
   `photo_square.webp` (S, 1071×1071) and `photo_near_square.webp` (N, 1071×1072). It made them with
   ImageMagick and recorded them in `tests/fixtures/ddphotos/README.md`.
   - Confirm they serve this spec. Add a fixture only if you can name a row that needs one.
3. **The harness grew** (`tests/ddphotos_parity.mjs`, 790 lines):
   - `ORIENTATION_ROWS` (13 × crop/pad = 26 rows, its own `ORIENTATION_ROW_COUNT`);
   - `INPUTS` with P, S and N;
   - **a self-test N3** ("the orientation key stripped").

   So the frame's new controls can't be called N3 and N4. Name them **N4 (one pixel of crop)** and
   **N5 (replay framing shift)**, or choose names and say so. Decide how your rows sit beside SPEC-163's:
   - reuse its row shape;
   - say whether any of its rows already covers one of yours;
   - say how the summary line will show every row family ran. SPEC-163's verify noted the 26 rows'
     count prints only on failure; that's filed as `aspect-orientation-loose-ends`, and you may take
     it.
4. **The fold is still to do.** STAGE-060 marks "Label the parity harness's table checks" as folded
   into SPEC-161. The three `P5:` lines are still at `tests/ddphotos_parity.mjs:406–408`.
5. **The rows 15–16 transpose** the frame says to delete is still there (`:142–143`, `:360–362`).
6. **DEC numbers:** DEC-116 and DEC-117 are taken. At Step 0, find the next free number
   (`/bin/ls decisions`, and `/usr/bin/grep -rn 'DEC-118'` over `projects/` and `guidance/`). Allocate it
   for the replay DEC and say which.

## Things in the frame to check before you build on them
Drive these on `main` (release build and slim wasm, both made by `just ddphotos-parity` into a fresh
target dir) and on your prototype:
- **The worst case for `aspect` at replay, re-driven** (ruling 3).
  - Use the frame's ratios (at least 1:1, 4:5, 3:2, 16:9 and 9:16), its sizes, and its copy/original
    pairs (exact 4×, the camera sizes, the 1700 px case, and a sweep of random originals).
  - Run it **with `orientation = "match"`**, on landscape, portrait, square and near-square copies.
  - Give the worst difference per form (box and aspect) by original/copy ratio band: below 2×, 2× and
    up, and exact integer multiples. These numbers become the DEC's bound.
  - Give a negative control that shows the sweep can see a difference: for example, a deliberately
    wrong rounding in the model turns cases red.
- **The page's prediction equals wasm on the copy, for both forms.** Feed `planEdit` with the
  `aspect_size` port and `followOrientation`, and compare with `info()` of wasm's output:
  - over every preset × W, P, S, N and R;
  - over a sweep of copy sizes, including near-squares and 1–12 px sides.

  Exact is the bar. Any miss means the port is wrong.
- **The oriented-base fix.** Drive the worker's `load` sequence in Node on the real slim wasm: R's
  `info()` gives 1600×1071, and the oriented base gives 1071×1600. Confirm `planEdit` then needs no
  transpose. Confirm rows 15–16's sizes (803×1200, 723×1080) still hold without it.
- **W-refit and R-refit.** Re-drive both with the new `planEdit`. R-refit must tell the fix from the
  stored-size bug in output size, watermark size **and** `capped`. If the preset form changes either
  row's numbers, say so.
- **Gravity.** A centred preset: does it emit no `gravity` (crop defaults to `Center`, `crop.rs:134`)
  or an explicit `gravity = "center"`? Pick one. Pin the recipe text with a golden, and say why it
  is readable.
- **The "swap" button** the frame mentions (3:2 on a portrait photo shown as 2:3, with a swap).
  - With the key, what does a swapped preset emit? The aspect as written without the key, or a box?
  - The model must express it even though the button is the second half's. Decide, and give the
    recipe text for each state.
- **Fractions for the box form.** The frame drove `String(n / side)` over 5,729,060 boxes, all reading
  back to the same pixel edges. Re-run a smaller version on your port, to confirm the port and not
  only the arithmetic.
- **The PSNR floor and N5.** Measure the replay's PSNR on a Pillow-made 4× original, for every replay
  row of both forms.
  - Set the floor from the measurement. Then drive N5 (a one-copy-pixel framing shift with the size
    kept) and show it falls **below** the floor on every replay row it targets.
  - If it doesn't, the floor can't see a 1 px shift. Say so, and propose what can (for example,
    edge-strip comparison or a smaller crop). Don't build on a floor N5 can't beat.
  - For `aspect` rows, whose replay sizes may differ by the driven bound, say how the comparator
    aligns them before PSNR, or that it compares only sizes.
- **Size.** The frame said [M] with a box-only model, about 41 native-against-wasm rows and about 22
  replay rows. Two crop forms and the key may push it to [L].
  - Recount with the derivation next to every count.
  - If it's [L], **stop at that finding**. Report `SPLIT` with a proposed cut (for example, the
    model, the oriented-size fix and native-against-wasm rows first, then the replay mode and its
    DEC). Don't design past it.

## Design calls that are yours
- **The model's shape in `recipe.js`.** Exported functions and their signatures:
  - the crop state (`null`, `{aspect: [a, b], follow, gravity?}` or `{box: {x, y, w, h}}` in oriented
    copy pixels, or your own);
  - the preset's box (for showing it and for the second half's drag start);
  - `planEdit` with the crop ahead of the resize;
  - the crop step in `pixelRecipe`, `cliRecipe` and `previewRecipe`, after `auto-orient` and before
    `resize`.

  Give each with `file:line` and the bytes `recipe.js` grows by.
- **`worker.js` and `edit.js`.** The oriented-base render at `load`, the size it reports, and the one
  `edit.js` call site. There is no crop control yet. Say what the user sees change: R's plan and
  watermark fit.
- **The replay mode.** How the harness makes the original (Pillow, at test time, nothing large
  committed), which rows replay, the comparator limbs (sizes per the DEC's bound, then PSNR), and its
  B-cases.
- **Every row**, each with its input, form, recipe text or golden, and a literal expected size from
  the frame's numbers or an independent model, **never from `recipe.js` or crustyimg**. Mark each
  row as one of: native against wasm exact; within DEC-110; or replay within the new DEC.
- **The new DEC's full text**, including `affected_scope` in block-list form (HANDOFF trap 7). Its
  bounds are the ones you drove, per form and ratio band. Build writes it (it's a new record, not an
  amendment); say so in the file list.
- **What doesn't change, and what pins it:**
  - TABLE's 16 rows and their `GOLDEN_*` texts (moved only by the P5 relabel and the transpose
    deletion: say exactly how);
  - SPEC-163's 26 rows and N3;
  - N1 and N2's expected-red lists (say how they grow);
  - the recipe corpus;
  - `src/`.

  The build's proof: no `src/` diff, and the fixture diff.
- **Docs.** The page's README controls text is out of scope ("Test-bed round and README"). Say whether
  any other surface describes what the page plans for an EXIF-rotated photo, and what changes there.
  List every behaviour sentence you add and every path it names.
- **CHANGELOG.** One "Changed" line for the oriented-size fix, if users of the page see it. The crop
  itself isn't visible until the second half, so say whether it gets a line now or then.
- **The wasm size.** Confirm 0 B on the slim bundle with `just wasm-size`, and state `recipe.js`'s
  growth.

## Tests: each one must be able to fail
HANDOFF traps 10, 12, 13, 14 and 17, AGENTS §15, and SPEC-163's ship lessons bind here. **Drive every
mutation row on a prototype; don't reason it.** Each row quotes its exact mutant edit, names the
location, lists every test expected red and every test expected green, and is marked *driven* or
*reasoned*. Read the red set off the prototype's run of the final test list.

**One rule from SPEC-163's verify matters most here.** A mutant in `recipe.js` changes the recipe text
that **both** native and wasm then run, so they still agree with each other. Only a literal expected
size, or an independent model, catches it. So every preset and box row asserts a literal size, and
the table has at least one `recipe.js` mutant **driven through the full harness** to show which rows
catch it. Also write every square-photo preset row in both orders (`3:2` and `2:3`): a row already in
the order a mutant produces can't catch that mutant.

Mutants, at least:
- `followOrientation`'s comparison `w > h` → `w >= h`, and the swap inverted;
- the `aspect_size` port's short side rounding changed (floor for half-up);
- `planEdit` fed the stored size, not the oriented size (R-refit must go red; trap 10);
- the crop placed after `resize` in `cliRecipe` (the order test must fail: drive it, because crop
  and resize may commute on some rows);
- the box fractions written with fixed precision (`toFixed(4)`);
- an untouched preset emitting a box (the golden must go red);
- the `orientation` key dropped from a following preset (P's preset rows must go red);
- N4 and N5 themselves: drive each, and show its expected-red list exactly;
- trap 14: restore `main`'s `recipe.js`, `worker.js` and `edit.js` verbatim, and list everything that
  goes red.

**Every count in the spec carries its derivation** (the command, or the sum) next to the number.

## Overlap with other open work
Run `gh pr list --state open --json number,title,files` at Step 0. #237 (heic via `heifer`, parked)
is expected; say whether it overlaps. Write the map into Implementation Context **from the PRs' file
lists**, and tell the build to rebase onto `main` before each push and to re-check the map after each
rebase.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and Implementation
Context. You don't write the code or the DEC file; the DEC's text goes in the spec, and build writes
the file. Separate build and verify sessions follow: one builds from your text alone, and another
verifies it, so everything the builder needs must be in the spec (AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you're in the
maintainer's main checkout, not a worktree, stop and say so. Then build the overlap map and allocate
the DEC number.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 (the coverage map in
   "During design", "a bound or 'can't happen' is a claim until its worst input is driven", the
   refusal rule) and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for". These
   bind PROJ-017.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`: your entry, the crop-mode entry
   after it, the folded relabel, and the slim-bundle finding.
4. `/guidance/constraints.yaml`, then DEC-110, DEC-112 (with rule 8), DEC-113 and DEC-114.
5. **SPEC-142** (the parity harness, and what it deferred to this spec) and **SPEC-163** (the page
   port, its parity rows and N3, and its verify and ship notes), both in `specs/done/`.
6. The page code in full: `integrations/ddphotos/edit/recipe.js`, `worker.js` and `edit.js`, then
   `tests/ddphotos_parity.mjs`, `tests/ddphotos_decode.py`, and the `ddphotos-parity` recipe in the
   `justfile`.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive before you write.** Use Node against the real slim wasm and a release binary, and give
  every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC that names two things needs a
  test for each.
- Expected values come from literals or an independent model, never from the code under test.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Inputs, Outputs, Acceptance Criteria, Failing Tests and
Implementation Context, and keep `## Frame`. If you find the frame wrong, say so in a short
`### Design corrections to the frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-161 build`.
2. Update STAGE-060's backlog:
   - change SPEC-161's marker from `(framed 2026-11-22)` to `(designed <date>)`, keeping `[M]` or the
     size you found;
   - correct any sentence in SPEC-161's entry that still says presets emit a box.
3. Update the timeline: mark design `[x]` with a 2–4 line summary, and put `prompts/SPEC-161-build.md`
   on the build line.
4. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-161): design the page's crop model and the 4x replay`.
5. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
   - Check with `git ls-remote origin refs/heads/main`: a "Changes must be made through a pull request"
     notice can still mean the push landed.
   - ⛔ **If a permission check refuses the push, stop and report.** Name the commits and give the exact
     command for the maintainer to run. Don't say it's on `main` until `ls-remote` shows it.
6. Read back `git show --stat HEAD`, and run `just status`.

### Guardrails
- ⛔ **What you may change:** the spec, the timeline and the stage file only.
  - No code changes, and no DEC file.
  - No pushed probe branch.
  - No version bump, no tag, no CHANGELOG edit.
  - Never edit PROJ-011's files.
- **Shell traps:**
  - In an interactive terminal `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout
    to a file.
  - A piped command reports the pipe's exit code, so redirect output and read `$?`.
  - zsh doesn't word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
    (`${R}:`).
  - macOS has no `timeout(1)`.
  - `rtk` can truncate `grep`, `ls` and `git log` output, so use `/usr/bin/grep`, `/bin/ls` and
    `/usr/bin/git` for anything you count.
- **Scratch space:**
  - Never share a `CARGO_TARGET_DIR` with another session.
  - Remove the prototype worktree, its target dir and the 4× originals before you finish, and say how
    much space that freed (measure before you delete).
  - Write scratch files only in your session's scratchpad, never `/tmp` or the system temp dir.
  - The harness's own `mkdtempSync(tmpdir())` is existing code; leave it.
- **Never drive the maintainer's browser.**
- **Budget ~70 exchanges.** The replay sweep and the driven mutation rows are the extra.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- **Identify it by content, not recency:**
  `/usr/bin/grep -l "SPEC-161 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.
- **Dedup by `.message.id`.** Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any `subagents/` transcripts.
- **Price** at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is
  ×1.25 the input rate, and cache_read ×0.10.
- **Measure at your last turn, after the push.** Append a `design` entry to the spec's `cost.sessions`
  in a final small commit, and push it the same way.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-161 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main` (or "not pushed" and why). Then, short:
- the DEC number allocated;
- each check above, with what you drove:
  - the `aspect` replay bound per form and ratio band, with its negative control;
  - prediction against wasm;
  - the oriented base;
  - W-refit and R-refit;
  - gravity;
  - the swap;
  - fractions;
  - the PSNR floor against N5;
  - size;
- any correction to the frame;
- the model's shape, one line per function, with `file:line`;
- the AC list, one line each;
- the rows, by family, with counts and their derivations;
- the Failing Tests and the mutation table: each row with its quoted edit, red and green tests, and
  driven or reasoned;
- the DEC's bounds, and its `affected_scope`;
- what pins "nothing else changes";
- docs and CHANGELOG decisions;
- the wasm size and `recipe.js` growth;
- the overlap map, from the PRs' file lists;
- anything the build prompt must carry (checkpoints, CI legs, tools such as Pillow and `wasm-pack`);
- the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own line.
