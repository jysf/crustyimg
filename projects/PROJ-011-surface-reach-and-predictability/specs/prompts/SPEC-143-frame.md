# SPEC-143 — FRAME prompt: a watermark that draws nothing is refused

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-143-frame -- --model opus
```

**Reserved:** **SPEC-143**, and **DEC-109** for the design session. DEC-109 may be released unused
if the change only amends an existing DEC.

Your item is one entry in STAGE-054's Spec Backlog, filed `(not yet written)`:
"**A watermark with no visible pixels is drawn as nothing at exit 0.**" It was filed by SPEC-135's
frame on 2026-10-27. Read it in full, including the ruling line. Touch no other item.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why this, and why now
- 0.8.1 shipped on 2026-10-30. This opens **0.8.2**. It is the last open piece of STAGE-054's
  first success criterion: "no CLI or recipe path draws an invisible mark at exit 0".
- **The maintainer ruled on 2026-10-30: `opacity = 0` is refused**, like the other invisible
  marks, with the same exit codes. The way to turn a watermark off is to remove the step. **Do not
  reopen this.** Frame the spec around one rule: a mark that would change no pixel is refused.
- It is [S] on paper. Keep it that way. Anything else you find goes to a new STAGE-054 backlog
  entry (HANDOFF rule 6), not into this spec. In particular, the neighbouring entry "A text
  watermark's raster has no memory bound beyond `size`" is **out of scope**.

## What is known (the backlog's evidence; reproduce it, don't trust it)
- Driven on `main` at `4d6808b` (before 0.8.0). On **both** the CLI and the recipe path,
  `--text " "` / `text = " "`, `--size 0.01` / `size = 0.01`, and `--opacity 0` /
  `opacity = 0.0` all exit 0 and change 0 pixels.
- For blank text and a tiny size: `render_text` returns a 1×1 transparent canvas when no glyph is
  drawable (about `src/text/mod.rs:194-197`).
- `visible_pixels()` (`src/operation/mod.rs:1398`) is already computed at construction (about
  `:1451`, `:1482`, `:1634`), but nothing refuses a mark whose count is 0.
- The suggested fix shape: refuse a 0-visible-pixel mark at construction, with exit 2 on the CLI
  and exit 1 in a recipe (the invocation-level codes SPEC-135 used).

## Questions this frame must answer
1. **Premise.** On a release build of current `main`, drive every row on both paths: blank text,
   a tiny `size`, `opacity` 0, and an image watermark that is fully transparent. Count changed
   pixels with an independent tool (ImageMagick or Python, never crustyimg). Negative control: a
   normal mark must change pixels. Record exit codes and stderr.
2. **Does `visible_pixels()` see opacity?** Read the function and every caller in full. Is opacity
   applied before or after the count? If after, a 0-visible check alone will not catch
   `opacity = 0`, and the design needs a second condition or must count after opacity.
3. **Where "almost zero" lands.** Find the smallest opacity, and the smallest `size`, that still
   changes at least one pixel in the 8-bit output, on a light and on a dark background. Say
   whether "0 visible pixels" can be judged from the mark alone, or only after it is composited
   onto the image. The rule must not refuse a faint mark that does change pixels, and must not
   let one through that changes none. This decides where the check lives.
4. **Every path that builds a watermark.** List every construction site (CLI `watermark`, recipe
   step, `apply`, `build`, the wasm `transform`) with `file:line`, and say whether one check at
   construction covers them all. Check the browser path's error type and exit/abort behaviour.
5. **Message and codes.** Propose the exact error line for each cause (blank text, too small,
   opacity 0, transparent image), saying what to do instead. Confirm exit 2 on the CLI and exit 1
   in a recipe match how SPEC-135's refusals behave today; drive one SPEC-135 refusal to show it.
6. **Bytes.** Every mark that draws today must give byte-identical output after the fix. Name the
   check design must require.
7. **Docs.** List every sentence that describes `--opacity`, `opacity` or an empty `--text`
   (`docs/cli-reference.md`, the recipe guide, `skills/crustyimg/SKILL.md`, `--help`). Say which
   change.
8. **Size.** It should be [S]. If it grows, recommend the cut.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context.
Separate design, build and verify sessions follow. The orchestrator only sequences.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (1–12). Binding.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`:
   your item in full, and the success criteria at the top.
4. SPEC-135 (in `specs/done/`): its Frame, Design and the refusal it shipped. DEC-106 and any DEC
   it cites for watermark validation.
5. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Reproduce on a release build of
  `main` (`cargo build --release`, in a fresh `CARGO_TARGET_DIR` that you delete at the end) with
  a negative control. Record the exact command and output. **Measure the scenario, not the
  control.**
- **Size the verification, not just the code** (AGENTS §2). Count each AC matrix: causes × paths.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### The id is reserved — use exactly SPEC-143
Before allocating, confirm SPEC-143 is unused everywhere:
```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do git grep -l 'SPEC-143' "$b" -- . ; done
```
Then run `just new-spec "<title>" STAGE-054 PROJ-011`. If it allocates anything other than
SPEC-143, `git mv` the spec and timeline files and fix the id inside them. Then
`just advance-cycle SPEC-143 frame`. Write no DEC at frame.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (decisions and related specs), `cycle: frame`.
- `## Context` and `## Goal`: what the user sees today (your reproduction), and what they see when
  it ships, in the user's terms (the command, the stderr line, the exit code).
- A `## Frame` section directly after `## Goal`: verdict (go / no-go / split), premise check with
  evidence, the answers to questions 1–8, sizing and why, scope in/out, rough build file list,
  open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary; add `[ ]` lines for design
  (`prompts/SPEC-143-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), no PR.
1. On **go**: `just advance-cycle SPEC-143 design`. On no-go or split: leave `cycle: frame`.
2. In STAGE-054's backlog, change **only your item's** `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-143** (framed <date>)`. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of STAGE-054's backlog, and update its `**Count:**`
   line with a grep you have just run.
3. `git commit -s`, ending the message with
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **The DCO sign-off is mandatory.**
   Message: `docs(SPEC-143): frame — a watermark that draws nothing is refused`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. Other sessions
   push to `main` too; if the push is rejected, rebase and push again.
5. Read back what you pushed (`git show --stat HEAD`, and the STAGE-054 diff), and run
   `just backlog` to confirm the tooling sees SPEC-143.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag.
- `cargo test` is not needed at frame.
- A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~45 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** `grep -l "FRAME prompt: a watermark that draws nothing" ~/.claude/projects/<slug>/*.jsonl`.

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
First line: `SPEC-143 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the premise check (confirmed / wrong, one line of evidence per row, both paths);
- whether `visible_pixels()` sees opacity, and the "almost zero" thresholds;
- every construction site, and whether one check covers them;
- the proposed messages and exit codes;
- size and why; scope in/out;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings and where you filed them;
- your measured cost.
