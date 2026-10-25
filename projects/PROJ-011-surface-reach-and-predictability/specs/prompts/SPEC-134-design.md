# SPEC-134 — DESIGN prompt: a `--name-template` extension pins the output format

Cycle: **design**. Fresh CLI session, own worktree (`just wt docs/spec-134-design -- --model opus`).
Starts from the stub the frame session pushed to `main` (ee96747). **Reserved: SPEC-134, DEC-105.**
STAGE-050 backlog item: "[S] **A literal extension in `--name-template` does not pin the output
format**".

**Read the stub's `## Frame` section first** — it holds the reproduction across every verb and is
your binding scope. This prompt was rewritten 2026-10-25 to match it and to carry two new rulings.

## What the frame found
The bug is wider than the backlog: **every verb that encodes ignores the template's extension, at
one input and at many** (`resize`, `thumbnail`, `auto-orient`, `apply` single + multi, plain-recipe
`build`), writing the source format into a `.png`/`.webp` name. `web`, `optimize` and `apply
--recipe web` write **AVIF** into `.png`/`.webp` names while stderr says "jpeg → avif". And `apply`
and `build` disagree today: a `build` target with `recipe = "web"`, `name = "{stem}.png"` already
writes a real PNG.

## ⚠ Correction: DEC-087 is not reversed — it is EXTENDED
The earlier prompt (and STAGE-050, SPEC-126, SPEC-127) said DEC-087 holds the rule "a template's
literal extension names the file, it does not pin the format". **It does not.** DEC-087 is titled
"`build`'s name template **is** the format pin" (a literal extension pins and skips the AVIF-aware
decision; `{ext}` preserves). The "names the file only" rule exists only in
`docs/api-contract.md` (~line 490), and no DEC contains it. So:
- **DEC-105 extends DEC-087's rule to every verb that encodes.** It does not supersede DEC-087. Set
  `supersedes` / links accordingly, and say in DEC-105 that the contrary sentence in the API
  contract was undocumented by any DEC.
- `docs/api-contract.md` changes in the same build.
- Correct the misattribution in the STAGE-050 entry for your item (only your item).

## Rulings (settled — do not re-open)
1. **Honour the template (maintainer, 2026-10-24):** a recognised extension in `--name-template`
   pins the output format, exactly as a recognised `-o` extension does, on every verb that encodes,
   at both arities, and on plain-recipe `build` targets.
2. **(2026-10-25) On `web`, `optimize` and `apply --recipe web`, a recognised template extension
   pins the format and skips the automatic choice**, as `-o` and `build` (DEC-087) already do.
   `{ext}` or no extension keeps the automatic choice.
3. **(2026-10-25) `meta` refuses, exit 4, before writing anything**, when the template names a
   different format than the source: the metadata lane cannot convert without re-encoding pixels
   (constraint `metadata-not-via-pixel-encode`). The message says to convert first. Same format →
   unchanged.

## Design calls that are yours
- **One helper, about six call sites.** `build` already has `target_format_plan`
  (`src/cli/build.rs:63`). ⚠ Reusing it as-is breaks two must-not-change cases: plain `build` with
  `name = "{stem}"` or `"{stem}.txt"` exits 0 today and would exit 4. Design the shared helper so
  those stay as they are.
- **Unrecognised or missing extension.** `-o` silently falls through on an unrecognised extension,
  while `build`'s terminal-`optimize` template exits 4. Pick one behaviour per surface, state it
  per surface, and keep `{name}`, `{ext}` and extensionless templates as they are.
- **`{ext}` templates must stay byte-identical** to `main`. Pin it with a test.
- **Precedence**, three cases: template vs `--format`, vs the recipe's `format` (SPEC-127 /
  DEC-099), and vs `-o`. Match the `-o` precedent and cite it; say whether a disagreement is an
  error or which wins.
- **`build`'s cache key changes** when a plain target moves from preserve to pinned. Pin that
  consequence with a test, and say how it rides 0.8.0's single lockfile migration (no
  `CACHE_SCHEMA_VERSION` bump unless you can show one is needed).
- **One table-driven test** checks the real thing (the output's magic bytes match the template's
  extension) across 9 encoding verbs × {1, N} inputs, plus the precedence and must-not-change
  rows. Count the rows against the frame's roster, and make the test assert its row count.
- **File `responsive`** as a new STAGE-050 backlog item: it accepts `--name-template` and silently
  ignores it (exit 0); it should refuse (exit 2) or honour it. Do not fix it here.
- Cross-reference PROJ-013 / STAGE-037's `-o` item (pinned `web` quality drops 85→80 and loses its
  report): note it, do not absorb it.

## Overlap the orchestrator needs to know about
- **SPEC-133** (watermark damage rule) shares ~7 files with you (`src/cli/{ops,optimize,common,
  build}.rs`, tests, docs). The two **will not build in parallel**. List, for each shared CLI
  file, the functions you touch, so the order can be chosen and the second build's rebase stays
  small.
- **SPEC-131** (strict recipe params, building now) stays out of `src/cli/**`; you can build in
  parallel with it. Shared only by append: `CHANGELOG.md`, `docs/api-contract.md`.
- **SPEC-132** (`build --watch`) edits `watch_impl` in `src/cli/build.rs`; name the `build.rs`
  functions you touch.
Frame's estimated file list: `src/cli/{ops,optimize,common,build,mod}.rs`, `src/sink/mod.rs`;
tests `tests/{apply_batch,build,build_cache,optimize,sink}.rs` + new
`tests/name_template_pins_format.rs`; docs `docs/api-contract.md`, `CHANGELOG.md`; new DEC-105.

## DECs to read
**DEC-087** (the rule you extend — read it, do not trust a summary of it), DEC-098, DEC-099,
DEC-064, DEC-015 (exit codes), DEC-058/DEC-101 (cache key), and whatever
`./scripts/decisions-audit.sh` reports for the files you list. Constraint
`metadata-not-via-pixel-encode` in `/guidance/constraints.yaml`.

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
recency**: `grep -l "SPEC-134 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by `.message.id`**: Claude Code writes one
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
