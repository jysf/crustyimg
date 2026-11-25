# SPEC-157 — FRAME prompt: `build` destinations have one identity

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-157-frame -- --model opus
```

**Reserved: SPEC-157** (STAGE-063, reserved 2026-11-10 with SPEC-154 to SPEC-158). Before you
allocate, confirm SPEC-157 is unused as a spec id:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-157' "$b" -- . ; done
```

Expected hits: this prompt, STAGE-063, the SPEC-154 frame prompt and
`docs/reviews/2026-11-10-external-code-review.md`. If a spec file or another reservation uses
SPEC-157, stop and report. Don't pick another number.

Your item is the STAGE-063 entry "**`build` destinations have one identity: no overwritten
sources, no aliased collisions (review P1 x2).**" Read it in full, and SPEC-158's entry next to it:
it shares a root cause (one resolved identity for paths and bytes), and the review's staff-engineer
note proposes one flow for both.

## Why this spec exists
Two P1 findings from the review, one cause:
- A target with `source = "src/photo.png"` and `out = "src"` replaced the source and exited 0. A
  `build --check` straight after exited 7.
- With an existing `alias/` directory, targets writing `dist/result.png` and
  `alias/../dist/result.png` exited 0. One file existed, and the lockfile recorded two paths and two
  hashes.

`output_collision_key` (`src/cli/build.rs` ~376) is deliberately **not** canonicalized ("it need not
exist yet"), so `check_output_injective` (~393) compares path spellings. `lock_output_path` (~473)
is its post-decode twin. Nothing compares a destination with the declared sources.

## Questions this frame must answer
1. **Premise, driven.** Reproduce both findings with a manifest in your scratchpad. Quote the
   manifest, the exit codes, the files left and the lockfile. Then try the neighbours: two targets
   whose `out` differ only by `./` or a trailing slash, by case on a case-insensitive APFS volume,
   and an `out` reached through a symlinked directory.
2. **One identity, defined.** Recommend how a destination is resolved before anything is written,
   when the file and maybe its directory don't exist yet. For example: canonicalize the longest
   existing ancestor, then join the rest lexically after normalizing `.` and `..`. Say what it does
   with `..` past a symlink, on case-insensitive filesystems, and on Windows (`\\?\` prefixes,
   drive-letter case). Say which code uses it: planning, the injectivity check, the sink, and the
   lockfile.
3. **Source overlap.** Define "a destination overlaps a declared source": the same file, or a file
   any target reads, including watermark assets and recipe files (`recipe_asset_paths`). Across
   targets, not only within one. And `--watch`: say whether an output inside a watched root makes a
   rebuild loop today, and whether this rule closes it.
4. **When it's refused, and with what.** Before any write, at planning. Give the exit code (read
   DEC-106 and the `build` exit codes in `docs/cli-reference.md`) and the message, naming both
   targets or the target and the source.
5. **Symlinked output directories.** DEC-061 accepts them as a compatibility tradeoff, and the
   reviewer wrote outside the tree through one. Read DEC-061 and DEC-035. Lay out the options (keep
   accepting, refuse, accept only when the link stays inside the project root) with what breaks for
   each, and recommend one. This is a maintainer question. The entry says this spec needs a design
   cycle and a DEC: give its content. Don't name a number.
6. **`build --check` and the lockfile.** What does `--check` compare, and does it write anything?
   (The review's principal engineer asks for that to be documented.) After the fix, what does the
   lockfile hold for one destination?
7. **Split or not.** Overlap refusal and the alias collision could be two specs, and SPEC-158 (the
   lockfile size contract and the hash-one-read-encode-another race) touches the same flow.
   Recommend the boundary. Fold nothing from SPEC-158 unless it keeps this [M].
8. **Tests.** Which cases are integration tests on a temp tree, which need Windows (CI only), and
   which need a case-insensitive volume. For each guarantee, a mutation row with its exact edit, and
   say how the red set will be read: a full `--no-fail-fast` run of every target (AGENTS §15).
9. **Docs.** Where `build`'s `out`, collisions and symlinks are described (`docs/build.md` or
   wherever it lives; grep), and what each says after.
10. **Fold-ins.** Read every open STAGE-063 entry and recommend fold or leave.
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
Run `just new-spec "<title>" STAGE-063 PROJ-011`. If it gives anything other than SPEC-157, `git mv`
the spec and its timeline to SPEC-157 and fix the id inside both. Then run
`just advance-cycle SPEC-157 frame`.

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
1. On **go**, run `just advance-cycle SPEC-157 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-063's backlog, change your item's marker from `- [ ] **SPEC-157** (reserved 2026-11-10)`
   to `- [ ] **SPEC-157** (framed <date>)`. Fold another entry only if you recommend it **and** it
   doesn't change the size; otherwise put it to the maintainer. On a split, file the second half as a
   new `- [ ] (not yet written)` entry right after yours. File an out-of-spec finding as a new
   `- [ ] (not yet written)` entry in the stage it belongs to. Update the `**Count:**` line with a
   grep you've just run, and show it. Something with no stage goes in `guidance/ideas.yaml`.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message: `docs(SPEC-157): frame one identity for build destinations`.
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
- **Budget ~40 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it by content, not
recency: `/usr/bin/grep -l "SPEC-157 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

Dedup by `.message.id`: lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take the
three once per id and the **MAX** `output_tokens`. Price each component at the model that ran
(`.message.model`): Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 the input rate,
cache_read ×0.10. Measure at your last turn, after the push, and append a `frame` entry to the
spec's `cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate
anchors, in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-157 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check: the reproduction, quoted, with its exit code;
- one line per question above, with the evidence;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
