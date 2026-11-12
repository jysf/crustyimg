# SPEC-160 — FRAME prompt: `edit --pad` and `--trim`, and the `parse_color` sign fix

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-160-frame -- --model opus
```

**Reserved: SPEC-160.** PROJ-011's STAGE-063 reserves SPEC-154 to SPEC-158; SPEC-159 shipped on
2026-11-12. Before you allocate, confirm SPEC-160 is unused as a spec id:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-160' "$b" -- . ; done
```

The expected hits are this prompt, STAGE-058, DEC-115, and SPEC-159's spec and prompts (which
name SPEC-160 as the next spec). If a spec file of its own or another reservation uses SPEC-160,
stop and report. Don't pick another number. ⚠ `just new-spec` skips any id named in prose, so it
will likely offer SPEC-161 or later; take SPEC-160 anyway (see "Allocate the id").

Your item is the STAGE-058 Spec Backlog entry "**`edit --pad` and `--trim`, on SPEC-153's ordering
rule.**" Read it in full, with its ruling and the stale-wording paragraph. Two more things are ruled
into it:
- the entry "**kind: claim. `parse_color` accepts a sign, so `+f+f+f` is read as `#0f0f0f`.**"
  (ruled 2026-11-10: folds in);
- the stale "single image" wording on three sites (added at SPEC-153's ship).

Other entries bear on it and are named below; touch no item except as this prompt says.

## Why this spec exists
STAGE-058 promises five lossless geometry ops as recipe steps **and** as `edit` flags. SPEC-153 gave
`edit` `--crop`, `--rotate` and `--flip` under the typed-order rule (DEC-114). `pad` (SPEC-151, with
SPEC-159's transparent fill) and `trim` (SPEC-152) exist as recipe steps but have no flag. **This
spec closes STAGE-058.**

## Settled; don't re-open
- **Typed order** (DEC-114): op flags run in the order typed, `auto-orient` first and positionless,
  each op flag at most once (exit 2), modifier flags positionless. `--pad` and `--trim` join it.
- **`--trim` is a plain switch**, with optional `--trim-color` and `--trim-tolerance`, written to the
  recipe only when given. Not `--trim[=COLOR]`: clap then swallows the input path (driven at
  SPEC-153's frame).
- **`--pad-color` is required**; there is no CLI-only default colour.
- **The alpha rule** (DEC-115): a fill alpha the output format can't store is refused before the
  pipeline runs. DEC-115 part 4 says `run_pixel_op` checks only a **forced** format today, and that
  the per-input check for an unforced format **arrives with this spec**, the first caller that can
  reach it.
- **The `parse_color` sign fix folds in** (proposed fix in the entry: refuse any byte that isn't an
  ASCII hex digit, in `parse_color` itself).

The frame's job is everything those leave open.

## What already exists (read it, don't assume it)
- **`edit`** (`src/cli/mod.rs`'s `Edit` args, `src/cli/ops.rs`'s `edit` path, `edit_op_order` and
  `build_edit_ops`, wherever they live now; grep): how `--crop`'s pixel form is converted to
  fractions of the buffer the step receives, how a pixel crop with `--save-recipe` on a batch is
  refused (DEC-114 part 6, P16), and the parse-time messages that name the flag (DEC-106).
- **SPEC-153's Frame** (`specs/done/SPEC-153-…md`, "Each flag's syntax, and what `--save-recipe`
  writes" and "Where each failure happens"). It recorded the split-off half's syntax: `--pad N|W:H`
  (`N` whole pixels ≥ 1 of the buffer's long edge, written as `border = N/long`), optional
  `--pad-gravity` for the aspect form only. It also drove a `conflicts_with` between
  `--pad-aspect` and `--pad-border`. Say which shape you recommend: one `--pad` taking either form,
  or two flags, and why.
- **`pad`** (`src/operation/pad.rs`): the `border` and `aspect` forms, `round(border·long)`, the size
  caps, `parse_fill_color` (`transparent` or hex) and `introduced_alpha`.
- **`trim`** (`src/operation/trim.rs`): the corner-pixel default, `color`, `tolerance` (8-bit steps,
  scaled at 16 bits), and the whole-match failure. Whether that failure goes through the refusal hook
  (skipped per image in a batch) or fails at apply; read SPEC-152's design.
- **`run_pixel_op`** (`src/cli/ops.rs`): the forced-format check at the top, where the format is
  resolved for an unforced input (after the pipeline runs), and SPEC-159's mutation M18 (the
  per-input check that wasn't reachable then).
- **`parse_color`** (`src/text/mod.rs`) and every caller: `pad`, `trim`, `watermark` (recipe and the
  `watermark --color` flag), `parse_fill_color`, and the two new flags. SPEC-152's explicit non-ASCII
  check, which the fix may make redundant.
- **The three "single image" sites** quoted in the entry: `src/cli/mod.rs` (the `edit` `--help` doc
  comment), `docs/api-contract.md`, `docs/cli-reference.md`. Line numbers will have moved; find them by
  content.

## Questions this frame must answer
1. **Syntax.** One `--pad` or two flags; `--pad-color` taking `transparent` and `#RRGGBBAA`;
   `--pad-gravity` only with the aspect form (refused with the border form, exit 2, the way
   `--crop-gravity` is with the pixel crop). `--trim-tolerance`'s range and type. Every parse-time
   refusal and its message, naming the flag, not the key (DEC-106).
2. **Pixels to a fraction, exactly.** `--pad N` writes `border = N/long` of the buffer the step
   receives. Is `round((N/long)·long) == N` for every `N` and every `long` up to the 50,000 px cap, in
   `f64`? **Drive it** over the full range (AGENTS §15: a bound is a claim until its worst input is
   driven). Say what happens when `--pad` follows a `--resize-max` or `--rotate 90` (the long edge of
   the buffer the step receives, not of the input).
3. **`--save-recipe` on a batch.** A saved `border` fraction gives a different pixel count on each
   image size. Should `--pad N` with `--save-recipe` on a batch be refused like a pixel `--crop`
   (DEC-114 part 6), or is a fraction of the long edge the intended meaning? Recommend, with the
   aspect form's answer too. Say whether DEC-114 needs amending, or a new DEC (next free id
   **DEC-116**), or neither.
4. **The alpha rule on `edit`.** `edit in.jpg --pad 10 --pad-color transparent` keeps the JPEG
   format. **Drive it on `main` today** (it can't run: `--pad` doesn't exist; drive the nearest thing,
   or reason from `run_pixel_op` and say so). Recommend where the per-input check for an unforced
   format goes, so that one input exits 4 with nothing written and a batch that keeps formats fails
   that input with exit 6, matching `apply`. Say whether the check can run before the pipeline (the
   format is known from the input) or only after, and why DEC-115 part 4's wording then needs a
   follow-up edit.
5. **`trim`'s whole-match message.** It quotes `'color'`/`'tolerance'`. Recommend how `edit` words it
   with `--trim-color`/`--trim-tolerance` (and with no `--trim-color`, the corner pixel). Then say what
   a whole match does in an `edit` batch: skipped per image, or that input fails with exit 6? Read the
   STAGE-058 entry "**The batch summary says 'watermark skipped' for any refusing op**": if `trim`
   refuses through the hook, this spec is the first to make that message wrong on `edit`. Recommend
   fold or leave.
6. **`parse_color`.** Drive `+f+f+f` on `main` through every route (`pad`, `trim`, `watermark` recipe,
   `watermark --color`). Recommend the fix's exact rule and say whether the non-ASCII check goes.
   **This changes behaviour:** a recipe or flag that loads today will be refused. Name the exit code
   and message on each route, whether the CHANGELOG needs a "Changed" line, and whether any bundled
   recipe, doc example or golden uses a sign (grep).
7. **Typed order, driven.** `--pad 10 --pad-color red --rotate 90` against `--rotate 90 --pad …` on a
   non-square input, and `--trim` before and after `--pad`. Say what the saved recipe holds and that
   `apply` replays it to the same pixels: the test SPEC-153's frame designed ("`--save-recipe` replays
   exactly"), extended to `pad` and `trim`, including 16-bit and grey sources and a transparent fill.
8. **The stale wording.** Quote each site as it reads on `main` today, and what it says after
   (`edit` takes one image or a batch; a pixel `--crop` or `--pad N` with `--save-recipe` needs one
   input, if question 3 says so).
9. **wasm.** `edit` isn't in wasm. The `parse_color` fix is, through `transform`'s recipes. Say which
   wasm rows or counts change, if any.
10. **Closing the stage.** This spec closes STAGE-058. Besides your item and the `parse_color` entry
    it folds, the stage has five open entries today (re-read them all and recount: the recipe-output
    guard report, `crop`'s orientation match, the "watermark skipped" message, the `auto-orient`
    warning on tagged inputs, and `watermark`'s nine gravities with a margin). For each, recommend one: **fold** into this spec (only if it doesn't
    change the size), **leave** in STAGE-058 (and say whether the stage can still ship), or **move**
    to another stage or `guidance/ideas.yaml`, with where. Don't move anything; recommend. This is a
    maintainer question.
11. **Docs.** Every place that lists `edit`'s flags or describes `pad`/`trim` from the CLI side, and
    what each says after: `--help`, `docs/cli-reference.md`, `docs/api-contract.md`,
    `docs/recipes.md`, `recipes/README.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`. Per
    SPEC-159's ship reflection, list each behaviour sentence the build must **drive on every exit
    path it names**, not only the first.
12. **Size.** The backlog says [S]. Two flags with five option flags, a new per-input check, a
    behaviour change in `parse_color`, three wording sites and the replay matrix may make it [M]. If
    it is, say what you would cut to keep it [S], and recommend.

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
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §13 git, §15 (its verify rules bind every cycle:
   "A bound or 'can't happen' is a claim until its worst input is driven" and "Classify every
   finding") and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order and "Explicitly
   out of scope".
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item, every open entry, the `delivers` lines, the exit criteria and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
   (traps 5, 10, 12, 13, 14 and 17). PROJ-017 has no HANDOFF of its own, so these bind here. The file
   exists on `main`; if it isn't in your checkout, say so.
5. DEC-095, DEC-106, DEC-110, DEC-112, DEC-114, DEC-115.
6. SPEC-151, SPEC-152, SPEC-153 and SPEC-159 in `projects/PROJ-017-…/specs/done/`, with their ship
   reflections. Apply their proposals in your stub where they arise: a mutation row quotes its exact
   mutant edit, a count carries its derivation, build drives each doc sentence on every exit path,
   and design re-reads its DEC against its own corrections.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code.** On a release build of `main` (fresh `CARGO_TARGET_DIR`,
  deleted at the end): `edit in.png --pad 10 --pad-color red` and `edit in.png --trim` (record clap's
  refusal and exit code), and the `+f+f+f` drives in question 6.
- **Drive the bound in question 2**, and the clap behaviour of the syntax you recommend (repeats,
  conflicts, `requires`, a negative or zero `N`, `--trim` before the input path).
- **Size the verification, not just the code** (AGENTS §2): flags × forms × source colour types
  (8/16-bit, grey, grey+alpha, RGB, RGBA) × typed positions × one input vs a batch ×
  `--save-recipe` replay × output formats for the alpha rule × `parse_color` routes.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), and any fold-in's files.
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Allocate the id
Run `just new-spec "<title>" STAGE-058 PROJ-017`. If it gives anything other than SPEC-160,
`git mv` the spec and its timeline to SPEC-160 and fix the id inside both. Then run
`just advance-cycle SPEC-160 frame`. If you recommend a DEC, it takes the next free id (DEC-116 on
`main` today); recommend its content, don't write it.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-095, DEC-106, DEC-112, DEC-114, DEC-115, SPEC-151, SPEC-152, SPEC-153, SPEC-159), and
  `cycle: frame`.
- `## Context` and `## Goal`: what a CLI user can't do today, and what they can when this ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the syntax you recommend, with every parse-time refusal;
  - the pixel → fraction bound, driven, and the batch `--save-recipe` rule;
  - the alpha rule on `edit` and where its check lives;
  - `trim`'s whole-match wording and batch behaviour;
  - the `parse_color` fix, its routes and what changes for a user;
  - where each failure happens (parse, load, before the pipeline, apply) and its exit code;
  - the stage-closing table: each open STAGE-058 entry and your recommendation;
  - sizing and why;
  - scope in/out, including fold-ins;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-160-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-160 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-160** (framed <date>)`. Mark the `parse_color` sign entry
   `- [x] (folded into SPEC-160)` the way SPEC-152's folded entries were. Fold any other entry only if
   you recommend it **and** it doesn't change the size; otherwise leave it and put it to the
   maintainer. On a split, file the second half as a new `- [ ] (not yet written)` entry right after
   yours. File any out-of-spec finding as a new `- [ ] (not yet written)` entry at the end of the
   stage it belongs to. Update the `**Count:**` line with greps you've just run, and show them.
   Something with no stage goes in `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-160): frame edit --pad and --trim`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-058 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag, no CHANGELOG edit. A probe you drive at frame
  lives in a scratch directory and is never committed.
- Don't move or close any other STAGE-058 entry (question 10 is a recommendation), and don't change
  STAGE-058's status.
- Other sessions may be working at the same time. Never share a `CARGO_TARGET_DIR`. If
  `git rebase origin/main` conflicts in a file outside PROJ-017, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS has
  no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~`, `/tmp` or the system temp dir.
- **Budget ~45 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-160 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

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
Plain lists only. First line: `SPEC-160 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- the syntax you recommend, and every parse-time refusal;
- the pixel → fraction bound: driven over what range, and the result;
- the batch `--save-recipe` rule for `--pad N`, and any DEC content;
- the alpha rule on `edit`: where the per-input check lives, exit codes;
- `trim`'s whole-match wording and batch behaviour;
- the `parse_color` fix: the `+f+f+f` drives on each route, the new behaviour, and the CHANGELOG line;
- the three stale-wording sites, quoted before and after;
- whether any existing output changes, and what pins it;
- the stage-closing recommendations, one line per open entry;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
