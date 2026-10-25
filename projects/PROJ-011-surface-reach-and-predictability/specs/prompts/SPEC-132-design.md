# SPEC-132 — DESIGN prompt: `build --watch` watches recipe asset paths

Cycle: **design**. Fresh CLI session, own worktree (`claude -w spec-132-design`). Starts from
the stub the frame session pushed to `main`. **Reserved: SPEC-132, DEC-103** (only if a DEC is
warranted; likely none for an S).
STAGE-050 backlog item: "[S] ⚡ **`build --watch` never watches a recipe's asset paths.**"

## What this spec is for
`WatchSet` (`src/build/watch.rs`, ~106-114) watches the manifest's directory (shallow), each
source root (recursive) and each recipe's directory (shallow) — never the file a recipe step's
`image` / `font` key names. With the overlay at `brand/logo.png` and the recipe at
`recipes/r.toml`, editing a source rebuilds; editing `brand/logo.png` triggers nothing, silently.
SPEC-129 made the cache correct once a build runs; this spec makes `--watch` start one.

## Design calls that are yours
- **Watch the file or its directory?** Editors often save by rename (write temp + rename), which a
  file-level watch can miss. Drive it.
- **When the asset set changes** — a recipe edit points `image` at a new file, or an asset is
  deleted / does not exist yet. Is the `WatchSet` recomputed after each rebuild?
- Reuse the asset walk `build` already does (`resolve_recipe_assets` / `registry.asset_keys`,
  `src/cli/common.rs`) rather than a second definition of "which keys are assets".
- ⚠ **Configuration-dependent, so easy to fake-pass:** if the overlay sits inside a source root or
  beside its recipe, today's watch already catches it by accident. The failing tests must put the
  asset **outside** every currently-watched path, and the spec must say so.
- **Count the nouns:** `image` and `font` are two assets → at least one test each (the exact trap
  SPEC-128 fell into).
- Prefer a test that does not depend on filesystem-event timing where possible (e.g. assert the
  computed watch set), plus one end-to-end watch test if the existing watch tests show a reliable
  pattern — read `src/build/watch.rs`'s tests and say which pattern you chose and why.

## Overlap the orchestrator needs to know about
Three other STAGE-050 specs (SPEC-131 typed params, SPEC-133 watermark damage rule, SPEC-134
name-template) touch `src/cli/**` and `src/operation/**`. If yours can stay inside
`src/build/watch.rs` (plus tests) it builds in parallel with them; if it must touch
`src/cli/build.rs` or `src/cli/common.rs`, list it — that decides the order.

## DECs to read
DEC-100 (the asset seam), DEC-101, DEC-058, and any DEC governing `src/build/watch.rs`
(`./scripts/decisions-audit.sh` lists them).

---

## Rules every 0.8.0 design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. The frame stub for your spec must now be in
`projects/PROJ-011-surface-reach-and-predictability/specs/` at your reserved id with
`cycle: design`. If it is missing or still `cycle: frame`, stop and report — do not create it. A separate session builds it
from your spec alone, and another verifies it, so everything the builder needs must be in the spec
(AGENTS §15). The orchestrator (another session) sequences builds; it does not design.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` — "Read first" and "Traps this
   wave paid for". Those traps are binding here.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-050-recipe-reach.md` — your
   item's backlog entry holds measured evidence and maintainer rulings. A ruling is settled; do not
   re-open it. An open question the ruling left to the spec is yours to settle.
4. `projects/PROJ-011-surface-reach-and-predictability/brief.md`, `/guidance/constraints.yaml`,
   and every DEC this prompt names.
5. A recent shipped spec as the shape to match:
   `projects/PROJ-011-surface-reach-and-predictability/specs/done/SPEC-129-build-cache-hashes-watermark-asset-content.md`.

### How to design well here
- **Read the code before writing the spec.** SPEC-128's backlog title was wrong about the code; a
  design that only read the backlog item would have specced the wrong thing. Cite `file:line`.
- **Drive the current behaviour** on a release build of `main` before you describe it. **Measure
  the scenario, not the control**: the row that carries an argument must use the feature the
  argument is about. Give every measurement a negative control (self-diff = 0) and say what it is.
- **Ask what the user actually sees** — the TOML they write, the stderr line, the exit code.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing (SPEC-128 shipped "overlay/font" with only overlay tested).
- **Size the verification, not just the code.** If the AC matrix is L-shaped, the spec is L —
  then split it (AGENTS §2) and tell the orchestrator before writing two specs.
- Every AC names the verb surface it applies to (AGENTS §15, "An AC may not transfer between
  verbs"); the Failing Tests section lists each test by name and says why it is RED on `main` today.
- List, in Implementation Context, **every source file the build will touch** (best estimate, by
  path). The orchestrator uses this list to keep two concurrent builds off the same files.
- Name the negative control per independent condition (AGENTS §15) so verify knows what to revert.
- **Everything in PROJ-011 ships as ONE release (0.8.0).** Do not plan a version bump, a release,
  or a `CACHE_SCHEMA_VERSION` change unless your item genuinely needs one — and if it does, say so
  as a design call with its reason.
- Comments and user-facing text read behaviour-first, with no spec/DEC ids in them.

### Ids are reserved — use exactly these
The frame session already created your spec and timeline at the reserved SPEC id; do not run
`just new-spec`. Its `## Frame` section (verdict, sizing, scope in/out, rough file list) is your
binding scope. If you find it wrong, say so in your report rather than silently widening or
narrowing it. Fill Acceptance Criteria, Failing Tests and Implementation Context; keep `## Frame`.
Highest DEC on `main` is DEC-101. Write any DEC with the **block-list** `affected_scope` form
(an inline array is silently dropped by `scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - src/cli/ops.rs
```
Whether a DEC is written at design or at build is your call; say which in the spec.

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR. You are in your own worktree:
1. Set the spec's `cycle:` to `build` (`just advance-cycle SPEC-NNN build`).
2. In the STAGE-050 backlog, change **only your item's** marker (the frame left it as
   `- [ ] **SPEC-NNN** (framed <date>)`) to `(designed <date>)`. Touch no other item — other design sessions are
   editing their own entries in the same file right now.
3. `git commit -s` (**DCO sign-off is mandatory** — CI fails without it; it has bitten this repo
   four times), conventional message `docs(SPEC-NNN): design <short title>`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. If the rebase
   conflicts in STAGE-050, keep every other session's edits and re-apply only yours.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/` and `.` replaced by `-`. **Identify it by content, not
recency**: `grep -l "SPEC-132 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by `.message.id`**: Claude Code writes one
JSONL line per *content block*, and lines sharing an id repeat identical `input_tokens` /
`cache_creation_input_tokens` / `cache_read_input_tokens` while `output_tokens` grows — take those
three once per id and the **MAX** output. Summing every line over-counts ~2×; taking the first
line's output under-counts ~11×; both mistakes are on record. Price per component at the model you
actually ran (Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 input, cache_read ×0.10
input), never a flat rate. Append a `design` entry to the spec's `cost.sessions` with real numbers
and a `tokens_breakdown`, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: spec id, path, and the commit SHA on `main`. Then: AC count and failing-test count;
each design call and how you settled it; the file list the build will touch; anything that needs a
**maintainer** ruling (one line each, with your recommendation); your measured cost. Keep it short —
the orchestrator reads the spec itself.
