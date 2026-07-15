# The territory crustyimg owns

> A living positioning note: the space we are claiming, why it's open, what we own vs.
> adjacent tools, and what we deliberately do NOT own. Pairs with `docs/moat.md` (what's
> *built*) and `docs/roadmap.md` (how we *get there*). Refine freely — this is a working
> thesis, not a fixed claim. Snapshot: 2026-07-05.

## The one thing we own

**The pre-deploy image asset layer.** The single pure-Rust binary that turns your *source*
images into the best *delivery* artifacts under a declared intent — **decided, explained,
verified, and manifested** — for any SSG, build step, or CI, with **no service and no
runtime**, and safe on untrusted input.

Said as a wedge: *"Tell it the outcome you want; get the smallest artifact that meets it, in
the right modern format, with a record of every decision and a manifest a build can consume —
from one binary with zero system dependencies."* This extends the shipped wedge ("set the
look, not the number") from **one optimized file** to **a described, verifiable set of
assets**.

## Who it's for

Developers and teams who prepare images in a **build/CI/pre-deploy** context and want it
**automatic, local, and reviewable** — not a hosted service, not a Node toolchain, not a GUI:
- static-site / JAMstack authors (Eleventy, Astro, Hugo, Zola, Jekyll, plain static)
- build-step / Makefile / GitHub-Actions shops optimizing an asset tree
- anyone who reached for `sharp`, `squoosh-cli` (now abandoned), or `magick` in a pipeline
- teams that want image changes **reviewed like code** (deterministic, reproducible recipes)

## Why the space is open (grounded)

Each adjacent layer has a structural gap crustyimg is shaped to fill:

| Layer | Incumbents | Their shape / gap | crustyimg's claim |
|---|---|---|---|
| **Delivery / CDN** (per-request) | Cloudinary, imgix, imgproxy, thumbor | Hosted service; `f_auto` decides per-request at the edge. Great, but it's a *service* you must adopt and pay, and it decides at *delivery* time. | The **local / CI / no-CDN `f_auto`** — the same "declare intent, tool decides" at *build* time, deterministic, no service. |
| **Build step** (Node) | sharp, squoosh, eleventy-img, plaiceholder | Node + a native addon (install/ABI/CI-breakage friction); no standalone binary; squoosh-cli is **abandoned**. | A **single self-contained binary**, no runtime, **+ a manifest** neither sharp nor ImageMagick emits. |
| **CLI optimizer** | rimage, ImageMagick, caesium-clt | rimage is **C-backed** (mozjpeg/libavif) and narrow (optimize-only). ImageMagick is cryptic and **unsafe by default** (the ImageTragick/GhostScript RCE lineage → mandatory `policy.xml` hardening). | **Pure-Rust, safe-on-untrusted, a general engine** (decide + explain + lint + manifest), not a narrow single-purpose compressor or a cryptic Swiss-army knife. |
| **Page audit** | Lighthouse, PageSpeed | Runs **in-browser against a deployed URL**; can't touch a source asset tree. | **Source-file, no-URL, pre-deploy `lint`** with a CI exit code. |
| **Editor** | Photoshop, GIMP, Squoosh UI | Manual, interactive. | **NOT us** — automatic only (see anti-goals). |

Underneath: the **component (crate) layer is hot** — a pure-Rust imaging frontier exists
(zune, the awxkee SIMD cluster, ravif/rav1d, ssimulacra2, jxl-oxide) — but **nobody assembles
it into one ergonomic, safe, single-binary engine for the build layer.** That's the opening.

## The three pillars that make the territory ours

1. **Outcome-driven & explainable.** Declare intent (a quality target or a byte budget, a
   profile), and the engine picks format + quality + dimensions and produces the best artifact
   — then **tells you why** (format chosen, bytes saved, perceptual score). The `f_auto`
   pattern, made *local*, *transparent*, and *reproducible*.
2. **Safe, permissive, pure-Rust.** One static binary, zero system deps, **safe on untrusted
   input** by default (the direct answer to ImageMagick's #1 liability), MIT/Apache so it's
   freely embeddable and redistributable.
3. **Build-native & verifiable.** Deterministic, reproducible recipes reviewed like code; a
   **source-file linter** with a CI exit code; and a **machine-readable manifest** — designed
   for the build/CI/pre-deploy layer, not a running service or a GUI.

No single competitor combines all three. That combination *is* the territory.

## Scope discipline — what we optimize for, defer, and won't be

Naming this protects the identity **without foreclosing the future**. Three tiers, not a wall
of "nots":

**What we optimize for (the core, now).** A pure-function (`input + recipe → output`),
build-time, **single local binary** — automatic, deterministic, safe, explainable. Every
near-term decision serves the lens *"does this help produce a better artifact automatically?"*

**Deliberately deferred — open future branches (not foreclosed):**
- **Editor use.** crustyimg already has `edit` + a recipe pipeline, and geometry/color/effects
  ops are editor-adjacent. We are **optimization-first, not editor-first** — but manual,
  editor-style use is a legitimate secondary use (including the maintainer's own), and a future
  interactive/TUI surface (the ratatui recipe editor) is a *welcome differentiator*, not a
  betrayal. The guardrail is **sequencing, not prohibition**: the automatic engine leads; the
  editor rides on the same `Operation`/recipe core.
- **Possible future commercial direction.** A revenue branch may exist eventually as a
  **separate, deferred product** for a paying audience — always *additive to a different buyer*,
  never gating the free engine, never rug-pulling, never becoming a bandwidth-metered CDN. The
  guardrails above are the public commitment. The specifics are kept out of this public repo
  (private, gitignored `business/` notes) — this doc only records the *boundary*, not the plan.

**What we won't be (firm):**
- **We don't absorb the separate web-content / site builder.** That's the maintainer's distinct
  tool; the **manifest is the seam** that keeps them apart. No HTML generation, templating,
  routing, or OG-card rendering *in crustyimg*.
- **We don't rug-pull the core.** The engine and the ability to run fully local/offline stay
  forever-free and permissive — the research is blunt that taking away something that was free
  (HashiCorp/Redis/LocalStack) is what kills goodwill. Any future Pro/hosted offering is
  *additive to a different buyer*, never a crippling of the free CLI.
- **We don't become the thing we're replacing** — cryptic (ImageMagick), unsafe-by-default, a
  required runtime (Node), or a heavy system dependency. Safe, permissive, pure-Rust,
  single-binary is non-negotiable — it's the whole wedge.

*(On statelessness: the pure-function model is a **strength** to lean on — a stronger determinism
story than Terraform's mutable state, its worst part. A future content-addressed cache is local
and derived, not the mutable-infra state we avoid.)*

## How we widen it (the ownership ladder)

Each rung deepens ownership of the pre-deploy layer (see `docs/roadmap.md` for the projects):

```
shipped:   one optimized file  (outcome-driven perceptual compression)
   ↓ PROJ-002/003   auto-decided + explained   ("the local f_auto", + the planner)
   ↓ PROJ-004       linted + enforced in CI     (source-file, no-URL lint)
   ↓ PROJ-005       a described, verifiable asset set + manifest   (the web-workflow interface)
   ↓ PROJ-006/007   full geometry + format coverage + reproducible caching
   =  own the pre-deploy image asset layer
```

## The part the technical moat can't win alone

Owning the space *technically* is necessary but **not sufficient**. Adoption is a **separate
branch of work** — design partners, real-world feedback, docs/site, cross-tool benchmarks
(`BENCHMARKS.md`), and SSG/CI integrations — tracked as the "Reach & adoption" track in
`docs/roadmap.md`. A capable, safe engine that nobody has adopted isn't yet a territory held;
it's a territory *claimed*. The build-out earns the claim; the adoption track holds it.

## Pointers
- What's built: `docs/moat.md`. How we get there: `docs/roadmap.md`. The scoping evidence:
  `docs/research/proj-002-findings.md` (+ the `proj-002-design-*.md` briefs).
