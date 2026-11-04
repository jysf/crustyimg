# SPEC-144 — VERIFY prompt: a tag push publishes every channel, on runners that exist

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build SPEC-144.

```
just wt verify/spec-144 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-144-release-pipeline:refs/remotes/origin/feat/spec-144-release-pipeline
git reset --hard origin/feat/spec-144-release-pipeline
```

Draft PR #243's head was `17c8b8a` when this prompt was written, with CI green (19 pass, 12
skipped). Record the SHA you verify. The maintainer pasted this prompt into a fresh session and
will paste your final message back to the orchestrator, so it must stand on its own. **This gates
the next release:** GitHub retired `macos-14` on 2026-11-02, and no tag can release until this
merges.

## What the build claims
- The spec is
  `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-144-a-tag-push-publishes-every-channel-on-runners-that-exist.md`.
  Read it all: `## Frame` with rulings 1–5, `## Design` with Calls 1–9, the 18 ACs, `## Checks`
  (C1–C7, G1–G14, R1–R6, L1–L4, D1–D2, A1, V1–V3), the mutation table, and `## Build Completion`.
  Also read DEC-111 and the amendment notes on DEC-067, DEC-041 and DEC-040.
- **Commits:** `1839fce` (checks and guard), `df93971` (the pipeline), `2f53b5d` (a:
  `pr-run-mode = "upload"`, docs, DEC notes), `e88db1e` (a C3 addition from the mutation pass),
  `400092f` (b: upload removed), `17c8b8a` (Build Completion only).
- AC-1 to AC-17 met; AC-18 is the first real tag, after merge.
- C1–C7 and G1–G14 RED on `main` with their lines, GREEN at (b). R1–R6 as specified: R5 skips
  0.8.2 on both real registries, and R6 exits 3 with `COULD NOT ASK`.
- **Live run `41786007198` at (a):** aarch64 on `macos-15`, x86_64-apple on `macos-15-intel`, Linux
  on `ubuntu-22.04`, Windows on `windows-2022`. `host`, Homebrew, both custom publish jobs and
  `announce` skipped. `file` on the aarch64 binary says arm64.
- Mutations M1a–M19 all went red for their reason; M17–M19 are new.
- **Two design calls were wrong,** and the build changed them:
  1. **The npm dry run.** Setting `npm_config_dry_run=true` for the whole step also made the smoke
     test's `npm pack` a dry run, so there was no tarball. And npm 11 refuses even a dry run of a
     version already on the registry. The build put an `npm` wrapper in the PR job only, which
     turns just `npm publish` into a dry run aimed at `127.0.0.1:9`.
  2. **AC-15's "zero actionlint findings"** fails with shellcheck on: 6 findings inside dist's
     generated `release.yml`, also on `main`, fixable only by hand-editing a generated file.
- **Out-of-spec finding:** dist's `github-release = "announce"` would move the GitHub Release from
  `host` into `announce`, after the registries publish, without changing any `needs:`. C3 now
  asserts only `host` creates the Release, and M19 proves it.
- One CI leg (`heic`, ubuntu) failed once with "No space left on device" and passed on re-run.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. **No tag, no publish of any kind except
`--dry-run`, no `gh workflow run`** on any release or publish workflow, and no change to npm or
crates.io settings. Do not fix what you find; report it. Do not edit the spec or its timeline.

## Check, in order
1. **Scope and commit order.**
   - Compare `git diff --stat origin/main...HEAD` with the spec's `## Outputs` and Build
     Completion.
   - Every commit is signed off. No `src/`, `Cargo.toml`, `Cargo.lock` or version change.
   - `400092f` (b) removes exactly what `2f53b5d` (a) added for upload mode. `17c8b8a` touches only
     the spec.
   - `dist-workspace.toml` at the head has no `pr-run-mode`.
2. **`release.yml` is generated, not hand-edited.** With `dist` 0.32.0, run
   `dist generate --mode=ci` in a scratch copy and `git diff --exit-code` (V1), plus
   `dist generate --mode=ci --check` (C1). Then run `dist plan` and quote each target's runner.
3. **The checks, run yourself.** Run `scripts/check-release-pipeline.sh` and
   `scripts/release-publish-guard.sh --self-test` at the head (GREEN), and on a scratch worktree
   of `origin/main` with the two scripts copied in (RED). Read each RED line: is it red for the
   check's reason (trap 13), not a missing tool or a script bug?
4. **⚠ Are the checks run anywhere automatically?** `ci.yml` does not appear to call either
   script. Confirm by reading every workflow.
   - If nothing runs them, then C2 ("`pr-run-mode` is not left at `upload`"), C4/C7 (permissions),
     C6 (no tag trigger) and the self-test protect nothing after merge. They guard only if a human
     runs them.
   - The spec's design said "a script, kept in `scripts/` and run in CI or at verify". Judge
     whether a CI step (path-filtered on `dist-workspace.toml`, `.github/workflows/**`,
     `scripts/release-*`, `scripts/check-release-*`) is a punch item for this PR, and say why.
     Note that the Release workflow's own `plan` job catches drift in `release.yml` (C1's job), but
     not `upload` mode or permissions.
5. **The two publish workflows, read whole.** For `publish-npm.yml` and `publish-crates.yml`:
   - triggers: `workflow_call` and a path-filtered `pull_request` only; no `push`, no tag glob;
   - `publish` runs only on `event_name == 'push' && ref_type == 'tag'` and a validated `publish`
     verdict. **Inside a `workflow_call` from a tag-triggered `release.yml`, what are
     `github.event_name` and `github.ref_type`?** Confirm from GitHub's docs that the caller's
     context is passed through, so `publish` is not skipped on the real tag. Quote the doc line.
     A gate that can never be true on the real tag is the worst failure this spec can have.
   - permissions per job: npm `publish` alone holds `id-token: write`; crates `publish` holds
     `contents: read` only (ruling 5); `guard` and `dry-run` hold no `id-token` and name no secret;
   - `CARGO_REGISTRY_TOKEN` appears only in the crates `publish` step;
   - Node 24 via `setup-node`, and the `npm` ≥ 11.5.1 step.
6. **The npm dry-run wrapper (design call 1, changed by the build).** This is new code that no
   design reviewed.
   - It is defined only in the PR `dry-run` job. The tag-path `publish` job runs the real `npm`.
     Show where the wrapper lives and that no tag-path step can see it (`PATH`, a file on disk, an
     env var).
   - It turns **only** `npm publish` into a dry run, and passes `npm pack`, `npm install` and
     everything else through unchanged.
   - It aims at `127.0.0.1:9`, so even if the dry-run flag were lost, the publish has nowhere to
     go. Prove it: run the wrapper locally with the dry-run part removed, and show it fails to
     connect rather than publishing.
   - Read the PR's dry-run job log once (`gh run view <run> --log --job <job>`) and quote the
     `(dry-run)` line and the smoke test's pass.
7. **Drive the guard against the real registries.** At a `v0.8.2` worktree with the script copied
   in: `npm v0.8.2` and `crates v0.8.2` → `SKIP`, exit 0. With `RELEASE_GUARD_*_URL` pointed at a
   closed port → `COULD NOT ASK`, exit 3. Then check a `404`, a `500` and an unparsable `200` with
   a local fake server: each exits 3. Only a `200` without the version says `publish`.
8. **The live-run evidence, re-read.** Read run `41786007198`'s jobs once with
   `gh api repos/jysf/crustyimg/actions/runs/41786007198/jobs` and quote each target's runner
   label and conclusion, and the skipped publish jobs. Confirm the run was at (a)'s SHA. Read the
   current head's Release `plan` job: upload mode is off, so only `plan` runs.
9. **Re-apply the mutation table yourself, one row at a time** (M1a–M19), in a scratch worktree.
   Apply each, check `git diff` is non-empty, run C1–C7 and the self-test (and A1 where the row
   says), record every red check with its quoted line, then revert and confirm green. Compare with
   the build's record.
10. **Mutations the spec did not list.** Apply each and name the check that catches it, or report a
    gap:
    - **M-a:** the npm `publish` job's `if:` drops the verdict condition, so it runs on `skip`;
    - **M-b:** the guard treats `HTTP 200` with an **empty** versions list as `skip`;
    - **M-c:** the PR `dry-run` job gains `secrets: inherit`, or `NODE_AUTH_TOKEN: ${{ secrets.… }}`;
    - **M-d:** the wrapper is moved to a job-level `env`/`PATH` shared with `publish` (for example a
      workflow-level `env`);
    - **M-e:** `publish-crates.yml`'s `publish` job gains `id-token: write` (ruling 5; M17 claims
      this, so confirm it).
11. **AC-15 (actionlint).** Run `actionlint` 1.7.12 on the three files with and without
    shellcheck. Confirm the 6 findings are in generated `release.yml` and also on `main`, and that
    the two hand-written files are clean either way. Judge the build's relaxation: is the AC now
    worded so it can still fail?
12. **Docs and record**, against the workflows:
    - `RELEASING.md`: the one-time npm setting names **`release.yml`** (not `publish-npm.yml`);
      the publish order; re-running a failed publish job; when to turn on "disallow tokens"; the
      hand fallback as recovery only; the first-real-tag checklist. Each claim matches the files.
    - DEC-111 matches what was built, including ruling 5 and the wrapper. The notes on DEC-067,
      DEC-041 and DEC-040 keep `superseded_by: null`. `./scripts/decisions-audit.sh` reports no
      errors.
    - PROJ-011 `HANDOFF.md` and the comment-only edits: no behaviour change.
13. **CI.** Run `gh pr checks 243` once at the head you verified, and read the `heic` re-run: was
    the failure a runner disk issue unrelated to this diff?

## Guardrails
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. In zsh, `$B:t` is a
  modifier, so write `"${B}:path"`. macOS has no `timeout(1)`; give curl `--max-time`.
- A piped command reports the pipe's exit code: redirect output and read `$?`.
- Kill any fake registry server you start, and check with `lsof -i` before you finish.
- Delete scratch worktrees and downloaded artifacts when you are done. Keep scratch files in a
  scratch directory, never in `~`.
- **Budget ~70 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: search for `VERIFY prompt: a tag push publishes`. `grep -c` can come back
  empty through the `rtk` wrapper; count with Python or `/usr/bin/grep`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`), never at a flat
  rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and
  cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #243 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #243 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–13, one or two lines each with evidence;
- item 4: whether anything runs the checks automatically, and your ruling;
- item 5: the `event_name`/`ref_type` answer inside `workflow_call`, with the quoted doc line;
- item 6: the wrapper findings;
- the mutation table as you ran it, and M-a to M-e, each caught (by which check) or a gap;
- what only the first real tag can prove, and whether `RELEASING.md` tells the maintainer how to
  check it;
- optional cleanups, listed separately from punch items;
- your cost.
