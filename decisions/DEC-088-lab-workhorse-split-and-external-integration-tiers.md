---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-088
  type: decision
  confidence: 0.86
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: 3fe6a9a9-d3cd-477d-a4f3-5bab33ebf8cb

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-10
supersedes: null
superseded_by: null

affected_scope:
  - "docs/territory.md"
  - "docs/feature-set-triage-2026-08.md"
  - "docs/migrating.md"
  - "src/operation/registry.rs"
  - "src/source/mod.rs"
  - "guidance/license-watchlist.yaml"

tags:
  - positioning
  - scope
  - lab
  - security
  - dependencies
---

# DEC-088: the lab/workhorse split, and the three tiers of external integration

## Decision

Two decisions, recorded together because the second is what keeps the first safe.

**1. `crustyimg-lab` is sanctioned as a second binary, and the fence is
*generalization*, not *capability*.**

- **Lab may produce anything. The workhorse accepts only what generalizes across a batch.**
- Lab is CLI- and pipeline-based like the workhorse, ships as its own **single static binary**,
  stays **pure-Rust and permissively licensed**, and **rides the shared `Operation`/recipe core**
  rather than forking it.
- A lab recipe is a **superset** of a workhorse recipe — same `[[op]]` array, same registry. A
  recipe carrying a lab-only op that reaches the workhorse exits **4** ("capability not built in",
  the existing precedent for AVIF on lean builds), naming the op and pointing at lab.

This is **not a reversal of an anti-goal.** `docs/territory.md`'s anti-goals section already placed
Editor use in the *"Deliberately deferred — open future branches (not foreclosed)"* tier, with the
guardrail stated as **"sequencing, not prohibition: the automatic engine leads; the editor rides on
the same `Operation`/recipe core."** Both halves of that condition are now met (see Context). This
DEC records that the deferred branch is **active**, and fixes its fence before the first spec.

**2. External integration has three tiers. Two are sanctioned; the third is not built.**

| tier | pattern | status |
|---|---|---|
| **1. File interchange** | Read/write files another tool produced or consumes | **Preferred.** No process spawned, no attack surface, and **hashable — so it participates in the build cache key.** |
| **2. Pipes** | `-` for stdin, `-o -` for stdout; the user composes the pipeline | **Sanctioned, and already implemented across the verb set** — driven `stdin → stdout` on the released 0.7.0 binary, **11/11 exit 0** (`info`, `resize`, `thumbnail`, `convert`, `web`, `optimize`, `auto-orient`, `watermark`, `edit`, `lint`, `apply`). `src/source/mod.rs:158`. |
| **3. Spawning a process** | crustyimg invokes an external binary | **NOT BUILT.** See Alternatives. |

**Corollary, and the operative rule going forward:** a request to "use tool X" is triaged as —
*is X a permissive Rust crate?* → an ordinary dependency decision under DEC-018. *Is X a file
format?* → tier 1. *Is X a pipeline stage?* → tier 2. **If none of the three, it is out of scope,
not a delegate.**

## Context

### Why the editor branch is open, not closed

The line habitually quoted as the anti-goal — *"Editor — Photoshop, GIMP, Squoosh UI — NOT us,
automatic only"* — is `docs/territory.md:40`, a **row in the competitive-landscape table**
(`## Why the space is open`), not an anti-goal. Its "their shape / gap" column reads **"Manual,
interactive"**, and the cell defers explicitly: *"(see anti-goals)"*.

The anti-goals section says nearly the opposite. **"Editor" appears zero times in the firm
"What we won't be" tier.** It appears in the deferred-but-open tier, which states that
geometry/color/effects ops are *"editor-adjacent"*, that manual editor-style use is *"a legitimate
secondary use"*, and that a future interactive/TUI surface is *"a welcome differentiator, not a
betrayal."*

Its condition — the engine leads, the editor rides the shared core — is satisfied on both halves:

- **The automatic engine led.** PROJ-010 closed four launch-gating stages and **0.7.0 is live on
  crates.io, Homebrew and the Releases page** (2026-08-10), verified by driving the published
  binary.
- **The shared core is already open.** `OperationRegistry` is `pub`, `register(&mut self, name,
  ctor)` is `pub` (`src/operation/registry.rs:63,92`), `pub mod registry` is re-exported, and its
  own doc comment states that outside registration *"without touching the recipe parser"* is **the
  whole point of the registry seam**. No architectural change is required for lab ops to ride it.

### Why the fence is generalization

It is **derived rather than asserted**, which is why it should hold. Pixel coordinates do not
survive a batch: `rect = [100,200,400,300]` is meaningless applied to a thousand images of different
sizes. That single fact splits mask producers cleanly — luminance, chroma, alpha and
percent/gravity shapes generalize and are safe in a `build` recipe; absolute rects and seed floods
do not and are lab-only. The fence falls out of the domain instead of being imposed on it, so it
tells you where a *future* feature belongs without re-litigating scope.

### Why tier 3 is different in kind

**The direction of control is the entire security story.** ImageMagick's CVE lineage
(ImageTragick / the Ghostscript RCEs) does not come from *using* Ghostscript — it comes from
*deciding to*, implicitly, based on file content. A pipe inverts that: the user composes the
pipeline; crustyimg spawns nothing, has no allowlist to get wrong, no shell to escape, and no
filename to interpolate.

This is load-bearing positioning, not a preference. `docs/territory.md:38` wins the CLI-optimizer
layer on ImageMagick being *"cryptic and **unsafe by default** (the ImageTragick/GhostScript RCE
lineage → mandatory `policy.xml` hardening)"*, and `docs/migrating.md` sells *"there is no
`policy.xml` because there is no delegate system to lock down."* A delegate system would make both
claims false.

### The external tools we would actually want are not external

Probed 2026-08-10 against the crates.io API:

| wanted for | finding |
|---|---|
| PNG lossless recompression | **`oxipng` — v10.2.0, MIT, pure Rust, 1.74M downloads, updated 2026-08-09.** An ordinary dependency decision. |
| deflate | **`zopfli` — v0.8.3, Apache-2.0, pure Rust, 101M downloads.** Same. |
| better JPEG | `mozjpeg` — already on the licence watchlist (`mozjpeg-encode`), with DEC-022's off-by-default feature pattern as its way in. |
| GIF optimization | No `gifsicle-sys` on crates.io — but converting animated GIF → animated WebP/AVIF beats optimizing a GIF, so the need dissolves. |
| metadata | ExifTool is the gold standard and is **Perl** — a required runtime, which the firm tier forbids. `src/metadata/tiff.rs` (718 lines) already covers our needs. |
| video | ffmpeg — out of scope; crustyimg is images. |

## Alternatives Considered

- **Build the delegate system carefully (allowlist, array args, no shell, temp files).** Rejected.
  The constraints are correct and still insufficient: a spawned tool **cannot participate in the
  build cache key** (`build` cannot know `oxipng` was upgraded), so it breaks reproducibility — the
  property PROJ-007 exists to provide. It would also force lab to forfeit the "safe on untrusted
  input" claim. If it is ever revisited, external ops must additionally be **forbidden in `build`
  recipes** (exit 4).
- **Keep lab inside the main binary behind a feature flag.** Rejected for the demo-size reason: the
  npm/wasm bundle carries a profiled size claim (DEC-066, SPEC-074) asserted by
  `tests/npm_smoke.mjs`. Two binaries off one engine keeps the main demo lean and lets a lab demo
  carry masks/filters without a size budget.
- **Split the engine into a `crustyimg-core` crate first.** Rejected as unnecessary. `crustyimg` is
  already lib + bin, and Cargo does not build a dependency's binary target, so lab can depend on
  `crustyimg` directly. The only leak is that `clap`/`clap_complete`/`indicatif` are unconditional;
  a `cli` feature fixes that, mirroring the existing `display`/`viuer` pattern. **A rename would be
  churn for no gain.**
- **Leave the editor branch deferred and decide per feature.** Rejected. "Programmable editor set"
  can absorb infinite features; deciding case by case is how an anti-goal erodes silently rather
  than being amended consciously.

## Consequences

- **`docs/territory.md:40` must be amended** — its bare *"NOT us — automatic only"* is now
  misleading about what is actually excluded. Manual **interactive GUI** editing remains out; the
  programmable CLI editor is lab. Amended in the same change as this DEC, original preserved.
- **Two items are constrained by the firm tier**, and this DEC does not clear them:
  - **Window display** pulls X11/Wayland on Linux — the heavy system dependency the firm tier calls
    non-negotiable. Off-by-default at most (`--features window`); `view` on viuer already covers the
    terminal case, which is the one in regular use.
  - **External tools (tier 3)** — not built, per Decision 2.
- **STAGE-042's conformance matrix gains a stdin/stdout axis.** Tier 2 works today — driven 11/11
  on the released binary — so the axis exists to **keep** it working, not to discover whether it
  does. Sanctioning it as an integration surface is what turns "`-o -` happens to work" into a
  contract, and contracts need the same unenumerated-cell protection as decide-vs-pinned.
- **Lab must not inherit the workhorse's safety claims by association.** Whatever lab ships, the
  "safe on untrusted input" language belongs to the workhorse and its threat model (PROJ-007 /
  DEC-062), and lab's own posture must be stated separately.
- Recipe compatibility is one-way by design: workhorse recipes run in lab; lab recipes may not run
  in the workhorse (exit 4).

## Validation

**Right if:** a proposed lab feature can be placed on the correct side of the fence without a scope
argument; a "can we use tool X" request resolves through the three-tier triage without reopening
positioning; and no `policy.xml`-shaped hardening surface ever appears in either binary.

**Wrong if:** the generalization fence turns out not to cut cleanly — e.g. a genuinely useful op
that generalizes across a batch is nonetheless unimplementable in the workhorse, or a lab-only op
becomes so commonly wanted in `build` that exit 4 is mostly friction. Either would mean the fence is
tracking the wrong property and should be re-derived, not widened ad hoc.

**Also wrong if** the two-binary split proves to cost more than it saves — if the shared core needs
constant `pub` widening for lab, that is evidence the seam is in the wrong place.

## References

- `docs/territory.md` — the three-tier scope discipline; the competitive table; line 40 (amended).
- `docs/feature-set-triage-2026-08.md` — the 18-item feature set this decision unblocks, its §2
  (the corrected anti-goal reading) and §3.2 (the external-integration tiers).
- `docs/migrating.md` — the "no delegate system to lock down" claim this protects.
- `DEC-018` — permissive licence policy / cargo-deny; the gate for tier-1 crate adoption.
- `DEC-022` — the off-by-default feature pattern for a non-pure-Rust encoder.
- `DEC-062` — the threat model lab must not silently inherit.
- `DEC-066` / `SPEC-074` — the wasm size claim motivating two binaries.
- `src/operation/registry.rs:63,92` — the seam that makes lab possible with no architectural change.
- `src/source/mod.rs:158` — tier 2, already implemented.
