# SPEC-150 — DESIGN prompt: two messages tell the truth, and `--help` carries no spec ids

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-150-design -- --model opus
```

Starts from the stub the frame pushed to `main` (`3402652`). **Reserved: SPEC-150.** Size **M**. The maintainer pastes
this prompt into a fresh session and pastes your final message back to the orchestrator, so it must stand on its own. A
separate session builds from your spec alone.

⚠ **Another orchestrator thread owns PROJ-017 (geometry as recipe steps).** Never edit anything under
`projects/PROJ-017-geometry-as-recipe-steps/`. These are PROJ-017's, so the build must not edit them: `docs/recipes.md`,
`recipes/README.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`'s Unreleased section, `src/operation/`, `src/recipe/`,
`tests/common/mod.rs`, `tests/wasm_roundtrip.rs`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`,
`scripts/wasm-ci-check.sh`. Put the text for any of them in the spec and list it as a coordination item in your report
(the frame found `docs/recipes.md:148` repeats lint's "each with a runnable fix"; `tests/wasm_roundtrip.rs:283` asserts
the words "filesystem" and "out of scope", which the new browser sentence must keep). `src/cli/mod.rs`, `src/wasm.rs`,
`docs/cli-reference.md`, `AGENTS.md` are shared: keep each hunk to the smallest that does the job. Ask the
orchestrator (in your report) whether PROJ-017 has a build in flight that holds any file in your list; do not assume.
If a rebase conflicts in a PROJ-017 file, stop and say so.

**Read the stub's `## Frame` first.** It holds the reproductions, the id-sweep counts, the guard-test shape, the
replacement text and the fold-in rulings. It is your binding scope.

## Rulings (settled by the orchestrator 2026-11-06 on the frame's recommendations; do not re-open)
1. **A folder with no images stays an error, exit 3.** Only the wording changes. State what is true: the folder was read
   and held no supported image at its top level (say whether the message should mention `-r`/a glob if the resolver
   supports them; drive it). A missing path keeps `input not found or unreadable: <path>`, which is true there. The six
   callers (`src/cli/ops.rs:42`, `build.rs:333`, `optimize.rs:146` and `:1619`, `report.rs:227` and `:505`) must not each
   re-decide: **one** place produces the message from one cause (a typed empty-result error, or one helper); show how
   the callers share it. `responsive` and `diff`, which print "could not read image input" even for a folder, are in.
2. **Fold-ins in:** `optimize -q`/`web -q` blaming flags the user did not pass; the garbled `meta` "does not support"
   messages and their siblings (`Gif yet`, `WebP yet`, `unknown yet`); the unescaped `"` in a path (and one shared path
   display helper, if it is cheaper than patching each message). **Out:** `--keep-gps=false` (clap's message is true),
   O-1/O-2, the `optimize`-to-stdout size item.
3. **Ids in code comments are out of scope.** The guard scans only non-comment lines, `--help`, and completion scripts.
4. **DEC-003 is amended now, docs only:** pixel-lane encodes drop all metadata (ICC included; the frame drove it).
   Correct the SPEC-149 amendment's claim that `meta copy` works on PNG: it is **JPEG only** (it exits 4 on PNG). Fix
   `AGENTS.md:503` and `docs/architecture.md:150` to match. Use an amendment line, not a new DEC, unless you find a
   real decision. Take any DEC id from `ls decisions/` after your last rebase.
5. **`docs/cli-reference.md` is cleaned of ids** (about 10 lines), including the stale "lands in SPEC-111" at `:312`.

## Design calls that are yours
- **The guard test** (`tests/help_has_no_internal_ids.rs`, as the frame specs it). Settle: how it walks `Cli::command()`
  (root, `meta` and every nested subcommand, `-h` and `--help`), the minimum-page floor and where that number comes from
  (it must be derived, with a floor test, never a hand list: HANDOFF traps 11 and 17); how the completion scripts are
  generated; how the non-comment source scan finds comment lines and stops at `#[cfg(test)]` without being fooled by
  `//` inside a string or a raw string; line endings normalised (CRLF on Windows); the pattern split so the test cannot
  match itself. Say where an id could still hide and whether you add a cheap second net (for example scanning
  `demo/demo.js` and `demo/index.html`, which the frame found carry DEC-065 and DEC-069).
- **The exact replacement text** for every help line that cites an id (the frame has the high-traffic ones; complete the
  table, and write the rule for the rest). `lint`'s sentence must be true given `fix: null`.
- **The browser refusal** sentence in `src/wasm.rs:236-240`: no id, no internal words ("wasm surface", "boundary"),
  keeps "filesystem" and "out of scope". Drive it in the Node round-trip test path or quote the existing assertion.
- **The folder message wording**, per ruling 1, as a table of verb × shape → exact message and exit code, driven on the
  sketch (not typed from memory). Include the glob that matches only non-images.
- **`optimize -q`**: which function builds the message, what is true (the default `AutoQuality::Fast` trips
  `reject_quality_with_auto`), and the exact new message. Does exit 2 stay? Say why.
- **Docs.** Exact sentence changes for each file, PROJ-017-owned files marked. **Proposed CHANGELOG text** (the frame has
  a `Fixed` draft; refine it; do not write `CHANGELOG.md`).

## Acceptance and tests: each one must be able to fail
Count the nouns in each AC against the checks (trap 2). Every check names what turns it red.
- **The guard:** a row per surface (help, completions per shell, source strings) that goes red when an id is planted
  there, shown by the mutation table; a floor assertion that goes red if the walk sees fewer pages than the CLI has.
- **Folder messages:** per verb × shape, exit code and exact stderr, and **nothing written**. Control (trap 14): restore
  the old call; every row red with the old text quoted. A row that reaches the message by the wrong cause is not a pass
  (trap 13): the missing-path control must still print the old true message.
- **`-q` message** and **meta messages**: driven end to end, not by matching a constant (SPEC-149 lesson: a message that
  tells the user something is a claim; drive what it says).
- **Mutation table (trap 12):** exact location and every check expected red for: one caller restored to the old message;
  the shared helper returning the missing-path text; an id re-added to one `--help` string; to one nested `meta`
  subcommand; to a completion-only path if one exists; to `src/wasm.rs`; `lint`'s sentence restored; the `-q` message
  restored; the `"` escaping removed; the walk made to visit only the root. Each red for its reason.
- **Trap 15:** assign each command's output first, then validate. **Trap 16:** no step-level env var that changes more
  than the one command. **Trap 11:** nothing skips silently on Windows (paths, CRLF).
- **Size.** [M]. Report the cut before writing it if verification pushes past M. The build must also run
  `cargo build --no-default-features` and `just wasm-check`.

## Files
The frame's list. Mark each shared file and give the smallest hunk; confirm none is a PROJ-017-owned path.

## DECs and traps
DEC-003, DEC-050/DEC-059 (what the replaced help text was documenting; keep the behaviour statements true), the
SPEC-149 spec under `specs/done/` and `tests/cli_flags_honest.rs` (the model for a command-tree walk); HANDOFF traps 2,
10 to 17 and rules 1 to 7.

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it.

### Step 0
Own worktree. `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-150-two-messages-tell-the-truth-and-help-carries-no-spec-ids.md`
with `cycle: design`; if not, stop and report. Own `CARGO_TARGET_DIR`; never share one with another session.

### Read first
1. `/AGENTS.md` (§2, §8, §10, §12, §13, §15). 2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`
(binding). 3. The stub's `## Frame` and STAGE-063. 4. `src/source/mod.rs`, `src/cli/ops.rs`, `build.rs`,
`optimize.rs`, `report.rs`, `src/cli/mod.rs`, `src/wasm.rs`, `src/metadata/mod.rs`, `src/error.rs`.
5. `/guidance/constraints.yaml`.

### How to design well here
- **Probe, don't assume.** Re-run the frame's reproductions for what you design; run the guard sketch and quote its hit
  count on `main` before putting numbers in the spec. Use your own target dir.
- ⛔ Nothing that publishes or tags. No `git tag`, `npm publish`, `cargo publish`, `gh workflow run`.
- **Ask what the user sees:** the exact message for each case.

### Ids
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests and Implementation Context; keep `## Frame`. If the
frame is wrong, say so in the report.

### Landing the design
Design commits go to `main` directly, no PR.
1. `just advance-cycle SPEC-150 build`.
2. In STAGE-063's backlog change only SPEC-150's marker to designed with the date. Mark design `[x]` in the timeline
   with 2 to 4 lines and add `prompts/SPEC-150-build.md` to the build line.
3. `git commit -s`, message `docs(SPEC-150): design two messages tell the truth`, ending
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. If you amend DEC-003 / write the other docs in the
   *spec* only (the build applies them), say so; if the DEC amendment is a decision file, commit it here.
4. `git fetch origin && git rebase origin/main`, `git push origin HEAD:main` (retry on rejection).
5. `git show --stat HEAD`; run `just backlog`.

### Guardrails
No code or workflow changes outside the spec, timeline, stage file, `guidance/ideas.yaml` and a DEC amendment. No
version bump, tag or publish. A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not
word-split unquoted parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`.
**Budget ~55 exchanges.**

### Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/` (absolute working directory with every `/`, `.`, `_` replaced by
`-`). Identify **by content**: search for `DESIGN prompt: two messages tell the truth`. Python or `/usr/bin/grep`.
⚠ **Dedup by `.message.id`**; input, cache_creation, cache_read once per id, **MAX** `output_tokens`. Price per
component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25, cache_read ×0.10). Append a
`design` entry to `cost.sessions` before the final commit.

### Report back (your final message)
First line: `SPEC-150 design: <commit SHA on main> — <N> ACs, <M> checks`. Then, short:
- each design call and how you settled it; the folder-message table (verb × shape → message, exit);
- the guard test: how it walks, the floor and its derivation, the hit count on `main`, where an id could still hide;
- the replacement-text table; the browser sentence; the `-q` message;
- the mutation table: location and every check expected red;
- the build file list with shared files and smallest hunks, confirming it avoids PROJ-017-owned files; coordination
  items for the orchestrator; the DEC-003 amendment lines;
- the proposed CHANGELOG text;
- anything needing a **maintainer** ruling, each with your recommendation;
- build on Sonnet or Opus, and why; your measured cost.
