# SPEC-147 — DESIGN prompt: `rotate` by quarter turns and `flip`, as recipe steps

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-147-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`c9a33af`, cost in `6198d77`).
**Reserved: SPEC-147.** This spec writes **one DEC**: what a geometry step does with metadata.
See "Ids" below.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise check, and the exiftool runs on tagged inputs;
- the recommended metadata rule;
- the keys, the direction and the `flip` naming;
- the refusal table;
- exactness, native versus wasm, the docs surfaces, the sizing, the scope and the file list.

**The PROJ-011 orchestrator is running its last stage (STAGE-063, SPEC-148 to 150) at the same
time.** It has said it won't touch `src/operation/` or `src/recipe/`. If your rebase conflicts in
a file outside PROJ-017, stop and report.

## Rulings (settled — do not re-open)
The maintainer ruled on 2026-11-05, accepting all four of the frame's recommendations:
1. **A positive `angle` is clockwise**, as displayed after any earlier step. `-90` turns
   anticlockwise.
2. **`flip` keeps `axis = "horizontal" | "vertical"`.** `horizontal` mirrors left and right, and
   `vertical` mirrors top and bottom. The doc entry says so in one sentence.
3. **The non-multiple refusal names no stage or spec id.** It reads, for example, `'angle' must
   be a multiple of 90, got 45; rotating by any other angle is not supported yet`. User-facing text
   carries no internal ids.
4. **`angle = 0`, `360` and `-360` load as a no-op.** The page can then emit a zero straighten
   as a valid step.

Also accepted from the frame:
- **`angle` is a TOML number, integer or float, read as `f64`, in [-360, 360].** The key table
  checks type, finiteness and range; the constructor checks "a multiple of 90" and normalises to
  0/90/180/270. STAGE-059 later lifts only the multiple-of-90 check, so the key's type and range
  never change.
- **Two ops, one spec:** `rotate` and `flip`, each with its own key table.
- **Reuse the pixel work `AutoOrient` already does** (`DynamicImage::rotate90`,
  `rotate180_in_place`, `rotate270`, `fliph_in_place`, `flipv_in_place`). They keep every colour
  type and bit depth (DEC-095).
- **Every refusal is at load (DEC-102).** None needs the image, none uses the refusal hook, and
  `apply` has no failure path.
- **The metadata rule:** a geometry step acts on the buffer it receives and writes no metadata. A
  recipe meant as the photo is displayed starts with `auto-orient`.

## Design calls that are yours
- **The DEC.** Write the metadata rule as a new DEC, with the frame's exiftool table as its
  evidence. It covers:
  - the rule itself, and that it changes nothing `crop` shipped;
  - why the "stale tag rotates twice" trap can't happen today (the pixel lane writes no EXIF,
    DEC-003), and that the live trap is the opposite: a recipe without `auto-orient` drops the
    tag without applying it;
  - **the one change that would reopen the trap:** if the pixel lane ever carries EXIF through,
    it must set Orientation to 1 and fix or drop the pixel dimensions. Say where that obligation is
    written so whoever makes the change meets it (the DEC alone isn't enough: name the code site
    that would carry EXIF, and say whether a comment or a test holds the line);
  - the rejected alternatives (compose the tag; write Orientation = 1; auto-orient implicitly in
    `apply`), and why.

  Use the block-list `affected_scope` form.
- **The exact refusal messages.** Give each row of the frame's refusal table its exact message, in
  the existing `recipe step N ('rotate'): …` shape, citing `file:line` for the `crop` and
  `resize` messages you match. One AC row and one test case per refusal. Two refusals that share
  a message must each still be driven. Include `inf` and `-inf` as well as `nan`.
- **Normalisation, stated so a test can check it.** Map every accepted angle to 0/90/180/270
  (for example `-90 → 270`, `-270 → 90`, `±180 → 180`, `0/±360 → 0`). Give the full table, one
  row per accepted value in {−360, −270, −180, −90, 0, 90, 180, 270, 360}, and require a test per
  row. Say what a no-op step does: returns the buffer unchanged, and its corpus output must be
  byte-identical to the input's.
- **The exactness tests.** The frame sizes them; pin them:
  - four `rotate 90`s and two flips each return the input's pixels exactly, across the frame's ten
    colour types, on a seeded non-square buffer (7×5);
  - **absolute position**, not just round trips: a round trip passes even if the op rotates the
    wrong way. Check that one named pixel lands where clockwise says, and that `horizontal`
    mirrors left↔right. Make sure a mutation that turns anticlockwise goes red here;
  - `rotate 180` equals `flip horizontal` then `flip vertical`, per colour type;
  - a 90° turn swaps the dimensions.
- **The orientation row.** One test on an input tagged orientation 6 that was not auto-oriented:
  `rotate` acts on the stored pixels, and the output carries no Orientation tag. Say what reads the
  output's tags, and make it a reader other than crustyimg's own writer where you can.
- **The wasm rows.** Say where they live (`tests/wasm_roundtrip.rs`) and what they assert:
  dimensions and pixels exactly, plus the same refusal text at load. ⚠ **Every wasm test you add
  changes `scripts/wasm-ci-check.sh`:** `FULL_COUNT` and `SLIM_COUNT`, **and** the `--self-test`
  fixtures, which hardcode the counts (around :204–225: the fixture log lines and rows 1–5 and 9).
  SPEC-145 changed only the constants and went red in `front-matter validation`. Write the new
  literals into an AC, and require `scripts/wasm-ci-check.sh --self-test` and
  `scripts/wasm-ci-check.sh workflow` to pass (run them with `LC_ALL=en_US.UTF-8`; the workflow
  check fails on a US-ASCII locale).
- **The corpus rows.** The frame proposes two recipes in SPEC-141's corpus. Name them and their
  steps, and give the new golden-line count as a literal (the frame says 57 → 63; recount it).
  Follow `tests/fixtures/recipe_corpus/README.md` exactly. **No existing `.output` line may
  change:** an AC requires a diff of `tests/fixtures/recipe_corpus/` against `origin/main` that
  shows only added files and the bumped literals.
- **Docs.** The frame lists the surfaces (`docs/recipes.md:80` and `:242`, `docs/data-model.md`,
  `docs/backlog.md:191-192`, `recipes/README.md`, `skills/crustyimg/SKILL.md`). Re-check each
  `file:line`, find any it missed (`/usr/bin/grep -rn "rotate\|flip"`), and say what each line
  will say. Each doc line is an AC that verify checks by reading. The direction and `flip`
  sentences from rulings 1 and 2 are two of them. The `rotate` doc entry also says to put
  `auto-orient` first for a recipe meant as the photo is displayed.

## Tests: each one must be able to fail
HANDOFF traps 12 and 14 bind here. **Each mutation row names the code location where it's
applied, and lists every row expected red and every row expected green.** If a claim in the table
is something you worked out rather than drove, mark it as such: SPEC-145's verify passed on an
"analytic maximum" that `f64` broke, and that cost two extra rounds. At minimum:
- **Direction mutation:** `rotate90` swapped for `rotate270`. The absolute-position test goes red;
  say which round-trip tests stay green, which is why the position test exists.
- **Flip-axis mutation:** `fliph` swapped for `flipv`. The mirror-position test goes red.
- **Normalisation mutation:** map `-90` to 90. That row's test goes red.
- **No-op mutation:** treat `0` as 90. The no-op row and its corpus output go red.
- **Range mutation:** remove the [-360, 360] bound. The `450` refusal goes red.
- **Multiple-of-90 mutation:** remove the check. The `45` refusal goes red, and say what `apply`
  would do with 45 if the check were gone.
- **Metadata mutation:** if cheap, make the op carry the input's Orientation tag through. The
  orientation row goes red.
- **wasm count mutation:** set `FULL_COUNT` one lower. The count check goes red, and the self-test
  stays green only if its fixtures were moved with it.
- **Corpus guard:** add a `rotate` step to an existing corpus recipe on a scratch copy. SPEC-141's
  guard goes red, naming that recipe.
- Trap 14: a test of the real code path calls the production function, not a copy.

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
   "Traps this wave paid for" (traps 11, 12 and 14 especially). PROJ-017 has no HANDOFF of its
   own, so this one binds.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`.
4. `/guidance/constraints.yaml`, DEC-003, DEC-095, DEC-102, DEC-110, DEC-112, and the registry DEC
   behind SPEC-131.
5. **SPEC-145 in `specs/done/`, as the shape to match.** It's the most recent op added this way:
   its ACs, refusal messages, mutation table, corpus rows, wasm rows, Build Completion and ship
   reflection.
6. `tests/fixtures/recipe_corpus/README.md` and `scripts/wasm-ci-check.sh`.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `AutoOrient` and its tests,
  the registry's key tables and `ParamKind::Number`, `crop`'s registration (the newest pattern),
  how a step's output reaches the encoder without metadata, and the wasm entry point.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR`
  that you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them. That
  includes every refusal message you write (ruling 3).
- No version bump, and no change to `CACHE_SCHEMA_VERSION` or the lockfile format. Say in an AC
  why a new op needs neither.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in your report rather than silently
widening or narrowing it. For the DEC, run `ls decisions/` after your last rebase and take the
next free id. DEC-112 was the last when this was written, and the PROJ-011 orchestrator's SPEC-148
is in design now and may take DEC-113. Check again just before you push, and rename if you've
been beaten to it.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-147 build`.
2. In STAGE-058's backlog, change SPEC-147's marker from `(framed 2026-11-05)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary that includes
   the four rulings, and add `prompts/SPEC-147-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-147): design rotate and flip, with the geometry-metadata DEC`.
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
- **Budget ~40 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-147 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read
is ×0.10. Measure at your last turn, after the push, and append a `design` entry to the spec's
`cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-147 design: DONE | SPLIT | BLOCKED`, then the commit SHA on `main`. Then, short:
- the DEC id and its one-line decision, and where the "reopen" obligation is held;
- the AC list (one line each), and the Failing Tests / mutation table with the rows expected red
  and green, marking any row you reasoned rather than drove;
- the normalisation table;
- the corpus rows added, the new golden-line literal, and the new `FULL_COUNT`/`SLIM_COUNT`;
- the docs surfaces, each with `file:line`;
- size, and any place you disagree with the frame;
- anything the build prompt must carry (checkpoints, CI legs, files the PROJ-011 orchestrator
  must keep clear).

Then your measured cost.
