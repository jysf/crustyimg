# 0.8.2 — RELEASE CUT prompt: bump, roll the CHANGELOG, gate, release PR

Cycle: **release prep** (one part). Fresh CLI session, own worktree, **Sonnet**. The work is
mechanical, with a strict checklist.

```
just wt chore/release-0-8-2 -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Where things stand
- **0.8.2 is everything on `main` since `v0.8.1`, and nothing else.** One spec is in it, and it
  has shipped: SPEC-143 (PR #235, `06eb552`), "a watermark that draws nothing is refused".
- **The version is 0.8.2 (maintainer, 2026-10-31).** `[Unreleased]` has one `### Breaking` entry
  (`watermark` refuses a mark that would draw nothing). As with 0.8.1, the maintainer treats a fix
  that refuses what used to be silently wrong as a patch. Do not reopen this, and do not edit
  `RELEASING.md`'s policy.
- **No part 1 this time.** SPEC-143's build wrote the `[Unreleased]` entry and the doc and skill
  changes, and its verify checked them on the binary. The skill already says "Written against
  crustyimg **0.8.x**".
- **You do RELEASING.md steps 1–5, as a PR.** The release commit changes `Cargo.toml` and
  `Cargo.lock`, so it goes through CI like code.
- ⛔ **Steps 6–8 (create the tag, push it, verify the channels) are maintainer-only.** No session
  creates, pushes or deletes a tag. You also do not publish to crates.io, npm or Homebrew. The
  orchestrator writes the "ready to tag" note after your PR merges, because the tag must point at
  the merge commit on `main`, which you cannot know.
- **Other PRs are open** (#233, #234 add tests only; #226, #229 are docs). They are not part of
  0.8.2. If one merges while you work, rebase over it; it needs no CHANGELOG entry unless it
  touches `src/`. If one does touch `src/`, stop and report.

## Step 0
1. `git fetch origin && git rebase origin/main`. `main` must contain `06eb552`.
2. Confirm the starting state:
   - `Cargo.toml` says `version = "0.8.1"`;
   - `git tag -l v0.8.2` is empty locally, and `git ls-remote --tags origin v0.8.2` is empty on
     the remote;
   - `CHANGELOG.md` has a non-empty `## [Unreleased]` and no `## [0.8.2]`.

   If any of these fails, stop and report.

## Read first
- `RELEASING.md` in full. Its "Release-cut checklist" is the procedure.
- `scripts/release.sh`, which is what `just release` runs.
- `CHANGELOG.md`: `## [Unreleased]`, the `## [0.8.1]` section as the shape to match, and the
  link-reference block at the bottom (about line 800).
- `.github/workflows/`: which workflows a `v*` tag fires. List them for the report, so the
  maintainer knows exactly what the tag will publish.

## Do, in order (RELEASING.md steps 1–5)
1. **Step 2a check, before the roll.** Diff `[Unreleased]` against the code commits since the
   last tag.
   - Run `/usr/bin/git log --oneline v0.8.1..HEAD --no-merges -- src/ Cargo.toml` (the raw git
     binary: a wrapped `git log` can truncate) and map every commit to an `[Unreleased]` entry.
   - The orchestrator found one: `06eb552` (#235, SPEC-143's Breaking entry). Anything else
     unmapped: **stop and report**; do not write the entry yourself.
2. **Run `just release 0.8.2`.** Read its output. It must print
   `✓ Cargo.toml + Cargo.lock == 0.8.2`. Before you roll the CHANGELOG, it is expected to warn that
   there is no `## [0.8.2]` section yet.
   - Check `git diff --stat`: only `Cargo.toml` (one line) and `Cargo.lock` (the crustyimg entry
     only) may change.
   - If `Cargo.lock` changes any other package's version, stop and report. A release must not
     update dependencies.
3. **Roll the CHANGELOG (step 2).**
   - Move the whole `[Unreleased]` body under `## [0.8.2] - YYYY-MM-DD`, using today's local date
     (`date +%F`).
   - Leave an empty `## [Unreleased]` above it.
   - Keep every entry byte-for-byte, and keep the heading as it is (`### Breaking`).
   - Update the link references:
     ```
     [Unreleased]: https://github.com/jysf/crustyimg/compare/v0.8.2...HEAD
     [0.8.2]: https://github.com/jysf/crustyimg/compare/v0.8.1...v0.8.2
     ```
     Add the `[0.8.2]` line above `[0.8.1]`, in the same `compare` form.
   - **Prove nothing was lost.** Take the old `[Unreleased]` body and the new `[0.8.2]` body and
     `diff` them. The output must be empty. Paste the command and its exit code.
   - Re-run `just release 0.8.2`. It must now print `✓ CHANGELOG has a '## [0.8.2]' section`.
4. **Version strings elsewhere.** Grep the repo for `0\.8\.1` and `v0\.8\.1` outside history
   documents (`feedback/`, `docs/research/`, `projects/`, `decisions/`, `CHANGELOG.md`'s released
   sections). The orchestrator found:
   - `skills/crustyimg/SKILL.md:12` and `docs/recipes.md:5` say `0.8.x` / `v0.8.x`. Leave them;
     both are still true.
   - The pre-commit `rev: v0.8.1` pins in `README.md:417`, `.pre-commit-hooks.yaml:12` and
     `docs/github-actions.md:214`. **Leave them.** They are read on `main` before the tag exists,
     so pinning `v0.8.2` now would point users at a missing tag. The orchestrator bumps them after
     the release.

   Report any other hit, and change it only if it names the *current* release as a fact a user
   acts on. Say why for each.
5. **Dry-run publish (step 3):** `cargo publish --dry-run`. Fix nothing in the manifest yourself.
   Report any warning, and stop if it fails.
6. **Gate suite (step 4).** Run it on a clean tree, sequentially, each leg in a fresh
   `CARGO_TARGET_DIR`:
   - default: `cargo test` (stdout to a file, `--no-fail-fast`), then `cargo clippy -- -D warnings`
     and `cargo fmt --check`;
   - `cargo build --no-default-features`, and `cargo test --no-default-features`;
   - `cargo test --features webp-lossy`;
   - `cargo deny check`.

   Read `$?` for each. Run the built binary's `--version`: it must say `0.8.2`. Delete each target
   dir afterwards.
7. **Commit (step 5).** Stage only `Cargo.toml`, `Cargo.lock` and `CHANGELOG.md`, plus any file
   step 4 justified. Run `git commit -s -m "chore(release): v0.8.2"`, ending the message with
   `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. **The DCO sign-off is
   mandatory.**
8. **Open the PR.**
   - Push the branch: `git push origin HEAD:chore/release-0-8-2`.
   - Open it as a **draft**: `gh pr create --draft`.
   - Title: `chore(release): v0.8.2`.
   - Body:
     - the version check output;
     - the CHANGELOG-roll diff proof;
     - the gate results;
     - the list of what a `v0.8.2` tag will fire (from `.github/workflows/`);
     - the Claude Code footer.
   - ⛔ Never mark it ready, never merge it.
   - Run `gh pr checks <PR> --watch` **in the background**. This PR touches `Cargo.toml`, so the
     full matrix must run, not the docs-only path. When it exits, read one snapshot at the head SHA
     and check every leg, including windows-latest, lean, msrv, avif, heic and webp-lossy. Never
     poll.

## Guardrails
- ⛔ **No tag**: no `git tag`, no `git push --tags`, no `git push origin v0.8.2`. No publish
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
  `grep -l "RELEASE CUT prompt: bump, roll" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`. **Do not skip this.**

## Report back (your final message — it is pasted to the orchestrator)
First line: `0.8.2 cut: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short, **with the evidence inline**. Do not refer to an earlier message; the orchestrator
only sees this one.
- the step-2a commit map: any unmapped commit;
- the `just release` output lines (both runs);
- `git diff --stat` for the release commit;
- the CHANGELOG-roll proof: the diff command and exit code, and the new link references;
- the version-string grep and each decision;
- the `cargo publish --dry-run` result and any warnings;
- each gate leg's exit code, and the binary's `--version`;
- the CI legs at the head SHA;
- what a `v0.8.2` tag will fire, workflow by workflow;
- your cost.
