# SPEC-165 — DESIGN prompt: the engine decides whether the page's watermark fits

Cycle: **design**. Start a fresh CLI session in its own worktree, on **Opus**:

```
just wt docs/spec-165-design -- --model opus
```

Your spec is
`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-165-the-watermark-s-fit-on-a-small-or-thin-crop.md`,
at `cycle: design`. Read its `## Frame` in full, ending with "Maintainer rulings (2026-11-28)".
**Ruling 1 overrides the frame's recommendation.** When no watermark size fits, the page blocks Make.
It doesn't leave the mark off. Everything in the frame that says "off" or "left off" is
for you to re-derive as "blocked".

The maintainer pastes this prompt into a fresh session, then pastes your final message back to the
orchestrator. That message must make sense on its own. **Write it as plain lists, not box-drawn
tables.**

## Rulings (settled — do not re-open)
1. **When no size fits, block Make.** If the watermark is on, every copy and recipe the page produces
   carries it. The page never decides by itself to make an unmarked copy.
   - The message names three ways out: make the box larger, use shorter text, or turn the watermark off.
   - Shrinking comes first. The page takes the largest size up to today's `watermarkSize` ceiling that
     the engine accepts on the output **and** on the output 2 px smaller per side. The 2 px is DEC-118's
     box bound under 2×.
   - The page blocks only when no size ≥ 1 passes.
2. **(c): ask the engine through a wasm export.** The page never copies the 5 % cap or the fit rule
   into JS. Option (b), the conservative JS bound, is rejected.
3. **One [M] spec.** It isn't split.
4. **No readable floor.** It stays the idea `watermark-readable-floor`.
5. **Not in this spec:** engine rules, `MIN_CROP` (it stays 64), the crop mode's DOM, a slim-bundle
   gate, a shrinking margin.

DEC-119's sentence is: "The edit page asks the engine whether its watermark fits, and blocks Make when
no size does." You write DEC-119's content into the spec. The build writes the file.

## What ruling 1 opens (your calls)
1. **What the page shows when Make is blocked.** Give your choice for each, with its literal text:
   - **The preview:** the crop without the mark plus the message, only the message, or something else.
     Today's "Preview failed: watermark refused: …" must never appear for the page's own mark. If it
     does, the page's check is wrong, and a test must catch it.
   - **The Make button:** disabled with a visible reason tied to it (`aria-describedby` or similar),
     or enabled and refusing on click. Say which, and why.
   - **The CLI and pixel recipe panels:** hidden, shown with the message, or shown as they are. They
     must never offer a recipe the engine refuses.
   - **Leaving the blocked state:** turning the watermark off, shortening the text or changing the
     size (and later, changing the box) must clear it on the next plan, with no reload.
2. **The plan's return shape.** `planEdit` runs synchronously on the main thread, and the wasm lives in
   the worker. The frame gives two options: a `plan` message to the worker, or a `fits` callback passed
   to `planEdit` so it stays pure for the Node harness. Pick one, and say:
   - what the Node harness passes in for the callback (a native-backed oracle or literal rows, never a
     JS copy of the engine's rule; trap 19);
   - how a blocked plan is represented (for example a `blocked: { reason }` field), and what every
     caller of `planEdit` does with it. List each caller with its `file:line`.
3. **The export's signature.** The frame measured `markMetrics(text, size)` at +697 B brotli. A
   verdict export (text, size, gravity, margin, outW, outH → accepted or the refusal's kind and
   message) may be smaller and keeps the rule in one place. Pick one, re-measure it with
   `just wasm-build-slim`, and record the Mac figure. CI's figure differs; the build records it.
   Say whether `src/operation/mod.rs` needs a new `pub` item, and confirm that no rule changes.
4. **The sentences.** Fix the exact text of:
   - the reworded `#wm-capped` hint ("Made smaller to fit this photo: …");
   - the blocked message;
   - the README's Watermark row. The frame's "or left off" is now wrong. Something like "and Make is
     blocked, with the reason, when it cannot fit at any size".
   List every behaviour sentence the build must drive on every exit path (AGENTS §15): preview, Make,
   both recipes, W/P/R/S, and the 4× and 3.75× originals.
5. **DEC-110 for the export.** On the same output size, the export's verdict must equal native
   `apply`'s verdict, in rows with literal expected values on both targets. Name the rows.

## Things in the frame to check before you build on them
Each one is only a claim until you drive it (AGENTS §15; HANDOFF traps 13 and 17).
1. **Without a crop, nothing changes on the fixture shapes.** Re-drive F10 and F11, and the golden
   rows' sizes (`GOLDEN_PIXEL_BOTH`, table rows 3, 7, 11), through the new rule.
2. **An uncropped 64×1600 photo is refused today.** The frame worked this out from `planEdit`'s
   arithmetic and didn't run it on a file. Make the fixture in your scratchpad and run it natively.
   Quote the refusal. That photo is the page row's subject, so it must block on the page with no crop.
3. **The replay edge.** The frame's 6000×4016 (3.75×) original refused the copy's accepted mark in
   30 of 30 cases. Under the new rule, the 2 px headroom must make the page block that edit, or pick
   a size the CLI accepts. Re-drive the row natively.
   - **Portrait originals weren't driven.** Drive at least one portrait fractional original, the
     mirror of the 3.75× case. If 2 px doesn't hold there, report it and put it to the maintainer.
     Don't widen the headroom yourself.
4. **The height axis.** The frame says the engine checks both axes once (c) is in. Drive a case where
   the height check alone refuses: a box under 64 px is fine for a native probe. Show the export
   reports it.
5. **Each literal in F1–F11.** The frame took them from its model, which agreed with native on 420
   of 420 sampled runs. Re-drive every literal natively before you write it, and convert the "off"
   rows (F3, F5, F9 and the replay row) to "blocked".

## Tests: each one must be able to fail
- **The mutation table.** Drive it yourself on a prototype in your scratchpad. Keep the frame's M1–M8
  where they still apply:
  - **M7 becomes:** a blocked plan still lets Make run, or shows no message, or a recipe panel offers
    the refused mark.
  - **Add mutants for the blocked state:**
    - the block is not cleared when the watermark is turned off;
    - the preview shows "Preview failed" instead of the message;
    - the export returns "accepted" for every input.
  - Each row quotes its exact edit, every test expected red with **each red row's failure message
    from a run**, and the green tests (traps 12 and 13). A control edits a scratch copy, never the
    real file (trap 14). Mark any row you couldn't run as "not driven".
- **Trap 19:** both sides of the parity harness read `recipe.js`, so every guard on the new rule is a
  literal. The shared mutant (headroom 0) must be caught by a literal. Say which literal catches each
  mutant.
- **Trap 20:** thin strips go in both orientations, and the square goes in both orders.
- **Trap 22:** the page row marks the old window, waits for the new document at its URL, and runs
  once against a server that delays the body.
- **Nothing else changes.** Say what pins it: the existing tables, the replay rows apart from the new
  one, the orientation and alpha rows, N1–N5, and the parity summary line's counts, with each new
  count derived. No engine rule changes.

## Temp folders (set up before the first harness run)
- Before the first run, count the system temp folder's `ddphotos-*` folders and their size. Delete
  none of them: some belong to other sessions.
- Run every harness and headless command with `TMPDIR` set to a folder in your scratchpad, and
  confirm after the first run that the folders landed there. Point Chrome's `--user-data-dir` at the
  scratchpad too.
- Never run the harness while you're editing files it reads.
- Empty the folder after you've read each red run.
- At the end, count again and report both counts.
- Large originals (6000×4016) go in the scratchpad and are deleted at the end.
- Write the build's temp-folder instructions into Implementation Context.

## Rules every design session follows

**You are the design cycle only.** You write the Acceptance Criteria, the Failing Tests and the
Implementation Context. You don't write the code. A separate build session then works from your text
alone, and a separate verify session checks it, so everything the builder needs must be in the spec
(AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you're in the
maintainer's main checkout and not a worktree, stop and say so. Then build the overlap map with
`gh pr list --state open --json number,files` and `git branch -r`. PR #237 is parked; list it, and
don't touch it.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 and §16 habit 6.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for",
   especially 11–14, 17, and 19–22.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`:
   your entry and the DOM entry after it, whose live "Made smaller" note reads your verdict.
4. `/guidance/constraints.yaml`, then DEC-045, DEC-104, DEC-110, DEC-112 and DEC-118.
5. In `specs/done/`, SPEC-128 and SPEC-143 (the watermark's rules and their reflections) and SPEC-164
   (the page check and its punch lists).
6. The engine: `refusal()` and `WATERMARK_COVERAGE_CAP_PERCENT` in `src/operation/mod.rs`,
   `src/text.rs`, and `src/wasm.rs` with its tests in `tests/wasm_roundtrip.rs`.
7. The page code in full: `integrations/ddphotos/edit/recipe.js`, `edit.js`, `worker.js`, `index.html`.
   Then `tests/ddphotos_parity.mjs`, `tests/ddphotos_page.mjs`, `tests/ddphotos_upscale.py`, and
   CI's `ddphotos-parity` job.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive before you write.** Use native `apply`, the real slim wasm, Node against `recipe.js`, and
  a headless Chrome you start yourself. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.**
- Expected values come from literals or native runs, never from the code under test.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Inputs, Outputs, Acceptance Criteria, Failing Tests and
Implementation Context, and keep `## Frame`. If the frame is wrong somewhere, say so in a short
`### Design corrections to the frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go straight to `main` (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-165 build`.
2. Set `agents.architect` and `agents.implementer` to real model ids. They still read the
   template's `claude-opus-4-7`.
3. In STAGE-060's backlog, change SPEC-165's marker from `(framed 2026-11-27)` to
   `(designed <date>)`. Keep `[M]` unless you found a different size.
4. In the timeline, mark design `[x]` with a 2–4 line summary, and put `prompts/SPEC-165-build.md`
   on the build line.
5. Commit with `git commit -s`. **The DCO sign-off is mandatory.** Use the message
   `docs(SPEC-165): design the engine's watermark-fit check and the blocked Make`, with the trailer
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
6. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
   - Check the push with `git ls-remote origin refs/heads/main`.
   - ⛔ **If a permission check refuses the push, stop and report it** (trap 21). Name the commits,
     and give the maintainer the exact command to run.
7. Read the commit back with `git show --stat HEAD`.

### Guardrails
- ⛔ **What you may change:** the spec, the timeline and the stage file. Nothing else.
  - No code changes, no DEC file, and no pushed probe branch.
  - No version bump, no tag, and no CHANGELOG edit.
  - Never edit PROJ-011's files.
- **Never drive the maintainer's browser.** A headless Chrome you start yourself, with its profile in
  your scratchpad, is fine.
- **Shell traps:**
  - A piped command reports the pipe's exit code, so redirect output and read `$?`.
  - zsh doesn't word-split unquoted parameters, and `${R}:` needs braces.
  - macOS has no `timeout(1)`.
  - Use `/usr/bin/grep`, `/bin/ls` and `/usr/bin/git` for anything you count.
- **Scratch space:**
  - Never share a `CARGO_TARGET_DIR` with another session.
  - Before you finish, remove the prototype, the target dirs, the originals and Chrome's profiles.
    Measure them first, and say how much space that freed.
- **Budget ~70 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`.
- **Find it by its content, not by which file is newest:**
  `/usr/bin/grep -l "SPEC-165 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.
- **Dedup by `.message.id`.** Take the input count and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any `subagents/` transcripts.
- **Price** at the model in `.message.model`: Opus is $5/$25 per MTok and Sonnet $3/$15.
  cache_creation costs ×1.25 the input rate, and cache_read ×0.10.
- **Measure at your last turn, after the push.** Append a `design` entry to `cost.sessions`, commit
  it as a final small commit, and push it the same way.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-165 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main` (or "not pushed" and why). Then, briefly:
- each of the five frame checks, with what you drove (the portrait original and the uncropped
  64×1600 first);
- any correction to the frame;
- the blocked state: what the preview, Make, both panels and the hints show, with the literal text;
- the plan's return shape and the worker or callback choice, with every caller;
- the export's signature, its measured size, and any new `pub` item;
- the AC list, one line each;
- the rows by family (fit, replay, export, page), with counts and how you derived them;
- the mutation table: each row with its quoted edit, its red tests with their failure messages, its
  green tests, and whether it was driven;
- what pins "nothing else changes";
- the docs, the behaviour sentences, and DEC-119's content;
- the overlap map, from the PRs' file lists;
- anything the build prompt must carry (checkpoints, CI legs, wasm, Chrome, the temp-folder rules);
- any question for the maintainer, with your recommendation;
- the system temp folder's `ddphotos-*` counts before and after, and the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own line.
