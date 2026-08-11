---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-010
  status: active
  priority: critical
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-07-26
shipped_at: null

value:
  thesis: >
    Fix the launch-gating classifier regression (dithered/halftoned graphics promoted to
    lossy AVIF after resize), confirm hostile-input behavior on the native CLI and wasm
    builds, then deliver three carried-forward stages of code health, CLI surface, and
    housekeeping — so the Show HN / r/rust launch has a correct default path on every
    input the web verb touches, documented behavior on bad inputs, and a codebase that
    is legible, fast to build in dev, and ready for the scrutiny and contributions a
    launch brings.
  beneficiaries:
    - "Show HN / r/rust readers who try `crustyimg web <file>` on a dithered or halftoned graphic — a scan, a print artifact, an archival image — and get a correct result at the default `--max`"
    - "The maintainer, shipping and supporting the launch — fewer 'it made my file bigger' surprises to explain"
    - "First PR contributors landing in a codebase with a settled CLI surface and a cascade that is internally consistent"
    - "Users who tab-complete file paths on bash or zsh after a brew install"
  success_signals:
    - "A dithered/halftoned graphic that currently produces an 18.5x-larger lossy AVIF through the default `web` path instead produces a correct lossless (or smaller lossy) output — verified against the two committed boundary specimens"
    - "The classifier runs before the resize pipeline (or the entropy threshold is scale-aware), proved by re-running the re-derived negative control from the review findings — PHOTO_ENTROPY_STRONG = 5.5 must make a guard go red, which today it does not"
    - "Every input in the committed hostile corpus produces no hang, a clear user-facing message, and a documented exit code — on both native CLI and headless wasm"
    - "The strict-JSON escape_json tail from SPEC-097 ships, and every declined code-health candidate is recorded as declined with its reason"
    - "SPEC-092 `convert --to` is live; shell completions ship via Homebrew, complete file paths on bash and zsh, and signal staleness on surface changes"
  risks_to_thesis:
    - "The classifier fix is the real engineering unknown. If the correct fix is deeper than 'move classification before resize' or 'make entropy threshold scale-aware' — e.g. if the classifier needs a different metric, or the pipeline architecture makes pre-resize classification expensive — it could grow into multiple specs and expand the launch-gating timeline. Calibrated by the review findings, which narrow the fix to two concrete approaches."
    - "The hostile-input pass could find real defects (panic, hang, wrong exit code) that need triage and fixes — this is the purpose of running it, but unanticipated defects extend the stage. The committed corpus is designed to surface them cheaply."
    - "The carried-forward stages are individually cheap but collectively produce noise across many modules; the maintainer may decide some items are not worth the churn. STAGE-036 in particular is mostly a candidate list with no provenance in this repo, and is framed so that declining an item is a recorded outcome rather than a gap. The brief sequences each as optional per maintainer judgment."
    - "Three of the seven review findings brought into STAGE-034 (rule 6's dead code, the DOC_ENTROPY_MAX band, the Icon ordering) are only cheap if the narrow rule-4 gating fix wins. If the design spec chooses the pre-resize placement fix instead, they become separate work and the launch-gating stage grows."
---

# PROJ-010: Post-launch correctness and consolidation

## What This Project Is

The **correctness + launch-readiness wave** — scheduled immediately after PROJ-008 (WASM core + demo) and before the Show HN / r/rust launch. It fixes the one known engine regression that makes the flagship `web` verb produce a *worse* result on certain real inputs (dithered/halftoned graphics promoted to lossy AVIF at **18.5×** the input size after the resize pipeline flips their content class). It confirms that the CLI and wasm build handle hostile/edge input without hangs or panics. And it takes over three carried-forward stages from PROJ-008 — code health, CLI convenience, and repo housekeeping — that are individually cheap but collectively raise the codebase's readiness for the scrutiny a launch brings.

Two pre-launch stages (launch-gating), three post-launch stages (optional, per maintainer judgment). All sequenced so the gating work lands first and the optional work doesn't delay the Show HN.

## Why Now

- **The classifier regression is live on the default path, and a launch would amplify it.** `crustyimg web <dithered-graphic.png>` demonstrably hands a user a file that is both **18.5× larger than its input and visually degraded** (844,492 B out for a 45,527 B source, `larger_than_source: true`, SSIMULACRA2 69.2) — through the default path, with no flags. The input is an ordinary 1-bit halftone, a print/scan artifact rather than a contrived case, and at native size the same file passes through untouched. The trigger is the downscale ratio: any dithered or halftoned source whose long edge exceeds 2048 by more than ~20% is exposed by default. The post leads with `web`.

- **The blast radius is dithered and halftone graphics — not screenshots, not favicons.** The review's screenshot framing did not reproduce on magnitude: four substituted screenshots top out at entropy 1.14 / 0.80 / 2.04 / 2.37 and all stay lossless, and sub-129 px input hits the `Icon` rule. This matters operationally, not just for accuracy: **a screenshot-only fixture corpus would go green against the real defect.** Scope the fixtures to dithered and halftoned sources.

- **PROJ-008 is shipped and nothing else is in flight.** The wasm core, npm library, demo page, README, BENCHMARKS, CLI freeze, and launch-readiness infrastructure are done. The next thing to do before a launch is to fix what is broken and confirm what is assumed — not to add capability.

- **Three carried-forward stages were waiting on this framing decision.** STAGE-031/032/033 were written under PROJ-008, have spec-level detail, and share no engine files with the classifier fix. PROJ-008's own reflection recorded them as awaiting a home. They can be interleaved with the gating work or sequenced after it, at the maintainer's choice.

## Success Criteria

- The classifier does not promote dithered/halftoned graphics to `photograph` at any `--max` setting — verified by driving the two committed boundary specimens against a release build.
- The cascade is left internally consistent: rule 6 reachable or deleted, the `[4.0, 4.5)` contradiction band resolved, rule 5 reachable, and `--profile docs` doing something for promoted images.
- Every hostile/edge input in the committed corpus produces no hang, a clear message, and a documented exit code — on both native CLI and headless wasm.
- The strict-JSON `escape_json` tail ships; every declined code-health candidate is recorded as declined, with its reason.
- SPEC-092 `convert --to` is live; shell completions ship via Homebrew, complete file paths on bash/zsh, and signal staleness.
- The launch-readiness hostile-input blocker moves off "hold natively; confirm in the browser" to a stated, driven outcome.

## Scope

### In scope
- **Classifier regression fix** — two specs: (a) fix the classification-placement or scale-aware-entropy bug that lets `--max` flip the content class, and resolve the cascade contradictions the same change stands in; (b) evidence integrity — commit the two boundary specimens, re-establish six named guard sites with negative controls, correct DEC-047's false claims.
- **Hostile/edge input confirmation pass** (SPEC-107) — drive a committed corpus against native CLI and headless wasm; fix anything it finds; update launch-readiness board.
- **Code health** — the strict-JSON `escape_json` tail carried from PROJ-008 STAGE-031, plus triage of an explicitly-unsourced candidate list (clippy sweep, test-speed stratification, edition migration, `pulp`, `zlib-rs`). Triage means each is framed or recorded as declined; none is committed here.
- **CLI surface enhancement** — SPEC-092 `convert --to` verb and extra bundled recipes.
- **Shell completions** — SPEC-106: `ValueHint` on path args, Homebrew formula install, staleness signal, bash/zsh verification.
- **Repo tooling** — CI trigger dedup, DCO pre-push hook, `just size` + binary-size baseline, `just wasm-size` banner fix, `lifetime-report` port, `activity:` front-matter field.

### Explicitly out of scope
- New image formats, codecs, or engine capabilities — this wave fixes and confirms the shipped engine, it does not extend it.
- New backend/service/CDN — the no-service guardrail from PROJ-008 stands.
- The Show HN / r/rust go/no-go decision — that is a maintainer decision on human-hardware and timing grounds, tracked on `docs/launch-readiness.md`.
- LLM-free benchmark refresh — separately sequenced, gated on the code-review triage.
- Encoder threading — a probe, sequenced separately.
- The browser half of hostile-input pass (demo UI surfacing, mobile behavior) — folds into the maintainer's mobile device test.

## Stage Plan

- [x] STAGE-034 (**shipped 2026-07-28**) — **Classifier regression fix** (launch-gating). SPEC-108 the fix + cascade consistency (DEC-084, PR #121); SPEC-109 evidence integrity (PR #114). The 18.5× blow-up is fixed on `main`. New stage.
- [x] STAGE-035 (**shipped 2026-07-30**) — **Hostile/edge input confirmation pass** (launch-gating). SPEC-107 (PR #127, DEC-085), moved out of PROJ-008 STAGE-033 so a launch gate does not sit inside a post-launch stage. Driven at design rather than assumed: **no hang, no panic, no OOM on any input; nothing on release over 0.25 s** — and it found a live defect on the flagship `web` path (a truncated JPEG succeeded silently, exit 0, empty stderr), so "verification only" did not hold. Ships a committed 8-file hostile corpus, a native CLI harness, four closed wasm gaps, and the launch-board blocker closed with the browser half named and left with the maintainer. New stage.
- [ ] STAGE-036 (proposed) — **Engineering quality and code health** (post-launch). The continuation of PROJ-008 STAGE-031 (which shipped 097/098/099 and closed there): the `escape_json` tail plus an unsourced candidate list to triage.
- [ ] STAGE-037 (proposed) — **Post-launch CLI surface** (post-launch). SPEC-092 `convert --to` + social/archive recipes. Re-homed from PROJ-008 STAGE-032 by `git mv`, content unchanged.
- [ ] STAGE-038 (proposed) — **Post-launch polish and repo housekeeping** (post-launch). SPEC-106 completions + six CI/DCO/size/tooling chores. Re-homed from PROJ-008 STAGE-033 by `git mv`, minus SPEC-107.
- [x] STAGE-039 (**shipped 2026-08-09**) — **Shipped-verb correctness** (D-1/D-2 launch-gating, D-3 cheap). SPEC-110 `convert` orientation + sweep (PR #133, DEC-086); SPEC-111 `build` runs bundled recipes (PR #138, DEC-087); the `docs/data-model.md` chore, now pinned by `tests/docs_ops.rs`. **This closes the last launch-gating repo work.** Each item turned out larger than framed: D-1 was seven broken verbs rather than one, D-2 needed a design cycle the stage said it did not, and D-3's test caught two flaws in itself. New stage, added 2026-07-26 from re-verified exploration findings.
- [ ] STAGE-040 (**active**) — **Release readiness for 0.7.0.** The stage that makes PROJ-010's work reach a user: `v0.6.0` was tagged 2026-07-24 and every PROJ-010 fix landed after it, so the released CLI still has all four defects while the demo (which redeploys from `main`) does not. Two items: SPEC-112 (`wasm::transform` runs the bundled recipes — the README claims it does and it does not, driven), then the 0.7.0 cut. New stage, added 2026-08-09 at the STAGE-039 close-out; STAGE-039 is closed and cannot be reopened, so this is its continuation.

- [x] STAGE-040 (**shipped 2026-08-10**) — see the entry above, now closed: SPEC-112 merged (PR #144) and **0.7.0 is live on crates.io, Homebrew and Releases**, verified by driving the downloaded binary rather than by reading three green workflows.
- [ ] STAGE-041 (proposed, **added 2026-08-10**) — **Launch content and publication plan.** There is no post, no asset, no channel list and no schedule anywhere in this repo — verified by search. Scoped to *plan*, not draft: the narrative is the maintainer's voice.
- [ ] STAGE-042 (proposed, **added 2026-08-10**) — **Release-safety instruments.** Four of PROJ-010's five defects escaped the same way — an unenumerated cell of a matrix. Delivers a conformance matrix derived from the code's own lists, a release-lag signal, a wasm CI leg, and two `RELEASING.md` steps.
- [ ] STAGE-043 (proposed, **added 2026-08-10**) — **Pinned-path correctness.** Every PROJ-010 fix landed on the *decide* path; the **pinned** path kept its defects. `optimize x.jpg -o out.jpg` on an already-compressed source returns a file **2.02× larger, exit 0, empty stderr** — driven on the shipped 0.7.0 binary. Plus `build` swallowing the truncated-JPEG warning `apply` prints.

**Count:** 4 shipped / 0 active / 4 pending + 1 on hold

**Launch-gating: COMPLETE, and delivered.** STAGE-034 ✅, STAGE-035 ✅, STAGE-039 ✅ and STAGE-040 ✅ are shipped, and 0.7.0 is live on all three channels.

### Sequence from here (decided 2026-08-10)

**STAGE-043 → STAGE-041 → STAGE-042**, with the rest trimmed:

1. **STAGE-043 first**, and **before the launch post.** Not because it blocks the demo — the demo
   never takes the pinned path — but because the post will name `optimize`, and
   `optimize photo.jpg -o out.jpg` silently returning a doubled file is a top comment rather than
   a bug report. It is the same defect class as STAGE-034's 18.5× blow-up and, in one respect,
   worse: **the 18.5× case at least reported the size.** Ships as 0.7.1, or folds into 0.8.0
   alongside STAGE-042.
2. **STAGE-041 next.** The product is correct and nobody knows it exists; everything else is
   polish on a tool with no users.
3. **STAGE-042 after.** It protects the *next* release rather than this one, and its matrix design
   should absorb STAGE-043's root cause — cross entry points with **both modes** (decide and
   pinned), not just the default one.
4. **STAGE-036** now holds **two real items**: the `escape_json` tail, and — added 2026-08-10 —
   **decomposing `src/cli/optimize.rs`**, which is what `src/cli/mod.rs` was before SPEC-097 split
   it. Measured by *production* lines (excluding test modules): **1,716**, versus 1,107 for the
   next largest and 1,002 for `cli/mod.rs` *after* its split — so 1.55× the runner-up and 71%
   bigger than the file that was judged to need splitting. Ranking by total lines hides this.
   Follows SPEC-097's method (byte-identity proven by an independent oracle) and **must land after
   STAGE-043**, which changes behaviour in that same file. Its five unsourced candidates were
   triaged and **declined 2026-08-10**, text preserved in full with a per-item reason so any can
   be revived.
5. **STAGE-037** is **`on_hold`**, by its own long-standing criterion: pull on an adoption signal,
   not on the launch clock. Nothing has changed, so `proposed` was overstating it.
6. **STAGE-038** is triaged rather than flat — CI's duplicate-run job (every PR runs the 3-OS
   matrix twice) and SPEC-106 completions are the only items with real cost or real user impact.

**Still maintainer-only** on `docs/launch-readiness.md`: the device pass (the ~60 MP RAW preview
decode has never run on hardware), re-verifying the install **paths** at 0.7.0 (the binary itself
is confirmed), the post narrative, and the go/no-go.

### How the carried stages were re-homed (2026-07-26)

`PROJ-008/brief.md` and `docs/backlog.md` both recorded STAGE-031/032/033 as deliberately left in
place, awaiting the next project's thesis. That decision is now made, and the three were **not**
treated alike:

| PROJ-008 | Here | Mechanism |
|---|---|---|
| STAGE-031 | STAGE-036 | **Not moved.** STAGE-031 had three shipped specs (097/098/099, PRs #103/#102/#104, DEC-078/079) whose files live in PROJ-008's `specs/done/`; moving it would have relocated PROJ-008's shipped work and PR provenance into a project that has not started. It is now `shipped` there. STAGE-036 is its **continuation**, inheriting the one unframed tail item, the shelved-directive record, and the byte-identity oracle gate. |
| STAGE-032 | STAGE-037 | `git mv`, content unchanged. No spec had shipped under the old number. |
| STAGE-033 | STAGE-038 | `git mv`, minus SPEC-107 → STAGE-035. |

## Dependencies

### Depends on
- PROJ-008 (shipped 2026-07-25) — the CLI surface, wasm build, and classifier code this project fixes and confirms.
- The classifier review findings (`docs/research/pr113-classifier-review-findings.md`) — the re-derived boundary specimens and negative control design that define this project's first stage.

### Enables
- A launch (Show HN / r/rust) that has a correct default path, documented hostile-input behavior, and a legible codebase ready for contributors.
- Future PROJ-011+ work (Wave 4 manifest, Wave 5 geometry, post-1.0 beta items) on a clean, maintained foundation.

## Project-Level Reflection

*Filled in when status moves to shipped.*
