# SPEC-144 — DESIGN prompt: a tag push publishes every channel, on runners that exist

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-144-design -- --model opus
```

Starts from the stub the frame session pushed to `main` (`7f0c4c8`). **Reserved: SPEC-144,
DEC-111.** The frame recommends a new DEC-111 over amending DEC-067. It **must land before the
next tag**: GitHub retired `macos-14` on 2026-11-02, so the next release fails until it does.

**Read the stub's `## Frame` section first.** It holds the premise evidence, the runner fix and its
`dist plan` probe, the PR-proof plan, the npm trusted-publishing facts with quotes, Option A vs B,
the DEC-111 draft rule, the doc sentence list, the sizing and the file list. It is your binding
scope. Its `### Maintainer rulings` subsection is settled.

## Rulings (settled — do not re-open)
1. **crates.io moves behind the binaries too:** a cargo-dist custom publish job after `host`. The
   tag-triggered `publish-crates.yml` goes away.
2. **npm "disallow tokens" is turned on after the first green CI publish,** not before.
3. **No GitHub Environment or required reviewer.** The tag push is the one deliberate act.
4. **crates.io keeps `CARGO_REGISTRY_TOKEN`** in this spec. Its trusted publishing is a later
   STAGE-054 entry.
5. From the frame, accepted by the orchestrator:
   - Option B: npm and crates.io run as cargo-dist custom publish jobs (`./publish-npm`,
     `./publish-crates`), `needs: [plan, host]`;
   - `[dist.github-custom-runners] aarch64-apple-darwin = "macos-15"`; stay on dist **0.32.0**;
   - the npm trusted-publisher setting names **`release.yml`**, the calling workflow;
   - out of scope: ubuntu-22.04 (its own entry), a dist upgrade, crates.io OIDC, npm package
     changes, wasm tests in CI (its own entry).

## Design calls that are yours
- **The two publish workflows, line by line.** Write `publish-npm.yml` and the reworked
  `publish-crates.yml` (or a new name; say which, and why) as `workflow_call` jobs. For each one,
  give:
  - the trigger and inputs cargo-dist passes (`plan`), and what the job reads from them;
  - `permissions` (`id-token: write`, `contents: read`), and whether `secrets: inherit` reaches the
    crates token;
  - the toolchain steps. Copy from `pages.yml` where you can. Node 24 via `actions/setup-node` so
    `npm` ≥ 11.5.1; state the exact action versions;
  - the publish command: `just wasm-npm-publish` unchanged, and `cargo publish --locked`.
- **Guards before any publish.** The tag version equals `Cargo.toml`'s version (`GITHUB_REF_NAME`
  inside a `workflow_call`: check which ref it holds). The tree is clean. `pkg/` matches
  `Cargo.toml`. Say where each guard lives, and what it prints when it refuses.
- **Idempotent re-runs.** Each job skips cleanly, with exit 0 and a clear line, when that exact
  version is already on its registry: `npm view crustyimg-wasm@X version` and the crates.io API
  (a `User-Agent` is required). A registry outage or a 404 must not be read as "not published".
  Say how the check tells "absent" from "could not ask", and what happens on each.
- **Prereleases.** cargo-dist skips custom publish jobs for prereleases unless
  `publish-prereleases` is set. Decide whether a `v0.9.0-rc.1` tag should publish anywhere, and
  what it does today with `publish-crates.yml` gone.
- **Proving it before the next real tag.** The frame's plan: the build PR sets
  `pr-run-mode = "upload"`, so that PR's Release run builds all four targets without hosting or
  publishing; a final commit sets it back to `"plan"`. Specify:
  - what the build records from that run (each target's runner label, from the job API, plus the
    artifact list);
  - how verify proves the publish jobs are **wired** without running them (the regenerated
    `release.yml`'s `needs:` graph, the `if:` conditions, and `actionlint` if available);
  - a dry run of each publish step that runs in the PR: `npm publish --dry-run` through the recipe,
    or a recipe variant, and `cargo publish --dry-run --locked`. Can a `workflow_call` job run in
    dry-run mode on a PR without any path to a real publish? If not, say what replaces it;
  - the **first real tag** as the only proof of the OIDC exchange: what the maintainer checks, and
    the documented hand fallback if it fails.
- **The generated file.** `release.yml` is generated. The build must regenerate it with
  `dist generate` at 0.32.0, never hand-edit it. Specify the check that proves it: re-running
  `dist generate` leaves `git diff` empty. Does CI already run `dist`'s own consistency check
  (`plan` fails on a stale workflow)? Find out.
- **The DEC.** Write DEC-111 from the frame's draft rule, plus the supersede notes on DEC-067
  (Publishing section) and DEC-041 (point 2), and the DEC-040 extension. Or amend instead, and say
  why. Use the block-list `affected_scope` form.
- **Docs.** Word each sentence in the frame's Q7 list. `RELEASING.md` must say, plainly:
  - the one-time npm setting (package → Settings → Trusted publisher → GitHub Actions,
    `jysf` / `crustyimg` / `release.yml`);
  - the order a tag publishes in;
  - what to do when one publish job fails (re-run failed jobs);
  - when to turn on "disallow tokens" (ruling 2);
  - the hand fallback, as recovery only.

## Acceptance and tests: each one must be able to fail
This spec changes workflows, not `src/`, so most checks are structural or live-run. Each AC still
names how it is checked and what would make that check go red:
- **Structural checks of the generated `release.yml`:** the aarch64 runner is `macos-15`, both
  custom jobs need `host`, `announce` needs both, and no other job moved. Prefer a script, kept in
  `scripts/` and run in CI or at verify, over prose, so verify can run it against a mutated file.
- **The PR's live run:** four targets built, with the runner labels quoted from the API.
- **The guards:** run each refusal locally against a scratch checkout: tag ≠ version, a dirty tree,
  "already published" with a version that is on the registry, and "registry unreachable". Each
  prints its line and gives the exit code you specify.
- **A mutation table (trap 12),** with the exact location for each row and every check expected
  red. At least: drop `host` from a custom job's `needs`; hand-edit `release.yml` without
  regenerating it; remove the runner override; remove the tag-version guard; make the idempotency
  check treat a 5xx as "absent".
- **Trap 13:** each RED row names the failure message that proves it is red for its reason.

**Size.** The frame says low [M]. If it grows past M, report the cut before writing it.

## Files
The frame's list: `dist-workspace.toml`; `.github/workflows/release.yml` (regenerated);
`.github/workflows/publish-npm.yml` (new); `.github/workflows/publish-crates.yml` (reworked or
replaced); comment-only edits to `justfile`, `scripts/wasm-npm-finalize.mjs` and
`tests/npm_smoke.mjs`; `RELEASING.md`; PROJ-011 `HANDOFF.md`; `decisions/DEC-111-…md` (new),
`DEC-067-…md` and `DEC-041-…md` (supersede notes), DEC-040 if extended; any `scripts/` check you
add.

## DECs to read
DEC-040, DEC-041, DEC-052 (what release artifacts build), DEC-066 (the wasm size profile), DEC-067,
DEC-015 (exit codes, for any script you add), and any DEC naming `.github/workflows/` or
`dist-workspace.toml` (`./scripts/decisions-audit.sh`).

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-144-*.md` with `cycle: design`, and
its `### Maintainer rulings` list must exist. If not, stop and report; do not create it. A
separate session builds it from your spec alone, and another verifies it, so everything the
builder needs must be in the spec (AGENTS §15).

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (1–13). Binding. Traps 12 and 13 apply directly.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`:
   SPEC-144's entry, the ubuntu-22.04 and crates.io-OIDC entries (both out of scope), and the
   success criteria.
4. `RELEASING.md`, `dist-workspace.toml`, every file in `.github/workflows/`, and the `justfile`
   wasm recipes.
5. `/guidance/constraints.yaml` and every DEC named above.
6. The tools' own docs, for the versions in use: cargo-dist 0.32.0's config reference (custom
   publish jobs, `github-custom-runners`, `pr-run-mode`, `publish-prereleases`) and npm's
   trusted-publishing page. Quote at most one short phrase per fact, and cite the page.

### How to design well here
- **Probe, don't assume.** Run `dist plan` and `dist generate` in a scratch copy and read the
  output. Discard the scratch copy.
- ⛔ **Nothing that publishes or tags.** No `git tag`, no `npm publish` (except `--dry-run`), no
  `cargo publish` without `--dry-run`, no `gh workflow run` on `release.yml` or any publish
  workflow, no change to npm or crates.io settings.
- **Ask what the maintainer actually sees** at each step of a release: the tag push, the Actions
  page, a failed job, a re-run.
- **Count the nouns in every AC against the checks.** An AC naming two jobs needs a check per job.

### Ids are reserved — use exactly these
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests (or "Checks", if that reads
truer for workflows; say so) and Implementation Context; keep `## Frame`. If you find the frame
wrong, say so in your report rather than silently widening or narrowing it. The highest DEC on
`main` was DEC-110 when this was written. Other projects add DECs on `main` in parallel: re-check
`ls decisions/` after your last rebase, and if DEC-111 is gone, take the next free id and say so.
Use the **block-list** `affected_scope` form (an inline array is silently dropped by
`scripts/decisions-audit.sh`):
```yaml
affected_scope:
  - .github/workflows/release.yml
```

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR.
1. `just advance-cycle SPEC-144 build`.
2. In STAGE-054's backlog, change **only** SPEC-144's marker from `(framed 2026-10-31)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2–4 line summary, and add the
   build prompt path `prompts/SPEC-144-build.md` to the build line.
3. `git commit -s`, ending the message with
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **The DCO sign-off is mandatory.**
   Message: `docs(SPEC-144): design the tag-driven release pipeline`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. Other sessions
   push to `main` too; if the push is rejected, rebase and push again.
5. Read back `git show --stat HEAD` and run `just backlog`.

### Guardrails
- ⛔ No workflow or code changes outside the spec, timeline, stage file and the DEC. No version
  bump, no tag, no publish.
- A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~55 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** search it for `DESIGN prompt: a tag push publishes`. `grep -c` can come back empty
through the `rtk` wrapper; count with Python or `/usr/bin/grep`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read
×0.10. Append a `design` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown` and `estimated_usd`, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-144 design: <commit SHA on main> — <N> ACs, <M> checks`. Then, short:
- each design call and how you settled it;
- the two publish workflows in outline: trigger, permissions, toolchain, guards, publish command;
- the idempotency check, and how it tells "absent" from "could not ask";
- prereleases: what a `-rc` tag does;
- the PR-proof plan, the dry runs, and what only the first real tag can prove;
- the generated-file check;
- the mutation table: each mutation's location and every check expected red;
- the file list the build will touch;
- whether DEC-111 is used or released;
- anything that needs a **maintainer** ruling, each with your recommendation;
- whether to build on Sonnet or Opus, and why;
- your measured cost.
