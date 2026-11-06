# SPEC-147 — FRAME prompt: `rotate` by quarter turns and `flip`, as recipe steps

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-147-frame -- --model opus
```

**Reserved: SPEC-147.** `main` has SPEC-141 to SPEC-146 (SPEC-146 is PROJ-011's wasm CI job),
and SPEC-140 is left for PROJ-016 (DEC-107). Before you allocate, confirm SPEC-147 is unused
everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-147' "$b" -- . ; done
```

The expected hits are this prompt and PROJ-017's brief. If a spec file or another reservation
uses SPEC-147, stop and report. Don't pick another number.

Your item is the STAGE-058 Spec Backlog entry "**`rotate` (quarter turns) and `flip`.**" Read it
in full. It's build-order line 4 of 13. Touch no other item.

## Why this spec exists
PROJ-017 makes geometry a recipe step, so the ddphotos page and the CLI share one saved recipe.
`crop` shipped as SPEC-145 (DEC-112). `rotate` and `flip` are the next two lossless ops: every
output pixel is an input pixel, so their tests can be exact. STAGE-059's any-angle rotate extends
this spec's `rotate` op and key table, and the `edit` flags spec (line 8) needs it too.

**The trap this spec must not fall into:** an image can carry an EXIF orientation tag. A recipe
that rotates the stored pixels and then writes the tag back unchanged makes a viewer apply the
tag's rotation on top, so the output displays wrong. Both sides exit 0, so nothing reports it.
SPEC-145 deferred this question to this spec, which owns it in full: what an orientation tag
means after a geometry step, and what `PixelXDimension`/`PixelYDimension` say. Crop shipped
under a stated rule, and that rule must still hold once this spec answers.

## What already exists (read it, don't assume it)
- **`AutoOrient`** (`src/operation/mod.rs`, about :1112) and its tests (about :2485–2565). It
  already rotates and flips by the orientation tag. Find the rotate/flip calls it makes and say
  whether `rotate` and `flip` share them. Read what it does to the tag afterwards (reset to 1?
  strip it?).
- **SPEC-145's orientation rule** (`specs/done/SPEC-145-…resize.md`, "EXIF orientation: defer,
  with a stated rule", about :472): `x` and `y` are in the pixel buffer the step receives, and
  `apply` adds no orientation step. The page always auto-orients first (`recipe.js:97`).
- **How the pipeline writes metadata.** Find where a pixel op's output gets its EXIF back, and
  what happens to the orientation tag and the pixel dimensions after `crop` today. Drive it:
  crop an image tagged orientation 6 on a release build of `main`, and read the output's tags
  with `exiftool` (a tool other than crustyimg).
- **SPEC-110** (shipped): `edit` bakes orientation first. A hand-written recipe may not.
- **The registry and key tables** (`src/operation/registry.rs`), DEC-102 (refused at load,
  naming the step), and how `crop`'s table (SPEC-145) and `resize`'s `mode` key spell an
  enumerated value and a number.
- **SPEC-141's recipe-output guard** (`tests/recipe_corpus_output.rs`): new recipes add golden
  lines, and no existing line may change.
- **SPEC-142's parity harness** and DEC-110: native and wasm must agree. Lossless ops should
  agree exactly.
- **SPEC-146's wasm job** (`scripts/wasm-ci-check.sh`): it pins the exact number of wasm tests
  in `FULL_COUNT` and `SLIM_COUNT`, and its self-test fixtures assume those numbers. SPEC-145's
  crop added two wasm tests and went red at merge until both were moved. Any wasm row this spec
  adds must update both the counts and the self-test, in the same PR.

## Questions this frame must answer
1. **The `angle` key.** Integer or float in TOML, and whether `90.0` is accepted. STAGE-059
   lifts the multiples-of-90 restriction on the same key, so pick a type that won't need a
   breaking change then. Which values are accepted: 0, 360, negative (`-90`), more than 360?
   Recommend one spelling, with the alternative you rejected.
2. **Direction.** Is a positive angle clockwise or anticlockwise? Name the precedents that
   disagree (`image::imageops::rotate90` is clockwise; maths convention is anticlockwise; CSS
   `rotate()` is clockwise), and recommend one. The doc entry states it in one sentence.
3. **`flip`.** `axis = "horizontal" | "vertical"`: does "horizontal" mirror left-to-right, or
   flip across the horizontal axis? Both readings are common. Pick one, check it against
   `image::imageops::fliph`'s meaning, and say whether to rename the values to make it
   unambiguous.
4. **One spec or two, one op or two.** `rotate` and `flip` as two ops in one [S] spec, or split?
   Is `rotate = 180` the same pixels as flipping both ways? (It should be; make it a test.)
5. **EXIF orientation and metadata — the main question.** For an input tagged orientation N that
   was not auto-oriented:
   - which buffer `rotate` and `flip` act on (the stored pixels, following SPEC-145's rule);
   - what the output's orientation tag must be so the result displays as intended, and whether
     the op resets it, strips it, or composes it;
   - what `PixelXDimension`/`PixelYDimension` say after a 90° turn, and after `crop`;
   - whether the C2PA behaviour on the pixel lane is unchanged.

   Drive today's behaviour first (see "What already exists"), then recommend the rule. Say
   whether it needs a DEC (DEC-113 would be next) and whether it changes anything `crop` shipped.
6. **Refusals.** A non-multiple of 90, a bad type, an unknown axis, missing keys. All should be
   refused at load (DEC-102). Confirm none needs the image, so none uses the refusal hook. (The
   STAGE-058 entry "State when an op uses the refusal hook" explains why a condition that means
   wrong output must not go through it.) Give each refusal's message shape. The non-multiple
   refusal names the any-angle stage.
7. **Exactness, sized for verification.** Rotate four times and flip twice, and each returns the
   input's pixels exactly: for 8-bit and 16-bit, grey and RGB, with and without alpha (DEC-095).
   Also non-square images, where a 90° turn swaps the dimensions. Name the fixtures.
8. **Native and wasm.** The ops should be byte-exact across native and wasm. Say which corpus
   recipes and wasm rows prove it, and the `FULL_COUNT`/`SLIM_COUNT` change that follows.
9. **Docs.** Find every place a rotate or flip verb or step is promised
   (`/usr/bin/grep -rn "rotate\|flip" docs/ recipes/ skills/ README.md`) and say what changes.
   The verb set is frozen at 14 (STAGE-030), so a planned `crustyimg rotate` verb line becomes a
   recipe-step entry.
10. **`edit --rotate` / `--flip`:** confirm they're out (line 8 owns them, with the ordering DEC).
11. **Size.** Target **[S]**. If the orientation answer needs a DEC and touches the metadata
    path, say whether that makes it an [M], and recommend a split if so.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. You recommend its content. Separate design, build and verify
sessions follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 backlog, §13 git, §15 cycle rules).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order and its ⚠
   notes.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item, the exit criteria, and the design notes ("Metadata after geometry").
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for". PROJ-017 has no HANDOFF of its own, so these rules bind here.
5. DEC-095, DEC-102, DEC-110, DEC-112, and the registry DEC behind SPEC-131.
6. The SPEC-141, SPEC-142 and SPEC-145 specs in `projects/PROJ-017-…/specs/done/`, including
   SPEC-145's ship reflection.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Confirm there is no `rotate` or
  `flip` op today (`/usr/bin/grep` the registry). Write a scratch recipe with a `rotate` step on
  a release build of `main` (fresh `CARGO_TARGET_DIR`, deleted at the end), and record the exact
  refusal it gets today.
- **Drive the orientation question; don't reason it out.** Make tagged inputs with a tool other
  than crustyimg (`exiftool -Orientation#=6`, for example). Run `crop` and `auto-orient` on them
  today, and read every output's orientation and pixel-dimension tags with `exiftool`. Quote
  what you saw. A claim about what a viewer will do is a claim; say what you drove.
- **Size the verification, not just the code** (AGENTS §2): angles × axes × colour types × bit
  depths × alpha × square/non-square × orientation tags × native/wasm.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), including `scripts/wasm-ci-check.sh` if wasm rows are added.
- **Questions only the maintainer can answer:** one line each, with your recommendation.
  Direction and the `flip` axis naming are likely ones.

### Allocate the id
`just new-spec` allocates by scanning the working tree. Run
`just new-spec "<title>" STAGE-058 PROJ-017`. If it gives anything other than SPEC-147, `git mv`
the spec and its timeline to SPEC-147 and fix the id inside both. Then run
`just advance-cycle SPEC-147 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-095, DEC-102, DEC-110, DEC-112, SPEC-131, SPEC-141, SPEC-142, SPEC-145), and
  `cycle: frame`.
- `## Context` and `## Goal`: what a recipe author can't do today, and what they can when this
  ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the orientation and metadata behaviour you drove, and the rule you recommend;
  - the recommended key spellings and direction, and the alternatives you rejected;
  - the refusal table;
  - sizing and why;
  - scope in/out;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-147-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-147 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-147** (framed <date>)`. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of the stage it belongs to, and update that stage's
   `**Count:**` line with a grep you've just run. Something with no stage goes in
   `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-147): frame rotate and flip as recipe steps`.
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
`grep -l "SPEC-147 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

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
First line: `SPEC-147 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- the orientation and metadata behaviour you drove, quoted, and the rule you recommend;
- the `angle` type and direction, and the `flip` axis naming, each with the alternative you
  rejected;
- the refusal table;
- one op or two, and the 180° = both-flips test;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them.

Then your measured cost.
