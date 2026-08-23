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
    - "AMENDED 2026-08-23 — every declined code-health candidate is recorded as declined with its reason. The original signal also required the strict-JSON escape_json tail from SPEC-097 to ship; that clause is WITHDRAWN, and the work is retained as a backlog item rather than a launch-readiness criterion. Reason, checked rather than assumed: RFC 8259 s7 requires escaping only quote, reverse solidus and U+0000-U+001F. 0x7F and the C1 range (U+0080-U+009F) are LEGAL UNESCAPED, so src/cli/report.rs:116 is already spec-compliant and the item is hardening, not a correctness fix. It was labelled a correctness issue, and that label is what made it look like a release blocker."
    - "AMENDED 2026-08-23 — shell completions ship via Homebrew, complete file paths on bash and zsh, and signal staleness on surface changes. The original signal also required SPEC-092 (`convert --to` rename + social/archive recipes); SPEC-092 was CANCELLED 2026-08-23 and that half is withdrawn. A signal that binds a project to work it has decided not to do is a signal that guarantees the project cannot close — amend it or cancel the work, not neither."
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
- ~~SPEC-092 `convert --to` is live~~ (**cancelled 2026-08-23** — see the amended signal above); shell completions ship via Homebrew, complete file paths on bash/zsh, and signal staleness.
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
- [ ] STAGE-044 (proposed, **added 2026-08-10**) — **The `meta` lane cannot emit a broken manifest.** Driven by a spike against 0.7.0: `meta set --artist` takes a file whose Content Credentials validate `Valid` and emits one a validator reports `Invalid` / `assertion.dataHash.mismatch` — manifest fully intact, hash broken. The pixel lane drops credentials cleanly and `meta strip` drops them cleanly *by accident*; `meta set` keeps and breaks them. A bug fix, explicitly **not** the C2PA feature work.
- [ ] STAGE-043 (proposed, **added 2026-08-10**) — **Pinned-path correctness.** Every PROJ-010 fix landed on the *decide* path; the **pinned** path kept its defects. `optimize x.jpg -o out.jpg` on an already-compressed source returns a file **2.02× larger, exit 0, empty stderr** — driven on the shipped 0.7.0 binary. Plus `build` swallowing the truncated-JPEG warning `apply` prints.
- [ ] STAGE-045 (proposed, **added 2026-08-11**) — **Adopted-source-format integrity on the decide path.** The mirror image of 043: `Image::source_format()` is an *adopted label* for SVG (`Png`), HEIC (`Png`) and RAW (`Jpeg`), and the auto-decision reads it as the container on disk — so `optimize` can pass the **source container through verbatim** while reporting a format it never produced. `cat logo.svg | crustyimg optimize - --out-dir out/` writes **`out/stdin.jpg` containing XML**, described as a PNG. Driven at `08b367d` with committed fixtures; reaches `optimize`, `web`, `apply --recipe web` and `build` through one seam. Found while building SPEC-113, which hit the same adopted-label problem on the pinned side.

**Count:** 4 shipped / 0 active / 6 pending + 1 on hold

**Launch-gating: COMPLETE, and delivered.** STAGE-034 ✅, STAGE-035 ✅, STAGE-039 ✅ and STAGE-040 ✅ are shipped, and 0.7.0 is live on all three channels.

### Sequence from here (decided 2026-08-10)

**STAGE-043 + STAGE-045 + STAGE-044 → STAGE-041 → STAGE-042**, with the rest trimmed
(STAGE-045 added to the wave 2026-08-11):

0. **STAGE-044 alongside 043**, and in the same release. Both are shipped-verb correctness on lanes
   PROJ-010 never swept — 043 the *pinned* path, 044 the *metadata* lane — and both are small. 044
   is the more embarrassing of the two if found by someone else: emitting a manifest a validator
   reads as **tampering** attributes a forgery to the file's signer, which is a worse failure than
   simply dropping the credentials. It is also more niche than 043's `optimize`, so it is not a hard
   launch gate on its own — but there is no reason to split the wave.
1. **STAGE-043 first**, and **before the launch post.** Not because it blocks the demo — the demo
   never takes the pinned path — but because the post will name `optimize`, and
   `optimize photo.jpg -o out.jpg` silently returning a doubled file is a top comment rather than
   a bug report. It is the same defect class as STAGE-034's 18.5× blow-up and, in one respect,
   worse: **the 18.5× case at least reported the size.** Ships as 0.7.1, or folds into 0.8.0
   alongside STAGE-042.
1b. **STAGE-045 with 043, not after it.** Both change the same two files and both answer the same
   question — *when are the raw source bytes a valid output?* SPEC-113 introduces
   `pipeline_altered_source` as the shared answer; SPEC-115 adds its second half. Landing them far
   apart invites a divergent second copy of the judgement. Same release, and for the same
   launch-post reason: "it wrote my SVG into a `.jpg`" is a top comment, not a bug report.
2. **STAGE-041 next.** The product is correct and nobody knows it exists; everything else is
   polish on a tool with no users.
3. **STAGE-042 after.** It protects the *next* release rather than this one, and its matrix design
   should absorb STAGE-043's root cause — cross entry points with **both modes** (decide and
   pinned), not just the default one — **plus STAGE-045's third axis, input family**: every one of
   SVG/HEIC/RAW was tested only in the mode that hid the defect.
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

### Amendment (2026-08-15): two STAGE-042 items pulled forward, ahead of STAGE-041

The sequence above still holds for the bulk of the work — **STAGE-041 precedes the rest of
STAGE-042**. Two of 042's items were taken early, deliberately, and this records why so the
repo does not tell two stories:

1. **The `npm publish` guard** (chore, done). The unguarded path was *shorter than the guarded
   one*: the chain stopped at `wasm-npm-smoke`, so the real publish was `cd pkg && npm publish`
   — no build, no size profile, no smoke test — against a gitignored `pkg/` that nothing ties to
   the current checkout. npm publishes are effectively irreversible, and this was raised
   immediately before the 0.7.0 publish. **Sequencing a guard behind the launch content it is
   meant to protect gets the order backwards.**
2. **SPEC-118, the conformance matrix** (framed, not built). Framing it early costs nothing and
   blocks nothing; building it can still wait for 041. It was framed now because the evidence for
   its design — SPEC-111, SPEC-112 and SPEC-115's unenumerated cells — was fresh from shipping
   STAGE-043/044/045, and would have to be reconstructed later.

**STAGE-042 is therefore `active` with work in it while STAGE-041 has not started.** That is a
deviation from the declared order, taken knowingly rather than drifted into.

**What it cost, which is the more useful record.** Running SPEC-116, SPEC-117 and SPEC-118 in
flight across three unmerged branches broke an assumption the tooling makes: `next_id` scans only
the working tree, so with 116 and 117 sitting in an open PR it minted **SPEC-116 a second time**.
Stage counts also had to be hand-maintained across PRs all session. The framework assumes work is
**serial and merged before the next begins**; it does not currently support the parallelism used
here. Filed as a STAGE-042 item, and it belongs to the close-out discussion below — the same root
as the multi-session concurrency question, arriving from a different direction.

### Amendment (2026-08-15b): STAGE-046 added, and it precedes STAGE-041

Three exploration sessions on 2026-08-15 drove four defects on **shipped** verbs, including the
flagship `web`. They are recorded with their measurements in `docs/backlog.md` and homed on
**STAGE-046 — output fidelity on shipped verbs**:

- animated input is silently flattened, and `lint` recommends the command that does it;
- ops widen to RGBA and never narrow back (`+12.4%` bytes for a channel carrying no information);
- the same call truncates 16-bit input to 8-bit;
- `resize` resamples in sRGB rather than linear light, with no premultiplied alpha (the alpha half
  was filed unconfirmed and was confirmed by a repo-wide grep the same day).

**STAGE-046 precedes STAGE-041** — maintainer decision, 2026-08-15. The reasoning is not that the
defects are new (they shipped in 0.7.0 and are not regressions) but that **STAGE-041 publishes the
claim they contradict.** The launch content promotes quality-per-byte. Putting that claim in public
while the flagship verb measurably degrades its input is the risk the ordering avoids.

This is the second time in one week that the declared order has been amended, both times for the
same reason: **a guard or a correction sequenced behind the launch it protects is backwards.**
Amendment 2026-08-15 (above) says it in those words about the npm guard. That is now a pattern
worth naming at close-out rather than a one-off.

**What this does NOT change:** STAGE-041 remains the next content-facing work, and its two
reconciliation items — stale `BENCHMARKS.md` numbers, and install one-liners undriven since 0.5.0
— are untouched and still owed.

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

**Written 2026-08-23.** Five stages shipped, **18 specs**, two releases (0.7.0 and 0.7.1),
0.7.1 live on all three channels.

### Did we deliver the outcome?

**Yes, and the thesis held — but the outcome that mattered was not the one framed.** The brief
promised a correct default path on every input `web` touches, documented hostile-input behaviour,
and a codebase ready for scrutiny. All three landed. **What was not anticipated is that the wave
would spend most of its cost on defects nobody knew existed at framing** — silent frame loss,
silent depth halving, machine-dependent AVIF output, resampling in the wrong colour space. The
launch-gating classifier fix that motivated the project is one of eighteen specs.

### How many specs did it take?

**18 shipped.** The brief did not predict a number, which in hindsight is why nothing flagged that
the project had stopped being a bounded wave — it accumulated **61 open items across 8 stages**
before anyone triaged it as a set.

### What changed between starting and shipping?

**The project stopped being about the launch and became the correctness lane**, and nobody decided
that. It happened because a defect found mid-wave has to go *somewhere*, and the active project is
where `just backlog` can see it. ⚠ **That is a structural pull, not a discipline failure**, and it
will recur: the framework has no home for work that is real but does not share the active project's
thesis. PROJ-011 and PROJ-012 were created partly to give that pull somewhere better to go.

### Lessons that should update AGENTS.md, templates, or constraints

**Already applied:**
- ⚡ **AGENTS §10 — a decision that outlives its session gets a `- [ ]` where `just backlog` reads
  it, or it is not decided.** Earned four times in one week, most sharply by a decided `.cube` LUT
  feature invisible for 13 days in a file no command reads.
- ⚡ **AGENTS §3 — a success signal naming specific work binds the project to it.** Cancel the work
  *or* amend the signal; doing neither is how a project becomes immortal by accident. PROJ-010 was
  held open by SPEC-092, which nobody intended to build.
- **AGENTS §15** gained the negative-control rules: one revert per independent condition, and the
  evidence is the behavioural flip, not a binary hash.
- **`projects/_templates/spec.md`** now requires a file inventory built from `git diff --name-only`.
- **`AGENTS.md` §1's "Active project" line** was stale for nine projects and now points at
  `just status` instead of naming one.

**Still unapplied, and the most valuable of them:**
- ⚠ **A cycle's cost block structurally under-reports** — a cycle cannot count the messages that
  write it. Four cycles in one wave, all under, by 3–7%. `cost-snippet.md` warns about *premature*
  readings; it does not say the residual is unavoidable and the orchestrator must re-derive.
- ⚠ **Reporting and gating scripts should span projects; authoring scripts should not.** Nobody had
  drawn that line, and activating PROJ-011 silently made `cost-audit` vacuous — it now passes
  having checked **zero** specs, with a message identical to the one it prints after checking 18.

### Should any spec-level reflections be promoted?

**Three, and they are the ones this project actually paid for:**

1. ⚡ **Verify is the cheapest and most valuable cycle — five waves running.** 11–28% of build cost,
   and **every substantive defect this project found came from a verify pass or a punch list, never
   from a build.** The last wave's eight punch-list items contained **zero code defects** — all were
   records claiming more than had been measured, including two decision records that contradicted
   themselves in their own text. **The argument is for shorter builds and more review**, and it has
   five waves of evidence.
2. ⚡ **A test that cannot fail is worse than a missing one**, and this project shipped two before
   learning to drive the control. The generalisation that took longest to see: **a green whose
   control was never verified to apply is not evidence.** It recurred at the end in a new costume —
   `cost-audit` passing having checked nothing.
3. **Drive it before you file it.** Every finding filed late in this project carries a reproduction
   that was re-run, and **two of eight external findings turned out sharper than reported** while
   **three review batches each led with a flagship recommendation that did not survive contact**.
   The provenance of a finding predicts its quality better than its source does.

### What did we defer to the next project?

- **PROJ-011** took the invocation-consistency defects and the recipe-reach gap.
- **PROJ-012** took animated output and ICC transforms — the two rules `lint` can name but not fix.
- **STAGE-041** was carried to PROJ-011 for continuity; ⚠ **it does not share that project's
  thesis** and must not be counted toward it.
- **~24 items remain here** — see `TRIAGE-2026-08-23.md`. ⚠ **PROJ-010 does not close empty**, and
  pretending otherwise would be the same dishonesty as a success signal pointing at cancelled work.


### Parked for the close-out discussion (added 2026-08-14)

**Multi-session concurrency without ownership.** Raised deliberately here rather than
resolved in-flight, because it is a process question, not a spec.

STAGE-043's and STAGE-045's work was done across overlapping sessions — at one point two
were live in the **primary checkout simultaneously** (2026-08-12 05:06–05:31), which
AGENTS §16.5 forbids in as many words. The implementation was left uncommitted and swept
into a WIP commit by an agent that had not authored it and said so in the commit message
(*"Unexplained and still owed: why tests/input_raw.rs and tests/common/mod.rs are
modified"*). That open question then crossed a session boundary and was closed with a
plausible rationale instead of a measurement, which is how SPEC-113's vacuous RAW test
reached `main`.

Symptoms traced to the same root, all documented:
- A test that passed with and without the fix it existed to protect (SPEC-113, fixed pre-merge).
- Three specs whose `cycle:` field never moved through build or verify, where SPEC-112
  three specs earlier had done it correctly.
- A build cycle with no recoverable cost record, because it ran in an orchestrator's main
  loop rather than as a metered subagent.

**Questions for the discussion, not answered here:** is one-session-per-worktree
enforceable rather than documented (a pre-commit hook refusing `feat/spec-*` commits from
the primary checkout is the obvious candidate)? Should a spec have a single named owning
session for its whole build cycle? And what should happen when a session ends mid-cycle —
today the answer is "another agent sweeps it up", which is precisely what failed.

See SPEC-113's ship reflection and `docs/framework-feedback/tooling-that-fails-silent.md`.
