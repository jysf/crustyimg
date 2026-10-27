# SPEC-135 — FRAME prompt: `watermark` refuses a size or scale that is not a sensible number

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-135-frame -- --model opus
```

**Reserved: SPEC-135.** DEC-106 is reserved for the design session; it may be released unused if
the change only amends DEC-102 or DEC-104.

Your item is the **first** entry in STAGE-054's Spec Backlog: "⚡ **Before 0.8.0: a `watermark`
size or scale that is not a sensible number is refused, never silently drawn as nothing.**" It
holds two earlier filings, moved verbatim from STAGE-050. Read both in full. Touch no other item.

## Why this spec exists, and its deadline
STAGE-050's Stage Ship (2026-10-27, `3512120`) judged its criterion 4, "a watermark that cannot
sensibly be applied is refused or skipped, never silently dropped", only **partly** met. Driven on
a release build of `main` at `1e63016`:
- `watermark --size nan` and `--size inf`: exit 0, output byte-equal to `--opacity 0`;
- `watermark --scale nan`: exit 0, 1 pixel changed out of 3.2 M;
- `watermark --scale inf`: refused, but the message names a `4294967295×4294967295 px mark`;
- the recipe path already refuses `nan` and `inf` (SPEC-131, DEC-102).

0.8.0 headlines that rule, so the maintainer ruled (2026-10-27) that this spec ships **before
0.8.0**, and that 0.8.0 is this spec plus release prep, nothing else. Scope creep here delays
the release. Anything you find beyond this item goes to a backlog home (STAGE-054 for 0.8.1),
not into this spec.

## Settled (do not re-open)
- The CLI must refuse what the recipe already refuses. A watermark that renders as nothing at
  exit 0 is the defect.
- A text mark's `size` gets an upper bound, the **same** on the CLI and in a recipe. SPEC-131's
  design measured `size = 100000.0` in a recipe: accepted, 52 s of wall time over 3 small inputs,
  exit 0.
- `--scale inf`'s message must not print a saturated `u32`.
- STAGE-054's design note: the CLI check reuses SPEC-131's per-op schema
  (`OperationRegistry::param_specs`, `src/operation/mod.rs`), not a second hand-written copy, so
  the two paths cannot drift again. If that is impossible or wrong, say why. Do not silently
  write a copy.

## Questions this frame must answer
1. **Premise.** Reproduce the four CLI rows above on a release build of current `main`, with a
   negative control: an output diffed against itself shows 0 changed px, and a normal
   `--size 48` mark shows a non-zero change. Read where the CLI builds the op (`new_text` /
   `new_image`, reached from `src/cli/`) and where the recipe path validates, and cite `file:line`
   for both.
2. **Sweep the verb's numeric inputs, then stop.** List every numeric flag `watermark` takes
   (`--size`, `--scale`, `--opacity`, margin or offset if any). For each, record what the CLI
   accepts and what the recipe schema accepts for `nan`, `inf`, `-inf`, `0`, a negative and a
   huge value. Every row where the two paths disagree is in scope. Do not sweep other verbs: a
   finding on another verb is an out-of-spec line with a proposed STAGE-054 backlog entry
   (HANDOFF rule 6).
3. **The cap.** Recommend the upper bound on text `size`, with evidence: time a few sizes on a
   release build (for example 500, 2 000, 10 000, 100 000) on a small and a large image. Does it
   belong in the schema (a range row) or in DEC-036's resource-limit family? Does `--scale` need
   an upper bound too, or does SPEC-133's fit test already refuse every scale too big to fit?
   This is a **maintainer question**: give one number and why.
4. **Exit codes and arity.** A bad `--size` is the same for every input, so it is a usage error
   on the invocation, not a per-image skip. Confirm what the contract says
   (`docs/api-contract.md`, the exit-code table) and recommend:
   - the CLI exits before reading any input, on one input, on a batch, on a glob that resolves to
     one file, and on stdin (STAGE-050's lesson: input arity is not argument count);
   - the recipe path keeps exit 1 at load (SPEC-131).
5. **Size.** Target **[S]**. If the sweep makes it larger, recommend the cut: what ships in 0.8.0
   and what gets a STAGE-054 entry.
6. **Bytes.** This should change no output for any accepted mark. Name the check design must
   require: a passing mark byte-identical to `main`'s binary.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context. A
separate, fresh design session does that from your stub, then separate sessions build and verify.
The orchestrator only sequences; it does not frame or design. The design prompt is written after
you land, from your stub.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (1–10). Binding.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`:
   your item in full, and the Design Notes.
4. DEC-102 (recipe params validated at load) and DEC-104 (the watermark damage rule).
5. `specs/done/` SPEC-131 and SPEC-133: their Build Completion and Verify Readout, for how the
   schema and the refusal path work today.
6. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Reproduce on a release build of
  `main` (`cargo build --release`, in a fresh `CARGO_TARGET_DIR` that you delete at the end) with
  a negative control, and record the exact command and output. **Measure the scenario, not the
  control**: the row that carries the argument must use the flag it is about.
- **Size the verification, not just the code** (AGENTS §2). Count the AC matrix: flags × values ×
  arity × CLI/recipe.
- **Scope boundary:** what is in, what is out, and where each out item lives (an existing backlog
  entry, or "needs filing" with a one-line description).
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation. Do not ask
  what the code, a DEC or a ruling already answers.

### Ids are reserved — use exactly these
`just new-spec` allocates ids by scanning the working tree. Run
`just new-spec "<title>" STAGE-054 PROJ-011`; if it allocates anything other than SPEC-135,
`git mv` both files (spec and timeline) to SPEC-135 and fix the id inside them. Then
`just advance-cycle SPEC-135 frame`. Do not write any DEC at frame.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (decisions and related specs), `cycle: frame`.
- `## Context` and `## Goal`: what the user sees today (your reproduction), and what they see when
  this ships, in the user's terms (the command, the stderr line, the exit code).
- A new section `## Frame` directly after `## Goal`: verdict (go / no-go / split), premise check
  with evidence, the sweep table, the cap measurement, sizing and why, scope in/out, rough build
  file list, open maintainer questions.
- `### Out of scope (for this spec specifically)` filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary; add `[ ]` lines for design
  (`prompts/SPEC-135-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), no PR.
1. On **go**: `just advance-cycle SPEC-135 design`. On no-go or split: leave `cycle: frame`.
2. In STAGE-054's backlog, change **only your item's** `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-135** (framed <date>)`. File any out-of-spec finding as a new `- [ ] (not yet
   written)` entry at the end of STAGE-054's backlog, and update its `**Count:**` line with a grep
   you have just run.
3. `git commit -s`. **The DCO sign-off is mandatory.** Message: `docs(SPEC-135): frame <short title>`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed (`git show --stat HEAD`, and the STAGE-054 diff), and run
   `just backlog` to confirm the tooling sees SPEC-135.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag.
- `cargo test` is not needed at frame.
- A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- **Budget ~50 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** `grep -l "SPEC-135 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read
×0.10. Append a `frame` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown`, `estimated_usd` and the rate anchors, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-135 frame: GO | NO-GO | SPLIT — <commit SHA on main>`. Then, short:
- the premise check (confirmed / wrong, with one line of evidence per row);
- the sweep table (flag × value, CLI vs recipe);
- the recommended cap, with the timings;
- size and why; scope in/out;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings and where you filed them;
- your measured cost.
