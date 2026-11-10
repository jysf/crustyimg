# SPEC-153 — FRAME prompt: `edit` flags for the five geometry ops, and the ordering rule

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-153-frame -- --model opus
```

**Reserved: SPEC-153.** `main` has SPEC-141 to SPEC-152 (SPEC-149 and SPEC-150 are PROJ-011's
STAGE-063), and SPEC-140 is left for PROJ-016 (DEC-107). Before you allocate, confirm SPEC-153 is
unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-153' "$b" -- . ; done
```

The expected hits are this prompt and PROJ-017's brief. If a spec file or another reservation
uses SPEC-153, stop and report. Don't pick another number.

Your item is the STAGE-058 Spec Backlog entry "**`edit` flags for the five, and the ordering
rule.**" Read it in full. It's build-order line 8 of 13, and the **last spec in STAGE-058**: the
stage closes when it ships. Other entries bear on it and are named below; touch no item except as
this prompt says.

## Why this spec exists
PROJ-017 makes geometry a recipe step, so the ddphotos page and the CLI share one saved recipe.
All five ops have shipped as recipe steps: `crop` (SPEC-145, DEC-112), `rotate` and `flip`
(SPEC-147, DEC-113), `pad` (SPEC-151) and `trim` (SPEC-152). STAGE-058 promises one more thing:
"`edit` flags for all five, with a written ordering rule, and `--save-recipe` that writes them".
The brief's success signal: each op "is reachable from `crustyimg edit`".

**The ordering rule is the hard part, and it is a DEC** (the brief's risk list). `edit` applies
its flags in a fixed canonical order today, whatever order they're typed in
(`auto-orient` → `resize` → `invert`; `src/cli/mod.rs`'s `Edit` doc and `build_edit_ops` in
`src/cli/ops.rs`). Geometry doesn't commute: crop-then-rotate is not rotate-then-crop, and
pad-then-trim is not trim-then-pad. Either geometry takes stated positions in the canonical order,
or `edit` honours the order the flags are typed in for geometry (or for everything). The frame
recommends one with evidence; design writes the DEC.

## What already exists (read it, don't assume it)
- **`edit`** (`src/cli/mod.rs` `Edit`, `src/cli/ops.rs` `build_edit_ops` and `run_edit`, and
  `tests/edit.rs`): its three flags, the canonical order, the "at least one op flag" rule, how it
  builds each op through the registry (DEC-005), and how `--save-recipe` writes `Recipe::from_ops`.
  Read the `edit_ops_canonical_order` tests.
- **DEC-005** (recipe TOML and the registry), which is where the canonical order comes from. Say
  whether the ordering DEC amends it or stands beside it.
- **DEC-106**: a CLI verb that builds its op directly checks its flags through the op's key table
  before any file is read. Say how each new flag meets it, and what exit code a bad flag gets
  (today's `edit` maps a bad param to exit 2).
- **DEC-112**: a recipe carries lengths as fractions of the image. The stage says pixels "come
  through `edit --crop`, which converts them". That conversion needs the image size, so it happens
  after decode, not at flag parse. Say where, and whether DEC-106's "before any file is read" can
  still hold for the parts that don't need the size.
- **SPEC-110**: `edit` bakes orientation first. Say whether geometry runs on the oriented pixels,
  and what `--save-recipe` records so `apply` replays the same thing on the same (tagged) input.
- **The five ops' key tables** (`src/operation/crop.rs`, `mod.rs` for rotate/flip, `pad.rs`,
  `trim.rs`): what each flag must map onto, including `crop`'s two forms, `pad`'s two forms and
  `trim`'s optional keys.
- **Clap**: whether it can report the order flags were typed in (`ArgMatches::indices_of`), and
  whether a flag can repeat (`--rotate 90 --crop … --rotate 90`). Drive it in a scratch crate, don't
  assume it.
- **The docs that describe `edit`**: `docs/cli-reference.md`, `docs/USAGE.md`, `docs/recipes.md`,
  `docs/api-contract.md`, `skills/crustyimg/SKILL.md`, the README, and `crustyimg edit --help`
  (`/usr/bin/grep -rn -- '--resize-max\|canonical order' docs/ skills/ README.md src/`).

## Questions this frame must answer
1. **The ordering rule — the main question.** Recommend one, with evidence from the commands you
   drove:
   - **Canonical positions:** where each of the five goes relative to `auto-orient`, `resize`,
     `invert` and each other. Say which orders a user can then not express, and whether that loses
     anything real (`--save-recipe` then hand-editing the TOML is the escape hatch).
   - **Typed order:** geometry (or everything) runs in the order typed. Say what happens to the
     existing three flags and their canonical order (a behaviour change for `edit --invert
     --auto-orient`? DEC-005 and `tests/edit.rs` pin today's order), and whether a flag may repeat.
   - Whatever you recommend, `auto-orient` stays first (SPEC-110). Say whether that's a rule or a
     special case.
2. **Each flag's syntax.** One line per flag, with the keys it maps to and what `--save-recipe`
   writes:
   - `--crop`: pixels (`WxH+X+Y`?), an aspect with a gravity, or fractions? The stage says pixels
     come through `edit` and become fractions. **Drive the round trip:** pixels → fraction (an
     `f64` written to TOML) → read back → pixels under DEC-112's rounding must give the same
     rectangle on the same image. Search for a size and rectangle where it doesn't, over a stated
     range, and name the input at the edge (AGENTS §15: a bound is a claim until its worst input is
     driven).
   - `--rotate 90|180|270` and `--flip horizontal|vertical` (short forms `h`/`v`?).
   - `--pad`: the aspect form, the border form, and `color` (required for `pad`). One flag with a
     mini-syntax, or `--pad-aspect`/`--pad-border` plus `--pad-color`? Say which reads better in
     `--help`, and what clap makes easy to refuse.
   - `--trim`: bare `--trim` for the defaults, plus `color` and `tolerance`. Same question.
   - Name the near-miss spellings and conflicts to refuse (`--crop` with both forms, `--pad-color`
     with no `--pad`), and say at what exit code.
3. **`--save-recipe` replays exactly.** The stage's test: steps that `apply` replays to the same
   pixels. Say what the test compares (decoded pixels, not file bytes, unless encode is
   deterministic), across which flag combinations and colour types, and on a tagged input.
4. **Refusals.** Bad flag values at parse (DEC-106), per-image failures at apply (crop out of bounds
   on this image, `trim`'s whole match, `pad`'s 0 px border): the exit codes on `edit` (one input
   only) and the message shapes. Confirm each message is the op's own (the same text a recipe gets),
   or say where `edit` must word it differently (a flag name, not a key name).
5. **What doesn't change.** Today's three flags, their output bytes and `--save-recipe`'s output for
   them. The recipe corpus guard (SPEC-141) is untouched, since `edit` isn't in it; say whether
   `tests/edit.rs` or `scripts/cli-golden.py` already pins `edit`'s bytes, and what pins them after.
6. **wasm.** `edit` is CLI only. Confirm no wasm row or count changes.
7. **Fold-ins.** Read every open STAGE-058 entry and recommend fold or leave for each, with why.
   Likely candidates: "`apply` and `build` warn when a recipe with no `auto-orient` step meets a
   tagged input" (it names `edit --save-recipe`), and "`crop`'s aspect can follow each image's
   orientation". Fold only what doesn't change the size. `pad`'s transparent fill is its own spec.
8. **Docs.** Every place that states `edit`'s flags or order, and what each says after.
9. **Out:** say where each lives: `pad` with a transparent fill (its own STAGE-058 entry), any-angle
   `--rotate` (STAGE-059), the ddphotos page (STAGE-060), `edit` flags for ops outside the five
   (`watermark`, `optimize`).
10. **Size, and whether to split.** The backlog says [S]. Five flags with their syntax and
    conflicts, a DEC, a possible behaviour change to today's flags, and a replay test across
    combinations may be an [M]. If it is, recommend a split that keeps each half shippable (for
    example: the ordering DEC with `--crop`, `--rotate` and `--flip` first, then `--pad` and
    `--trim`), and say which half closes STAGE-058.

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
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §13 git, §15 (its two newest verify rules bind
   every cycle: "A bound or 'can't happen' is a claim until its worst input is driven" and
   "Classify every finding") and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order, the risk on
   `edit`'s order, and "Explicitly out of scope".
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item, every open entry, the exit criteria ("`edit` accepts each op as a flag, applies them
   in the documented order, and `--save-recipe` writes steps that `apply` replays to identical
   pixels") and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
   (traps 10, 12, 13, 14 and 17). PROJ-017 has no HANDOFF of its own, so these bind here. If the
   file isn't in your checkout, say so.
5. DEC-005, DEC-102, DEC-106, DEC-112, DEC-113.
6. SPEC-145 and SPEC-152 in `projects/PROJ-017-…/specs/done/`, including their ship reflections:
   SPEC-152's proposes that a mutation row quote its exact mutant edit and that a count carry its
   derivation. Apply both in your stub where they arise.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code.** `crustyimg edit --crop …` and `--rotate 90` on a release
  build of `main` (fresh `CARGO_TARGET_DIR`, deleted at the end): record the exact refusal and exit
  code. Then show the order problem with recipes, not words: a non-square input through `crop` then
  `rotate`, and `rotate` then `crop`, via `apply`; quote both output sizes, and say which order a
  user typing `edit --rotate 90 --crop …` would expect.
- **Drive clap**, in a scratch crate: typed order across different flags, a repeated flag, and how a
  conflict is reported.
- **Drive the crop pixel round trip** (question 2), over a stated range of image sizes and
  rectangles, and name the edge input if one exists.
- **Size the verification, not just the code** (AGENTS §2): flags × forms × typed orders × conflicts
  × colour types × tagged input × `--save-recipe` replay.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), and any fold-in's files.
- **Questions only the maintainer can answer:** one line each, with your recommendation. The
  ordering rule, whether today's three flags change behaviour, and the `--crop` syntax are likely
  ones.

### Allocate the id
Run `just new-spec "<title>" STAGE-058 PROJ-017`. If it gives anything other than SPEC-153,
`git mv` the spec and its timeline to SPEC-153 and fix the id inside both. Then run
`just advance-cycle SPEC-153 frame`. The ordering DEC will take the next free DEC id (DEC-114 on
`main` today); recommend its content, don't write it.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-005, DEC-102, DEC-106, DEC-112, DEC-113, SPEC-110, SPEC-145, SPEC-147, SPEC-151, SPEC-152),
  and `cycle: frame`.
- `## Context` and `## Goal`: what a CLI user can't do today, and what they can when this ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the order problem, driven, and the ordering rule you recommend, with the DEC's content;
  - each flag's syntax and what `--save-recipe` writes;
  - the crop round trip, driven, and its range;
  - where each failure happens (parse, load, apply) and its exit code;
  - sizing and why;
  - scope in/out, including fold-ins;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-153-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-153 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-153** (framed <date>)`. If you recommend folding an entry, mark it
   `(folded into SPEC-153)` the way SPEC-152's folded entry was. On a split, file the second half
   as a new `- [ ] (not yet written)` entry right after yours. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of the stage it belongs to, and update that stage's
   `**Count:**` line with a grep you've just run. Something with no stage goes in
   `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-153): frame edit flags for the five geometry ops`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-058 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. A probe you drive at frame lives in a
  scratch directory and is never committed.
- Other sessions may be working at the same time. Never share a `CARGO_TARGET_DIR`. If
  `git rebase origin/main` conflicts in a file outside PROJ-017, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~` or `/tmp`.
- **Budget ~40 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-153 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

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
Plain lists only. First line: `SPEC-153 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- the order problem as driven: both orders' output sizes, quoted;
- the ordering rule you recommend, the alternatives, and the DEC's content;
- what clap did: typed order, repeats, conflicts;
- each flag's syntax and what `--save-recipe` writes;
- the crop pixel round trip: the range driven and any edge input;
- the replay test you recommend;
- refusals: where each fails and its exit code;
- whether today's three flags change behaviour;
- the fold-in candidates: folded or not, and why;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
