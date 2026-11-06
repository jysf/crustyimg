# SPEC-149 — DESIGN prompt: no flag is accepted and ignored

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-149-design -- --model opus
```

Starts from the stub the frame pushed to `main` (`ee77e79`). **Reserved: SPEC-149.** Size **M**. The maintainer
pastes this prompt into a fresh session and pastes your final message back to the orchestrator, so it must stand
on its own. A separate session builds from your spec alone.

⚠ **Another orchestrator thread owns PROJ-017 and runs at the same time.** Never edit anything under
`projects/PROJ-017-geometry-as-recipe-steps/`. `docs/recipes.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`'s
Unreleased section, `tests/common/mod.rs`, `src/operation/` and `src/recipe/` are PROJ-017's: the build must not
edit them (propose the text in the spec; a one-line edit there is a coordination item for the orchestrator, listed
in your report). `src/cli/mod.rs`, `src/cli/ops.rs`, `src/cli/optimize.rs`, `README.md`, `docs/cli-reference.md`,
`docs/api-contract.md`, `docs/data-model.md` and `AGENTS.md` are shared: keep each edit to the smallest hunk that
does the job, so a later merge with SPEC-151 (`pad`, may add a verb arm in `src/cli/mod.rs`) is mechanical. If a
rebase conflicts in a PROJ-017 file, stop and say so.

**Read the stub's `## Frame` first.** It holds the reproduction, the per-flag option analysis, the font-message
locations, the docs list and the sizing. It is your binding scope.

## Rulings (settled by the maintainer 2026-11-06; do not re-open)
1. **`--keep-gps`: refuse now, build later.** Every verb exits 2 with a message saying pixel-lane encodes drop all
   metadata and naming `crustyimg meta copy --from <in> --to <out> -y` (JPEG and PNG). Hide it from `--help`. Keep
   it documented, but as refused, with the workaround. Do **not** remove the flag from the parser. The real feature
   is filed on PROJ-013 STAGE-052 (done by the orchestrator); do not design it here. The refusal is a placeholder
   for a future feature, so the message must not say the flag "does not exist" or "was removed".
2. **`-v` / `-vv`: implement the minimal stderr logger** (~30 lines, no `env_logger`; direct `log = "0.4"`, already
   in the tree). `-v` = warn, `-vv` = info and debug, `-Q` wins, default stays silent so nothing changes for anyone
   not passing it. The frame has not driven a warning: **you must**, with an SVG that triggers a named usvg
   `warn!`, and quote the stderr line. Decide where the logger is installed (before any verb runs, once) and what
   happens with `build --watch`, `-j` threads and wasm (the logger must not be linked into the wasm build; show
   `just wasm-check` still passes).
3. **Refuse the rest, exit 2:** `responsive --name-template` and `responsive --format`; `build --strict` without
   `--check`; `--check`, `--strict` and `--no-cache` on every verb but `build`. One guard table beside the existing
   `--watch` guard (SPEC-071), not scattered `if`s. `responsive`'s message says its names are fixed
   `{stem}-{w}w.{ext}` and points `--format` users at `--formats`. A `responsive` template goes to
   `guidance/ideas.yaml` beside the `{hash}` token idea; say what you add.
4. **Missing `watermark --font` and `--image`: keep exit 3, name the file** (`could not read font file "…": …`
   and `could not read watermark image "…": …`, matching the recipe path's wording). Fix `std::fs::read(path)
   .map_err(ImageError::Io)?` at `src/cli/ops.rs:1446` and `watermark_overlay` at `:1433`.
5. **DECs:** DEC-003 and DEC-024 each get an amendment line (no new DEC unless a real decision is made; the logger
   and the guard table are not one). Take DEC ids from `ls decisions/` after your last rebase (PROJ-017 holds
   DEC-113; you take 114+ only if you need one) and use the block-list `affected_scope` form.
6. **Out of scope:** implementing `--keep-gps`, a `responsive` template, token validation of `--name-template`
   (unknown tokens are written literally today), findings A to C filed on STAGE-063 (flag/`-o` oddities on `meta`
   and `info`, the `optimize -q 10` message, the DEC-003 artist/copyright drift). Say if one of them blocks a ruling.

## Design calls that are yours
- **The guard table.** Where it lives, how it names a verb and a flag, how it covers a verb added later (the test
  must iterate `Cli::command().get_subcommands()`, including `meta`'s subcommands, never a hand list). The exact
  message for each refusal; whether `--help` still lists the refused flags (hidden vs "build only" help text) and
  what clap's own `requires`/`conflicts_with` could do instead (prefer one mechanism; justify).
- **Every verb × every refused flag**, as a table in the spec with the exact message and exit code. Include
  `--check` as used by non-`build` verbs that have their own `--check` (grep: does any other verb define one?).
  Make sure no verb that legitimately reads a global flag is refused (positive controls).
- **The logger.** Module, level mapping, format of a line, `-Q` precedence, behaviour with `-v -v`/`-vv`, and a
  test that proves the line appears with `-v`, is absent without, and absent with `-v -Q`.
- **Docs.** Exact sentence changes for each file in the frame's answer 5, with PROJ-017-owned files marked.
- **Proposed CHANGELOG text** (the frame has a draft; refine it; do not write `CHANGELOG.md`).

## Acceptance and tests: each one must be able to fail
Count the nouns in each AC against the checks (trap 2). Every check names what turns it red.
- **Refusals:** per verb and flag, exit 2, the message text, and **nothing written** (directory listing before and
  after). Control (trap 14): remove the guard; every refusal row red with exit 0 quoted. Positive controls:
  `build --check --strict` still runs and fails on variance; `build --no-cache` still rebuilds; `build --watch`'s
  existing refusals unchanged.
- **Verb coverage is derived, not listed (traps 11, 17):** a test that iterates the command list must fail if the
  list is empty or a verb is missing the guard; show the verb count it saw.
- **`-v`:** the SVG fixture's warning line with `-v`, not without, not with `-v -Q`. A row that cannot show the
  line is a failure, never a skip. Drive the maximising input (trap 17): the verb that emits the most/least.
- **Font/image:** restore the old line verbatim → the row goes red for its reason (message lacks the path).
- **Mutation table (trap 12):** exact location and every check expected red for: guard deleted; one verb dropped
  from the table; `-Q` precedence dropped; logger installed after the verb runs; `--keep-gps` message dropped
  `meta copy`; font line restored; `responsive` refusing only one of its two flags; `build --strict` alone allowed;
  `--no-cache` refused on `build` (positive control red). Each red for its reason (trap 13).
- **Trap 15:** assign each command's output first, then validate. **Trap 16:** no step-level env var that changes
  more than the one command. **Trap 11:** nothing skips silently on Windows.
- **Size.** [M]. If verification pushes it past M, report the cut before writing it.

## Files
The frame's list, plus `src/main.rs` or a new logger module, `Cargo.toml`/`Cargo.lock` (direct `log`; show
`cargo tree -i log` and that no new crate enters, and `cargo deny` stays green), `tests/cli_flags_honest.rs` (new),
one SVG fixture, `guidance/ideas.yaml`, and the docs listed in the frame. Mark each shared file and give the
smallest hunk. Confirm none is a PROJ-017-owned path.

## DECs and traps
DEC-003, DEC-024, DEC-012 (clap), SPEC-071's guard, SPEC-134's template decision (DEC-105); HANDOFF traps 2, 10 to
17 and rules 1 to 7.

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it.

### Step 0
Own worktree. `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-149-no-flag-is-accepted-and-ignored.md` with
`cycle: design`; if not, stop and report. Own `CARGO_TARGET_DIR`; never share one with another session.

### Read first
1. `/AGENTS.md` (§2, §8, §10, §12, §13, §15). 2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`
(binding). 3. The stub's `## Frame` and STAGE-063. 4. `src/cli/mod.rs` (`GlobalArgs`, dispatch, the `--watch`
guard), `src/cli/ops.rs`, `src/cli/optimize.rs`, `src/main.rs`, `docs/api-contract.md`. 5. `/guidance/constraints.yaml`.

### How to design well here
- **Probe, don't assume.** Re-run the frame's reproductions for what you design; run the guard and logger sketch
  before putting numbers in the spec. Use your own target dir.
- ⛔ Nothing that publishes or tags. No `git tag`, `npm publish`, `cargo publish`, `gh workflow run`.
- **Ask what the user sees:** the exact message a script author gets for each refused flag.

### Ids
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests and Implementation Context; keep `## Frame`.
If the frame is wrong, say so in the report.

### Landing the design
Design commits go to `main` directly, no PR.
1. `just advance-cycle SPEC-149 build`.
2. In STAGE-063's backlog change only SPEC-149's marker to designed with the date. Mark design `[x]` in the timeline
   with 2 to 4 lines and add `prompts/SPEC-149-build.md` to the build line.
3. `git commit -s`, message `docs(SPEC-149): design no flag is accepted and ignored`, ending
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
4. `git fetch origin && git rebase origin/main`, `git push origin HEAD:main` (retry on rejection).
5. `git show --stat HEAD`; run `just backlog`.

### Guardrails
No code or workflow changes outside the spec, timeline, stage file, `guidance/ideas.yaml` and a DEC amendment.
No version bump, tag or publish. A piped command reports the pipe's exit code: redirect and read `$?`. zsh does
not word-split unquoted parameters. macOS has no `timeout(1)`. Scratch files in a scratch directory, never in `~`.
**Budget ~55 exchanges.**

### Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/` (absolute working directory with every `/`, `.`, `_` replaced by
`-`). Identify **by content**: search for `DESIGN prompt: no flag is accepted and ignored`. Python or `/usr/bin/grep`.
⚠ **Dedup by `.message.id`**; input, cache_creation, cache_read once per id, **MAX** `output_tokens`. Price per
component at the model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25, cache_read ×0.10). Append a
`design` entry to `cost.sessions` before the final commit.

### Report back (your final message)
First line: `SPEC-149 design: <commit SHA on main> — <N> ACs, <M> checks`. Then, short:
- each design call and how you settled it; the guard table (verb × flag × message);
- the logger: where installed, a quoted real warning line, wasm still builds;
- the mutation table: location and every check expected red;
- the build file list with shared files and smallest hunks, confirming it avoids PROJ-017-owned files; any
  coordination item for the orchestrator; whether a DEC is used and the amendment lines;
- the proposed CHANGELOG text;
- anything needing a **maintainer** ruling, each with your recommendation;
- build on Sonnet or Opus, and why; your measured cost.
