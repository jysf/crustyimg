# SPEC-160 — DESIGN prompt: `edit --pad` and `--trim`, and the `parse_color` sign fix

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-160-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`07f2e79`, cost in `e1fdfea`, rulings
and the stage moves recorded after). **No new DEC** (the frame's finding: DEC-114 parts 1–6 already
cover `--pad`, `--trim` and the batch rule). If you find one is needed after all, the next free id
is DEC-116; stop and put it in your report as a maintainer question rather than write it.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the syntax, with every parse-time refusal (the clap rows driven in a scratch crate);
- the `--pad N` → `border = N/long` bound, driven over 1..=50,000 × 1..=50,000;
- the batch `--save-recipe` rule (refuse `--pad N` like a pixel `--crop`);
- the alpha rule on `edit`: two checks in `run_pixel_op`, both gated on `introduces_alpha()`;
- `trim`'s whole-match wording on `edit`, and that neither `pad` nor `trim` uses the refusal hook;
- the `parse_color` fix, driven on four routes, including the new `+f+f+f+f` alpha case;
- the replay matrix and the order rows that can fail;
- the docs list, the three "single image" sites, wasm, the file list.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer accepted all three of the frame's recommendations on 2026-11-11 (recorded in the stub
under `### Maintainer rulings`):
1. **Stage close:** this spec closes STAGE-058. Four entries moved off the stage (three to
   `guidance/ideas.yaml`, one to STAGE-060) and **`watermark`'s nine gravities with a margin is
   folded in** (`- [x] (folded into SPEC-160)` in STAGE-058; read that entry in full). It's
   test-only: one row per gravity with a non-zero margin, expected offsets as literals, and the
   mutations it names (the north/south swap in `Gravity::placement`, and a margin dropped).
2. **[M], uncut:** the `parse_color` fold and the 16-bit and grey replay rows stay.
3. **One `--pad N|W:H`**, matching `--crop`; `--pad-gravity` with the aspect form only.

Settled before that: typed order (DEC-114); `--trim` a plain switch with optional `--trim-color` and
`--trim-tolerance`, written only when given; `--pad-color` required, no CLI default; the alpha rule
(DEC-115); the `parse_color` fix folds in (refuse any byte that isn't an ASCII hex digit, in
`parse_color` itself).

## Another orchestrator may be building in nearby files
PROJ-011's STAGE-063 reserved SPEC-154 to SPEC-158. Open PRs touched `src/cli/ops.rs` (#240) and
`src/cli/mod.rs` (#226) at SPEC-159's build. At your Step 0, run
`gh pr list --state open --json number,title,files` and list every file another open PR or pushed
`feat/spec-15[4-8]*` branch touches that this spec also touches. Write that map into Implementation
Context **from the PRs' file lists, not from their specs' text**. Tell the build to rebase onto
`main` before each push and to re-check the map after each rebase.

## Things in the frame to check before you build on them
SPEC-159's design corrected eight things in a frame whose model was right. Drive these on a release
build of `main` and on your prototype:
- **The two gated alpha checks.** The frame says both can run before the pipeline because
  `output_format_for` never reads the processed image. Re-read every input it takes on the `edit`
  path. Then list what makes a format "batch-wide" on `edit`: `--format`, `-o`'s extension (one
  input only?), an `--out-dir` name template's literal extension (does `edit` take
  `--name-template`?), anything else. Drive each, and the kept-format batch, on the prototype.
- **Gating, driven.** The frame drove `edit in.png --crop 10x10+0+0 --format bogus` and
  `resize --exact 60000x10 --format bogus` (both exit 1 today) and says an ungated hoist would make
  them exit 4. Keep both as rows that pin "no existing exit code changes", and find any other verb
  through `run_pixel_op` whose exit code a mistake here could move.
- **`edit`'s alpha wording.** The frame says `UnstorableAlpha`'s `Display` would print
  `recipe step 1 ('pad'): 'color' = …` on `edit` (index 1 because of the `auto-orient` bake) and
  recommends a flag-named line. Confirm the index on the prototype, then pick where the `edit` line is
  built (a second `Display` mode on `UnstorableAlpha`, or a rewording at the call site) and pin it
  verbatim, JPEG and GIF.
- **The `--pad N` wrapper's `introduced_alpha`.** The frame names the mutant (the wrapper doesn't
  forward it; every check passes silently and JPEG gets a black border, exit 0). Drive it. Also check
  the wrapper forwards anything else `Pad` reports through the `Operation` trait (read the trait;
  list every default method and whether the wrapper must forward it).
- **`watermark --color`.** The frame quotes a different message for the flag
  (`invalid color '+f+f+f': expected RRGGBB, #RRGGBB, or RRGGBBAA`, exit 2). Find which function
  produces it and confirm it goes through `parse_color`, so one fix covers it. If it doesn't, say so.
- **`parse_color`'s callers.** List every caller (`/usr/bin/grep -rn "parse_color" src/`), including
  wasm and anything outside the four routes the frame drove, and say what each prints after the fix.
- **The order rows.** The frame found that a border pad and a quarter turn commute (byte-identical
  either way), so it moved the order rows to the aspect form and `trim`. Drive each order row on the
  prototype and show the two orders differ, as a negative control for the test.
- **The canvas cap in pixels.** The frame recommends the wrapper check the cap in pixels before
  conversion, naming `--pad` and the image size (DEC-114 part 5). Drive the worst case: an `N` that
  fits the cap on one side of a non-square image and not the other.

## Design calls that are yours
- **The code shape.** `EditOp::{Pad, Trim}`, `edit_op_order`'s table, the `--pad N` wrapper (like
  `PixelCrop`), the `--trim` wrapper that rewords the whole match, `run_edit`'s widened batch check,
  `run_pixel_op`'s two gated checks, and `parse_color`'s rule. Name each with its file and the
  `file:line` it changes.
- **Every message, verbatim,** with its exit code: every parse-time refusal (the frame's list plus
  any you add), the batch `--save-recipe` line, the `edit` alpha lines (JPEG, GIF; one input, batch
  pre-check, per input in a batch), the canvas-cap line, the `--trim` whole-match lines (with and
  without `--trim-color`), and the `parse_color` refusal on each route. Cite `file:line` for wording
  you reuse. One AC row and one test per message. No message names a spec, stage or DEC id.
- **The replay matrix.** Pick the final rows from the frame's 24 + 4 and say why each is there.
  Each row: `edit … --save-recipe r.toml -o a.png`, then `apply --recipe r.toml -o b.png` on the same
  input, `a == b` decoded pixel for pixel, plus the saved recipe's text (`border` at shortest-repr
  `f64`; `gravity`, `color`, `tolerance` absent when not typed). Expected values come from literals
  or an independent model, never from the code under test.
- **What doesn't change.** Every `edit` output with today's flags (SPEC-153's byte pins), every recipe
  output (SPEC-141's corpus guard), every `watermark` and `pad` output with an unsigned colour. Say
  what pins each; add a pin only where none exists.
- **`watermark`'s nine gravities.** Expected offsets as literals, computed by hand or by an
  independent model, for a mark on a non-square image with a non-zero margin.
- **wasm.** The `parse_color` fix reaches `transform`. Decide whether to add a wasm refusal row. If you
  do, the build must also update `scripts/wasm-ci-check.sh`'s counts and its self-test fixtures, and
  both go in the build file list (SPEC-159's build missed them; only CI's wasm leg caught it, since
  `just check` doesn't run `just wasm-test`).
- **Docs.** Re-check each surface in the frame's list at its `file:line`, find any it missed, and write
  what each will say. List **every behaviour sentence and every exit path it names**; the build drives
  each sentence on each path, and verify drives them again. SPEC-159's only product defect was a doc
  sentence driven on its exit-4 path and not its exit-6 path.
- **CHANGELOG.** The "Added" lines for `edit --pad` and `--trim`, and the "Changed" line for
  `parse_color`, verbatim.
- **DEC-115's ship-time wording.** Re-read DEC-115 against your design (SPEC-159's ship lesson: its
  DEC described a call site its own design had changed). Write the exact replacement text for part 4,
  the Neutral consequence and the SPEC-160 reference into Implementation Context, for ship to apply
  after the merge. Don't edit DEC-115 now: `main` must stay true until the code lands.
- **Cache.** Say whether `CACHE_SCHEMA_VERSION` or the lockfile format must change (a recipe that
  loaded with a signed colour now fails at load), with the reason.
- **Size.** The frame says [M]. Say whether that holds, with the count's derivation next to it.

## Tests: each one must be able to fail
HANDOFF traps 10, 12, 13, 14 and 17 and AGENTS §15 bind here. **Drive every mutation row at design on
a prototype; don't reason them.** Each row **quotes its exact mutant edit**, names the code location,
every test expected red and every test expected green, and is marked *driven* or *reasoned*. Read the
red set **off the prototype's run of the final test list**, not off the reasoning: SPEC-159's table
missed FT-2 in three rows because a `Display` test read the value the mutant changed. A row marked
"expected all green" names the search that found no splitting input. At least:
- **The wrapper:** `--pad N`'s wrapper doesn't forward `introduced_alpha`.
- **Each alpha check removed:** the batch-wide pre-check; the per-input check.
- **The gate removed:** both checks run for every pipeline (the `--format bogus` rows should red).
- **The batch `--save-recipe` check** not widened to `--pad N`.
- **`parse_color`:** `from_str_radix` back on the raw pair; the ASCII-hex test replaced by `is_ascii()`.
- **The `--trim` wrapper:** the whole-match message left in key form.
- **Typed order:** `pad` or `trim` missing from `edit_op_order`'s table (what does the step list do?).
- **The pixel → fraction step:** divide by the short edge instead of the long edge.
- **`watermark` gravities:** north placed as south; the margin dropped.
- **Trap 14:** restore `main`'s `src/cli/ops.rs` and `src/cli/mod.rs` verbatim and list everything that
  goes red (and whether the unit tests still compile).

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
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 and §16 habit 6 (no
   CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   PROJ-017 has no HANDOFF of its own, so these bind. The file exists on `main`.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md` (your entry, the folded
   `parse_color` and nine-gravities entries).
4. `/guidance/constraints.yaml`, DEC-095, DEC-102, DEC-106, DEC-112, DEC-114, DEC-115.
5. SPEC-151, SPEC-152, SPEC-153 and SPEC-159 in `specs/done/`, as the shape to match: Design, ACs,
   exact messages, mutation tables, Build Completion and all four ship reflections. SPEC-153 is the
   closest model (`edit` flags, typed order, the replay test).

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `Edit` in `src/cli/mod.rs`,
  `edit_op_order`, `EditFlags` and `build_edit_ops`, `PixelCrop`, `run_edit` and `run_pixel_op` in
  `src/cli/ops.rs`, `output_format_for`, `src/operation/pad.rs`, `src/operation/trim.rs`,
  `src/operation/alpha.rs`, `parse_color` and `parse_fill_color`, `Gravity::placement`, and the
  `watermark` flag path.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-160 build`.
2. In STAGE-058's backlog, change SPEC-160's marker from `(framed 2026-11-11)` to
   `(designed <date>)`, keeping `[M]` (or the size you found). Mark design `[x]` in the timeline with a
   2–4 line summary, and add `prompts/SPEC-160-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-160): design edit --pad and --trim, and parse_color's sign`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back `git show --stat HEAD`. `just backlog` reads only the active project (PROJ-011), so use
   `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC edits (DEC-115's new
  wording goes into the spec for ship). No pushed probe branch. No version bump, no tag. Never edit
  PROJ-011's files.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does
  not word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS
  has no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Remove the prototype worktree and its target
  dir before you finish. Write scratch files in your session's scratchpad, never `/tmp` or the system
  temp dir.
- **Budget ~65 exchanges** (driven mutation rows and the replay matrix are the extra).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-160 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take those
three once per id and the **MAX** `output_tokens`. Include any `subagents/` transcripts.

Price each component at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read is ×0.10. Measure at your last turn, after the
push, and append a `design` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-160 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- any correction to the frame, with what you drove (the two gated checks and what makes a format
  batch-wide, the gating rows, `edit`'s alpha wording and step index, the wrapper's trait methods,
  `watermark --color`'s parser, every `parse_color` caller, the order rows, the canvas cap);
- the code shape, one line per piece with `file:line`;
- the AC list (one line each);
- the Failing Tests and the mutation table, each row with its quoted edit, red and green tests, and
  driven or reasoned;
- every exact message, with its exit code;
- the replay matrix's rows, and what pins "nothing else changes";
- the docs surfaces with `file:line`, and the behaviour sentences with every exit path each names;
- the CHANGELOG lines, verbatim;
- DEC-115's ship-time replacement text;
- whether wasm gets a row (and so whether `scripts/wasm-ci-check.sh` is in the file list);
- the overlap map with other open PRs and branches, from their file lists;
- size with its derivation, and anything the build prompt must carry (checkpoints, CI legs);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
