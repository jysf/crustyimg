# SPEC-150 — FRAME prompt: two messages tell the truth, and `--help` carries no spec ids

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-150-frame -- --model opus
```

**Reserved:** **SPEC-150** (STAGE-063, reserved with 148 and 149). No DEC reserved. The maintainer pastes this prompt
into a fresh session and will paste your final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 (geometry as recipe steps)** and may be building its next spec at the
same time. It edits `src/operation/`, `src/recipe/`, `docs/recipes.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`'s
Unreleased section, `tests/common/mod.rs`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`,
`tests/wasm_roundtrip.rs`, `scripts/wasm-ci-check.sh`. This spec's likely files (`src/cli/mod.rs` help strings,
`src/wasm.rs`, `docs/cli-reference.md`) can overlap: **list every overlap** so the orchestrator can coordinate. Never
edit `projects/PROJ-017-geometry-as-recipe-steps/`. If a rebase conflicts in a PROJ-017 file, stop and say so.

## Why this, and why now
STAGE-063 is PROJ-011's last stage. SPEC-148 (`lint` fixes what it names) and SPEC-149 (no flag is accepted and
ignored) shipped. SPEC-150 is the stage's third spec and the last one before the 0.8.3 release cycle. Full text of every
item is in `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`.
Core items (from the stage entry):
1. **A folder with only subfolders** gives a message that is not true (drive it: `resize <dir> …`, `info <dir>`,
   `build`, `web`, `optimize` on a directory whose children are all directories, or only non-images; and an empty one).
2. **The browser's asset refusal** names a spec id (`src/wasm.rs:237`); a browser user sees an internal id. Find every
   user-visible string in `src/wasm.rs` and the web demo that carries a spec or DEC id, or says something false about
   what the browser build can do.
3. **Every `SPEC-` / `DEC-` id in rendered `--help`**, for every verb and for `crustyimg --help`. Known hits from
   SPEC-149's verify: `--no-cache` (SPEC-064), `--check` (SPEC-066), the `--frozen`/`--locked` note and `--strict`
   (DEC-059), `--watch` (SPEC-067), and the `build` (SPEC-063, DEC-057), `lint` (SPEC-050, DEC-050) and `meta`
   (SPEC-087, SPEC-089) descriptions. `lint --help`'s "(each with a runnable fix)" line is the SPEC-148 design's ask:
   reword it so it is true now (`fix: null` exists for stdin, 16-bit PNG and formats the metadata lane cannot write).
Candidate fold-ins, each for you to **rule in or out with a reason** (they share files or a theme):
4. Finding B: `optimize -q 10` and `web -q` exit 2 with a message that blames flags the user did not pass
   (`src/cli/optimize.rs` near :837 and :691).
5. The garbled `metadata lane does not support meta copy supports JPEG only in v1 yet` (find where it is built; check
   every sibling message built the same way, for `meta strip|clean|set|copy`).
6. `--keep-gps=false` gets clap's `unexpected value 'false'` instead of the R6 line (SPEC-149's guard table).
7. A `"` in a path prints unescaped in `could not read font file "a"b.ttf": …` (`watermark_overlay` in `src/cli/ops.rs`);
   is there a shared path-display helper the other messages already use?
8. Finding C: DEC-003's preserve claim. SPEC-149's design already amended DEC-003 and DEC-024 on `main`; check what is
   left (`AGENTS.md:503`, `SECURITY.md:50`, `docs/`), probably nothing.
Do **not** fold in O-1/O-2 (the `optimize` unreachable budget and `max_bytes = 0`) or the `optimize`-to-stdout size
item without saying why; they belong with `optimize` behaviour, not messages. State where each out item lives.

## Questions this frame must answer
1. **Reproduce each on current `main`** (build it; say the version and profile). Command, exit code, stdout/stderr,
   what the user sees. For item 1, give a table: directory shapes × verbs (a verb that takes a folder, with and without
   `-o`/`--out-dir`/`-r`). Which message is false, and what is true?
2. **The id sweep, as a measurement.** Write down the command that lists every rendered help page
   (`Cli::command()` walk, the way `tests/cli_flags_honest.rs` walks leaves) and its grep for `SPEC-|DEC-`; run it and
   paste the count of hits by verb. Also sweep the other user-visible text: error messages (`grep -rn '"[^"]*\(SPEC\|DEC\)-[0-9]' src`),
   `--version`/long-about, completions text, the wasm strings, the `docs/` pages that are *user* docs (not research or
   sessions). List which are user-facing and which are code comments (comments are fine: HANDOFF's "comments plain, no
   spec refs" is a separate preference, say if it applies to comments too, as a finding, not scope).
3. **A durable guard.** Recommend the cheapest test that makes the sweep stay done: a test that walks every help page
   and every error constant and fails on `SPEC-`/`DEC-`. Say how verify proves it can fail (HANDOFF traps 11 to 17: a
   mutation row names its location; the guard must iterate the real command tree, not a hand-kept list; a test that
   exercises nothing reports green). Where could an id hide that the walk cannot see (a runtime-formatted string, a
   `long_about` built by `format!`, a clap `after_help`)?
4. **What replaces each id.** For every help line that cites an id, propose the plain sentence that should replace it
   (behavior-first). This is the build's text, so make it exact for the high-traffic ones (`build`, `lint`, `meta`,
   `--check`, `--no-cache`, `--watch`, `--strict`), and a rule for the rest.
5. **Compatibility and docs.** Is any help string quoted by a doc, a golden test, an Action input, or SKILL.md
   (`grep`)? List every doc that mirrors a help line and must change with it (mark PROJ-017-owned files; do not edit
   them). Propose CHANGELOG text if anything is user-visible (it is probably `Fixed`); do not edit CHANGELOG.
6. **Size and scope.** Expect [S]–[M]. State the verification cost. If item 1 and the id sweep differ so much that
   two specs are cleaner, say SPLIT and how; do not create the second id yourself.

## Questions only the maintainer can answer
A numbered list at the end of your report, each with your recommendation and the one-line consequence. Likely: which
fold-ins are in; whether a folder with only subfolders should be an error (exit 2/3) or a quiet no-op with a note;
whether comments with spec ids are in scope. The spec stays at `cycle: frame` with a "rulings pending" line if they are
not known by the time you finish (go/no-go can still be GO with the rulings open).

---

## Rules every frame session follows

**You are the frame cycle only.** Go/no-go plus a correctly sized, correctly scoped spec stub. You do **not** write
acceptance criteria, failing tests or Implementation Context.

### Step 0
Own worktree (`just wt`). `git fetch origin && git rebase origin/main`. If you are in the maintainer's main checkout,
stop and say so. Own `CARGO_TARGET_DIR`.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules (1 to 7), "Traps this wave
   paid for" (1 to 17). Binding.
3. STAGE-063 and `/guidance/constraints.yaml`.
4. SPEC-149's spec under `specs/done/` (its guard table and `tests/cli_flags_honest.rs` are the model for a command-tree
   walk) and SPEC-148's (`lint`'s fix lines); `src/cli/mod.rs` (help strings), `src/wasm.rs`, `src/cli/ops.rs`,
   `docs/cli-reference.md`.

### What a good frame does here
- **Drive it before you describe it.** Questions 1 and 2 are measurements; paste the counts.
- ⛔ Run nothing that publishes or tags. No `git tag`, no `npm publish`, no `gh workflow run`.
- Rough build file list (paths) with the PROJ-017 overlaps marked.

### The id is reserved: use exactly SPEC-150
```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-150' "$b" -- . ; done
```
Expect only STAGE-063 and this prompt (and maybe PROJ-017's brief). Then `just new-spec "<title>" STAGE-063 PROJ-011`;
`just new-spec` scans only the working tree and may allocate another number; if it is not SPEC-150, `git mv` the spec
and timeline files and fix the id inside them. Then `just advance-cycle SPEC-150 frame`. Write no DEC at frame.

### What to write into the stub
Front matter (`type`, `priority`, `complexity`, `value_link`, `references`, `cycle: frame`); `## Context`; `## Goal`
(what the maintainer sees today; after); a `## Frame` section right after `## Goal` (verdict, premise check with
evidence, answers 1 to 6, sizing, scope in/out with a ruling per fold-in, rough build file list, open maintainer
questions); `### Out of scope (for this spec specifically)`. Acceptance Criteria, Failing Tests and Implementation
Context stay as template placeholders. Timeline: frame `[x]` with 2 to 4 lines; `[ ]` lines for design
(`prompts/SPEC-150-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly, no PR.
1. On **go**: `just advance-cycle SPEC-150 design`. On no-go or split: leave `cycle: frame`.
2. In STAGE-063's backlog update the SPEC-150 line with the framed date and size, and tick or re-point any fold-in
   backlog entries you ruled in (leave a pointer, do not delete their text); file any out-of-spec finding as a new
   `- [ ] (not yet written)` entry in the stage it belongs to; re-derive each `**Count:**` with a grep you have just
   run (cross-check; `rtk` can corrupt counts). Edit no other stage.
3. `git commit -s`, message `docs(SPEC-150): frame — two messages tell the truth, --help carries no spec ids`, ending
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
4. `git fetch origin && git rebase origin/main`, `git push origin HEAD:main` (rebase and retry if rejected).
5. `git show --stat HEAD` and `just backlog` to confirm the tooling sees SPEC-150.

### Guardrails
- ⛔ No code, no DEC, no version bump, no tag, no publish. Restore any mutation exactly; `git diff` empty after.
- A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not word-split unquoted
  parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

### Measure your own cost
Transcript JSONL is under `~/.claude/projects/<slug>/` (`<slug>` = absolute working directory with every `/`, `.` and
`_` replaced by `-`). Identify it **by content**: search for `FRAME prompt: two messages tell the truth`. Use Python or
`/usr/bin/grep`. ⚠ **Dedup by `.message.id`**; input, cache_creation and cache_read once per id, MAX `output_tokens`.
Price per component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25, cache_read ×0.10).
Append a `frame` entry to the spec's `cost.sessions` before the final commit.

### Report back (your final message)
First line: `SPEC-150 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- each message reproduced (command, exit, output), with the directory-shape table;
- the id-sweep counts by verb and the other user-facing hits;
- the guard test you recommend, and where an id could still hide;
- replacement text for the high-traffic help lines;
- a ruling per fold-in (in/out, with a reason);
- docs that mirror help lines (PROJ-017 overlaps marked); proposed CHANGELOG text;
- scope in/out, build file list, sizing and verification cost;
- the numbered maintainer questions, each with your recommendation;
- anything out of spec; your measured cost.
