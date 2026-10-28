# 0.8.0 — RELEASE CUT prompt: bump, roll the CHANGELOG, gate, release PR

Cycle: **release prep, part 2 of 2**. Fresh CLI session, own worktree, **Sonnet**. The work is
mechanical, with a strict checklist.

```
just wt chore/release-0-8-0 -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Where things stand
- **The maintainer ruled (2026-10-27): 0.8.0 is everything on `main` since `v0.7.1`, and nothing
  else.** Every spec in it is shipped.
- Part 1 is merged as `9927613` (PR #208). It finished `[Unreleased]` in `CHANGELOG.md`, the
  recipe guide, the agent skill (already "Written against crustyimg **0.8.x**") and
  `RELEASING.md`. A verify session checked part 1's claims on the binary.
- **You do RELEASING.md steps 1–5, as a PR.** The release commit changes `Cargo.toml` and
  `Cargo.lock`, so it goes through CI like code. 0.7.0 did the same (PR #146).
- ⛔ **Steps 6–8 (create the tag, push it, verify the channels) are maintainer-only.** No session
  creates, pushes or deletes a tag. You also do not publish to crates.io, npm or Homebrew. The
  orchestrator writes the "ready to tag" note after your PR merges, because the tag must point at
  the merge commit on `main`, which you cannot know.

## Step 0
1. `git fetch origin && git rebase origin/main`. `main` must contain `9927613`.
2. Confirm the starting state:
   - `Cargo.toml` says `version = "0.7.1"`;
   - `git tag -l v0.8.0` is empty locally, and `git ls-remote --tags origin v0.8.0` is empty on
     the remote;
   - `CHANGELOG.md` has a non-empty `## [Unreleased]` and no `## [0.8.0]`.

   If any of these fails, stop and report.

## Read first
- `RELEASING.md` in full. Its "Release-cut checklist" is the procedure.
- `scripts/release.sh`, which is what `just release` runs.
- `CHANGELOG.md`: `## [Unreleased]`, the `## [0.7.1]` section as the shape to match, and the
  link-reference block at the bottom (about line 744).
- `.github/workflows/`: which workflows a `v*` tag fires. List them for the report, so the
  maintainer knows exactly what the tag will publish.

## Do, in order (RELEASING.md steps 1–5)
1. **Step 2a check, before the roll.** RELEASING.md now says to diff `[Unreleased]` against the
   code commits since the last tag.
   - Run `git log --oneline v0.7.1..HEAD --no-merges -- src/ Cargo.toml` and map every commit to
     an `[Unreleased]` entry.
   - Part 1 found two commits with no entry: `545f931` and `9f1becd`, which change only
     `[profile.dev]` and need none. Anything else unmapped: **stop and report**; do not write the
     entry yourself.
2. **Run `just release 0.8.0`.** Read its output. It must print
   `✓ Cargo.toml + Cargo.lock == 0.8.0`. Before you roll the CHANGELOG, it is expected to warn that
   there is no `## [0.8.0]` section yet.
   - Check `git diff --stat`: only `Cargo.toml` (one line) and `Cargo.lock` (the crustyimg entry
     only) may change.
   - If `Cargo.lock` changes any other package's version, stop and report. A release must not
     update dependencies.
3. **Roll the CHANGELOG (step 2).**
   - Move the whole `[Unreleased]` body under `## [0.8.0] - YYYY-MM-DD`, using today's local date
     (`date +%F`).
   - Leave an empty `## [Unreleased]` above it.
   - Keep every entry byte-for-byte, and keep the heading order (Added, Breaking, Changed, Fixed).
   - Update the link references:
     ```
     [Unreleased]: https://github.com/jysf/crustyimg/compare/v0.8.0...HEAD
     [0.8.0]: https://github.com/jysf/crustyimg/compare/v0.7.1...v0.8.0
     ```
     That uses the same `compare` form as `[0.7.1]`.
   - **Prove nothing was lost.** Take the old `[Unreleased]` body and the new `[0.8.0]` body and
     `diff` them. The output must be empty. Paste the command and its exit code.
   - Re-run `just release 0.8.0`. It must now print `✓ CHANGELOG has a '## [0.8.0]' section`.
4. **Version strings elsewhere.** The orchestrator's grep found only `Cargo.toml` and
   `Cargo.lock`; the npm package is built from the crate. One more candidate is
   `docs/recipes.md:5`, "works in the shipped v0.7.x". Read the paragraph. If its `[today]` marks
   describe 0.8.0, change it to `v0.8.x`; otherwise leave it, and say why. Do not touch history
   documents (`feedback/`, `docs/research/`, `projects/`, `decisions/`).
5. **Dry-run publish (step 3):** `cargo publish --dry-run`. Fix nothing in the manifest yourself.
   Report any warning, and stop if it fails.
6. **Gate suite (step 4).** Run it on a clean tree, sequentially, each leg in a fresh
   `CARGO_TARGET_DIR`:
   - default: `cargo test` (stdout to a file, `--no-fail-fast`), then `cargo clippy -- -D warnings`
     and `cargo fmt --check`;
   - `cargo build --no-default-features`, and `cargo test --no-default-features`;
   - `cargo test --features webp-lossy`;
   - `cargo deny check`.

   Read `$?` for each. Run the built binary's `--version`: it must say `0.8.0`. Delete each target
   dir afterwards.
7. **Commit (step 5).** Stage only `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and
   `docs/recipes.md` if you changed it. Run
   `git commit -s -m "chore(release): v0.8.0"`. **The DCO sign-off is mandatory.**
8. **Open the PR.**
   - Push the branch: `git push origin HEAD:chore/release-0-8-0`.
   - Open it as a **draft**: `gh pr create --draft`.
   - Title: `chore(release): v0.8.0`.
   - Body:
     - the version check output;
     - the CHANGELOG-roll diff proof;
     - the gate results;
     - the list of what a `v0.8.0` tag will fire (from `.github/workflows/`);
     - the Claude Code footer.
   - ⛔ Never mark it ready, never merge it.
   - Run `gh pr checks <PR> --watch` **in the background**. This PR touches `Cargo.toml`, so the
     full matrix must run, not the docs-only path. When it exits, read one snapshot at the head SHA
     and check every leg, including windows-latest, lean, msrv, avif, heic and webp-lossy. Never
     poll.

## Guardrails
- ⛔ **No tag**: no `git tag`, no `git push --tags`, no `git push origin v0.8.0`. No publish
  (`cargo publish` without `--dry-run`, npm, Homebrew). No push to `main`. These are the
  maintainer's.
- ⛔ No source or test changes, and no dependency changes. The only lockfile change is the
  crustyimg version line.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh
  does not word-split unquoted parameters. macOS has no `timeout(1)`.
- Never share a target dir between differently-featured builds, and never run two legs at once.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~50 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "0.8.0 — RELEASE CUT prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`. **Do not skip this.** The previous
  release session did.

## Report back (your final message — it is pasted to the orchestrator)
First line: `0.8.0 cut: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short, **with the evidence inline**. Do not refer to an earlier message; the orchestrator
only sees this one.
- the step-2a commit map: any unmapped commit;
- the `just release` output lines (both runs);
- `git diff --stat` for the release commit;
- the CHANGELOG-roll proof: the diff command and exit code, and the new link references;
- the `docs/recipes.md` decision;
- the `cargo publish --dry-run` result and any warnings;
- each gate leg's exit code, and the binary's `--version`;
- the CI legs at the head SHA;
- what a `v0.8.0` tag will fire, workflow by workflow;
- your cost.
