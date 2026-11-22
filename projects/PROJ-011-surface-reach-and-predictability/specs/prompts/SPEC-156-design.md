# SPEC-156 — DESIGN prompt: `meta strip` keeps the Adobe APP14 colour-transform marker

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-156-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`aeda951`, cost in `c0832c4`, rulings
recorded after). **No new DEC.** The frame recommends an **amendment to DEC-003** and a one-line note
on DEC-029. You write both texts into Implementation Context, for ship to apply after the merge.
Don't edit either DEC now: `main` must stay true until the code lands.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise, driven on two decoders: the CMYK fixture's pixel (8,8) goes `#BF4020` → `#5D861E`,
  SSIMULACRA2 41.94;
- the three-component case: harmless with component IDs R,G,B, broken with IDs 1,2,3;
- the sweep: only `strip_all` drops APP14, no path copies it, and the two existing full-range tests
  (`src/metadata/mod.rs:630`, `:1712`) can't fail on this bug;
- APP14's bytes, the other APPn markers, C2PA and PNG;
- the regression design, its decoder, the fixtures and the negative control;
- M1 to M4; the DEC-003 amendment text; the docs lines.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer accepted both of the frame's recommendations on 2026-11-22 (recorded in the stub under
`### Maintainer rulings`):
1. **Keep APP14 only when it starts with `Adobe\0`.** Any other APP14 is stripped. M3 (drop the
   signature check) is a mutation row.
2. **The orientation and ICC warning belongs to the new STAGE-063 entry**, not here. This spec's docs
   say only what strip now keeps.

Also settled: no change to ICC (the DEC-003 entry folded into SPEC-150), the C2PA manifest check and
its warning, `PNG_METADATA_CHUNKS`, MPF and gain maps. SPEC-148's O-5 and SPEC-150's wasm item stay in
their entries.

## Order and neighbours
SPEC-154 lands first; this spec's **build** starts after it merges. Designs for SPEC-155 and SPEC-157
may run alongside yours. SPEC-155 touches `src/sink/mod.rs`, `src/cli/ops.rs` and `src/cli/build.rs`;
this spec should touch only `src/metadata/` and tests, but check. At Step 0, run
`gh pr list --state open --json number,title,files` and `git branch -r`, and list every file another
open PR or pushed `feat/spec-15[4-8]*` branch touches that this spec also touches. Write that map into
Implementation Context **from their file lists, not their specs' text**. PR #240 touches
`src/metadata/` (SPEC-150's wasm item); say whether it's still open and what it changes there. On a
rebase conflict in STAGE-063's file, keep both sides and recount the `**Count:**` line with a fresh
grep. On a conflict in any other file, stop and report. Never edit
`projects/PROJ-017-geometry-as-recipe-steps/`.

## Things in the frame to check before you build on them
Drive these on a release build of `main` and on a prototype (never pushed):
- **Every caller of `strip_all`.** The frame drove `meta strip`. List every caller
  (`/usr/bin/grep -rn "strip_all" src/`): a recipe `strip` step, `optimize`, `build`, `responsive`,
  wasm. For each, say whether a kept APP14 reaches its output and whether that output could be a
  re-encoded RGB JPEG. If any caller strips **after** a re-encode and keeps the source's APP14, that's
  the opposite bug (an RGB scan labelled YCCK): drive it.
- **Who reads "metadata is gone".** A kept APP14 is a segment that wasn't there after strip before.
  Check `lint` (any rule that flags remaining APPn segments or "not stripped"), `meta show` /
  `info`'s metadata listing, `crustyimg diff`'s metadata view, and any golden that pins the stripped
  output's bytes or segment list. Drive each on the prototype with the CMYK fixture and quote the
  before and after. Any visible change goes in an AC and the CHANGELOG.
- **The signature check's exact bytes.** Is it `Adobe` followed by `\0`, or just `Adobe` with a
  length floor? Quote the segment bytes from both fixtures (the frame gives the 14-byte CMYK form and a
  33-byte variant in `signed_gps.jpg`). Decide the length rule: too short to hold the transform byte →
  strip it or keep it? Drive a truncated APP14 through `image` and djpeg on the prototype.
- **`image` honours APP14.** The frame says the `image` decoder gives the same wrong colour as djpeg,
  so the test can decode with it. Re-drive that on the crate version in `Cargo.lock`, and keep djpeg
  (or ImageMagick) as the independent check in the spec's evidence.
- **The new fixture.** The three-component, transform-0, IDs-1,2,3 file. Write its exact recipe (the
  `cjpeg -rgb` command and the byte edit of the IDs, with offsets) so the build makes the same bytes,
  and say how the test proves the fixture really has IDs 1,2,3 and transform 0 (assert the bytes, not
  only the render).
- **The two existing sweeps.** Read `:630` and `:1712`. Say what each must become so it can fail on
  this bug: run it on an image that has an APP14.
- **wasm and the lean build.** Does `strip_all` compile into wasm, and does the lean
  `--no-default-features` build use it? Say which CI legs exercise the change.

## Design calls that are yours
- **The code shape.** The exception in `strip_all`'s loop, the signature check, and the
  `:1712` reconstruction, each with `file:line`. One helper, used in both places, so M4 is a real
  mutant.
- **The tests.** For each fixture (CMYK, the new three-component file, `signed_gps.jpg`): decoded RGBA
  equal before and after strip; the APP14 segment byte-identical; nothing else from APP1–APP15 or COM
  left. A non-Adobe APP14 (build one in the test from literal bytes) is removed. A C2PA control: the
  manifest-drop warning reports the same with and without the fix. Expected values from literals or an
  independent decoder, never from the code under test.
- **What doesn't change.** Every other `meta` verb's output bytes, every pixel-lane output, the PNG
  lane. Say what pins each today; add a pin only where none exists.
- **The DEC-003 amendment and the DEC-029 note,** verbatim, as ship-time text.
- **Docs.** `docs/cli-reference.md:231`, `strip_all`'s doc comment (`src/metadata/mod.rs:157-165`),
  and any surface the frame missed (`--help` long text, `README.md:193`, `skills/crustyimg/SKILL.md:148`,
  `SECURITY.md:88`). Re-check each `file:line` and write each sentence verbatim. List **every behaviour
  sentence and every exit path it names**; the build drives each sentence, and verify drives them
  again.
- **CHANGELOG.** The "Fixed" line, verbatim.
- **Size.** The frame says [S]. Say whether that holds, with the count's derivation next to it.

## Tests: each one must be able to fail
HANDOFF's traps and AGENTS §15 bind here. **Drive every mutation row at design on the prototype;
don't reason them.** Each row **quotes its exact mutant edit**, names the code location, every test
expected red and every test expected green, and is marked *driven* or *reasoned*. Read the red set
**off the prototype's full `cargo test --no-fail-fast` run of every target**, not off the reasoning.
At least the frame's M1 to M4, plus:
- the exception keyed on the marker alone, ignoring the signature (that is M3; the non-Adobe APP14
  test must red);
- the exception widened to all of APP14 and APP0–APP15's structural guesses (`0xE2`): which tests red?
- the negative control the frame names: put `0xEE` back into the sweep and show the pixel tests fail,
  quoting the failure message;
- **Trap 14:** restore `main`'s `src/metadata/mod.rs` verbatim and list everything that goes red.

**Every count in the spec carries its derivation** (the command, or the sum), next to the number.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and Implementation
Context. You don't write the code or edit any DEC. Separate build and verify sessions follow: one
builds the spec from your text alone, and another verifies it, so everything the builder needs must
be in the spec (AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so. Then build the overlap map above.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 and §16 habit 6 (no
   CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`:
   your entry, SPEC-154 to SPEC-158, and the new orientation entry (so you don't drift into it).
4. `docs/reviews/2026-11-10-external-code-review.md`: the strip finding.
5. `/guidance/constraints.yaml`, DEC-003, DEC-029.
6. A shipped PROJ-011 spec in `specs/done/` (SPEC-150 is a recent one) as the shape to match.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `strip_all`, its callers,
  `remove_segments_by_marker` or whatever does the sweep, the two test sweeps, the manifest check.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a test
  per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context, and
keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the frame`
subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-156 build`.
2. In STAGE-063's backlog, change SPEC-156's marker from `(framed 2026-11-22)` to `(designed <date>)`,
   keeping `[S]` (or the size you found). Mark design `[x]` in the timeline with a 2–4 line summary,
   and add `prompts/SPEC-156-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-156): design keeping APP14 in meta strip`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. On a reject,
   rebase and push again; never force.
5. Read back `git show --stat HEAD` and the STAGE-063 diff.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC edits. No committed
  fixture (the build makes it from your recipe). No pushed probe branch. No version bump, no tag.
  Don't raise or plan a release.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout to
  a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS has no
  `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Remove the prototype worktree and its target
  dir before you finish. Write scratch files in your session's scratchpad, never `/tmp` or the system
  temp dir.
- **Budget ~35 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content, not
recency:**
`/usr/bin/grep -l "SPEC-156 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take those
three once per id and the **MAX** `output_tokens`. Include any `subagents/` transcripts.

Price each component at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read is ×0.10. Measure at your last turn, after the
push, and append a `design` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-156 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- any correction to the frame, with what you drove (every `strip_all` caller, who reads "metadata is
  gone", the signature bytes and length rule, `image`'s APP14 handling, the new fixture's recipe, the
  two sweeps, wasm and lean);
- the code shape, one line per piece with `file:line`;
- the AC list (one line each);
- the Failing Tests and the mutation table, each row with its quoted edit, red and green tests, and
  driven or reasoned;
- what pins "nothing else changes";
- the DEC-003 amendment and the DEC-029 note, verbatim;
- the docs surfaces with `file:line`, and the behaviour sentences;
- the CHANGELOG line, verbatim;
- the overlap map with other open PRs and branches, from their file lists;
- size with its derivation, and anything the build prompt must carry;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
