# SPEC-142 — DESIGN prompt: the recipe the ddphotos page emits gives the same pixels natively and in wasm

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-142-design -- --model opus
```

Starts from the stub the frame pushed to `main` (`9174087`). **Reserved: SPEC-142.** The frame
found no DEC is needed. Write one only if the design forces a tolerance on "same recipe, same
pixels" (see "Ids" below). SPEC-141, the recipe-output guard, may be in design at the same time.
It touches different files.

**Read the stub's `## Frame` section first.** It holds the 48-row premise check (native and wasm
agree exactly on today's page recipes on the Mac), the property, the per-format rule, where the
harness runs, both driven controls, the sizing, the scope and the rough file list. It is your
binding scope.

## Rulings (settled — do not re-open)
The maintainer ruled on 2026-10-30, accepting all three of the frame's recommendations:
1. **A new `ci.yml` job, plus a `just` recipe** that runs the same harness locally. Not a `just`
   recipe alone.
2. **Move the recipe code into `integrations/ddphotos/edit/recipe.js` now.** `pixelRecipe`,
   `cliRecipe`, `watermarkSize`, and the `readEdit` arithmetic the frame's probe had to copy, if it
   belongs there. The page and the harness both import it. No build step: the page stays a static
   file set.
3. **Leave STAGE-053's wasm-test leg out of this job** (the chore "A wasm32 CI leg running `just
   wasm-check` + `just wasm-test`"). Shape the job so that chore becomes **one added step** later,
   and say in the spec what that step would be.

Also accepted from the frame:
- **The property is engine parity, exactly.** The same recipe on the same input bytes gives equal
  dimensions and zero differing pixels through wasm `transform` and native `apply`. It covers both
  `pixelRecipe` and `cliRecipe`. `cliRecipe` rows run as PNG, which drops the terminal `optimize`.
  **Replay parity** (a 4× original) belongs to STAGE-060's crop spec, with the frame's two cautions
  recorded there. Don't build it here.
- **Per-format rule:**
  - PNG: bytes and dimensions;
  - JPEG: decoded pixels, with bytes reported and asserted only if the x86 run proves they match;
  - WebP: decoded pixels, never bytes;
  - AVIF: decoded pixels, one row only.

  The required rows are the page's own paths: PNG (the copy) and JPEG at quality 88 (the preview).
- **The `wasm` side runs the slim build the page ships**, as the frame did. Say whether the full
  build also gets a row.
- **Compare dimensions first, then decoded RGBA directly.** ImageMagick 7.1.2's `-metric AE` gives
  no pixel count. Of the frame's negative controls, a width change went red only through the
  dimension check.
- `ci.yml`'s changed-paths filter (`changes` job, the "Classify changed paths" step, about
  `:31–57`) must count `integrations/ddphotos/` as a code path. Today an edit to `edit.js` alone
  skips every heavy job.

## Design calls that are yours
- **The x86 unknown is resolved in build, not here.** Every frame row ran on aarch64. On x86_64,
  the JPEG decoder and the AVIF encoder can take SIMD paths the wasm build can't. Don't push code
  to find out. Write the spec so **both outcomes are pre-specified**:
  - (i) x86 agrees: the rule as above;
  - (ii) x86 differs on a format: name what the row falls back to. Pixels-not-bytes is fine for
    JPEG. A **pixel** difference on PNG or the decoded rows is a finding to report, not something
    to loosen.

  The build's **first checkpoint** is a WIP draft PR with the harness and the CI job. Its first
  x86 run decides (i) or (ii). The build reads it with a backgrounded `gh pr checks --watch`,
  never a poll loop.
- **The independent decoder.** The harness compares decoded RGBA that **neither side's crustyimg
  decoded**. Name the decoder:
  - a Node package, which is a new dependency in a repo with no `package.json` for tests (check
    for one);
  - Python + Pillow, or ImageMagick on the runner (check what `ubuntu-latest` actually has
    installed, from the runner-images manifest, not from memory);
  - a minimal in-harness PNG reader, which works only for PNG rows.

  Choose one, cite the evidence, and say how the CI job installs it.
- **How `edit.js` imports `recipe.js`** without a build step: an ES module (`<script
  type="module">`), and whether `worker.js` needs it. Check the installer docs
  (`integrations/ddphotos/README.md`) and the two test-bed rounds (#224, #227) for anything that
  assumed one file. Say what the browser load check is and who runs it. ⛔ Never drive the
  maintainer's live browser: a headless Node import of the module, plus the test-bed round on
  STAGE-060's last item, may be enough. Say which.
- **The fixtures** (`tests/fixtures/ddphotos/`) are made outside crustyimg: ImageMagick plus
  exiftool, including an EXIF-orientation-6 JPEG. Say whether to commit the bytes (recommended, so
  CI needs no generator) or a script, their sizes, and the licence or provenance line.
- **The CI job**: the toolchain, wasm-pack/wasm-bindgen pinning (match `pages.yml` and
  `wasm-bindgen-cli 0.2.126` from `justfile`), caching, and the measured or estimated minutes.
  It must not lengthen time to green; show it runs in parallel. Name the one step STAGE-053's
  chore would add.
- **The row table.** The frame says about 16 automated rows on one fixture pair. Write the table
  out: edit × recipe function × format × assertion. The harness asserts that **row count as a
  literal**.

## Tests: each one must be able to fail
HANDOFF trap 12 binds: **each mutation row names the code location where it is applied, and lists
every row expected red and every row expected green.**
- **Negative control 1 (driven at frame):** native-side opacity 0.6 → 0.61 goes red on the
  watermark rows only (largest difference about 3/255). Name where the mutation is applied. It must
  be in the harness's native input, not in `recipe.js`, or both sides move together and the
  control proves nothing.
- **Negative control 2:** width 1024 → 1023 goes red on the dimension check. List its rows.
- **Shared-source control:** a mutation in `recipe.js` itself changes both sides and must stay
  **green** on parity. Make it go red on a positive control instead: the dimensions match the
  page's own arithmetic, asserted as literals. Otherwise a broken `recipe.js` passes.
- **Positive controls:**
  - each output's dimensions match the page's arithmetic;
  - a resize row changes size;
  - a watermark row's pixels differ from the unwatermarked row;
  - the row count equals the table size.
- **A harness that runs nothing must fail.** If the wasm module fails to load, or the recipe
  import throws, the job is red, not "0 rows, pass".
- **CI path filter:** an `edit.js`-only change classifies as code. Show how this is tested (a
  dry-run of the classify step on a fake diff, for example).

**Size.** The frame says S, at the top end. If it grows, the frame's cuts apply in this order:
1. leave the `recipe.js` move to the crop spec and slice `edit.js`;
2. hand the CI job to STAGE-053's chore.

Both reverse a maintainer ruling, so **report the cut and stop. Don't apply it.**

## Files
The frame's rough list:
- `integrations/ddphotos/edit/recipe.js` (new) and `edit/edit.js` (and `worker.js` if it builds
  recipes)
- `tests/ddphotos_parity.mjs` (new)
- `tests/fixtures/ddphotos/` (new)
- `justfile` (a `ddphotos-parity` recipe, or a name you choose)
- `.github/workflows/ci.yml` (the job and the path filter)
- `integrations/ddphotos/README.md` (the new file in the install list, and how it's tested)

## DECs to read
DEC-064 (wasm codec parity), DEC-066 (the wasm size profile), the DEC behind `transform` (grep
`decisions/` for `wasm`), and any DEC whose scope names `ci.yml`, `integrations/` or `src/wasm.rs`
(`./scripts/decisions-audit.sh`).

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-*.md` with `cycle: design`. If it is
missing or still `cycle: frame`, stop and report; do not create it. A separate session builds from
your spec alone, and another verifies it, so everything the builder needs must be in the spec
(AGENTS §15).

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (traps 11 and 12 especially). Binding for PROJ-017.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`.
4. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-053-guards-that-cover-this-project.md`,
   item 1 and the wasm32 CI leg chore, so this job is shaped to take it.
5. `/guidance/constraints.yaml` and every DEC named above.
6. A recent shipped spec as the shape to match:
   `projects/PROJ-011-surface-reach-and-predictability/specs/done/SPEC-135-*.md`.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive the current behaviour** with a native release build in a fresh `CARGO_TARGET_DIR` and
  a wasm build, then delete both target dirs. Re-use the frame's scratchpad scripts only after
  reading them. Give every measurement a negative control. **Measure the scenario, not the
  control.**
- **Count the nouns in every AC against the Failing Tests list.**
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No change to `src/`, and no shipped-binary change. If the design finds one is needed, that's a
  frame error: report it.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context;
keep `## Frame`. If you find the frame wrong, say so in your report rather than silently widening or
narrowing it. If you must write a DEC, re-check `ls decisions/` after your last rebase and take the
next free id. Use the **block-list** `affected_scope` form.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-142 build`.
2. In STAGE-060's backlog, change **only** SPEC-142's marker from `(framed 2026-10-30)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add the
   build prompt path `prompts/SPEC-142-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-142): design the ddphotos parity harness`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. SPEC-141's
   design touches STAGE-058, not STAGE-060, so the rebase should be clean.
5. Read back `git show --stat HEAD`. ⚠ `just backlog` reads only the active project (PROJ-011),
   so use `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline, the stage file and (only if needed) a DEC.
  Don't edit `integrations/ddphotos/`, `ci.yml` or `justfile`; the build does that. No pushed
  probe branch, no version bump, no tag.
- ⛔ Never drive the maintainer's live browser.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- If SPEC-141's design is running beside you, never share a `CARGO_TARGET_DIR` with it.
- **Budget ~50 exchanges.** The wasm build is the slow step, so start it early and in the
  background.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-142 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and cache_read
is ×0.10. Append a `design` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown`, `estimated_usd` and the rate anchors, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-142 design: DONE | BLOCKED`, then the commit SHA on `main`. Then, short:
- the AC list (one line each), the row table, and the mutation table with the rows expected red
  and green;
- the independent decoder, and how CI gets it;
- the `recipe.js` import shape, and the browser load check;
- the CI job's shape, its minutes, and the step STAGE-053's chore would add;
- both x86 outcomes, and the build's first checkpoint;
- size, and any place you disagree with the frame;
- anything the build prompt must carry.

Then your measured cost.
