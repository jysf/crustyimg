# SPEC-131 — FRAME prompt: typed per-operation parameter structs

Cycle: **frame**. Fresh CLI session, own worktree (`claude -w spec-131-frame`). **Reserved:
SPEC-131** (DEC-102 is reserved for the design session, not you).
STAGE-050 backlog item: "[M] **Typed per-operation parameter structs**" (the last entry in the
Spec Backlog).

## What to frame
`OperationParams` is a `BTreeMap<String, toml::Value>` read through hand-rolled getters
(`src/operation/mod.rs`, ~39-71), so a malformed recipe is found when an op runs, on input 400 of
500. The intended outcome: a malformed recipe fails when it is loaded, naming the bad key and step,
on every surface that loads a recipe (`apply --recipe`, `build`, the wasm `validate()` in
`src/wasm.rs`).

## Questions this frame must answer
- **Premise:** demonstrate today's late failure on a release build — a recipe with a misspelled or
  wrong-typed param over a small batch, showing which input it fails on and what stderr says.
  Also check: does an *unknown* key fail at all today, or is it silently ignored? That changes the
  scope materially.
- **Size.** Backlog says M. Count the registered ops and their params, and the verb surfaces that
  load a recipe, and size the *verification*: every op × {unknown key, wrong type, out of range} ×
  every surface is a big matrix. Is it still M, or L that must split (e.g. by op family, or
  "parse-time rejection" vs "typed structs internally")? Recommend; do not write two stubs.
- **DEC-005 round-trip risk** in one paragraph: `to_toml()` feeds the build cache key and lockfile
  (DEC-058, DEC-101). Note whether byte-stable re-emission looks achievable — design settles it.
- **Overlap:** this touches every op, so it likely collides with SPEC-133 (watermark) and possibly
  SPEC-134 (CLI naming) at build. Your rough file list decides build order.

---

## Rules every 0.8.0 frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly-sized, correctly-scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context — a
separate, fresh design session does that from your stub, then separate sessions build and verify.
The orchestrator (another session) only sequences; it does not frame or design.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
You should be in your own worktree (`claude -w <name>`). Run `git fetch origin`, then
`git rebase origin/main` (a fresh worktree has nothing to conflict). If `git status` shows you are
in the maintainer's main checkout rather than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` — "Read first" and "Traps this
   wave paid for". Binding.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-050-recipe-reach.md` — your
   item's backlog entry in full. A maintainer ruling there is settled; do not re-open it.
4. `projects/PROJ-011-surface-reach-and-predictability/brief.md`, `/guidance/constraints.yaml`.
5. The design prompt that will follow you: `specs/prompts/SPEC-NNN-design.md` (your id). Your stub
   must hand that session what it needs; flag anything in it your frame finds wrong.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** SPEC-128's backlog title was wrong
  about the code — `Watermark` was already an `Operation`. Read the code the item names and cite
  `file:line`. If the item is a bug, reproduce it on a release build of `main` (`cargo build
  --release`) with a negative control (a self-diff or unchanged-input run that shows 0), and record
  the exact command and output. **Measure the scenario, not the control**: the row that carries
  the argument must use the feature it is about.
- **Size the verification, not just the code** (AGENTS §2). If the AC matrix will be L-shaped
  (many verbs × many modes), the spec is L: recommend the split, and **do not write two stubs** —
  say so in your report and the orchestrator decides.
- **Scope boundary**: state what is in, what is explicitly out, and where each out item lives (an
  existing backlog entry, or "needs filing" with a one-line description).
- **Rough build file list** (paths) — the orchestrator uses it now to decide which builds can run
  in parallel. Best estimate; design refines it.
- **Questions only the maintainer can answer**: one line each, with your recommendation. Do not ask
  what the code, a DEC or a ruling already answers.

### Ids are reserved — use exactly these
`just new-spec` allocates ids by scanning the working tree, and other sessions are running in
parallel, so it may hand you the wrong id. Run `just new-spec "<title>" STAGE-050 PROJ-011`; if it
allocates anything other than your reserved SPEC id, `git mv` both files (spec + timeline) to the
reserved id and fix the id inside them. Then `just advance-cycle SPEC-NNN frame`. Do not write any
DEC at frame — the reserved DEC id is the design session's.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (decisions/related specs you know apply), `cycle: frame`.
- `## Context` and `## Goal` — what the user sees today (with your reproduction), and what they see
  when this ships, in the user's terms (the TOML, the stderr line, the exit code).
- A new section `## Frame` directly after `## Goal`: verdict (go / no-go / split), premise check
  with evidence, sizing and why, scope in/out, rough build file list, open maintainer questions.
- `### Out of scope (for this spec specifically)` filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary; add `[ ]` lines for design
  (`prompts/SPEC-NNN-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), no PR.
1. On **go**: `just advance-cycle SPEC-NNN design`. On no-go or split: leave `cycle: frame`.
2. In the STAGE-050 backlog, change **only your item's** `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-NNN** (framed <date>)`. Touch no other item — other sessions edit their own
   entries in the same file.
3. `git commit -s` — **DCO sign-off is mandatory**; the "DCO sign-off check" fails any PR carrying
   an unsigned commit, and it has bitten this repo four times. Message: `docs(SPEC-NNN): frame <short title>`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. If the rebase
   conflicts in STAGE-050, keep every other session's edits and re-apply only yours.
5. Read back what you pushed (`git show --stat HEAD`, and the STAGE-050 diff) and check it is what
   you meant, not just that the push succeeded.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/` and `.` replaced by `-`. **Identify it by content, not
recency** (another session may be newer): `grep -l "SPEC-131 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.
⚠ **Dedup by `.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing
an id repeat identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens`
while `output_tokens` grows — take those three once per id and the **MAX** `output_tokens`. Summing
every line over-counts ~2×; taking the first line's output under-counts ~11×; both mistakes are on
record. Price each component at the model that actually ran (Opus $5/$25 per MTok, Sonnet $3/$15;
cache_creation ×1.25 the input rate, cache_read ×0.10), never a flat rate on the total. Append a
`frame` entry to the spec's `cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd`
and the rate anchors, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-NNN frame: GO | NO-GO | SPLIT — <commit SHA on main>`. Then, short: the premise
check (confirmed / wrong, with the one-line evidence); size and why; scope in/out; the rough build
file list; maintainer questions with your recommendation; anything wrong in the design prompt;
your measured cost.
