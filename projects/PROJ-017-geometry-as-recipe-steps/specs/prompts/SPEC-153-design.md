# SPEC-153 — DESIGN prompt: `edit` flags for crop, rotate and flip, and the ordering rule

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-153-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`ff3cb2e`, cost in `3724b08`, rulings
recorded after). **SPEC-153 writes DEC-114**, the ordering rule. Confirm DEC-114 is still free
(`ls decisions/ | /usr/bin/grep DEC-114`, and a `git grep DEC-114` across every branch); if anything
else uses it, stop and report rather than pick another number.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise check and the order problem, driven (60×40: crop→rotate 40×30, rotate→crop 20×60);
- the ordering rule's seven parts (DEC-114's content) and Alternatives A–C;
- each flag's syntax and what `--save-recipe` writes, and what clap 4.6.1 did;
- the crop pixel round trip, driven to 50,000 px;
- where each failure happens and its exit code;
- the replay matrix and six mutation rows, each with its edit;
- sizing, scope, fold-ins (none), the docs list and the file list.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer ruled on 2026-11-10, accepting all five of the frame's recommendations (recorded in
the stub under `### Maintainer rulings`):
1. **Typed order.** Every op flag (`--resize-max`, `--invert`, `--crop`, `--rotate`, `--flip`) runs in
   the order typed. `auto-orient` is always first and positionless; `--auto-orient` stays an accepted
   no-op wherever it's typed. Each op flag appears at most once. Modifier flags (`--crop-gravity`)
   have no position and `require` their op.
2. **`edit --invert --resize-max N` may change its output.** CHANGELOG "Changed" line.
3. **`--crop` takes `WxH+X+Y` (pixels) and `W:H` (aspect, optional `--crop-gravity`) only.** No
   fraction or percent form on the flag.
4. **A pixel crop past the edge of the image the step receives is exit 1**, with `edit`'s own message
   naming the flag and that image's size.
5. **Split as filed.** This spec: DEC-114, `--crop`, `--rotate`, `--flip`, the typed-order machinery and
   the replay test. `--pad`/`--trim` is STAGE-058's next entry, framed separately; don't design it,
   but write DEC-114 so it covers them without amendment.

Also accepted from the frame:
- **A pixel length means pixels of the image as that step receives it,** converted to a fraction when
  the step runs and written at full `f64` precision (never a fixed number of decimals).
- **`--save-recipe` writes what ran**, so it's captured after the run.
- **`--flip horizontal|vertical`**, no `h`/`v`. **`--rotate`** takes what the op's key table takes.
- **No fold-ins.**

## Another orchestrator may be building in nearby files
PROJ-011's STAGE-063 reserved SPEC-154 to SPEC-158 (an external code review, PR #260). SPEC-154 is
CI-only, but SPEC-155 to SPEC-158 change `src/sink/`, `src/metadata/`, `src/cli/build.rs`, and
possibly `src/cli/common.rs`, `src/cli/ops.rs` and `CHANGELOG.md`. At your Step 0, run
`gh pr list --state open --json number,title,files` and list every file another open PR or pushed
`feat/spec-15[4-8]*` branch touches that this spec also touches. Write that map into Implementation
Context **from the PRs' file lists, not from their specs' text** (SPEC-152's design map, written from
the other spec's text, missed a file). Tell the build to rebase onto `main` before each push and to
re-check the map after each rebase.

## Things in the frame to check before you build on them
SPEC-152's design corrected eight things in a frame whose Python model was right. Drive these:
- **The crop round trip ran in what?** The frame swept 2.68e9 rectangles. Say which code it ran
  (Python, a Rust model, or the crate). Then re-drive a sample **through the crate's real path**: the
  converter you design → the `toml` writer the CLI uses → `Recipe` load (the DEC-112 `x + width ≤ 1`
  check) → `crop`'s own edge rounding. At minimum: the frame's (640, 9, 631) case, n = 1..512
  exhaustive, edge-touching to 50,000 on a stride, and the 6-decimal mutant as the control that
  turns red. Quote the counts.
- **Rule 2's replay claim.** The frame says letting `--auto-orient` take its typed position would replay
  a mirrored tag (2, 4, 5, 7) differently. Drive it on the binary with an Orientation-5 or 7 input and
  `[rotate, auto-orient]` against `[auto-orient, rotate]`, and quote the pixel difference. That is
  M6's splitting input.
- **The `value_source` filter (M2).** Drive which flags in `Edit` get an index from `indices_of` when
  not typed (the frame saw `auto_orient=false` at index 8). If no flag that the order reads can get a
  default index, M2 is equivalent: say so with the search that showed it, or find the row that reds.
- **Indices for valued flags.** Drive `--crop=…` against `--crop …`, and a flag typed after the input
  path against before it, so the order collection is pinned for each spelling.
- **`--save-recipe` today on a failed run.** Drive whether `main` writes the recipe file when the run
  fails (for example `--resize-max 0`, or an unwritable output). Under capture-after-run a failed run
  should write no recipe; state the rule and make it a row. Say whether that's a change from today.
- **Negative angles.** `--rotate -90` needs `allow_negative_numbers`. Drive what it does to
  `edit -- …`-style inputs and to an input path that starts with `-`, and decide.
- **The byte pin.** The frame says nothing pins `edit`'s output today. Record the decoded-pixel
  hashes and `--save-recipe` text for `--resize-max N`, `--invert`, `--resize-max N --invert` and
  `--auto-orient --resize-max N` on a noise fixture **from a release build of `main` now**, write
  them into the spec as literals, and say how the fixture is seeded (independently of the code under
  test). Build asserts those literals before and after its change. Also record `--invert
  --resize-max N` on `main`: that is the "Changed" row, and the test must show it now differs.

## Design calls that are yours
- **DEC-114.** Write it in `decisions/` from `_template.md`: the rule's parts, why typed order (and
  that Alternative A was weighed and declined on 2026-11-10), what it changes, how it relates to
  DEC-005 (which never stated an order), DEC-112 (no pixel unit enters a recipe) and DEC-113. Word it
  so the split-off `--pad`/`--trim` needs no amendment.
- **The exact messages.** Every parse refusal (exit 2, before any file is read, DEC-106) and both
  apply failures (exit 1), each verbatim, each naming the flag, not the recipe key. Cite `file:line`
  for each message you reuse (`parse_aspect`, `ROTATE_PARAMS`/`Rotate::from_params`, `FLIP_PARAMS`,
  `GRAVITY_NAMES`, clap's own lines). One AC row and one test per refusal, including trap 10 rows
  (`--crop 10X10+0+0`, `--crop 10x10`, `--crop 10×10+0+0`, `--crop 0x10+0+0`, `--crop -1x10+0+0`,
  `--flip h`, `--rotate 45`, a repeated `--rotate`, `--crop-gravity` alone and with the pixel form).
  No message names a spec, stage or DEC id.
- **The past-the-edge message** for a pixel crop after a resize or rotate: it names the size of the
  image that step receives, not the decoded size. Make that a row.
- **The replay test** as the frame's matrix, with each row asserting decoded pixels, colour type,
  dimensions, and the saved step list **in order**. Required: a crop after `--resize-max` and after
  `--rotate 90` on a non-square input, a rectangle where `X/n + W/n ≠ (X+W)/n` bitwise with
  `X + W = n`, 8-bit RGB, RGBA, grey and 16-bit inputs, and the tagged Orientation-6 row.
- **`edit_flag_order_independent`** is rewritten on a fixture where the orders differ, asserting they
  do. Name the new test.
- **Docs.** Re-check each surface in the frame's list at its `file:line`, find any it missed, and say
  what each line will say: `crustyimg edit --help`, `docs/cli-reference.md`, `docs/api-contract.md`,
  `docs/recipes.md`, `skills/crustyimg/SKILL.md` (check), `CHANGELOG.md` ("Added" and "Changed").
  Each is an AC verify checks by reading.
- **Size.** The frame says [M], ~55 rows. Say whether that holds after design, with the count's
  derivation written next to it.

## Tests: each one must be able to fail
HANDOFF traps 10, 12, 13, 14 and 17 and AGENTS §15 bind here. **Drive every mutation row at design on
a prototype; don't reason them.** Each row **quotes its exact mutant edit** (SPEC-152's build drove a
different mutant from the spec's, and the spec's row went unconfirmed until verify), names the code
location, every test expected red and every test expected green, and is marked *driven* or
*reasoned*. A row marked "expected all green" names the search that found no splitting input. Start
from the frame's M1–M6 and add at least:
- **Ties and spelling:** the order taken from flag declaration order instead of the typed index.
- **The pixel converter rounding** (`f32` division, or `X / W` with W from the wrong axis).
- **The past-the-edge check** removed: what does the op do instead, and which row reds?
- **The modifier** `--crop-gravity` dropped from the saved recipe when given.
- **Trap 14:** restore `main`'s `build_edit_ops` verbatim and list everything that goes red.

**Every count in the spec carries its derivation** (the command, or the sum), next to the number
(SPEC-152's AC-3 named the groups but not their sizes, and the build had to recover them by search).

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests, Implementation
Context and DEC-114. You don't write the code. Separate build and verify sessions follow: one builds
the spec from your text alone, and another verifies it, so everything the builder needs must be in
the spec (AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so. Then build the overlap map above.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 and §16 habit 6 (no
   CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   PROJ-017 has no HANDOFF of its own, so these bind.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`.
4. `/guidance/constraints.yaml`, DEC-005, DEC-102, DEC-106, DEC-112, DEC-113.
5. SPEC-152 and SPEC-151 in `specs/done/`, as the shape to match: Design, ACs, exact messages,
   mutation tables, Build Completion and both ship reflections.
6. SPEC-110 and SPEC-111 (`auto_orient_prefix`, the saved recipe's leading `auto-orient`), and
   SPEC-032 (the canonical order this spec supersedes).

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `Commands::Edit` and its doc in
  `src/cli/mod.rs`, `run_edit` and `build_edit_ops` in `src/cli/ops.rs`, `auto_orient_prefix`,
  `crop.rs` (`parse_aspect`, `rect_edges`, its key table), `rotate_flip.rs`' key tables, and the
  recipe writer `--save-recipe` uses.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, no change to `CACHE_SCHEMA_VERSION` or the lockfile format; say in an AC why none
  is needed. wasm: the frame found no change; confirm with a grep and say so in an AC.

### Ids
Don't run `just new-spec` (it allocated SPEC-154 for this spec at frame, because a reservation in
prose looked like a used id). Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-153 build`.
2. In STAGE-058's backlog, change SPEC-153's marker from `(framed 2026-11-10; SPLIT, this half [M])`
   to `(designed <date>, [M])` (or the size you found). Mark design `[x]` in the timeline with a
   2–4 line summary, and add `prompts/SPEC-153-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-153): design edit flags for crop, rotate and flip; DEC-114 typed order`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back `git show --stat HEAD`. `just backlog` reads only the active project (PROJ-011), so use
   `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline, the stage file and DEC-114. No pushed probe
  branch. No version bump, no tag. Never edit PROJ-011's files.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does
  not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Remove the prototype worktree and its target
  dir before you finish. Write scratch files in your session's scratchpad, never `/tmp`.
- **Budget ~60 exchanges** (driven mutation rows and the byte pin are the extra).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-153 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take those
three once per id and the **MAX** `output_tokens`. Include any `subagents/` transcripts.

Price each component at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read is ×0.10. Measure at your last turn, after the
push, and append a `design` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-153 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- DEC-114's file name and its rule in one line per part;
- any correction to the frame, with what you drove (the round trip through the crate, rule 2's
  replay, M2, valued-flag indices, a failed run's recipe, negative angles);
- the byte-pin literals and how the fixture is seeded;
- the AC list (one line each);
- the Failing Tests and the mutation table, each row with its quoted edit, red and green tests, and
  driven or reasoned;
- every exact message, with its exit code;
- the docs surfaces, each with `file:line`;
- the overlap map with other open PRs and branches, from their file lists;
- size with its derivation, and anything the build prompt must carry (checkpoints, CI legs);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
