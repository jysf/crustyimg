# SPEC-148 — FRAME prompt: `lint` fixes what it names

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-148-frame -- --model opus
```

**Reserved:** **SPEC-148** (STAGE-063, reserved in commit `a9539be`). No DEC reserved; design reserves one if it needs it.
The maintainer pastes this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 (geometry as recipe steps) and runs at the same time.** It edits
`src/operation/`, `src/recipe/`, `docs/recipes.md`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`
and `skills/crustyimg/SKILL.md`, `CHANGELOG.md`'s Unreleased section. This spec must not touch them (if a fix
needs one, say so in the frame instead), and you must never edit anything under
`projects/PROJ-017-geometry-as-recipe-steps/`. If a rebase conflicts in a PROJ-017 file, stop and say so.

## Why this, and why now
STAGE-063 is PROJ-011's last stage: user-facing output that is wrong or misleading, then release 0.8.3.
This spec takes the two `lint` items, framed together because both are `lint` rules:
1. **`lint`'s suggested `fix:` commands write the image to stdout.** Run exactly as printed, each writes the
   encoded image to stdout even on a TTY, so the file is not fixed. The form that works is `-o <f> -y`.
2. **`privacy/gps-metadata-leak` fires on a GPS block with no coordinates.** A Nikon P1100 JPEG writes a GPS IFD
   holding only `GPSVersionID`; `lint` flags it as a location leak.
Both came from the `jysf/crustyimg-example` demo site (2026-10-29). Full text of both is in
`projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`.
Neither was driven on 0.8.1 or later. Related: PROJ-012 (`proposed`, "lint can fix what it names") is a
different, larger idea (animation, ICC); say whether any of its text overlaps this spec.

## Questions this frame must answer
1. **Reproduce both on current `main`.** For (1): run `lint` on a fixture that triggers every rule that prints a
   `fix:`, copy each printed line verbatim, run it in a scratch copy, and report: stdout bytes, whether the file
   changed, whether the finding cleared. List every rule and its fix line (`grep` the source, then drive).
   For (2): build a JPEG with a GPS IFD of only `GPSVersionID` (the repo's own metadata writer or `exiftool` if
   installed; say which) and one with real coordinates; report `lint`'s result on each. Is the P1100 behaviour
   reproducible from a repo fixture, or does the spec need to add one?
2. **Fix (1): which option?** Carry `-o <f> -y` in the printed fix, vs. make the encoding verbs refuse to write
   binary to a TTY. Check blast radius: every other verb, scripts that pipe stdout on purpose, the skill/docs. A
   TTY refusal is a behaviour change for all verbs; a printed fix is local to `lint`. Recommend one, with the
   evidence. Also: does `-o <f> -y` overwrite the source safely (atomic write? what if the format changes with
   the verb)? Does the fix line need quoting for paths with spaces?
3. **Does the fix actually clear the finding?** For each rule: after running the corrected fix, re-run `lint` on
   the output. Any rule whose fix leaves the finding in place (for example a `resize` that needs a size the rule
   cannot know) is its own finding: report it, and say whether it is in or out of this spec.
4. **Fix (2): the rule.** What does the rule key on today, and which tags make a GPS block a location leak
   (`GPSLatitude`+`GPSLongitude`, and their Ref tags; what about altitude, timestamp or `GPSMapDatum` alone)?
   Recommend the exact predicate. Keep a positive control: real coordinates still flag. Check the same predicate
   in `meta clean --gps` and in anything else that decides "has GPS" (`grep` for the tag names), so they do not
   disagree.
5. **Release implications.** Both change `lint` output that scripts may parse. Is `fix:` text or the finding set
   covered by a documented contract (`docs/api-contract.md`, the GitHub Action's output, the demo site)? A
   change there is user-visible in 0.8.3's notes; say what to write (a CHANGELOG line is PROJ-017-adjacent: the
   Unreleased section is shared, so propose the text, do not edit it).
6. **Docs.** Every doc or help string that shows a `fix:` line or lists the GPS rule; list each sentence that
   changes (outside the PROJ-017 files above).
7. **Size and scope.** Expect [S]. State the verification cost, including how verify proves a printed fix, run
   verbatim, clears its finding (HANDOFF traps 11 to 15 apply: a test that cannot fail, mutation locations).
   Give the scope boundary and where each out item lives.

---

## Rules every frame session follows

**You are the frame cycle only.** Go/no-go plus a correctly sized, correctly scoped spec stub. You do **not**
write acceptance criteria, failing tests or Implementation Context.

### Step 0
Own worktree (`just wt`). `git fetch origin && git rebase origin/main`. If you are in the maintainer's main
checkout, stop and say so. Use your own `CARGO_TARGET_DIR`; never share one with another session.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules, "Traps this wave
   paid for" (1 to 16). Binding.
3. STAGE-063 (above) and `/guidance/constraints.yaml`.
4. `src/lint/` (or wherever the rules live: find it), `src/cli/` for the `lint` verb, `docs/cli-reference.md`,
   `skills/crustyimg/SKILL.md` (read only), DEC-050 and any DEC naming `lint` (`./scripts/decisions-audit.sh`).
5. `projects/PROJ-012-lint-can-fix-what-it-names/brief.md` (overlap check only).

### What a good frame does here
- **Drive it before you describe it.** Questions 1, 3 and 4 are measurements.
- ⛔ Run nothing that publishes or tags. No `git tag`, no `npm publish`, no `gh workflow run`.
- Rough build file list (paths). It must not include the PROJ-017 files above.
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### The id is reserved — use exactly SPEC-148
Confirm SPEC-148 is unused outside its reservation:
```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-148' "$b" -- . ; done
```
Expect only STAGE-063 and this prompt. Then `just new-spec "<title>" STAGE-063 PROJ-011`. `just new-spec` scans only
the working tree and may allocate another number; if it is not SPEC-148, `git mv` the spec and timeline files
to SPEC-148 and fix the id inside them. Then `just advance-cycle SPEC-148 frame`. Write no DEC at frame.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity`, `value_link`, `references`, `cycle: frame`.
- `## Context`, `## Goal` (what the maintainer sees today; what they see when it ships).
- A `## Frame` section directly after `## Goal`: verdict (go / no-go / split), premise check with evidence,
  answers to questions 1 to 7, sizing and why, scope in/out, rough build file list, open maintainer questions.
- `### Out of scope (for this spec specifically)`. Leave Acceptance Criteria, Failing Tests and Implementation
  Context as template placeholders.
- Timeline: frame `[x]` with 2 to 4 lines; `[ ]` lines for design (`prompts/SPEC-148-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly, no PR.
1. On **go**: `just advance-cycle SPEC-148 design`. On no-go or split: leave `cycle: frame`.
2. In STAGE-063's backlog, update the SPEC-148 line with the framed date and size; file any out-of-spec finding
   as a new `- [ ] (not yet written)` entry in the stage it belongs to; re-derive each `**Count:**` with a grep
   you have just run. If the frame says SPLIT, say how in the report and let the orchestrator reserve new ids.
   Edit no other stage.
3. `git commit -s`, message `docs(SPEC-148): frame — lint fixes what it names`, ending
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory.
4. `git fetch origin && git rebase origin/main`, `git push origin HEAD:main` (rebase and retry if rejected).
5. Read back (`git show --stat HEAD`) and run `just backlog` to confirm the tooling sees SPEC-148.

### Guardrails
- ⛔ No code, no DEC, no version bump, no tag, no publish. Restore any mutation exactly; `git diff` empty after.
- A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not word-split unquoted
  parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`.
- **Budget ~45 exchanges.**

### Measure your own cost
Transcript JSONL is under `~/.claude/projects/<slug>/` (`<slug>` = absolute working directory with every `/`, `.`
and `_` replaced by `-`). Identify it **by content**: search for `FRAME prompt: lint fixes what it names`. Use
Python or `/usr/bin/grep` (`rtk` can corrupt counts). ⚠ **Dedup by `.message.id`**; take input, cache_creation and
cache_read once per id and the **MAX** `output_tokens`. Price per component at the model that ran (Opus $5/$25,
Sonnet $3/$15; cache_creation ×1.25, cache_read ×0.10). Append a `frame` entry to the spec's `cost.sessions`
(`tokens_total`, `tokens_breakdown`, `estimated_usd`, rate anchors) before the final commit.

### Report back (your final message)
First line: `SPEC-148 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the reproduction of (1) per rule (fix line, stdout bytes, file changed?, finding cleared?) and of (2);
- the option you recommend for (1), with the blast-radius evidence;
- the GPS predicate you recommend and where else "has GPS" is decided;
- rules whose fix does not clear the finding;
- scope in/out, build file list, sizing and verification cost;
- maintainer questions, each with your recommendation;
- anything out of spec; your measured cost.
