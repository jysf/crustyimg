# STAGE-050 — STAGE SHIP prompt: Recipe Reach

Cycle: **stage ship** (`FIRST_SESSION_PROMPTS.md`, Prompt 1d, made specific here). Fresh CLI
session, own worktree, **Opus**:

```
just wt docs/stage-050-ship -- --model opus
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Where things stand
Every spec in STAGE-050 has shipped and been archived to `specs/done/`: SPEC-127, 128, 129, 131,
132, 133 and 134. The last two shipped on 2026-10-27 (`9765d3c`, `328efa3`). All of it is on
`main` and goes out as **one release, 0.8.0**. Release prep runs after you, in separate sessions.
**This stage closes before the release; it does not wait for the tag.**

## Step 0
`git fetch origin && git rebase origin/main`. Then run `just status` and `just backlog`, and confirm
that no SPEC in STAGE-050 is still open. If one is, stop and report.

## Read first
- `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-050-recipe-reach.md`, all of it:
  the front-matter `value_contribution`, Success Criteria, Scope, Spec Backlog and Design Notes.
- Every shipped spec in `specs/done/` that belongs to this stage, including the `## Build
  Completion`, `## Reflection (Ship)`, build-phase reflection and `cost` block of each.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: the "Traps this wave paid for"
  (1–10) and the rules.
- `/AGENTS.md` §8, §10 and §15.

## Do, in order

### 1 — Success Criteria, driven, not read
For each of the four criteria, say **met / partly / not met**, with evidence from a **release build
of current `main`**: use a fresh `CARGO_TARGET_DIR`, and delete it at the end.
- **Criterion 1** ("one `crustyimg build` watermarks and optimizes a photo directory from a
  manifest"). Drive it end to end:
  - a scratch directory of at least five photos, mixed JPEG and PNG, at real sizes, plus one too
    small for the mark;
  - a `crustyimg.toml` manifest and a recipe with a `watermark` step (text or image) and a
    terminal `optimize`.
  - Run `build`, then `build --check`, then a cached re-run.
  - Report exit codes, the summary line, the skip warning for the small image, and the output
    formats. Sniff each format with `file`, not by the name.
- **Criterion 2:** a recipe pins `format` and `quality`, and `apply --recipe` reaches what
  `convert` reaches. Show matching output on at least one pair.
- **Criterion 3:** a malformed recipe fails at load and names the bad key. Show an unknown key, a
  wrong type and an out-of-range value, over a batch of several inputs, with no output written.
- **Criterion 4:** a watermark that cannot sensibly be applied is refused or skipped, never
  silently dropped. One input → exit 2; a batch → skip with a warning.
  - Also state plainly that `--size nan`, `--size inf` and `--scale nan` still drop the mark
    silently at exit 0. That is an open backlog item, and it is a counter-example to this
    criterion on the CLI path. Judge whether that makes criterion 4 **partly** met, and say so
    either way.

### 2 — value_contribution
- Check each `delivers` line against what shipped. Flag any spec whose `value_link` claimed
  something it did not deliver.
- Check the three `explicitly_does_not` lines held. Drive or grep each one; for example, confirm
  `watermark` writes no EXIF or C2PA.

### 3 — Summary
Three sentences: built vs planned, speed (use dates and the specs' `cost.totals`, summed), and
emergent behaviour (what shipped that nobody planned, e.g. SPEC-132's Linux idle-rebuild fix).

### 4 — `## Stage-Level Reflection`
Write it into the stage file. Draw it from the specs' reflections and HANDOFF's traps; do not
invent anything. Name the patterns that repeated across specs. Two that did:
- **tests that passed hollow:** SPEC-129, SPEC-133, and SPEC-134's M2;
- **edges the design's surface table missed:** a glob or directory that resolves to one input, and
  stdin, in SPEC-133 and SPEC-134.

### 5 — Front-matter
`stage.status: shipped` and `shipped_at: 2026-10-27`. Then run `just roadmap` and confirm STAGE-050
renders as completed. A file you wrote is not a file the tooling sees until you read it back.

### 6 — A home for every open backlog item
The `**Count:**` line says 9 items are pending. None is a spec yet, and none blocks 0.8.0. For each
one, propose a home, with a one-line reason:
- (a) a new stage in PROJ-011;
- (b) an existing stage, for example STAGE-053 "guards that cover this project";
- (c) the post-0.8.0 plan (PROJ-014, "Wave 4");
- (d) close it, giving the reason.

⚠ **Do not move anything into PROJ-013.** It is still `proposed`, so `just backlog` cannot see it,
not even with `--all` (HANDOFF trap 8).

**Move nothing yourself.** Put the table in your report; the orchestrator and the maintainer
decide. Leave the items in STAGE-050 until then.

### 7 — Proposed process changes
The ships proposed three changes, and the orchestrator adopted one as HANDOFF trap 10. Judge all
three together, and propose where each belongs: `/AGENTS.md`, the design or build templates under
`projects/_templates/`, `guidance/`, or HANDOFF only.
- **Trap 10** (SPEC-134 ship): a helper that derives a value needs a test input where the derived
  and naive values differ, plus a mutation that swaps them.
- **SPEC-133 ship:** a design that turns an error into a silent fallback or skip must ask the build
  for an "abort on any skip" probe, to separate tests that flip from tests that pass hollow.
- **SPEC-133 ship:** every design's surface table includes one input reached through a glob or
  directory, and stdin.

**Write each as a diff in your report. Do not apply any of them.** Remember that crustyimg is
also a testbed for the template: a change to `projects/_templates/` or `/AGENTS.md` reaches every
future project, so say whether each change is crustyimg-specific or general.

### 8 — Commit
1. Commit the stage file (front-matter and Stage-Level Reflection) to `main` directly:
   `git commit -s -m "ship(STAGE-050): Recipe Reach — reflected, shipped"`. **The DCO sign-off is
   mandatory.**
2. Push: `git fetch origin && git rebase origin/main && git push origin HEAD:main`.
3. Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No version bump, no tag, no release, no CHANGELOG edit. Release prep owns those.
- Do not edit code, tests, DECs, `/AGENTS.md` or templates. Propose only.
- `cargo test` is not needed. If you do run it, it fails `display_sink_refuses_non_tty` in an
  interactive terminal; redirect stdout to a file.
- A piped command reports the pipe's exit code, and zsh does not word-split unquoted parameters.
- **Budget ~60 exchanges.**

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`. Find it by content:
`grep -l "STAGE-050 — STAGE SHIP prompt" ~/.claude/projects/<slug>/*.jsonl`.

Dedup by `.message.id`: take input, cache_creation and cache_read once per id, and the MAX
output_tokens. Price each component at the model that actually ran, never at a flat rate: Opus
$5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `STAGE-050 ship: shipped — <commit SHA on main>`. Then:
- the four Success Criteria, each met / partly / not met, with one line of evidence;
- value_contribution: any `delivers` or `explicitly_does_not` line that did not hold;
- the three-sentence summary;
- the backlog-home table: item, proposed home, reason;
- the three process changes as diffs, each marked crustyimg-specific or general;
- your cost.
