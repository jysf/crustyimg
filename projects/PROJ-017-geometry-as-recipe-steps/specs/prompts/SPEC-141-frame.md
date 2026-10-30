# SPEC-141 — FRAME prompt: no existing recipe's output changes (the PROJ-017 guard)

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-141-frame -- --model opus
```

**Prerequisite:** PR #231 (which frames PROJ-017) is merged to `main`. If
`projects/PROJ-017-geometry-as-recipe-steps/` is not on `origin/main`, stop and say so.

**Reserved: SPEC-141.** Write no DEC at frame. This spec should need none.

The id needs care. `main` ends at SPEC-138. SPEC-139 is taken on branch
`claude/hungry-swanson-c33241` (a lint spec). DEC-107 names SPEC-139 and SPEC-140 for
PROJ-016's two RAW specs, so leave **SPEC-140** for PROJ-016. SPEC-142 is reserved for the
parity-harness frame, which may run at the same time as this one. Before you allocate, confirm
SPEC-141 is unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do git grep -l 'SPEC-141' "$b" -- . ; done
```

The only hits should be this prompt and PROJ-017's planning files. If a spec file or another
reservation already uses SPEC-141, stop and report. Don't pick another number.

Your item is the **first** entry in STAGE-058's Spec Backlog: "**The no-bytes-changed guard,
first.**" Read it in full. Touch no other item.

## Why this spec exists
PROJ-017 adds five geometry ops (`crop`, `rotate`, `flip`, `pad`, `trim`) as recipe steps and
`edit` flags. One of its success signals is that **no existing recipe's output bytes change**:
every op is new, so no lockfile is invalidated and no cached artifact goes stale. Today that's a
claim. This spec makes it a gate, built **before** any op, so each later PROJ-017 spec can show
it moved nothing.

It's build-order line 1 of 13 (PROJ-017 brief, "Build order"). Lines 2 (`crop`) and 4
(`rotate`/`flip`) wait on it. Keep it **[S]**.

## What already exists (read it, don't assume it)
Two pins exist already. Neither one is this guard.
- **`tests/recipe_round_trip.rs`, `to_toml_is_byte_identical_to_main_over_the_corpus`** (about
  :319), over `tests/fixtures/recipe_corpus/` (10 `.toml` + `.golden` pairs, plus
  `bundled_{web,product,gallery}.golden`). It pins each recipe's **serialized TOML**, which feeds
  the cache key and lockfile hash. It doesn't run the recipe, so it can't see a pixel change. A
  `CRUSTYIMG_BLESS` test (about :374) rewrites the goldens.
- **`scripts/cli-golden.py`** (SPEC-097, 440 lines, stdlib Python). It runs a fixed command set
  through a binary and diffs stdout, stderr, exit code and **the hash of every file written**
  against a captured oracle. Its fixtures come from a hand-written pure-Python PNG writer, so they
  don't depend on the binary under test. It was built to prove a refactor changed nothing. It
  isn't in `justfile` or CI (check this). Its case list covers `info`, `diff`, `build`, `lint` and
  some `apply`. Check which recipes it runs.

The likely shape is to run every corpus recipe (and the bundled ones) on an independent fixture
and pin a hash of the output. That's either a third pin on the existing Rust corpus test or a
set of new `cli-golden.py` cases. Confirm or refute it.

## Questions this frame must answer
1. **The inventory.** List every recipe that exists today:
   - `tests/fixtures/recipe_corpus/`;
   - the three bundled recipes (`recipes/`, and what `bundled::resolve` hands out);
   - the examples in `docs/recipes.md`, `docs/cli-reference.md` and `README.md`;
   - the build fixtures;
   - the recipes the ddphotos page emits (`integrations/ddphotos/edit/edit.js`, `pixelRecipe`
     and `cliRecipe`);
   - the agent skill's examples, if any.

   For each, record whether it's already run (not just parsed) by a test, which test, and whether
   its output is pinned. Cite `file:line`. Every recipe that's neither run nor pinned is a gap.
2. **What to pin.** Choose between the **encoded output bytes** and a **hash of the decoded
   pixels plus dimensions, colour type and bit depth**. The success signal says bytes. The CI
   matrix is Linux, macOS and Windows, x86 and ARM. Find out whether each encoder in the corpus
   (PNG, JPEG, WebP, AVIF via rav1e) gives the same bytes on every leg; the rav1e question
   matters most, because its assembly paths differ by CPU. Recommend one rule per format, with
   evidence from a local run plus what the existing CI-run tests already show. A pin that is red
   on one leg is useless, and a pin loosened until it's always green is worse.
3. **Where it lives.** Choose between the Rust corpus test (it runs in `cargo test` on every CI
   leg already), `cli-golden.py` (it exercises the real binary, but isn't wired into CI), or both.
   Recommend one. The guard has to run on every PR in this project without anyone remembering to
   run it.
4. **Fixtures.** Inputs must be independent of the code under test: generated in the test, not
   by crustyimg. They also need to make each step do something: a `max` resize needs an input
   larger than its bound, and a watermark needs room to fit. Say which fixture sizes and colour
   types (8-bit RGB, RGBA, 16-bit) the guard needs.
5. **It must be able to fail.** Name the negative control design must require. For example,
   change the resize filter or a watermark default on a scratch branch and show the guard turns
   red, naming the recipe. Also add a positive control: a dimension or pixel assertion, so a
   guard that runs no steps can't pass. Drive one mutation yourself at frame if it's cheap.
6. **Blessing.** When a later spec intends to change an output, how is the pin updated? Is there
   a `CRUSTYIMG_BLESS`-style switch, and does the diff show which recipe changed? Recommend the
   rule the PROJ-017 specs follow: any re-bless in a PROJ-017 PR is a verify finding unless the
   spec's acceptance criteria name it.
7. **Size.** Target **[S]**. If the inventory turns up many unrun recipes, recommend the cut:
   what the guard covers now, and what gets a backlog entry.

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
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, success signals and build
   order.
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item in full, and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for". PROJ-017 has no HANDOFF of its own yet, so these rules bind here.
5. DEC-058 (the cache key) and the lockfile DEC (find it with `./scripts/decisions-audit.sh` or by
   grepping `decisions/`): what "output bytes changed" breaks for a user.
6. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code, not the backlog text.** Run the existing corpus test and
  `cli-golden.py` on a release build of `main` (`cargo build --release`, in a fresh
  `CARGO_TARGET_DIR` that you delete at the end). Record the exact commands and output.
- **Size the verification, not just the code** (AGENTS §2). Count the recipe × fixture × format
  matrix.
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Allocate the id
`just new-spec` allocates by scanning the working tree, so it will hand out SPEC-139. Run
`just new-spec "<title>" STAGE-058 PROJ-017`, then `git mv` the spec and its timeline to SPEC-141
and fix the id inside both. Then run `just advance-cycle SPEC-141 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (decisions and related specs, including SPEC-097 and the spec behind the corpus test), and
  `cycle: frame`.
- `## Context` and `## Goal`: what is and isn't pinned today, and what a PROJ-017 builder sees
  when this ships (the command, and the red output naming a recipe).
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the inventory table;
  - the pin rule per format;
  - sizing and why;
  - scope in/out;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-141-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-141 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change **only your item's** `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-141** (framed <date>)`. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of the stage it belongs to, and update that stage's
   `**Count:**` line with a grep you've just run. Something with no stage goes in
   `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-141): frame the PROJ-017 no-bytes-changed guard`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-058 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. A mutation you drive at frame lives in a
  scratch branch or a scratch copy and is never committed.
- `cargo test` is needed only for the premise check.
- **Concurrent builds.** If the parity-harness frame (SPEC-142) is running beside you, never
  share a `CARGO_TARGET_DIR` with it.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`grep -l "SPEC-141 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and
cache_read is ×0.10. Before your final commit, append a `frame` entry to the spec's
`cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate anchors.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-141 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- the inventory: run or not, and pinned or not, per recipe source;
- the pin rule per format, and where the guard lives;
- the negative control, and whether you drove it;
- size and why; scope in/out;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them.

Then your measured cost.
