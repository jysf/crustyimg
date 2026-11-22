# SPEC-155 — FRAME prompt: a write that fails must not report success or destroy the old file

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-155-frame -- --model opus
```

**Reserved: SPEC-155** (STAGE-063, reserved 2026-11-10 with SPEC-154 to SPEC-158). Before you
allocate, confirm SPEC-155 is unused as a spec id:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-155' "$b" -- . ; done
```

Expected hits: this prompt, STAGE-063, the SPEC-154 frame prompt and
`docs/reviews/2026-11-10-external-code-review.md`. If a spec file or another reservation uses
SPEC-155, stop and report. Don't pick another number.

Your item is the STAGE-063 entry "**A write that fails must not report success or destroy the old
file (review P1).**" Read it in full.

## Why this spec exists
`Sink::File` and `Sink::Dir` (`src/sink/mod.rs`, `write` ~480 and `write_bytes` ~632) run the
symlink and overwrite guards, open the destination with `truncate(true)`, wrap it in a `BufWriter`,
call `write_all` and return `Ok(())`. Nothing calls `flush()` or `sync_all()`, so an error the
`BufWriter` hits when it is dropped is lost. And because the old file is truncated before the first
byte is written, a failure part-way through destroys it. That includes `-y` over the input itself.
The review reports that with a one-byte file-size limit, `convert` exited 0 and left a one-byte PNG.

⚠ **The mechanism is a claim until you drive it.** `BufWriter::write_all` on a buffer larger than its
capacity writes straight through and returns the error. So the missing flush may lose only a
*small* output's error, and a file-size limit may kill the process with `SIGXFSZ` instead of
returning `EFBIG`. Find out which failures actually exit 0 today, and on which sizes.

## Questions this frame must answer
1. **Premise, driven.** Reproduce the review's case (`ulimit -f` in a subshell, with `SIGXFSZ`
   ignored and not ignored) on a small output and a large one, through `convert` (`Sink::File`) and a
   batch (`Sink::Dir`). Also drive a full disk or a read-only destination, whichever you can make in
   your scratchpad. Quote each exit code and the file left behind. Say which cases are silent
   success, which are a loud failure with a destroyed old file, and which are fine.
2. **Every writer, not two.** Sweep for every place crustyimg writes a user-visible file: both sink
   arms in both functions, `--save-recipe` (`src/cli/ops.rs`, `fs::write`), the build lockfile
   (`src/cli/build.rs` `write_lock`, which already uses a temp file; read it), the build cache
   (`src/build/cache.rs`), `meta` writes in place, `lint --fix` if it writes, and anything else the
   sweep finds. Paste the commands. For each one, say whether it has the same bug.
3. **The fix's shape.** The entry proposes: stage the bytes beside the destination, flush and
   `sync_all`, then rename over it. Evaluate it, including:
   - the temp file's name, and that it is created in the destination's directory (rename across
     filesystems isn't atomic);
   - its permissions (a new file's mode versus the replaced file's);
   - cleanup on every error path, and what a killed process leaves behind;
   - that the symlink and overwrite guards still run first (DEC-035), and that the rename can't be
     redirected through a symlink created in between;
   - whether directory `fsync` is needed for the rename to be durable, and whether it's worth it here.
   Is there a crate already in `Cargo.lock` that does this (`tempfile`'s `persist`)? Check its
   licence against `/guidance/constraints.yaml` and whether it's a dev-only dependency today.
4. **Windows.** Rename over an existing file behaves differently there (`MoveFileEx` with
   `REPLACE_EXISTING`; a destination open in another process fails). Say what `std::fs::rename`
   does on Windows and what the test must cover. `just check` doesn't run Windows; CI does.
5. **stdout and the display sink.** Writes to stdout can fail too (a closed pipe). Say whether this
   spec covers them or leaves them, and why.
6. **How to test a failing write.** The entry asks for an injected failing writer. Say where the
   seam goes so a test can make the second write, the flush or the rename fail, without making the
   sink generic for production. Say also which end-to-end case runs on CI (a read-only directory, a
   file-size limit) and on which OSes.
7. **Mutation rows.** For each guarantee (error propagates, old file survives, no temp file left),
   name the exact edit that removes it, and say how the red set will be read: a full
   `--no-fail-fast` run of every target (AGENTS §15).
8. **The DECs.** Read DEC-035 (symlink hardening), DEC-044 (output dir auto-create) and any DEC that
   describes what a write promises. Say whether this needs a new DEC ("a write is all or nothing")
   or an amendment, and give its content. Don't name a number.
9. **Fold-ins.** Read every open STAGE-063 entry and recommend fold or leave. Likely none.
10. **Docs.** Does any doc promise anything about writes (`SECURITY.md`, `docs/cli-reference.md`, the
    `-y` help)? Quote each and say what it says after.
11. **Size.** The backlog says [M]. Confirm or resize, and say why.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. You recommend its content. Separate design, build and verify sessions
follow. The maintainer pastes this prompt into a fresh session and pastes your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

### Running alongside other frames
Four frame sessions may run at the same time: SPEC-154, SPEC-155, SPEC-156 and SPEC-157, all in
STAGE-063. PROJ-017 is paused until these ship (its SPEC-161 frame prompt is written but on hold).
Frames write only docs, but all four edit the same stage file.
- On a rebase conflict in STAGE-063's file, keep both sides' marker changes and recount the
  `**Count:**` line with a fresh grep. On a conflict in any other file, stop and report.
- **Don't name a DEC number.** Parallel frames would each claim the same next id. Write "a new DEC"
  and its content. Design allocates the id.
- Never edit `projects/PROJ-017-geometry-as-recipe-steps/`.
- **SPEC-154 lands first** (its own prompt says why). That binds the builds, not the frames. Say in
  your stub that this spec's build starts after SPEC-154 has merged.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather than
a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §12, §13 git, §15 (including "During design",
   "A bound or 'can't happen' is a claim until its worst input is driven", "Classify every finding",
   and the mutation-row rule: the red set is read off a full `--no-fail-fast` run of every target)
   and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`:
   your entry, SPEC-154 to SPEC-158, every other open entry, and the success criteria.
3. `docs/reviews/2026-11-10-external-code-review.md`: your finding and its evidence. **Its
   reproductions were never re-driven in this repo.** Drive yours before you rely on any number in it.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for". If
   the file isn't in your checkout, say so.
5. The DECs named in your questions.
6. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code.** On a release build of `main` (fresh `CARGO_TARGET_DIR`,
  deleted at the end), reproduce the finding and quote the output and exit code. A cause the
  orchestrator confirmed by reading is not a symptom you drove.
- **Find the whole class, not the one instance.** Sweep with commands, and paste them. The scope of a
  grep is itself a claim.
- **Size the verification, not just the code** (AGENTS §2). Name the axes your questions raise, and
  the OS legs that CI runs but `just check` doesn't (Windows paths and renames, Linux).
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Allocate the id
Run `just new-spec "<title>" STAGE-063 PROJ-011`. If it gives anything other than SPEC-155, `git mv`
the spec and its timeline to SPEC-155 and fix the id inside both. Then run
`just advance-cycle SPEC-155 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`, and
  `cycle: frame`.
- `## Context` and `## Goal`: what goes wrong for a user today, and what is true when this ships.
- A `## Frame` section directly after `## Goal`. It holds the verdict (go / no-go / split), the
  premise check with evidence, the answer to each question above, sizing and why, scope in and out,
  the rough build file list, and open maintainer questions.
- `### Out of scope (for this spec specifically)`.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design, build,
  verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-155 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-063's backlog, change your item's marker from `- [ ] **SPEC-155** (reserved 2026-11-10)`
   to `- [ ] **SPEC-155** (framed <date>)`. Fold another entry only if you recommend it **and** it
   doesn't change the size; otherwise put it to the maintainer. On a split, file the second half as a
   new `- [ ] (not yet written)` entry right after yours. File an out-of-spec finding as a new
   `- [ ] (not yet written)` entry in the stage it belongs to. Update the `**Count:**` line with a
   grep you've just run, and show it. Something with no stage goes in `guidance/ideas.yaml`.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message: `docs(SPEC-155): frame all-or-nothing writes`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. On a reject,
   rebase and push again; never force.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-063 diff.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag, no CHANGELOG edit. A probe you drive at frame
  lives in a scratch directory and is never committed.
- ⛔ Don't raise or plan a release. STAGE-063's release line is not this spec's business.
- Never share a `CARGO_TARGET_DIR` with another session.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout to
  a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does
  not word-split unquoted parameters, and reads `$R:r…` as a modifier, so brace it (`${R}:`). macOS
  has no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and
  `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~`, `/tmp` or the system temp dir.
- **Budget ~35 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it by content, not
recency: `/usr/bin/grep -l "SPEC-155 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

Dedup by `.message.id`: lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take the
three once per id and the **MAX** `output_tokens`. Price each component at the model that ran
(`.message.model`): Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 the input rate,
cache_read ×0.10. Measure at your last turn, after the push, and append a `frame` entry to the
spec's `cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate
anchors, in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-155 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check: the reproduction, quoted, with its exit code;
- one line per question above, with the evidence;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
