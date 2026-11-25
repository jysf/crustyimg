# SPEC-145 — DESIGN prompt: `crop` as a recipe step, in units that survive a resize

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-145-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`a96f0da`). **Reserved:
SPEC-145.** This spec writes **one DEC**: the crop unit rule. See "Ids" below.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise check;
- the preview-versus-original numbers;
- the recommended unit syntax and rounding rule;
- the two forms;
- the refusal table;
- colour type, orientation and native-versus-wasm;
- the docs surfaces, the sizing, the scope and the rough file list.

**Another orchestrator may be running PROJ-011 work at the same time.** It's been told to stay
out of `src/operation/`, `src/recipe/`, `docs/recipes.md` and the recipe corpus. If your rebase
conflicts in a file outside PROJ-017, stop and report.

## Rulings (settled — do not re-open)
The maintainer ruled on 2026-11-05, accepting all three of the frame's recommendations:
1. **Fractions only.** A recipe's crop carries fractions from 0 to 1 of the image the step
   receives, as TOML numbers. No pixel units in a recipe. DEC-091 (Fence A) is unchanged. Pixels
   for CLI users arrive later through `edit --crop`, which converts them to fractions. That flag
   is the `edit` flags spec's work, not this one.
2. **`aspect` takes whole numbers only**, as `"W:H"` with positive integers: `"239:100"`, not
   `"2.39:1"`. A decimal is refused at load with a message that shows the integer spelling.
3. **A crop that rounds to 0 px is refused, never clamped.** It exits 1 at apply, naming the
   step, the way DEC-109 refuses a watermark that would draw nothing.

Also accepted from the frame:
- **One `crop` op with two key groups that can't be mixed:** rectangle (`x`, `y`, `width`,
  `height`) or aspect (`aspect`, `gravity`, defaulting to centre). No `mode` key. Mixing groups,
  a partial rectangle, and no keys at all are each refused at load.
- **Rounding:** each edge rounds on its own, in `f64`, with halves rounded up:
  - left = round(x·W), right = round((x+width)·W);
  - top = round(y·H), bottom = round((y+height)·H).

  Load refuses x + width > 1 and y + height > 1, so the crop stays inside the image without
  clamping.
- **Reuse** `imageops::crop_imm` and `Gravity::placement`. Don't reuse resize's `crop_center`.
- **Orientation is deferred.** Crop's coordinates are the buffer the step receives. The rotate/flip
  spec owns the orientation-tag question.
- **The split, if needed:** (a) the DEC plus the rectangle form, then (b) the aspect form. (a)
  alone unblocks the page. Take it only if the AC and test count makes the spec more than an
  [M], and say so.

## Design calls that are yours
- **The DEC.** Write the crop unit rule as a new DEC, with the rulings above as its decision. It
  covers:
  - why fractions (the preview-versus-original numbers, and Fence A);
  - the rounding rule, stated so a test can check it;
  - the rejected alternatives and why: telling units apart by TOML type, `"10%"` strings, and a
    pixel key family;
  - what `pad` should inherit. `pad` is the next spec to carry lengths, so state the rule it
    follows, or state that `pad` decides for itself.

  Use the block-list `affected_scope` form.
- **Update STAGE-058's text to match ruling 1.** Its Scope line (about :26, "in pixels or in
  fractions of the image") and its crop backlog entry (about :90, "Every length is accepted in
  pixels or as a fraction") both promise pixels. Rewrite both to say fractions in recipes, with
  pixels via `edit --crop`. Change nothing else in the stage.
- **The exact load-refusal messages.** The frame lists about 17 load refusals. Give each one its
  message, in the existing `recipe step N ('crop'): …` shape (read how `resize` and `watermark`
  word theirs, and cite `file:line`). Don't let one test cover a refusal class: one AC row and one
  test case per refusal. Two refusals that share a message must still each be driven.
- **The `aspect` placement maths.** Given W×H and an aspect a:b, state the largest a:b rectangle
  that fits, then where `gravity` puts it. Use integer maths, which ruling 2 makes possible. Say
  how the short side rounds, and when that gives 0 px (ruling 3). Show one worked example with
  numbers that don't divide evenly.
- **The property tests.**
  - **Size independence:** the same fractions on the same image at 1200 px and 6000 px give edges
    within 1 px when scaled. The frame measured 0.40 px max over 200,000 cases. Fix the case
    count and the RNG seed, so a run is repeatable.
  - **Exactness:** every output pixel equals the input pixel at the offset (DEC-095), on rgb8,
    rgba8 and rgba16.
- **The wasm row.** Say where it lives (`tests/wasm_roundtrip.rs` per the frame) and what it
  asserts: dimensions exactly, pixels exactly (crop doesn't resample, so DEC-110's 1/255
  allowance doesn't apply), and the same refusal text at load.
- **The corpus rows.** The frame proposes two `v2_crop_*` recipes in SPEC-141's corpus. Name them,
  name their steps, and give the new golden-line count as a literal. Then follow
  `tests/fixtures/recipe_corpus/README.md` exactly. **No existing `.output` line may change:** an
  AC requires a diff of `tests/fixtures/recipe_corpus/` against `origin/main` that shows only
  added files and the bumped literals, with no modified `.output`.
- **Docs.** `docs/recipes.md:60-65` becomes a recipe-step entry, and smart crop moves to the
  post-1.0 row. Find the other op tables (`recipes/README.md`, `skills/crustyimg/SKILL.md`,
  `docs/cli-reference.md` if it lists ops, the wasm README if it lists ops) and list each one with
  `file:line`. Each doc line is an AC that verify checks by reading, so say what each one says.

## Tests: each one must be able to fail
HANDOFF traps 12 and 14 bind here. **Each mutation row names the code location where it's applied,
and lists every row expected red and every row expected green.** At minimum:
- **Rounding mutation:** floor instead of round on the right edge. The rounding property test goes
  red, and a named exact-edge case goes red with its numbers. Name every other row that stays green.
- **Bounds mutation:** remove the load check for x + width > 1. That refusal's case goes red. And
  because ruling 3 forbids clamping, show what the apply path does instead. If it panics in
  `crop_imm` or silently clamps, the mutation must still turn something red. Say what.
- **Zero-px mutation:** clamp to 1 px instead of refusing. The 0-px apply case goes red.
- **Mixed-forms mutation:** accept both groups and silently prefer one. The mixed-forms case goes
  red.
- **Aspect-maths mutation:** swap W and H in the fit. The worked example goes red.
- **Wasm mutation:** if cheap, change the wasm path's rounding alone, and the wasm row goes red.
- **Corpus guard:** add a crop step to an existing corpus recipe on a scratch copy, and SPEC-141's
  guard goes red naming that recipe. This proves the "no existing output changes" AC can fail.
- Trap 14: a test of the real code path calls the production function, not a copy. Restoring the
  old code verbatim must turn it red.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests, Implementation
Context and the DEC. You don't write the code. Separate build and verify sessions follow: one
builds the spec from your text alone, and another verifies it, so everything the builder needs
must be in the spec (AGENTS §15).

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (now 16; traps 11, 12 and 14 especially). PROJ-017 has no HANDOFF
   of its own, so this one binds.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`.
4. `/guidance/constraints.yaml`, DEC-091, DEC-095, DEC-102, DEC-109, DEC-110, and the registry DEC
   behind SPEC-131.
5. The SPEC-141 spec in `specs/done/`, and `tests/fixtures/recipe_corpus/README.md`: how a new op
   adds rows.
6. A recent shipped op spec as the shape to match: the one that added `watermark` to the registry
   (STAGE-050, `projects/PROJ-011-…/specs/done/`), including its Build Completion and Verify
   Readout.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: the registry's key tables, the
  load-time validation, how a step gets the image size at apply, `Gravity::placement`, and the
  wasm entry point.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them. That
  includes every refusal message you write.
- No version bump, and no change to `CACHE_SCHEMA_VERSION` or the lockfile format. Say in an AC
  why a new op needs neither.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in your report rather than silently
widening or narrowing it. For the DEC, run `ls decisions/` after your last rebase and take the
next free id (DEC-111 was the last when this was written). Other sessions add DECs in parallel,
so check again just before you push.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-145 build`.
2. In STAGE-058's backlog, change SPEC-145's marker from `(framed 2026-11-04)` to
   `(designed <date>)`, and make the two text fixes above. Mark design `[x]` in the timeline with
   a 2–4 line summary, and add the build prompt path `prompts/SPEC-145-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-145): design crop as a recipe step, with the unit DEC`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back `git show --stat HEAD`, and run `./scripts/decisions-audit.sh` to see the new DEC
   listed with its scope. ⚠ `just backlog` reads only the active project (PROJ-011), so use
   `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline, the stage file and the DEC. No pushed probe
  branch. No version bump, no tag.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect output and read
  `$?`. zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can
  truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you
  count.
- Never share a `CARGO_TARGET_DIR` with another session.
- **Budget ~50 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-145 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read
is ×0.10. Append a `design` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown`, `estimated_usd` and the rate anchors, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-145 design: DONE | SPLIT | BLOCKED`, then the commit SHA on `main`. Then, short:
- the DEC id and its one-line decision;
- the AC list (one line each), and the Failing Tests / mutation table with the rows expected red
  and green;
- the rounding rule and the aspect maths, with the worked example;
- the corpus rows added, and the new golden-line literal;
- the docs surfaces, each with `file:line`;
- size, whether you split, and any place you disagree with the frame;
- anything the build prompt must carry (checkpoints, CI legs, which files the PROJ-011
  orchestrator must keep clear).

Then your measured cost.
