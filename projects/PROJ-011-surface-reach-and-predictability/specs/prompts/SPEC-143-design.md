# SPEC-143 — DESIGN prompt: a watermark that draws nothing is refused

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-143-design -- --model opus
```

Starts from the stub the frame session pushed to `main` (`e079978`). **Reserved: SPEC-143,
DEC-109.** The frame leans to a new DEC-109 over amending DEC-104. If you amend instead, say
DEC-109 is released. This spec opens **0.8.2**.

**Read the stub's `## Frame` section first.** It holds the reproduction, the opacity analysis,
the "almost zero" thresholds, the construction-site table, the proposed messages, the byte rows,
the doc list and the sizing. It is your binding scope. Its `### Maintainer rulings` subsection is
settled.

## Rulings (settled — do not re-open)
1. **`opacity = 0` is refused**, like every other invisible mark: exit 2 on the CLI and exit 1 in a
   recipe. To turn a watermark off, remove the step.
2. **The rule is decided from the mark:** refuse a mark with no pixel of alpha > 0 after opacity.
   A faint mark that changes nothing on one particular image is drawn as asked.
3. **A positive opacity too small to keep any alpha is refused too** ("too faint").
4. **`Watermark::new_image` / `new_text` stay public and infallible.** No signature change and no
   new `Refusal` variant (it is not `#[non_exhaustive]`). Anything you add to the library API is
   additive only.
5. From the frame, accepted by the orchestrator:
   - `visible_pixels()` and DEC-104's coverage figure **do not change**.
     `opacity_does_not_change_coverage` stays green **without edits**;
   - `--opacity` stays off DEC-102's key table, so today's `--opacity 1.5` message is unchanged;
   - no new recipe key, no `params()` change, no cache-key or lockfile change;
   - out of scope: the text-raster memory bound, the `render_text` library panic, library
     callers, and a `scale` resize that leaves no alpha.

## Design calls that are yours
- **The predicate, and that it cannot disagree with the composite.** The frame reduces it to
  `round(max_alpha16 × opacity) == 0`, using the 16-bit branch's expression (`src/operation/mod.rs`
  about `:1777-1780`). The 8-bit branch (`:1833-1836`) rounds at 8 bits. Show, with numbers,
  whether an opacity exists that keeps alpha at 16 bits but rounds to 0 at 8 bits (or the reverse),
  and say which branch a mark takes. The check must never accept a mark the composite then draws
  as nothing, on either branch. If the two branches differ, the predicate follows the stricter
  one, or the spec says why not.
- **Where it runs.** One predicate on the built `Watermark`, called from two places:
  `run_watermark` (`src/cli/ops.rs`, before `run_pixel_op` reads any input) and the end of
  `from_params` (`src/operation/mod.rs:1518`, which covers `apply`, `build`, `--watch` and wasm).
  Name the method and its return type (for example a cause enum), and confirm it is additive.
- **The CLI order (DEC-106).** `--opacity 0`, blank `--text` and a colour with alpha `00` are
  decidable from the flags. Decide whether they are separate checks with SPEC-135's flag checks at
  the top of `run_watermark`, or the same predicate called early. Either way,
  `watermark /no/such.png --opacity 0` exits **2**, not 3. A too-small `--size` and a transparent
  `--image` need the rendered or loaded overlay; they come after that read and before any input.
  Give the row that proves each order.
- **The cause classifier.** Blank text, too small and colour alpha `00` all render no visible
  pixel. Specify the order of checks (colour alpha, then whitespace-only text, then too small) and
  what "whitespace-only" means: a non-breaking space? a zero-width joiner? a character the Go
  font has no glyph for? Drive each on the binary and decide which cause it reports.
- **The messages.** Finalise the six CLI and six recipe lines from the frame's §5 table. Each says
  what to do instead. Opacity values in a message must print as the user typed them, or in a form
  that cannot mislead: test at least one value whose `f32` rendering is inexact (trap 10), such as
  `0.000001` and `0.1`.
- **Bytes.** The frame names the rows: the three watermark corpus fixtures, the CLI controls (text,
  image, `--opacity 0.6`, `--tile`, `--scale 0.1`, a 16-bit base), and the faint-but-accepted rows
  (`--opacity 0.00001`, `--size 0.2`, white on white). Specify the before/after `cmp` against a
  binary built from pre-change `main` so it can fail: name the mutation that turns it red. Note that
  the 0.8.2 bump does not touch watermark pixels, but it does change cache keys.
- **The boundary rows.** From the frame's §3: the largest opacity that is still refused and the
  smallest that is accepted, for an opaque mark on the 16-bit branch. Both are tests. A test that
  refuses `0.000001` alone cannot tell a correct threshold from "refuse everything below 0.01".
- **CHANGELOG.** The frame proposes a **Changed** entry. Scripts that used `opacity 0` as an off
  switch now exit 2 or 1, so weigh `Breaking`, as 0.8.1 did for `build`'s flags. Word the entry.
- **The DEC.** Write DEC-109, or amend DEC-104 (and DEC-106 for the order). Say why.

## Tests: each one must be able to fail
- **Every refusal row is RED on `main` today** (exit 0 there). Failing Tests records `main`'s
  measured exit code and that 0 pixels changed.
- **"Nothing written" is asserted on the files:** no output file, and inputs unchanged.
- **Each accepted faint row is a test**, and so is each boundary value.
- **Negative controls:** a normal text mark and a normal image mark still draw on every path.
- **The wasm path:** at least one refusal row through `transform` returns a thrown error, not a
  panic or abort. Say which runner drives it (`just wasm-test` or the wasm-bindgen test runner).
- **The mutation table (trap 12).** For each mutation, name the exact code location where it is
  applied, and list **every** test expected red, on both paths (CLI vs recipe, native vs wasm). At
  least: drop the predicate from `from_params`; drop it from `run_watermark`; use `visible == 0`
  instead of the post-opacity check; compare at 8 bits instead of 16 (or the reverse); move the
  flag-decidable checks after the input read.

**Size.** The frame says [S], upper end, with about 16 refusal rows. If it grows past S, use the
frame's cut (one `build` row, one wasm row, a shared message template) and report it. Keep all six
causes: dropping "too faint" reopens the gap (trap 9).

## Files
The frame's rough list:
- `src/operation/mod.rs` (the predicate, the `from_params` call), `src/cli/ops.rs`
  (`run_watermark`), `src/cli/mod.rs` (`--opacity` has no help line today; add one);
- `tests/watermark_cli_params.rs`, the recipe and `build` watermark tests, and a wasm test;
- `recipes/README.md:185` and `:236-237` (the latter promises that `opacity = 0` is drawn and must
  be rewritten), `docs/cli-reference.md`, `docs/api-contract.md`, `docs/data-model.md`,
  `skills/crustyimg/SKILL.md`, `CHANGELOG.md`;
- `decisions/DEC-109-…md`, or the DEC-104 / DEC-106 amendments.

## DECs to read
DEC-104 (coverage, decided from the mark), DEC-106 (flag checks before file reads), DEC-102 (the
recipe key table and its messages), DEC-015 (exit codes), DEC-045 and DEC-054 (the bundled font),
and any DEC naming `src/operation/mod.rs` or `src/cli/ops.rs` (`./scripts/decisions-audit.sh`).

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-143-*.md` with `cycle: design`. If
it is missing or still `cycle: frame`, stop and report; do not create it. A separate session builds
it from your spec alone, and another verifies it, so everything the builder needs must be in the
spec (AGENTS §15).

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (1–12). Binding. Traps 9, 10 and 12 apply directly.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`:
   your item, the success criteria and the Design Notes.
4. `/guidance/constraints.yaml` and every DEC named above.
5. `specs/done/SPEC-135-*.md` as the shape to match (Build Completion and Verify Readout included).
   It is the closest prior spec: watermark refusals, the same exit codes, the same two paths.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end, before you describe it. Give every measurement a negative control.
  **Measure the scenario, not the control.** Count pixels with ImageMagick or Python, never
  crustyimg.
- **Ask what the user actually sees:** the command, the stderr line, the exit code.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Every AC names the surface it applies to (CLI `watermark`, `apply`, `build`, wasm `transform`).
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump, no `CACHE_SCHEMA_VERSION` or lockfile format change.

### Ids are reserved — use exactly these
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests and Implementation Context;
keep `## Frame`. If you find the frame wrong, say so in your report rather than silently widening or
narrowing it. Highest DEC on `main` was DEC-108 when this was written. Other projects (PROJ-016,
PROJ-017) add DECs on `main` in parallel: re-check `ls decisions/` after your last rebase, and if
DEC-109 is gone, take the next free id and say so. Use the **block-list** `affected_scope` form
(an inline array is silently dropped by `scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - src/operation/mod.rs
```

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR.
1. `just advance-cycle SPEC-143 build`.
2. In STAGE-054's backlog, change **only** SPEC-143's marker from `(framed 2026-10-30)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add the
   build prompt path `prompts/SPEC-143-build.md` to the build line.
3. `git commit -s`, ending the message with
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **The DCO sign-off is mandatory.**
   Message: `docs(SPEC-143): design refusing a watermark that draws nothing`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. Other sessions
   push to `main` too; if the push is rejected, rebase and push again.
5. Read back `git show --stat HEAD` and run `just backlog`.

### Guardrails
- ⛔ No code changes outside the spec, timeline, stage file and the DEC. No version bump, no tag.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout
  to a file. A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does
  not word-split unquoted parameters. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~55 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** `grep -l "DESIGN prompt: a watermark that draws nothing" ~/.claude/projects/<slug>/*.jsonl`.

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
First line: `SPEC-143 design: <commit SHA on main> — <N> ACs, <M> failing tests`. Then, short:
- each design call and how you settled it, including the exact twelve messages;
- the predicate, and the 8-bit vs 16-bit finding with numbers;
- the cause classifier's order, and what each odd character reports;
- the boundary values and the byte-identity check, with the mutation that turns each red;
- the mutation table: each mutation's code location and every test expected red;
- the file list the build will touch;
- whether DEC-109 is used or released;
- anything that needs a **maintainer** ruling, each with your recommendation;
- whether to build on Sonnet or Opus, and why;
- your measured cost.
