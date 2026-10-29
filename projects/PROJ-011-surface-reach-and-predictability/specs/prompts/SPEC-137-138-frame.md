# SPEC-137 + SPEC-138 — FRAME prompt: 0.8.1's two silent wrong-output bugs

Cycle: **frame**, for two specs in one session. They are unrelated in code but share a reason:
each one gives the user wrong output at exit 0 with no warning. Fresh CLI session, own worktree,
**Opus**:

```
just wt docs/spec-137-138-frame -- --model opus
```

**Reserved:**
- **SPEC-137:** SVG text in a font family other than Go draws nothing.
- **SPEC-138:** `build` accepts `--format` and ignores it.
- **DEC-107** and **DEC-108** are reserved for the design sessions. Either may be released
  unused if a change only amends an existing DEC.

Your items are two entries in STAGE-054's Spec Backlog, both filed `(not yet written)`:
- "**An SVG font family other than Go draws no text.**" Filed by PR #211's verify, 2026-10-28.
- "**`build --format` is accepted and silently ignored.**" Filed by the 0.8.0 release-notes verify,
  2026-10-27.

Read both in full. Touch no other item.

## Why these, and why now
- 0.8.0 shipped on 2026-10-28. The maintainer chose these two to open 0.8.1 (2026-10-28), ahead
  of new features: each one is a silent wrong output in a shipped verb.
- Both are [S] on paper. Keep them that way. Anything else you find goes to a STAGE-054 backlog
  entry (HANDOFF rule 6), not into these specs.

## SPEC-137 — what is known (the verify's evidence; reproduce it, don't trust it)
- `<text font-family="sans-serif">`, `serif`, `Helvetica`, `Zapfino` or
  `"Helvetica, sans-serif"` renders **zero ink** and exits 0 with no warning, on released 0.7.1,
  on 0.8.0 (resvg 0.47) and on resvg 0.49.
- No family, `Go`, or a list that contains `Go` renders.
- Installed system fonts are **not** read, as designed: only the bundled Go font is loaded
  (DEC-045, DEC-054).
- The code: `hardened_options()` in `src/image/svg.rs` (about lines 177-194) loads only
  `FONT_DATA` into a fresh `fontdb::Database` and sets `opt.font_family = FONT_FAMILY`. That sets
  the default for text with no family; it does nothing for a named or generic family.
- The likely fix is to point fontdb's generic families (`sans-serif`, `serif`, `monospace`,
  `cursive`, `fantasy`) at Go, plus a fallback so an unknown named family uses Go too. Confirm or
  refute this against usvg 0.49's font resolution (`usvg::FontResolver`, `fontdb::Database`).

## SPEC-138 — what is known (reproduce it, don't trust it)
- `build --format png` exits 0 and writes the target's own format, with or without a name
  extension or a recipe `format`.
- `--format` is a **global** flag (`src/cli/mod.rs` about :88-90), so `build --help` lists it as
  "Force output format". `src/cli/build.rs` never reads it.
- `apply --recipe` honours it: `--format`, then the `-o` or template extension, then the recipe's
  `format`, then the source format.
- 0.8.0's docs say: on `build`, the target's name or the recipe's `format` sets the format, and
  `--format` has no effect.

## Questions this frame must answer

### SPEC-137
1. **Premise.** On a release build of current `main`, render each family above, plus no family
   and `Go`. Count ink with an independent tool (ImageMagick or Python, never crustyimg). Negative
   control: the no-family SVG must show non-zero ink. Also check the browser build:
   `src/wasm.rs`'s SVG path uses the same `hardened_options()`. Cite `file:line`, and drive it
   only if `just wasm-test` makes that cheap.
2. **Which families, and what the fix covers.** Make a table of each family form (generic,
   named-installed, named-unknown, a list, a quoted list, `font-weight="bold"`,
   `font-style="italic"`), showing today's ink, what the fix would draw, and how usvg resolves it.
   Bold and italic matter: only one Go face is bundled, so say what usvg does when asked for a
   weight it doesn't have.
3. **The rule (a maintainer question).** Recommend one:
   - every family falls back to Go, silently;
   - every family falls back to Go, with a warning naming the family;
   - fall back, but refuse an SVG whose text cannot render at all.

   SVGs name fonts the bundled set will never have, so silence vs a warning is the real choice.
   Check how `lint` and `info` treat SVGs, in case a warning there is the better home.
4. **Bytes.** An SVG with no family, or with `Go`, must be **byte-identical** to `main`'s
   output. Name the check that design must require.

### SPEC-138
5. **Premise.** Reproduce on a release build of `main`, using three targets: one with a name
   extension, one with a recipe `format`, and one with neither. Check on released 0.7.1 too
   (`/opt/homebrew/bin/crustyimg` is now 0.8.0 after `brew upgrade`; get 0.7.1's release asset
   from the GitHub Release if you need it, or say 0.7.1 was not driven).
6. **Sweep the global flags on `build`, then stop.** List every global flag (`src/cli/mod.rs`) and
   check whether `build` honours it, ignores it, or refuses it: `--format`, `--quality`,
   `--out-dir`, `-j`, `--json`, `--name-template`, `-y` and any others. Every flag that is
   accepted and silently ignored is the same bug. Recommend whether it belongs in this spec or in
   a backlog entry. Don't sweep other verbs.
7. **Honour or refuse (a maintainer question).** Recommend one, with evidence:
   - **honour:** `--format` wins over the target name and the recipe's `format`, as on `apply`.
     The cache key and the lockfile must then include it. Say what `build --check` / `--frozen`
     does when `--format` changes.
   - **refuse:** exit 2, with a message naming the flag and saying how to set the format
     (target name or recipe).

   Weigh it against `build`'s promise: a build is declared in its manifest, so a CLI override
   makes the same manifest produce different bytes. Read DEC-057 (build) and DEC-105 (template
   pins the format), and the build-cache DEC.
8. **Bytes.** A `build` without `--format` must produce byte-identical output and the same cache
   keys as `main`. Name the check.

### Both
9. **Size.** Each should be **[S]**. If either grows, recommend the cut.
10. **One spec or two at design.** They're framed together to save a session. Say whether they
    should stay separate through design and build. The orchestrator expects separate, since they
    touch different files.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub for each. You do **not** write acceptance criteria, failing tests or Implementation
Context. Separate design, build and verify sessions follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for". Binding.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`:
   your two items in full.
4. SPEC-137: DEC-045 and DEC-054 (the bundled font, SVG text), and DEC-034 (SVG hardening).
   SPEC-138: DEC-057, DEC-105 and the build-cache DEC (find it with `./scripts/decisions-audit.sh`
   or grep `decisions/`).
5. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Reproduce on a release build of
  `main` (`cargo build --release`, in a fresh `CARGO_TARGET_DIR` that you delete at the end) with
  a negative control. Record the exact command and output. **Measure the scenario, not the
  control.**
- **Size the verification, not just the code** (AGENTS §2). Count each AC matrix.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), for each spec.
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Ids are reserved — use exactly these
`just new-spec` allocates ids by scanning the working tree. Run
`just new-spec "<title>" STAGE-054 PROJ-011` once per spec. If it allocates anything other than
SPEC-137 then SPEC-138, `git mv` the files (spec and timeline) and fix the id inside them. Then
`just advance-cycle SPEC-137 frame` and `just advance-cycle SPEC-138 frame`. Write no DEC at frame.

### What to write into each stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (decisions and related specs), `cycle: frame`.
- `## Context` and `## Goal`: what the user sees today (your reproduction), and what they see when
  it ships, in the user's terms (the command, the stderr line, the exit code).
- A `## Frame` section directly after `## Goal`: verdict (go / no-go / split), premise check with
  evidence, the table or sweep, sizing and why, scope in/out, rough build file list, open
  maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary; add `[ ]` lines for design
  (`prompts/SPEC-13N-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), no PR.
1. On **go**: `just advance-cycle SPEC-13N design`. On no-go or split: leave `cycle: frame`.
2. In STAGE-054's backlog, change **only your two items'** `- [ ] (not yet written)` markers to
   `- [ ] **SPEC-137** (framed <date>)` and `- [ ] **SPEC-138** (framed <date>)`. File any
   out-of-spec finding as a new `- [ ] (not yet written)` entry at the end of STAGE-054's
   backlog, and update its `**Count:**` line with a grep you have just run.
3. `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-137, SPEC-138): frame the two 0.8.1 silent-output bugs`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed (`git show --stat HEAD`, and the STAGE-054 diff), and run
   `just backlog` to confirm the tooling sees both specs.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag.
- `cargo test` is not needed at frame.
- A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~60 exchanges** for both.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** `grep -l "SPEC-137 + SPEC-138 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read
×0.10. Split the session's cost evenly between the two specs, and append a `frame` entry to each
spec's `cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate
anchors (note "half of a shared frame session"), before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First lines: `SPEC-137 frame: GO | NO-GO | SPLIT` and `SPEC-138 frame: GO | NO-GO | SPLIT`, then
the commit SHA on `main`. Then, short, per spec:
- the premise check (confirmed / wrong, with one line of evidence per row);
- the family table (SPEC-137) or the global-flag sweep (SPEC-138);
- size and why; scope in/out;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings and where you filed them.

Then your measured cost, total and per spec.
