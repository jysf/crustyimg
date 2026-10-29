# SPEC-138 — DESIGN prompt: `build` refuses the output flags its manifest owns

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-138-design -- --model opus
```

Starts from the stub the frame session pushed to `main` (`ccb299c`). **Reserved: SPEC-138,
DEC-108.** The frame expects an amendment to DEC-057 rather than a new DEC; if so, say DEC-108 is
released. This is one of the two specs that open 0.8.1 (SPEC-137 is the other, designed separately).

**Read the stub's `## Frame` section first.** It holds the reproduction, the global-flag sweep, the
honour-vs-refuse analysis, the sizing, the scope and the rough file list. It is your binding scope.
Its `### Maintainer rulings` subsection is settled.

## Rulings (settled — do not re-open)
1. **Refuse.** `build --format` exits 2. It is not honoured.
2. **All four:** `--format`, `-o`, `--out-dir` and `--name-template` are refused on `build` and
   `build --watch`, through one guard.
3. From the frame, accepted by the orchestrator:
   - the guard lives in `src/cli/mod.rs`'s dispatch, next to the existing `--watch` guard (about
     `:867`), so it runs before any manifest, lockfile or input is read;
   - `-q`, `-j`, `-Q`, `--no-cache`, `--check` and `--watch` stay honoured; `-y` stays a no-op;
   - `--keep-gps`, `-v` and `--strict` without `--check` are out (filed on STAGE-054).

## Design calls that are yours
- **How the guard knows a flag was given.** Read each flag's definition in `GlobalArgs`
  (`src/cli/mod.rs:71-144`). An `Option` that is `None` is easy. A flag with a **default value**
  (check `--name-template` in particular) cannot be detected by comparing against the default: a
  user who types the default value is still passing the flag. Check for env-var bindings too.
  Choose `ArgMatches::value_source`, an `Option` without a default, or another way, and cite the
  clap version's API. Add a test row that types each flag's default value explicitly.
- **The messages.** One per flag, or one shared shape. Each must name the flag the user typed
  (`-o` vs `--output` if both spellings exist) and say where the manifest sets it: the target's
  `name` extension or the recipe's `format` for `--format`; the target's `out` / `name` for the
  other three. Check those manifest key names against the build manifest parser, not the frame.
  Show each exact stderr line in an AC.
- **Several refused flags at once.** Name all of them, or the first one? Say which, and test it.
- **Order against other errors.** A refused flag with a missing manifest, a bad manifest, or
  `--check` against a stale lockfile must exit 2 for the flag, not the other error's code. Say
  which rows prove it.
- **Byte identity, and the version-bump trap.** The frame wants (a) no diff under `src/build/` or
  in `src/cli/build.rs`'s plan and key code, and (b) a golden `crustyimg.build.lock` produced by
  `main`'s binary. Every cache key includes the crate version (DEC-058), so a committed golden lock
  goes red at the 0.8.1 bump. Choose how to handle that:
  - a golden that the bump commit regenerates (say who does it and how they know);
  - a check that strips or normalises the version before comparing (prove it still catches a
    key change);
  - a one-time Build Completion comparison of `main`'s binary against the new one, plus the
    structural no-diff check.

  Whatever you choose must be able to fail: name the mutation that turns it red.
- **CHANGELOG.** 0.8.0's notes say `--format` "has no effect" on `build`. Word the new entry now.
  It is a behaviour change (exit 0 → exit 2 for scripts that pass these flags), so decide between
  `Changed` and a `Breaking` note, and say why.
- **The DEC.** Amend DEC-057's Neutral bullet ("ignored by `build`" → "refused"), or write DEC-108.
  Check DEC-105 (the template pins the format) and DEC-087 for wording that also needs changing.

## Tests: each one must be able to fail
- **Every refusal row is RED on `main` today** (it exits 0 there) and says so in Failing Tests,
  with `main`'s measured exit code and what `main` wrote.
- **"Nothing written" is asserted on the files**, not the exit code: no output file, no
  `.crustyimg/` cache entry, an unchanged lockfile (hash it before and after).
- **`build --watch`** with a refused flag exits 2 and does not start watching. Prove the test
  cannot hang: it must fail fast if the guard is missing, not wait forever. Say how (a child process
  with a deadline, for example).
- **Positive control:** `apply` still honours all four flags. Drive each one on `apply`, so the
  control proves the flags work where they belong.
- **Mutation check**, stated in the spec for verify to run: drop one flag from the guard and name
  the row that goes red.

**Size.** The frame says [S], about 8 rows in one test file. The default-value and several-flags
rows may add a few; if it grows past S, report the cut, don't apply it silently.

## Files
The frame's rough list:
- `src/cli/mod.rs` (the guard);
- a new `tests/build_refuses_output_flags.rs`;
- `tests/fixtures/build_golden/` (if you keep the golden);
- `docs/cli-reference.md` (the build section and the global-flag table's `--format` row),
  `README.md` if it implies `build` takes these flags, `CHANGELOG.md`;
- `decisions/DEC-057-…md` (an amendment), or DEC-108.

SPEC-137's build will also touch `CHANGELOG.md`, `README.md` and `docs/cli-reference.md`. The two
builds will run one after the other; note in Implementation Context that the second build rebases
over the first's CHANGELOG entry.

## DECs to read
DEC-057 (the build manifest and its semantics), DEC-058 (build cache keys), DEC-105 (the template
pins the format), DEC-087, DEC-099 (recipe `format`), DEC-101 (the format plan in the key),
DEC-015 (exit codes), and any DEC naming `src/cli/mod.rs` or `src/cli/build.rs`
(`./scripts/decisions-audit.sh`).

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-138-*.md` with `cycle: design`. If
it is missing or still `cycle: frame`, stop and report; do not create it. A separate session builds
it from your spec alone, and another verifies it, so everything the builder needs must be in the
spec (AGENTS §15).

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for". Binding.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`:
   your item, the success criteria and the Design Notes.
4. `/guidance/constraints.yaml` and every DEC named above.
5. A recent shipped spec as the shape to match: `specs/done/SPEC-135-*.md` (Build Completion and
   Verify Readout included). It shows what verify will do to your tests.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end, before you describe it. Give every measurement a negative control.
  **Measure the scenario, not the control.**
- **Ask what the user actually sees:** the command, the stderr line, the exit code.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Every AC names the surface it applies to (`build`, `build --watch`, `build --check`, `apply`).
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, no `CACHE_SCHEMA_VERSION` or lockfile format change: the frame expects none.

### Ids are reserved — use exactly these
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests and Implementation Context;
keep `## Frame`. If you find the frame wrong, say so in your report rather than silently widening or
narrowing it. Highest DEC on `main` was DEC-107 when this was written. Other projects (PROJ-015, PROJ-016) are adding DECs on `main` in parallel, so if you write a new DEC, re-check `ls decisions/` after your last rebase and take the next free id (DEC-108 unless it is gone). Use the **block-list**
`affected_scope` form (an inline array is silently dropped by `scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - src/cli/mod.rs
```

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR.
1. `just advance-cycle SPEC-138 build`.
2. In STAGE-054's backlog, change **only** SPEC-138's marker from `(framed 2026-10-28)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add the
   build prompt path `prompts/SPEC-138-build.md` to the build line.
3. `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-138): design build refusing the output flags`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. SPEC-137's
   design may be landing at the same time; if the rebase conflicts in STAGE-054, keep both markers.
5. Read back `git show --stat HEAD` and run `just backlog`.

### Guardrails
- ⛔ No code changes outside the spec, timeline, stage file and the DEC. No version bump, no tag.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout
  to a file. A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does
  not word-split unquoted parameters. macOS has no `timeout(1)`.
- **Budget ~50 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** `grep -l "SPEC-138 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

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
First line: `SPEC-138 design: <commit SHA on main> — <N> ACs, <M> failing tests`. Then, short:
- each design call and how you settled it, including the exact refusal messages;
- how the guard detects a flag with a default value;
- the byte-identity check, and the mutation that turns it red;
- the negative controls, one per condition;
- the file list the build will touch;
- whether DEC-108 is used or released;
- anything that needs a **maintainer** ruling, each with your recommendation;
- whether to build on Sonnet or Opus, and why;
- your measured cost.
