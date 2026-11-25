# SPEC-151 — DESIGN prompt: `pad` as a recipe step

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-151-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`5f6482a`, cost in `d457e02`).
**SPEC-151 writes no DEC**: the frame showed `pad` stays inside DEC-095, DEC-112 and DEC-113. If
you find a rule that needs one, stop and report rather than write it.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise check and the driven runs (alpha into JPEG, a tagged input, `resize fit` + pad);
- the keys and the two forms;
- where each per-image condition fails, and what a batch gets;
- the colour-type and bit-depth rule;
- the refusal table (shapes only), the exactness matrix, the scope, the two folded entries and the
  file list.

**The PROJ-011 orchestrator is running STAGE-063 (SPEC-149 and SPEC-150) at the same time.** It
has said it won't touch `src/operation/` or `src/recipe/`. If your rebase conflicts in a file
outside PROJ-017, stop and report.

## Rulings (settled — do not re-open)
The maintainer ruled on 2026-11-06, accepting all five of the frame's recommendations:
1. **Split.** SPEC-151 is `pad` with opaque fills. `transparent`, and any colour with alpha below
   `ff`, is refused at load here; a separate STAGE-058 entry lifts that later.
2. **`color` is required, with no default.** A `pad` step without it is refused at load.
3. **A `border` is a fraction of the long edge,** the same number of pixels on every side.
4. **`transparent` into a format with no alpha will be refused (exit 4)** by the later entry. Not
   built here, but SPEC-151's refusal message for `transparent` must not promise anything else.
5. **A grey image with a chromatic fill widens to RGB** (RGBA if it had alpha), at the same depth.
   A grey fill on a grey image keeps the luma type.

Also accepted from the frame:
- **One op, two forms told apart by their keys** (no `mode` key): `aspect` (+ optional `gravity`,
  default `center`) or `border`. Never both.
- **No pixel-size form.** `width`/`height` are refused as unknown keys. "Pad to 1080×1080" is
  `resize fit` then `pad aspect`.
- **No `pad` condition uses the refusal hook.** Two conditions fail at apply: a border that rounds
  to 0 px, and a canvas past resize's caps.
- **Reuse** crop's `parse_aspect` and its messages, `Gravity::placement`, `parse_color` and
  watermark's message, and resize's caps.
- **Folded in:** "SPEC-147's verify cleanups that need a PR" (a stricter `orientation_of`; the
  corpus README's literal table cut to point at the test files), and "State when an op uses the
  refusal hook and when it fails at apply" (the `Operation::refusal` doc comment). Mark both
  `(folded into SPEC-151)` entries' fate in your ACs; ship ticks them.

## Things in the frame to check before you build on them
- **The aspect canvas formula is worded loosely.** The frame says "its long side is the image's,
  and its short side is `round(long · b / a)`". That is wrong for a square or wrong-way image: a
  4000×4000 image padded to `16:9` has a canvas long side of about 7111, not 4000. State the rule
  per axis (whichever axis binds keeps the image's length; the other grows), give it as a formula a
  test can check, and drive at least: a square to `16:9` and `9:16`, a landscape to a portrait
  aspect, and an image one pixel off the aspect.
- **"Already at the aspect" is a no-op:** define it as "the computed canvas equals the image", not
  as an exact ratio test, and drive the near-miss (for example 1200×801 to `3:2`). Say what it
  gives.
- **The frame's 332,800-size sweep** claims the canvas is never smaller than the image and its
  short side is within 0.5 px. Trap 17: say which formula the sweep ran, and re-drive it against the
  formula you write, including `f64` near-ties and the largest aspect numbers (65535).
- **`pad` then `crop` is the identity.** The frame proposes it; drive it on the release binary with
  odd sizes before you make it an AC. Crop rounds fractions to pixels (DEC-112).
- **`Rgb32F` / `Rgba32F`** (a float TIFF can decode to them). The frame flags that DEC-095's
  helpers take the 8-bit route. Decide what `pad` does with them (keep the variant with a fill of
  `c / 255.0`, or something else) and give it a test, or refuse it with a message. Don't let it
  truncate silently.

## Design calls that are yours
- **The exact refusal messages.** Give each row of the frame's refusal table its exact text, in
  the existing `recipe step N ('pad'): …` shape, citing `file:line` for each `crop`, `watermark` and
  `resize` message you match. One AC row and one test case per refusal. Include `inf`, `-inf` and
  `nan` for `border`, and `"transparent"` and a translucent `#RRGGBBAA` for `color`. No message
  names a spec, stage or DEC id. The `transparent` message says it is not supported, in plain
  words, without naming a later spec.
- **The two apply failures:** exact messages (naming the image size and what was asked), exit 1 on
  one input and exit 6 in a batch with nothing written for the failing input. Drive the batch
  shape on crop's equivalent first, as the frame did, and make each a test.
- **The colour-type rule as a table a test can check:** each of the 8 integer colour types ×
  {grey fill, chromatic fill} → output type, plus the float variants. The 16-bit fill is `×257`;
  pin `#123456` on `Rgb16` as `(0x1212, 0x3434, 0x5656)`, and an alpha channel the input had stays,
  with the border at full alpha.
- **The odd pixel:** with `center` and an odd remainder, the extra fill goes right or bottom
  (`floor(remainder / 2)` on the left/top), matching crop. One test: a 4×3 image to `1:1` at
  `center` sits at `(0, 0)` with one fill row below.
- **The exactness tests**, pinned from the frame's matrix: for each colour type, a seeded image
  whose pixels are all distinct, at every gravity and an odd and an even remainder. Every input
  pixel is byte-equal at its offset, and every other pixel equals the fill. Seed the buffers
  independently of the code under test. Add **absolute position** rows (a named corner pixel lands
  where the gravity says), because equality over "every input pixel at its offset" needs the offset
  computed independently, not by calling `Gravity::placement` from the test.
- **The corpus rows.** Two recipes in SPEC-141's corpus (the frame proposes `v2_pad_aspect` with a
  non-centre gravity and `v2_pad_border`). Recount the literals yourself (the frame says
  `RECIPES` 18 → 20, `ROWS` 60 → 66, `GOLDEN_LINES` 63 → 69, the `54` → 60, `pairs.len()` 18 → 20)
  and write them into an AC. **No existing `.output` line may change:** an AC requires
  `git diff --diff-filter=MD origin/main -- tests/fixtures/recipe_corpus/` to show only the README
  (from the folded entry) and nothing else.
- **The folded README fix.** Say exactly what the "Adding a recipe" table says afterwards, so the
  build can't put numbers back in it.
- **The wasm rows.** `tests/wasm_roundtrip.rs`: a pad transform equal to the expected pixels, and
  a load refusal plus one apply failure giving the native message. ⚠ **Every wasm test you add
  changes `scripts/wasm-ci-check.sh`:** `FULL_COUNT` and `SLIM_COUNT` **and** the `--self-test`
  fixtures that name them. Re-read both counts on `main` after your last rebase (51/49 when this
  was written; PROJ-011's STAGE-063 might move them), write the new literals into an AC, and
  require `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and `… workflow` to pass.
- **Docs.** Re-check each surface in the frame's list at its `file:line`, find any it missed
  (`/usr/bin/grep -rn -i "\bpad\b"` over `docs/ recipes/ skills/ README.md`), and say what each
  line will say. Each is an AC verify checks by reading. The `pad` entry says: `color` is required,
  what a border fraction is of, that a grey image with a coloured fill becomes RGB, and that
  `transparent` isn't supported yet. It points at `resize fit` + `pad aspect` for an exact box.

## Tests: each one must be able to fail
HANDOFF traps 12, 14 and 17 bind here, and SPEC-147's ship asked for this: **drive the cheap
mutation rows at design; don't reason them.** Each row names the code location, every test
expected red and every test expected green, and is marked *driven* or *reasoned*. Drive them on a
scratch copy of the code in a fresh target dir, never committed. At minimum:
- **Gravity floor → ceil** on the odd remainder. The 4×3 row and the absolute-position rows go red.
- **Gravity swapped** (for example `north` placed as `south`). The position rows go red; say which
  equality rows stay green.
- **`×257` → `<< 8`** for the 16-bit fill. The `#123456` row goes red.
- **Long edge → short edge** for the border. The border size rows go red.
- **The opaque check removed.** The translucent and `transparent` refusals go red.
- **The 0 px check removed.** Its apply-failure row goes red; say what is written instead.
- **The cap check removed.** Its row goes red (use a cheap case that doesn't allocate the canvas;
  say how).
- **Colour-type widening removed** (grey + chromatic fill keeps grey). The colour-type table rows
  go red.
- **wasm count:** `FULL_COUNT` one lower. The count check and the self-test go red.
- **Corpus guard:** add a `pad` step to an existing corpus recipe on a scratch copy. SPEC-141's
  guard goes red, naming that recipe.
- **`orientation_of`:** feed it a malformed `eXIf` chunk. The stricter version fails; the old
  `.ok()?` version passes.
- Trap 14: a test of the real code path calls the production function, not a copy. And restore
  `main`'s registry without the `pad` line: list everything that goes red.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and
Implementation Context. You don't write the code. Separate build and verify sessions follow: one
builds the spec from your text alone, and another verifies it, so everything the builder needs
must be in the spec (AGENTS §15).

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own. Keep it plain text with short
lists; a wide box-drawn table can garble when copied out of the terminal.

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" (rule 7), the
   rules and "Traps this wave paid for" (traps 11, 12, 14 and 17 especially). PROJ-017 has no
   HANDOFF of its own, so this one binds.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`.
4. `/guidance/constraints.yaml`, DEC-095, DEC-102, DEC-110, DEC-112, DEC-113, and the registry DEC
   behind SPEC-131.
5. **SPEC-147 in `specs/done/`, as the shape to match**: its ACs, refusal messages, mutation table
   (and the two cells its build corrected), corpus rows, wasm rows, Build Completion and ship
   reflection. SPEC-145 for `crop`, whose parser and placement `pad` reuses.
6. `tests/fixtures/recipe_corpus/README.md` and `scripts/wasm-ci-check.sh`.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `crop.rs` (`parse_aspect`,
  `aspect_size`, the 0 px apply failure), `Gravity::placement`, `parse_color`, resize's caps, the
  DEC-095 narrowing helpers, the registry's `ParamKind::Number` and `OneOf`, and the wasm entry
  point.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR`
  that you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, and no change to `CACHE_SCHEMA_VERSION` or the lockfile format. Say in an AC
  why a new op needs neither.
- Sizing: if the design comes out bigger than [S], say so and why in your report.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report, rather than silently widening or
narrowing it.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-151 build`.
2. In STAGE-058's backlog, change SPEC-151's marker from `(framed 2026-11-06)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add
   `prompts/SPEC-151-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-151): design pad as a recipe step`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back `git show --stat HEAD`. ⚠ `just backlog` reads only the active project (PROJ-011),
   so use `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC. No pushed probe
  branch. No version bump, no tag.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect output and read
  `$?`. zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can
  truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you
  count.
- Never share a `CARGO_TARGET_DIR` with another session. Driving mutations on a scratch copy
  rebuilds the crate each time; clean the copy and its target dir before you finish.
- **Budget ~50 exchanges** (the driven mutation rows are the extra ten).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-151 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read
is ×0.10. Measure at your last turn, after the push, and append a `design` entry to the spec's
`cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-151 design: DONE | SPLIT | BLOCKED`, then the commit SHA on `main`. Then, short:
- any correction to the frame (the canvas formula, the no-op definition, the sweep, the float
  variants), with what you drove;
- the AC list (one line each);
- the Failing Tests and the mutation table, with the rows expected red and green, each marked
  driven or reasoned;
- the colour-type table;
- the corpus rows, the new literals, and the new `FULL_COUNT`/`SLIM_COUNT`;
- the docs surfaces, each with `file:line`;
- size, and anything the build prompt must carry (checkpoints, CI legs, files PROJ-011 must keep
  clear).

Then your measured cost.
