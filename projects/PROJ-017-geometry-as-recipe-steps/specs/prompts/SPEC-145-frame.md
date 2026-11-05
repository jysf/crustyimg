# SPEC-145 — FRAME prompt: `crop` as a recipe step, in units that survive a resize

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-145-frame -- --model opus
```

**Reserved: SPEC-145.** `main` has SPEC-141 to SPEC-144, and SPEC-140 is left for PROJ-016
(DEC-107). Before you allocate, confirm SPEC-145 is unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-145' "$b" -- . ; done
```

The expected hits are this prompt, PROJ-017's brief, and one line in PROJ-011's
`SPEC-144-frame.md` (about :137) that only names it as a number `just new-spec` might hand out.
If a spec file or another reservation uses SPEC-145, stop and report. Don't pick another number.

Your item is the **second** entry in STAGE-058's Spec Backlog: "**`crop` as a recipe step, with
size-independent units.**" Read it in full. Also fold in the later entry "**`docs/recipes.md`
still advertises a `crustyimg crop` verb**": that entry says to fold it into this spec's docs.
Touch no other item.

## Why this spec exists
PROJ-017 makes geometry a recipe step, so the ddphotos page and the CLI share one saved
recipe. `crop` is build-order line 2 of 13 and the critical path: `pad`, `trim`, crop on the
page, and the `edit` flags all wait on it. It's sized **[M]**, and it carries the project's
riskiest design call.

**The trap it must not fall into:** the page edits a downscaled preview, and `apply` replays the
recipe on the full-size original. A crop written in preview pixels lands in the wrong place on
the original. Both sides still exit 0, so nothing reports it. The spec writes the DEC that says
which units a recipe may carry and how they round to pixels.

## What already exists (read it, don't assume it)
- **`Gravity`** (`src/operation/mod.rs`, about :1147–1200): the anchor type. Reuse it; don't add
  a second one.
- **`resize`'s key table** (`src/operation/registry.rs`, about :630): `mode`, `width`, `height`,
  `percent`. `percent` is the nearest existing size-independent unit. Read how it's parsed,
  validated at load (DEC-102) and rounded, and whether the crop unit can share that code or that
  spelling.
- **Resize modes that already crop.** `resize`'s cover/fill mode crops to fill a box. Find the
  crop it does internally, and say whether `crop` reuses it. Note that a CLI audit found
  `--fill`/`--cover` documented backwards; check what the code does, not the docs.
- **The ddphotos page** (`integrations/ddphotos/edit/`): how the page sizes its preview, and
  what it would have to emit for a crop. The page's crop UI is STAGE-060's, not yours. Your DEC
  must make that UI possible with no pixel maths that depends on the preview size.
- **SPEC-141's recipe-output guard** (`tests/recipe_corpus_output.rs`, README in
  `tests/fixtures/recipe_corpus/`). A new op adds recipes to the corpus and new golden lines;
  no **existing** line may change. Say how the build proves that.
- **SPEC-142's parity harness** (`tests/ddphotos_parity.mjs`) and DEC-110: native and wasm agree
  on dimensions exactly. A crop must hold that across a fraction-to-pixel rounding rule that runs
  in both builds.

## Questions this frame must answer
1. **The unit syntax.** Pixels, fractions, or both, and how a recipe writes each: for example
   `x = 0.1` as a float, `x = "10%"` as a string, or a separate key family. Weigh:
   - TOML typing: is `1` a pixel and `1.0` a fraction? That reads ambiguously.
   - DEC-102: every bad value is refused at load, naming the key and the step.
   - The page: it can emit fractions with no knowledge of the original's size.
   - The CLI user, who wants pixels.

   Recommend one, with the alternative you rejected and why.
2. **The rounding rule.** How a fraction becomes a pixel rectangle on the real image: round,
   floor or ceil, and on which edges. The rectangle must stay inside the image, and a fraction
   crop of the same image at two sizes must agree "within one pixel of rounding" (STAGE-058's
   exit criteria). Write the rule so it's testable, and name the test.
3. **The two crop forms.** A rectangle (`x`, `y`, `width`, `height`) and an aspect
   (`aspect = "16:9"` plus `gravity`). Are they one op with exclusive key groups or two modes?
   How does `resize`'s `mode` key set the precedent? What does a recipe with keys from both
   forms get?
4. **Refusals, and when they happen.** List each bad crop: zero area, negative, out of bounds,
   a fraction over 1, a malformed aspect, mixed forms. For each, say whether it's refused at load
   (exit 1, no image needed) or at apply (needs the image size), and the message shape. Pixel
   out-of-bounds can only be caught at apply. Say whether it's refused or clamped, and why.
5. **Colour type and bit depth** are preserved (DEC-095). Crop never resamples, so this should be
   exact. Name the fixtures that prove it: 8-bit RGB, RGBA and 16-bit.
6. **EXIF orientation.** If a recipe crops an image that carries an orientation tag and wasn't
   auto-oriented, which way is "x"? The rotate spec owns the full question (STAGE-058's next
   entry). Say whether crop must decide it now, or can defer it with a stated rule.
7. **Docs.** `docs/recipes.md:60-65` lists "Crop to an arbitrary rect / gravity / aspect
   [planned — PROJ-006]" as a `crustyimg crop` verb. The verb set is frozen at 14 (STAGE-030),
   so that line becomes a recipe-step entry. Also find where smart crop is listed the same way,
   and every other place a `crop` verb is promised (`/usr/bin/grep -rn "crustyimg crop"`).
8. **Is `edit --crop` in or out?** The backlog puts the `edit` flags for all five ops in a later
   spec, with the ordering DEC. Confirm that `crop` here is recipe-only, or argue otherwise.
9. **Size.** Target **[M]**. If the DEC plus the op plus the refusals plus the docs is more than
   an M, recommend the split. A likely one is to land the DEC and the rectangle form first, then
   the aspect form.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write the DEC. You recommend its content. Separate design, build and verify
sessions follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 backlog, §13 git, §15 cycle rules).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the success signals, the
   build order and its ⚠ notes on units and ordering.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item, the exit criteria, and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (now 16). PROJ-017 has no HANDOFF of its own, so these rules bind
   here.
5. DEC-102 (step params validated at load), DEC-095 (colour type and bit depth), DEC-110
   (native/wasm agreement), and the registry DEC behind SPEC-131.
6. The SPEC-141 and SPEC-142 specs in `projects/PROJ-017-…/specs/done/`.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Confirm there is no crop op or
  key today (`/usr/bin/grep` the registry and `src/operation/`). Find resize's internal crop.
  Write a scratch recipe with a `crop` step on a release build of `main` (fresh `CARGO_TARGET_DIR`,
  deleted at the end) and record the exact refusal it gets today.
- **Show the trap with numbers.** Take a real preview width the page uses and a real original
  width. Show where a preview-pixel crop lands on the original, and where the fraction form lands.
- **Size the verification, not just the code** (AGENTS §2): the forms × units × refusals ×
  colour types × native/wasm matrix.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation. The unit
  spelling is likely one.

### Allocate the id
`just new-spec` allocates by scanning the working tree. Run
`just new-spec "<title>" STAGE-058 PROJ-017`. If it gives anything other than SPEC-145, `git mv`
the spec and its timeline to SPEC-145 and fix the id inside both. Then run
`just advance-cycle SPEC-145 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-095, DEC-102, DEC-110, SPEC-131, SPEC-141, SPEC-142), and `cycle: frame`.
- `## Context` and `## Goal`: what a recipe author can't do today, and what the page and the CLI
  can do when this ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the preview-versus-original numbers;
  - the recommended unit syntax and rounding rule (the DEC's content, for design to write);
  - the refusal table, load versus apply;
  - sizing and why;
  - scope in/out;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-145-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-145 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-145** (framed <date>)`. If you fold the `docs/recipes.md` entry in, mark that
   one `- [ ] (folded into SPEC-145)`. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of the stage it belongs to, and update that stage's
   `**Count:**` line with a grep you've just run. Something with no stage goes in
   `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-145): frame crop as a recipe step`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-058 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. A probe you drive at frame lives in a
  scratch directory and is never committed.
- **Another session may be working PROJ-011 at the same time.** Never share a `CARGO_TARGET_DIR`
  with it. If `git rebase origin/main` conflicts in a file outside PROJ-017, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-145 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and
cache_read is ×0.10. Before your final commit, append a `frame` entry to the spec's
`cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate anchors.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-145 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- the preview-versus-original numbers;
- the recommended unit syntax and rounding rule, and the alternative you rejected;
- the refusal table, load versus apply;
- the two forms: one op or two modes;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them.

Then your measured cost.
