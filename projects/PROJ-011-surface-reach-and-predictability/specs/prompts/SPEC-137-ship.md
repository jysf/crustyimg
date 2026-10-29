# SPEC-137 — SHIP prompt: SVG text in any font family renders in the bundled font

Cycle: **ship**. This is bookkeeping only. The code is on `main` as `71b4f38` (PR #217,
squash-merged 2026-10-29).
- Verify returned a punch list at `fb09421`, with one item: the Linux system-font row passed
  silently in CI.
- The punch list fixed it (`5cfd87f`, docs `3de8ccf`). The orchestrator read CI run 40931258121
  and found a `using <font>` line on each OS: DejaVu Sans, Helvetica and Arial. Then it marked
  the PR ready.
- CI on `main` passed at `71b4f38`.

Fresh CLI session, own worktree; Sonnet is enough:

```
just wt docs/spec-137-ship -- --model sonnet
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Step 0 — precondition, then get onto current `main`
1. `git fetch origin && git rebase origin/main`.
2. Open the timeline:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-137-svg-text-in-any-font-family-renders-in-the-bundled-font-timeline.md`.
3. ⛔ **Proceed only if both of these hold.** Otherwise stop and report.
   - The punch-list line is `[x]` and says PR #217 was marked ready.
   - PR #217 is merged: `gh pr view 217 --json state`.

## Read first
- `/AGENTS.md` §4 (cost), §13 (verify and ship bookkeeping lands on `main`) and §15 "During
  ship".
- The spec in full: `## Frame` with its rulings, `## Design`, the DEC-054 amendment,
  `## Build Completion` with its `### Punch list (verify)` subsection, and the cost block.
- The timeline.
- `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
  and the rules.
- STAGE-054's `## Spec Backlog`, all of it. SPEC-137's entry is at about line 230.

## One thing that changed at merge
PR #218 (ddphotos integration) merged just before #217. It put SVG decoding behind a default-on
`svg` feature. As a result, the merged `tests/svg_font_family.rs` is gated
`#![cfg(all(feature = "svg", not(target_arch = "wasm32")))]`, and the wasm rows carry
`#[cfg(feature = "svg")]`.
- The `--no-default-features` (lean) CI leg no longer runs SPEC-137's tests. That is by design:
  a lean build refuses SVG input outright.
- `just wasm-test` still passes `svg` explicitly (`_wasm_features`), so the wasm row still runs
  there.

Confirm both by reading `71b4f38`'s `Cargo.toml`, `justfile` and the test headers, not this
summary. Record the change in the reflection. It is not a defect.

## Do, in order
1. **Mark ship `[~]`** in the timeline.
2. **Write `## Reflection (Ship)`.** Make it outcome-focused, with three answers drawn from the
   evidence: the Build Completion, the verify readout in the timeline, and the merge history. Do
   not invent anything.
   1. **What would I do differently next time?** Facts to weigh:
      - **The design asked for something CI cannot show.** It said a Linux SKIP was expected and
        told the build to quote the SKIP line from the CI log. libtest hides stderr for passing
        tests, so neither a SKIP line nor a `using` line could ever appear. The build inferred
        from green CI, and only verify caught it. The fix made the skip fail when coverage is
        required, and printed the font with `--show-output`.
      - **Two hashes in the spec were never pasted from a run.** The blank render `bb918147…`
        should have been `bd30b081…`, and the mixed `<tspan>` row `6893dab3…` should have been
        `1accc9c7…`. No assertion used either, so nothing went red.
      - **Commit 1's wasm test did not compile** (a duplicate constant) until `d830c58`. The build
        could not run the wasm row on unmodified code, and it said so. Verify then ran it there
        and it was RED, as it should be.
      - **Verify's first PR-head wasm run failed** on a stale library from a target dir shared
        with the baseline worktree. That is a known trap that struck again.
      - **The PR binary served a blank cached `build` output made by `main`**, because both
        report 0.8.0. The spec accepted this in Call 7. It closes when 0.8.1 bumps the version.
      - **What held:**
        - every negative control and mutation M-a to M-d flipped as predicted;
        - ink was counted with ImageMagick on both binaries;
        - released 0.7.1 and 0.8.0 were driven before the CHANGELOG claimed "0.5.0 through
          0.8.0";
        - the pinned hash held on all three OSes.
   2. **Does any template, constraint or decision need updating?** Name it, or say none. The
      candidate to weigh: a HANDOFF trap or design-template line saying "a test that can skip
      must fail where its coverage is required, not print; libtest hides stderr for passing
      tests, so a printed SKIP is invisible in CI." Say whether it is crustyimg-specific or
      general. **Propose it; do not apply it.**
   3. **Is there a follow-up spec to write before I forget?** Read STAGE-054's backlog first.
      Verify's cleanups were all done in the punch list, so file nothing for them. File a new
      entry only if the evidence shows real remaining work, and say why. "None" is a fine
      answer.
3. **Cost.** `cost.sessions` holds frame ($2.91, half of the shared SPEC-137/138 frame), design
   ($5.56), build ($2.26) and punchlist ($0.90).
   - **The build was measured before its final turns.** Re-measure it from its full transcript,
     and replace the number if it differs.
   - **Add one entry for verify**, plus one for your own `ship`.
   - Every transcript is under `~/.claude/projects/`. The `…` in each path is
     `-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg--claude-worktrees-`.

     | cycle | transcript | confirm with `grep -c` | reported |
     |---|---|---|---|
     | build (re-measure) | `…feat-spec-137-svg-font-family/5fc6f6d2-0512-44ab-8ca2-c912de8ae010.jsonl` | `BUILD prompt: SVG text in any font` | $2.26 |
     | verify (add) | `…verify-spec-137/0e634256-ba87-473e-864b-0a7fe5c31957.jsonl` | `VERIFY prompt: SVG text in any font` | $4.75 |

   - **Before measuring each transcript, confirm it by content** with the grep in the table.
     Read the model from each transcript's `.message.model`; don't assume it. The dollar figures
     above are the sessions' own estimates, for sanity-checking only. Do not copy them.
   - **Recompute `cost.totals` from every entry:**
     - `tokens_total` is the sum of the non-null totals;
     - `estimated_usd` is the sum of the non-null values;
     - `session_count` counts every entry.

     Run `just cost-audit` and confirm SPEC-137 passes.
4. **Mark ship `[x]`** in the timeline, with the merge (`71b4f38`, PR #217, 2026-10-29) and the
   total cost.
5. **Advance and archive.** Run `just advance-cycle SPEC-137 ship`, then
   `just archive-spec SPEC-137`. Then read STAGE-054's SPEC-137 entry back.
   - It should read `- [x] **SPEC-137** (merged 2026-10-29, PR #217) — …`. Fix it by hand if the
     script left it otherwise. Touch no other entry. SPEC-138 stays `(designed 2026-10-28)`.
   - Recount the `**Count:**` line with a grep you have just run, not by arithmetic. Today it
     reads `1 shipped / 2 active / 15 pending`. Expect `2 shipped / 1 active / 15 pending`, plus
     any entry you file in step 2.3.
6. **Check what the tooling sees.**
   - Run `just status` and `just backlog`. Confirm SPEC-137 shows as shipped.
   - Run `just roadmap`: STAGE-054 must stay **active**. ⛔ **Do not run Stage Ship**, and do not
     mark STAGE-054 shipped. SPEC-138 and other 0.8.1 items remain.
7. **Commit to `main` directly.** Ship bookkeeping does not go through a PR (AGENTS §13).
   - Commit:
     `git commit -s -m "ship(SPEC-137): SVG text in any font family renders in the bundled font — reflected, archived"`.
     **The DCO sign-off is mandatory.**
   - Push: `git fetch origin && git rebase origin/main && git push origin HEAD:main`.
   - Read back `git show --stat HEAD` and check it is what you meant.

## Guardrails
- ⛔ No version bump, no tag, no release and no CHANGELOG edit. Release prep runs later, in
  separate sessions.
- Do not edit code, tests, CI, DECs, HANDOFF or templates. If the reflection says one needs
  changing, propose the edit in your report; the orchestrator decides.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~40 exchanges.**

## Measure cost (yours and the transcripts above)
- Your own transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Identify it **by content, not recency**:
  `grep -l "SHIP prompt: SVG text in any font" ~/.claude/projects/<slug>/*.jsonl`. The transcript
  may store the em-dash escaped, so search for the text after it.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran, never at a flat rate: Sonnet $3/$15 per
  MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and cache_read ×0.10.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-137 ship: archived — <commit SHA on main>`. Then:
- the three reflection answers, one line each;
- any follow-up you filed, or "none" and why;
- the re-measured build entry, old → new;
- the two new cost entries (verify and ship), each with cycle, model, tokens and USD;
- `cost.totals` (tokens, USD, session count) and the `just cost-audit` result;
- the recounted `**Count:**` line, and confirmation that STAGE-054 is still active;
- any template, constraint or DEC change you propose;
- your measured cost.
