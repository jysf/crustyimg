# SPEC-148 — DESIGN prompt: `lint` fixes what it names

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-148-design -- --model opus
```

Starts from the stub the frame pushed to `main` (`b910f9a`). **Reserved: SPEC-148.** No DEC reserved; see "Ids".
About 40 lines of code; the verification is the expensive part. The maintainer pastes this prompt into a fresh
session and pastes your final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 and runs at the same time.** Never edit anything under
`projects/PROJ-017-geometry-as-recipe-steps/`. The build file list must not include `src/operation/`, `src/recipe/`,
`docs/recipes.md`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`, `skills/crustyimg/SKILL.md` or
`CHANGELOG.md`'s Unreleased section (propose changelog text in the spec; do not write it). If a rebase conflicts in
a PROJ-017 file, stop and say so.

**Read the stub's `## Frame` first.** It holds the reproduction (every printed fix, its stdout bytes, whether the
file changed and the finding cleared), the GPS predicate, the files and the sizing. It is your binding scope.

## Rulings (settled by the orchestrator, adopting the frame's recommendations; do not re-open)
1. **Print the working form; no TTY refusal.** The printed fix carries `-o <file> -y`. Out of scope: refusing
   binary-to-TTY, atomic writes.
2. **Quote paths** in every printed fix so a path with a space or a `'` runs as one argument (POSIX single-quote
   quoting; say what Windows `cmd`/PowerShell users see, and test or explicitly scope it).
3. **Fix `convert --format <file>`**: today `--format` swallows the file name. Print a form that runs.
4. **A fix must not pair `--format` with an `-o` of a different extension** (`--format` beats the extension).
   Keep the source format unless the rule is about the format.
5. **`oversized-bytes`**: the fix must carry the rule's own budget (`optimize --max-size <N>`, N derived from the
   rule's threshold), so it clears what it names. The frame showed `optimize` alone fails a 100 KB budget.
6. **16-bit PNG (`wrong-colorspace`)**: print no command; say in words what to do instead and that crustyimg cannot
   convert bit depth. The JSON `fix` field and SARIF message must handle "no fix" without emitting an empty or
   misleading string; state the schema consequence (the frame says no bump for a present fix; decide for an absent one).
   CMYK keeps a fix (`convert --format jpeg`, quoted, with `-o … -y`).
7. **GPS predicate**: the rule fires when any of `GPSLatitude`, `GPSLongitude`, `GPSDestLatitude`,
   `GPSDestLongitude` is present, **or `GPSAreaInformation`** (free-text place name). A block with only
   `GPSVersionID`, altitude, timestamp or map datum is not a leak. `meta clean --gps` is unchanged (it removes the
   whole block, so it always clears the finding). Positive control: real coordinates still flag. The existing unit
   fixture holding only `GPSLatitudeRef` needs a coordinate.
8. **Fixture**: hand-built bytes for the P1100 shape (GPS IFD with only `GPSVersionID`); no external camera file.
9. **Out of scope:** ICC and bit-depth work and animated inputs (PROJ-012), `--keep-gps` (SPEC-149), the
   `optimize` stdout-vs-`-o` size discrepancy (filed separately on STAGE-063; but note whether `-o <f> -y` is the
   path the 459,577 B figure came from), the blog post and CHANGELOG (ship cycle, maintainer's public pages).

## Design calls that are yours
- **One formatter.** `Finding::fix_command` (`src/lint/mod.rs:152`) is the single source for the human `fix:` line,
  the JSON `fix` field and the SARIF message. Specify its new shape, how a rule says "no command", and where the
  quoting lives. Show the exact printed line for every rule, before and after.
- **How each printed fix is proved.** The headline test runs each printed fix string through `sh -c` against a
  scratch copy and asserts: empty stdout, the file changed, `lint` on the result no longer reports that finding, and
  the original format is kept. Decide how it stays honest on Windows CI (trap 11: no row may skip silently; a row
  that cannot run must fail, or be replaced by an equivalent that runs there; say which and why).
- **Which rules print a fix at all**, and their budget/threshold inputs: list every rule in a table with its fix
  text, the input the formatter needs, and whether the rule knows it.
- **JSON/SARIF contract**: `docs/github-actions.md` and any documented schema. Say what changes in each output and
  whether a consumer parsing `fix:` text breaks.

## Acceptance and tests: each one must be able to fail
Count the nouns in each AC against the checks (trap 2). Every check names what turns it red.
- **Each printed fix, run verbatim, fixes its finding** (one row per rule in the table above, plus a path with a
  space and one containing `'`). Control (trap 14): put the **old formatter back verbatim**; every row must go red
  for its reason (stdout non-empty, file unchanged), quoted.
- **GPS rule**: version-only block → no finding; real coordinates → finding; `GPSAreaInformation` alone → finding;
  altitude/timestamp/datum only → no finding. Control: restore the old `src/lint/mod.rs:355` line verbatim; the
  version-only row goes red.
- **No-fix path**: the 16-bit PNG case prints words and no command in human, JSON and SARIF output; control: a
  formatter that returns an empty string is caught.
- **Mutation table (trap 12)**: exact location and every check expected red, for each of: drop `-y`; drop `-o`;
  drop the quoting; swap `--format`/`-o` extension pairing; remove the `--max-size` budget; GPS predicate widened
  back to "any tag"; GPS predicate narrowed to latitude only (the area-information and destination rows must go
  red); the no-fix case emitting an empty `fix`. Each must be red for its reason (trap 13), and a golden needs a
  hash-mismatch mutation.
- **Trap 11:** a test that skips where its coverage is required fails. **Trap 15:** assign each command's output
  first, then validate it, and count every failure kind. **Trap 10 and 16:** quoting and any env var in a step.
- **Size.** [S]. If verification pushes it past S, report the cut before writing it.

## Files
The frame's list: `src/lint/mod.rs`, `src/lint/rules.rs`, `src/lint/report.rs`, `tests/lint.rs` (plus the
fixture), `docs/github-actions.md`. Confirm none is a PROJ-017 path. If a doc comment or `--help` string shows a
fix line, list it.

## DECs and traps
DEC-050 and any DEC naming `lint` (`./scripts/decisions-audit.sh`); HANDOFF traps 2, 10 to 16.

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. A separate session builds it from your spec
alone, and another verifies it, so everything the builder needs must be in the spec (AGENTS §15).

### Step 0
Own worktree. `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-148-lint-fixes-what-it-names.md` with
`cycle: design`; if not, stop and report. Use your own `CARGO_TARGET_DIR`; never share one with another session.

### Read first
1. `/AGENTS.md` (§2, §8, §10, §12, §13, §15). 2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`
(read-first, rules, traps 1 to 16; binding). 3. The stub's `## Frame` and STAGE-063. 4. `src/lint/`, `tests/lint.rs`,
`docs/github-actions.md`, the `lint` verb in `src/cli/`. 5. `/guidance/constraints.yaml` and the DECs above.

### How to design well here
- **Probe, don't assume.** Re-run the frame's reproduction for the rules you design; run the fixes you specify
  (`sh -c`) before putting them in the spec. Use your own target dir.
- ⛔ Nothing that publishes or tags. No `git tag`, `npm publish`, `cargo publish`, `gh workflow run`.
- **Ask what the user sees:** the exact `fix:` line they copy, on a path with a space, in the terminal and in CI.

### Ids
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests and Implementation Context; keep `## Frame`. If
the frame is wrong, say so in the report. If you need a DEC, `ls decisions/` after your last rebase and use the
**block-list** `affected_scope` form. Prefer no DEC unless a real decision is made; say which.

### Landing the design
Design commits go to `main` directly, no PR.
1. `just advance-cycle SPEC-148 build`.
2. In STAGE-063's backlog change only SPEC-148's marker to designed with the date. Mark design `[x]` in the
   timeline with 2 to 4 lines and add `prompts/SPEC-148-build.md` to the build line.
3. `git commit -s`, message `docs(SPEC-148): design lint fixes what it names`, ending
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. DCO sign-off is mandatory.
4. `git fetch origin && git rebase origin/main`, `git push origin HEAD:main` (retry on rejection).
5. Read back `git show --stat HEAD`; run `just backlog`.

### Guardrails
No code or workflow changes outside the spec, timeline, stage file (and a DEC if used). No version bump, tag or
publish. A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not word-split unquoted
parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`. **Budget ~50 exchanges.**

### Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/` (absolute working directory with every `/`, `.`, `_` replaced by
`-`). Identify **by content**: search for `DESIGN prompt: lint fixes what it names`. Python or `/usr/bin/grep`
(`rtk` can corrupt counts). ⚠ **Dedup by `.message.id`**; input, cache_creation, cache_read once per id, **MAX**
`output_tokens`. Price per component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25,
cache_read ×0.10). Append a `design` entry to `cost.sessions` before the final commit.

### Report back (your final message)
First line: `SPEC-148 design: <commit SHA on main> — <N> ACs, <M> checks`. Then, short:
- each design call and how you settled it; the new formatter shape and the table of every rule's printed fix
  (before and after);
- the no-fix path and the JSON/SARIF consequence;
- how the headline test runs on Windows without a silent skip;
- the mutation table: location and every check expected red;
- the build file list, confirming it avoids PROJ-017's files; whether a DEC is used;
- anything needing a **maintainer** ruling, each with your recommendation;
- build on Sonnet or Opus, and why; your measured cost.
