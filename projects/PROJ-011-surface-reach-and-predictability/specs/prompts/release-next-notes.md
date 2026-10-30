# Next release — RELEASE NOTES prompt: CHANGELOG, agent skill, docs

Cycle: **release prep, part 1 of 2** (docs only). Fresh CLI session, own worktree, **Opus**. The
sweep is small (five code-bearing PRs since `v0.8.0`), but every claim has to be driven on a
release binary, and one entry is missing today.

```
just wt docs/release-next-notes -- --model opus
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Where things stand
- The release is **everything on `main` since `v0.8.0`, and nothing else.** Both specs planned
  for it have shipped: SPEC-137 (`71b4f38`, PR #217) and SPEC-138 (`fc05ef9`, PR #221).
- ⚠ **The version number is not settled.** It was planned as 0.8.1, but `RELEASING.md`'s
  versioning policy says a patch bump has "no new commands, no changed flags, no changed exit
  codes", and `[Unreleased]` holds a `### Breaking` entry, a `### Changed` default-feature change
  and an `### Added` build recipe. The maintainer is ruling between 0.8.1 and 0.9.0. **Write
  nothing that names the next version number:**
  - no `0.8.1` or `0.9.0` in any file you change;
  - leave the skill's "Written against crustyimg **0.8.x**" line as it is.

  Step 4 below asks for the facts the ruling needs.
- **Part 2 is not yours.** A later session runs `just release X.Y.Z`, rolls `[Unreleased]`, and
  writes the release commit.
- ⛔ **Creating and pushing the tag is maintainer-only.** No session does it.

The code-bearing PRs since `v0.8.0`:

| PR | merge | what | `[Unreleased]` today |
|---|---|---|---|
| #216 | `2d225d0` | `optimize --ssim/--target` runs its search for AVIF on native builds | **missing** |
| #218 | `b265337` | ddphotos edit page, `svg` feature default-on, slim wasm build | Added + Changed |
| #217 | `71b4f38` | SVG text in any font family draws in the bundled font (SPEC-137) | Fixed |
| #224 | `268e8d7` | ddphotos edit page fixes (`integrations/` only) | none; check none is needed |
| #221 | `fc05ef9` | `build` refuses `-o`, `--out-dir`, `--name-template`, `--format` (SPEC-138) | Breaking |

## Step 0
`git fetch origin && git rebase origin/main`. Run
`git log --oneline v0.8.0..origin/main --no-merges -- src/ Cargo.toml tests/ integrations/ pkg/ wasm/`
and check the table against it. If a code-bearing commit is missing from the table, add it to
your work and report it.

## Read first
- `RELEASING.md`: the versioning policy, and steps 2 and 2a. These are the rules you carry out.
- `CHANGELOG.md`: `## [Unreleased]` in full, and `## [0.8.0]` for the house style. Entries are
  bold-lead and behaviour-first, and name the command, the exit code and what the user sees.
- For each PR: its description, its diff to `src/` and `tests/`, and the CHANGELOG lines it wrote
  (`git show <merge> -- CHANGELOG.md`). For SPEC-137 and SPEC-138, also the spec's Goal,
  Acceptance Criteria and `## Build Completion`, under
  `projects/PROJ-011-surface-reach-and-predictability/specs/done/`.
- `skills/crustyimg/SKILL.md` and `README.md`, in full.

## Do, in order

### 1 — CHANGELOG `[Unreleased]`
- **Add the #216 entry** under `### Fixed`. Find out from the diff and its tests:
  - what `optimize --ssim` / `--target` did for an AVIF output before (on which builds);
  - what it does now;
  - whether the browser build changed too (#216 touches `src/wasm.rs`).

  Then drive it: run the same `optimize` on an AVIF output with the `v0.8.0` release binary and
  with `main`'s, and report the exit code, output size and any message from each. The entry
  says only what the run shows.
- **SPEC-137's entry and the build cache.** A build cache written by a 0.8.0 binary holds blank
  SVG-text outputs. The version bump changes every cache key, so the next `build` re-renders
  them. Confirm that from the cache-key code (where the version enters the key) and say whether
  the entry should tell users "the next `build` re-renders once". Add a short clause only if it
  is true.
- **Check every existing entry against the binary.** For each of the four, drive the claim on a
  release build of `main` (fresh `CARGO_TARGET_DIR`, deleted at the end) and record the command
  and exit code:
  - Breaking: `build --format png` and each of the other three flags exit 2 and write nothing,
    with `--watch` and `--check` too;
  - Changed: an SVG input to a `--no-default-features` build exits 4 with "rebuild with
    --features svg";
  - Fixed: an SVG with `font-family="sans-serif"` draws text (count non-background pixels);
  - Added: `just wasm-build-slim` writes `pkg-slim/`.
- **Overclaim grep.** Grep `[Unreleased]` for `never`, `always`, `all`, `every`, `any`. Each one
  must be true on the binary, or be narrowed.
- Edit an existing entry only where it is wrong or misleading. Keep the heading order.
- ⛔ Do **not** roll `[Unreleased]`, change the link references or bump any version. That is
  part 2.

### 2 — `skills/crustyimg/SKILL.md` (RELEASING step 2a)
Walk `[Unreleased]`, and for each change make sure an agent using the skill would not be
surprised. At least:
- `build` refuses those four flags (exit 2). Any skill example passing one of them to `build` is
  now broken; fix it. Say how to set format and location in the manifest instead;
- SVG input needs the `svg` feature, which is on by default (relevant only to an agent building
  from source with `--no-default-features`);
- SVG `<text>` draws in the bundled Go font whatever its `font-family`, upright and regular;
- whatever #216 changed for `optimize --ssim/--target` with AVIF.

State what to do, not the history. Then run **every** command example in the skill on the release
binary, not only the ones you touched, and record each exit code. A skill example that fails is a
bug; fix it.

### 3 — `README.md` and `docs/`
Find sentences that these five PRs made false. Grep at least for:
- `build` shown with `--format`, `-o`, `--out-dir` or `--name-template`, in `README.md`,
  `docs/`, `recipes/` and `.github/` examples. The GitHub Actions guide added in #222/#223 shows
  `crustyimg build` for deploys; check its examples especially;
- SVG text, fonts or `font-family`;
- `--no-default-features` lists of what a lean build reads.

Fix each one. **The jysf.org project page and the two GitHub Actions repos are out of your
reach.** If a claim there is now false, list it for the maintainer.

### 4 — Facts for the version ruling (report only; change nothing for it)
For each `[Unreleased]` entry, say which RELEASING policy line it touches: new command, changed
flag, changed exit code, changed default, or bug fix only. Quote the policy text. For the
Breaking entry, also say what `build --format png` did on 0.8.0: exit 0, and what it wrote. Give
no recommendation; the maintainer decides.

## Land it
- Branch `docs/release-next-notes`. Commit with `git commit -s` and end each message with
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **The DCO sign-off is mandatory.**
  Use one commit per numbered section, or fewer.
- Open a **draft** PR: `gh pr create --draft`.
  - Title: `docs(release): release notes and agent skill for the next release`.
  - Body: the CHANGELOG coverage table, the list of examples run with their exit codes, and the
    Claude Code footer.
- ⛔ A draft stays a draft. Never mark it ready and never merge.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background. When it exits, read one
  snapshot at the head SHA and check every leg.

## Guardrails
- ⛔ No source or test changes. No version bump, no `Cargo.toml` or `Cargo.lock` change, no tag
  and no release. No version number for the next release in any file.
- No spec or DEC ids in user-facing text: the CHANGELOG, the skill and `README.md`.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no
  `timeout(1)`.
- A target dir shared across worktrees serves a stale library: use a fresh one per binary.
- Keep scratch files in a scratch directory, never in `~`. Delete scratch target dirs at the end.
- **Budget ~60 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Identify it **by content, not recency**:
  `grep -l "RELEASE NOTES prompt: CHANGELOG, agent skill, docs" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Opus $5/$25 per
  MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `Release notes: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short:
- the CHANGELOG coverage table (PR → entry, heading, added / edited / already there);
- the #216 run on both binaries, and the entry you wrote;
- the cache re-render finding;
- every existing entry's driven check, with exit codes;
- the overclaim grep and what you changed;
- the skill changes, and every skill example's exit code (count, plus any that failed);
- other false sentences you fixed, and any for the maintainer (jysf.org, the Actions repos);
- the step 4 table for the version ruling;
- your cost.
