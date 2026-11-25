# STAGE-058 — STAGE SHIP prompt: Lossless geometry

Cycle: **stage ship** (`FIRST_SESSION_PROMPTS.md`, Prompt 1d, made specific here). Run it in a fresh
CLI session with its own worktree, on **Opus**:

```
just wt docs/stage-058-ship -- --model opus
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Where things stand
Every spec in STAGE-058 has shipped and is archived in
`projects/PROJ-017-geometry-as-recipe-steps/specs/done/`. There are eight:
- SPEC-141: the no-bytes-changed guard;
- SPEC-145: `crop`, and DEC-112's units rule;
- SPEC-147: quarter-turn `rotate` and `flip`;
- SPEC-151: `pad`;
- SPEC-152: `trim`;
- SPEC-153: `edit --crop`/`--rotate`/`--flip`, and DEC-114's typed order;
- SPEC-159: transparent `pad`, and DEC-115's no-alpha output rule;
- SPEC-160: `edit --pad`/`--trim`, and `parse_color` refusing a sign.

The last one shipped on 2026-11-13 (`9308f34`). The backlog's `**Count:**` line reads
`18 resolved (8 shipped specs, 6 folded, 4 moved) / 0 active / 0 pending`.

**PROJ-017 is still `proposed`**, and PROJ-011 is the active project, so `just status`,
`just backlog` and `just roadmap` won't show this stage. That is expected. Don't treat it as an
error, and don't change the project's status.

## Step 0
1. Run `git fetch origin && git rebase origin/main`.
2. Confirm with `/usr/bin/grep` on the stage file that no `- [ ] **SPEC-` or
   `- [ ] (not yet written)` line remains, and that all eight specs above are in `specs/done/`.
   If either check fails, stop and report.

## Read first
- `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`,
  all of it: the front-matter `value_contribution`, Success Criteria, Scope, Spec Backlog, Design
  Notes and Dependencies.
- `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the success signals and the
  Stage Plan.
- Each of the eight specs. Read the `## Build Completion`, the build-phase reflection,
  `## Reflection (Ship)` and the `cost` block of each.
- DEC-112, DEC-114 and DEC-115.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` ("Traps this wave paid for" and
  the rules). It **does** exist at that path. SPEC-159's and SPEC-160's ships both wrongly said
  the repo tracks no HANDOFF.
- `/AGENTS.md` §8, §10, §15 and §16.

## Do, in order

### 1 — Success Criteria, driven, not read
For each of the five criteria, say **met / partly / not met**. Base it on evidence from a
**release build of current `main`**, built in a fresh `CARGO_TARGET_DIR` that you delete at the
end. SPEC-160's ship mapped each criterion to a test by name only; drive each one on the binary.
1. **Each op is registered with a key table, and a bad key or value is refused at load, naming
   the step.** For each of the five ops, drive one unknown key and one bad value in a two-input
   batch. Each must exit at load, name the step, and write nothing.
2. **Rotate-90 four times and flip twice return the input exactly**, in 8-bit and 16-bit, with
   and without alpha. Use four recipes on four fixtures you make yourself. Compare pixels with a
   decoder that isn't `image`, such as ImageMagick `compare -metric AE`.
3. **A fractional crop gives the same relative region on a 1200 px and a 6000 px copy**, within
   one pixel of rounding. Make the 6000 px image and a 1200 px downscale of it, run one crop
   recipe on both, and compare the boxes against DEC-112's rounding rule.
4. **`edit` applies each op as a flag in the documented order, and `--save-recipe` writes steps
   that `apply` replays to identical pixels.** Run one `edit` that uses all five geometry flags in
   a non-canonical typed order. Then `apply` the saved recipe and compare the bytes. Quote the
   saved step list.
5. **The recipe corpus gives byte-identical output before and after the stage.** Run
   `cargo test --release --test recipe_corpus_output` and quote the count. Then say what the
   golden was pinned against: was it pinned before SPEC-145, the stage's first op? Give the
   commit.

### 2 — value_contribution
- Check each of the five `delivers` lines against what shipped.
- Flag any spec whose `value_link` claimed something it didn't deliver.
- Check that each of the three `explicitly_does_not` lines held:
  - **"Resample":** for one output of each op, show that every output pixel is an input pixel or
    the fill colour. Use a small script with a decoder you didn't write.
  - **"Detect content":** read `trim`'s code and say what it compares.
  - **"Change the output of any existing recipe":** criterion 5 covers it.

### 3 — The Design Notes, checked
The stage file has four design notes. Say whether each held, with evidence.
- **"Metadata after geometry"** asks what EXIF `PixelXDimension`/`PixelYDimension` and
  orientation say after a crop or rotate. It also asks that the `meta` lane behaviour (C2PA
  dropped on the pixel lane) stay unchanged.
  - Find whether any of the eight specs tested this.
  - If none did, drive it: crop a JPEG that has those tags, rotate it 90°, and read the tags back
    with `exiftool`.
  - Report what you see. A wrong dimension tag is an out-of-spec finding, and it needs a home
    (step 6).

### 4 — Summary
Write three sentences:
- built vs planned (the original six-entry backlog against eight specs, with 6 entries folded
  and 4 moved);
- speed: dates from SPEC-141's frame to SPEC-160's ship, and the eight specs' `cost.totals`,
  summed with a script;
- emergent behaviour: what shipped that nobody planned. Examples are DEC-115's alpha rule, and
  `parse_color` refusing a sign.

### 5 — `## Stage-Level Reflection`
Write it into the stage file, replacing the template's placeholder bullets. Draw it only from the
specs' reflections, HANDOFF's traps and the timelines; invent nothing. Name the patterns that
repeated across specs. Check at least these:
- **Mutation tables:** cells were incomplete at build in SPEC-159 and SPEC-160, and possibly
  earlier. How many specs, and was the red set reasoned rather than read off a run?
- **Doc sentences:** SPEC-153, SPEC-159 and SPEC-160 each had a doc sentence that verify found
  wrong or incomplete, after build had "driven" the docs.
- **A rule made stale by its own change:** DEC-115 was written at design and corrected at
  SPEC-159's ship. A code comment was missed in SPEC-160.
- **CI legs `just check` doesn't run:** wasm in SPEC-159, Windows paths in SPEC-160.
- **What went well:** for example, wording-only punch lists needing no re-verify (AGENTS §15)
  from SPEC-147 on, and the frame's driven bounds.

### 6 — Front-matter and the brief
- **Stage file:** set `stage.status: shipped` and `shipped_at: 2026-11-13`.
- **Brief:**
  - change STAGE-058's line in the Stage Plan to `- [x] STAGE-058 (shipped 2026-11-13) — …`;
  - recount the Stage Plan's `**Count:**` line with `/usr/bin/grep` and show the greps;
  - check STAGE-059 and STAGE-060, which may be listed as "proposed" while one is underway, and
    report what you find. Don't change them.
- ⛔ Don't change `project.status`; the maintainer decides that.
- Read both files back after editing. A file you wrote is not a file the tooling sees until you
  read it back.

### 7 — A home for every open item
STAGE-058 has no pending entry. But step 3 may find one, and the eight specs' reflections may
name follow-ups that no entry holds. For each, propose a home with a one-line reason:
- STAGE-059;
- STAGE-060;
- `guidance/ideas.yaml`;
- close it, with the reason.

⚠ Do not propose PROJ-013 (`proposed`, invisible to `just backlog`, HANDOFF trap 8). **Move
nothing yourself.**

### 8 — Proposed process changes
The last two ships proposed six changes, and none has been applied. Judge them together. For
each, say whether one already covers it and where:
- AGENTS §15–16, HANDOFF, `guidance/`, or the templates under `projects/_templates/`.

Then propose where each one belongs.
- **SPEC-159 ship:**
  1. Build drives each doc sentence on every exit path the spec names, not one.
  2. Design re-reads its DEC against its own corrections before it hands off.
  3. The build's file list includes what `just wasm-test` touches when `src/wasm.rs` changes.
- **SPEC-160 ship:**
  4. When a change makes a rule stale, grep comments as well as DECs for the old rule.
  5. A mutation row's red set is read off a full `--no-fail-fast` run of every target, not off
     the FT list.
  6. A test that asserts a path compares it after normalising separators.

Merge any that overlap: 2 and 4 may be one rule.

**Write each surviving change as a diff in your report. Don't apply any of them.** crustyimg is
also a testbed for the template, and a change to `projects/_templates/` or `/AGENTS.md` reaches
every future project. So mark each change **crustyimg-specific** or **general**.

### 9 — Commit
1. Commit the stage file and the brief to `main` directly, with
   `git commit -s -m "ship(STAGE-058): lossless geometry — reflected, shipped"` and the trailer
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **The DCO sign-off is mandatory.**
2. Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a
   reject, rebase and push again; never force.
3. Read back `git show --stat HEAD`. Only the stage file and the brief may appear.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, tests, CI, DECs, `/AGENTS.md`, HANDOFF, `guidance/` or templates. Propose only.
  Only the stage file and the brief change.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file.
- A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters, and macOS has no `timeout(1)`.
- `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything
  you count.
- Never share a `CARGO_TARGET_DIR` with another session. Write scratch files only in your
  session's scratchpad, never `/tmp`. Delete scratch images and target dirs when you're done.
- **Budget ~60 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by its **first user message**. That message is the paste line, `Read and follow
  …/STAGE-058-ship.md`, not this prompt's title.
- Dedup by `.message.id`. Take input, cache_creation and cache_read once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts.
- Price at Opus's $5/$25 per MTok. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Measure at your last turn. A stage has no `cost` block, so report the numbers only.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `STAGE-058 ship: shipped — <commit SHA on main>`. Then:
- the five Success Criteria, each met / partly / not met, with one line of driven evidence;
- value_contribution: any `delivers` or `explicitly_does_not` line that didn't hold;
- the four design notes, each held or not, with the metadata drive quoted;
- the three-sentence summary, with the summed cost;
- the brief's STAGE-058 line and `**Count:**` as they now read, and what you found about
  STAGE-059/060's status;
- open items with proposed homes, or "none";
- the process changes as diffs, each marked crustyimg-specific or general, with what already
  covers it;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
