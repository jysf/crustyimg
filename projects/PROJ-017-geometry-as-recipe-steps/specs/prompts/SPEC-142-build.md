# SPEC-142 — BUILD prompt: the ddphotos page's recipes give the same image natively and in wasm

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design gives the harness's
shape, the 16-row table, the comparator rule, the CI job's YAML and a mutation table. One
question is still open, and the first x86_64 CI run answers it: is the native-vs-wasm difference
within Rule B there too?

```
just wt test/spec-142-ddphotos-parity -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that
message must stand on its own. This is line 3 of PROJ-017's build order. STAGE-060's crop and
straighten specs add rows to this harness later.

## What you are building
- `integrations/ddphotos/edit/recipe.js`: the page's recipe code moved into one DOM-free module.
  `edit.js` and `worker.js` import from it.
- `tests/ddphotos_parity.mjs`: runs 16 page recipes through native `apply` and through the slim
  wasm `transform`, decodes both outputs with Pillow, and compares them under **Rule B
  (DEC-110)**. It has a `--self-test`.
- `tests/ddphotos_decode.py`: the independent decoder.
- `tests/fixtures/ddphotos/`: two committed fixtures and a provenance README.
- `scripts/ci-code-paths.sh`: the `changes` job's path classifier, with a `--self-test`.
- A `just ddphotos-parity` recipe, and an advisory `ddphotos-parity` job in `ci.yml`.
- A README line in `integrations/ddphotos/README.md`.
- **No `src/` change.**

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build` with `blocked: false`;
- the amend commit `a616840` is in your history;
- `decisions/DEC-110-*.md` exists.

If any check fails, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-ddphotos-page-native-vs-wasm-parity-harness.md`.
   Read all of it:
   - `## Frame` and both rulings blocks;
   - `## Design finding` (kept as the record);
   - AC-1 to AC-13;
   - FT-1 to FT-13, the row table, the golden recipe texts and the mutation table;
   - `## Implementation Context` and `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in
   your report; don't quietly re-decide it.
2. **DEC-110**, in full, before you write the comparator. Also read DEC-064, DEC-065, DEC-066 and
   DEC-059 yourself.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 10, 11 and 12 especially). These bind PROJ-017. Also read
   `/guidance/constraints.yaml`.

## Rulings (settled — do not re-open)
1. A new `ci.yml` job plus a `just` recipe. The job is advisory, never a required check.
2. Move the recipe code into `recipe.js` now.
3. STAGE-053's wasm-test leg stays out of this job. Leave only the YAML comment marking its spot.
4. **Rule B (DEC-110):**
   - dimensions are always exact;
   - rows with no `resize` step (1, 3, 9, 11) are exact;
   - resampling rows may differ by ≤ 1/255 per channel on ≤ K pixels, with **K ≤ 78**.
   - The cap is never raised. Anything over it is BLOCKED.
5. **Orchestrator ruling (2026-10-30), already in the spec.** JPEG bytes are asserted only on
   rows 9 and 11, and only under outcome (i). Rows 10 and 12 resample, so their bytes are
   reported, the same as the resampling PNG rows. The amend session flagged that asserting them
   was stricter than the rule.

## New evidence since the design (from SPEC-141's CI run, PR #233)
SPEC-141 pinned native pixels on all three CI targets. That tells you what to expect on x86:
- **Every recipe that resamples gives different native pixels on Linux, Windows and macOS.**
  - This includes a watermark recipe that only scales its logo.
  - Linux and Windows are both x86_64 and still differ from each other.
- **The text watermark and the no-resize path gave identical native pixels on all three.** So
  rows 1, 3, 9 and 11 are likely to stay exact on x86.
- **The resampling rows' K on x86 is genuinely unknown.** wasm's arithmetic doesn't change with
  the host, so native Linux and Mac can't both be 1 pixel from it. Expect K to come out above
  aarch64's 1. It may come out above the cap, in which case you stop at BLOCKED, as designed.
- **The resizer picks its SIMD path from the CPU at run time**
  (`fast_image_resize::Resizer::new()`, `src/operation/mod.rs:586`). So K may depend on the
  runner's CPU model, not only on its OS. Record which CPU each run had: add a first step to the
  job, `grep -m1 'model name' /proc/cpuinfo; grep -ow 'avx512f\|avx2\|sse4_1' /proc/cpuinfo | sort -u`.
  Put it in Build Completion next to K.
- **Print each row's wasm-side output SHA-256** in the run log, on both targets. Then Build
  Completion can say whether wasm itself gave identical bytes on the Mac and on the runner. If
  it didn't, that's a finding: report it, don't explain it away.

## Order
1. **`recipe.js` and the module check first.**
   - Move the functions exactly as the spec's "New exports" lists them, `planEdit` line for line
     from `readEdit`.
   - FT-10's checks: import in Node; each page file copied to a temp `.mjs`, then `node --check`;
     the imported names are a subset of the exports.
   - **Paste the golden recipe texts (FT-8) from your own first run of `recipe.js`.** Then diff
     them against the spec's two texts. A difference is a finding, not a reason to edit the
     spec's text.
2. **The fixtures.**
   - If ImageMagick 7 and exiftool are present, make them with the spec's exact commands, and
     record the versions in the fixture README.
   - Otherwise stop and report. Never generate them with crustyimg.
   - Record the sizes and SHA-256 of both files.
3. **The harness, then `--self-test`.**
   - The comparator is **one pure function** of `(dims, bytesEqual, px, max, resamples, format)`.
   - `K` and the cap `78` are named constants at the top, each with a comment that says what
     they bound. Assert `K <= 78` at start.
   - Derive "resamples" from the recipe text, and assert that the no-resample set is
     `{1, 3, 9, 11}`.
   - Use a fresh `fs.mkdtempSync` dir per run, because `apply` exits 5 rather than overwrite.
     Remove it on success; keep it and print its path on failure.
   - Every argument is required: `--pkg`, `--bin`, `--python`.
   - Build into `target/ddphotos-parity/`, never into the repo's `pkg-slim/`. Build the slim
     package through `wasm-build-slim`'s feature set and profile.
   - Make `just ddphotos-parity` green locally. **Set K to the aarch64 maximum (1).**
4. **Checkpoint 1: a draft PR, early.**
   - Commit with `git commit -s`, push, and open the PR **as a draft**.
   - Start `gh pr checks <PR> --watch` **in the background**. Never poll.
   - When it exits, read the `ddphotos-parity` job's log at the PR's true head SHA, every row
     line and the CPU line.
   - Then apply the spec's "x86_64: both outcomes pre-specified":
     - **(i)** every row is within Rule B, and JPEG rows 9 and 11 are byte-equal: assert those
       bytes.
     - **(ii)** JPEG bytes differ, but the pixels are within the bound: pixels only.
     - Either way, set K to the max over both targets, push, and get a second green run.
     - **Over the bound is BLOCKED.** That means any difference on rows 1, 3, 9 or 11, above
       1/255 on any row, above 78 pixels, or any size difference. Push the WIP, and report the
       row, its numbers, whether its PNG twin moved, and the runner's CPU line. Don't loosen
       anything.
   - Record the job's wall time (cold, and warm if a second run gets a cache).
5. **The path classifier (AC-11).**
   - `scripts/ci-code-paths.sh` holds the one regex, and the `changes` job runs its
     `--self-test` first.
   - ⚠ A broken classifier silently skips every code leg. After the change, confirm from the PR's
     own run that the matrix legs **ran**, not skipped. This PR touches `integrations/` and
     `tests/`, so it must count as code.
6. **The mutation table, one row at a time** (HANDOFF trap 12). This covers N1, N2, S1–S5,
   H1–H4, C1–C2, and B-max, B-count, B-edge, B-exact, B-dims and B-derive. For each row:
   - apply it where the table says, and check `git diff` is non-empty;
   - run the harness (and `--self-test` where the row names it);
   - record exactly which rows and cases went red, against the table;
   - restore it, and show it green.

   Notes:
   - N1 and N2 are applied in the harness, after the recipe text leaves `recipe.js`, never in
     `recipe.js`.
   - Any row that names native code: rebuild the binary **clean**, because reverting source
     doesn't rebuild it.
   - S3's FT-8 is listed as "the build confirms". Confirm it.
   - **FT-11:** drive each case by hand, and paste the exit code and message.
7. **The browser round (AC-8, FT-13).**
   - Run the test bed's `just retest <your worktree path>` once: headless Chromium on its own
     servers. **Never drive the maintainer's browser.**
   - Paste the pass/fail counts per server.
   - The test bed's HTTP check names files one by one, so it won't list `recipe.js`. Report that
     as the test bed's gap. **Never edit the test bed.**
8. **Clean checks.**
   - The default `cargo test` leg, once, in a fresh `CARGO_TARGET_DIR`. Nothing in `src/` moved,
     so it's a regression check.
   - `node --check` on every `.mjs`/`.js` you touched.
   - `shellcheck scripts/ci-code-paths.sh`, if it's installed.
   - `git diff --stat origin/main...HEAD -- src/ Cargo.toml Cargo.lock` must be empty. Paste it.

## The things most likely to go wrong
- **Two copies of the recipe code.** AC-1 says the harness re-types nothing. If the harness
  computes a size or a recipe itself, a bug in `recipe.js` moves both sides together and stays
  green.
- **A comparator that never compares.** B1–B7 exist because "every row passed" proves nothing.
  B-edge also turns the plain run red on the target where K was observed. Show that line.
- **A printed skip is invisible** (trap 11). There is no skip path: a missing package, binary,
  Pillow or export exits non-zero (AC-7).
- **A binary off `PATH`.** Homebrew's 0.8.x reports the same version as this checkout. Use the
  release binary from `target/ddphotos-parity/`.
- **Rebuilding the fixtures changes their pixels.** Commit the bytes once, and don't regenerate
  them to make a row pass.
- **Stay in your files** (the spec's Outputs). Anything else is a deviation, so say why.

## Guardrails
- ⛔ No `src/` change. No version bump, release or tag. No change to `Cargo.toml` or `Cargo.lock`.
  Don't edit the test bed.
- **Every commit uses `git commit -s`.** The DCO check fails a PR with an unsigned commit.
- **Never poll CI.** Run `gh pr checks --watch` in the background only.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no
  `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Run Python that reads test outputs with `python -I`.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids. The one exception
  is the comment on K and the cap, which may name DEC-110, as the spec says.
- **Budget ~90 exchanges.** Outcome (i) or (ii) costs a second CI round. If you're past budget
  with the harness green on CI, stop and report what's left.

## When you finish
1. **Write K back into DEC-110's Validation section:**
   - the value;
   - the targets (aarch64 locally, plus the x86_64 runner and its CPU line);
   - the CI run id.

   Don't touch the rule or the cap.
2. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory;
   - the empty `src/` diff;
   - the outcome, (i) or (ii), with K, both targets' per-row `px`/`max`, the wasm hashes on each
     target, and the job's wall time;
   - every mutation row with its red and green sets, against the table;
   - FT-11's four cases and FT-13's counts;
   - the golden recipe texts' diff against the spec;
   - a build-phase reflection that names any design call that turned out wrong.
3. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-142 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
     MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
4. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-142 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator
     marks it ready. Never mark it ready yourself.
   - Don't edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `test(ddphotos): the page's recipes give the same image natively and in wasm (SPEC-142)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-110, DEC-064, DEC-065, DEC-066, DEC-059;
     - Constraints checked;
     - New decisions: none (DEC-110 gains its measured K);
     - the browser-round counts;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-142 build: draft PR #NNN at <head SHA> — CI <green | red: jobs> | BLOCKED: <row>`.
Then, briefly:
- the outcome, (i) or (ii), with K, the runner's CPU line, and the job's wall time;
- each target's per-row results (`px`/`max`) for the resampling rows;
- whether wasm's output hashes matched across the two targets;
- AC completion (13 rows);
- each mutation row's flips against the table, and any row that didn't match;
- FT-11 and FT-13;
- the empty `src/` diff, and files changed against the spec's list;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost.
