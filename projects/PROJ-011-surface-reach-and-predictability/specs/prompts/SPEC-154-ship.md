# SPEC-154 — SHIP prompt: the CI path classifier runs the gates for every build and test input

Cycle: **ship**. Bookkeeping, plus the post-merge proof that the build PR couldn't give (AC-18).
Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-154-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to the
orchestrator, so that message must stand on its own. **Write it as plain lists, not box-drawn
tables.**

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. ⛔ **Proceed only if both hold.** Otherwise stop and report.
   - PR #266 is merged: `gh pr view 266 --json state,mergeCommit,mergedAt`. Record what you see.
   - The timeline
     (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-154-ci-path-classifier-runs-the-gates-for-every-build-and-test-input-timeline.md`)
     has build and verify `[x]`, and the punch list done (as its own line or inside verify's).
3. Confirm the merge brought the work onto `main`:
   - `scripts/ci-code-paths.sh` holds `DOCS=` and `EXCEPT=`;
   - the spec's `cost.sessions` has `build`, `verify` and `punchlist` entries;
   - `bash scripts/ci-code-paths.sh --self-test > out.txt 2>&1; echo rc=$?` gives rc=0 and 88 rows.

   If not, stop and report.

## Read first
- `/AGENTS.md` §4 (cost), §13 (ship bookkeeping lands on `main`) and §15 "During ship".
- The spec in full:
  - Frame, with the maintainer rulings;
  - Design and its corrections;
  - the ACs, especially AC-9 and AC-18;
  - the mutation table;
  - "What the build cannot prove, and ship's step";
  - `## Build Completion` with its `### Punch list (verify)` subsection;
  - the cost block.
- The timeline, all of it.
- STAGE-063's `## Spec Backlog`
  (`projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`).
  Read the SPEC-154 entry, and the entry "A rename out of code into a docs-only folder skips the CI
  gates", which verify's finding became.

## The facts to reflect on (from frame to punch list)
- **The frame ($3.08)** proved the premise. A PR touching only `recipes/web.toml` showed all three
  required checks green with nothing built or tested, and pages didn't run. It found 20 misclassified
  paths in 9 groups. It recommended a docs-only allow-list, the README exception and a
  `docs/development.md` paragraph, and the maintainer took all three.
- **Design ($5.70)** corrected the frame three ways:
  - the allow-list costs 0 extra runs over a fixed deny-list (the frame said 2);
  - pages needs `recipes/**` for the smoke test's byte comparison, not the wasm bundle;
  - the include check needed two guards against passing vacuously: a floor on the number of
    included files the scan finds, and a check that each resolved path is a real repo path.

  It wrote the 227-line script verbatim and drove 32 mutations. It could run the script only under
  BSD tools.
- **Build (Sonnet, $1.37 as measured)**:
  - both baselines and the script matched the spec byte for byte;
  - all 32 mutations matched the table, with no wrong cell;
  - the PR's Linux `changes` job printed 88 rows, 0 failed, so there was no GNU/BSD difference;
  - it force-pushed its rebased branch;
  - its id grep found two fixture paths, which it recorded as a deviation.
- **Verify ($4.13)**:
  - it ran all 1,303 tracked paths through both scripts: 0 moved from code to docs-only, and 39
    moved to code, exactly the design's list;
  - it traced every command each gated job runs and found no docs-only file read;
  - its punch list had one wording item and one assert item, neither changing a verdict:
    - ship's proof edit to `recipes/web.toml` would have turned the pages smoke red;
    - the include check reads only the plain-literal form, narrower than two sentences claimed.
  - It found a pre-existing hole: git reports a rename as its new path only, so moving `src/x.rs`
    to `docs/x.rs` skips the gates. This is now a STAGE-063 entry.
  - It used `git checkout --detach`, because auto mode blocked `reset --hard`.
- **Punch list:** one session did the work as `80fc4bb`:
  - ship's proof now uses `recipes/gallery.toml`, which nothing byte-compares;
  - `docs/development.md` and AC-9 were narrowed;
  - the script was left unchanged.

  A second session, started from the same prompt, found the head had moved and stopped without
  changes, as the prompt told it to. It left the script header's over-claiming sentence and AC-18's
  `web.toml` wording for the orchestrator. The header sentence is now noted in the STAGE-063 rename
  entry.

## Do, in order
1. **Mark ship `[~]`** in the timeline, and add a `merge` line above it:
   `- [x] **merge** (<date>, maintainer) — PR #266 as <merge SHA>.` Check it against `gh`.
2. **The post-merge proof (AC-18).** This is the only evidence that the classification works on
   Actions.
   - Make a branch from `main`, named `proof/spec-154-classifier`. Add one comment line at the
     **end** of `recipes/gallery.toml`, such as `# classifier check, not for merge`. Commit with
     `git commit -s`, push, and open a **draft** PR titled
     `proof: a recipes-only change runs the gates (do not merge)`.
   - Run `gh pr checks <n> --watch` once, in the background, and take one snapshot when it exits.
     Never poll, and never write your own wait loop.
   - From that snapshot and the `detect changed paths` job log (one read), quote:
     - the verdict line `-> code paths touched: running the full gates`;
     - that each of the three `build / test / clippy / fmt (<os>)` legs ran its Test step and didn't
       skip it;
     - that `build + browser smoke` (pages) ran.
   - Then close the PR **unmerged** and delete its branch (`gh pr close <n> --delete-branch`).
   - If any of the three didn't happen, stop and report. Don't mark AC-18 met.
3. **The docs-only side.** Your own ship commit (step 8) is docs-only. After you push it, read the
   `changes` job of that push's CI run once, and quote `-> docs/specs only: heavy jobs will skip`. If
   the run hasn't finished, say so and quote the most recent completed docs-only push on `main`
   instead, with its SHA.
4. **Correct the spec in place** (mark each `(corrected at ship)`). Only these, the reflection and
   the cost block change in the spec body:
   - **AC-18:** it still names `recipes/web.toml`. Change it to `recipes/gallery.toml`, and tick it
     with step 2's evidence.
   - Leave the Frame's history as written.
5. **Write `## Reflection (Ship)`**, outcome-focused, three answers drawn only from the evidence:
   1. **What would I do differently?** Weigh at least these:
      - The frame's numbers (cost per extra run, pages' reason) were reasoned and design corrected
        them; the classifier was right either way.
      - Design couldn't run GNU tools, so the build PR's own `changes` job became the Linux run of
        record. That worked here, but only because the job prints the self-test.
      - The proof edit chosen at design would have turned the pages smoke red. No one checked what
        reads the file being edited for the proof.
      - Two punch-list sessions ran from one prompt. The second stopped correctly at the moved head.
   2. **Does any template, constraint or decision need updating?** Candidates:
      - "a proof edit touches a file nothing byte-compares; grep its readers first";
      - verify and punch-list prompts use `git checkout --detach origin/<branch>`, since auto mode
        blocks `git reset --hard`;
      - "a self-test that can only check written rows needs an independent whole-tree pass at verify"
        (verify's all-paths comparison).
      Check AGENTS §15–16, HANDOFF, `guidance/` and recent ship reflections first, and say which
      already covers each. **Propose it; don't apply it.**
   3. **A follow-up spec?** The rename hole is already filed in STAGE-063, and two smaller items are
      in `guidance/ideas.yaml` (quoted non-ASCII paths, the include floor's drift). File nothing new
      unless the evidence names a gap none of these covers, and say why.
6. **Cost.** Every session's entry must be in `cost.sessions`: frame, design, build, verify,
   punchlist, and your own `ship`.
   - **Re-measure build.** Its $1.37 over 23 messages was measured before its CI wait and final
     turns.
   - **The punch list had two sessions.** Measure both and record one `punchlist` entry with their
     sum; its note names both.
   - Check that verify's entry, which the punch list recorded from verify's report, matches its
     transcript.
   - **Find each transcript by its marker.** The markers are each prompt's title on `main`:
     - `SPEC-154 — FRAME prompt`
     - `SPEC-154 — DESIGN prompt`
     - `SPEC-154 — BUILD prompt`
     - `SPEC-154 — VERIFY prompt`
     - `SPEC-154 — PUNCH LIST prompt`

     Read each prompt's first line to confirm its title. Use the transcripts whose **first user
     message** holds the title, because the orchestrator's transcript also contains every title.
   - **Dedup by `.message.id`.** Count the three input counts once per id, and take the **MAX**
     `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
   - **Price** at the model in `.message.model`: Sonnet $3/$15, Opus $5/$25. cache_creation is
     ×1.25 the input rate and cache_read ×0.10.
   - Recompute `cost.totals` from every entry. Run `just cost-audit` and quote the line that names
     SPEC-154. If it fails on another spec, say which and that it isn't yours.
7. **Mark ship `[x]`** with the merge SHA, PR #266, the date and the total cost. Then run
   `just advance-cycle SPEC-154 ship` and `just archive-spec SPEC-154`.
   - Read STAGE-063's SPEC-154 entry back. It should read
     `- [x] **SPEC-154** (merged <date>, PR #266) — [S] …`, in the style of the stage's other shipped
     entries. Fix it by hand if not.
   - Touch no other entry.
   - Recount the `**Count:**` line with greps you just ran, and show them. Today it reads
     `3 shipped / 0 active / 20 pending`; expect `4 shipped / … / 19 pending`, unless another session
     changed the stage meanwhile (say so if one did).
   - ⛔ Don't change STAGE-063's `status` or its reflection. The stage has far more to ship.
8. **Commit to `main` directly** (AGENTS §13).
   - Commit with
     `git commit -s -m "ship(SPEC-154): CI runs the gates for every path not provably docs-only — reflected, archived"`
     and the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. DCO sign-off is
     mandatory.
   - Push with `git fetch origin && git rebase origin/main && git push origin HEAD:main`. On a
     reject, rebase and push again.
   - Read back `git show --stat HEAD`, then do step 3.

## Guardrails
- ⛔ No version bump, tag, release or CHANGELOG edit.
- Don't edit code, the script, CI, `docs/`, HANDOFF, templates or any DEC. The only branch you push
  besides `main` is the proof branch, and you delete it. Propose anything else in your report.
- The proof PR stays a draft and is closed unmerged. Never merge it, and never mark it ready.
- The spec's body changes only by step 4's corrections, the reflection and the cost block.
- Another session may push to `main` while you work. Rebase; never force.
- A piped command reports the pipe's exit code. zsh does not word-split unquoted parameters, and it
  reads `$R:r…` as a modifier, so brace it (`${R}:`). `rtk` can truncate `grep`, so count with
  `/usr/bin/grep` or Python.
- Write scratch files only in your session's scratchpad, never `/tmp`.
- **Budget ~40 exchanges.**

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-154 ship: archived — <commit SHA on main>`. Then:
- the merge SHA and date;
- the proof PR's number, its three quoted results, and that it was closed unmerged and its branch
  deleted;
- the docs-only push's `changes` line, with its SHA;
- AC-18 as it now reads;
- STAGE-063's SPEC-154 line as it now reads, and the recounted `**Count:**` line with its greps;
- the three reflection answers, one line each;
- any follow-up filed, or "none" and why;
- every cost entry you added or re-measured, old → new;
- `cost.totals` and the `just cost-audit` line;
- any template, constraint or DEC change you propose;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
