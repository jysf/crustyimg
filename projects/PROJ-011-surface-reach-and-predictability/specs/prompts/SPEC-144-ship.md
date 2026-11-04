# SPEC-144 — SHIP prompt: a tag push publishes every channel, on runners that exist

Cycle: **ship**. This is bookkeeping only. The code is on `main` as `a13d568` (PR #243,
squash-merged 2026-11-04).
- Verify returned ✅ APPROVED at `17c8b8a`, with no punch items. The orchestrator marked the PR
  ready.
- Before the merge, the branch was updated from `main` (`197906e`). That brought in only
  orchestrator docs commits. The macOS test leg was then cancelled because no arm64 runner was free.
  A re-run passed, and the maintainer merged.
- **AC-18 is still open by design.** The first real tag is the only thing that can prove it: the npm
  OIDC exchange, a real npm publish, and `cargo publish` from the new job. Ship this spec now and
  record AC-18 as pending the first tag. Do not mark it met.

Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-144-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. Open the timeline:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-144-a-tag-push-publishes-every-channel-on-runners-that-exist-timeline.md`.
3. ⛔ **Proceed only if both of these hold.** Otherwise stop and report.
   - The verify line is `[x]` and says ✅ APPROVED and that the PR was marked ready.
   - PR #243 is merged: `gh pr view 243 --json state,mergeCommit`.

## Read first
- `/AGENTS.md` §4 (cost), §13 (verify and ship bookkeeping lands on `main`) and §15 "During
  ship".
- The spec in full:
  - `## Frame` with rulings 1–5;
  - `## Design` with Calls 1–9;
  - the 18 ACs and the mutation table;
  - `## Build Completion` with its build-phase reflection;
  - the cost block.
- The timeline: the build and verify lines.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
  (1–15) and the rules.
- STAGE-054's `## Spec Backlog`, all of it.
- `RELEASING.md`: the "first tag after the publish jobs moved" checklist.

## What build and verify reported (the facts to reflect on)
- **Ruling 5 came after design.** The maintainer chose least privilege (the crates.io job holds no
  `id-token`), and the build applied it to the spec: Call 1, Call 2, AC-4, AC-7, C4, C7, M11, and
  two new rows, M17 and M18.
- **One design call would have failed in CI.** Design set `npm_config_dry_run=true` for the whole
  dry-run step. That also turns the smoke test's `npm pack` into a dry run, so pack writes no
  tarball. The build replaced it with an `npm` wrapper on the dry-run job's `PATH` that fakes only
  `publish`, aimed at the dead address `127.0.0.1:9`. No design reviewed the wrapper. Verify
  confirmed:
  - the tag-path publish job cannot see it;
  - with the dry-run flag removed, it still targets the dead port and fails to connect.
- **AC-15 was relaxed.** `dist`'s generated `release.yml` has 6 shellcheck findings that are
  identical on `main`.
- **Out-of-spec finding.** `dist`'s `github-release = "announce"` moves the GitHub Release creation
  from `host` to `announce`, after the registry publishes, without changing any `needs`. C1–C7 all
  stayed green under it. The build made C3 assert that only `host` creates the Release, and added
  M19.
- **The design caught two gaps of its own through probes.** Custom jobs get no `contents`
  permission. `npm view` cannot tell "absent" from "unreachable". Both were fixed in design, before
  any code.
- **The orchestrator's premise was wrong.** The verify prompt said no CI job runs the check
  scripts. In fact, `publish-crates.yml`'s PR dry-run job runs both, path-filtered. The check is not
  required, and verify ruled that this is not a punch item.
- **Verify's gaps (none can cause a wrong publish):**
  - the npm guard reads a 200 with no `.versions` key as "publish";
  - no G row covers an empty versions list (M-b);
  - no check catches a workflow-level or publish-job `npm_config_*` env (M-d2, M-d3);
  - the path filter misses `.github/workflows/**` and `justfile`;
  - actionlint runs nowhere automatically;
  - DEC-111 says nothing about ruling 5 or the wrapper.

  ⛔ **These are already filed** as one STAGE-054 entry, "Tighten the release-pipeline checks
  SPEC-144's verify left open" (`2e69a00`). Don't file them again.
- **Two runner incidents.** The `heic (ubuntu)` leg failed once on a runner disk-full error. On the
  updated head, the macOS leg was cancelled because no arm64 runner was free. Both passed on re-run,
  and neither involved this diff.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Mark AC-18 as pending, not met.** Find the place where the spec records AC completion (Build
   Completion's AC table, or the ACs themselves). Make AC-18 read as pending the first real tag,
   pointing to RELEASING.md's checklist. Change nothing else in the ACs.
3. **Write `## Reflection (Ship)`.** Make it outcome-focused, with three answers drawn from the
   evidence above, the Build Completion and the timeline. Do not invent anything.
   1. **What would I do differently next time?** Weigh at least these three:
      - a design call setting an environment variable for a whole step, which changed every command
        in that step, not only the one it targeted;
      - a security ruling (least privilege) that arrived after design and had to be threaded
        through seven places;
      - an orchestrator premise ("nothing runs the checks") that was never checked against the
        workflows.
   2. **Does any template, constraint or decision need updating?** Two candidates for a HANDOFF
      trap:
      - "an env var set on a step applies to every command in that step";
      - "inside a reusable workflow, `github.*` is the caller's context".

      Check traps 1–15 first; if one already covers it, say which. **Propose it; do not apply it.**
      DEC-111's missing ruling-5 sentence is already in the filed STAGE-054 entry.
   3. **Is there a follow-up spec to write before I forget?** Read STAGE-054's backlog first. Two
      entries already exist: the crates.io trusted-publishing entry and the tighten-the-checks
      entry. File something only if the evidence shows a gap that neither covers. Otherwise say
      "none" and why.
4. **Cost.** `cost.sessions` holds three entries: frame ($4.15), design ($4.78) and build ($11.97).
   Each was measured before its session's last turns.
   - **Re-measure all three** from their full transcripts, and replace the numbers if they differ.
   - **Add one entry for verify**, plus one for your own `ship`.
   - Every transcript is under `~/.claude/projects/`. The `…` in each path is
     `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg--claude-worktrees-`.

     | cycle | transcript | confirm by content | reported |
     |---|---|---|---|
     | frame (re-measure) | `…docs-spec-144-frame/8805a15c-159f-4fba-94e9-e92a1e2c095b.jsonl` | `FRAME prompt: a tag push` | $4.15 |
     | frame (check) | `…docs-spec-144-frame/1c28d8fd-69fe-492b-959a-f275f38cbf9d.jsonl` | see below | — |
     | design (re-measure) | `…docs-spec-144-design/90249c55-61e1-4782-b4f0-a1bcd9168345.jsonl` | `DESIGN prompt: a tag push` | $4.78 |
     | build (re-measure) | `…feat-spec-144-release-pipeline/86638582-3e1a-490e-8cba-d0e3a25b6c5b.jsonl` | `BUILD prompt: a tag push` | $11.97 |
     | verify (add) | `…verify-spec-144/91387cb9-c46c-4e24-8ec6-4c216b039f3f.jsonl` | `VERIFY prompt: a tag push` | $5.30 |

   - **The second frame transcript** (`1c28d8fd`, 6 message ids, 2026-11-01 03:57–05:57 UTC) is a
     separate, later session in the frame worktree. Read what it did.
     - If it did SPEC-144 work, such as recording the maintainer's rulings, add it as its own
       `frame` entry, with a note saying what it did.
     - If it did something else, leave it out and say why.
   - **Confirm each transcript by content before measuring it.** `grep -c` can come back empty
     through the `rtk` wrapper; count with Python (`open(f).read().count(...)`) or `/usr/bin/grep`
     run outside a command substitution. Read the model from each transcript's `.message.model`;
     don't assume it. The dollar figures above are the sessions' own estimates, for
     sanity-checking only. Do not copy them.
   - **Recompute `cost.totals` from every entry:**
     - `tokens_total` is the sum of the non-null totals;
     - `estimated_usd` is the sum of the non-null values;
     - `session_count` counts every entry.

     Run `just cost-audit` and confirm SPEC-144 passes. Quote the line that names it; if no line
     names it, say so.
5. **Mark ship `[x]`** in the timeline, with the merge (`a13d568`, PR #243, 2026-11-04), the total
   cost, and "AC-18 pending the first real tag". Add a `[x]` re-verify line only if the timeline has
   an open one; it should not.
6. **Advance and archive.** Run `just advance-cycle SPEC-144 ship`, then
   `just archive-spec SPEC-144`. Then read STAGE-054's SPEC-144 entry back.
   - It should read `- [x] **SPEC-144** (merged 2026-11-04, PR #243) — …`. Fix it by hand if the
     script left it otherwise. Touch no other entry.
   - Recount the `**Count:**` line with a grep you have just run, not by arithmetic. Today it
     reads `4 shipped / 0 active / 25 pending`, with SPEC-144 still `[ ]`. Expect
     `5 shipped / 0 active / 24 pending`, plus any entry you file in step 3.3. If the grep
     disagrees, trust the grep and say why.
7. **Check what the tooling sees.**
   - Run `just status` and `just backlog`. Confirm SPEC-144 shows as shipped.
   - Run `just roadmap`: STAGE-054 must stay **active**. ⛔ **Do not run Stage Ship**, and do not
     mark STAGE-054 shipped; the orchestrator decides that.
8. **Commit to `main` directly.** Ship bookkeeping does not go through a PR (AGENTS §13).
   - Commit:
     `git commit -s -m "ship(SPEC-144): a tag push publishes every channel — reflected, archived"`,
     with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. **The DCO
     sign-off is mandatory.**
   - Push: `git fetch origin && git rebase origin/main && git push origin HEAD:main`. Other
     sessions push to `main` too; if the push is rejected, rebase and push again.
   - Read back `git show --stat HEAD` and check it is what you meant.

## Guardrails
- ⛔ No version bump, no tag, no release, no publish and no CHANGELOG edit. Do not touch npm or
  crates.io settings. Do not run any release or publish workflow.
- Do not edit code, tests, CI, workflows, DECs, `docs/`, `RELEASING.md`, HANDOFF or templates. If
  the reflection says one needs changing, propose the edit in your report; the orchestrator
  decides.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

## Measure cost (yours and the transcripts above)
- Your own transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Identify it **by content, not recency**: it contains `SHIP prompt: a tag push`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran, never at a flat rate: Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-144 ship: archived — <commit SHA on main>`. Then:
- how AC-18 now reads;
- the three reflection answers, one line each;
- any follow-up you filed, or "none" and why;
- the re-measured frame, design and build entries, old → new;
- what the second frame transcript was, and whether you added it;
- the new verify and ship entries, each with cycle, model, tokens and USD;
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- the recounted `**Count:**` line, and confirmation that STAGE-054 is still active;
- any template, constraint or DEC change you propose;
- your measured cost.
