---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-041                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-08-10
shipped_at: null

value_contribution:
  advances: >
    PROJ-010 made the product defensible; nobody knows it exists. Every gating defect is
    fixed and 0.7.0 is live on all three channels, and the launch that work was for has
    no post, no asset, and nowhere written down to publish it. This stage produces the
    content and the plan — it does not write the maintainer's narrative for them.
  delivers:
    - "A publication plan: which channels, in what order, on what schedule, with what asset each needs"
    - "The factual scaffolding a hostile reader will check — honest numbers, honest limits, the RAW split stated correctly"
    - "A pre-publication checklist that is driven, not assumed — install paths at 0.7.0, the live demo, the README's fenced commands"
  explicitly_does_not:
    - "Write the post. The narrative is the maintainer's voice; a Show HN in someone else's voice reads wrong and gets read that way."
    - "Publish anything. Every outward-facing post is maintainer-authorized, like the tag and the npm publish."
    - "Decide the go/no-go. That is the maintainer's call and stays on docs/launch-readiness.md."
---

> ⚠ **MOVED from PROJ-010 to PROJ-011 on 2026-08-23, at the maintainer's request, so it can be
> continued in the active project. Read this before counting it toward anything.**
>
> **This stage does NOT share PROJ-011's thesis.** PROJ-011's thesis is *a declared `build` can
> express what the CLI can do*; launch content and publication is unrelated to it. The stage is
> carried here for **continuity, not coherence** — its `value_contribution` advances PROJ-010's
> thesis, which has been delivered, and it should **not** be read as advancing PROJ-011's or as
> evidence for or against it at project reflection.
>
> ⚠ **Its repo status understates reality and has since 2026-08-16.** The maintainer has reported
> that **three of its four items are substantially done outside this repo**, so `just backlog`
> counting four open items is wrong and has been for a week. **Do not re-plan this stage against
> what the repo says** — get the real status first, mark what is done, and see what actually
> remains. It may be one item, or none.
>
> 📌 It was moved rather than closed because closing a stage whose true status is unknown would be
> guessing. Moving it unblocked PROJ-010's closure without pretending to know something nobody
> here knows.

# STAGE-041: launch content and publication plan

## What This Stage Is

The stage that turns a correct product into a launched one. PROJ-010 spent four stages making
`crustyimg web` do the right thing on every input it touches, and STAGE-040 got that work into
users' hands as 0.7.0. **What does not exist anywhere in this repo is a post, an asset, a channel
list, or a schedule** — verified 2026-08-10 by search, not assumed. `docs/territory.md` holds the
positioning and `docs/roadmap.md` Track B holds the adoption thesis, but neither is content and
neither says where or when.

This stage closes that gap up to, and stopping at, the maintainer's own words.

## Why Now

- **The only remaining launch blockers are maintainer-only, and one of them is this.**
  `docs/launch-readiness.md`'s critical path is down to the device pass, the install-path
  re-verification, **the post draft**, and the go/no-go. Three of those four are hours of work;
  the post has been an unchecked box (*"A GIF/screenshot + the post narrative"*) since the board
  was written.
- **The evidence is at its freshest.** BENCHMARKS.md's numbers, the hostile-input corpus, the
  0.7.0 fixes and the driven halftone result are all current and all defensible right now. A post
  written six weeks from now re-derives them or, worse, does not.
- **Waiting costs the strongest version of the story.** The honest framing — *we publish the
  benchmarks where we lose* — is most credible while the release that earned it is the newest one.

## Success Criteria

- A **publication plan** exists naming each channel, its order, its timing, its asset
  requirements and its audience-specific framing — concrete enough that the maintainer executes
  it without re-deciding anything.
- The **factual scaffolding is written and every number in it is traced to a command that
  produced it**, not to a previous document. [[a-citation-looks-like-prose-not-a-claim]]
- The **CLI-vs-demo RAW split is stated correctly** wherever it appears: the CLI reads RAW via the
  embedded preview, and since SPEC-103 the demo does too, behind a stated 60 MP gate.
  The board has flagged this as a required correction to the draft since STAGE-028.
- **The pre-publication checklist is driven, not read**: install one-liners at 0.7.0 on each
  channel, the live demo on a real page, and the README's fenced commands.
- **Goals 2 and 3 appear nowhere in any public artifact** — see Design Notes.
- The maintainer has everything needed to write the narrative, and **the narrative is not written
  here**.

## Scope

### In scope
- The publication plan: channels, order, schedule, per-channel framing and asset needs.
- The factual scaffolding: the honest-numbers table, the honest-limits section, install
  one-liners, and prepared answers to the questions a hostile reader will actually ask.
- The demo asset — a GIF or screenshot sequence — specified and produced.
- The pre-publication verification pass.
- Correcting the RAW split wherever the existing material states it wrongly.

### Explicitly out of scope
- **Writing the post narrative.** Maintainer's voice, maintainer's call.
- **Publishing.** Every post is outward-facing and maintainer-authorized.
- **The go/no-go decision**, which stays on `docs/launch-readiness.md`.
- Any code change. If the launch pass finds a defect, it gets filed, not fixed here — the same
  discipline that kept STAGE-040 from growing.
- Sustained adoption work (SEO, docs site, integrations). If this grows past a launch into a
  campaign it wants its own project under roadmap Track B, not another stage here.

## Amendment (2026-08-16): what changed under this stage while it waited

Written 2026-08-10. Since then PROJ-010 shipped nine more specs and framed a defect stage that
precedes this one. Three consequences, so nobody executes the backlog below against stale
assumptions.

### 1. `BENCHMARKS.md` predates the fixes it is supposed to showcase

Last written **2026-07-23**. **Thirteen specs have shipped since** — including **SPEC-108**, the
classifier fix for the 18.5× blow-up, which is the single change most likely to move a
quality-per-byte table, plus SPEC-110 (orientation baked on every pixel verb), SPEC-113 and
SPEC-115 (`optimize` never grows a pinned output / never passes through bytes it cannot name).

So the "honest numbers" item is not a refresh-if-convenient. **Every number in that file was
produced by a binary that no longer exists**, and the stage's own success criterion says each one
must trace to a command that produced it.

### 2. STAGE-046 precedes this stage and will move the numbers again

Maintainer decision, 2026-08-15. Two of its four items — colour-type/bit-depth preservation and
linear-light resampling — **change output bytes for every existing recipe** by design; that is
why they carry a lockfile-migration story. A benchmark table or an install-verification pass
completed before they land gets redone.

**What that means in practice:**

| item | safe to do now | why |
|---|---|---|
| The publication plan | ✅ **yes** | channels, order, timing and per-channel framing do not depend on any measured number |
| Prepared answers to hostile questions | ✅ **yes** | *why not sharp*, *it's slower*, *squoosh-cli is abandoned* are positioning, not measurement |
| The RAW-split correction | ✅ **yes** | a factual correction that is true regardless of release |
| The demo asset | ⚠️ **partly** | *"client-side, zero network requests"* is durable; anything showing byte counts is not |
| The honest-numbers table | ❌ **wait** | will be re-derived after STAGE-046 |
| Install-path verification | ❌ **wait** | STAGE-046 produces a release; verification is version-specific |

### 3. Two new facts belong in the honest-limits section

Both measured this week, both the kind a hostile reader finds:

- **Animated input is flattened to one frame.** SPEC-119 made it warn on GIF, APNG and animated
  WebP, and `lint --max-warnings 0` is the strict gate — **except in directory mode for animated
  WebP**, where the `IMAGE_EXTENSIONS` gap makes it a false green (STAGE-042, PRIORITY).
  Animated *output* does not exist yet.
- **`resize` resamples in sRGB, not linear light** — measured, premise confirmed (DEC-092), fix
  pending on STAGE-046. Worth knowing before publishing a quality claim, even if it stays out of
  the copy.

**Neither is a reason to delay the launch.** They are reasons the honest-limits section should be
written after STAGE-046, not before.

## Spec Backlog

- [ ] (not yet framed) — **The publication plan.** Which channels, in what order, on what
  schedule. Show HN and r/rust are the two the repo has always named; the plan should decide
  whether they go same-day or staggered, what a Tuesday-morning-ET posting actually buys, and
  which of lobste.rs / This Week in Rust / the Rust Discord / HN's "Show HN" rules are in or out
  **with a reason either way**. Per-channel framing differs materially: r/rust cares about the
  Rust story and the crate, HN cares about the wedge and the demo. Complexity **S**.

- [ ] (not yet framed) — **The factual scaffolding.** Everything a hostile reader checks, written
  so the maintainer can drop it into their own prose:
  - the honest-numbers table from `BENCHMARKS.md` — **including that crustyimg wins size 0 of 8
    and runs 3–14× slower on the clock**, and the per-core wash that explains it as threading
    rather than the encoder;
  - the honest-limits section (WebP lossless-only, AVIF encode-not-decode, the `file://`
    explanation, HEIC opt-in, the 60 MP demo RAW gate);
  - the install one-liners, **verified at 0.7.0** rather than copied forward;
  - prepared answers to the predictable ambushes: *"why not sharp"*, *"it's slower"*,
    *"squoosh already does this"* (squoosh-cli is **abandoned** — that is the wedge, per
    `docs/territory.md`), *"why should I trust the quality numbers"* (one SSIMULACRA2 scorer,
    iso-quality rule, published harness).
  **Every number re-derived from a command, not copied from a doc.** Complexity **M**.

- [ ] (not yet framed) — **The demo asset.** The board asks for "a GIF/screenshot". Decide which,
  specify what it shows, and produce it. The strongest candidate is the thing no competitor can
  show honestly: a real image converted **client-side with zero network requests**, which
  SPEC-077/078 already prove and instrument. Must be produced from the live 0.7.0 demo, not a
  local dev build. Complexity **S–M**.

- [ ] (not yet framed) — **The pre-publication verification pass.** Driven, not read:
  `cargo install` / `cargo binstall` / `brew install jysf/tap/crustyimg` each actually run at
  0.7.0; the README's fenced commands executed (SPEC-082 did this at 0.5.0 and it has not been
  repeated); the live demo loaded and driven on the deployed page. **The 0.7.0 binary itself is
  already confirmed** — downloaded, checksum-matched against both its `.sha256` and the Homebrew
  formula, and driven (STAGE-040) — so what is open is the *install paths*, not the artifact.
  Complexity **S**.

**Count:** 0 shipped / 0 active / 4 pending (none framed)

## Design Notes

- **The honest-loss framing is an asset, and it only works if we lead with it.** `BENCHMARKS.md`
  publishes that crustyimg loses on size 0 of 8 and is slower on the clock. A reader who finds
  that themselves after reading a puff piece is hostile for the rest of the thread; a reader who
  is handed it up front extends credit for everything else. SPEC-083 already made this call and
  paid for it — the post should not quietly retreat from it.
- **⚠ Goals 2 and 3 stay out of every public artifact.** crustyimg serves three purposes — a real
  tool, a template testbed, and an agent-workflow experiment — and **only the first is public**.
  No post, asset, comment or README change may reference the latter two. This is the single
  easiest thing to get wrong in launch content, because the process story is genuinely the more
  interesting one to the people who built it.
- **The RAW split has a known-wrong version in circulation.** The board has carried *"the draft
  needs a CLI-vs-demo RAW split fix"* since STAGE-028. Correct statement: the CLI extracts the
  camera's embedded preview for `.dng`/`.cr2`/`.nef`/`.arw`; the demo does too since SPEC-103,
  declining previews above 60 MP with a note pointing at the CLI (DEC-082, tuned by SPEC-104
  against a real Leica file). It is not a RAW *develop* — no demosaic, no white balance — and the
  post must say so.
- **A launch post is an irreversible outward-facing act**, in the same class as the tag push and
  the npm publish. It gets the same treatment: prepared here, fired by the maintainer.
- **Nothing here is gated on STAGE-042.** The instruments stage that follows protects the *next*
  release; it does not block this one.

## Dependencies

### Depends on
- **STAGE-040** (shipped) — 0.7.0 must be live before install one-liners can be verified at it,
  and before a post can point at it. Done.
- `docs/territory.md` (the wedge), `docs/roadmap.md` Track B (the adoption thesis),
  `BENCHMARKS.md` (the numbers), `docs/launch-readiness.md` (the board and the go/no-go).

### Enables
- The Show HN / r/rust go/no-go — after this, the only open items are the maintainer's device
  pass and the decision itself.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
