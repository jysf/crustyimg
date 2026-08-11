# Feature-set triage — 2026-08-10

**Source.** A feature set produced in a separate exploration session and handed to this one by the
maintainer on 2026-08-10. **It is recorded here with provenance on purpose.** STAGE-036 carries a
five-item candidate list from an untracked draft that no committed record explains, and that list
read as a backlog to every session that met it until it was formally declined. This document exists
so this set does not become the same thing.

**What was done to it.** Every load-bearing factual claim was checked against the repo or against
crates.io on 2026-08-10, and the findings are recorded inline. Five checks changed the planning
answer. The verification standard is [[verify-handed-crate-lists-adversarially]] — a previous
session was handed 11 crates of which 2 did not exist and only 3 were viable.

**Status of this document.** A triage, not a backlog. Nothing here is committed work. Items marked
**→ existing home** are already planned elsewhere and should not be re-planned; items marked
**needs a decision first** are blocked on something other than effort.

---

## 1. Corrections — claims that did not survive checking

These are listed first because each one changes what the item costs or whether it is possible.

### 1.1 crustyimg is not single-threaded (affects §5, "the biggest functional gap")

`rayon = "1.12.0"` is **already a direct dependency** (`Cargo.toml:182`) and is already used in six
source files: `src/cli/build.rs`, `src/cli/optimize.rs`, `src/cli/ops.rs`, **`src/image/avif.rs`**,
`src/build/cache.rs`, `src/build/mod.rs`. Batch work is already parallel — `run_apply` fans out over
rayon and each task rebuilds its own pipeline because `Operation` is not `Send`.

So the framing *"single-threaded because it runs in a browser — correct for wasm and needlessly true
for native"* is **false as stated**, and the item is billed as the biggest functional gap at effort
**M**. `BENCHMARKS.md`'s finding that crustyimg is 3–14× slower on the clock but **a wash per core**
is consistent with batch already being threaded: the remaining gap is **within-image** parallelism
(a single large encode), which is a different and probably harder problem than adding rayon.

**Action: re-measure before planning.** The question to answer is where the wall-clock actually goes
on a single large image, not whether to adopt rayon.

### 1.2 The operation registry is already open (affects the whole lab packaging story)

The note asks *"is the registry a closed match on a fixed set of names? If so, that's the thing to
change."* It is not:

```rust
pub struct OperationRegistry { … }                                  // registry.rs:63
pub fn register(&mut self, name: &'static str, ctor: Constructor)   // registry.rs:92
```

`pub mod registry;` with `pub use registry::{OperationRegistry, RegistryError};`
(`src/operation/mod.rs:25-26`). Its own doc comment says *"New operations call register to add
themselves — without touching the recipe parser (the whole point of the registry seam)."*

**External registration works today.** The hardest-sounding part of the lab packaging design is
already built. What is genuinely unverified is whether `Constructor` and the `Operation` trait are
public enough for an out-of-crate implementor — check that before writing a spec, but the seam
exists.

### 1.3 Crate reality check

Checked against the crates.io API on 2026-08-10.

| crate | claimed for | finding |
|---|---|---|
| **`lut-rs`** | §2 LUT | **DOES NOT EXIST.** Named as one of three Rust LUT crates. |
| **`evalexpr`** | §8 (as an anti-recommendation) | **AGPL-3.0-only.** Rejected in the note on performance grounds; the licence is the harder blocker — it breaches `no-agpl-default-deps` (DEC-018). Right conclusion, wrong reason. |
| **`rawler`** | RAW Tier 2 (below the line) | **LGPL-2.1.** Not merely "a large jump in scope" — copyleft against a statically-linked single binary. Belongs on the licence watchlist, not the backlog. |
| **`wagahai_lut`** | §2, "take the crate" | Exists, but **v0.1.0, one release (2026-01-27), 544 downloads**, MIT. **Declined — see §3.1**; the repo's own in-housing precedent applies. |
| **`lut-cube`** | §2 alternative | v0.2.0, 1,969 downloads, licence reads **non-standard** — must clear `cargo deny` before it is an option even as a parser-only dep. |
| **`img_hash`** | §3 dedup | Last updated **2021-05-04**; effectively abandoned. |
| **`image_hasher`** | §3 dedup | v3.1.1, **2026-02-21**, 797k downloads, MIT OR Apache-2.0. This is the live fork — **use this one**; the note lists them as equivalents. |
| **`thumbhash`** | §1 placeholders | v0.1.0, last updated **2023-03-22**, MIT. Stale but tiny and the format is frozen; acceptable, but `blurhash` (v0.2.3, 746k dl, Apache-2.0/MIT) is the healthier half of the pair. |
| `qcms` | §12 ICC | ✅ v0.3.0, 2.5M downloads, MIT. Good call over `lcms2` bindings. |
| `jxl-oxide` | JPEG XL | ✅ v0.12.6, 2.0M downloads, MIT OR Apache-2.0, updated 2026-05. |
| `webp-animation` | §11 | ✅ v0.10.0, MIT OR Apache-2.0, updated 2026-04. |
| `show-image` | §14 window | ✅ v0.14.1, BSD-2-Clause. Permissive. |
| `kurbo`, `logos` | §7, §8 | ✅ both permissive and heavily used. |
| `meval` | §8 (anti-recommendation) | v0.2.0, last updated **2018**. Also correctly rejected. |

**Watchlist actions:** add `rawler`/LGPL (RAW Tier 2) and note `evalexpr`/AGPL against §8 in
`guidance/license-watchlist.yaml`, per its stated process — *"whenever a DEC/spec/review rejects or
defers a dependency for LICENSE reasons, add an entry (do NOT silently drop it)."*

### 1.4 Two items are already planned here

- **§1 (placeholders + dominant colour) is roadmap Wave 4**, marked `← NEXT` on 2026-08-10. The wave
  row already reads *"placeholders (blurDataURL + thumbhash/blurhash) + dominant color + favicon"*,
  and `docs/feature-exploration.md:18` has both at 💎. **Independent convergence**, which is a
  genuine signal for what PROJ-011 should be — not a new idea.
- **§6 (cross-verb invariant harness) is STAGE-042**, framed earlier the same day from the same
  evidence (four defects that were cross-cutting properties nobody specified). **Merge, do not build
  twice** — §6's phrasing is sharper and should be folded into STAGE-042's spec when it is framed.

Also: **decomposing `src/cli/optimize.rs`** (below the line in the note) was added to STAGE-036 on
2026-08-10, sequenced after STAGE-043 because that stage changes behaviour in the same file.

---

## 2. The decision that gates half the list

**"Lab" is not a feature — it reverses a standing anti-goal.** `docs/territory.md` states the fence
that has been killing proposals: *"Editor — Photoshop, GIMP, Squoosh UI — NOT us, automatic only."*

§7 (masks), §8 (expression filters), §9 (sweeps), §10 (watch-preview), §14 (window display), §18
(external tools) and the second demo **only exist if that fence moves.** That is roughly half the
set, and it is a positioning decision, not an effort estimate.

The note proposes the right test, and it is a good one precisely because it is derived rather than
imposed:

> **Lab can produce anything; the workhorse only accepts what generalizes across a batch.**

That falls out of the mask design — pixel coordinates do not survive a batch, so absolute rects and
seed floods are lab-only while luminance/chroma/percent-gravity masks are safe in a `build` recipe.
A fence that emerges from the domain holds; one that is asserted erodes.

**This wants a DEC before any lab spec**, in the same class as DEC-009 (edition) or DEC-052 (HEIC) —
recording what the anti-goal now is, not merely that the old one was dropped. `territory.md` should
be amended in the same change, consciously, per the note's own closing warning.

**Until that decision exists, every lab item below is `blocked-on-decision`, not `proposed`.**

---

## 3. Triage

Effort labels are the source's. "Home" is where the item would live if pulled.

### Take — workhorse, no new decision required

| # | Item | Effort | Verdict |
|---|---|---|---|
| §1 | Placeholders (blurhash/thumbhash) + dominant colour in the manifest | S | **→ existing home: roadmap Wave 4.** Strongest candidate for PROJ-011's opening spec. Nearly free — every image is already decoded. Use `blurhash` + `thumbhash`, noting the latter's staleness. |
| §3 | Perceptual dedup lint rule | S–M | **Take.** Best differentiation per unit of effort in the set; fits the existing `lint` framework with no new command; image bloat in git history is permanent. **Use `image_hasher`, not `img_hash`.** |
| §6 | Cross-verb invariant harness | M | **→ existing home: STAGE-042.** Fold the framing in. |
| §11 | Animated GIF → animated WebP/AVIF | M | **Take when pulled.** Closes `format/animated-gif`, which today detects a problem it cannot fix. "A linter that says *and here's the fix*" is the right argument. `webp-animation` verified. |
| §12 | ICC colour transforms | M | **Take when pulled, `qcms` only.** `lcms2` bindings would breach the zero-system-dependency property that the whole identity rests on. Closes `color/wrong-colorspace` the same way §11 closes its rule. |
| §15 | SVG optimization | M | **Take when pulled.** `usvg` is already in the tree for rasterize. ⚠ The claim *"there is no good Rust svgo"* is **unverified** — check before it becomes a positioning line. |
| — | JPEG XL decode | S | **Take when pulled.** Exactly the PROJ-009 input-reach pattern; `jxl-oxide` verified healthy. |
| — | Favicon / app-icon / OG sets | S–M | **→ Wave 4** (the roadmap row already names favicon). `skrifa` + `zeno` are already in the tree for text. |
| — | Declared convolution kernels | S | **Take.** Covers everything neighbourhood-based without giving expressions arbitrary pixel access — a genuinely good scope fence. |

### Measure or decide first

| # | Item | Blocker |
|---|---|---|
| §5 | Threading behind a feature flag | **Premise false (§1.1).** rayon is already in use. Re-measure where single-image wall-clock actually goes before this is a plan. |
| §2 | LUT op — **build the `.cube` reader in-house** | **Decided 2026-08-10, see §3.1 below.** The *feature* is a take; the *dependency advice* was reversed by the maintainer once the crate data landed. |
| §4 | Lint in wasm | **Measure bundle size first.** `just wasm-size` before/after. The profiled size is load-bearing: DEC-066, SPEC-074, the npm README's claim, and `tests/npm_smoke.mjs` asserts it. Unblocks §17. |
| §13 | Autofix mode on `crustyimg-action` | **Design, not effort.** The agent-reads-SARIF-and-opens-a-PR route sidesteps fork-safety and write permissions, which is the right insight — but it is a different product (a bot) with its own trust model. |
| §17 | MCP server exposing measurements | **Gated on §4.** The reframe — *crustyimg is a measurement instrument, and measurement is what LLMs are worst at* — is the strongest strategic idea in the set and deserves its own framing session. Note it needs no new capability: `lint --format json`, `diff --json`, `info --json`, `build --check` all exist. |
| §16 | Brand consistency as a build gate | **Positioning, gated on §2.** The one genuinely commercial angle surfaced. Worth recording in `territory.md` even before §2 exists. |

### 3.1 The LUT dependency call — reversed by the maintainer, 2026-08-10

The source note said: *"Do not write a `.cube` parser. LUT application is commodity — `wagahai_lut`,
`lut-cube`, `lut-rs` in Rust alone… Take the crate."* **The maintainer reversed this** once the
crate data came back, and the reversal is right:

- `lut-rs` **does not exist**.
- `lut-cube` is 1,969 downloads with a licence crates.io reports as **non-standard** — which is a
  `cargo deny` question before it is anything else (DEC-018, `permissive-license-policy-cargo-deny`).
- `wagahai_lut` is **v0.1.0, one release, 544 downloads**. Against DEC-018's supply-chain gate that
  is not a commodity pick; it is the class of dependency most likely to need replacing.

**The precedent argument is the decisive one, and it verifies.** This repo has repeatedly in-housed
small dependencies rather than carry that risk — confirmed 2026-08-10:

| dropped | replaced with | evidence |
|---|---|---|
| `little_exif` | own TIFF-IFD reader/writer | **absent from `Cargo.lock`**; `src/metadata/tiff.rs` is **718 lines** |
| `ab_glyph` | `skrifa` + `zeno` | **absent from `Cargo.lock`**; both replacements are direct deps |

`.cube` is genuinely trivial: a few header lines (`TITLE`, `LUT_3D_SIZE`, `DOMAIN_MIN`/`DOMAIN_MAX`)
then N³ RGB triples. **~100 lines to parse, ~50 for trilinear interpolation — about a fifth of the
TIFF-IFD writer this repo already wrote and maintains.**

**Decision: build it in-house.** If a parser dependency is still wanted, `lut-cube` for *parsing
only* is the fallback — 4× the downloads and a much smaller trust surface than a full apply-path
crate — but **its licence must clear `cargo deny` first**, and parser-only still means the licence
applies. Interpolation stays in-house either way.

**What is NOT reversed:** the feature itself, and its differentiator. *"The LUT inside a reproducible
build — the file's hash becomes part of the cache key, so changing the grade invalidates exactly the
affected outputs and `build --check` catches an accidental grade change in review"* is genuinely
unoccupied territory, and it is a **workhorse** feature, not a lab one. It is also what unlocks §16
(brand consistency as a build gate), the one commercial angle in the set.

### Blocked on the lab decision (§2 above)

§7 named composable masks (M–L) · §8 expression filters with bake-to-`.cube` (L) · §9 parameter
sweeps with contact sheets (S–M) · §10 watch-preview loop (S) · §14 window display, hard-gated (S) ·
§18 external tools (design) · the second lab demo.

Two notes worth preserving whatever is decided:

- **§8's bake-to-`.cube` is the novel piece.** ImageMagick has `-fx` and separately has `hald-clut`;
  nothing compiles the former into the latter. It also solves the architecture problem elegantly —
  the main binary never needs an evaluator, wasm stays small, and the cache key just hashes a file.
  The "no prior art found" claim is **unverified** and should be checked before it is published.
- **§18 is a genuine hazard, not just a feature.** `migrating.md` currently sells *"there is no
  `policy.xml` because there is no delegate system to lock down."* ImageMagick's delegate system is
  the source of most of its CVEs. If lab ever gains external tools, lab must **not** inherit
  crustyimg's "safe on untrusted input" claim, and external ops must be forbidden in `build` recipes
  (exit 4 covers it) because they break the cache key.

### Below the line, unchanged

Entropy-based smart crop (already 💎 in `feature-exploration.md`; preserves the no-ML pillar) ·
op stack with undo/checkpoints (lab) · **full RAW demosaic — now licence-blocked, see §1.3.**

---

## 4. Packaging — the note's conclusion holds, and is cheaper than stated

No `crustyimg-core` rename is needed: `crustyimg` is already lib + bin, and Cargo does not build a
dependency's binary target. The `cli` feature gate for `clap`/`clap_complete`/`indicatif` mirrors
the pattern already used for `viuer` behind `display`. And per §1.2 the registry seam is already
open, so `crustyimg-ops-lab` as a separate crate is a packaging choice rather than an architectural
one.

**Unverified and worth one check before a spec:** whether `Constructor` and the `Operation` trait
are public enough for an out-of-crate implementor, and whether `required-features` on `[[bin]]`
behaves as expected with the existing default feature set.

---

## 5. What this changes about the plan

Nothing in the immediate sequence. **STAGE-043 → STAGE-041 → STAGE-042** stands, with §6 folded into
042.

For **PROJ-011**, this set adds a second independent argument for **roadmap Wave 4** (web-asset
manifest + placeholders + dominant colour + favicon), with §1 as its opening spec and §3
(perceptual dedup) as a strong, cheap companion that needs no new wave.

The **lab decision** is the one thing that should not be made incrementally, spec by spec. It is
worth its own session, and it should produce a DEC and a `territory.md` amendment before any of the
seven items that depend on it is framed.
