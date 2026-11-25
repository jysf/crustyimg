# SPEC-155 — DESIGN prompt: a file write is all or nothing

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-155-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`88089d4`, cost in `73f40f5`, rulings
recorded after). **A new DEC is expected** ("a file write is all or nothing"; its content is in the
frame's Q8). You write its final text into Implementation Context, for the build to create. **Don't
name its number.** Other STAGE-063 designs may run alongside yours and claim ids. Write
`DEC-<next>` and tell the build to take the next free id at build time.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise, driven: silent exit 0 only under the 8 KiB buffer; above it, a loud failure that has
  already destroyed the old file, on every sink arm, under `ulimit -f`, a full disk and
  `--save-recipe`;
- the sweep of every writer (four sink arms, `--save-recipe`, `write_lock`, the cache);
- the helper's shape: temp in the destination's directory, `create_new`, mode rules, one RAII
  remover, guards first, no `BufWriter`, no `fsync`, hand-rolled on `std`;
- Windows' rename rules and the three Windows rows;
- the `StageFs` seam and `FailingFs`;
- six mutation rows; the DEC content; the docs lines; F1 and F2.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer accepted all four of the frame's recommendations on 2026-11-22 (recorded in the stub
under `### Maintainer rulings`):
1. **Read-only directory, writable file, `-y`: refused** with exit 5 and a message naming the
   directory. No in-place fallback.
2. **No `fsync`**, file or directory. The DEC says power loss isn't covered.
3. **F1 in scope:** `--save-recipe` gains the overwrite guard (existing recipe without `-y` → exit 5,
   old bytes kept).
4. **Full disk:** the loud failure is accepted.

Also settled: F2 in scope (`write_lock` moves onto the helper); hand-rolled on `std`, no new
dependency; a replaced file keeps its permission bits but not its owner, ACLs, xattrs (Finder tags on
macOS) or hard links, and the docs say so; stdout and the display sink are out; the build cache is
out.

## Order and neighbours
SPEC-154 lands first; this spec's **build** starts after it merges. Designs for SPEC-156 and
SPEC-157 may run alongside yours. **SPEC-157 also touches `src/cli/build.rs`** (destination identity
and the lockfile's paths), and SPEC-158 will too. At Step 0, run
`gh pr list --state open --json number,title,files` and `git branch -r`, and list every file another
open PR or pushed `feat/spec-15[4-8]*` branch touches that this spec also touches. Write that map
into Implementation Context **from their file lists, not their specs' text**, and say which
functions in `build.rs` each side changes, so the two builds rebase cleanly. On a rebase conflict in
STAGE-063's file, keep both sides and recount the `**Count:**` line with a fresh grep. On a conflict
in any other file, stop and report. Never edit `projects/PROJ-017-geometry-as-recipe-steps/`.

## Things in the frame to check before you build on them
Drive these on a release build of `main` and on a prototype (never pushed):
- **Parallel writes.** `build` (and any batch path) may write many outputs from several threads in
  one process. Find out whether it does (rayon, `par_iter`, threads), and make the temp name's `<n>`
  unique per process across threads. Drive a build with many targets into one directory on the
  prototype and show no collision, no `AlreadyExists` failure, and no temp left.
- **Dot-named temps seen by crustyimg itself.** The frame asks whether `source::resolve_directory`
  skips dotfiles. Check it, every glob input path, and **`build --watch`**. Does a temp appearing in
  a watched directory trigger a rebuild, or a loop? Drive `--watch` on the prototype with an output
  inside a watched root.
- **Writing over the input.** `-y` over the input itself, and `meta` writing in place. Is the input
  fully read and its handle closed before the rename? On Windows a rename over a file still open
  without `FILE_SHARE_DELETE` fails, so a handle the process itself holds would turn every in-place
  write into exit 5 on Windows. Read each path and cite `file:line`; if a handle can be open, say how
  the build closes it first, and add a Windows row.
- **The read-only-directory refusal.** Which call fails (`create_new` with `PermissionDenied`), how
  that error reaches exit 5 today, and the message, verbatim, naming the directory. Drive it on the
  prototype, and as root it can't be driven, so say what CI runs as.
- **`--save-recipe`'s surfaces.** List every verb that takes `--save-recipe` (grep `src/cli/`) and
  check that each also takes `-y`. If one doesn't, that's a design question to put in your report.
- **The mode rules.** Drive a new file under umask 022 and 077, and `-y` over a 0600 and a 0640 file.
  Confirm `std`'s `OpenOptions` default is 0666 masked by umask on the CI toolchain, and that
  copying permissions compiles to a no-op or the right thing on Windows.
- **wasm32.** `src/sink` compiles for wasm. Show the helper compiles there (`cargo check --target
  wasm32-unknown-unknown` with the features the wasm build uses), and gate any Unix-only call
  (`OpenOptionsExt`, `PermissionsExt`) with `cfg`.
- **Windows' rename on the CI toolchain.** The frame says newer std tries a POSIX-semantics rename
  first. Read the toolchain CI pins (`rust-toolchain.toml` or the workflow) and its std source for
  `fs::rename` on Windows, and say what the three Windows rows (replace, held-open destination,
  read-only destination) will see.
- **The 8 KiB line.** Re-drive the premise's small/big split with the real `BufWriter` capacity
  (quote it). The tests use outputs on both sides of it.

## Design calls that are yours
- **The code shape.** The helper, `StageFs`, `RealFs`, the RAII remover, the four sink arms,
  `--save-recipe` and `write_lock`, each with its file and the `file:line` it changes. Show
  `FailingFs` writes chunk by chunk so "the second write fails" leaves partial bytes in the temp.
- **Every message, verbatim,** with its exit code: the write-failed line (does today's
  `could not write output` stay, and does it name the file?), the read-only-directory line, the
  `--save-recipe` overwrite refusal (reuse the image-output wording; cite it). One AC row and one test
  per message. No message names a spec, stage or DEC id.
- **The tests.** Unit tests per injection point (create, Nth write, flush, set-permissions, rename),
  each asserting `Err`, old bytes unchanged (or no file), and no `.crustyimg-` temp. End-to-end rows on
  Linux and macOS: `ulimit -f` with XFSZ ignored and at default, both sizes, `Sink::File` and
  `Sink::Dir`, image lane and byte lane; read-only directory (skipped as root); mode kept on `-y`;
  `--save-recipe` with and without `-y`; parallel `build`. Windows rows: replace, held-open
  destination, read-only destination. Say which run where; Windows runs only on CI.
- **What doesn't change.** Every output's bytes (a write path change must not touch encoders). Say
  what pins it today, and add a pin only where none exists.
- **The DEC's final text,** from the frame's Q8 plus the rulings. Include a "see also" to DEC-035
  and DEC-059, and say whether DEC-059's lockfile line needs a pointer to the new DEC (as ship-time
  text, not an edit now).
- **Docs.** `SECURITY.md:41-44` and `:87`, `docs/cli-reference.md:55`, and where the `-y` note goes
  (the frame leaves that to you). Re-check each `file:line`, find any surface the frame missed
  (`README.md`, `skills/crustyimg/SKILL.md`, `--help` long text), and write each sentence verbatim.
  List **every behaviour sentence and every exit path it names**; the build drives each sentence on
  each path, and verify drives them again.
- **CHANGELOG.** The "Fixed" line (all-or-nothing writes) and the two "Changed" lines (read-only
  directory refused; `--save-recipe` needs `-y` to replace), verbatim.
- **Size.** The frame says [M]. Say whether that holds, with the count's derivation next to it.

## Tests: each one must be able to fail
HANDOFF's traps and AGENTS §15 bind here. **Drive every mutation row at design on the prototype;
don't reason them.** Each row **quotes its exact mutant edit**, names the code location, every test
expected red and every test expected green, and is marked *driven* or *reasoned*. Read the red set
**off the prototype's full `cargo test --no-fail-fast` run of every target**, not off the reasoning.
A mutant that doesn't compile stops `cargo test`, so run that row's targets separately and say so.
At least the frame's M1 to M6, plus:
- the temp name made non-unique across threads (a fixed `<n>`): the parallel `build` row must red;
- the remover disarmed before the rename instead of after: which row reds?
- the read-only-directory refusal replaced by an in-place fallback: the refusal row must red;
- **Trap 14:** restore `main`'s `src/sink/mod.rs` verbatim and list everything that goes red.

Windows rows can't be driven locally. Mark them *reasoned*, say what CI will show, and tell the
build to read the Windows leg's red set on its PR.

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
   your entry, SPEC-154 to SPEC-158.
4. `docs/reviews/2026-11-10-external-code-review.md`: the write finding.
5. `/guidance/constraints.yaml`, DEC-035, DEC-044, DEC-059, DEC-061.
6. A shipped PROJ-011 spec in `specs/done/` (SPEC-150 is a recent one) as the shape to match: ACs,
   exact messages, mutation tables, Build Completion.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `src/sink/mod.rs` (`write`,
  `write_bytes`, `reject_symlink_destination`, `guard_overwrite`), `--save-recipe` in
  `src/cli/ops.rs`, `write_lock` in `src/cli/build.rs`, the build's output loop, `meta`'s in-place
  path, `source::resolve_directory`, and the `--watch` loop.
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-155 build`.
2. In STAGE-063's backlog, change SPEC-155's marker from `(framed 2026-11-22)` to
   `(designed <date>)`, keeping `[M]` (or the size you found). Mark design `[x]` in the timeline with
   a 2–4 line summary, and add `prompts/SPEC-155-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-155): design all-or-nothing writes`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. On a reject,
   rebase and push again; never force.
5. Read back `git show --stat HEAD` and the STAGE-063 diff.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC file (its text goes in
  the spec). No pushed probe branch. No version bump, no tag. Don't raise or plan a release.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does
  not word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS
  has no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- `ulimit -f` and a full-disk image go in a subshell or a mounted image in your scratchpad, never on
  the main volume. Unmount and delete the image before you finish.
- Never share a `CARGO_TARGET_DIR` with another session. Remove the prototype worktree and its target
  dir before you finish. Write scratch files in your session's scratchpad, never `/tmp` or the system
  temp dir.
- **Budget ~55 exchanges** (driven mutation rows and the parallel and `--watch` probes are the
  extra).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-155 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take those
three once per id and the **MAX** `output_tokens`. Include any `subagents/` transcripts.

Price each component at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read is ×0.10. Measure at your last turn, after the
push, and append a `design` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-155 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- any correction to the frame, with what you drove (parallel writes, dot-named temps and `--watch`,
  writing over the input and open handles, the read-only refusal, `--save-recipe`'s verbs, the mode
  rules, wasm32, Windows' rename, the 8 KiB line);
- the code shape, one line per piece with `file:line`;
- the AC list (one line each);
- the Failing Tests and the mutation table, each row with its quoted edit, red and green tests, and
  driven or reasoned;
- every exact message, with its exit code;
- what pins "no output bytes change";
- the DEC's final text, and any ship-time pointer for DEC-059;
- the docs surfaces with `file:line`, and the behaviour sentences with every exit path each names;
- the CHANGELOG lines, verbatim;
- the overlap map with other open PRs and branches, from their file lists, and the `build.rs`
  functions each side changes;
- size with its derivation, and anything the build prompt must carry (checkpoints, CI legs, the
  Windows red set);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
