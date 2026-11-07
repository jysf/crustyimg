# SPEC-152 — DESIGN prompt: `trim` as a recipe step

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-152-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`539f727`, cost in `46cf6d4`).
**SPEC-152 writes no DEC**: the frame showed `trim` stays inside DEC-095, DEC-102, DEC-112 and
DEC-113, and the maintainer ruled the tolerance scale an op rule in the doc entry. If you find a rule
that needs a DEC, stop and report rather than write it.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise check, and ImageMagick's `-trim`/`-fuzz` driven on eight inputs;
- the tolerance scale, driven in a Python model, including the float rule and its splitting input;
- the keys, the corner rule, the alpha rule, and where each per-image condition fails;
- the `pad`-then-`trim` inverse and its condition;
- the refusal table (shapes only), the exactness matrix, the corpus and wasm plan, the fold-in and
  the file list.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables**: wide tables have lost lines in transit.

## Rulings (settled — do not re-open)
The maintainer ruled on 2026-11-06, accepting all seven of the frame's recommendations:
1. **With no `color`, `trim` cuts against the top-left pixel**, read in the image's own samples, and
   trims all four sides against it.
2. **`tolerance` is in 8-bit steps at every depth**: as written at 8 bits, `× 257` at 16 bits, and a
   float sample is read as its nearest 16-bit value, then the 16-bit rule applies. Written in the doc
   entry, with no DEC.
3. **A pixel with alpha 0 matches a colour with alpha 0, whatever colour is hidden under it.**
   Otherwise every channel, alpha included, is compared.
4. **"Uniform" means one colour, not one width.** Each side is trimmed on its own to the bounding box
   of the pixels that don't match.
5. **The whole image matching fails at apply** (exit 1 on one input; in a batch that input fails, the
   others are written, exit 6). No `trim` condition uses the refusal hook.
6. **`tolerance` is an integer**, 0..=255, default 0 (exact). `8.0` is refused as the wrong type.
7. **`color` is optional and may be translucent** (`#00000000` trims a transparent margin), unlike
   `pad`'s.

Also accepted from the frame:
- **Keys:** `color` and `tolerance`, nothing else. `fuzz`, `threshold`, `sides`, `border`, `margin`
  are refused as unknown keys.
- **Nothing to trim is a no-op** that returns the input unchanged, as `pad` already at its aspect.
- **Folded in:** "Pin `resize`'s cap message text" (test-only). Not folded: the watermark-gravity,
  guard-reporting and "watermark skipped" entries.
- **Corpus:** two recipes that pad twice and trim once (`v2_trim_corner`, `v2_trim_tolerance`), and
  two wasm rows.

## Another session is building in the same files
**PROJ-011's SPEC-150 build is about to start (or running).** Its design touches files this spec
also touches:
- `src/operation/registry.rs` and `src/operation/mod.rs`: test-assertion messages that carry a spec
  id (about `registry.rs:657` and `mod.rs:2563`);
- `src/wasm.rs` (the asset refusal, about :236-240);
- `tests/wasm_roundtrip.rs:283`, which asserts on that refusal's text;
- a new check that scans `src/` for user-facing strings that name a spec id.

Different hunks, but the same files. Read SPEC-150's spec (`projects/PROJ-011-…/specs/`) for its
exact file list. In your Implementation Context, name every file both specs change, say which hunks
are whose, and tell the build to rebase onto `main` before each push and re-read the wasm counts
after the last rebase. No string `trim` adds may name a spec, stage or DEC id, so SPEC-150's new
check stays green. If SPEC-150 has merged by the time you design, design against `main` as it is.

## Things in the frame to check before you build on them
SPEC-151's frame had a canvas formula that was wrong for a square image, and design caught it by
driving. Do the same here:
- **The float rule** ("0 wrong in either direction, exhaustively") was driven in Python. Re-drive it
  in Rust, in a throwaway prototype of the comparison: every `c` and `t` in 0..=255, the `f32` value
  exactly `t` steps away and one past, through the same `f32` → `f64` → round path the build will
  write. Say which path you drove. Then decide **NaN** (the frame leaves it to you): what a NaN
  sample compares as, and a row that pins it. Also decide out-of-range floats (below 0, above 1).
- **The inverse condition** ("held in 19,764 of 19,764 random cases") was a Python model of both
  ops. Drive it through the release binary's real `pad` and a prototype `trim` on a handful of odd
  cases, including the intended failure (an all-white top row padded with white).
- **The corpus recipes.** The fixtures are noisy gradients (`tests/recipe_corpus_output.rs:132`).
  Check that each recipe's every step acts and the output differs from the fixture under the
  guard's own controls, and compute the expected dims yourself. Confirm `v2_trim_tolerance` pins the
  16-bit edge on `rgba16_2600x1730` (`#080808` is `0x0808`, 2056 from `#000000`, at `tolerance = 8`):
  a `<` for `<=` mutant must turn it red.
- **The alpha rule with a tolerance.** Rule 3 is exact at alpha 0. Say what happens with
  `color = "#00000000"`, `tolerance = 8` and a border pixel of alpha 3 with a hidden colour far from
  black: does it match? State the rule so a test can check it, and make it a row.
- **The whole-match message.** The frame's shape prints the colour as `#rrggbb`. A 16-bit or float
  top-left pixel can't be printed that way without rounding. Pick a wording that is true at every
  depth (for example name "the top-left pixel" without a hex value, or print the samples), and say
  why.
- **`ParamKind::UInt` has no `max`.** Adding one changes a public enum that the registry publishes
  (find every consumer: `param_specs`, the wasm schema export, ddphotos, `validate()`), and three
  existing rows get `max: None`. A constructor check avoids that but loses the table's standard
  message. Pick one, cite what you found, and give the exact message.

## Design calls that are yours
- **The exact refusal messages.** Give each row of the frame's refusal table its exact text, in the
  `recipe step N ('trim'): …` shape, citing `file:line` for each message you match (`pad`'s colour
  message, the registry's unknown-key, wrong-type and range lines). One AC row and one test case per
  refusal, including the trap 10 rows (`colour`, `tolerence`, `16777217`). No message names a spec,
  stage or DEC id.
- **The apply failure:** exact message, exit 1 on one input and exit 6 in a batch with nothing written
  for the failing input. Drive the batch shape on the binary through a stand-in (crop's 0 px case,
  as SPEC-151's design did) and make each a test.
- **The match rule as a table a test can check:** for each of the 10 colour types, the default and a
  stated colour, at `t = 0` and at the edge (`t` and `t + 1` off), with the 16-bit row at 2056 and
  2057, the float rows including `c = 0, t = 1, s = f32(1/255)`, grey against a grey and a chromatic
  colour, and the alpha rows.
- **The bounding-box scan:** each side pinned by a one-step-outside pixel on that side only, borders
  of different widths per side, one side only, a one-pixel-wide survivor, odd sizes, and 1×1.
- **Exactness:** the output equals the input's sub-rectangle at the computed box, sample for sample,
  at every colour type. The expected box comes from literals in the test, never from the code under
  test. Seed buffers independently.
- **Metadata:** one row, as `pad` has (`trim_writes_no_orientation_tag_and_keeps_the_bundle`), with
  its control.
- **The corpus rows.** Recount the literals yourself (`RECIPES`, `EXPECTED_DIMS`, `ROWS`,
  `GOLDEN_LINES`, the pixel-row count, `pairs.len()`) and write them into an AC. No existing
  `.output` or `.golden` line may change: an AC requires
  `git diff --diff-filter=MD origin/main -- tests/fixtures/recipe_corpus/` to be empty.
- **The wasm rows.** `tests/wasm_roundtrip.rs`: a trim transform equal to the expected sub-rectangle,
  and a load refusal plus the whole-match failure with the native message. ⚠ Every wasm test changes
  `scripts/wasm-ci-check.sh`: `FULL_COUNT`, `SLIM_COUNT` and the `--self-test` fixtures that name
  them. Re-read both counts on `main` after your last rebase (53/51 when this was written), write the
  new literals into an AC, and require `--self-test` and `workflow` to pass.
- **The fold-in.** The exact `contains` assertions for resize's two cap lines, in the three tests
  the entry names (`src/operation/mod.rs` about :2351 and :3565, `tests/apply_batch.rs` about :461).
  Quote both lines from the binary.
- **Docs.** Re-check each surface in the frame's list at its `file:line`, find any it missed, and
  say what each line will say. The `trim` entry carries the determinism sentence, the top-left
  default (with "put `auto-orient` first"), the tolerance scale at each depth, the alpha rule, and
  that ImageMagick's `-fuzz` is a different measure. Each is an AC verify checks by reading.

## Tests: each one must be able to fail
HANDOFF traps 12, 13, 14 and 17 and AGENTS §15 bind here. **Drive the mutation rows at design on a
prototype; don't reason them.** SPEC-151 did, and its build found no wrong cell; its one reasoned row
was the one verify refuted. Each row names the code location, every test expected red and every test
expected green, and is marked *driven* or *reasoned*. **A row marked "expected all green" names the
search that found no input where the mutant and the code differ** (AGENTS §15). At minimum, each with
its splitting input:
- **`× 257` → `<< 8`** for the 16-bit tolerance: a 16-bit sample 2056 off. The corpus
  `v2_trim_tolerance` row on `rgba16` goes red by pixel hash or dims; say which.
- **`<=` → `<`**: any sample exactly `t` off.
- **The float rule replaced by `f64` `/ 255` maths**: `c = 0`, `t = 1`, `f32(1/255)`.
- **The alpha-0 rule removed**: the `a0` shape (alpha-0 border, mixed hidden colours).
- **Top-left read as another corner**: the `tlodd` shape.
- **One side's scan dropped** (one row per side): the one-step-outside rows.
- **The whole-match check removed**: a 1×1 image; say what is written instead.
- **Grey widened wrong** (a luma sample compared as `(L, 0, 0)`): grey against a chromatic colour.
- **wasm count** one lower: the count check and the self-test go red.
- **Corpus guard**: add a `trim` step to an existing corpus recipe on a scratch copy; the guard goes
  red, naming that recipe.
- **The fold-in**: change one character of resize's cap message; the three tests go red.
- **Trap 14**: restore `main`'s `with_builtins` verbatim (the whole function, not one deleted line)
  and list everything that goes red.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and Implementation
Context. You don't write the code. Separate build and verify sessions follow: one builds the spec
from your text alone, and another verifies it, so everything the builder needs must be in the spec
(AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, and §15, including its
   two new verify rules ("A bound or 'can't happen' is a claim until its worst input is driven" and
   "Classify every finding"). §16 habit 6 (no CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
   (traps 10, 12, 13, 14 and 17 especially). PROJ-017 has no HANDOFF of its own, so these bind.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`.
4. `/guidance/constraints.yaml`, DEC-095, DEC-102, DEC-110, DEC-112, DEC-113.
5. **SPEC-151 in `specs/done/`, as the shape to match**: its Design section, ACs, exact messages,
   colour-type table, mutation table, corpus and wasm rows, Build Completion, the punch list and the
   ship reflection. `trim` should read as `pad`'s mirror.
6. SPEC-150's spec (PROJ-011), for the shared files above.
7. `tests/fixtures/recipe_corpus/README.md` and `scripts/wasm-ci-check.sh`.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `pad.rs` (its table, colour
  parse, per-variant handling, `wide`/`unit`, the no-op), `crop.rs` (`crop_imm`, the 0 px apply
  failure), `parse_color`, the registry's `ParamKind::UInt` and its consumers, `with_pixels`, and the
  wasm entry point.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- **Counts in the spec come from a command, not memory.** SPEC-151's spec had two wrong counts (files
  changed, callers of a helper). Cite the command next to each count you write.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, and no change to `CACHE_SCHEMA_VERSION` or the lockfile format. Say in an AC why
  a new op needs neither.
- Sizing: if the design comes out bigger than [S], say so and why in your report.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report, rather than silently widening or
narrowing it.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-152 build`.
2. In STAGE-058's backlog, change SPEC-152's marker from `(framed 2026-11-06)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add
   `prompts/SPEC-152-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-152): design trim as a recipe step`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back `git show --stat HEAD`. ⚠ `just backlog` reads only the active project (PROJ-011),
   so use `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC. No pushed probe
  branch. No version bump, no tag.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
  zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep`
  and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session (SPEC-150's build may be running). Driving
  mutations on a prototype rebuilds the crate each time; remove the prototype worktree and its
  target dir before you finish.
- **Budget ~55 exchanges** (the driven mutation rows are the extra).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-152 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat rate:
Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read is
×0.10. Measure at your last turn, after the push, and append a `design` entry to the spec's
`cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-152 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- any correction to the frame (the float rule, NaN, the inverse, the corpus, the alpha-with-tolerance
  case, the whole-match message), with what you drove;
- the `UInt` max decision and what consumes `ParamKind`;
- the AC list (one line each);
- the Failing Tests and the mutation table, with the rows expected red and green, each marked driven
  or reasoned, and the search behind any "expected all green" row;
- the match-rule table;
- the corpus rows, the new literals, and the new `FULL_COUNT`/`SLIM_COUNT`;
- the docs surfaces, each with `file:line`;
- the files shared with SPEC-150, and which hunks are whose;
- size, and anything the build prompt must carry (checkpoints, CI legs, merge order with SPEC-150);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
