---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-038                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-010                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-26
shipped_at: null

# Re-homed 2026-07-26 from PROJ-008 STAGE-033, where it was framed but never
# started. No spec shipped under the old number. One change on the move:
# SPEC-107 left for STAGE-035, because it is launch-gating and this stage is
# not — the sequencing caveat the old file carried is now structural.

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    The friction that only shows up once someone actually uses the thing. PROJ-008 spent its
    length on the engine, the demo, and the launch story; this stage picks up the small,
    unglamorous defects and tooling gaps that no gate catches because no gate is looking —
    starting with a real usage report (shell completions were installed by hand, silently went
    stale at the surface freeze, and quietly stopped completing the flagship verb). Cheap work
    with a direct line to goal 1: a tool the maintainer actually enjoys using.
  delivers:
    - "Shell completions that install themselves via the package manager, complete file paths on every shell, and do not rot silently when the CLI surface changes"
    - "A whole-repo, all-time `lifetime-report` rollup (rule-based, deterministic, no LLM) alongside the existing report-daily / report-weekly tooling"
    - "An `activity:` front-matter field so a project's status says what KIND of work is live, not just active/shipped"
    - "The repo-tooling backlog's small standing annoyances closed out: PRs no longer running the 3-OS matrix twice, DCO sign-off made mechanical, an advisory release-binary size baseline, and the wasm-size banner telling the truth"
  explicitly_does_not:
    - "Touch the engine, the classifier, any codec, or the wasm/demo surface — no pixels change in this stage"
    - "Add, rename, or remove any of the 14 verbs frozen in STAGE-030 — completions describe the surface, they don't alter it"
    - "Do the LLM-free benchmark refresh or encoder threading — those are sequenced separately and gated on the code-review triage"
    - "Do launch coordination (the Show HN / r/rust go/no-go) — that is maintainer-blocked and lives on the launch board"
---

# STAGE-038: post-launch polish and repo housekeeping

## What This Stage Is

The catch-up stage for small things that are individually trivial and collectively the difference
between a tool that works and a tool that feels finished. None of them touch the engine:

1. **Shell completions** — a real user-visible bug, found the only way it could be found: by the
   maintainer trying to tab-complete a filename after `crustyimg web` and getting nothing.
2. **`just lifetime-report`** — port the whole-repo, all-time rollup tooling from
   `zany-animal-slots`. Repo-methodology work; complements `report-daily` / `report-weekly`.
3. **An `activity:` front-matter field** — make `status: active` say *what kind* of work is live.
4. **Four standing repo-tooling annoyances** — CI running every PR's 3-OS matrix twice, DCO
   sign-off that keeps being forgotten, no release-binary size baseline, and a `wasm-size` banner
   that lies about which build it measured.

The unifying thread is **maintenance-of-use, not construction**: nothing here adds capability.

**SPEC-107 is no longer here.** As STAGE-033 this stage carried the hostile/edge input pass with a
standing caveat that it was the one launch-gating item in a post-launch stage. The re-home resolved
that structurally: SPEC-107 now lives in **STAGE-035**, a launch-gating stage of its own, and this
stage is post-launch without exception.

## Why Now

- **The completions defect is live and user-visible today.** Anyone who installed completions
  before the STAGE-030 surface freeze has a script that silently stops completing `web` — the
  flagship verb the README and the post both lead with. That is a bad first five minutes for
  exactly the audience a launch would bring.
- **PROJ-008 shipped 2026-07-25.** This is the natural slot for cheap, low-risk work that
  doesn't want to be interleaved with an engine change.
- **Deliberately sequenced AFTER the `/code-review ultra` triage.** That triage has since
  happened: its findings are `docs/research/pr113-classifier-review-findings.md` and they became
  STAGE-034. Those get first claim on build capacity. This stage is what to pull when that queue
  is clear — or to interleave, since it shares no files with engine work.

## Success Criteria

- A `brew install` (and `cargo install`) user gets working completions **without knowing the
  `completions` subcommand exists**, and they regenerate on upgrade.
- Tab-completing a path argument offers files **on bash and zsh**, verified by driving a real
  shell — not by reading the generated script.
- A completion script generated against an older CLI surface **fails loudly or not at all**,
  never by silently offering nothing for a verb it doesn't know.
- `just lifetime-data` / `lifetime-report` / `lifetime-save` exist, are deterministic and
  LLM-free, and discover crustyimg's actual layout (`projects/PROJ-*/`, `decisions/DEC-*.md`,
  releases).
- `activity:` is accepted by `just validate`, documented in the project-brief template with a
  settled vocabulary, and backfilled on the active brief.
- All gates green on the full clean matrix (default / lean / `webp-lossy`, clippy each, fmt).

## Scope

### In scope
- `value_hint` across the CLI's path arguments; the Homebrew formula's completion install; the
  README / `--help` / CHANGELOG notes about completions and regeneration; a staleness signal.
- `scripts/lifetime-report.sh` + three `just` recipes + `reports/lifetime/`.
- The `activity:` field: template, `just validate`, optional `just status` surfacing, backfill.
- CI trigger de-duplication, a DCO pre-push hook, `just size` + a recorded binary-size baseline,
  and the `just wasm-size` banner label.

### Explicitly out of scope
- Engine / classifier / codec / wasm / demo changes as *feature* work; any change to the frozen
  verb set; the benchmark refresh; encoder threading; launch coordination.
- The hostile / edge input confirmation pass and everything fenced around it — that is STAGE-035
  now, including the browser-half split and the platform-aware RAW gating fence.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-106 (frame) — **shell completions: ship them, complete paths, don't rot silently.**
  Three separable defects, one spec. (a) Nothing installs completions — the brew formula ships
  none and the README never mentions the verb, so the maintainer hand-placed a script into a
  directory `omz update` can overwrite. (b) **Zero `ValueHint` in `src/`** → every path arg
  generates `_default`, not `_files`; zsh degrades gracefully (measured), **bash does not**
  (`complete -F` replaces the default file completion, leaving no fallback). (c) A pre-freeze
  script has no `web` case, so zsh's `#compdef` function claims the line, matches nothing, and
  offers **nothing** — while surviving verbs still complete, which is what made it read as
  "everything works except `web`." Detail + evidence in `docs/backlog.md`. Complexity **S**.
- [ ] (chore, may not need a spec) — **port the `lifetime-report` commands.** From
  `~/PSeven/experiments/zany-animal-slots/scripts/lifetime-report.sh` (~8.5 KB) + three recipes:
  `lifetime-data` (deterministic rollup), `lifetime-report` (same history wrapped as an LLM
  synthesis prompt), `lifetime-save` (→ `reports/lifetime/YYYY-MM-DD-HHMMSS.md`, timestamped to
  the second). Adapt discovery to crustyimg's layout; POSIX/bash-portable; no new dep. Queued
  item #1 in `docs/repo-tooling-backlog.md`. Complexity **S**.
- [ ] (chore, may not need a spec) — **add an `activity:` field to project front-matter.**
  Vocabulary is **not settled** — reconcile bragfile000's
  `requirements|design|build|test|blocked` against crustyimg's cycle model
  `frame|design|build|verify|ship`; keep it an open string with a documented suggested set, not
  a parse-rejecting enum. Queued item #2 in `docs/repo-tooling-backlog.md`. Complexity **S**.
- [ ] (chore) — **CI hygiene.** (a) `.github/workflows/ci.yml` triggers on both `push:` and
  `pull_request:` with no branch filter, so **every PR runs the full 3-OS matrix twice** —
  double cost, double the chance a network flake blocks a merge. (b) The `cargo-deny` action
  pulls a Docker Hub base image, so a required check can fail for reasons unrelated to this repo
  (it did, on PR #108: `dial tcp … i/o timeout`, while the same SHA passed in the duplicate run).
  Queued item #4. Complexity **S**.
- [ ] (chore) — **stop DCO sign-off recurring.** A verify-cycle commit has landed without `-s`
  three times, most recently blocking PR #108. No hook exists anywhere (verified: no
  `.githooks/`, no `.git/hooks/pre-push`). Mechanical fix: a local pre-push hook, and/or make
  `-s` explicit in the verify prompt's commit instruction. Queued item #5. Complexity **S**.
- [ ] (chore) — **track the release binary size (advisory baseline, not a gate).** SPEC-102 added
  +2,878,672 B (+22.4%) in one commit, visible only because that spec asked. No `just size`
  exists. Wants a recipe + a recorded baseline so drift shows in a diff. Keep the size check
  baseline-keyed and **separate** from any structural build-profile assertion
  ([[assert-the-build-profile-structurally-not-by-size]]). Queued item #6. Complexity **S**.
- [ ] (chore) — **`just wasm-size`'s banner mislabels a lean build.** [justfile:197](../../../justfile)
  calls `@just wasm-size` as a *nested* `just` invocation, which does not inherit `--set`, so it
  re-reads the default feature list: same artifact, same bytes, two different labels. Corrupts
  nothing today, but it is a live trap for whoever next re-measures SPEC-074's lean wasm
  baseline — exactly the number a mislabelled banner would poison. Queued item #7.
  Complexity **S**.
- [→] (chore) — **no CI leg runs `just wasm-test`.** **MOVED to STAGE-042** (2026-08-10), which is
  its real home: that stage's whole thesis is guards that do not run, and this is one. Left as a
  pointer rather than deleted so the trail from SPEC-112's verify is not lost.
- [ ] (chore) — **the README does not say `transform` skips the marker's semantics.** Found by
  SPEC-112's verify. `transform(bundled_toml, "png")` now runs the bundled recipe (SPEC-112),
  but the terminal `optimize` marker's semantics — the fast AVIF-aware decision, never-bigger,
  score — are *stripped*, not reproduced; `optimizeDetailed` is the decide-path counterpart, as
  designed. `README.md:34-36` says the TOML *runs*, which is true, so SPEC-112's AC-8 holds. But
  a JS consumer starting from `web` gets the downscale without the modernize, and nothing tells
  them. One or two sentences. Queued item #9. Complexity **S**.
- [ ] (chore) — **`gitignore_files_maybe/` is published to crates.io.** Found in the 0.7.0 pre-tag
  pass. `Cargo.toml:15`'s `exclude` lists `/decisions /docs /projects /reports /guidance /feedback
  /scripts /.github /.claude` but **not** `/gitignore_files_maybe`, so `crustyimg0.acorn` (an
  Acorn/SQLite editor document) and `crustyimg0.jpeg` (175×175) ship in the crate. Pre-existing,
  not a 0.7.0 regression — the `exclude` line is unchanged since `v0.6.0` and both files were in
  the v0.6.0 tree, so 0.5.0/0.6.0/0.7.0 all published them. Cosmetic (the whole package is 163
  files / 3.32 MB, far under the limit) and **a published version can never be re-published**, so
  this is a fix for the *next* cut, not a reason for a 0.7.1. Either add the dir to `exclude` or
  decide what it is and move it. Queued item #10. Complexity **S**.

**Count:** 0 shipped / 0 active / 1 spec + 8 chores pending (+1 moved to STAGE-042)

> **Sequencing.** Nothing here is launch-gating. STAGE-034 and STAGE-035 are; run them first.
> The old sequencing caveat is gone because the thing it protected against — a launch gate
> waiting on a housekeeping stage — was removed by moving SPEC-107 to STAGE-035.

## Design Notes

- **The completions diagnosis is recorded so the spec doesn't re-derive it.** Evidence gathered
  2026-07-26 against the installed 0.6.0 binary: the stale script offered `shrink` and
  `copy-metadata` (both gone at the freeze) and no `web`; 1,796 diff lines vs current; it also
  predated `build --watch/--check/--frozen`. Regenerating into a maintainer-owned fpath directory
  plus clearing `~/.zcompdump*` fixed it immediately.
- **⚠ Do not over-claim the `ValueHint` fix.** The zsh symptom was caused by (c), *not* (b) —
  once the script was current, `_default` completed files fine on zsh. The `value_hint` work is
  justified by **bash**, where the failure is real and total, and by not depending on a zstyle
  fallback we don't control. Verify on both shells; a fix verified only on zsh proves nothing
  about the shell that was actually broken. Per
  [[a-plausible-test-result-is-not-a-checked-one]], drive a real shell — reading the generated
  script is not evidence that TAB does anything.
- **The staleness class is the interesting finding, and it generalizes.** A `#compdef` function
  *suppresses* the shell's default behavior, so an out-of-date completion is strictly worse than
  no completion at all — it converts a working fallback into silence. STAGE-030's deliberate
  no-alias cutover guaranteed this for every existing install. Whatever the fix, the property
  worth designing for is: *the failure must be loud.*
- **This is a goal-1 stage.** All three items came from using the repo, not from a gate. Worth
  noting in the stage reflection: no test, lint, or CI leg could have surfaced any of them.

## Dependencies

### Depends on
- STAGE-030 (PROJ-008, shipped) — the frozen 14-verb surface the completions must describe.
- Nothing blocking. Sequenced after STAGE-034/035 by choice, not necessity.

### Enables
- Nothing blocks on this. It removes friction rather than unlocking capability.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
