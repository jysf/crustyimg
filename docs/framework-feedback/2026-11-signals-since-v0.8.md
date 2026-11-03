# Framework signals since the last harvest — for the template author

Written 2026-11-02, from crustyimg (claude-only variant) at v0.8.2. Companion to
`signals-harvest.md` (2026-07-04, v0.1.0), `process-feedback.md` and
`tooling-that-fails-silent.md` (2026-08-14). It covers what crustyimg learned about the
**template** between 2026-08-14 and now: PROJ-010 STAGE-045 through PROJ-011 SPEC-143.

This is about the spec-driven template, not about crustyimg. Every signal below is generic;
crustyimg-specific traps (image codecs, resvg, wasm size) are left out on purpose. The
evidence column names the file in this repo that holds the full account.

Same conventions as `signals-harvest.md`: `lesson` = a rule earned by recurring evidence,
`process-debt` = framework or tooling friction to fix, `risk` = a standing hazard.
`status: open` needs a decision or codification.

```yaml
signals:
  # ─────────────── PROCESS-DEBT (template files that are wrong or missing) ───────────────
  - id: per-spec-cost-ledger-and-method
    type: process-debt
    summary: Upstream the cost-capture enforcement AND the corrected token-counting method. Claude Code writes one JSONL line per content block, so summing lines over-counts ~2x and taking the first line's output under-counts ~11x.
    evidence: {files: [docs/cost-tracking.md, "projects/PROJ-011*/HANDOFF.md trap 3"], measured: "32 records corrected in place, 10 unrecoverable"}
    status: open
    notes: |
      Correct method: dedupe by `.message.id`, take input/cache_creation/cache_read from the
      group, take the MAX output_tokens. Transcripts live in one project dir PER WORKTREE and a
      subagent's is under `<session-id>/subagents/`; searching only the main dir wrongly called
      22 records unrecoverable (12 were in sibling dirs). `docs/cost-tracking.md` lists the files
      to port verbatim (cost-audit.sh, _lib.sh helpers, status.sh section, ci cost-data job,
      the `cost-captured-per-cycle` constraint, spec.md cost block). Maintainer ruled on
      2026-08-18 that the ledger belongs to the template, not this repo.

  - id: per-spec-cost-breakdown-and-outcome
    type: process-debt
    summary: Add a per-spec ledger (cost by cycle, description, outcome). Outcome is prose today, so it needs an `outcome:` front-matter block written at ship.
    evidence: {files: ["STAGE-047 design (parked)"], measured: "verify = 37% of build cost across PROJ-010; S-sized specs $15-$76 vs M $38-$142"}
    status: open
    notes: Two findings stand alone. Verify costs 37% of build, and complexity barely predicts cost (the S and M ranges overlap), so the template's S/M/L estimate field predicts little.

  - id: scripts-fail-silent-round-two
    type: process-debt
    summary: Three more template scripts succeed silently while doing nothing. Same family as tooling-that-fails-silent.md.
    evidence: {files: [scripts/_lib.sh, scripts/decisions-audit.sh, decisions/_template.md, "projects/PROJ-011*/HANDOFF.md traps 7 and 8"]}
    status: open
    notes: |
      (a) `find_spec()` globs `specs/prompts/*.md` as well as the spec, so `just advance-cycle`
      can edit a prompt file and print "Advanced ..." while the real spec never moves. Found by
      two sub-agents independently (SPEC-104).
      (b) `decisions-audit.sh` silently drops an inline-array `affected_scope`, so that DEC is
      invisible to `--changed` forever. The root cause is `decisions/_template.md`, whose example
      uses the unreadable form. Fix the template example to the block-list form.
      (c) `just backlog` (even `--all`) ignores a project whose status is still `proposed`, so
      54 filed items were invisible to the framework.

  - id: spec-template-file-inventory
    type: process-debt
    summary: spec.md's Build Completion has no file-inventory field, so every build added one by hand and one still came up a file short. Ask for it, built from `git diff --name-only` rather than recall.
    evidence: {commits: [c0d5e9b], specs: [SPEC-124, SPEC-125]}
    status: addressed-locally
    notes: Fixed in this repo's projects/_templates/spec.md. Port it.

  - id: dco-signoff-recurs-in-verify
    type: process-debt
    summary: Verify-cycle commits land without `-s` (4th recurrence). The house convention is not remembered; make `-s` explicit in the verify prompt's commit instruction or add a pre-push hook.
    evidence: {files: ["docs/repo-tooling-backlog.md item 5", "HANDOFF trap 5"]}
    status: open

  - id: local-check-weaker-than-ci
    type: process-debt
    summary: The documented local gate (`just check`) skips CI-only legs, so a clean local matrix shipped a red PR. The agent instructions should list exactly what CI runs that the local gate does not.
    evidence: {specs: [SPEC-128], files: ["HANDOFF trap 5"]}
    status: open
    notes: Here the missing leg was a bundle-size gate, but any project with an extra CI job has the same gap. Related and still open from the first harvest, verify-runs-lean-no-default-features-build.

  # ─────────────── LESSONS (verify and build discipline) ───────────────
  - id: a-document-is-not-a-backlog
    type: lesson
    summary: A decision recorded in a document no command reads does not exist to the framework. If no stage backlog is the right home, it is a proposal, not a decision.
    evidence: {files: ["AGENTS.md backlog section"], measured: "4 cases in one week, longest 13 days lost"}
    status: open
    notes: Cases were 358 lines of research in docs/backlog.md, 4 audit findings filed nowhere (lost twice), 3 items in feedback/ plus a verify readout, and a decided feature in a triage doc. Worth a line in the template AGENTS.md and a lint that lists documents nothing parses.

  - id: measure-the-scenario-not-the-control
    type: lesson
    summary: When a claim rests on a driven table, the row carrying the claim must use the feature the claim is about. An AC naming two things needs a test per thing.
    evidence: {specs: [SPEC-127, SPEC-128], n: 3, pattern: same-outcome, files: ["HANDOFF traps 1 and 2"]}
    status: open
    notes: Free at design time. Count the nouns in each AC against the tests listed.

  - id: check-removed-as-redundant-needs-reproof
    type: lesson
    summary: When a check moves across a type boundary, add a test row just past the boundary of the type the code actually reads. The build's negative controls cannot see this, because each reverts a check that still exists.
    evidence: {specs: [SPEC-131], files: ["HANDOFF trap 9"]}
    status: open
    notes: A schema checked an f64, the operations read an f32, and a value of 1e-50 passed validation and read as 0.0. Caught only by the retroactive verify's own probes.

  - id: derived-value-needs-a-differing-input
    type: lesson
    summary: Any build or fix that adds or relies on a derived value (a count, a resolved list, formatted output) asks for one test row where the derived and naive values differ, plus a mutation that swaps them.
    evidence: {specs: [SPEC-133, SPEC-134, SPEC-135], n: 3, pattern: same-outcome, files: ["HANDOFF trap 10"]}
    status: open
    notes: Each case cost a punch list and a re-verify. Printed output counts as derived, because every test value happened to be exact in the narrower type.

  - id: a-skippable-test-must-fail-where-required
    type: lesson
    summary: A test that can skip must fail when an environment variable says its coverage is required, and CI sets that variable on the step that must prove it ran. A printed SKIP line is invisible in CI.
    evidence: {specs: [SPEC-137], files: ["HANDOFF trap 11"]}
    status: open
    notes: libtest hides stderr for passing tests. A design never asks for a log line without saying how it reaches the log.

  - id: mutation-rows-name-location-and-expected-red-set
    type: lesson
    summary: A design names the code location for each mutation and lists every test expected red, including tests on the other path (plain vs watch, native vs wasm). A red row is red for its reason only if its failure message says so.
    evidence: {specs: [SPEC-138, SPEC-143], n: 2, pattern: same-outcome, files: ["HANDOFF traps 12 and 13"]}
    status: open
    notes: Two rows in SPEC-143 (FT-11, FT-12) were red for the wrong reason, and the build recorded one as "returned Ok", which was false. A golden needs at least one mutation that goes red on a hash mismatch, not on a refusal or exit code.

  - id: fix-control-is-the-old-code-restored
    type: lesson
    summary: The control for a fix is the old code put back verbatim, not a new mutation. A test for a fixed path calls the production function.
    evidence: {specs: [SPEC-141], files: ["HANDOFF trap 14"]}
    status: open
    notes: The first punch list's new test drove a closure that never called the fixed function, so all 9 tests stayed green with the bug restored. It cost a second punch list and a second re-verify. Also, a bless or golden guard's design should add a new case by the README alone and record where it first goes green.

  - id: ci-step-assign-then-validate
    type: lesson
    summary: A CI step that branches on a command's output assigns it first, then validates it. Under `bash -e` a failing command substitution inside `[ ... ]` does not stop the step. A summary line counts every failure kind.
    evidence: {specs: [SPEC-142], files: ["HANDOFF trap 15"]}
    status: open
    notes: A crashing path classifier reported docs-only and every heavy job skipped green. Design should list the command crashing, and printing junk, as mutation rows. Before moving code, grep its downstream readers.

  - id: verify-before-you-commit-read-back
    type: lesson
    summary: After a scripted edit, read back what was written and diff it against what was intended. Checking that the edit succeeded is not checking that it was right.
    evidence: {files: ["HANDOFF trap 4"], measured: "a lazy regex scattered 20 corrections onto the wrong YAML entries and reached main"}
    status: open
    notes: Since caught a phantom backlog bullet, an invented DEC filename and an invented token field.

  # ─────────────── RISK (standing hazards the template encourages) ───────────────
  - id: shared-target-dir-across-worktrees
    type: risk
    summary: The template encourages worktrees and parallel sessions, but a build target dir shared across worktrees serves a stale library even sequentially, and concurrent differently-featured builds corrupt it. Use a fresh CARGO_TARGET_DIR (or equivalent) per worktree and per feature leg.
    evidence: {files: ["auto-memory: a-target-dir-shared-across-worktrees-serves-a-stale-library", "concurrent-differently-featured-builds-corrupt-a-shared-target-dir"]}
    status: watch
    notes: Language-specific in its detail (Cargo), generic in its shape. A stale incremental build is a false green, so shared-code changes need a clean full-matrix build.

  - id: a-green-gate-on-one-os-is-not-the-matrix
    type: risk
    summary: Verify reads the CI legs, not the headline status. A PR can merge on 16/16 green and a retroactive verify still finds a defect (SPEC-129).
    evidence: {specs: [SPEC-129]}
    status: watch
    notes: Draft PRs gating merge has worked. SPEC-131 merged before verify and needed a retroactive pass.
```
