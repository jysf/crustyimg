# SPEC-135 — DESIGN prompt: `watermark` refuses a size or scale that is not a sensible number

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-135-design -- --model opus
```

Starts from the stub the frame session pushed to `main` (`823f845`). **Reserved: SPEC-135,
DEC-106.** STAGE-054's first backlog item. **This spec gates 0.8.0**: the release is this spec plus
release prep, nothing else (maintainer, 2026-10-27).

**Read the stub's `## Frame` section first.** It holds the reproduction, the CLI-vs-recipe sweep,
the cap measurements, the exit-code and arity analysis, the scope and the rough file list. It is
your binding scope. Its `### Maintainer rulings` subsection is settled.

## Rulings (settled — do not re-open)
1. **Cap: `size <= 1024`**, as a range row in `WATERMARK_PARAMS`, so the CLI and recipes share one
   bound. It also closes the `--size 100000` panic (exit 101, in `zeno`).
2. **Marks that draw nothing on both paths are out** (`opacity = 0`, blank text, `size = 0.01`).
   They are filed on STAGE-054 for 0.8.1. Do not refuse them here.
3. **Keep `f32` parsing on the CLI.** Document the residual: a value within one `f32` step of a
   bound is accepted on the CLI and refused in a recipe.
4. From the frame, accepted by the orchestrator:
   - the CLI validates through the shared table, not a second hand-written copy;
   - a bad flag is a usage error (exit 2) **before any file is read**, including the font and the
     overlay;
   - the recipe path is unchanged except for the new cap (exit 1 at load);
   - `--scale` gets no upper bound; only its saturated `4294967295×…` message is fixed, on both
     paths.

## Design calls that are yours
- **How the CLI reaches the table.** CLI `resize` already builds its op through the registry and
  maps the error to exit 2 (`src/cli/ops.rs:254-280`, per the frame). Choose between:
  - building the flags into `OperationParams` and running the registry's check;
  - a public per-key check the CLI calls.

  Whichever you pick, the message must name the **flag the user typed** (`--size`), not the recipe
  key (`'size'`). Show the exact stderr line in an AC. Removing the hand-written `size`, `scale`
  and `opacity` checks in `src/cli/ops.rs` is expected. Say what replaces each.
- **The message for an oversized mark.** No refusal may print `4294967295` or any clamped value as
  a mark dimension, on either path. Choose: compute in `u64`, or say the mark is wider than the
  image. Drive it with `--scale inf`, `--scale 1e39` and `scale = 4294967296` in a recipe.
- **The `--size` help text**, the `docs/data-model.md` range table and `docs/api-contract.md`: the
  cap, the before-any-IO order, and the `f32` residual (ruling 3).
- **DEC-106 or an amendment to DEC-102.** The frame leans DEC-106, because the CLI verb validating
  through the registry's table while it builds its op directly refines DEC-031 as well as DEC-102.
  DEC-102's Consequences line about the `size` upper bound must change either way. Say whether the
  DEC is written at design or at build.
- **CHANGELOG lines** (written at build, worded by you now):
  - **Fixed:** the CLI non-finite `--size`/`--scale` rows; the `--size 100000` panic; the
    saturated message.
  - **Breaking:** `size > 1024` is refused, on the CLI and in a recipe.

  Check whether released 0.7.1 really panics before the Fixed line says it does. The frame read
  the code at the tag but did not run the released binary. Use the `v0.7.1` release asset, or
  build the tag in a fresh target dir.
- **Wasm.** `transform` takes a recipe, so it inherits the cap through the schema. Decide whether
  that needs its own AC row or is covered by the schema's unit row, and say why.

## Tests: each one must be able to fail
STAGE-050's lesson, now in HANDOFF trap 10: a test whose input makes the old and the new behaviour
agree passes hollow. So:
- **Every refusal row is RED on `main` today** and says so in Failing Tests, with `main`'s actual
  exit code. For example, `--size nan` exits 0 on `main`.
- **Input arity is not argument count.** For at least two flags, cover: one input, a batch
  directory, a glob that resolves to one file, stdin, and a missing input. Each must exit 2 before
  reading anything. Add the font and overlay ordering rows: a bad flag with a missing `--font` or
  `--image` exits 2, not 3.
- **Boundaries at the type the op reads** (trap 9): `size = 1024` is accepted and `1025` refused,
  on both paths. Also add one row pinning the documented `f32` residual on the CLI.
- **Byte identity:** an accepted mark is byte-identical to `main`'s binary. Cover the default text
  size, `size = 1024`, and an image with `--scale`, each through the CLI and through
  `apply --recipe`. Say how the test gets `main`'s bytes (golden files, or a recorded hash).
- **Negative controls for verify, one per independent condition.** Name each mutation and the test
  it must turn RED:
  - the CLI bypasses the table and goes back to the hand checks;
  - the cap row is removed;
  - the saturated message is restored;
  - the flag checks move back after the IO.

**Size.** The frame says upper [S], about 29 matrix rows, most of them in one table-driven test.
If it grows past S, use the frame's cut line, not a wider spec: the font/overlay ordering and the
`f32` documentation may move to STAGE-054. Report the cut; don't apply it silently.

## Files
The frame's rough list:
- `src/operation/mod.rs` (the `size` row; `Refusal::DoesNotFit` / `mark_size` message);
- maybe `src/operation/registry.rs`;
- `src/cli/ops.rs` (`watermark_overlay`, `run_watermark`);
- `src/cli/mod.rs` (help text);
- a new `tests/watermark_cli_params.rs`, plus a row in `tests/recipe_strict_params.rs`;
- `docs/api-contract.md`, `docs/data-model.md`, `CHANGELOG.md`, `recipes/README.md` if it states
  the size range;
- the DEC.

List every source file and, for `src/cli/ops.rs`, the functions you touch. No other build is
running, so this is for verify's scope check.

## DECs to read
DEC-102 (recipe params validated at load), DEC-104 (the damage rule), DEC-031 (operations stay
filesystem-free), DEC-036 (recipe resource limits), DEC-063 (decode limits), DEC-015 (exit codes),
and any DEC naming `src/text/**` or `src/cli/ops.rs` (`./scripts/decisions-audit.sh`).

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-135-*.md` with `cycle: design`. If
it is missing or still `cycle: frame`, stop and report; do not create it. A separate session builds
it from your spec alone, and another verifies it, so everything the builder needs must be in the
spec (AGENTS §15).

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (1–10). Binding.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`:
   your item, the success criteria and the Design Notes.
4. `/guidance/constraints.yaml` and every DEC named above.
5. A recent shipped spec as the shape to match: `specs/done/SPEC-133-*.md` (Build Completion and
   Verify Readout included). It shows what verify will do to your tests.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end, before you describe it. Give every measurement a negative control.
  **Measure the scenario, not the control.**
- **Ask what the user actually sees:** the command, the stderr line, the exit code.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Every AC names the surface it applies to (CLI `watermark`, `apply --recipe`, `build`).
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, no `CACHE_SCHEMA_VERSION` or lockfile change: the frame expects none
  (checking never rewrites `OperationParams`). If you find one is needed, say so as a design call.

### Ids are reserved — use exactly these
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests and Implementation Context;
keep `## Frame`. If you find the frame wrong, say so in your report rather than silently widening or
narrowing it. Highest DEC on `main` is DEC-105. Write DEC-106 with the **block-list**
`affected_scope` form (an inline array is silently dropped by `scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - src/cli/ops.rs
```

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR.
1. `just advance-cycle SPEC-135 build`.
2. In STAGE-054's backlog, change **only** SPEC-135's marker from `(framed 2026-10-27)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add the
   build prompt path `prompts/SPEC-135-build.md` to the build line.
3. `git commit -s`. **The DCO sign-off is mandatory.** Message: `docs(SPEC-135): design <short title>`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back `git show --stat HEAD` and run `just backlog`.

### Guardrails
- ⛔ No code changes outside the spec, timeline, stage file and (if you write it now) DEC-106. No
  version bump, no tag.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout
  to a file. A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does
  not word-split unquoted parameters. macOS has no `timeout(1)`.
- **Budget ~60 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** `grep -l "SPEC-135 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read
×0.10. Append a `design` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown` and `estimated_usd`, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-135 design: <commit SHA on main> — <N> ACs, <M> failing tests`. Then, short:
- each design call and how you settled it, including the exact refusal messages;
- whether released 0.7.1 panics at `--size 100000` (driven, not read);
- the negative controls, one per condition;
- the file list the build will touch;
- anything that needs a **maintainer** ruling, each with your recommendation;
- whether to build on Sonnet or Opus, and why;
- your measured cost.
