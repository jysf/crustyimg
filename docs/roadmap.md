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
| **1 ✅** | **Input reach — AVIF / SVG / RAW (+ opt-in HEIC)** — **SHIPPED (PROJ-009)** | 0.5.0 | Default, no-system-dep decode reach: **AVIF decode** (rav1d, patent-free) + **SVG rasterize** (resvg) + **RAW embedded full-res JPEG preview** extraction. **HEIC is a feature-gated `heic` build only** (libheif where a system HEVC codec is present) — *never* in the default binary (DEC-052: AGPL wall + HEVC patents). RAW ≠ HEIC: RAW carries a usable embedded JPEG preview; HEIC does not (its thumbnail is HEVC too), so HEIC is real-decode-or-nothing. | **PROJ-009** (framed 2026-07-07; AVIF-decode STAGE-016/SPEC-058 build-ready) + DEC-052 | **AVIF-decode alone justifies keeping it early** — it also feeds the shipped `optimize`/auto-format engine as a new candidate format, beyond input reach. Honest headline: *"AVIF/SVG/RAW just work with no system deps; HEIC in the `heic` build."* NOT "iPhone photos just work." *Long pole is the corpus + fuzzing, not the code.* |
| **2 ✅** | **Build + cache + lockfile + `--watch`** — **SHIPPED (PROJ-007, closed 2026-07-12)** | 0.5.x | A `build` workflow over recipes/`apply` with a **content-addressed cache** (incremental rebuild), a reproducibility **lockfile** (`--check`/`--frozen` CI drift gate), and `--watch`. "A Makefile for images, verifiable." **Delivered in full across 5 stages / 9 specs (SPEC-063..071)**, including a closing hardening & security sweep (STAGE-024): a threat-model note, the pre-1.0 decoder fuzz gate **actually run**, and a peak-decode-memory cap. | PROJ-007 (determinism) | Best daily-driver feature and **near-ideal agent work** (turborepo/Bazel-shaped, deterministic, clear pass/fail). `--watch` is table-stakes DX for the frontend audience. |
| **3 ✅** | **WASM core + demo page** — **SHIPPED (PROJ-008, closed 2026-07-26); 0.6.0 tagged, demo live, `crustyimg-wasm` published** | 0.6.0 | Compile the (already I/O-agnostic) core to **WASM**, ship an **npm-packaged library**, and a **squoosh.app-style client-side demo page**. | PROJ-008 (WASM seam) | Demo = **highest-ROI marketing artifact** (zero-install "try it," inherently shareable). npm lib = a sharp no-native-binary alternative. **This is where the "watch it just work" moment lives** — in-browser AVIF/SVG conversion; **time the Show HN here.** Must stay **client-side** (no backend) to honor the no-service/no-CDN guardrail. |
| **4 ← NEXT** | **Web-asset manifest + placeholders + favicon** | 0.6.x | Path-keyed **manifest** Sink (`responsive`/`apply --manifest`) + **placeholders** (`blurDataURL` + thumbhash/blurhash) + **dominant color** + **favicon** multi-size set. | **PROJ-005** (unchanged) | The interface any SSG/build consumes. Sequenced after install momentum; go deep in **Eleventy first** (`eleventy-crustyimg`), then an Astro image service. **The maintainer's own Eleventy photo blog is the dogfood testbed** — a real site, real photos, a real build, which is what a plugin has to be designed against rather than a toy fixture. HEIC-in-the-manifest is a real edge (needs #1). |
| **5** | **Geometry (non-smart)** | 0.6.x | **crop** (rect/gravity/aspect) + **rotate/flip/trim/pad** — and *only* those. **Every op must be a recipe step, not only a CLI flag**, and **rotate takes any angle** (a 2.5° straighten, with a choice of crop-to-fit or expand-the-canvas), not only quarter turns. | PROJ-006 (geometry half) — **no project directory exists yet**, so framing starts from this row. **First consumer: the ddphotos edit page** (`integrations/ddphotos/`, merged in #218), which today can only resize and watermark because recipes have no crop or rotate. | Code-bound, table-stakes; completes the local toolchain and makes 1.0 feel finished. **The two additions come from the editor, not from the CLI:** a page that shows a live preview must emit the same edit as a recipe the CLI can re-run on the original, so a geometry op that exists only as a flag cannot be offered there at all. Straightening is the first thing a photo owner reaches for after crop, and it is still "rotate" — no judgment involved, so it stays inside this non-smart wave. ⚠ `resize --fill` already scale-and-crops to a box; that is not a crop op and does not cover a free rectangle or an aspect preset. |

| **6** | **The browser tool + the recipe playground** | post-0.6.0 (adoption track — can land either side of the 1.0 cut) | Two halves of one idea. (a) Treat the browser demo as a **real tool**, not only a showcase: presets, batch, a byte budget, remembered settings. (b) **Formalize the recipe format and give it a playground** — a documented op reference/schema, a `validate(recipe_toml)` wasm export returning *structured* errors, and a demo page where you edit a recipe, validate it live, run it on your own photo, and copy the exact TOML into CI. | PROJ-008's wasm surface (`transform` already accepts arbitrary recipe TOML; recipes are already a versioned `[[step]]` pipeline) | **The validator in the browser is the CLI's actual validator compiled to wasm** — not a JS reimplementation, so it cannot drift. Nobody else can honestly say "what you just watched run is exactly what your CI will do." Squoosh is archived, and the upload-based tools can't make the privacy claim at all. **A playground is an on-ramp, not a destination** — you leave with a file to run elsewhere — so it strengthens the CLI rather than competing with it, and it feeds #4's manifest and the SSG plugins, which consume recipes. **Do NOT invent new syntax:** TOML is already parseable, diffable, versioned and free. Error quality *is* the feature — "invalid recipe" is worthless, "step 2: unknown op `reisze` — did you mean `resize`?" is the product. |

| **6b** | **The ddphotos editor saves the edit, not the pixels** | post-0.6.0 (adoption track, alongside #6) | Today the edit page (`integrations/ddphotos/`) can only **download** a copy: a ddphotos site is static files with no server, so the page has nowhere to write, and a downloaded copy put back into the album bakes the edit over the original. Instead the page saves a **recipe sidecar** next to the original (`IMG_1234.jpg.crustyimg.toml`), and a step the owner runs **before** ddphotos' `photogen` applies every sidecar to its original; the owner then deploys exactly as today (Cloudflare Pages, rsync to Apache/nginx, or S3 + CloudFront). Saving comes in tiers: **(1)** on the owner's machine (`./ddphotos run`), the browser's File System Access API writes the sidecar straight into the album folder — no server, Chromium-only, other browsers fall back to downloading the sidecar; **(2)** for a site kept in GitHub and built by Cloudflare Pages, the page commits the sidecar through GitHub's API and the push triggers the rebuild. | #218's edit page and its CLI recipe; `build`/`apply` (Wave 2); #6's recipe validator; Wave 5's geometry ops | **The recipe is already the product's core idea**: an edit that is reproducible, always made from the original, reviewable in git, and re-runnable by the CLI. Saving pixels throws all of that away; saving the recipe keeps it, and needs **no change from ddphotos** — every piece lives in its `config/` directory or in crustyimg. ⚠ **Sidecar discovery is new surface**: `build` reads a manifest today (`crustyimg.build.toml`, whose targets bind a glob to one recipe each), not one recipe per photo, so how the pre-`photogen` step finds and names them has to be designed, not assumed. ⚠ **Tier 2 needs sign-in and a CI build that can run `photogen`** (Go + libvips; ddphotos has a Docker mode for it) — prove tier 1 first. ⚠ **Declined: an upload endpoint** (e.g. a Cloudflare Worker + R2) — it means operating a server for someone else's static site. **The one thing only ddphotos can add** is an "Edit" button in its lightbox; today visitors reach `/edit/` by URL. A separate test-bed repo now builds a real ddphotos gallery with the editor installed and re-tests it per crustyimg commit; tier 1 is proved there before anyone contacts ddphotos' author. **First step, before any saving: a prebuilt download of the edit page** (proposed 2026-10-29; no stage owns the editor yet, so this is a proposal, not a decision). Installing the page today means `just ddphotos-edit-install`, which needs just, rustup's stable toolchain with the wasm target, and wasm-pack. [ddphotos-app](https://github.com/dougdonohoe/ddphotos-app), ddphotos' desktop front end (Java, AGPL), is how non-technical owners run it: it needs only Docker, and it does no image processing of its own, so crustyimg does not duplicate it. Its users can never run the install command. Ship the built `edit/` folder (page, worker and the slim `pkg-slim/` vendor files, about 4 MB unpacked) as a zip on each GitHub release, to unzip into `config/static/`. Because the app ends at `ddphotos build`, which copies that folder, the page then works on an app-built site with no change from ddphotos. ⚠ `release.yml` is generated by cargo-dist, so the zip needs dist's custom-job hook or its own workflow, not a hand edit. ⚠ Unverified: where the app keeps a site's `config/` folder, and that the page works on a site the app built; drive both in the test bed before claiming it. **Three more proposals (2026-10-29, from reading ddphotos' `pkg/photogen`; no stage owns them):** **(a) Follow photogen's own sidecar convention.** photogen already reads `IMG_1.jpg.photogen.json` beside a photo, named after the file's full name for metadata the file cannot carry, so `IMG_1.jpg.crustyimg.toml` follows a pattern ddphotos already accepts. Design sidecar discovery to match it, and cite it when proposing the pre-`photogen` step. **(b) Decide public versus owner-only before the pitch.** Installed as today, `/edit/` ships with the public site, because the export copies all of `config/static/`, so every visitor gets a make-a-copy tool. That exposes nothing new (the 1600 px copy is downloadable anyway), but the tiers above are owner tools, so offer an install that runs on `./ddphotos run` (the app's Run tab) and is left out of the export, or say plainly how to leave it out. **(c) A measured quality report on ddphotos' own output, as evidence rather than code.** photogen writes every photo at one fixed WebP quality (600 px grid at q85, 1600 px full at q90). Run crustyimg's similarity search over the test bed's export and report, per photo, how much smaller each file could be at the same perceived quality. This asks ddphotos to change nothing, and is something concrete to bring its author. ⚡ It is a measurement whose answer may be "not much": do not promise savings in advance. ⚠ **Do not pitch crustyimg as photogen's replacement.** photogen already decodes HEIC and AVIF, converts every colour profile to sRGB with bundled P3 and Rec.2020 profiles (`color.go`, which crustyimg cannot do yet: the STAGE-052 item), and strips metadata. crustyimg's ground here is the in-browser editor, the recipe, and measurement. Lower priority: editing photos in password-protected albums, which the site's own JavaScript already decrypts with the password. |

| **7** | **Grading — the `.cube` LUT op** | post-1.0 (**not gated**; the infrastructure shipped ahead of it) | A `lut` operation taking a `.cube` file and a strength, applied **in linear light at the source's own bit depth** — reachable from a recipe and the CLI, with the `.cube` file's *content* in the build cache key. | **PROJ-015 / STAGE-055 / SPEC-136** (framed 2026-10-28, re-homed from PROJ-011 the same day), riding SPEC-128's asset seam + SPEC-129's asset hashing + SPEC-121/122's depth and linear light | **The differentiator is the reproducible build, not the LUT.** *"The LUT inside a reproducible build — the file's hash becomes part of the cache key, so changing the grade invalidates exactly the affected outputs and `build --check` catches an accidental grade change in review."* Nothing else in this space occupies that, and it is what unlocks **brand consistency as a build gate** — the one commercial angle in `docs/feature-set-triage-2026-08.md`'s set. **It sits here, below the line, because a grade is not table-stakes for "complete local toolchain"** — but it is the cheapest remaining capability on this list and the only unblocked one, because **four shipped specs each named it as their second consumer** and built to its shape (DEC-100 states the asset seam was generalized "enough for a SECOND parameter-rich, asset-bearing op (the `.cube` LUT …)" and proves it with a fixture in `tests/registry_seam.rs`). Budget **250–350 lines**, **zero new dependencies** — measured against `lut-cube` 0.2.0's 329, which is declined on a supply-chain signal (its only tests hardcode the author's local DaVinci Resolve paths), not on its licence (plain MIT). ⚠ **Its old blocker is cleared, not forgotten**: a `.cube` applied in a non-linear pipeline is baked against the wrong transfer function — the 8-bit half became conditional on an 8-bit source at SPEC-121, and the transfer-function half closed when SPEC-122 shipped the linear machinery. ⚠ **The engine stays crustyimg's** — reviewed 2026-10-28 against a sibling proposal to host it in `irradiance` and declined, because `single-image-library` is a blocking constraint and a RAW decoder must not become a second pixel-processing library. **This row exists because the item was previously homed nowhere**: it lived only under `docs/backlog.md`'s "workhorse items with no home", which `just backlog` does not read, so no command in the repo could surface it. |

| **8** | **RAW Tier-2 — develop the sensor, not the preview** | post-1.0 (**not gated**; framed, with the library it consumes at 0.1.0) | An off-by-default `raw-develop` feature that hands a DNG to **`irradiance`** — a permissive, pure-Rust, zero-runtime-dependency RAW develop library — and gets real developed pixels plus container-true metadata instead of the camera's embedded JPEG. The embedded preview stays the default and becomes the fallback. | **PROJ-016 / STAGE-056** (framed 2026-10-28), implementing irradiance's cross-repo `PROJ-001 / STAGE-004`; governed by **DEC-107**, amending DEC-055 and extending DEC-089 | **This row exists because the item was mis-filed as frontier work.** "RAW Tier-2" sat inside the 2.0+ *"Opt-in intelligence / new frontends"* row beside AI super-res and face-aware crop. It is not that kind of item: it is framed, reviewed against this repo's HEAD, and needs one feature flag. **DEC-055 deferred it on a false dichotomy** — *"it needs LGPL `rawler` … or a multi-month from-scratch effort"* — which treated those as the only routes. The from-scratch effort was made, **outside this repo**, as a permissively-licensed general-purpose library verified bit-exact against an independent implementation; so the size estimate was right and the assumption that such work could only land here or under copyleft was wrong. ⚠ **Narrow on purpose:** `irradiance` 0.1.0 is **monochrome DNG only** — no CFA, so no demosaic, white balance or colour matrix exists to call. Colour RAW is a dependency version bump once irradiance's own PROJ-002 lands, not a crustyimg project. ⚠ **Gated on a crates.io publish, not on code**: an optional `path` dependency with no `version` makes `cargo publish` fail for the *whole crate*, so the committed manifest waits for irradiance 0.1.0 to be published. ⚠ **The default build does not change** — that is a success criterion, asserted on a real RAW fixture, not a caveat. **It also fixes a defect with no other cure:** `docs/backlog.md` records that RAW loses 100% of EXIF and that RAW orientation is never read, and that the measured DNG's embedded previews carry **no APP1 at all** — so threading the preview's EXIF forward cannot work even in principle. The container's IFD0 is the only source, and a DNG container parser is what reads it. |

| **9** | **Provenance that survives a screenshot** | post-1.0 (research-gated — the dependency question decides whether it is feasible at all) | An **invisible, robust watermark** carried in the pixels, plus the metadata half: sign a **C2PA / Content Credentials** manifest on output, and embed a small durable identifier that survives a screenshot so the manifest can be recovered afterwards. Extends the existing `watermark` verb rather than inventing a surface. | Existing `watermark` (`--image`/`--text`), the SSIMULACRA2 scorer, and the metadata lane | **Only pixel-domain marks survive a screenshot** — a screenshot re-renders pixels and carries no metadata, and copy+paste usually transfers a bitmap only, so EXIF/XMP/**C2PA alone dies to both**. That is why the C2PA architecture pairs a signed manifest (rich, fragile) with a **soft binding** (tiny, durable) — they are complementary, not alternatives. **crustyimg has an unusual advantage here: SSIMULACRA2 can measure whether the mark is actually imperceptible**, which most implementations cannot do for themselves. ⚠ **Two things to settle before this is real.** (1) **A load-bearing dependency probe**: does a permissive, pure-Rust robust-watermarking crate exist? If it needs a C library, it breaks the zero-dependency thesis and belongs on the license watchlist instead — this decides feasibility, so probe before framing. (2) **A direction conflict to resolve deliberately**: `web` *strips* metadata by default, for privacy. Adding provenance back in pixel form is a change of stance, not an increment, and the privacy story has to be restated honestly. Also accept the tradeoff triangle up front — robustness, imperceptibility and payload trade against each other; a mark that survives screenshot + rescale + recompression carries **tens of bits**, an identifier rather than a story, and nothing is unbreakable under cropping or deliberate attack. |

**→ Cut 1.0 here.** The complete, trustworthy local toolchain: the shipped engine + broad input
reach + build/cache/lockfile + manifest + geometry + a library & demo page, with distribution
live and a **CLI-quality pass** (Track B) folded in. Coherent, defensible, every piece something
people want and that we can make *excellent*.

**One quality item sits inside that line, and it is a defect rather than a wave.** `resize`
**resamples in sRGB space, not linear light** (`src/operation/mod.rs`'s `resize_and_crop`, added
SPEC-121 2026-08-18, still hands `fast_image_resize` a plain `u8`/`u16` sRGB buffer — zero hits for
`gamma`/`linear`/`premultipl` in the operation and image modules, measured 2026-08-15, unchanged by
SPEC-121). High-contrast edges darken on downscale — a quality defect in the **most-used
operation** of a tool whose headline claim is quality-per-byte, which is why it is above the line
and not in post-1.0 polish. The fix is contained and does **not** require a 16-bit pipeline:
convert to linear `f32`/`u16` inside `Resize::apply`, resample, convert back on the way out
(`fast_image_resize` 5.x already provides `U16x4`/`F32x4` and `MulDiv`). ⚠ It **changes output
bytes for every existing recipe**, so it invalidates PROJ-007 build lockfiles and needs its own DEC
and migration story — and its premise should be **measured with SSIMULACRA2 before it is
specced**, not assumed. Full entry, including the premultiplied-alpha question (REFUTED by
SPEC-120, DEC-092 — `fast_image_resize` already premultiplies) and the now-answered question of
whether the pipeline should preserve >8 bits at all (SPEC-121: yes — `Invert`/`Resize`/`Watermark`
now preserve the input's bit depth instead of collapsing every image to 8-bit RGBA), is in
`docs/backlog.md`.

### Post-1.0 — sequenced below the line on purpose (deferred, not dropped)

| Wave | Ships | Status / gate | Notes |
|---|---|---|---|
| **Engine-backed lint rules** (STAGE-014: `legacy-format`, `excessive-jpeg-quality`, `indexed-png-opportunity`) | post-1.0 | **Gate replaced 2026-08-15 — now decidable.** The old gate ("waits for an adoption signal") could not turn green: nothing instruments that signal, `feedback/` holds no user request, and the named dogfood is the maintainer's own blog. The new gate is a **measurement**: run `lint` over the Eleventy photo-blog corpus and count the silent misses. Non-trivial → activate, `legacy-format` first. Essentially none → **close the stage**. | Both halves of the 2026-07-07 premise have moved: PROJ-009 (input reach) **shipped**, and 0.7.0 launched on four channels. `format/legacy-format` is the one rule carrying a positioning claim — territory.md:39 wins the Page-audit layer on being the source-file Lighthouse, and the 10 shipped rules answer neither `uses-webp-images` nor `uses-optimized-images`. The 2026-07-07 counter-argument ("cheap to build is why not to over-invest") stands and is kept on the stage record. See `projects/PROJ-004-image-lint/stages/STAGE-014-engine-backed-rules.md`. |
| **Optimization planner** (PROJ-003) | post-1.0 | **Defer / fold.** A `plan`/dry-run (byte deltas before writing) + combined goal object; already slimmed to orchestration, "may fold into 002/004." | Not an install-and-share moment. Keep the `--dry-run`/`plan` idea; it can ride along a feature wave rather than headline its own. |
| **Smart-crop / auto-color / redaction** | post-1.0, **beta** | **Judgment-bound.** The code is an afternoon; the *tune-until-it-looks-right-on-a-diverse-corpus* loop runs on human eyeballs, not compute. | Ship behind a clearly-labeled **beta** flag; redaction is **manual-region** (auto-detect would break the no-ML identity). Don't let it near the 1.0 quality bar. |
| **Format completion** — jpegli, permissive quantizer (indexed/lossy PNG, better GIF) | post-1.0 | **Upstream-gated** — AVIF decode (native, SPEC-058/DEC-053) and perceptual AVIF (`--target`/`--ssim`, PR #216) have shipped and left this row; but **the "better GIF" half's gate has cleared (measured 2026-08-16)**; the rest stands. Original text kept: *"rav1d maturity is exogenous; the jpegli license question is a decision, not a commit."* | Do it when the blockers clear, not on a calendar. Unblocks `indexed-png` as a real fix. **On "better GIF":** the upstream no longer gates it. AV1 encode (`rav1e`) and decode (`re_rav1d`) are both **already in-tree** and BSD-2, and a permissive pure-Rust ISO-BMFF writer with `av01`/`av1C` exists (`mp4-atom`, MIT/Apache). Animated GIF → animated AVIF measured **8.8–11× smaller** than the source GIF, and **6.3× smaller than animated WebP at higher quality**. Evidence in `docs/backlog.md` (three entries, 2026-08-16) and `docs/probes/animated-gif-to-av1/`; an **unscheduled** draft is at `docs/research/draft-spec-animated-avif-output.md`. **This is a stale-premise correction, not a scheduling change** — nothing here is framed, and the row stays post-1.0. |
| **PROJ-007 hardening maintenance backlog** (STAGE-024 deferred tail, 8 items) | post-1.0 | **Triaged + deferred 2026-07-12.** None gate a 1.0 "trust it in CI on untrusted files" claim. | canonicalize-contain-out (accept+documented write-escape residual; own spec w/ opt-out for symlinked out dirs) · pre-decode format sniff (fails-closed today) · full-pipeline peak envelope · `--max-pixels` cap-raise dial (revisit trigger live) · lint decode-seam audit + not-inspected rule · unusual-filename `.to_str()` sweep · cache-key build-profile · orphan-output prune. Full list + per-item rationale in `projects/PROJ-007-reproducible-build/stages/STAGE-024-hardening-and-security-sweep.md`. Pull individually if usage surfaces the need. |
| **Animated output** — animated GIF → animated WebP (and, separately priced, AVIF) | post-1.0, **but paired with STAGE-046** | **Not gated — unfiled until 2026-08-15.** This is the *capability* half of STAGE-046's repair: the tool currently accepts animated GIFs, warns about them, and silently flattens them to one frame. Until this ships, `lint`'s `format/animated-gif` rule recommends a destructive command, so STAGE-046 has to blunt the advice rather than fix it. | **A pure-Rust route needs zero new dependencies** and no core change: `image`'s `AnimationDecoder` (already used to *count* frames at `src/lint/rules.rs:303`) → the existing `Pipeline` run once per frame (ops are pure) → `image-webp` 0.2.4 frame encode → an **in-house `VP8X`/`ANIM`/`ANMF` RIFF mux**, *estimated* 150–250 lines. ⚠ `webp-animation` is **not** the clean take it was triaged as — it wraps `libwebp-sys2` (C); see `docs/backlog.md`. ⚠ **Animated AVIF is a separate, unpriced question** — a HEIF image sequence, materially harder than RIFF. Lossless-only is not a compromise for GIF sources: they are palettized by definition. Detail: `docs/backlog.md` → "animated input is silently flattened". |
| **`serve` (self-hostable image server)** | 2.0 | **Demand-gated.** | Weakest market for a pure-Rust pitch, least-stable surface, a different buyer. Let real pull decide it exists at all. Commercial rationale is private (`business/`, gitignored). |
| **Opt-in intelligence / new frontends** (PROJ-008 stretch: AI super-res, face-aware crop, JPEG XL when browsers land) | 2.0+ | Frontier, opt-in, never on the default path. | Pure-Rust preferred; AI/C bits feature-gated and BYO. **RAW Tier-2 was removed from this row 2026-10-28** — it is framed as PROJ-016 and now has its own wave (Track A #8). It never belonged beside AI super-res: it needs one feature flag and a permissive library that already exists, not a research bet. |

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
  2026 trust baseline). One correctness item folded in here — **`optimize` must never silently
  enlarge a same-format output**: measured 2026-07-13 that `optimize x.jpg -o out.jpg` (and
  `--profile preserve`) can be **2.65× larger** than the source, because a format pin / `preserve`
  skips the beat-source passthrough the default auto path enforces (`optimize_decide_one` →
  `decide::pick_winner`, `src/cli/mod.rs` ~4031/4201; the auto path was verified to pass the raw
  source bytes through unchanged). Fix: when the output format == the source format and a re-encode
  would grow the file, keep the original (+ a one-line stderr note); cross-format `-o out.png` /
  `--format` stays exempt (deliberate `convert` intent), and decide whether `--profile preserve`
  (the documented engine-off regression anchor, DEC-048/DEC-059) is guarded automatically or only
  opt-in. Refs: clig.dev, no-color.org, clap_mangen/clap_complete.
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
  native today, neither a SPEC-073 defect. (Both fixed in PR #216, which also made the
  perceptual search drive native AVIF.)
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
- **Demo page (STAGE-027) — SKELETON SHIPPED 2026-07-13** (SPEC-077, PR #85): the wasm engine runs
  as a **real web page** — drop an image → convert client-side → download — proven in headless
  Chrome (init over HTTP, zero network requests during conversion, output decoded by an independent
  decoder). `.github/workflows/pages.yml` deploys to GitHub Pages **gated on the browser smoke** and
  is the repo's **first CI job that builds through `just wasm-build`** (partially closes the "CI
  never runs the wasm smokes" carry). Carries: (1) **✅ RESOLVED 2026-07-13 — Pages enabled; the demo
  is LIVE at https://jysf.github.io/crustyimg/** (deploy job green, `.wasm` served as `application/wasm`,
  engine initializes in-browser) — the deploy leg is proven end-to-end. (2) **✅ SPEC-078 SHIPPED
  2026-07-13 (PR #86) — STAGE-027 COMPLETE:** all conversions run in a Web Worker (main thread stays
  responsive — verified with a negative control), AVIF both directions (encode off-thread + `.avif`
  input via `createImageBitmap`), the explain readout; driven CLEAN in Chrome/Firefox/Safari via
  three separate clients. **Carries:** (a) **mobile (iOS Safari / Android Chrome) still unverified**
  — a real-device test is a STAGE-028 launch blocker before the Show HN; (b) a **quality/byte-budget
  arg on the wasm surface** — ✅ **delivered by SPEC-079 (STAGE-029, PR #87, DEC-068)**, which added
  `optimizeDetailed(input, out_format, speed?, maxBytes?, target?) → OptimizeResult`, a per-call
  rav1e speed knob (measured 3.6× at speed 10), a returned SSIMULACRA2 `score()`, and Auto-picks-AVIF
  for photos.
- **STAGE-029 (demo launch-quality) — IN PROGRESS.** Kicked off by a measured finding that the demo
  mis-serves photos (slow + often no-improvement `optimize`). SPEC-079 (the hero-agnostic surface)
  shipped. SPEC-080 (demo redesign) was held for a strategy reconciliation — now **RESOLVED** (see
  STAGE-030): the hero is the **`web`** flow (downscale + content-modernize to AVIF + never-bigger).
  SPEC-080 will be reframed to mirror `web`'s definition once SPEC-085 lands.
- **★ STAGE-030 (command taxonomy & CLI-quality freeze) — NEW, ACTIVE (2026-07-14).** The strategy
  reconciliation ran a real benchmark (8 photos 0.7–47 MP + a q-sweep) and became a full pre-launch
  **CLI surface freeze**, adopted by the maintainer: **full taxonomy redesign, folded into PROJ-008,
  before the Show HN.** Evidence: today's `optimize` default = **24% in 16.5 s** (2/8 passthrough,
  49 s on 47 MP) vs a downscale→AVIF `web` path = **98% in 2.7 s** (size-insensitive); the "faceplant"
  is the byte-budget AVIF search re-encoding rav1e (9–74 s); the content branch (graphic → lossless
  WebP) already works. **Hard cutover** (~20→14 verbs, no aliases): a `web` flagship, `optimize`
  demoted to a keep-dims byte-primitive (search → opt-in), **`shrink` removed**, `convert --to`, a
  `meta` group, bundled recipes, a unified audit report + committed bench. Specs **SPEC-084–089**,
  DEC-069; **SPEC-084 SHIPPED 2026-07-14 (PR #88, $7.10)** — fast fixed-quality AVIF (`Mode::Fast`,
  q85) in the DEFAULT decision, the native twin of SPEC-079, converging the native/wasm Auto paths;
  verified CLEAN after a fix pass that caught a never-bigger+honesty blow-up on the metadata-bearing
  graphic edge (a lossy source forced off raw-passthrough now gets a compact lossy re-encode, never a
  lossless blow-up; a genuinely-larger output is reported honestly). **SPEC-085 SHIPPED 2026-07-15
  (PR #89, DEC-070, $5.25)** — the **`web` flagship verb** (downscale→modernize→never-bigger→strip→
  orient→score, reusing SPEC-084's engine) + a **bundled-recipe registry** (web/gallery/product);
  `web == apply --recipe web` delivered via a terminal-optimize recipe step (DEC-070). DEC-070
  follow-ups: `build` manifest doesn't run terminal-optimize recipes yet (`UnknownOperation`;
  `run_build` deferred), + a clearer unknown-recipe-name error. **SPEC-086 SHIPPED 2026-07-15 (PR #90,
  DEC-071, $3.30)** — `optimize --verify` (opt-in score; JSON gains an `"ssim"` field, non-verify
  byte-identical) + **`shrink` removed** (hard cut, no alias; `web` absorbs it) + the stale doc-comment
  fixed. **The taxonomy freeze's core is done** (fast AVIF default + `web` flagship + `optimize` demoted
  + `shrink` gone). **Next: SPEC-087 (`meta` group) + SPEC-088 (unified audit + committed bench)**, then
  optional SPEC-089 (`convert --to`). Sequence: STAGE-030 → STAGE-029 demo (reframed to the `web` hero +
  recipe presets) → STAGE-028 README/BENCHMARKS (SPEC-082/083, reserved) → Show HN.
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

- **Batch report (`--report[=path]`) — small, unscheduled.** A flag on the batch path that writes
  a summary of a multi-file run: per-file input→output format / bytes / savings, plus timing and
  errors/skips, aggregating the existing per-file `--json` audit report (the `optimize.explain/v1`
  schema) into one artifact — JSON for machines and/or a compact markdown/CSV table for humans. Its
  value is the **CI / deploy-pipeline** question "what did this batch actually do, and did anything
  fail or grow?" — a natural fit for the single dependency-free binary in CI. Reuses the audit
  report + the batch fan-out already shipped; the new work is aggregation, a writer, and the flag.
  Consider a nonzero exit code when any file errored or grew, so CI can gate on it. Frame as its own
  spec if pulled. (Also tracked in `docs/backlog.md`.)

---

## Relationship to existing docs
- **Reframes** the 2026-07-05 engine-ladder ordering (this doc supersedes it) and the crop-led
  `docs/backlog.md` (geometry → Wave 5 + the smart-crop beta; format/RAW → input-reach +
  format-completion).
- **Ladders up to** `docs/territory.md` (the space we're claiming) and its scope discipline.
- **Grounded in** `docs/research/proj-002-findings.md` + `proj-002-design-*.md`.
- **Snapshot of what's built:** `docs/moat.md`.

*Refine this doc as research lands and as each wave ships and the next is framed for real.*
