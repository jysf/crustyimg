# SPEC-152 — FRAME prompt: `trim` as a recipe step

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-152-frame -- --model opus
```

**Reserved: SPEC-152.** `main` has SPEC-141 to SPEC-151 (SPEC-149 and SPEC-150 are PROJ-011's
STAGE-063), and SPEC-140 is left for PROJ-016 (DEC-107). Before you allocate, confirm SPEC-152 is
unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-152' "$b" -- . ; done
```

The expected hits are this prompt and PROJ-017's brief. If a spec file or another reservation
uses SPEC-152, stop and report. Don't pick another number.

Your item is the STAGE-058 Spec Backlog entry "**`trim`.**" Read it in full. It's build-order
line 6 of 13. Other entries bear on it and are named below; touch no item except as this prompt
says.

## Why this spec exists
PROJ-017 makes geometry a recipe step, so the ddphotos page and the CLI share one saved recipe.
`crop` (SPEC-145, DEC-112), `rotate` and `flip` (SPEC-147, DEC-113) and `pad` (SPEC-151) have
shipped. `trim` is the last of the five ops line 8's `edit` flags need, and `pad`'s inverse.

**`trim` is the one op that reads pixel content to decide its geometry.** The other four take
their output box from keys alone. That makes three things new:
- **The output size depends on the pixels,** so the same recipe gives different sizes on different
  images, and a "whole image matches" case exists that no other op has.
- **A tolerance is a number compared against samples,** so its scale on 16-bit and float images is
  part of the contract (the backlog's ⚠).
- **It is the op most likely to drift into "smart" territory** (the brief's risk list and its "out
  of scope: smart crop, saliency"). The frame keeps it deterministic and says where the line is.

## What already exists (read it, don't assume it)
- **`pad`** (`src/operation/pad.rs`, SPEC-151 in `specs/done/`): the key table, `color` (required,
  opaque, `parse_color`), the per-variant handling of all ten `DynamicImage` types, the colour-type
  table, and how its per-image conditions (0 px border, caps) fail **at apply**, not through the
  refusal hook. `trim` should read as its mirror.
- **`crop`** (`src/operation/crop.rs`) and DEC-112: how a sub-rectangle is cut per variant, and
  crop's 0 px failure at apply (exit 1 on one input, exit 6 in a batch, the others written).
- **DEC-113**: a geometry step acts on the buffer it receives and writes no metadata. Say whether
  `trim` is covered as written.
- **DEC-095**: ops keep colour type and bit depth. A trim only removes pixels, so say whether any
  case can change either.
- **`parse_color`** (`src/text/mod.rs`): what a stated colour means on a grey, 16-bit and float
  image, and what `pad` decided (×257 at 16 bits, `c/255.0` in float).
- **The refusal hook** (`Operation::refusal`) and its doc comment, which SPEC-151 rewrote to say when
  an op uses the hook and when it fails at apply. STAGE-058's entry "The batch summary says
  'watermark skipped' for any refusing op" is still open; if any `trim` condition goes through the
  hook, that entry comes first or is folded in. Say which.
- **The registry and key tables** (`src/operation/registry.rs`), DEC-102 (refused at load, naming
  the step), `ParamKind` (is there an integer kind, or only `Number`?), and DEC-112 rule 6 (read
  numbers as `f64`).
- **SPEC-141's recipe-output guard** (`tests/recipe_corpus_output.rs`). Its three fixtures are
  generated, seeded images (`fixture()`, about :132), not photos, and the guard has an "every
  acting step acts" control. A `trim` step alone on those fixtures may find no border at all. Read
  how the fixtures are made and say what a `trim` corpus recipe has to look like to act on them
  (for example `pad` then `trim`).
- **SPEC-142's parity harness** and DEC-110: native and wasm must agree, exactly.
- **`scripts/wasm-ci-check.sh`**: read the current `FULL_COUNT` and `SLIM_COUNT` on `main` (53 and
  51 after SPEC-151, unless PROJ-011 has moved them since) and the self-test fixtures that name
  them. Any wasm row this spec adds moves both, in the same PR.
- **ImageMagick's `-trim` and `-fuzz`**, as a possible oracle. Drive it and say what its tolerance
  measures (per channel or a colour distance, and on what scale), which corner or colour it trims
  against, and what it does when the whole image matches. Where `trim`'s rule differs, say whether
  ImageMagick can still be an oracle for some cases, and which.

## Questions this frame must answer
1. **The colour it trims.** The backlog says "the corner pixel or a stated `color`".
   - Which corner? Top-left only, or all four must agree? What if they differ: refuse, use one, or
     trim each side against its own corners?
   - The key for a stated colour: `color`, matching `pad`? Is it optional here (corner by default)
     where `pad` requires it? Say why the two differ, if they do.
   - Alpha: does a stated colour have to be opaque, as `pad`'s does? Is a fully transparent pixel
     matched on its alpha alone, or on its hidden colour too? Drive it: an RGBA PNG whose border is
     alpha 0 with mixed hidden colours.
2. **The tolerance — the main question.**
   - Per channel (the backlog's word) or a distance? Integer or fraction? The key name.
   - **Its scale:** a tolerance of `8` on an 8-bit image, then on a 16-bit one (×257, so `8` means
     `0x0808`? or 8 raw 16-bit steps?), on a float image, and on grey. The backlog's ⚠ says to pin
     this with a test on a 16-bit source. Recommend one rule and make it a row.
   - Inclusive or exclusive at the edge (`|a − b| <= t`?), and does alpha take part?
   - Range, and what `0` means (exact match).
3. **Which sides.** Trim each side independently to the bounding box of non-matching pixels
   (ImageMagick's way), or only a border of equal width all round? The backlog says "a uniform
   border": say whether "uniform" means one colour or one width, and recommend.
4. **The whole image matches.** The backlog says refuse rather than give 0×0. Where: at apply, as
   `pad`'s 0 px border does (exit 1 on one input, exit 6 in a batch, the others written)? Or the
   hook (skipped in a batch, exit 0)? Use SPEC-145's M4b and SPEC-151's evidence, and say what a
   batch gets.
5. **Nothing to trim** (no border at all): a no-op that returns the input unchanged, as `pad`'s
   no-op does? Confirm and make it a test.
6. **Exactness, sized for verification.** The output is the input's sub-rectangle, copied exactly,
   at every colour type and depth (DEC-095). Name the shapes: borders of different widths on each
   side, a border on one side only, 1 px images, a one-pixel-wide survivor, odd sizes, a border pixel
   one step outside the tolerance.
7. **Inverse of `pad`.** `pad` with a fill, then `trim` against that fill, should give back the
   original box **when the original's own edge doesn't match the fill**. State the condition
   precisely, make it a test, and include the case where it fails on purpose (an image whose edge
   already matches the fill trims further than the pad added).
8. **Determinism and the "smart" line.** State in one sentence what `trim` looks at and what it
   never does, for its doc entry. Name anything you considered and kept out (for example
   "ignore noise", "trim to content", per-side tolerance) and where it would go.
9. **Refusals at load.** A bad colour, a tolerance out of range or of the wrong type, unknown keys
   (`fuzz`, `threshold`, `sides`?). All at load (DEC-102), naming the step, none naming a spec,
   stage or DEC id. Give each message's shape, and the trap 10 rows (a value inexact in `f32`, a
   near-miss spelling).
10. **Metadata.** Confirm DEC-113 covers `trim`. SPEC-151's frame drove the tagged-input case on
    `crop`; say whether that evidence carries over or needs driving again.
11. **Corpus and wasm.** Which corpus recipes prove byte-exactness across the three targets, given
    the generated fixtures, and the wasm rows and count change that follow.
12. **Fold-ins.** Three open STAGE-058 entries are small and could ride in this build. Read each and
    recommend fold or leave, with why:
    - "Pin `resize`'s cap message text" (filed at SPEC-151's ship; it asks to be folded into the
      next STAGE-058 spec's build);
    - "Pin `watermark`'s nine gravities, with a margin";
    - "Recipe-output guard: report the file-set and line-count failures together".
    Fold only what doesn't change the size.
13. **Docs.** `docs/recipes.md` lists "**Trim** **[planned — PROJ-017]**" (about :120) and its
    PROJ-017 line (about :282) says `trim` is planned. Find every other promise
    (`/usr/bin/grep -rn -i "\btrim\b" docs/ recipes/ skills/ README.md integrations/`) and say what
    changes. The verb set is frozen at 14 (STAGE-030), so it's a recipe-step entry, not a verb.
14. **Out:** `edit --trim` (line 8), `pad`'s transparent-fill entry, the ddphotos page
    (STAGE-060), and any content-aware trimming (post-1.0). Confirm each.
15. **Size.** Target **[S]**. If the tolerance rule needs a DEC, or the colour or alpha rules
    grow it, say whether that makes it an [M], and recommend a split if so.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. You recommend its content. Separate design, build and verify
sessions follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables**: wide tables have lost lines in transit.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §13 git, and §15, which now holds two verify
   rules that bind every cycle: "A bound or 'can't happen' is a claim until its worst input is
   driven" and "Classify every finding". §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order, the risk on
   `trim`, and "Explicitly out of scope".
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item, the entries named above, the exit criteria, and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
   (traps 10, 12, 13, 14 and 17). PROJ-017 has no HANDOFF of its own, so these bind here.
5. DEC-095, DEC-102, DEC-110, DEC-112, DEC-113.
6. SPEC-151 in `projects/PROJ-017-…/specs/done/`, in full, including its ship reflection: an
   "expected all green" mutation row was reasoned, and verify refuted it with an `f32`/`f64` split.
   A tolerance compared against samples is exactly where such a split hides.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Confirm there is no `trim` op today
  (`/usr/bin/grep` the registry). Write a scratch recipe with a `trim` step on a release build of
  `main` (fresh `CARGO_TARGET_DIR`, deleted at the end), and record the exact refusal it gets.
- **Drive the tolerance, alpha and oracle questions; don't reason them out.** Make inputs with a
  tool other than crustyimg (ImageMagick or Python stdlib): a bordered 8-bit RGB image, a 16-bit
  one with a border one step outside a tolerance, an RGBA one with an alpha-0 border of mixed hidden
  colours, and one that is all border. Run ImageMagick's `-trim` with and without `-fuzz` on each,
  and quote what it did.
- **If you recommend a bound** (for example "a 16-bit sample within `t × 257` matches"), say how
  you drove it and over what range, and name the input at the edge.
- **Size the verification, not just the code** (AGENTS §2): colour sources × tolerances × bit
  depths × colour types × alpha × border shapes × whole-match and no-op × native/wasm.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), including `scripts/wasm-ci-check.sh` if wasm rows are added,
  and any fold-in's files.
- **Questions only the maintainer can answer:** one line each, with your recommendation. The
  corner rule, the tolerance scale, the alpha rule and "uniform" are likely ones.

### Allocate the id
Run `just new-spec "<title>" STAGE-058 PROJ-017`. If it gives anything other than SPEC-152,
`git mv` the spec and its timeline to SPEC-152 and fix the id inside both. Then run
`just advance-cycle SPEC-152 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-095, DEC-102, DEC-110, DEC-112, DEC-113, SPEC-141, SPEC-142, SPEC-145, SPEC-151), and
  `cycle: frame`.
- `## Context` and `## Goal`: what a recipe author can't do today, and what they can when this
  ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the tolerance, alpha and oracle behaviour you drove, and the rules you recommend;
  - the recommended keys, and the alternatives you rejected;
  - where each per-image condition fails (load, hook, apply), and why;
  - the refusal table;
  - sizing and why;
  - scope in/out, including fold-ins;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-152-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-152 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-152** (framed <date>)`. If you recommend folding an entry, mark it
   `(folded into SPEC-152)` the way SPEC-151's folded entries were. File any out-of-spec finding as
   a new `- [ ] (not yet written)` entry at the end of the stage it belongs to, and update that
   stage's `**Count:**` line with a grep you've just run. Something with no stage goes in
   `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-152): frame trim as a recipe step`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-058 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. A probe you drive at frame lives in a
  scratch directory and is never committed.
- **PROJ-011 is working at the same time** (SPEC-149 and SPEC-150). Never share a
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
`/usr/bin/grep -l "SPEC-152 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

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
Plain lists only. First line: `SPEC-152 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- what ImageMagick's `-trim` and `-fuzz` did on each input you drove, quoted;
- the colour source (corner rule, stated `color`) and the alpha rule;
- the tolerance: spelling, range, scale at 8-bit, 16-bit, float and grey, and how you drove it;
- which sides, and what "uniform" means;
- whole-image-matches and nothing-to-trim: the outcome, where it fails, and what a batch gets;
- the `pad`-then-`trim` inverse and its condition;
- the refusal table;
- the corpus and wasm plan;
- the three fold-in candidates: folded or not, and why;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
