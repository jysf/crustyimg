# SPEC-164 — FRAME prompt: crop on the ddphotos edit page, the crop mode

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-164-frame -- --model opus
```

**Reserved: SPEC-164.** The highest spec id on `main` is SPEC-163, and SPEC-154 to SPEC-158 are
reserved by PROJ-011's STAGE-063. Before you allocate, confirm SPEC-164 is unused:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-164' "$b" -- . ; done
```

The only expected hits are this prompt and the reservation note on your STAGE-060 entry. If a spec
file of its own or another reservation uses SPEC-164, stop and report. Don't pick another number.
⚠ `just new-spec` skips any id named in prose, so it will likely offer SPEC-165 or later; take
SPEC-164 anyway (see "Allocate the id").
If you recommend a DEC, it takes the next free id, which is DEC-119 on `main` today. Recommend its
content; don't write it.

Your item is the STAGE-060 Spec Backlog entry "**Crop on the page: the crop mode.**" It is the second
half of the crop entry, split at SPEC-161's frame. Read it in full. Touch no other item except as
this prompt says.

## Why this spec exists
SPEC-161 shipped everything Node can test:
- the crop model in `recipe.js`: `aspectSize`, `followOrientation`, `presetCrop`, `swapCrop`,
  `cropBox` and `planEdit(photo, form, crop)`;
- the upright photo size, from `loadPhoto`;
- 36 crop rows and 17 replay rows within DEC-118.

**But nothing on the page calls the crop model yet.** `edit.js` is unchanged, so a photo owner still
can't crop. This spec adds the part they touch: a crop mode, presets, a box they drag, and the
wording that tells them what the CLI recipe will do. It's the first STAGE-060 spec whose main
risk is in the browser, not in the recipe.

## Settled; don't re-open
- **The recipe model** (SPEC-161 and DEC-118):
  - an untouched preset emits `aspect = "a:b"`, plus `orientation = "match"` unless the photo is
    square or the preset is 1:1;
  - a swapped preset writes the aspect as shown, with no key;
  - a dragged box emits four fractions;
  - the crop runs after `auto-orient` and before `resize`.

  **This spec calls those functions and adds no recipe logic.** If the UI needs a value the model
  doesn't give, say so as a finding. Don't plan to compute it in `edit.js`.
- **The interaction's shape** (SPEC-161's `## Frame`, "The interaction", ruled 2026-11-22):
  - a crop mode over the oriented base image, rendered once at load;
  - a DOM box that renders nothing while dragged;
  - "Done" schedules one finished preview;
  - Pointer Events, with `touch-action: none`;
  - a focusable box that arrow keys move and Alt+arrow resizes;
  - presets 1:1, 4:5, 3:2 and 16:9 that follow the photo's orientation, with a swap;
  - a live size line.

  The frame works out the details inside that shape and doesn't replace it.
- **Units** (DEC-112), **step order**, **parity** (DEC-110) and **a step acts on the buffer it
  receives** (DEC-113), as in SPEC-161.
- **Out of the stage** (`explicitly_does_not`): no save flow or sidecar, no ddphotos code change, and
  no flip, pad or trim on the page.
- **The page is a static add-on**, built from `integrations/ddphotos/edit/` and installed by
  `just ddphotos-edit-install`, on the slim wasm (`pkg-slim`).
- **Never the maintainer's browser.** Any browser check runs in headless Chromium. The test bed's
  browser round on all three servers is a separate STAGE-060 entry ("Test-bed round and README").

The frame's job is everything those leave open.

## What already exists (read it; don't assume)
- **The page:** `integrations/ddphotos/edit/`, meaning `edit.js` (377 lines), `recipe.js`,
  `worker.js`, `index.html`, `edit.css` and the README. Read how `edit.js` builds `photo` from
  `meta`, runs `runPreview` (debounced at 200 ms), shows the CLI panel and the "Made smaller to fit
  this photo" note. Read what the worker returns from `load`: SPEC-161's verify found `loadPhoto`
  returns the upright size and an upright base JPEG with no Orientation tag. Check that, and say
  what the page does with that base today.
- **SPEC-161** in `specs/done/`: its Frame (the interaction and the split), the rulings, Design's
  model section, DEC-118 and its ship reflection.
- **The parity harness:** `tests/ddphotos_parity.mjs` and its module check (`checkModules`). The
  check pins which names `edit.js` and `worker.js` may define. A crop mode that defines one of
  them turns it red.
- **Browser checks that exist today.** Find every headless-browser check in this repo: CI's
  `pages.yml` "build + browser smoke" job, any Playwright or Chromium use in the `justfile` or
  `scripts/`, and the test bed's checks
  (`~/claudia/web/jysites/fogcityphotosandbeyond/crustyimg-ddphotos-testbed`, no git remote; read
  `specs/prompts/testbed-editor-clean.md` for its rules, and its `CLAUDE.md` only for what its
  checks can reach). Say which of them could drive this page, and which can't.

## Questions this frame must answer
1. **The coordinate chain.** The box is dragged in screen pixels over a scaled image. The model
   works in copy pixels on the upright copy. Write down every conversion: screen to displayed
   image, displayed image to copy, and copy to `cropBox`/the fractions. Say where rounding happens
   and which function owns each step. Is there a pure function `edit.js` needs that `recipe.js`
   doesn't have? If so, it belongs in `recipe.js`, and Node can test it. Recommend.
2. **Preset, drag and swap: the state machine.** Name each state (no crop, untouched preset,
   swapped preset, dragged box) and each transition: choosing a preset, swapping it, dragging,
   Reset, Done, leaving and re-entering crop mode, and changing another form field while a crop is
   set. Say what recipe each state emits, using SPEC-161's functions. Include the edge cases:
   - swap on a square photo, and on 1:1;
   - a preset after a drag;
   - a drag that ends exactly on a preset's box (it stays a box, by ruling 1);
   - the box made smaller than the engine's minimum crop.
3. **What wasm renders, and when.** Count the renders for one session: load, enter crop mode,
   drag, Done, a form change outside crop mode. Say whether the oriented base is shown at copy size
   or scaled, and what that costs in memory on a large copy. A drag must not queue a render on
   each pointer move.
4. **Keyboard, touch and screen readers.** The minimum: arrow keys move the box one copy pixel
   (Shift: 10), Alt+arrow resizes, presets, Reset and Done are real buttons, and a live text line
   gives the box's size in pixels. Say which ARIA roles and live-region politeness fit. Say what a
   touch drag does near the image's edge, and whether pinch is out of scope. Recommend the minimum
   that ships.
5. **The live size line and the watermark note.** The size line reads `planEdit`'s prediction for
   the current box and form. Say when "Made smaller to fit this photo" appears during crop mode,
   and when it changes.
6. **The CLI panel.** Its wording today, and the one sentence the backlog entry adds: run the
   recipe on the original or any file at least as large as the copy, because a smaller file gives
   a smaller output or a refused watermark (DEC-118 rule 5). SPEC-161's verify drove that case: an
   800×536 original with W 3:2 at 1080 and the watermark gave the page 1080×720 and the CLI 800×533,
   exit 0, with no warning. Write the sentence you recommend, and say where it shows.
7. **How the build is checked without the maintainer's browser.** Split the spec's claims into
   three groups:
   - what Node can test (pure functions, the emitted recipe text for each state);
   - what needs headless Chromium (pointer and keyboard events, the DOM box, the render count);
   - what only the test bed's round can show.

   For the headless part, recommend where it lives: in this repo as a new check, or in the test
   bed's round. Say what it costs CI, and say its negative control (something that must turn it
   red). Per HANDOFF trap 11, a check that can skip must fail where its coverage is required.
8. **Mutations a design can table.** Name five or more mutants that a working UI test must catch,
   each in the state machine or the coordinate chain. For example: drag deltas not scaled by the
   display ratio, swap emitting the key, Done not clearing the dragged state, an arrow key moving
   one screen pixel. Say which group from question 7 catches each. Traps 13 and 19–21 apply.
9. **wasm size.** This spec should add no Rust and no wasm. Confirm with the numbers from
   `just wasm-size` on `main`.
10. **Fold or leave.** For "Test-bed round and README" and "Straighten on the page", recommend
    leave, fold or move, with the reason. Folding is allowed only if the size doesn't change. Don't
    move anything; recommend.
11. **Docs.** The page README's controls section, the CLI panel, `integrations/ddphotos/README.md`,
    and any doc that lists what the page can do. Recommend which wording belongs here and which
    belongs to the test-bed entry. Per AGENTS §15, list each behaviour sentence the build must
    drive on every exit path it names.
12. **Size.** The backlog says [M]. A state machine, a coordinate chain, keyboard and touch, the
    size line, the CLI sentence and a headless check may be more than [M]. If it is, recommend a
    split, and say what each half ships on its own.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is a go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. You recommend its content. Separate design, build and verify sessions
follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather than
a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §12, §13 git, §15 (its rules bind every cycle)
   and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order and
   "Explicitly out of scope".
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`:
   your item, every open entry, the `delivers` lines, the Success Criteria and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for",
   especially traps 11, 13, 14 and 19–21 (19–21 are new as of 2026-11-27).
5. DEC-110, DEC-112, DEC-113 and DEC-118.
6. SPEC-161 and SPEC-163 in `specs/done/`, with their ship reflections.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code.** Confirm that nothing in `edit.js` calls the crop model
  today, and that the worker's `load` returns the upright base. Run `just ddphotos-parity` once and
  record its summary line (it should read 16 table, 36 crop, 17 replay, 26 orientation and 2 alpha
  rows).
- **Drive the coordinate chain (question 1)** on numbers. Take W (1600×1071), P (1071×1600) and R
  (stored 1600×1071, upright 1071×1600). Use a display width of 375 px (a phone) and 1200 px. Push a
  box through the chain both ways, and report the worst copy-pixel error from screen rounding. A
  bound is a claim until its worst input is driven (trap 17).
- **Prototype the headless check, if you recommend one.** Show that it can load the page, drive one
  drag and read back the emitted recipe. Keep the prototype in your scratchpad, and never commit
  it.
- **Size the verification, not just the code** (AGENTS §2): states × transitions × orientations
  (landscape, portrait, square, EXIF-rotated) × pointer, keyboard and touch.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Temp folders (if you run the harness)
The harness keeps its `mkdtempSync(tmpdir())` folders whenever a run fails.
- Before the first run, count the system temp folder's `ddphotos-*` folders. Don't delete any of
  them: some belong to other sessions.
- Run every harness command with `TMPDIR` set to a folder in your scratchpad, and confirm the
  folders landed there.
- Never run the harness while you're editing files it reads.
- At the end, count the system temp folder's `ddphotos-*` again and report both counts.

### Allocate the id
Run `just new-spec "<title>" STAGE-060 PROJ-017`. If it gives anything other than SPEC-164, `git mv`
the spec and its timeline to SPEC-164, and fix the id inside both. Then run
`just advance-cycle SPEC-164 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references` (DEC-110,
  DEC-112, DEC-113, DEC-118, SPEC-161, SPEC-163), and `cycle: frame`.
- `## Context` and `## Goal`: what a photo owner can't do on the page today, and what they can when
  this ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go, no-go or split);
  - the premise check, with evidence;
  - the coordinate chain, driven;
  - the state machine and the recipe each state emits;
  - the render count;
  - keyboard, touch and screen readers;
  - the size line and the watermark note;
  - the CLI sentence;
  - what is testable where, and the headless check you recommend, with its negative control;
  - the mutants;
  - the wasm size;
  - fold or leave;
  - docs;
  - sizing and why, and any split;
  - scope in and out;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-164-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-164 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-060's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-164** (framed <date>)`, and drop the entry's reservation note. Fold another entry only if you recommend it **and** it
   doesn't change the size: mark it `- [x] (folded into SPEC-164)`. Otherwise leave it and put it to
   the maintainer. On a split, file the second half as a new `- [ ] (not yet written)` entry right
   after yours. File any out-of-spec finding as a new `- [ ] (not yet written)` entry at the end of
   the stage it belongs to. Update the `**Count:**` line with greps you've just run, and show them.
   Today it reads `4 resolved (3 shipped specs, 1 folded) / 0 active / 4 pending`. Something with no
   stage goes in `guidance/ideas.yaml` (AGENTS §10).
3. Commit with `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-164): frame the edit page's crop mode`, with the trailer
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. A "Changes
   must be made through a pull request" notice can still mean the push landed, so check with
   `git ls-remote origin refs/heads/main`.
   ⛔ **If a permission check refuses the push, stop and report** (trap 21). Name the commits and the
   command for the maintainer to run, and don't claim the push happened.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-060 diff.
   ⚠ **PROJ-017 is the active project, but the status tooling shows only the highest-numbered
   one.** If `just status` or `just backlog` doesn't show STAGE-060, that's expected. Don't change
   any project's status; use a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag, no CHANGELOG edit. A probe you drive at frame
  lives in a scratch directory and is never committed.
- Don't move or close any other STAGE-060 entry (question 10 is a recommendation), and don't change
  STAGE-060's status. Don't touch the test bed.
- **Never drive the maintainer's browser.** A headless Chromium you start yourself is fine.
- Other sessions may be working at the same time. Never share a `CARGO_TARGET_DIR`. If
  `git rebase origin/main` conflicts in a file outside PROJ-017, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh doesn't
  word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS
  has no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~`, `/tmp` or the system temp folder.
  Delete scratch target dirs and browser profiles at the end, and say how much space that freed.
- **Budget ~50 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Find it by content, not
recency:**
`/usr/bin/grep -l "SPEC-164 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per content block. Lines that share
an id repeat the same `input_tokens`, `cache_creation_input_tokens` and `cache_read_input_tokens`,
while `output_tokens` grows. Take those three once per id, and the **MAX** `output_tokens`. Include
any subagent transcripts under the session's `subagents/` folder.

Price each component at the model that actually ran (read `.message.model`), never at a flat rate:
Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
Measure at your last turn, after the push, and append a `frame` entry to the spec's `cost.sessions`
with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate anchors, in a final small
commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-164 frame: GO | NO-GO | SPLIT`, then the commit SHA on
`main`. Then, short:
- the premise check (confirmed or wrong, with one line of evidence) and the parity summary line;
- the coordinate chain: driven over what, the worst error, and any function `recipe.js` should
  gain;
- the state machine, with the recipe each state emits and the edge cases;
- the render count per session;
- keyboard, touch and screen readers: the minimum you recommend;
- the size line, the watermark note and the CLI sentence;
- what is testable in Node, in headless Chromium and only in the test bed, and the headless check
  you recommend, with its negative control and CI cost;
- the mutants, and which group catches each;
- the wasm size, with numbers;
- fold or leave for the two entries;
- size and why, scope in and out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- the system temp folder's `ddphotos-*` counts before and after, and the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
