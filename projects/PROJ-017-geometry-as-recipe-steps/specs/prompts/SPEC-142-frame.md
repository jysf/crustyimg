# SPEC-142 — FRAME prompt: the recipe the ddphotos page emits gives the same pixels natively and in wasm

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-142-frame -- --model opus
```

**Prerequisite:** PR #231 (which frames PROJ-017) is merged to `main`. If
`projects/PROJ-017-geometry-as-recipe-steps/` isn't on `origin/main`, stop and say so.

**Reserved: SPEC-142.** Write no DEC at frame. Design decides whether one is needed (see question
6).

The id needs care. `main` ends at SPEC-138. SPEC-139 is taken on branch
`claude/hungry-swanson-c33241`. SPEC-140 is left for PROJ-016 (DEC-107 names it). SPEC-141 is
the no-bytes-guard frame, which may run at the same time as this one. Before you allocate,
confirm SPEC-142 is unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do git grep -l 'SPEC-142' "$b" -- . ; done
```

The only hits should be this prompt and PROJ-017's planning files. If a spec file or another
reservation already uses SPEC-142, stop and report. Don't pick another number.

Your item is the **first** entry in STAGE-060's Spec Backlog: the native-vs-wasm parity harness.
Read it in full. Touch no other item.

## Why this spec exists
PROJ-017's thesis is that an edit made on the ddphotos edit page is a recipe, and the CLI replays
that recipe on the original with the same result. The page runs crustyimg compiled to wasm in
the browser. The CLI is the native build. They're built with different feature sets (the wasm
build drops threads and some codecs, and its AVIF encoder has no rayon), so "the same recipe gives
the same pixels" is a claim about two binaries. Nothing checks it today.

This spec builds the check **before** the page gains crop (build-order line 7) and straighten
(line 12), and it must be green on today's recipes. Then, when crop lands, a red harness means
crop broke parity, not that the harness was never right. It's build-order line 3 of 13, has no
dependency, and can run beside the no-bytes guard (SPEC-141). Keep it **[S]**.

## What already exists (read it, don't assume it)
- **`tests/wasm_roundtrip.rs`** holds `#[wasm_bindgen_test]` cases that run in Node through
  `just wasm-test` (plain cargo with `wasm-bindgen-test-runner`, not `wasm-pack test`). It
  asserts formats, dimensions and errors on the real wasm surface. As far as a quick read shows,
  it compares nothing to a native result. Check that.
- ⚠ **`just wasm-test` doesn't appear to run in CI.** `ci.yml` has no wasm leg. Only `pages.yml`
  builds wasm, for the demo. Confirm it. A parity check that runs only when someone remembers is a
  claim, not a gate.
- **`src/wasm.rs`, `transform(input, recipe_toml, out_format)`** (about :206) is the page's only
  pixel entry point. The page calls it from `integrations/ddphotos/edit/worker.js` (about :56 for
  the preview, :66 for the full result).
- **The page writes two different recipes** (`integrations/ddphotos/edit/edit.js`, about
  :266–292):
  - `pixelRecipe(edit)` is what wasm runs, on the **published** copy: `auto-orient`, an optional
    `resize` with `mode = "max"`, and an optional text `watermark`.
  - `cliRecipe(edit)` is what the page hands the user for the CLI, run on the **original**:
    the same steps but resized to the long edge the page made, plus `format` and a terminal
    `optimize`.

  So today's CLI replay is a different recipe on a different (larger) source. Pixel equality
  with the page isn't even the intended property for that pair.

## Questions this frame must answer
1. **The property, precisely.** Name what the harness asserts. There are at least two candidates:
   - **(a) Engine parity:** the same recipe on the same input bytes gives the same pixels through
     wasm `transform` and the native pipeline (or `crustyimg apply`).
   - **(b) Replay parity:** `cliRecipe` on a large original gives the image `pixelRecipe` gives on
     the published copy, at the same output scale, within a stated bound.

   (b) is what PROJ-017's thesis promises, and it's where a crop written in preview pixels would
   fail. (a) is exact and cheap. Recommend what this spec covers, with the bound for each, and
   where the rest lives. STAGE-060's crop spec is the obvious home for (b) on crop.
2. **Premise.** Drive (a) on today's recipes: run `pixelRecipe`'s output for a few edits (resize
   only, a watermark, both) through `just wasm-test` or a Node script over `pkg/`, and through a
   native release build, on one independent input. Compare decoded pixels with a tool that isn't
   crustyimg (Python or ImageMagick). Record the exact command and output. If they differ, that's
   a finding: say which step differs and by how much. Don't fix it.
3. **Pixels or bytes.** For PNG output, compare decoded pixels and the encoded bytes. For JPEG,
   WebP and AVIF, say which comparison is meaningful. The wasm AVIF encoder runs without rayon, so
   check whether its bytes can match native at all (`ravif`'s rayon-off shim: a disabled feature
   can swap in a different module). Recommend one rule per format.
4. **Use the page's own recipes, not copies.** The harness must get its recipes by calling
   `pixelRecipe` and `cliRecipe` from `edit.js` (load the file in Node, or factor the two
   functions into a module the page and the harness both import). It must not use a hand-typed
   TOML that can drift from the page. Say which, and what it costs the page. The page is a static
   file set with no build step, and that's a property to keep.
5. **Where it runs, and that it can fail.**
   - Recommend whether the harness goes into CI (a new wasm job: estimate its minutes from
     `pages.yml`'s wasm build) or stays a `just` recipe the STAGE-060 specs must run and paste.
   - Name the negative control design must require. For example, feed the native side a recipe
     with one parameter changed and show the harness goes red, naming the recipe and the
     difference.
   - Name the positive control: a dimension or pixel-change assertion, so a harness where both
     sides do nothing can't pass.
   - The repo has a lesson here: wasm32 compiling doesn't mean wasm32 runs, so run in the runner
     and compare against a native hash.
6. **Does this need a DEC?** If the page's recipe functions move into a shared module, or a CI job
   is added, check whether that's a decision under `guidance/constraints.yaml` or an existing DEC
   (DEC-066 for the wasm size profile, DEC-064 for wasm codec parity). Recommend one way or the
   other. Write none.
7. **Size.** Target **[S]**. If (b) or a CI job makes it larger, recommend the cut.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context.
Separate design, build and verify sessions follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 backlog, §13 git, §15 cycle rules).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the first risk (preview vs
   original units) and the build order.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-060-ddphotos-edit-page-crops-and-straightens.md`:
   your item in full, and the fixed page step order.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for". PROJ-017 has no HANDOFF of its own yet, so these rules bind here.
5. `integrations/ddphotos/README.md` ("How it was tested"), `edit/edit.js`, `edit/worker.js`.
6. DEC-064, DEC-066, and the DEC behind `transform` (find it with `./scripts/decisions-audit.sh`
   or by grepping `decisions/` for `wasm`).
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Build natively in a fresh
  `CARGO_TARGET_DIR` and build wasm with `just wasm-build` or `just wasm-test`. Delete both
  target dirs at the end. Record exact commands and output. **Measure the scenario, not the
  control.**
- **Size the verification, not just the code** (AGENTS §2). Count the edit × format × property
  matrix.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Allocate the id
`just new-spec` allocates by scanning the working tree, so it will hand out SPEC-139. Run
`just new-spec "<title>" STAGE-060 PROJ-017`, then `git mv` the spec and its timeline to SPEC-142
and fix the id inside both. If SPEC-141's frame has already landed, `new-spec` still gives a wrong
number, so check it either way. Then run `just advance-cycle SPEC-142 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (decisions and related specs), and `cycle: frame`.
- `## Context` and `## Goal`: what is and isn't checked today (your reproduction), and what a
  STAGE-060 builder sees when this ships (the command, and the red output naming an edit).
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the property (a and/or b) and its bounds;
  - the per-format comparison rule;
  - where it runs;
  - sizing and why;
  - scope in/out;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-142-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-142 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-060's backlog, change **only your item's** `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-142** (framed <date>)`. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of the stage it belongs to, and update that stage's
   `**Count:**` line with a grep you've just run. Something with no stage goes in
   `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-142): frame the ddphotos page native-vs-wasm parity harness`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. If SPEC-141's
   frame landed first, the rebase is clean: the two specs touch different stage files.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-060 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. Scripts you write to drive the premise
  stay in a scratch directory and are never committed. Don't edit `integrations/ddphotos/`.
- ⛔ Never drive the maintainer's live browser. Node and `wasm-bindgen-test-runner` are enough.
- **Concurrent builds.** If the SPEC-141 frame is running beside you, never share a
  `CARGO_TARGET_DIR` with it.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~50 exchanges.** The wasm build is the slow step, so start it early and in the
  background.

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-142 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and
cache_read is ×0.10. Before your final commit, append a `frame` entry to the spec's
`cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate anchors.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-142 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the premise check: did native and wasm agree on today's recipes, with one line of evidence per
  edit;
- the property this spec asserts, and where the rest lives;
- the per-format rule, and where the harness runs;
- the negative control, and whether you drove it;
- size and why; scope in/out;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them.

Then your measured cost.
