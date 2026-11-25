# SPEC-163 — DESIGN prompt: an aspect that follows each image's orientation

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-163-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`5e47edc`, cost in `cff2a59`, rulings
recorded after). **No new DEC:** the ruling is an amendment to DEC-112 (rule 8). You write its exact
text into Implementation Context; ship applies it after the merge. Don't edit DEC-112 now: `main`
must stay true until the code lands.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- today's behaviour, driven (crop and pad on landscape, portrait, square, near-square and
  EXIF-rotated inputs, with and without `auto-orient`);
- the normalization rule: the aspect's larger side goes along the larger side of the buffer the step
  receives; strictly wider is landscape; a square takes the aspect as written;
- the refusals, with today's load errors quoted;
- the `edit` flags, the page port, the 26 parity rows and their negative control, the goldens that
  must not move, the docs sentences, the handed-in items' overlap, and the rough file list.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer accepted all four of the frame's recommendations on 2026-11-25 (recorded in the stub
under `### Maintainer rulings (2026-11-25)`):
1. **The key:** `orientation = "match"`, one value. Absent means off, and off has no other spelling.
   `--save-recipe` writes the key only when its flag was typed.
2. **A square image** takes the aspect as written. `1:1` with the key is accepted and changes
   nothing. Strictly wider is landscape, with no tolerance.
3. **`edit` flags in this spec:** `--crop-orientation match` and `--pad-orientation match`, built like
   the gravity flags, a usage error (exit 2) with a pixel `--crop` or a border `--pad`. [M].
4. **The DEC:** amend DEC-112 with rule 8, "Aspects can follow the image".

Settled before that: named presets stay out of the recipe format; forcing values
(`"landscape"`, `"portrait"`) are out; the key reads the buffer the step receives (DEC-113), never
the EXIF tag; DEC-112's rounding doesn't change; wasm needs no code change.

## Overlap with other open work
PROJ-011 is on hold, and no other spec is building today. At your Step 0, run
`gh pr list --state open --json number,title,files` and list every file another open PR touches
that this spec also touches. #237 (heic via `heifer`, parked) is expected; say whether it overlaps.
Write the map into Implementation Context **from the PRs' file lists, not their text**, and tell the
build to rebase onto `main` before each push and re-check the map after each rebase.

## Things in the frame to check before you build on them
Drive these on a release build of `main` and on your prototype:
- **One shared helper or two.** The frame recommends one normalization helper in
  `src/operation/mod.rs`. Read `crop.rs`'s `aspect_size` and `pad.rs`'s `aspect_canvas`, and say
  whether both call sites can take the same `(a, b, w, h) -> (a', b')` without changing either
  rounding. Drive it: with the key off, every existing crop and pad test passes unchanged on the
  prototype.
- **The key table.** The frame says a `OneOf(["match"])` row in each op's key table. Read SPEC-131's
  `param_specs` and the table type, and confirm the four refusals' messages come from the table
  (unknown value, wrong type, wrong case) and from the op (box form, border form). Quote each from
  the prototype, verbatim.
- **The box-form and border-form refusals.** The frame models them on today's `gravity` refusals.
  Find where those live (`file:line`) and confirm the key's refusal sits in the same place, so a
  recipe with both `gravity` and `orientation` on a box crop gets one deterministic message. Say
  which.
- **`edit`'s flags.** Read SPEC-153's and SPEC-160's flag code (`EditFlags`, `build_edit_ops`, the
  gravity flags, `--save-recipe`). Say whether `--crop-orientation` without `--crop`, or with a pixel
  `--crop`, is refused by clap (`requires`/`conflicts_with`) or by `build_edit_ops`, and drive each
  combination's exit code and message in a scratch build. The gravity flags are the model: match
  their behaviour exactly, or say why not.
- **Batch.** `edit <dir> --crop 3:2 --crop-orientation match` over a mixed folder: drive it on the
  prototype and show the portrait and landscape outputs. Then `--save-recipe` on a batch: does the
  aspect form already allow a batch `--save-recipe` (unlike a pixel `--crop`)? Confirm.
- **The near-square row.** The frame uses 1071×1072 as N. Re-derive both outputs by hand from
  `aspect_size` and `aspect_canvas`, and confirm the "on" row differs from the "off" row for both
  ops. A row whose two outputs match proves nothing.
- **The parity harness rows.** Read `tests/ddphotos_parity.mjs`: how the alpha rows are written by
  hand, how the self-test feeds a wrong text and requires red, and the row-count line. Confirm the
  26 rows fit that shape without moving the TABLE count.

## Design calls that are yours
- **The code shape.** The helper, the two op changes, the key-table rows, the two flags in `Edit`,
  `EditFlags` and `build_edit_ops`, and the recipe writer. Name each with its file and the
  `file:line` it changes.
- **Every message, verbatim,** with its exit code: the four load refusals on each op (eight), the
  flag refusals (each combination you drove), and any batch line. Cite `file:line` for wording you
  reuse. One AC row and one test per message. No message names a spec, stage or DEC id.
- **The replay rows.** For each op: `edit … --crop 3:2 --crop-orientation match --save-recipe r.toml
  -o a.png`, then `apply --recipe r.toml -o b.png` on the same input; `a == b` pixel for pixel, plus
  the saved recipe's text (`orientation` present only when typed). Landscape, portrait and square
  inputs at least. Expected sizes come from literals or an independent model, never from the code
  under test.
- **What doesn't change.** The frame's list: the 22 recipe-corpus goldens, `recipe_round_trip`, the
  parity harness's `GOLDEN_*` texts, TABLE and alpha rows, and every existing crop, pad and `edit`
  geometry expectation. Say what pins each, add a pin only where none exists, and state the build's
  proof (no `CRUSTYIMG_BLESS` run; `git diff --name-status origin/main -- tests/fixtures/` shows only
  `A` lines).
- **Fixtures.** The P, S and N parity fixtures are made from the existing landscape photo with
  ImageMagick, not crustyimg. Give the exact commands and the sizes.
- **Docs.** Re-check each surface in the frame's list at its `file:line` (`docs/recipes.md`,
  `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md`, `skills/crustyimg/SKILL.md`,
  `edit --help`), find any it missed, and write what each will say. List **every behaviour sentence
  and every exit path it names**, one input and batch; the build drives each sentence on each path,
  and verify drives them again.
- **CHANGELOG.** The "Added" lines for the key and the two flags, verbatim, under Unreleased.
- **DEC-112's rule 8.** The exact amendment text and the `affected_scope` additions, for ship to
  apply. Re-read DEC-112 in full against your design first (SPEC-159's ship lesson: its DEC described
  a call site its own design had changed).
- **The page port.** Write, for SPEC-161's design, the exact rule `recipe.js` must implement (integer
  comparisons only) and one worked example per orientation. Put it in Implementation Context under
  its own heading; SPEC-161's design will read it from there.
- **Cache.** Say whether `CACHE_SCHEMA_VERSION` or the lockfile format must change, with the reason.
  A new optional key that no existing recipe uses may change nothing; say so with the evidence.
- **wasm.** The key reaches `transform` through the same loader. Decide whether to add a wasm row
  (for example in `just wasm-test`). If you do, the build must also update `scripts/wasm-ci-check.sh`'s
  counts and self-test fixtures, and both go in the build file list.
- **Size.** The frame says [M]. Say whether that holds, with the count's derivation next to it.

## Tests: each one must be able to fail
HANDOFF traps 10, 12, 13, 14 and 17 and AGENTS §15 bind here. **Drive every mutation row at design on
a prototype; don't reason them.** Each row **quotes its exact mutant edit**, names the code location,
every test expected red and every test expected green, and is marked *driven* or *reasoned*. Read the
red set **off the prototype's run of the final test list**, not off the reasoning. A row marked
"expected all green" names the search that found no splitting input. At least:
- **The comparison:** `w > h` changed to `w >= h` (only the square 9:16 row can catch it; confirm).
- **The swap inverted:** portrait gets the aspect as written and landscape gets it turned.
- **The helper skipped on one op:** crop normalizes, pad doesn't (and the reverse).
- **The key ignored at load:** the table accepts it, the op never reads it (the negative control).
- **Each refusal removed:** box form, border form, unknown value.
- **`--save-recipe`:** writes `orientation` when the flag wasn't typed; drops it when it was.
- **Each flag refusal removed:** `--crop-orientation` with a pixel `--crop`; `--pad-orientation` with a
  border `--pad`.
- **Trap 14:** restore `main`'s `src/operation/` and `src/cli/` files verbatim and list everything
  that goes red (and whether the unit tests still compile).

**Every count in the spec carries its derivation** (the command, or the sum), next to the number.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and Implementation
Context. You don't write the code or any DEC. Separate build and verify sessions follow: one builds
the spec from your text alone, and another verifies it, so everything the builder needs must be in
the spec (AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so. Then build the overlap map above.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 (the coverage map in
   "During design", "a bound or 'can't happen' is a claim until its worst input is driven", the
   refusal rule "a refusal added needs a 'must not refuse' row") and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   PROJ-017 has no HANDOFF of its own, so these bind.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md` (your entry and SPEC-161's).
4. `/guidance/constraints.yaml`, DEC-102, DEC-110, DEC-112, DEC-113, DEC-114.
5. SPEC-145, SPEC-151, SPEC-153 and SPEC-160 in `specs/done/`, as the shape to match: Design, ACs,
   exact messages, mutation tables, Build Completion and the ship reflections. SPEC-153 and SPEC-160
   are the closest models (`edit` flags, `--save-recipe`, the replay test).
6. SPEC-161's `## Frame` and "Maintainer rulings", for what the page needs from this key.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `src/operation/crop.rs`
  (`aspect_size`, the gravity refusal), `src/operation/pad.rs` (`aspect_canvas`, its refusals),
  `src/operation/mod.rs`, the registry's key tables, `Edit` and `EditFlags` in `src/cli/mod.rs`,
  `build_edit_ops` and the recipe writer in `src/cli/ops.rs`.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- **Every refusal gets a "must not refuse" row** beside it: the key with the aspect form, on each op,
  on a square and a near-square, from a recipe and from `edit`.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-163 build`.
2. In STAGE-060's backlog, change SPEC-163's marker from `(framed 2026-11-25)` to
   `(designed <date>)`, keeping `[M]` (or the size you found). Mark design `[x]` in the timeline with
   a 2–4 line summary, and add `prompts/SPEC-163-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-163): design an aspect that follows each image's orientation`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. A "Changes
   must be made through a pull request" notice can still mean the push landed: check with
   `git ls-remote origin refs/heads/main`.
5. Read back `git show --stat HEAD`, and run `just status`: PROJ-017 is the active project.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC edits (DEC-112's rule 8
  goes into the spec for ship). No pushed probe branch. No version bump, no tag, no CHANGELOG edit.
  Never edit PROJ-011's files.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does
  not word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS
  has no `timeout(1)`. `rtk` can truncate `grep`, `ls` and `git log` output, so use `/usr/bin/grep`,
  `/bin/ls` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Remove the prototype worktree and its target
  dir before you finish, and say how much space that freed. Write scratch files in your session's
  scratchpad, never `/tmp` or the system temp dir.
- **Budget ~60 exchanges** (driven mutation rows and the replay rows are the extra).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content, not
recency:** `/usr/bin/grep -l "SPEC-163 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`. If the
prompt was given as a file path, the title may not be in your first message; then match the
prompt's path instead.

Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
`output_tokens`. Include any `subagents/` transcripts. Price at the model in `.message.model`: Opus
$5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
Measure at your last turn, after the push, and append a `design` entry to the spec's
`cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-163 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- any correction to the frame, with what you drove (the shared helper, the key table's messages, the
  refusals' placement, the flags' clap behaviour, batch, the near-square row, the harness shape);
- the code shape, one line per piece with `file:line`;
- the AC list (one line each);
- the Failing Tests and the mutation table, each row with its quoted edit, red and green tests, and
  driven or reasoned;
- every exact message, with its exit code;
- the replay rows, and what pins "nothing else changes";
- the fixtures and the commands that make them;
- the docs surfaces with `file:line`, and the behaviour sentences with every exit path each names;
- the CHANGELOG lines, verbatim;
- DEC-112's rule 8 text and `affected_scope` additions;
- the page port for SPEC-161, with worked examples;
- cache and wasm decisions (and so whether `scripts/wasm-ci-check.sh` is in the file list);
- the overlap map with other open PRs, from their file lists;
- size with its derivation, and anything the build prompt must carry (checkpoints, CI legs);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
