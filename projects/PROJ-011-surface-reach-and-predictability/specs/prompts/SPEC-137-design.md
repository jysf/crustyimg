# SPEC-137 — DESIGN prompt: SVG text in any font family renders in the bundled font

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-137-design -- --model opus
```

Starts from the stub the frame session pushed to `main` (`ccb299c`). **Reserved: SPEC-137,
DEC-107.** The frame expects an amendment to DEC-054 rather than a new DEC; if so, say DEC-107 is
released. This is one of the two specs that open 0.8.1 (SPEC-138 is the other, designed separately).

**Read the stub's `## Frame` section first.** It holds the reproduction, the family table, the
usvg resolution reading, the sizing, the scope and the rough file list. It is your binding scope.
Its `### Maintainer rulings` subsection is settled.

## Rulings (settled — do not re-open)
1. **Silent fallback.** Every family renders in Go with no message. No warning, no `lint` rule here.
2. **Accept Go Regular.** Bold and italic draw upright in Go Regular. Document it; bundle no faces.
3. From the frame, accepted by the orchestrator:
   - the fix is in `hardened_options()` (`src/image/svg.rs`), pointing fontdb's five generic
     families at Go;
   - no custom font resolver, **if** your reading of usvg 0.49 confirms that its default selector
     always falls back to `serif` (the frame cites `usvg-0.49.0/src/text/mod.rs:114`). Re-read it
     yourself and quote the lines;
   - CJK and other glyphs Go lacks stay out of scope.

## Design calls that are yours
- **The exact code.** Write the five setter calls (or what replaces them) into Implementation
  Context, with the doc comment's wording. Say what happens to `opt.font_family`: is it still
  needed once `serif` points at Go?
- **Cross-platform byte identity.** The frame asks for SHA-256 hashes pinned as constants,
  captured from `main`. CI runs Linux x86-64, macOS arm64 and Windows. tiny-skia has
  architecture-specific SIMD paths, so the same SVG may not produce the same bytes everywhere.
  **Find out before you pin anything**:
  - if the CI artifacts or a cross build make it cheap, compare a render across architectures;
  - otherwise, read how tiny-skia picks its pipeline and say whether the output can differ.

  Then choose one:
  - one pinned hash per form;
  - a hash per architecture;
  - a relative check: each fixed form must be byte-equal to the `Go` render **in the same run**,
    plus one pinned hash per form, but only on the architecture you proved it on.

  A check that is green on the Mac and red on Linux is a failed design (HANDOFF trap: a green gate
  on one OS is not the matrix).
- **The system-font negative control.** Name the font installed on each CI runner image
  (for example DejaVu Sans on `ubuntu-latest`, Arial on `windows-latest`, Helvetica on
  `macos-latest`), and check each name rather than trusting this list. A row only proves something
  where that font is installed. Say what the test does on a host without it: skip with a printed
  reason, never pass silently.
- **How ink is counted in tests.** Byte-equality to the Go render covers most rows. For any row
  that asserts "ink > 0", count it from a decode that does not go through crustyimg's own pipeline.
  Every ink row also needs a negative control (the same SVG with the text removed draws 0).
- **The browser build.** The frame found that wasm uses the same `hardened_options()` with no
  `cfg` split (`src/image/mod.rs:595`). Decide whether that needs a wasm test row, or whether a
  `file:line` argument is enough, and say why.
- **The DEC.** Amend DEC-054 (the SVG hardening rules), or write DEC-107. Either way, record the
  rule: SVG text always renders in the bundled font, whatever family it names.
- **The docs sentence** (`docs/cli-reference.md` SVG section, README) and the **CHANGELOG `Fixed`
  line**, worded now and written at build. The line must say releases since the SVG decoder
  shipped were affected (the frame drove 0.7.1 and 0.8.0). Don't name a version you didn't drive.

## Tests: each one must be able to fail
- **Every fix row is RED on `main` today**, and Failing Tests says so with `main`'s measured ink:
  `sans-serif` 0, `Arial` with a `Go` `<tspan>` 359, and so on.
- **Every byte-identity row is GREEN on `main` today**, and must stay green. Say so per row.
- **Cover the family forms, not just one.** At least: one generic family, a named family that is
  installed, a named family that does not exist, a comma list without Go, a quoted list, a family
  set in `style=` and in a `<style>` element, lowercase `go`, and a mixed `<tspan>`.
- **Mutation check**, stated in the spec for verify to run: remove one setter call (for example
  `monospace`) and name the row that goes red. If no row does, the table is short a row.

**Size.** The frame says [S], about 18 assertions in one test file. If the cross-platform question
forces per-architecture goldens, say what that adds. If it grows past S, report the cut; don't
apply it silently.

## Files
The frame's rough list:
- `src/image/svg.rs` (`hardened_options()` and its unit tests);
- a new `tests/svg_font_family.rs`;
- `tests/fixtures/svg/`, or SVGs generated inline;
- `docs/cli-reference.md`, `README.md`, `CHANGELOG.md`;
- `decisions/DEC-054-…md` (an amendment), or DEC-107.

SPEC-138's build will also touch `CHANGELOG.md`, `README.md` and `docs/cli-reference.md`. The two
builds will run one after the other; note in Implementation Context that the second build rebases
over the first's CHANGELOG entry.

## DECs to read
DEC-054 (SVG rasterize, the hardening), DEC-045 (the bundled Go font), DEC-034 (decode limits),
DEC-066 (wasm size gate), and any DEC naming `src/image/svg.rs` (`./scripts/decisions-audit.sh`).

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-137-*.md` with `cycle: design`. If
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
- **Ask what the user actually sees:** the command, the output file, the exit code.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Every AC names the surface it applies to (CLI `convert`/`resize` on an SVG input, `apply
  --recipe`, the browser build).
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, no `CACHE_SCHEMA_VERSION` or lockfile change expected. If the render change for
  previously-blank SVGs affects `build`'s cache (a cached blank output would now be stale), say so
  as a design call: the cache keys on the crate version, so check whether 0.8.1's bump covers it.

### Ids are reserved — use exactly these
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests and Implementation Context;
keep `## Frame`. If you find the frame wrong, say so in your report rather than silently widening or
narrowing it. Highest DEC on `main` is DEC-106. If you write DEC-107, use the **block-list**
`affected_scope` form (an inline array is silently dropped by `scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - src/image/svg.rs
```

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR.
1. `just advance-cycle SPEC-137 build`.
2. In STAGE-054's backlog, change **only** SPEC-137's marker from `(framed 2026-10-28)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add the
   build prompt path `prompts/SPEC-137-build.md` to the build line.
3. `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-137): design SVG text in any font family`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. SPEC-138's
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
not recency:** `grep -l "SPEC-137 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

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
First line: `SPEC-137 design: <commit SHA on main> — <N> ACs, <M> failing tests`. Then, short:
- each design call and how you settled it, including the code change;
- the cross-platform answer, with its evidence (driven, or read with `file:line`);
- the system-font control per CI OS;
- the negative controls, one per condition;
- the file list the build will touch;
- whether DEC-107 is used or released;
- anything that needs a **maintainer** ruling, each with your recommendation;
- whether to build on Sonnet or Opus, and why;
- your measured cost.
