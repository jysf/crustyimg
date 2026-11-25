# SPEC-163 — FRAME prompt: an aspect that follows each image's orientation

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-163-frame -- --model opus
```

**Reserved: SPEC-163.** SPEC-162 shipped on 2026-11-25. Before you allocate, confirm SPEC-163 is
unused as a spec id:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-163' "$b" -- . ; done
```

The only expected hit is this prompt. If a spec file or another reservation uses SPEC-163, stop and
report. Don't pick another number. `just new-spec` may offer a different id; take SPEC-163 anyway
(see "Allocate the id").

Your item is the STAGE-060 Spec Backlog entry "**`crop`'s aspect can follow each image's
orientation.**" Read it in full, including its three dated paragraphs (moved from STAGE-058, ahead
of SPEC-161, builds after the STAGE-063 P1s, which have now shipped). Touch no other item except as
this prompt says.

## Why this spec exists
`aspect = "7:5"` always means 7 wide by 5 tall (SPEC-145). A recipe for "5×7 prints of this folder"
wants 7:5 on landscape photos and 5:7 on portrait ones, which today takes two recipes and a hand
sort. The maintainer ruled on 2026-11-22 (SPEC-161's "Maintainer rulings", item 4) that the edit
page's crop presets emit `aspect`, not a box, so a recipe copied from the page reads `aspect = "3:2"`
and works on any photo. That only holds across a folder of mixed photos if the aspect can follow
each photo's orientation. **So the page is this key's first user, and this spec builds before
SPEC-161's design.** SPEC-161 touches no Rust; this spec is the Rust half.

## Settled; don't re-open
- **Leaving the key off keeps SPEC-145's meaning.** No existing recipe, golden or `edit` invocation
  changes its output.
- **Named presets stay out of the recipe format** (`"5x7"`, `"square"`): a ratio is unambiguous, and
  presets belong to the page's buttons and a docs table.
- **The key covers both `crop` and `pad`** with one rule. SPEC-151's frame named this entry for
  `pad`, whose `aspect` also means W wide by H tall.
- **A geometry step acts on the buffer it receives and writes no metadata** (DEC-113). So
  "orientation" means the shape of that buffer, after any `auto-orient` step before it, not the
  EXIF tag.
- **Units** (DEC-112): crop and pad carry fractions or ratios, and each edge is rounded as SPEC-145
  and SPEC-151 define. This spec doesn't change any rounding.
- **Parity** (DEC-110): native `apply` and wasm `transform` run the same Rust, so they must agree
  exactly on every row this spec adds.

The frame's job is everything those leave open.

## What already exists (read it, don't assume it)
- **`crop`**: `src/operation/crop.rs`, with SPEC-145 in `specs/done/`: the `box` and `aspect`
  forms, `gravity`, `aspect_size`'s rounding, the minimum size and the refusals.
- **`pad`**: its op module and SPEC-151 (and SPEC-159/160 for the fill rules) in `specs/done/`: the
  `aspect` and `border` forms, `gravity`, and its refusals.
- **Recipe load and validation**: the registry's `param_specs` (SPEC-131) and DEC-102's "name the
  key and the step" rule for load errors.
- **`edit`'s flags**: `--crop W:H` with `--crop-gravity`, `--pad W:H` with `--pad-gravity`, and how
  `--save-recipe` writes each as a step (SPEC-153, SPEC-160).
- **The page's model**: `integrations/ddphotos/edit/recipe.js`, and SPEC-161's `## Frame` and
  rulings (`specs/SPEC-161-…`). The page will emit this key; SPEC-161's design ports the rule into
  `recipe.js`. You build nothing on the page.
- **The parity harness**: `tests/ddphotos_parity.mjs` (`just ddphotos-parity`).

## Questions this frame must answer
1. **Drive today's behaviour.** On a release build of `main` (fresh `CARGO_TARGET_DIR` under your
   scratchpad), run `crop` with `aspect = "3:2"` on a landscape, a portrait and a square image, and
   the same for `pad`. Quote the output sizes. Then run it on an EXIF-rotated JPEG (Orientation 6,
   tagged with `exiftool -Orientation=6 -n`) with and without an `auto-orient` step before it, and
   say which shape the step sees in each case.
2. **The key's name and values.** The entry suggests `orientation = "match"`. Weigh at least: a
   boolean (`follow_orientation = true`), an enum with one value now (`"match"`) and room later, and
   an enum that also allows forcing (`"landscape"`, `"portrait"`). Recommend one, and say what the
   default spells when written out (does `--save-recipe` ever write the default?).
3. **What a square image gets.** With the key on, a 3:2 aspect on a square buffer: 3:2, 2:3, or
   refuse? And a square aspect (`1:1`) with the key on: say it's a no-op, or refuse the combination.
   Recommend, with the reason a user would expect.
4. **Near-square images.** A 1001×1000 buffer is landscape by one pixel. Say whether that's the
   rule (strictly wider is landscape) and whether it surprises anyone; drive a case where the
   choice flips the output.
5. **Refusals.** The key with `crop`'s `box` form; with `pad`'s `border` form; an unknown value; the
   wrong type. For each, recommend refuse at load (exit 1, naming the key and the step per DEC-102)
   or ignore. Say what the existing load errors look like, quoted.
6. **`edit`'s flags.** Does `edit` need a flag for the key (for example `--crop-orientation match`,
   or a spelling on `--crop` itself), and does `--save-recipe` write it? Or is the key recipe-only
   for now? Recommend, and say whether `edit --help` changes.
7. **wasm and the page.** Confirm the key reaches wasm `transform` with no wasm-side change (same
   recipe loader). Say exactly what SPEC-161's design must port into `recipe.js` to predict the
   output size with the key on, and whether `aspect_size`'s rounding is shared by both branches.
8. **The parity rows.** List the rows this spec adds to `just ddphotos-parity`: landscape, portrait,
   square, near-square and EXIF-rotated with `auto-orient`, for `crop` and `pad`, with the key on
   and off. Name a negative control that goes red if the key is ignored (the "on" row must differ
   from the "off" row on a portrait input, or the row proves nothing). Say which existing goldens
   must stay byte-identical, and how build proves that.
9. **Docs.** `docs/recipes.md`'s `crop` and `pad` sections, the recipe README if it lists keys,
   `edit --help` if question 6 adds a flag, and a CHANGELOG line (written at build, not here). List
   each behaviour sentence the build must drive on every exit path it names (AGENTS §15).
10. **The three handed-in items** (PROJ-017 brief, "Handed in from PROJ-011"): `format = "bogus"`,
    `quality = 101`, and the wasm `validate()` export. They touch recipe load. Does this spec touch
    any file they'd touch? Say yes or no with the files, so the orchestrator can schedule them. Don't
    fold or frame them.
11. **A DEC?** Recommend whether the orientation rule needs its own DEC or an amendment to DEC-112
    (or the decision SPEC-145/151 recorded). Recommend its content; don't write it.
12. **Size.** The backlog says [S]. Two ops, one loader key, refusals, maybe an `edit` flag, parity
    rows and docs: say whether it's still [S], and split only if it isn't.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped spec
stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and you do
**not** write any DEC. You recommend its content. Separate design, build and verify sessions follow.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather than
a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §13 git, §15 (the coverage map in "During
   design", "a bound or 'can't happen' is a claim until its worst input is driven", the finding
   classes) and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order, "Handed in
   from PROJ-011", and "Explicitly out of scope".
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`:
   your item, every open entry, the Success Criteria and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   PROJ-011 is on hold, but its traps still apply.
5. DEC-102, DEC-110, DEC-112, DEC-113.
6. SPEC-145, SPEC-151 and SPEC-153 in `projects/PROJ-017-…/specs/done/`, with their ship
   reflections, and the STAGE-058 Stage-Level Reflection. Apply its lessons in your stub: a mutation
   row quotes its exact mutant edit, a count carries its derivation, and build drives each doc
   sentence on every exit path.
7. SPEC-161's spec, `## Frame` and "Maintainer rulings", for what the page needs from this key.
8. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code** (question 1), and drive the square and near-square cases
  (questions 3 and 4) before recommending.
- **Size the verification, not just the code** (AGENTS §2): two ops × key on/off × landscape,
  portrait, square, near-square × EXIF-rotated with and without `auto-orient` × the refusals.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation. The key's
  name (question 2) and the square rule (question 3) are probably two.

### Allocate the id
Run `just new-spec "<title>" STAGE-060 PROJ-017`. If it gives anything other than SPEC-163, `git mv`
the spec and its timeline to SPEC-163 and fix the id inside both. Then run
`just advance-cycle SPEC-163 frame`. If you recommend a DEC, it takes the next free id on `main`;
check with `ls decisions/`. Recommend its content; don't write it.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references` (DEC-102,
  DEC-112, DEC-113, SPEC-145, SPEC-151, SPEC-161), and `cycle: frame`.
- `## Context` and `## Goal`: what a recipe can't say today, and what it can when this ships.
- A `## Frame` section directly after `## Goal`, holding every answer above, the verdict, sizing,
  scope in/out, the rough build file list and the open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, then a `[ ]` **rulings** line if you have
  maintainer questions, and `[ ]` lines for design (`prompts/SPEC-163-design.md`), build, verify and
  ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-163 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-060's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-163** (framed <date>)`. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of the stage it belongs to, or in
   `guidance/ideas.yaml` if no stage owns it (AGENTS §10). Update the `**Count:**` line with greps
   you've just run, and show them.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-163): frame an aspect that follows each image's orientation`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. A "Changes
   must be made through a pull request" notice can still mean the push landed: check with
   `git ls-remote origin refs/heads/main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-060 diff. PROJ-017 is the active
   project since 2026-11-25, so `just status` shows this stage.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag, no CHANGELOG edit. A probe you drive at frame
  lives in your scratchpad and is never committed.
- Don't move or close any other STAGE-060 entry, and don't change any stage's or project's status.
- **Never drive the maintainer's browser.** You shouldn't need a browser at all.
- Other sessions may be working at the same time. Never share a `CARGO_TARGET_DIR`. If
  `git rebase origin/main` conflicts in a file outside PROJ-017, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS has
  no `timeout(1)`. `rtk` can truncate `grep`, `ls` and `git log` output, so use `/usr/bin/grep`,
  `/bin/ls` and `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~`, `/tmp` or the system temp dir.
  Delete your target folders when you finish, and say how much space that freed.
- **Budget ~35 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content, not
recency:** `/usr/bin/grep -l "SPEC-163 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`. If the
prompt was given as a file path, the title may not be in your first message; then match the
prompt's path instead.

Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
`output_tokens`. Price at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read ×0.10. Measure at your last turn, after the
push, and append a `frame` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-163 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- today's behaviour, driven and quoted (question 1);
- the key's name and values, and what `--save-recipe` writes;
- the square and near-square rules, with the driven cases;
- the refusals, with the quoted load errors;
- `edit`'s flags: added or not;
- what SPEC-161's design must port into `recipe.js`;
- the parity rows, the negative control, and the goldens that must not move;
- the docs sentences build must drive;
- the handed-in items' file overlap;
- DEC: new, amendment or none, with recommended content;
- size and why, scope in/out;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
