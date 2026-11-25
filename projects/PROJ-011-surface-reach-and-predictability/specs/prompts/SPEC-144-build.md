# SPEC-144 — BUILD prompt: a tag push publishes every channel, on runners that exist

Cycle: **build**. Fresh CLI session, own worktree, **Opus**. The code is small. The hard parts are
the live two-commit PR run (one check red by design on commit (a)), the 16-row mutation table, and
telling "absent" from "could not ask".

```
just wt feat/spec-144-release-pipeline -- --model opus
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own. **It must land before the next tag:** GitHub retired `macos-14` on
2026-11-02, and the next release fails until this merges.

## What you are building
- `dist-workspace.toml`: the Apple Silicon build moves to `macos-15`; npm and crates.io become
  cargo-dist custom publish jobs (`./publish-npm`, `./publish-crates`) that need `host`; per-job
  permissions. `release.yml` is **regenerated** with `dist` 0.32.0, never hand-edited.
- `publish-crates.yml` (reworked in place) and `publish-npm.yml` (new): `workflow_call` plus a
  path-filtered `pull_request`, with three jobs each (`guard`, `publish`, `dry-run`).
- `scripts/release-publish-guard.sh` (with `--self-test`) and `scripts/check-release-pipeline.sh`.
- DEC-111 is already written. You add the amendment notes on DEC-067, DEC-041 and DEC-040, and the
  docs in Call 9.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. Then check all three:
- the spec is at `cycle: build`, and its design commit `bb8398c` is in your history;
- its `### Maintainer rulings` list goes up to **ruling 5** (added after design);
- `dist --version` prints `cargo-dist 0.32.0`, and `actionlint` is installed (record its version).

If any of those fails, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-144-a-tag-push-publishes-every-channel-on-runners-that-exist.md`.
   Read all of these in full:
   - `## Frame` and its rulings 1–5;
   - `## Design`: "What the probes showed" and Calls 1–9;
   - all 18 ACs;
   - `## Checks`: C1–C7, G1–G14, R1–R6, L1–L4, D1–D2, A1, V1–V3, and the mutation table;
   - `## Implementation Context` and `## Notes for the Implementer`.

   **The design calls are settled,** except where ruling 5 overrides them. If one turns out wrong,
   say so in Build Completion and in your report. Do not quietly re-decide it.
2. **DECs:** DEC-111 (new, from design), DEC-067, DEC-041, DEC-040, DEC-052, DEC-066, DEC-015.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for". Binding; traps 12, 13 and 15 apply directly. Also `/guidance/constraints.yaml`.
4. `RELEASING.md`, every file in `.github/workflows/`, and the `justfile` wasm recipes.

## Rulings (settled — do not re-open)
1. crates.io moves behind the binaries too; the tag-triggered `publish-crates.yml` goes away.
2. npm "disallow tokens" is turned on after the first green CI publish. Not your job: the
   maintainer does it.
3. No GitHub Environment or required reviewer.
4. crates.io keeps `CARGO_REGISTRY_TOKEN`.
5. **Least privilege: the crates.io job holds no `id-token`.** This overrides the design.
   - `[dist.github-custom-job-permissions]`: `"publish-crates" = { contents = "read" }`, and
     `"publish-npm" = { contents = "read", id-token = "write" }`.
   - In `publish-crates.yml`, the `publish` job holds `contents: read` only.
   - **Apply it to the spec too:** Call 1, Call 2's `publish-crates.yml` section (the paragraph
     that says `publish` holds `id-token` deliberately), AC-4, AC-7, C4, C7 and any mutation row
     that names crates permissions. C4 and C7 check each job's permissions separately; write each
     check so it can fail if the crates job gains `id-token`.
   - List every spec edit in Build Completion.

## Order (verify checks each step)
1. **The checks first, against unmodified `main`.** Write both scripts and the self-test. Run
   C1–C7 and G1–G14 on `main` and quote each RED **with its failure line** (trap 13): C1–C7 should
   fail on missing files or a stale config, not on a script bug. Commit and push.
2. **The config and the regeneration.** Edit `dist-workspace.toml`, then
   `dist generate --mode=ci`. Then `dist plan` shows the aarch64 runner is `macos-15`.
3. **The two workflows,** per Call 2 with ruling 5 applied. Then C1–C7, the self-test and A1 go
   GREEN. Commit and push: the WIP checkpoint.
4. **The local refusal runs R1–R6.** Quote each output and read `$?` directly, with no pipe. R5
   runs against the real registries at a `v0.8.2` worktree, and **must** say `SKIP`. R6 must say
   `COULD NOT ASK` and exit 3.
5. **Commit (a): `pr-run-mode = "upload"`.** Regenerate, commit, push, and open the PR **as a
   draft**. The Release workflow's PR run must build all four targets and must not host or
   publish. C2 is expected RED on (a), by design; quote it.
   - Wait with one backgrounded `gh run watch <id> --exit-status`. Never poll.
   - Record L1 (each job's name, conclusion and **runner label** from `gh api …/jobs`), L2 (the
     artifact list) and L3 (`file` on the downloaded aarch64 binary says `arm64`).
   - Record D1 and D2: the two dry-run jobs' quoted lines. The npm one shows `(dry-run)`; the
     crates one shows `cargo publish --dry-run` packaging.
   - **If a build job fails on a runner,** report it with its annotation. Do not retry blindly.
6. **Commit (b): remove `pr-run-mode`.** Regenerate, commit, push. Record L4 (the `release.yml`
   diff between (a) and (b): only the upload-mode lines), and C2 back to GREEN.
7. **Run the mutation table, M1a–M16, one row at a time,** in a scratch worktree. For each one:
   apply it, check `git diff` is non-empty, run C1–C7 and the self-test (and A1 where the row
   says), record every red check **with its quoted line**, then revert and confirm green. M1b
   says no config path exists; confirm that or report one.
8. **Docs and record (Call 9):** `RELEASING.md` (the one-time npm trusted-publisher setting naming
   **`release.yml`**, the publish order, re-running a failed publish job, when to turn on
   "disallow tokens", the hand fallback as recovery only, and the first-real-tag checklist),
   PROJ-011 `HANDOFF.md`, the comment-only edits in `justfile`, `scripts/wasm-npm-finalize.mjs`
   and `tests/npm_smoke.mjs`, and the amendment notes on DEC-067, DEC-041 and DEC-040
   (`superseded_by` stays `null`). Then run V1–V3 and `./scripts/decisions-audit.sh`.

## The things most likely to go wrong
- ⛔ **Nothing publishes or tags.** No `git tag`, no `npm publish` or `cargo publish` without
  `--dry-run`, no `gh workflow run` on any release or publish workflow, no change to npm or
  crates.io settings. The PR's own runs are the only live runs.
- **Never hand-edit `release.yml`.** C1 (`dist generate --mode=ci --check`) and the PR's `plan` job
  both fail on drift.
- **The merged PR must carry commit (b),** not (a). If `upload` merges, every PR builds four
  release targets. C2 guards it; keep C2 in CI.
- **"Could not ask" is never "absent".** A 404, a 5xx, a curl failure and an unparsable body all
  exit 3 and publish nothing. Only HTTP 200 without the version means publish.
- **The verdict is validated before anyone acts on it** (trap 15): the step after the guard fails
  unless the guard wrote exactly `publish` or `skip`.
- **No PR job holds a secret or `id-token`.** C7 checks it; read the generated permissions
  yourself too.
- **The self-test's fake registry** binds `127.0.0.1` on a port it reads back, and is killed by a
  `trap`.
- zsh: quote `"===="`-style separators, and don't rely on word-splitting. macOS has no `timeout(1)`;
  give curl `--max-time`.
- **Stay in your files:** `dist-workspace.toml`, `.github/workflows/release.yml` (regenerated
  only), `publish-crates.yml`, `publish-npm.yml`, the two new `scripts/` files, the comment-only
  edits, `RELEASING.md`, PROJ-011 `HANDOFF.md`, DEC-067, DEC-041, DEC-040, and the SPEC-144 spec
  (ruling 5's edits and Build Completion). Anything else is a deviation; say why.

## Guardrails
- **Every commit uses `git commit -s`,** ending with
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. The DCO check fails any PR with an
  unsigned commit.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump.
- **Never poll CI.** Background `gh pr checks <PR> --watch` and `gh run watch`. When one exits,
  read one direct snapshot at the PR's true head SHA, every leg including windows-latest. A watch
  that exits before the checks register has waited for nothing: confirm the checks exist first.
- A piped command reports the pipe's exit code: redirect and read `$?`.
- Delete scratch worktrees and downloaded artifacts when you're done. Keep scratch files in a
  scratch directory, never in `~`.
- **Budget ~90 exchanges.** If the PR run is green and you are past that, stop and report what is
  left.

## When you finish
1. **Fill `## Build Completion`:**
   - every file from `git diff --name-only origin/main...HEAD`, not from memory;
   - ruling 5's spec edits;
   - C1–C7 and G1–G14 RED on `main`, each with its line, then GREEN;
   - R1–R6, L1–L4, D1–D2, A1 and V1–V3, quoted;
   - each mutation's red checks, with lines;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**: search for `BUILD prompt: a tag push publishes`.
     `grep -c` can come back empty through the `rtk` wrapper; count with Python or
     `/usr/bin/grep`.
   - ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines
     that share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
     **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
     output under-counts about 11×.
   - Price each component at the model that actually ran (read `.message.model`), never at a flat
     rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and
     cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance and leave the PR a draft.**
   - Run `just advance-cycle SPEC-144 verify`.
   - Do **not** edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `ci(release): publish npm and crates.io after the binaries; macos-15 runner (SPEC-144)`.
   - Body per AGENTS §13: Summary; Spec metadata; Decisions referenced (DEC-111 new, DEC-067,
     DEC-041 and DEC-040 amended, DEC-052, DEC-066); Constraints checked; the one-time npm setting
     the maintainer must make **before the next tag**; and the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-144 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short:
- commits (a) and (b) with their SHAs, and that the head carries (b);
- AC completion (18 rows);
- ruling 5's edits;
- C1–C7 and G1–G14 RED on `main` (with lines), then GREEN;
- R1–R6 results; L1 (the runner label for each target, quoted), L2, L3, L4; D1 and D2; A1;
- each mutation's red checks;
- files changed vs the spec's list;
- any design call that turned out wrong;
- any out-of-spec finding;
- your measured cost.
