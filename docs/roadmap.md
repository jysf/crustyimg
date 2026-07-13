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
| **2 ✅** | **Build + cache + lockfile + `--watch`** — **SHIPPED (PROJ-007, closed 2026-07-12)** | 0.5.x | A `build` workflow over recipes/`apply` with a **content-addressed cache** (incremental rebuild), a reproducibility **lockfile** (`--check`/`--frozen` CI drift gate), and `--watch`. "A Makefile for images, verifiable." **Delivered in full across 5 stages / 9 specs (SPEC-063..071)**, including a closing hardening & security sweep (STAGE-024): a threat-model note, the pre-1.0 decoder fuzz gate **actually run**, and a peak-decode-memory cap. | PROJ-007 (determinism) | Best daily-driver feature and **near-ideal agent work** (turborepo/Bazel-shaped, deterministic, clear pass/fail). `--watch` is table-stakes DX for the frontend audience. |
| **3 ← NEXT** | **WASM core + demo page** | 0.6.0 | Compile the (already I/O-agnostic) core to **WASM**, ship an **npm-packaged library**, and a **squoosh.app-style client-side demo page**. | PROJ-008 (WASM seam) | Demo = **highest-ROI marketing artifact** (zero-install "try it," inherently shareable). npm lib = a sharp no-native-binary alternative. **This is where the "watch it just work" moment lives** — in-browser AVIF/SVG conversion; **time the Show HN here.** Must stay **client-side** (no backend) to honor the no-service/no-CDN guardrail. |
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
| **PROJ-007 hardening maintenance backlog** (STAGE-024 deferred tail, 8 items) | post-1.0 | **Triaged + deferred 2026-07-12.** None gate a 1.0 "trust it in CI on untrusted files" claim. | canonicalize-contain-out (accept+documented write-escape residual; own spec w/ opt-out for symlinked out dirs) · pre-decode format sniff (fails-closed today) · full-pipeline peak envelope · `--max-pixels` cap-raise dial (revisit trigger live) · lint decode-seam audit + not-inspected rule · unusual-filename `.to_str()` sweep · cache-key build-profile · orphan-output prune. Full list + per-item rationale in `projects/PROJ-007-reproducible-build/stages/STAGE-024-hardening-and-security-sweep.md`. Pull individually if usage surfaces the need. |
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
  with a fuzz target must have it actually *run* before 1.0. **✅ RUN — SPEC-069 (DEC-062);** run
  record + repeat recipe in [`docs/research/proj-009-fuzz-run.md`](research/proj-009-fuzz-run.md),
  gate one-liner `just fuzz <target>`. The durable per-PR guard is now
  `tests/fuzz_regressions.rs` (regressions + `fuzz_corpus_never_panics` smoke), run by ordinary
  3-OS `cargo test`. Remaining pre-1.0 items are the two upstream-`avif-parse` follow-ups below +
  OSS-Fuzz (optional).
  - **`fuzz/avif_decode`** — **RUN (SPEC-069).** Surfaced 3 issues, all in `avif-parse` 2.1.0's
    container parser (none in `re_rav1d` / our YUV glue): a `check_parser_state` `debug_assert!`
    panic and a top-level box-size-bomb OOM — **both fixed at our boundary** (`box_sizes_fit` +
    `catch_unwind` + `frame_size_limit`, regressions committed); and a nested meta-box
    `TryVec::with_capacity` over-allocation — **documented upstream** (not boundary-fixable without
    vendoring avif-parse; mitigated by avif-parse's fallible alloc). **Pre-1.0 residual:** upstream
    `avif-parse` container robustness (report/patch/replace).
  - **`fuzz/svg_decode`** — **RUN (SPEC-069), CLEAN.** 2.39 M runs / 601 s, no crash. Residual risk
    on the SVG path closed for this budget.
  - **`fuzz/raw_preview`** — **RUN (SPEC-069).** 1.81 M runs / 602 s, no panic/crash. It surfaced one
    memory-amplification residual (**F-RAW-1**): a crafted embedded JPEG (SOF 16384×9776) peaked
    ~1.9 GB while passing every DEC-034 cap (`max_alloc` bounds single-alloc, not peak) — a transient
    memory DoS, pre-existing on the general `.jpg` path too. ✅ **CLOSED by SPEC-070 / DEC-063:** a
    declared-pixel budget (64 Mpix = a 1 GiB peak budget ÷ the measured ~4× amplification) is now
    enforced pre-decode at every seam; the reproducer's peak RSS drops 1.93 GB → 8.7 MB (exit 1, not
    exit 0) and it has graduated into the always-on corpus smoke. F-AVIF-3 (a *parse*-stage
    over-allocation, before dims are known) remains open. See the run record.
  - **`fuzz/heic_decode`** — **RUN best-effort (SPEC-069)** with system libheif `1.23.1` + ASAN. The
    libheif decode path was exercised (~107 k execs) with **no HEIC/libheif finding**; both runs were
    bounded by the shared `from_bytes` AVIF-first dispatch reaching `avif-parse`'s documented issues.
    HEIC decode is a **C** library (libheif/libde265) — pinned at the `v1_17` API floor, libheif's own
    `set_security_limits` is unreachable, so our DEC-034 handle-dimension pre-check is the *only*
    bound. **Pre-1.0 residual:** a dedicated HEIC-only fuzz entry (bypass the AVIF sniff) + bump to
    `v1_19` for `heif_context_set_security_limits`.
- **HEIC `heic`-feature follow-ups (post-STAGE-019, own specs if pulled).** (1) A **stride-padding test**
  with an odd-width HEIC fixture — the committed 64×48 fixture returns `stride == row_bytes`, so the
  row-padding copy path (proven correct at verify on a 67×45 image, stride 208 vs 201) is untested; commit
  a `sips`-made odd-width fixture. (2) **Windows `heic`** (libheif via vcpkg) — the CI job is macOS+Linux
  only today. (3) Bump the API floor to **`v1_19`** and wire `heif_context_set_security_limits` once a
  newer libheif is broadly available (drops the "pre-check is the only bound" caveat). (4) **HEIC alpha**
  coverage (the RGBA path exists but the fixture is opaque).
- **`lint <raw>` follow-up (own spec, post-STAGE-018).** `lint` decodes via `Image::from_bytes`
  (`src/lint/mod.rs:210`), so it bypasses the RAW extension-routing that `Image::load`/`Image::decode_path`
  do — `lint` on a `.nef` path does not read the embedded preview. NOT a SPEC-061 claim (its reach was
  optimize/convert/info/resize/batch), so left out of scope; now that `Image::decode_path` exists it would
  be a small change. Frame a spec if `lint <raw>` is ever wanted.
- **WASM core (STAGE-025 / SPEC-072) follow-ups — own specs if pulled.** SPEC-072 shipped the
  wasm build seam (PR #80, DEC-064): the pure engine runs a real decode→transform→encode
  round-trip in-browser with no backend (SVG + PNG/JPEG/GIF/WebP), AVIF *decode* gated out
  (returns a typed error), native builds unaffected. Carries: (1) **a shared `optimize` engine
  seam** both `cli` and `wasm` call, so the multi-candidate `pick_winner` solve isn't CLI-only
  (wasm's `optimize` currently takes the format shortlist's first candidate — honest, not
  best); (2) **a wasm CI job** — today only a local `just wasm-test` floor guards the wasm
  build (mirrors the fuzz-gate CI decision, DEC-062); (3) **bundle size = SPEC-074's brief**
  (now **1.52 MB brotli** with AVIF encode; prime levers = `ssimulacra2`, the resvg text stack,
  unused `image` codecs, and the deferred `crustyimg-core` crate split, DEC-064); (4) a
  **`not(wasm32)` cfg-alias** once a second default-ON feature carries a native-only dep (only
  `display` today). **SPEC-073 answered the AVIF-on-wasm question (DEC-065): encode is IN** —
  `rav1e` compiles AND runs in a wasm VM, so the shipped artifact is built `--features avif`
  and PNG → AVIF works in the browser (+345 KB brotli, +28%) — while **decode stays deferred**
  (`re_rav1d` still won't build for wasm32; the browser's own `createImageBitmap` reads `.avif`
  for the demo page). Carries: STAGE-027 must treat AVIF encode as a slow, worker-thread
  operation (rav1e runs serial — no wasm threads) and must decode `.avif` INPUTS itself.
  Also carries a small **docs-cleanup** (own spec): two stale native doc strings predating
  SPEC-058's native AVIF decode — `docs/api-contract.md` ("reading an `.avif` fails") and
  `quality::supports_perceptual_quality`'s doc comment ("no decoder built") — both wrong on
  native today, neither a SPEC-073 defect.
- **WASM core (STAGE-025) — COMPLETE 2026-07-12** (SPEC-072 seam + SPEC-073 AVIF encode + SPEC-074
  size, DEC-064/065/066): the pure engine runs in-browser (SVG + PNG/JPEG/GIF/WebP decode+encode,
  AVIF encode), no backend, **1.33 MB brotli**. Two follow-ups it hands the rest of PROJ-008:
  (1) **a wasm CI job MUST build through `just wasm-build`** — the size profile lives in the
  recipe's env vars (not `[profile.release]`, which native shares), so a bare `cargo build
  --target wasm32` silently ships **+109 KB** heavier; measured, real footgun. (2) **`optimize(_,
  "webp")` on wasm returns *lossless* WebP** (~320 KB vs a 44 KB JPEG) — there's no lossy-WebP
  encoder in the wasm feature set, so the perceptual path silently isn't available for WebP; resolve
  before **STAGE-027**'s demo offers WebP as an output. STAGE-027 also inherits: rav1e runs *serial*
  on wasm (Web Worker + progress) and must decode `.avif` inputs page-side via `createImageBitmap`.
- **npm library (STAGE-026 / SPEC-075) — SHIPPED 2026-07-13** (PR #84, DEC-067): the WASM core is
  an installable, typed npm package **`crustyimg-wasm`** (`--target web`, one artifact, lockstep
  version, publish gated) that runs client-side with **no native addon / no lifecycle script / zero
  deps** — the "sharp without the native addon" artifact, proven by a pack→install→run smoke.
  Carries: (1) the **bare `crustyimg` npm name is deliberately unclaimed** for a future
  npx-distributed CLI (esbuild/esbuild-wasm precedent — don't re-litigate); (2) the owed **wasm CI
  job must ALSO run `just wasm-npm-smoke`** (the package is proven on one Mac until CI runs it), on
  top of building through `just wasm-build`; (3) **SPEC-076** = the live `npm publish`, by hand, on
  maintainer approval (outward-facing/irreversible — the tooling deliberately can't reach it).
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
