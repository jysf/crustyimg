# crustyimg roadmap — the road to 1.0 and beyond

> **A living roadmap.** Per `AGENTS.md §2`, a project is framed formally only once the prior
> one ships, so everything below is **direction, not commitment** — IDs, versions, and scope
> are provisional and *will* change as research lands. Ladders up to `docs/territory.md`;
> grounds in `docs/research/proj-002-findings.md`. **Snapshot: 2026-07-07** — reordered
> around an *adoption-first* reconciliation (see "Sequencing rationale"); supersedes the
> 2026-07-05 engine-ladder snapshot.

## How to read this
- **Nothing shipped gets cut.** crustyimg is already at **0.4.0** with a deep surface
  (optimize/convert/resize/thumbnail/watermark/EXIF/recipes/apply/batch/view/`explain`/
  responsive/diff/info, the full 10-rule `lint`, and two shipped GitHub Actions). The "1.0
  line" below is only about **which *remaining* waves land + get polished + get adopted before
  we call it 1.0** — not a chopping block for anything that exists.
- Sequencing is driven by **adoption leverage and dependencies**, not feature ID. IDs are
  stable; drive by status/priority (the STAGE-008 precedent). Where a wave draws from an
  existing `PROJ-*`, it's noted; formal IDs are (re)assigned at framing time.
- The **Reach & adoption** track (Track B) runs continuously in parallel — it's the binding
  constraint, not a phase. It never waits its turn.
- Evidence/design briefs live in `docs/research/proj-002-*.md`; the PROJ-004/005 breakdowns in
  `docs/roadmap-draft-proj-004-005.md`.

---

## Track A — the build-out, sequenced to 1.0

### Road to 1.0 (in execution order)

| # | Wave | Ships | Delivers | Draws from | Why here |
|---|---|---|---|---|---|
| **1** | **Input reach — AVIF / SVG / RAW (+ opt-in HEIC)** | 0.5.0 | Default, no-system-dep decode reach: **AVIF decode** (rav1d, patent-free) + **SVG rasterize** (resvg) + **RAW embedded full-res JPEG preview** extraction. **HEIC is a feature-gated `heic` build only** (libheif where a system HEVC codec is present) — *never* in the default binary (DEC-052: AGPL wall + HEVC patents). RAW ≠ HEIC: RAW carries a usable embedded JPEG preview; HEIC does not (its thumbnail is HEVC too), so HEIC is real-decode-or-nothing. | **PROJ-009** (framed 2026-07-07; AVIF-decode STAGE-016/SPEC-058 build-ready) + DEC-052 | **AVIF-decode alone justifies keeping it early** — it also feeds the shipped `optimize`/auto-format engine as a new candidate format, beyond input reach. Honest headline: *"AVIF/SVG/RAW just work with no system deps; HEIC in the `heic` build."* NOT "iPhone photos just work." *Long pole is the corpus + fuzzing, not the code.* |
| **2** | **Build + cache + lockfile + `--watch`** | 0.5.x | A `build` workflow over recipes/`apply` with a **content-addressed cache** (incremental rebuild), a reproducibility **lockfile**, and `--watch`. "A Makefile for images, verifiable." | PROJ-007 (determinism) | Best daily-driver feature and **near-ideal agent work** (turborepo/Bazel-shaped, deterministic, clear pass/fail). `--watch` is table-stakes DX for the frontend audience. |
| **3** | **WASM core + demo page** | 0.6.0 | Compile the (already I/O-agnostic) core to **WASM**, ship an **npm-packaged library**, and a **squoosh.app-style client-side demo page**. | PROJ-008 (WASM seam) | Demo = **highest-ROI marketing artifact** (zero-install "try it," inherently shareable). npm lib = a sharp no-native-binary alternative. **This is where the "watch it just work" moment lives** — in-browser AVIF/SVG conversion; **time the Show HN here.** Must stay **client-side** (no backend) to honor the no-service/no-CDN guardrail. |
| **4** | **Web-asset manifest + placeholders + favicon** | 0.6.x | Path-keyed **manifest** Sink (`responsive`/`apply --manifest`) + **placeholders** (`blurDataURL` + thumbhash/blurhash) + **dominant color** + **favicon** multi-size set. | **PROJ-005** (unchanged) | The interface any SSG/build consumes. Sequenced after install momentum; go deep in **Eleventy first** (`eleventy-crustyimg`), then an Astro image service. HEIC-in-the-manifest is a real edge (needs #1). |
| **5** | **Geometry (non-smart)** | 0.6.x | **crop** (rect/gravity/aspect) + **rotate/flip/trim/pad** — and *only* those. | PROJ-006 (geometry half) | Code-bound, table-stakes; completes the local toolchain and makes 1.0 feel finished. |

**→ Cut 1.0 here.** The complete, trustworthy local toolchain: the shipped engine + broad input
reach + build/cache/lockfile + manifest + geometry + a library & demo page, with distribution
live and a **CLI-quality pass** (Track B) folded in. Coherent, defensible, every piece something
people want and that we can make *excellent*.

### Post-1.0 — sequenced below the line on purpose (deferred, not dropped)

| Wave | Ships | Status / gate | Notes |
|---|---|---|---|
| **Engine-backed lint rules** (STAGE-014: `legacy-format`, `excessive-jpeg-quality`, `indexed-png-opportunity`) | post-1.0 | **Demand-gated.** `lint` already shipped as a 10-rule catalog in 0.4.0; further lint *breadth* waits for an actual adoption signal (Action/Eleventy users asking). | Cheap to finish (reuses the shipped engine, no new deps) — but "cheap to build" is exactly why not to over-invest in the least-validated surface. Finish *if pulled*, not by default. See `projects/PROJ-004-image-lint/stages/STAGE-014-engine-backed-rules.md`. |
| **Optimization planner** (PROJ-003) | post-1.0 | **Defer / fold.** A `plan`/dry-run (byte deltas before writing) + combined goal object; already slimmed to orchestration, "may fold into 002/004." | Not an install-and-share moment. Keep the `--dry-run`/`plan` idea; it can ride along a feature wave rather than headline its own. |
| **Smart-crop / auto-color / redaction** | post-1.0, **beta** | **Judgment-bound.** The code is an afternoon; the *tune-until-it-looks-right-on-a-diverse-corpus* loop runs on human eyeballs, not compute. | Ship behind a clearly-labeled **beta** flag; redaction is **manual-region** (auto-detect would break the no-ML identity). Don't let it near the 1.0 quality bar. |
| **Format completion** — AVIF-decode / perceptual AVIF, jpegli, permissive quantizer (indexed/lossy PNG, better GIF) | post-1.0 | **Upstream-gated.** rav1d maturity is exogenous; the jpegli license question is a *decision*, not a commit. | Do it when the blockers clear, not on a calendar. Unblocks `indexed-png` as a real fix. |
| **`serve` (self-hostable image server)** | 2.0 | **Demand-gated.** | Weakest market for a pure-Rust pitch, least-stable surface, a different buyer. Let real pull decide it exists at all. Commercial rationale is private (`business/`, gitignored). |
| **Opt-in intelligence / new frontends** (PROJ-008 stretch: AI super-res, face-aware crop, JPEG XL when browsers land, RAW Tier-2) | 2.0+ | Frontier, opt-in, never on the default path. | Pure-Rust preferred; AI/C bits feature-gated and BYO. |

---

## Sequencing rationale (adoption-first, 2026-07-07)

The reorder came from a velocity-aware reconciliation. The core observations:

- **Agent speed collapses code-bound work but barely touches the rest.** The deterministic,
  well-specified waves (input-reach code, build/cache/lockfile, manifest/placeholders/favicon,
  non-smart geometry) are close to ideal agent work and can go fast. The **judgment-bound**
  (smart-crop/auto-color — tuning on eyeballs), **corpus-bound** (HEIC/RAW breadth — the long
  pole is real-world files you must assemble), and **upstream-gated** (rav1d/jpegli) work does
  *not* get faster because we can generate code faster.
- **Cheap generation *raises* the review bar on untrusted-input parsers.** Rust kills the
  memory-safety class; it does not kill a logic bug in a fast-generated HEIC/RAW parser fed a
  crafted file. Input-reach ships with a **fuzzing + real-corpus gate**, not just tests — this is
  the promise the whole identity rests on (`docs/territory.md`, safe-on-untrusted).
- **Adoption is the binding constraint, and velocity can't compress it.** We could ship every
  wave in weeks and still have zero users. So point the velocity at making the *right* 1.0
  airtight and getting it in front of people — **not** at maximizing release count. When building
  is cheap, the scarce discipline is **saying no**; the failure mode flips from "didn't ship
  enough" to "shipped a wide, shallow surface faster than we could make any of it excellent."
- **Breadth is the pitch — but only breadth in things people want, each of it good.** "Does
  everything, one static binary" sells; a wide-but-shallow surface hands every reviewer something
  to dunk on. Input-reach/optimize/convert/build breadth sells; *more* lint catalog nobody asked
  for, or a deterministic smart-crop visibly worse than the ML tools, does not.

**What moved and why:**
- **Input reach jumped from 0.7.0 to first** — but reframed after the HEIC spike (DEC-052,
  `docs/research/heic-input-reach-spike.md`). The default hook is **AVIF/SVG/RAW** (all permissive,
  patent-clean, no system deps); **HEIC is opt-in only** because the mature pure-Rust decoders are
  AGPL *and* HEVC carries patent exposure regardless of code license. AVIF-decode alone earns the
  early slot (it also feeds the shipped optimize engine), so the wave stays first even though the
  "iPhone photos just work" headline moved to the in-browser demo (Wave 3).
- **build/cache/lockfile and the WASM core + demo split out of PROJ-007/008 and moved up** — the
  demo page is the marketing artifact the Track-B funnel finally has to point at.
- **Lint expansion (STAGE-014) dropped below the 1.0 line.** The catalog already shipped in 0.4.0
  (we don't rug-pull it), but forward investment in the least-validated surface is gated on
  adoption signal — the sharpest live lesson from the reconciliation.
- **The planner (PROJ-003) folds/defers**, and **smart-crop/auto-color/redaction demote to a
  post-1.0 beta** — judgment-bound polish doesn't belong at the 1.0 bar.
- **`serve` stays 2.0 and demand-gated**; the WASM core is more valuable as a **library** than as
  a server (and is the same seam the private commercial notes depend on).

None of these strategy calls move because we can build faster. If anything, removing the build
constraint makes "*which* of this should exist" the more important question, not the less.

---

## Track B — Reach & adoption (continuous, the binding constraint)

Not a phase and not gated on the engine ladder — this is how the territory gets *held*, not just
claimed (`docs/territory.md`). It runs the entire time; if we build every wave and skip this we
have an impressive unused tool.

- **Distribution — always-on, never pausing.** Largely stood up already (v0.4.0 live on
  crates.io + Homebrew + GitHub Releases; `setup-crustyimg` + `crustyimg-action` shipped and
  tagged `v1`). The ongoing part is the part velocity can't compress: a one-liner install across
  `cargo binstall` / `brew` / `docker run` / `npx`, and **answering the questions people already
  ask** ("sharp install failed," "how do I batch-convert HEIC from the CLI" — which #1 above
  directly answers). This never waits its turn.
- **CLI quality & trust pass** (folds into 1.0). Close the concrete gaps: `NO_COLOR` +
  `--color=auto|always|never`, examples in `--help` (`after_help`), man pages (`clap_mangen`),
  fix-suggesting "did you mean" errors, the `-q` short-flag collision, broaden `--json` to batch
  commands, `--dry-run`/`-n`, and **SBOM + signed releases** (cosign/Sigstore via GH OIDC — the
  2026 trust baseline). Refs: clig.dev, no-color.org, clap_mangen/clap_complete.
- **Pre-1.0 hardening gates (must-do before cutting 1.0).** Untrusted-input decoders that shipped
  with a fuzz target must have it actually *run* before 1.0:
  - **`fuzz/avif_decode`** — the AVIF parse+decode target (SPEC-058/PR #65) ships but was **not run**
    (no nightly/cargo-fuzz in the build+verify envs). Run `cargo +nightly fuzz run avif_decode -- -runs=100000`
    (or wire it into CI/OSS-Fuzz) before a 1.0 release. The single real residual risk on the AVIF
    untrusted-binary path — mitigated today by corrupt-input + cap unit tests, but not fuzzed.
  - **`fuzz/svg_decode`** — the SVG parse+rasterize target (SPEC-060/PR #66) ships but was **not run**
    (no nightly/cargo-fuzz in the build+verify envs). Run `cargo +nightly fuzz run svg_decode -- -runs=100000`
    (seed from `tests/fixtures/svg`) before a 1.0 release. Mitigated today by malformed-input + oversize-cap
    + external-ref-refused tests, but not fuzzed — the residual risk on the SVG untrusted-text path.
  - **`fuzz/raw_preview`** — the RAW embedded-preview scan+decode target (SPEC-061/PR #67) ships but was
    **not run** (no nightly/cargo-fuzz in the build+verify envs). Run `cargo +nightly fuzz run raw_preview -- -runs=100000`
    (seed from `tests/fixtures/raw`) before a 1.0 release. Mitigated today by bounded-candidate + oversize-cap
    + no-preview-typed-error tests, but not fuzzed — the residual risk on the RAW untrusted-binary path.
  - Extend this list as the opt-in HEIC decoder (STAGE-019) lands.
- **`lint <raw>` follow-up (own spec, post-STAGE-018).** `lint` decodes via `Image::from_bytes`
  (`src/lint/mod.rs:210`), so it bypasses the RAW extension-routing that `Image::load`/`Image::decode_path`
  do — `lint` on a `.nef` path does not read the embedded preview. NOT a SPEC-061 claim (its reach was
  optimize/convert/info/resize/batch), so left out of scope; now that `Image::decode_path` exists it would
  be a small change. Frame a spec if `lint <raw>` is ever wanted.
- **Proof & distribution polish.** `BENCHMARKS.md` (cross-tool, honest equal-quality rule) · a
  real docs site + quickstart + recipe cookbook + the "why crustyimg" page + README badges · the
  **client-side demo page** (Wave 3) as the flagship "try it" artifact.
- **Ecosystem / SSG integration.** Ship the generic `--manifest` (Wave 4) + docs first (unlocks
  all six SSGs at once), then the two native plugins that showcase crustyimg best —
  **`eleventy-crustyimg`** (async shortcode, the no-Sharp `eleventy-img` analog) and an **Astro
  custom image service**. Ranked targets: Eleventy > Astro > Hugo > Next/Vite > Zola > Jekyll.
  Manifest contract: **key by source path**, self-contained per entry (sandboxed SSGs can't
  re-invoke the binary; see `docs/recipes.md §9`).
- **Upstream contributions (ecosystem citizenship).** On-brand for "assemble the pure-Rust imaging
  frontier": contribute **pure-Rust AVIF decode to `image-rs`** ([#2621](https://github.com/image-rs/image/issues/2621),
  a trivial PoC-backed PR) — benefits the whole ecosystem AND gives crustyimg a future migration off
  its direct `re_rav1d` dep. Tracked in `docs/contributions/upstream-image-rs-avif-decode.md`
  (proposed; parallel to PROJ-009, blocks nothing).
- **External users.** Find **design partners**, gather feedback, seed community (the abandoned
  `@squoosh/cli` audience + no-Node/Makefile shops are the warm leads).
- **Commercial direction (deferred, tracked privately).** The **CLI + Actions are adoption, and
  stay free.** Any revenue path is a *separate, deferred product* for a paying audience that
  reuses the same engine (the WASM/HTTP frontend seam is the enabler) — kept out of this public
  repo in private, gitignored `business/` notes. Guardrails (`docs/territory.md`): never gate the
  free engine, never rug-pull, never become a CDN.

---

## Future utilities backlog (small, unscheduled)

Low-priority CLI-utility ideas that don't belong to a wave; each gets framed only if pulled.

- **`crustyimg ls` (LOW priority — not built, not fully specced).** A read-only, format-aware
  `ls`: list the image files in the current directory (or given paths/globs) — exactly the set the
  batch commands would operate on — by reusing the shipped `source::resolve` (`src/source/mod.rs`,
  the dir/glob fan-out + image-extension allow-list). Optionally show compact per-file info
  (dimensions/size/format, like a terse `crustyimg info`), with `--json` for scripting. It is a
  **discovery/utility** command, **not** a lint feature: its value is answering "what will a batch
  run touch, and what are these files?" before you run one. A future session can spec it against
  `source::resolve` + the existing `info` probe + the `--json` writer; keep it read-only (never
  writes, never decodes pixels beyond a header probe).

---

## Relationship to existing docs
- **Reframes** the 2026-07-05 engine-ladder ordering (this doc supersedes it) and the crop-led
  `docs/backlog.md` (geometry → Wave 5 + the smart-crop beta; format/RAW → input-reach +
  format-completion).
- **Ladders up to** `docs/territory.md` (the space we're claiming) and its scope discipline.
- **Grounded in** `docs/research/proj-002-findings.md` + `proj-002-design-*.md`.
- **Snapshot of what's built:** `docs/moat.md`.

*Refine this doc as research lands and as each wave ships and the next is framed for real.*
