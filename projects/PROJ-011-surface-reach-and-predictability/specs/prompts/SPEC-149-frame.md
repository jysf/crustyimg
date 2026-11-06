# SPEC-149 — FRAME prompt: no flag is accepted and ignored

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-149-frame -- --model opus
```

**Reserved:** **SPEC-149** (STAGE-063, reserved in commit `a9539be`; PROJ-017 knows and has taken SPEC-151). No DEC
reserved. The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 (geometry as recipe steps) and runs at the same time.** It edits
`src/operation/`, `src/recipe/`, `docs/recipes.md`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`,
`tests/common/mod.rs`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`'s Unreleased section, and its next spec (`pad`,
SPEC-151) may add a verb and so touch `src/cli/mod.rs`. This spec's likely files overlap `src/cli/mod.rs`, `README.md`
and `docs/cli-reference.md`: **list every such overlap in the frame** so the orchestrator can coordinate before the
build. Never edit anything under `projects/PROJ-017-geometry-as-recipe-steps/`. If a rebase conflicts in a PROJ-017
file, stop and say so.

## Why this, and why now
STAGE-063 is PROJ-011's last stage (user-facing output that is wrong or misleading, then 0.8.3). SPEC-148 shipped
(`lint` fixes what it names). This spec takes the **accepted-and-ignored flags**, the stage's success criterion
"never accepted and ignored". Full text of each item is in
`projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`:
1. **`--keep-gps`** is a `GlobalArgs` field no verb reads. `convert signed_gps.jpg --format jpeg --keep-gps` exits 0
   and the output has 0 EXIF keys. Promised by `docs/cli-reference.md:21,38`, `README.md:193`, `CHANGELOG.md:711`.
2. **`-v` / `--verbose`** is read by no verb; `-vv` changes no output; no logger is installed, so usvg's own
   `log::warn!`s are never shown.
3. **`responsive --name-template`** is ignored (`src/cli/mod.rs:80-82`, global arg); every other encoding verb honours
   it since SPEC-134.
4. **A missing `--font`** reports "could not read image input" (exit 3) because the font read maps to
   `ImageError::Io` (`src/cli/ops.rs:1369`). Name the font.
5. Sibling, minor: `build --strict` without `--check` exits 0 and does nothing (its help says "With `--check`").
None was driven on 0.8.1 or later; drive each on current `main` first (they may have moved).

## Questions this frame must answer
1. **Reproduce each on `main`.** Command, exit code, stdout/stderr, what the user sees. For `--keep-gps`, test on a
   JPEG and a PNG that carry GPS (exiftool or the repo's own metadata writer; say which). Also list **every other
   `GlobalArgs` field** and prove by `grep` + driving that each is read by at least one verb. Any other
   accepted-and-ignored global flag is a finding (in or out of scope).
2. **Per-flag ruling: implement, refuse (exit 2) or remove.** For each of items 1 to 3 and 5 give the cost and blast
   radius of each option, with evidence, and recommend one:
   - `--keep-gps`: the pixel lane drops all metadata by design (`metadata-not-via-pixel-encode`). Could the flag
     route through the metadata lane (copy-then-clean) for JPEG/PNG, or is that a new feature? Is removing a
     documented global flag a breaking change for scripts that pass it (it exits 0 today)? What would refusing
     break? Who passes it (the Actions, the demo site, the skill: `grep`)?
   - `-v`: what would a minimal honest implementation print, and is it worth it? Removing it breaks scripts that pass `-v`.
   - `responsive --name-template`: honour (needs a `{w}` token; design the token set) vs refuse. How does the
     existing template validation treat unknown tokens?
   - `build --strict` alone: refuse (exit 2, like other flag-combination errors) vs document.
3. **The `--font` message.** Where do the font read and its error mapping live; what should the message say; does
   `watermark --font` in a recipe (`build`) and the wasm path share the code? List every verb that reads a font.
4. **Compatibility.** Is any of these flags covered by a documented contract (`docs/api-contract.md`, SKILL.md, the
   Action's inputs)? Refusing or removing is user-visible in 0.8.3's notes; say what to write (propose CHANGELOG text,
   do not edit it).
5. **Docs.** Every doc, help string and skill sentence that mentions each flag; list each that changes, and mark the
   PROJ-017-owned files.
6. **Size and scope.** Expect [S]–[M]. State verification cost, including how verify proves a refused flag really
   exits 2 on every verb and an implemented flag really changes the output (HANDOFF traps 11 to 17 apply). Give the
   scope boundary and where each out item lives. If the rulings differ so much that two specs are cleaner, say
   SPLIT and how; do not create the second id yourself.

## Questions only the maintainer can answer
Present them as a numbered list at the end of your report, **one per flag**, each with your recommendation and the
one-line consequence of each choice. The orchestrator relays the answers; the spec stays at `cycle: frame` with a
"rulings pending" line if they are not known by the time you finish (go/no-go can still be GO with the rulings open).

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
4. `src/cli/mod.rs` (`GlobalArgs`), `src/cli/common.rs`, `src/cli/ops.rs`, `docs/cli-reference.md`, `README.md`,
   `skills/crustyimg/SKILL.md` (read only), and DEC-034/DEC-040/SPEC-134's template decision (find them with
   `./scripts/decisions-audit.sh`).

### What a good frame does here
- **Drive it before you describe it.** Questions 1 to 3 are measurements.
- ⛔ Run nothing that publishes or tags. No `git tag`, no `npm publish`, no `gh workflow run`.
- Rough build file list (paths) with the PROJ-017 overlaps marked.

### The id is reserved: use exactly SPEC-149
```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-149' "$b" -- . ; done
```
Expect only STAGE-063, PROJ-017's brief, SPEC-151's frame prompt and this prompt. Then
`just new-spec "<title>" STAGE-063 PROJ-011`; `just new-spec` scans only the working tree and may allocate another
number; if it is not SPEC-149, `git mv` the spec and timeline files and fix the id inside them. Then
`just advance-cycle SPEC-149 frame`. Write no DEC at frame.

### What to write into the stub
Front matter (`type`, `priority`, `complexity`, `value_link`, `references`, `cycle: frame`); `## Context`; `## Goal`
(what the maintainer sees today; after); a `## Frame` section right after `## Goal` (verdict, premise check with
evidence, answers 1 to 6, sizing, scope in/out, rough build file list, open maintainer questions);
`### Out of scope (for this spec specifically)`. Acceptance Criteria, Failing Tests and Implementation Context stay as
template placeholders. Timeline: frame `[x]` with 2 to 4 lines; `[ ]` lines for design
(`prompts/SPEC-149-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly, no PR.
1. On **go**: `just advance-cycle SPEC-149 design`. On no-go or split: leave `cycle: frame`.
2. In STAGE-063's backlog update the SPEC-149 line with the framed date and size; file any out-of-spec finding as a
   new `- [ ] (not yet written)` entry in the stage it belongs to; re-derive each `**Count:**` with a grep you have
   just run (cross-check; `rtk` can corrupt counts). Edit no other stage.
3. `git commit -s`, message `docs(SPEC-149): frame — no flag is accepted and ignored`, ending
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
4. `git fetch origin && git rebase origin/main`, `git push origin HEAD:main` (rebase and retry if rejected).
5. `git show --stat HEAD` and `just backlog` to confirm the tooling sees SPEC-149.

### Guardrails
- ⛔ No code, no DEC, no version bump, no tag, no publish. Restore any mutation exactly; `git diff` empty after.
- A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not word-split unquoted
  parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`.
- **Budget ~45 exchanges.**

### Measure your own cost
Transcript JSONL is under `~/.claude/projects/<slug>/` (`<slug>` = absolute working directory with every `/`, `.` and
`_` replaced by `-`). Identify it **by content**: search for `FRAME prompt: no flag is accepted and ignored`. Use
Python or `/usr/bin/grep`. ⚠ **Dedup by `.message.id`**; input, cache_creation and cache_read once per id, MAX
`output_tokens`. Price per component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25,
cache_read ×0.10). Append a `frame` entry to the spec's `cost.sessions` before the final commit.

### Report back (your final message)
First line: `SPEC-149 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- each flag reproduced (command, exit, output), and the sweep of every other global flag;
- per flag: options with evidence, your recommendation;
- the font-message location and every verb that reads a font;
- compatibility and docs list, with the PROJ-017 overlaps marked;
- proposed CHANGELOG text;
- scope in/out, build file list, sizing and verification cost;
- the numbered maintainer questions, one per flag, each with your recommendation;
- anything out of spec; your measured cost.
