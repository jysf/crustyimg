# SPEC-157 — DESIGN prompt: `build` destinations have one identity

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-157-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`5a7db84`, cost in `e40f38a`, rulings
recorded after). **A new DEC is expected** (a build destination's identity; content in the frame's
answer 5), plus an **amendment to DEC-061** ("Residual (symlink)" and accepted-risk 3). You write
the DEC's final text and the amendment into Implementation Context. **Don't name the DEC's number.**
SPEC-155's design also adds a DEC, and others may run alongside yours. Write `DEC-<next>` and tell
the build to take the next free id at build time. Don't edit DEC-061 now: `main` must stay true until
the code lands, so ship applies the amendment.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise, driven: P1-a (`out = "src"` replaced the source), P1-b (one file, two lock records,
  one hash wrong, `--check` "up to date"), and about ten neighbours that also exit 0;
- the resolver: drop `.`, canonicalize the longest existing prefix, lexically clean the tail, strip
  the canonical build root;
- the overlap list, the refusal point and messages, the symlink options, `--check`'s real behaviour;
- no split; SPEC-158 owns bytes, this spec owns paths;
- the tests and M1 to M10; the docs.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer accepted all five of the frame's recommendations on 2026-11-22 (recorded in the stub
under `### Maintainer rulings`):
1. **A symlinked `out` resolving outside the build root is refused.** In-tree links keep working,
   de-aliased. The message says why and that the workaround is to build in the tree and copy.
2. **Chaining is refused:** one target's output may not be another target's source.
3. **Case-folded comparison on every OS**, so `a.png` and `A.png` are refused on Linux too.
4. **Exit 2** for every new refusal.
5. **A CHANGELOG note** covers lockfiles that drift once on paths after the upgrade.

Also settled: the `--watch` exclusion is folded in; nothing folds from SPEC-158; making `--check`
read-only is its own STAGE-063 entry, not this spec (this spec only documents what `--check`
writes); `apply` and the single-file verbs are out.

## Order and neighbours
SPEC-154 lands first. **SPEC-155 builds before this spec**, because both change `src/cli/build.rs`
and maybe `src/sink/mod.rs`. SPEC-155 adds a staged-write helper (temp in the destination's
directory, then rename) that every sink arm calls, and moves `write_lock` onto it. Read SPEC-155's
spec as it stands on `main` (its design may land while you work; re-read it after each rebase).
Design this spec's sink change **on top of** that helper: say which function receives the resolved
path, and that the temp goes in the resolved directory. If SPEC-155's design isn't on `main` yet,
design against its frame and say which line to re-check once it is.

At Step 0, run `gh pr list --state open --json number,title,files` and `git branch -r`, and list
every file another open PR or pushed `feat/spec-15[4-8]*` branch touches that this spec also touches.
Write that map into Implementation Context **from their file lists, not their specs' text**, and name
the functions in `build.rs` each side changes. On a rebase conflict in STAGE-063's file, keep both
sides and recount the `**Count:**` line with a fresh grep. On a conflict in any other file, stop and
report. Never edit `projects/PROJ-017-geometry-as-recipe-steps/`.

## Things in the frame to check before you build on them
Drive these on a release build of `main` and on a prototype (never pushed):
- **What "case-folded" means.** ASCII lowercase, full Unicode case folding, or folding plus
  normalization? APFS is also normalization-insensitive (`é` precomposed and decomposed are one
  file). Drive `café.png` in both forms on the Mac and say what the rule does with them. Pick one rule,
  say what it over-refuses on Linux, and pin it with rows. Use no new dependency unless the case
  needs it; if it does, check the licence against `/guidance/constraints.yaml`.
- **The build root.** Which directory is the root (DEC-057), and what does `canonicalize` give when
  the root itself sits under a link (on macOS, `/tmp` → `/private/tmp`; your scratchpad is under
  it)? Drive a build from a linked path and show the identity is still root-relative.
- **The longest existing prefix.** Drive an `out` whose existing part ends in a link to a file, not a
  directory; an `out` that exists as a regular file; a dangling link in the middle of `out`; and a
  `..` in the non-existent tail after a link. Say what each does.
- **Between planning and the write.** The plan resolves once; the sink writes later. A directory or
  link can appear in between (another process, or this build's own `create_dir_all`). Say what the
  sink checks again (DEC-035's per-file check stays) and what a race can still do.
- **Overlap needs every input before any work.** Directory sources and globs: are every target's
  inputs known at planning, before the cache opens? Recipe assets: is a built-in font a file? Drive a
  target whose output equals a file another target finds only through a glob.
- **The lockfile path and the cache.** Is the output path part of any cache key or the lockfile's
  record key, so that a new spelling invalidates the cache or changes `--check`? Read
  `cache_key_for` and the lock schema. Say whether `CACHE_SCHEMA_VERSION` or the lock version must
  change, with the reason, and drive the "drifts once" upgrade from a lockfile written by `main`.
- **`--check` runs planning too.** Show the new refusals fire under `--check` with the same exit
  code and nothing written.
- **The open `--watch` question.** The frame saw a source edit inside an excluded `out` rebuild but
  report the output as cached. Drive it again: is an edit missed, or is the report wrong? If it's a
  real miss caused by the exclusion, it's in scope; if it's older, file it.
- **Windows.** `canonicalize` gives `\\?\C:\…`. Read what the CI toolchain's std does, and say what
  the Windows-only rows assert. These run only on CI; tell the build to read the Windows leg's red set
  on its PR.
- **The order of errors.** A manifest can break several rules at once (a collision and an overlap, or
  an overlap and a `..` escape). Say which message wins and pin one row for it.

## Design calls that are yours
- **The code shape.** The resolver (`file:line` where it lives), its five call sites (planning, the
  overlap pass, the sink, the lockfile path, the `--watch` exclusion), the new error variants in
  `src/cli/mod.rs` and their exit code. Name each with the `file:line` it changes.
- **Every message, verbatim,** with exit 2: collision (two targets), overlap with a source, with a
  recipe file, with a watermark image, with a font, with the manifest, with the lockfile, under
  `.crustyimg/`, chaining, and the out-of-tree link (with the build-and-copy hint). Start from the
  frame's drafts. One AC row and one test per message. No message names a spec, stage or DEC id.
- **The tests.** The frame's list, sorted into integration (every OS), `#[cfg(unix)]`, Windows-only
  and `tests/build_watch.rs`. Add a row that a refused build leaves every file byte-identical and writes
  no lockfile, and the upgrade row. Expected values from literals, never from the code under test.
- **What doesn't change.** Every manifest that builds today without an alias, overlap or out-of-tree
  link builds byte-identically, with the same lockfile. Say what pins it (the existing build tests,
  goldens), and add a pin only where none exists.
- **The DEC's final text and DEC-061's amendment,** verbatim.
- **Docs.** `docs/cli-reference.md` (§build: the `out` paragraph and the corrected `--check`
  paragraph), `SECURITY.md:42–46` and `:87`, and any surface the frame missed (`docs/USAGE.md`,
  `skills/crustyimg/SKILL.md`, `--help` long text, the manifest docs). Write each sentence verbatim.
  List **every behaviour sentence and every exit path it names**; the build drives each sentence, and
  verify drives them again.
- **CHANGELOG.** The "Fixed" line and the "Changed" lines (out-of-tree links refused; chaining
  refused; case-folded collisions; the one-time lock path drift), verbatim.
- **Size.** The frame says [M] and names a split trigger (the sink can't take a resolved path without
  changing `apply`). Check the trigger against SPEC-155's helper and say whether [M] holds, with the
  count's derivation next to it.

## Tests: each one must be able to fail
HANDOFF's traps and AGENTS §15 bind here. **Drive every mutation row at design on the prototype;
don't reason them.** Each row **quotes its exact mutant edit**, names the code location, every test
expected red and every test expected green, and is marked *driven* or *reasoned*. Read the red set
**off the prototype's full `cargo test --no-fail-fast` run of every target**, not off the reasoning.
At least the frame's M1 to M10, plus:
- the case fold made ASCII-only (or Unicode-only, whichever you didn't pick): which row reds?
- the root not canonicalized (cwd under a link): which row reds?
- chaining allowed (overlap skips other targets' outputs-as-sources): the chaining row must red;
- the out-of-tree check run after the sink creates directories: the "nothing written" row must red;
- **Trap 14:** restore `main`'s `src/cli/build.rs` verbatim and list everything that goes red.

Windows rows can't be driven locally. Mark them *reasoned*, and say what CI will show.

**Every count in the spec carries its derivation** (the command, or the sum), next to the number.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and Implementation
Context. You don't write the code or any DEC file. Separate build and verify sessions follow: one
builds the spec from your text alone, and another verifies it, so everything the builder needs must
be in the spec (AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so. Then build the overlap map above.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 and §16 habit 6 (no
   CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`:
   your entry, SPEC-154 to SPEC-158, the new `--check` entry and the `build_watch` diagnosis.
4. SPEC-155's spec (the staged-write helper) as it stands on `main`.
5. `docs/reviews/2026-11-10-external-code-review.md`: the two `build` findings.
6. `/guidance/constraints.yaml`, DEC-010, DEC-035, DEC-057, DEC-059, DEC-061, DEC-106.
7. A shipped PROJ-011 spec in `specs/done/` (SPEC-150 is a recent one) as the shape to match.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `output_collision_key`,
  `check_output_injective`, `lock_output_path`, `prepare_target`, `find_output_collision`,
  `recipe_asset_paths` and `resolve_recipe_assets`, `write_lock`, `watch_roots`, and the sink's
  `Sink::Dir` path.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context, and
keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the frame`
subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-157 build`.
2. In STAGE-063's backlog, change SPEC-157's marker from `(framed 2026-11-22)` to `(designed <date>)`,
   keeping `[M]` (or the size you found). On a split, file the second half as a new
   `- [ ] (not yet written)` entry right after yours. Mark design `[x]` in the timeline with a 2–4 line
   summary, and add `prompts/SPEC-157-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-157): design one identity for build destinations`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. On a reject,
   rebase and push again; never force.
5. Read back `git show --stat HEAD` and the STAGE-063 diff.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC file or DEC edit. No
  pushed probe branch. No version bump, no tag. Don't raise or plan a release.
- Symlink probes stay inside your scratchpad. An out-of-tree probe target is another scratchpad
  directory, never `~` or a system path.
- `--watch` probes run in the background with a fixed lifetime you end yourself (macOS has no
  `timeout(1)`; record the PID and kill it). Never leave a watcher running.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout to
  a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). `rtk` can
  truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you
  count.
- Never share a `CARGO_TARGET_DIR` with another session. Remove the prototype worktree and its target
  dir before you finish. Write scratch files in your session's scratchpad, never `/tmp` or the system
  temp dir.
- **Budget ~65 exchanges** (ten-plus driven mutation rows, the watch probe and the case-folding probe
  are the extra).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content, not
recency:**
`/usr/bin/grep -l "SPEC-157 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take those
three once per id and the **MAX** `output_tokens`. Include any `subagents/` transcripts.

Price each component at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read is ×0.10. Measure at your last turn, after the
push, and append a `design` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-157 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- any correction to the frame, with what you drove (the case-fold rule, the root under a link, the
  prefix edge cases, the race, inputs known at planning, the cache and lock keys, `--check`, the open
  `--watch` question, Windows, the order of errors);
- the code shape, one line per piece with `file:line`, and how it sits on SPEC-155's helper;
- the AC list (one line each);
- the Failing Tests and the mutation table, each row with its quoted edit, red and green tests, and
  driven or reasoned;
- every exact message, with its exit code;
- what pins "every other manifest builds the same";
- the DEC's final text and DEC-061's amendment;
- the docs surfaces with `file:line`, and the behaviour sentences with every exit path each names;
- the CHANGELOG lines, verbatim;
- the overlap map with other open PRs and branches, from their file lists, and the `build.rs`
  functions each side changes;
- size with its derivation, any split, and anything the build prompt must carry (checkpoints, CI
  legs, the Windows red set);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
