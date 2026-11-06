# SPEC-151 — FRAME prompt: `pad` as a recipe step

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-151-frame -- --model opus
```

**Reserved: SPEC-151.** `main` has SPEC-141 to SPEC-148. SPEC-149 and SPEC-150 are reserved by
PROJ-011's STAGE-063, and SPEC-140 is left for PROJ-016 (DEC-107). Before you allocate, confirm
SPEC-151 is unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-151' "$b" -- . ; done
```

The expected hits are this prompt and PROJ-017's brief. If a spec file or another reservation
uses SPEC-151, stop and report. Don't pick another number.

Your item is the STAGE-058 Spec Backlog entry "**`pad`.**" Read it in full. It's build-order
line 5 of 13. Three other entries bear on it and are named below; touch no item except as this
prompt says.

## Why this spec exists
PROJ-017 makes geometry a recipe step, so the ddphotos page and the CLI share one saved recipe.
`crop` shipped as SPEC-145 (DEC-112), and `rotate` and `flip` as SPEC-147 (DEC-113). `pad` is the
fourth lossless op: every input pixel lands unchanged at an offset, and every new pixel is one
stated colour, so its tests can be exact. `trim` (line 6) is its inverse, and the `edit` flags
spec (line 8) needs both.

**`pad` is the first geometry op that adds pixels rather than moving them,** so it is the first to
meet three questions the earlier ops didn't:
- **Colour type.** DEC-095 says ops keep colour type and bit depth. Padding an RGB image with
  `transparent`, or a grey image with red, can't be done without changing it.
- **The output format.** `transparent` into JPEG has nowhere to go. The backlog entry says this is
  decided here, not left to the encoder.
- **Refusal hook or apply failure.** "Pad to 800×600" on a 1200 px image is a per-image condition.
  SPEC-145's mutation M4b showed that a condition routed through the refusal hook is skipped in a
  batch, which writes the image un-padded and exits 0.

## What already exists (read it, don't assume it)
- **`crop`** (`src/operation/crop.rs`) and DEC-112: fractions vs pixels, how `aspect` is spelled
  and parsed, `Gravity` (`src/operation/mod.rs`, about :1174) and how crop places by it, and the
  rounding rule crop states for an odd remainder.
- **`rotate` and `flip`** (`src/operation/rotate.rs`, `flip.rs`) and DEC-113: a geometry step
  acts on the buffer it receives and writes no metadata. Say whether `pad` is covered by DEC-113
  as written.
- **`parse_color`** (`src/text/mod.rs:51`), the colour parser `watermark` uses. It returns
  `[u8; 4]`. Read what spellings it accepts, and say what a fill colour means on a 16-bit image
  and on a grey one.
- **`resize`'s modes** (registry and `src/operation/`). If any mode already letterboxes or pads,
  say how `pad` relates to it. If none does, say so.
- **How a recipe's output format is chosen**: the recipe's `format` key, `--format`, and the
  source format (PROJ-011's brief has a table). When does the pipeline know the output has no
  alpha channel: at load, at apply, or only at encode?
- **What the encoders do today with an alpha channel into JPEG.** Drive it: take an RGBA PNG with
  transparent pixels through a recipe to JPEG on a release build of `main`, and read the output's
  pixels where the alpha was 0. Quote what you saw. That is today's flatten behaviour, whether or
  not anyone chose it.
- **The refusal hook** (`Operation::refusal`, `src/operation/mod.rs`) and the two STAGE-058
  entries about it: "The batch summary says 'watermark skipped' for any refusing op" and "State
  when an op uses the refusal hook and when it fails at apply". Both say "before `pad` or `trim`
  ships".
- **The registry and key tables** (`src/operation/registry.rs`), DEC-102 (refused at load, naming
  the step), and how `crop`'s and `rotate`'s tables spell enumerations and numbers.
- **SPEC-141's recipe-output guard** (`tests/recipe_corpus_output.rs`): new recipes add golden
  lines, and no existing line may change.
- **SPEC-142's parity harness** and DEC-110: native and wasm must agree. A lossless op should
  agree exactly.
- **SPEC-146's wasm job** (`scripts/wasm-ci-check.sh`): it pins `FULL_COUNT` (51) and
  `SLIM_COUNT` (49), and its self-test fixtures name those numbers. Any wasm row this spec adds
  moves both the counts and the self-test, in the same PR.

## Questions this frame must answer
1. **What sizes `pad` takes.** The backlog says "to an exact size or to an aspect". For each, give
   the keys and their spelling:
   - **exact size:** pixels, fractions, or both? DEC-112 chose fractions for crop so a recipe
     survives a resize. Does that reason apply to "pad to 1080×1080 for Instagram", where the
     pixel count is the point? Say whether `pad` breaks DEC-112 or extends it, and whether a DEC
     is needed (DEC-114 would be next).
   - **aspect:** reuse crop's `aspect` spelling and parser.
   - **a border** ("add 5% on every side"): in or out? It's the most common pad after aspect.
     Recommend.
2. **Target smaller than the image.** Pad to 800×600 on a 1200×900 image, or pad to 1:1 an image
   that is already square. For each: no-op, refuse, or crop? Then decide **where** it fails: at
   load, in the refusal hook (skipped per image in a batch), or at apply (fails the run). Use the
   M4b evidence and say which outcome a batch gets. If any `pad` condition goes through the
   refusal hook, the two STAGE-058 hook entries must be done first or folded in; say which.
3. **Aspect against orientation.** STAGE-058's entry "`crop`'s aspect can follow each image's
   orientation" says `pad` to an aspect has the same question. Decide it here or name where it
   goes. A recommendation that keeps `pad` consistent with `crop` is the default.
4. **Gravity and the odd pixel.** Reuse `Gravity`. With `center` and an odd remainder, which side
   gets the extra pixel? Match crop's rule, state it in one sentence, and make it a test.
5. **The fill colour — the main question.**
   - Spelling: reuse `parse_color` (`color = "#ff0000"`, names?) plus `transparent`. Is the key
     `color`, `fill` or `background`? Check what `watermark` and the docs already say.
   - **Colour type:** RGB padded with `transparent` must gain alpha, and grey padded with a
     colour must become RGB. Each either widens the output (breaking DEC-095's "keep") or is
     refused. Recommend one rule per case. Say whether a fill that fits the image's type (grey on
     grey, opaque on RGB) always keeps it.
   - **Bit depth:** a `[u8; 4]` colour on a 16-bit image. State the widening (×257?) and make it a
     test.
   - **The default** when no colour is given: refuse, black, white, or transparent?
6. **`transparent` into a format with no alpha.** Refuse or flatten against a stated colour. It's
   decided here (the backlog says so). Name when the pipeline can know (question in "What already
   exists"), and therefore where the refusal or the flatten lives. A refusal that can only happen
   at encode is still a refusal: give its message and exit code. Compare with today's flatten
   behaviour, which you drove.
7. **Metadata.** Confirm DEC-113 covers `pad` (acts on the buffer it receives, writes no
   metadata), or say what it misses. Drive it with one tagged input as SPEC-147's frame did:
   `exiftool -Orientation#=6`, run `crop` (the nearest shipped op) and read the output with
   `exiftool`.
8. **Refusals at load.** Missing keys, both size and aspect given, a bad colour, a zero or
   negative size, a non-positive aspect. All at load (DEC-102), naming the step. Give each
   message's shape. None names a spec, stage or DEC id (SPEC-147's ruling 3).
9. **Exactness, sized for verification.** Every input pixel lands unchanged at its offset, and
   every new pixel equals the fill: for 8-bit and 16-bit, grey and RGB, with and without alpha
   (DEC-095), at every gravity, and with odd remainders. Name the fixtures. `pad` then `crop` back
   to the original box should be the identity; make it a test.
10. **Native and wasm.** Which corpus recipes and wasm rows prove byte-exactness, and the
    `FULL_COUNT`/`SLIM_COUNT` change that follows.
11. **The fold-in entry.** STAGE-058's "SPEC-147's verify cleanups that need a PR" says to fold
    into the next STAGE-058 spec's build: a stricter `orientation_of` in `tests/rotate_flip.rs`,
    and the corpus README's stale literals. This is that spec. Confirm it fits without changing
    the size, and put it in scope. Its README fix touches the same literal table this spec's
    corpus recipes will move again, so say how the build keeps the table right afterwards.
12. **Docs.** Find every place a pad verb or step is promised
    (`/usr/bin/grep -rn -i "\bpad\b" docs/ recipes/ skills/ README.md`); `docs/recipes.md:98`
    lists "Trim / pad" as planned. Say what changes. The verb set is frozen at 14 (STAGE-030), so
    it's a recipe-step entry, not a verb.
13. **Out:** `edit --pad` (line 8), `trim` (line 6), and the ddphotos page (STAGE-060). Confirm
    each.
14. **Size.** Target **[S]**. If the colour-type rule needs a DEC, or the format question reaches
    into the encode path, say whether that makes it an [M], and recommend a split if so (for
    example `pad` with opaque fills first, `transparent` second).

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. You recommend its content. Separate design, build and verify
sessions follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own. Keep it plain text with short
lists; a wide box-drawn table can garble when copied out of the terminal.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 backlog, §13 git, §15 cycle rules).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order and its ⚠
   notes.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item, the three entries named above, the exit criteria, and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" (rule 7), the
   rules, and "Traps this wave paid for" (trap 17). PROJ-017 has no HANDOFF of its own, so these
   rules bind here.
5. DEC-095, DEC-102, DEC-110, DEC-112, DEC-113, and the registry DEC behind SPEC-131.
6. The SPEC-145 and SPEC-147 specs in `projects/PROJ-017-…/specs/done/`, including both ship
   reflections. SPEC-147's says cheap mutation rows should be driven at design, not reasoned, and
   that "apply never fails" should be stated as a claim over named shapes.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Confirm there is no `pad` op today
  (`/usr/bin/grep` the registry). Write a scratch recipe with a `pad` step on a release build of
  `main` (fresh `CARGO_TARGET_DIR`, deleted at the end), and record the exact refusal it gets.
- **Drive the format and colour questions; don't reason them out.** The alpha-into-JPEG run, the
  tagged-input run, and what `parse_color` returns for each spelling you recommend. Make inputs
  with a tool other than crustyimg (ImageMagick or Python stdlib) and read outputs with one too.
  Quote what you saw. A claim about what an encoder or viewer will do is a claim; say what you
  drove.
- **Size the verification, not just the code** (AGENTS §2): size modes × gravities × odd/even
  remainders × colour types × bit depths × alpha × fills × output formats × native/wasm.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), including `scripts/wasm-ci-check.sh` if wasm rows are added,
  and the fold-in entry's two files.
- **Questions only the maintainer can answer:** one line each, with your recommendation. The
  colour-type rule, the JPEG rule, the default fill and pixel-vs-fraction sizes are likely ones.

### Allocate the id
`just new-spec` allocates by scanning the working tree, and the tree has no SPEC-149 or SPEC-150
file yet, so it will likely give SPEC-149. Run `just new-spec "<title>" STAGE-058 PROJ-017`. If it
gives anything other than SPEC-151, `git mv` the spec and its timeline to SPEC-151 and fix the id
inside both. Then run `just advance-cycle SPEC-151 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-095, DEC-102, DEC-110, DEC-112, DEC-113, SPEC-131, SPEC-141, SPEC-142, SPEC-145, SPEC-147),
  and `cycle: frame`.
- `## Context` and `## Goal`: what a recipe author can't do today, and what they can when this
  ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the format, colour and metadata behaviour you drove, and the rules you recommend;
  - the recommended keys and size modes, and the alternatives you rejected;
  - where each per-image condition fails (load, hook, apply, encode), and why;
  - the refusal table;
  - sizing and why;
  - scope in/out, including the fold-in entry;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-151-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-151 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-151** (framed <date>)`. If you fold the fold-in entry or a hook entry into this
   spec, mark that entry `(folded into SPEC-151)` the way SPEC-145's docs entry was. File any
   out-of-spec finding as a new `- [ ] (not yet written)` entry at the end of the stage it belongs
   to, and update that stage's `**Count:**` line with a grep you've just run. Something with no
   stage goes in `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-151): frame pad as a recipe step`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-058 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. A probe you drive at frame lives in a
  scratch directory and is never committed.
- **PROJ-011 is working at the same time** (SPEC-148 and STAGE-063). Never share a
  `CARGO_TARGET_DIR` with it. If `git rebase origin/main` conflicts in a file outside PROJ-017,
  stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-151 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and
cache_read is ×0.10. Measure at your last turn, after the push, and append a `frame` entry to the
spec's `cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate
anchors in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-151 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- the alpha-into-JPEG and tagged-input behaviour you drove, quoted;
- the size modes and keys, each with the alternative you rejected, and whether a DEC is needed;
- target-smaller-than-image: the outcome, where it fails, and what a batch gets;
- the fill colour: spelling, default, the colour-type rule, the 16-bit widening;
- the `transparent`-into-JPEG rule and where it lives;
- the refusal table;
- the fold-in entry and any hook entry: folded or not;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them.

Then your measured cost.
