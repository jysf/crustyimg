# SPEC-133 — VERIFY prompt: a watermark refuses to damage an image

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build this.

```
just wt verify/spec-133 -- --model opus
```

Then put the worktree on the PR's code (the fresh branch has nothing on it to lose):
```
git fetch origin && git reset --hard origin/feat/spec-133-watermark-damage-rule
```
PR #206 is a **draft**; its head was `3dd23ff` when this prompt was written. Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

**What the build claims:** `Watermark` refuses a mark that does not fit the image (margin counted
only on the axes the gravity anchors, size taken after `scale`) or whose non-transparent pixels
cover more than 5 % of the image (`tile` and `scale` exempt). The decision is one defaulted
`Operation::refusal`. `Pipeline::run_with(Fail|Skip)` turns it into exit 2 on one explicit input
and into a skip on every batch and `build` path: the image is written without the mark, with a
warning line and a count. A recipe `text` step now refuses `scale` and `tile = true` at load.
Passing marks are byte-identical to `main`. 27/27 ACs; N1–N9 flipped; CI green on all 16 legs.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no version bump. Do not fix what you find:
report it with evidence and a suggested fix. Do not edit the spec or timeline.

## Read first
The spec on the PR branch
(`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-133-watermark-refuses-to-damage-an-image.md`),
all of it: Design Calls 1–6, the ACs, the N1–N9 table, and `## Build Completion` with its seven
deviations. Then **DEC-104** (new), **DEC-002** (the trait), **DEC-015** (exit codes; DEC-104
amends its partial-batch rule), **DEC-085** (unconditional warnings), **DEC-102** (SPEC-131's
mode check, which Call 5 extends), and HANDOFF.md's "Traps". Read each DEC yourself; do not trust
a summary.

## Nine specific things — in this order

### 1 — ⚡ What must NOT change, driven on `main` vs the PR (release binaries, fresh target dir each)
- **Passing marks are byte-identical.** Do not reuse the build's golden (it blessed its own
  hashes). Drive at least six passing cases yourself on both binaries and compare output bytes:
  the default text on 256×256 and 800×600, an opaque image mark, a tiled mark, a scaled mark, a
  16-bit PNG base, and one `apply --recipe` with a watermark plus another step. Then confirm
  the golden was blessed on untouched `src/`: `git show --stat ab94505` touches no `src/`, and
  its parent's `src/` equals `origin/main`'s.
- **Recipes without a watermark** are unaffected: one `apply --recipe` and one `build` project,
  identical bytes on both binaries.
- **`run` keeps its signature and meaning** for every caller the diff does not touch. List every
  `Pipeline::run` call site left in `src/` and say which policy each now gets.

### 2 — The rule, driven on the binary with fixtures you build yourself
Generate your own bases and overlays (do not reuse the test files' helpers). Count the overlay's
non-transparent pixels with a tool you did not write (e.g. Python + Pillow, or ImageMagick).
Drive, and give exit code plus message for each:
- fit at **equality** passes and +1 px is refused, on each axis, through `--margin` with a corner
  gravity;
- `center` and `north` ignore margin on the axes they do not anchor;
- coverage at exactly 5 % passes and one pixel more is refused, using an overlay with a
  transparent border (the transparent border must not count);
- `--opacity` does not change the count;
- `--scale` is fit-checked after scaling, and exempt from coverage. **Also try a scale that
  rounds** (an odd overlay at 0.5, or 0.333): does the size `refusal` checks equal the size
  `apply` composites? The build says one helper, `mark_size`, serves both. Confirm by reading it,
  then by driving a case at the rounding edge;
- `--tile`: one tile must fit, coverage exempt, margin ignored;
- a mark that fails both fit and coverage reports fit.

### 3 — The image the rule sees is the one entering the step
A recipe `[resize …, watermark …]` must judge the mark against the **resized** image, not the
input. Build two recipes where the answer differs:
- the input fits the mark but the resized image does not → refused (exit 2 on one input);
- the reverse, using a resize that enlarges → applied.

AC-18's fixtures cannot tell these apart (the mark fits neither size). If no test does, it is a
finding. Mutate the check to use the original input to confirm.

### 4 — Every surface in Call 2's table
Drive each row on the PR binary and record exit code, files written, and stderr:
- **One input:** `watermark`, `apply --recipe` (plain), `apply --recipe` + `optimize` pinned and
  auto-decide. Each must give exit 2, write nothing, and print one `error:` line.
- **Many inputs:** the same four, plus `build` with a Preserve plan and with a Decide plan. Each
  must skip the mark, apply the other steps, print a warning per image, and print the count
  line. `--quiet` keeps the warning and drops the count.
- **Edges the spec doesn't name** (SPEC-134 paid for skipping these):
  - one input given as a **glob or directory that resolves to a single file**: does it take
    the one-input path (exit 2) or the batch path (skip)? Is that documented?
  - stdin (`-`);
  - `--json` and `--explain json` on a batch that skips. Is the schema unchanged? Does the
    report still say the skipped image succeeded?
  - a recipe with **two** watermark steps that both refuse on one image: is K counted per image
    (once) or per step? Is each step warned about?
  - `build --check` after a skipped build.
- **wasm:** run `just wasm-test`. Read `src/wasm.rs` and check that the diff does not touch it.

### 5 — Deviation 2 needs your ruling: an existing recipe now fails to load
Call 5 makes a recipe `text` step with `scale`, or with `tile = true`, a load error. Released 0.7.x
accepted both and applied them (the frame measured 52.7 % / 56.2 % coverage). The build removed
`scale = 1` from SPEC-131's corpus recipe `v2_watermark_integer_numbers` so that it loads.
- Is that edit right, and does the corpus still prove what it was for (integers in number keys
  stay integers)? Is integer `scale` now untested anywhere?
- On the **0.7.1 release binary** vs the PR: a hand-written recipe with `text` + `scale = 0.5`.
  What does 0.7.1 do with it, and what does the PR do (exit code, message)?
- Is this called out as a breaking change for hand-written recipes in `CHANGELOG.md`,
  `recipes/README.md` and DEC-104? A user upgrading should not find out from an exit 1. If it
  is not, that goes on the punch list.

### 6 — The tests are not hollow
- **RED baseline:** copy the PR's test files onto `main` (unmodified `src/`). Check the build's
  claims: 14 of 16 `watermark_damage` tests fail for the spec's reasons, and the wasm test
  returns `Ok`.
- **The skip probe:** repeat the build's check yourself. Make any skip abort the process, rebuild
  in a fresh target dir, and run:
  - `apply_batch`, `build_watermark_cache`, `build_watch`, `watermark`,
    `colour_type_preservation` and `recipe_watermark` must pass, which shows they still draw the
    mark;
  - the skip tests must hit the abort.
- **Search for others the build missed:** list every test in `tests/` that runs a watermark on
  a batch or `build` path. A hollow pass is one where the mark is skipped and the test stays
  green. For each test, say whether the probe covered it.

### 7 — Your own mutations, beyond N1–N9
Run at least four. Use a **fresh** target dir for each mutation: reverting source does not
rebuild the binary. Suggested mutations:
- count coverage after `opacity`, or at 8 bits for a 16-bit overlay;
- change `>` to `>=` on the fit or coverage comparison (do the equality rows catch it?);
- count K per step instead of per image;
- emit the skip line on a `build` cache hit;
- the original-input mutation from item 3.

Name any mutation that no test catches.

### 8 — Boundaries, records, code quality
- **Diff size:** `git diff --name-only origin/main...HEAD` should list the 24 files in Build
  Completion. Check:
  - `src/wasm.rs`, `src/build/cache.rs` and the lockfile code are untouched;
  - `CACHE_SCHEMA_VERSION` is unchanged;
  - SPEC-134's `name_pinned_format` and `resolved_input_count` are untouched;
  - `tests/name_template_pins_format.rs` is unchanged and green.
- **Records:**
  - DEC-104 has a block-list `affected_scope` and `supersedes: null`. It states the amendment
    to DEC-015's partial-batch rule and the cached-run limitation.
  - `./scripts/decisions-audit.sh --changed origin/main` passes. Judge the overlap warnings.
  - No SPEC or DEC id appears in any comment or user-facing string the diff adds.
- **Code quality** (the maintainer's bar: "code quality is a must"):
  - `optimize_decide_one` now returns a 6-tuple. Should it be a named struct, the way
    `DecidedEncode` is?
  - Are the warning and count lines printed by one helper, or copied into each batch loop?
  - Is `Watermark::apply`'s own refusal guard a duplicate of `run_with`'s check, and is that
    duplication right?

  Report each as ⚠ only if it is wrong or likely to drift into a bug. Otherwise list it as an
  optional cleanup.

### 9 — Docs have no green
Drive every claim the diff adds to these files: `docs/api-contract.md` (the rule, exit 2, the
skip line, the count, the cached run), `docs/data-model.md`, `recipes/README.md`, the
`watermark --help` sentence, and both `CHANGELOG.md` lines. List any claim that is false. The
`--size nan` / `--size inf` / `--scale nan` item stays open, as the build reports. Confirm it on
the binary and check that no doc claims it is closed.

## Also
- **CI:** run `gh pr checks 206` at the head you verified. Check every leg: windows-latest,
  lean, MSRV, the feature legs and browser smoke. Do not poll. If any leg is pending, run
  `gh pr checks 206 --watch` in the background and read a direct snapshot when it exits.
- **Matrix once, yourself:** default, `--no-default-features` and `--features webp-lossy`. Give
  each leg a fresh `CARGO_TARGET_DIR` and run them one after another. Each leg runs the tests
  and `clippy --all-targets -D warnings`. Then run `fmt --check` and `just demo-build`.

## Guardrails
- **Read-only. No commits. Do not fix what you find.** Use scratch directories for every project
  you drive, and remove any scratch worktree you add.
- **Delete each target dir when you are done with it.** A recent session filled the disk.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file.
- A piped command reports the pipe's exit code: redirect output and read `$?`.
- zsh does not word-split unquoted parameters: pass feature flags as separate literal arguments.
- macOS has no `timeout(1)`.
- To get the 0.7.1 release binary, use an installed `crustyimg` and check its `--version`, or the
  GitHub release asset. A `main` build also reports 0.7.1, so don't use one for this.
- **Budget ~150 exchanges.** If you pass it, stop and report what you covered.

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content,
not recency**: `grep -l "SPEC-133 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran, never at a flat rate: Opus $5/$25 per MTok,
Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10. Report
`tokens_total`, a `tokens_breakdown` and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #206 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #206 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- Lead with **item 1**: are passing marks identical, yes or no for each case?
- Then **item 5**: your ruling on the recipe break.
- Then items 2–4 and 6–9, kept short.
- A punch list is numbered, with evidence and a suggested fix for each entry.
- List optional cleanups separately.
- End with your cost.
