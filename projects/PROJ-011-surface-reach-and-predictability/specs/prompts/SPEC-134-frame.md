# SPEC-134 — FRAME prompt: a `--name-template` extension pins the output format

Cycle: **frame**. Fresh CLI session, own worktree (`just wt docs/spec-134-frame`). **Reserved:
SPEC-134** (DEC-105, which amends DEC-087, is reserved for the design session).
STAGE-050 backlog item: "[S] **A literal extension in `--name-template` does not pin the output
format**".

## The ruling (settled — do not re-open)
Honour the template (maintainer, 2026-10-24): a recognised extension in `--name-template` pins the
output format exactly as a recognised `-o` extension already does. This reverses the rule in
`docs/api-contract.md` / DEC-087, so the spec amends DEC-087.

## Questions this frame must answer
- **Premise:** reproduce the stage table's two rows on a release build of `main` (JPEG source,
  `apply` single input, `{stem}_w.png` → JPEG bytes in a `.png` file), and check multi-input
  `apply` and one other verb the same way. Identify bytes with a sniff of the magic, not the name.
- **Reach:** list every verb that takes `--name-template` (grep `src/cli/**`, `src/sink/mod.rs`),
  and whether `build`'s manifest carries one. That roster sets the size.
- **The `web` / `optimize` batch case** is the design call the ruling left open (a pin skips the
  automatic format choice). Drive today's behaviour for `web --out-dir d/ --name-template
  '{stem}.webp'` so design starts from evidence; do not decide it here.
- **Size:** confirm S or say why not.

---

## Rules every 0.8.0 frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly-sized, correctly-scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context — a
separate, fresh design session does that from your stub, then separate sessions build and verify.
The orchestrator (another session) only sequences; it does not frame or design.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
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
recency** (another session may be newer): `grep -l "SPEC-134 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.
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
