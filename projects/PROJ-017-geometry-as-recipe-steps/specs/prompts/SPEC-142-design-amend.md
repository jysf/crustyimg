# SPEC-142 — DESIGN AMEND prompt: parity is bounded (Rule B); write the DEC and unblock

Cycle: **design**, amend only. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-142-design-amend -- --model opus
```

SPEC-142's design (`8e58a3c`) is complete except for one ruling, which it stopped for, correctly.
At the page's own sizes, native `apply` and wasm `transform` differ by 1/255 on 1–2 pixels after a
resize. The difference is deterministic, sits on a rounding boundary in the f32 linear-light
resize, and isn't caused by the decoder. **The maintainer has now ruled.** Your job is small:
- write the DEC;
- finalise the three places the design said would change;
- file the two findings;
- clear the block;
- advance the spec to build.

Don't redesign anything else.

## The ruling (2026-10-30 — settled, do not re-open)
**(B) Bounded, through a DEC.** It's Rule B as the design's "Design finding" section words it:
- **Dimensions are always exact.**
- **A row whose recipe has no resampling step stays exact.** These are rows 1, 3, 9 and 11 of the
  row table. Confirm that list against the table: any row with `resize` resamples.
- **A row that resamples may differ by at most 1/255 per channel, on at most K pixels.**
  - K is set from the build's first x86_64 CI run.
  - K is capped at **0.01 % of the row's pixels** (77 at 1080×723).
  - Above the cap, the build stops and reports BLOCKED. It doesn't raise the cap.
- The driven negative control (opacity 0.6 → 0.61: 3/255 on lossless rows, 13–22/255 on lossy
  ones) must still fail under the bound. Say so in the DEC and in FT-3.

Option (A), an exact engine fix, was not chosen **now**, because PROJ-015's working-type change
(`F32x4` → `U16x4`, PROJ-015 brief) rewrites this resize path anyway. Exactness stays a target and
is filed, not forgotten (see "File the findings").

## What to do
1. **Step 0.** Run `git fetch origin && git rebase origin/main`. The spec must be at
   `cycle: design` with `blocked: true`, and `8e58a3c` must be in your history. If either check
   fails, stop and report.
2. **Read:**
   - the spec's `## Design finding: blocked on a ruling` (about :413) and the row table (about
     :670);
   - AC-3, FT-3 and the "Comparator" note;
   - DEC-059, DEC-008, DEC-092, DEC-064 and `decisions/_template.md`;
   - `projects/PROJ-015-*/brief.md` (the working-type risk);
   - HANDOFF.md's "Traps this wave paid for" in PROJ-011, which binds PROJ-017.
3. **Write the DEC.** Run `ls decisions/` after your last rebase and take the next free id (DEC-109
   unless it's gone, since other sessions add DECs in parallel). Make the title the rule, e.g.
   "Native and wasm agree on dimensions exactly and on resampled pixels within 1/255".
   - Context: the measured table, the probable cause, and that the cause is stated as a prior,
     not proven.
   - Decision: Rule B above, with K's provenance, the cap, and the exact rows.
   - Alternatives: (A) the exact engine fix, and why not now; a looser bound, and why not.
   - Consequences, including that SPEC-141's native goldens are unaffected and that PROJ-015's
     working-type change must re-measure this bound.
   - Validation: what would make the DEC wrong, e.g. a difference above 1/255, or K above the cap
     on any target.
   - `affected_scope` uses the **block-list** form: `tests/ddphotos_parity.mjs`,
     `integrations/ddphotos/edit/**`, `src/wasm.rs`. Choose and justify the exact list.
   - Run `./scripts/decisions-audit.sh` and report any new structural error. Scope-overlap
     warnings are expected.
4. **Amend the spec. Change only these:**
   - **AC-3**, **FT-3** and the **Comparator** note now state Rule B. Name K as a value the build
     sets from the first x86 run and records in Build Completion and in the DEC.
   - FT-3 needs rows that can fail on each limb:
     - a 2/255 difference on one pixel is red;
     - K+1 pixels at 1/255 is red;
     - any difference on a no-resample row is red;
     - a dimension difference is red before any pixel comparison.

     These are comparator self-tests, so drive them through the harness's `--self-test` (AC-6).
     Add them to the mutation table with their expected red and green, per trap 12.
   - Add the DEC to `references`.
   - Rewrite the design's x86 outcomes for Rule B:
     - (i) x86 agrees: assert JPEG bytes, and K is the max over the targets;
     - (ii) JPEG bytes differ but the pixels are within the bound: assert pixels only;
     - anything over the bound is BLOCKED.
   - Replace `blocked: true` with `blocked: false`. Keep `## Design finding` as the record, with
     one line added: "Ruled (B), 2026-10-30, DEC-NNN."
   - Add a `### Maintainer rulings (2026-10-30, continued)` line with ruling 4: Rule B.
5. **File the findings** (AGENTS §10):
   - **The exactness target (A)** goes in `guidance/ideas.yaml` as `status: proposed`, with
     `likely_home` set to PROJ-015's working-type stage (not yet framed), and a `gate` saying
     that stage must re-measure native-vs-wasm resize parity and re-bless SPEC-141's goldens if
     native pixels move. Run `just ideas proposed` and confirm it's listed. Also add one line to
     PROJ-015's brief under the working-type risk, pointing at the DEC.
   - **wasm `info()` reports the stored size of an EXIF-rotated JPEG** (1600×1071 for a
     1071×1600 output), so `planEdit` predicts landscape. File it as a new `- [ ] (not yet
     written)` entry at the end of STAGE-060's backlog, or as a note on the crop item, whichever
     the stage's structure suits. Re-derive the `**Count:**` line with a grep you've just run. Say
     whether it's an `info()` bug (src, its own spec) or a page workaround (the crop spec).
6. **Land it.**
   - Run `just advance-cycle SPEC-142 build`.
   - Change the STAGE-060 marker to `(designed 2026-10-30)`. Mark design `[x]` in the timeline
     with a two-line summary that names the DEC, and add `prompts/SPEC-142-build.md` to the build
     line.
   - Run `git commit -s` (**the DCO sign-off is mandatory**), with the message
     `docs(SPEC-142): rule parity bounded (DEC-NNN); unblock for build`.
   - Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
   - Read back `git show --stat HEAD`.

## Guardrails
- ⛔ No code. Don't edit `integrations/ddphotos/`, `ci.yml`, `justfile`, or `src/`. No version
  bump, no tag.
- You don't need to re-drive the measurement. The design's table is the evidence. If you do
  re-drive any of it, use a fresh `CARGO_TARGET_DIR` and delete it afterwards.
- A piped command reports the pipe's exit code. `rtk` can truncate `grep` output, so use
  `/usr/bin/grep` for anything you count.
- **Budget ~25 exchanges.**

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content,
not recency**: `grep -l "SPEC-142 — DESIGN AMEND prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at the model that actually ran: Opus $5/$25 per MTok, with cache_creation
  at ×1.25 and cache_read at ×0.10.
- Append a `design-amend` entry to `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-142 design amend: DONE | BLOCKED`, then the commit SHA on `main` and the DEC id.
Then, short:
- the DEC's rule in one line;
- the new FT-3 rows and their mutation entries;
- the x86 outcomes as amended;
- where each finding was filed;
- anything the build prompt must carry;
- your measured cost.
