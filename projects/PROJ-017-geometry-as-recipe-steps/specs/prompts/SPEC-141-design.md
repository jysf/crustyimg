# SPEC-141 — DESIGN prompt: no existing recipe's output changes (the PROJ-017 guard)

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-141-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`5dfe0da`, plus `656d959`).
**Reserved: SPEC-141.** No DEC is expected. If you find one is needed, see "Ids" below.
SPEC-142 (the parity harness) may be in design at the same time. It touches different files.

**Read the stub's `## Frame` section first.** It holds:
- the premise check, including a filter mutation that every existing test missed;
- the recipe inventory;
- the pin rule per format;
- the fixtures, the controls and the blessing rule;
- the sizing, the scope and the rough file list.

It is your binding scope.

## Rulings (settled — do not re-open)
The maintainer ruled on 2026-10-30, accepting all three of the frame's recommendations:
1. **If the CI legs give different pixels, pin exactly, per OS and architecture.** Never use a
   tolerance. A loosened pin is the failure the PROJ-017 brief warns against.
2. **No byte pins for AVIF or the `optimize` search** (DEC-059 Option A). Pin the chosen format,
   the dimensions and the pixel hash of the pre-encode image instead. The only exception: the
   build's CI run shows every leg agrees on AVIF bytes. Even then, adding the pin is a verify
   question, not a build decision.
3. **Add the ddphotos-shaped recipe to the corpus now**
   (`tests/fixtures/recipe_corpus/v2_ddphotos_cli_shape.toml`, with its `.golden` and `.output`).

Also accepted from the frame:
- The guard lives in a new `tests/recipe_corpus_output.rs`, so `cargo test` runs it on every CI
  leg. `scripts/cli-golden.py` stays out of CI.
- The pin is taken **after the last step and before encoding**: width, height, colour type, bit
  depth and a SHA-256 of the pixels. PNG and lossless WebP also pin the encoded bytes.
- The fixtures are RGB8 2600×1730, RGBA8 1100×700 and RGBA16 1100×700. They are generated in the
  test, never by crustyimg. Every recipe step must actually do something on each fixture.
- The rule: a changed `.output` in a PROJ-017 PR is a verify finding, unless that spec's
  acceptance criteria name the recipe.

## Design calls that are yours
- **The cross-leg unknown is resolved in build, not here.** Nothing pins resize pixels today.
  Resize uses `f32` `powf` and different SIMD paths on x86 and ARM, so whether the legs agree is
  unknown. Don't push code to find out. Instead, write the spec so **both outcomes are already
  specified**:
  - **(i)** If every leg agrees, there is one `.output` per recipe.
  - **(ii)** If any leg differs, there are per-target goldens. Specify:
    - the file naming (for example `<stem>.<os>-<arch>.output`);
    - how the test selects the golden for its leg (`cfg!(target_os)`, `cfg!(target_arch)`);
    - what happens on a target with no golden: it must **fail**, never skip (HANDOFF trap 11:
      a printed skip is invisible in CI);
    - how a leg's goldens get blessed when the maintainer has only the Mac. One option is a
      CI artifact the build downloads with `gh run download`; choose and specify one.

  Make the build's **first checkpoint** a WIP draft PR carrying the test and the Mac goldens.
  Its CI result picks (i) or (ii). The build reads that result with a backgrounded
  `gh pr checks --watch`, never a poll loop. Then make an acceptance criterion assert whichever
  layout the build lands, with the leg list read from `ci.yml`, not typed from memory.
- **The golden format.** Choose a `.output` layout that a human can read in a diff. When a pin goes
  red, the message must name the recipe, the fixture, and which field changed (dimensions, colour
  type, depth, pixels or bytes). Show one example failure message in an AC.
- **Blessing.** The frame recommends a `CRUSTYIMG_BLESS` switch, like the one the corpus test has
  (`tests/recipe_round_trip.rs`, about :374). Specify it so that a bless can't happen in CI. Say
  how: for example, refuse when `CI` is set.
- **The row-count assertion.** 14 recipes × 3 fixtures = 42 pixel rows, plus the byte rows. The
  test asserts the count **as a literal**, so a recipe dropped from the list goes red. The new op
  specs will add rows, so say how the literal gets updated and that verify checks it.
- **Where the pin is taken.** "After the last step, before encoding" needs a seam: run the pipeline
  through the library API, or run the binary and decode its output. Read how `apply` drives
  `Pipeline` and the encoder, and cite `file:line`. Choose the seam that would still catch a change
  in the encoder settings for the byte-pinned formats.
- **The `docs/recipes.md` crop-verb finding** (filed on STAGE-058) is out of scope. Leave it alone.

## Tests: each one must be able to fail
HANDOFF trap 12 binds here. **Each mutation row names the code location where it is applied, and
lists every row expected red and every row expected green.**
- **Filter mutation:** `src/operation/mod.rs:584`, Lanczos3 → CatmullRom. List exactly which
  rows go red (every recipe that resizes, on every fixture) and which stay green (the rows that
  don't resize). The frame drove this against the old tests. Make it a required verify step
  against the new test.
- **Watermark mutation:** a default in the watermark op (name the line). Red on the watermark rows
  only. List them.
- **Positive control:** each row asserts its expected output dimensions as literals, so a guard that
  runs no steps can't pass with an all-identity golden.
- **Row-count control:** remove one recipe from the list, and the count assertion goes red.
- **Bless control:** with `CI` set, a bless refuses and writes nothing. Hash the golden directory
  before and after.
- The guard is new, so it isn't "RED on `main`" in the usual sense. Its Failing Tests entry is the
  set of mutation rows above, each run against the built test.

**Size.** The frame says [S]: about 250 lines of test, about 14 goldens, and no `src/` change. The
per-target branch may double the golden count but not the code. If the design grows past S,
report the cut rather than applying it silently.

## Files
The frame's rough list:
- `tests/recipe_corpus_output.rs` (new)
- `tests/fixtures/recipe_corpus/*.output` and `bundled_*.output` (new)
- `tests/fixtures/recipe_corpus/v2_ddphotos_cli_shape.toml` and its `.golden` (new)
- `tests/recipe_round_trip.rs`: add the new recipe, and change the count from 13 to 14
- `tests/fixtures/recipe_corpus/README.md`: document `.output` and the bless rule
- possibly `tests/common/mod.rs`, for a seeded fixture generator

No `src/` change. If the design finds one is needed, that's a frame error: report it.

## DECs to read
DEC-058 (cache key), DEC-059 (lockfile, Option A), DEC-095 (colour type and depth preserved),
DEC-102 (params validated at load), and any DEC whose scope names `tests/recipe_round_trip.rs` or
the recipe corpus (`./scripts/decisions-audit.sh`).

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-141-*.md` with `cycle: design`. If it's
missing or still at `cycle: frame`, stop and report. Don't create it. A separate session builds
the spec from your text alone, and another verifies it, so everything the builder needs must be
in the spec (AGENTS §15).

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (traps 11 and 12 especially). PROJ-017 has no HANDOFF of its own, so
   this one binds.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`.
4. `/guidance/constraints.yaml` and every DEC named above.
5. A recent shipped spec as the shape to match: `projects/PROJ-011-surface-reach-and-predictability/specs/done/SPEC-135-*.md`,
   including its Build Completion and Verify Readout. It shows what verify will do to your tests.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control. **Measure the scenario, not
  the control.** For example, confirm on each fixture that each recipe's steps change the image.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, and no change to `CACHE_SCHEMA_VERSION` or the lockfile format.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in your report rather than silently
widening or narrowing it. If you must write a DEC, run `ls decisions/` after your last rebase and
take the next free id. Other sessions add DECs in parallel. Use the **block-list**
`affected_scope` form (an inline array is silently dropped by `scripts/decisions-audit.sh`).

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-141 build`.
2. In STAGE-058's backlog, change **only** SPEC-141's marker from `(framed 2026-10-30)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add the build
   prompt path `prompts/SPEC-141-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-141): design the recipe-output guard`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. SPEC-142's
   design touches STAGE-060, not STAGE-058, so the rebase should be clean.
5. Read back `git show --stat HEAD`. ⚠ `just backlog` reads only the active project (PROJ-011),
   so use `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline, the stage file and (only if needed) a DEC. No
  pushed probe branch. No version bump, no tag.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect output and read
  `$?`. zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can
  truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you
  count.
- If SPEC-142's design is running beside you, never share a `CARGO_TARGET_DIR` with it.
- **Budget ~50 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-141 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read
is ×0.10. Append a `design` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown`, `estimated_usd` and the rate anchors, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-141 design: DONE | BLOCKED`, then the commit SHA on `main`. Then, short:
- the AC list (one line each), and the Failing Tests / mutation table with the rows expected red
  and green;
- the golden layout, both outcomes (i) and (ii), and the build's first checkpoint;
- the pin seam, with `file:line`;
- size, and any place you disagree with the frame;
- anything the build prompt must carry.

Then your measured cost.
