# Future Backlog — post-MVP waves

> A **candidate** backlog of deferred/post-MVP ideas, ranked into tentative
> waves. Per AGENTS.md §2, a project is framed formally only once the prior
> one ships — so these waves (PROJ-002, PROJ-003, PROJ-004+) are *direction*,
> not commitments, and IDs here are provisional. Sources: the ⏩ fast-follow
> and 💎 stretch markers in `docs/feature-exploration.md`, plus the brief's
> "Explicitly out of scope" and "Enables" sections (PROJ-001).
>
> The unifying reason almost everything below is cheap: the MVP lands the
> `Operation` trait + registry + recipe + Source/Sink architecture, so most
> new features are *just another `Operation`* (or a new encoder/Sink) that
> drops into the existing pipeline and recipe system without architectural
> change. Each item notes the enabling architecture already in place.

Complexity legend: **S** small · **M** medium · **L** large (native dep,
new metric, or new UI surface).

---

## ⚠ Live defects on shipped verbs (2026-07-26)

Surfaced by the read-only exploration in
[`docs/research/photo-preset-import-and-photographic-ops.md`](research/photo-preset-import-and-photographic-ops.md)
and **independently re-verified against this repo's source** in the PROJ-010 framing session. These
are not roadmap candidates — they are things that are wrong today, on verbs that shipped. Same class
as the classifier regression: a default path that hands the user a wrong result.

> **Homed 2026-07-26 → PROJ-010 STAGE-039 (shipped-verb correctness).** D-1 and D-2 are
> launch-gating alongside STAGE-034/035; D-3 is a cheap doc fix. The evidence stays here as the
> detailed source of truth; the stage holds the sequencing. The **unverified** reports at the end of
> this section were deliberately left out of that stage.

### D-1. `convert` strips EXIF Orientation without baking it — **confirmed, user-visible**

`run_convert` (`src/cli/optimize.rs:507`) builds `Pipeline::new()` at `:538` — the comment says so:
*"Pure re-encode: an empty pipeline returns the pixels unchanged."* The pixel-lane re-encode then
drops the metadata bundle. So the Orientation tag is discarded and **the rotation it described is
never applied**. `optimize` and `web` instead pin `auto-orient` first (`:790`, DEC-017).

Measured in the exploration with a positive control: `ctrl.jpg` (1200×800, `Orientation=6`, 14 EXIF
tags) → `convert` gives 1200×800 with the tag stripped and pixels unrotated → **displays sideways in
every viewer**, while `web` / `optimize` / `auto-orient` all correctly give 800×1200 (which is what
proves the harness can show the other result). **Orientation 6 is the ordinary phone-photo case.**

Two things this needs beyond the fix: a **design call** (bake vs preserve — `convert`'s contract is a
lossless-intent format change, so baking pixels is not obviously right), and a **mechanical sweep** of
every other re-encoding verb — `thumbnail`, `resize`, `responsive`, `edit` without `--auto-orient`.
Cite the grep and treat its scope as a claim ([[mechanical-sweeps-need-a-mechanical-check]]). The
regression fixture must assert **output dimensions**, not tag absence. Complexity **S–M**.

### D-2. `build` cannot run any bundled recipe — **confirmed, live since the manifest shipped**

`prepare_target` (`src/cli/build.rs:80`) calls `recipe.build_pipeline(registry)` directly at `:85`.
It never strips the terminal `optimize` marker: `grep` for `optimize` / `OPTIMIZE_STEP` /
`strip_terminal` in `build.rs` returns **0 each**. But **every** bundled recipe ends with
`op = "optimize"` — `src/recipe/bundled.rs:20` documents it and `:91` asserts it — and `optimize` is
a reserved pseudo-step, not a registered operation: `OperationRegistry::with_builtins`
(`src/operation/registry.rs:80-83`) registers exactly four (`identity`, `invert`, `resize`,
`auto-orient`).

So a manifest target bound to `web`, `product` or `gallery` fails at prepare time with
`UnknownOperation { name: "optimize" }`. The strip helper exists — `OPTIMIZE_STEP_OP` and its
consumer at `src/cli/optimize.rs:32-41` — it is simply not on the `build` path. Self-documented in
**DEC-070 point 4**. Complexity **S–M**. *(Subsumed if a recipe-schema `[output]` table ever lands,
which removes the positional hack entirely.)*

### D-3. `docs/data-model.md` advertises three operations that do not exist — **confirmed**

`docs/data-model.md:142-182` presents a worked "prep for web" recipe using `op = "unsharp"` (`:161`),
`op = "watermark"` (`:166`) and `op = "clean-gps"` (`:174`), plus CLI flags `--unsharp` and
`--watermark` (`:181-182`). **None of the three ops is registered** (four exist; see D-2), and
`watermark` is implemented but *deliberately* unregistered (`src/operation/mod.rs:784`,
`src/cli/ops.rs:945`), so naming it in a recipe fails `UnknownOperation`.

The doc labels the example illustrative, which is not enough: it is the first thing someone
evaluating *"can it run my pipeline?"* reads. Complexity **S**.

### Checked and NOT a defect here

- **"Squoosh is archived"** would be factually wrong (`archived: false`, 25.5k stars, maintainer
  retains commit access) — but **this repo does not say it.** `docs/launch-readiness.md:61` says
  *"squoosh-cli is **abandoned**"* and `docs/moat.md:21` says *"squoosh-cli is unmaintained"*, both of
  which are the correct, defensible claim. No change needed; recorded so it is not re-raised.

### Reported but NOT verified here — do not treat as findings yet

Each of these needs its own confirmation before it is spec-able:

- **RAW loses 100% of EXIF** (92 tags → 0) because `raw_preview` sets `metadata: None`
  (`src/image/mod.rs:462-468`), and **RAW orientation is never read**. The exploration also reports a
  *correction to DEC-055's planned fix*: the measured DNG's embedded previews contain **no EXIF at
  all** (a marker walk shows `FFDB → FFC0 → FFC4 → FFDD → SOS`, no APP1), so "thread the preview's
  APP1 forward" would not restore orientation — the container's IFD0 tag is the only source. The
  portrait-RAW case is explicitly stated as **cannot-determine**, not confirmed.
- **The metadata lane reaches only JPEG and PNG.** Mechanism is visible at
  `src/metadata/mod.rs:67-68` (only `ImageFormat::Jpeg`/`Png` map to a `Lane`), so metadata cannot be
  written on the AVIF output path — but the exploration's exact spelling
  (`Lane::Jpeg | Lane::Png`) returns **0 hits**, so its stated reach is unconfirmed.
- **AVIF byte-determinism is unbacked upstream.** `aomenc`/`vpxenc` ship `-D, --debug` *to become*
  deterministic; rav1e has no guarantee and a filed nondeterminism bug (#2781). If crustyimg's AVIF
  is not deterministic across thread counts, existing "reproducible" language is a false claim.
  **Measure before claiming either way.**
  ✅ **MEASURED — SPEC-123 / DEC-094 (2026-08-17).** Verdict: **the encoder ignores the thread
  setting.** `ravif` is compiled *without* its `threading` feature (reachable only via
  `image`'s `rayon` feature, which `avif = ["image/avif"]` does not enable), so it uses its own
  `rayoff` shim: the encode is **serial** and the tile count comes from
  `std::thread::available_parallelism()`, not from a rayon pool. 18/18 matrix cells
  (`convert`/`web`/`optimize` × 2 corpus inputs × `RAYON_NUM_THREADS` 1/4/14) produced identical
  SHA-256s, `--jobs` likewise, and 10 repeats/verb were run-to-run stable — with `cpu/wall ≈ 0.99`
  on every leg, i.e. the lever moved no work. Existing "reproducible" language is **not** falsified
  by the thread axis. Two riders that are: **(a)** the knob the encoder *does* read — the machine's
  core count — changes the bytes (positive control: a `--features image/rayon` probe moved them at
  1/4/14 tiles, and the shipped bytes land exactly on its 14-tile point on a 14-core host), and
  **core count is in neither the lockfile's `[env]` nor its arch/OS/codec caveat list
  (`src/build/lock.rs:32-37`)** — so `diff` can call a same-`target` cross-machine hash change a
  regression; **(b)** the shipped build pays the multi-tile compression penalty and collects none of
  the parallelism — **+1.5 %** bytes (photo) / **+47.9 %** (graphic) vs a 1-tile encode, at
  **5.7× / 4.4×** the wall clock of the same tiles encoded in parallel.
  **Open follow-ups:** correct `lock.rs`'s caveat list (SPEC-123 shipped no `src/` change, so this
  is filed, not done); scope `with_num_threads(Some(N))` as the *determinism* lever and
  `image/rayon` as the separate *performance* lever. Re-derive with
  `python3 scripts/spec123_avif_thread_determinism.py`.
- **Gain-map (UltraHDR) input is silently discarded**, destroying HDR the user can see on their own
  display — same class as D-1. Cheapest correct move is a **detection fixture**, not integration.

---

## Post-0.1.0 fast-follows — advisory elimination (→ clean 0.2.0)

Agreed at the v0.1.0 cut: ship 0.1.0 with the three accepted `deny.toml` advisory
ignores (DEC-042, all low-risk/unreachable/documented), then **eliminate them at the
source** and remove the ignores for a clean 0.2.0. These are the DEC-042 revisit triggers
made concrete. **Now framed as STAGE-010** (advisory elimination & dependency hygiene).

| Item | Value | Complexity | Approach (grounded) |
|---|---|---|---|
| **Drop `ttf-parser` (RUSTSEC-2026-0192)** — swap `ab_glyph` → **`skrifa` + `zeno`** in `watermark --text` — **SPEC-044 (design), DEC-045** | Removes the unmaintained font dep | **M** | ⚠️ **The original `fontdue` plan was a dead end** — a design-time probe found fontdue 0.9.3 *still depends on `ttf-parser` 0.21.1*, and RUSTSEC-2026-0192 is crate-wide (`patched=[]`, `informational=unmaintained`), so it would NOT remove the ignore. Retargeted to the advisory's own recommended alternative: **`skrifa` 0.44** (Google `fontations`, MIT/Apache, `ttf-parser`-free) for outlines/metrics + **`zeno` 0.3.3** (MIT/Apache) for mask rasterization. Probe-verified against the real Go font (ascent/advance/bounds match; `(coverage, Placement)` ≈ ab_glyph's `px_bounds()`+`draw()`). Behavior-preserving; drops pairwise kerning (nil effect — bundled font has no legacy `kern` table). Then delete the -0192 ignore. |
| **Drop `quick-xml` vulns (RUSTSEC-2026-0194/-0195)** — replace `little_exif` with an **in-house EXIF-tag writer** — **SPEC-045 (design), DEC-046** | Removes 2 real (unreachable) vulns + the last XML dep (`quick-xml`) + `brotli` | **M** | No drop-in exists (`nom-exif`/`kamadak-exif` are read-only; `little_exif` was ~the only pure-Rust read+write, DEC-029) — and `little_exif 0.6.23` is latest, still pinning vulnerable `quick-xml ^0.37` (no bump path). Write a minimal binary **TIFF-IFD serializer** for the tags we set (Artist/Copyright/ImageDescription) + selective **GPS-IFD removal**, on the raw TIFF block `img-parts` exposes. **Probe-validated**: a generic IFD parse→recurse-subIFD→re-serialize round-tripped a real JPEG (IFD0 + ExifIFD) byte-identical per `kamadak-exif`. Bounded/panic-free parser (untrusted EXIF). Then drop the -0194/-0195 ignores + `little_exif` (amends DEC-029). ⚠️ Does **NOT** remove `paste`/-2024-0436 — see the residual note below. |

Both remove `deny.toml` ignores on completion; do the font swap first (SPEC-044, cheaper), the EXIF writer second (the meatier, higher-value one — kills actual vulnerabilities). **Net after both:** `deny.toml` goes from **3 ignores → 1** (not 0).

> **Lesson (fontdue dead-end):** the backlog's "fontdue has its OWN parser — no ttf-parser"
> was outdated; modern fontdue delegates parsing to `ttf-parser`. An *unmaintained* advisory
> (`patched = []`) is crate-wide, so swapping to a different version of the same crate never
> clears it — only removing the crate does. Probe the actual dep tree before trusting a
> "drops dep X" plan. See DEC-045.

> **Residual — `paste` (RUSTSEC-2024-0436) stays (DEC-046):** the original plan said the EXIF
> writer would also drop the `paste` chain. It won't. `paste` reaches the graph via **both**
> `little_exif` **and** `rav1e`→`ravif`→`image` (the `avif` feature), and `deny.toml` uses
> `[graph] all-features = true`, so the `rav1e` path keeps `paste` in the evaluated graph.
> `rav1e 0.8.1` is latest (no fix). So `-2024-0436` (an unmaintained *build-time* proc-macro,
> the lowest-risk of the four) remains a documented ignore for 0.2.0 — revisit when `rav1e`
> drops `paste`. Maintainer-accepted 2026-07-04. Same lesson as fontdue: probe the *full*
> feature graph before claiming a "drops dep X" outcome.

**Also (S, UX polish):** the shipped `--help` leaks internal jargon into command
descriptions — e.g. `view … (STAGE-002; stub in STAGE-001)` (view is no longer a stub),
plus `STAGE-00X` / `DEC-0XX` references across several subcommands. Clean the clap
doc-comments in `src/cli/mod.rs` so user-facing help reads for end users (no stage/DEC
refs, no stale "stub" text). Found during the v0.1.0 install smoke-test.

## PROJ-002 — next wave after MVP

> **Scoping status (2026-07-05):** PROJ-002 is being framed **research-first**. Before
> committing the wave, a dedicated research session runs `docs/research/proj-002-scoping-research.md`
> (survey adjacent-tool demand + the pure-Rust/permissive crate landscape + validate the
> **"image-asset engine for web workflows"** thesis) → `docs/research/proj-002-findings.md`,
> which feeds a planning session that writes the `brief.md` + stages. Current bet: **`crop`
> (+ smart/content-aware crop) ships as 0.3.0** and opens PROJ-002; the web-asset-engine
> track (placeholders, manifest, favicon/icon sets, palette) is the differentiating thesis
> under test. Runway: **0.2.1** = PATCH-003 dep bumps + scheduled deny CI (hygiene) →
> research → **0.3.0** = crop → PROJ-002 build-out.

High value, low complexity, all drop into the `Operation` trait + recipe
system. **`crop` is the lead item (user-flagged)** — the brief calls out the
geometry extras as explicitly on the roadmap and deferred to this near-term
follow-up, with `crop` first.

### Geometry extras (lead: `crop`)

| Item | Value | Complexity | Enabling architecture in place |
|---|---|---|---|
| `crop` (rect / gravity / center / aspect) | **Lead item.** The most-requested missing geometry op; pairs with resize for exact framing | S–M | `Operation` trait; `gravity` anchor concept already defined (shared with watermark); recipe chaining |
| `rotate` | Arbitrary/90° rotation; complements `auto-orient` | S | `Operation` trait + pipeline |
| `flip` / `flop` | Vertical / horizontal mirror | S | `Operation` trait + pipeline |
| `trim` | Auto-remove uniform border | S–M | `Operation` trait + pipeline |
| `pad` / `extend` | Add border/canvas to a target size | S | `Operation` trait + gravity anchor |

### Effects catalog (the `Operation`-trait playground)

| Item | Value | Complexity | Enabling architecture in place |
|---|---|---|---|
| grayscale, sepia, solarize, invert | Common quick filters; were in the prototype | S | `Operation` trait; recipes make presets trivial |
| pixelize | Privacy/redaction + stylistic | S | `Operation` trait |
| sobel / edges | Edge-detection effect | S–M | `Operation` trait + **`image::imageops::filter3x3`** (arbitrary 3×3 kernel, already compiled in). ⚠ This row previously read "imageproc convolution" — corrected 2026-08-15: `Cargo.toml:68` explicitly rejects imageproc ("it drags in sdl2/nalgebra"), and `filter3x3` makes it unnecessary. |

### Format / web-optimize

| Item | Value | Complexity | Enabling architecture in place |
|---|---|---|---|
| **WebP output** | Biggest real web-size win; the headline fast-follow | M | `convert`/`optimize` encode path + codec policy (DEC-004); `image` already supports WebP |

---

## PROJ-003 — later

Higher complexity, a native/feature-gated dep, a new metric, or a broader
suite. Still additive on the same architecture.

| Item | Value | Complexity | Enabling architecture in place |
|---|---|---|---|
| **AVIF output (feature-gated)** | Best modern compression; slow pure-Rust encode behind a cargo feature | L | Codec policy already gates native/slow codecs behind off-by-default features (DEC-004) |
| `open` in external app | Hand off to Preview / Safari / Chrome / OS default | S | Sink abstraction (a non-rendering "open" sink) |
| `compare` (SSIM / PSNR) | "Did optimization hurt quality?" — quality measurement | M | Read-only inspect path (like `info`); two-image read |
| target-size / target-quality auto-tuning | "Smallest file ≥ SSIM threshold" — a real differentiator | L | Builds on `compare` metric + `optimize` encode loop |
| color / tone suite (brightness/contrast/gamma/levels/curves) | Full tonal editing | M | Each is an `Operation`; recipe chaining |
| montage / contact-sheet | Grid of images (was in original docs) | M | Source list + a compositing Sink |
| append (H / V) | Concatenate images horizontally/vertically | S–M | `Operation` over a Source list |
| blurhash / thumbhash | Placeholder hashes for web loading | M | Read-only encode-side output, like `info --json` |
| placeholder fetch (Picsum / Unsplash) | Pull sample/placeholder images | M | New Source variant (network fetch); note: would be the first network dependency |

### Input formats — camera RAW + HEIC/HEIF

Reading formats crustyimg can't decode today. The `image` decode surface is
PNG/JPEG/GIF/BMP/TIFF/ICO/WebP (+AVIF behind `--features avif`); these add new
*input* decode paths. See `guidance/license-watchlist.yaml`
(`raw-camera-decode`, `heic-heif-decode`) for the full license analysis.

| Item | Value | Complexity | Enabling architecture / notes |
|---|---|---|---|
| **RAW → jpg/png (Tier 1: embedded preview)** | Nikon NEF, Canon CR2/CR3, Fuji RAF, Leica DNG/RWL, Sony ARW → basic best-effort convert | **M** | **Permissive, pure-Rust, recommended.** Extract the full-res embedded JPEG (no demosaic). Reuses `kamadak-exif` (TIFF/EXIF IFDs, already a dep) + `image` re-encode; CR3 needs ISOBMFF box parsing (shared with HEIF below). No copyleft/patents. |
| RAW development (Tier 2: demosaic) | True sensor development (WB + color) — higher quality | L | `rawler` (LGPL-2.1) behind an opt-in `raw` feature + a `cargo-deny` exception (ansi_colours precedent), or a from-scratch demosaic (X-Trans is hard). Overkill for basic conversion. |
| HEIC/HEIF → jpg/png | iPhone / modern-camera photos | L / n/a | **No permissive in-tool path** — HEVC has no permissive pure-Rust decoder (imazen `heic` = AGPL; `libheif-rs` = LGPL + system libheif; from-scratch HEVC = rejected, scale+patents). **Fallback: pre-convert/shell-out (`sips`/`heif-convert`)** — no license obligation. Settled unless a permissive HEVC decoder appears. |

**Tier-1 RAW spec sketch (the buildable one):** a new decode path that, on a
recognized RAW extension/magic, locates the largest embedded JPEG preview
(TIFF `IFD`/`SubIFD` `JPEGInterchangeFormat`/preview tags for NEF/CR2/DNG/RWL/ARW;
the RAF header's JPEG offset+length for Fuji; the `PRVW`/`THMB` ISOBMFF box for
Canon CR3), decodes it via `image`, and feeds it into the normal pipeline (so
`convert`/`optimize`/`thumbnail` all work). Bound it with the existing decode limits
(STAGE-006). Failure mode when no full-size preview exists → clear exit 4 with a
"RAW development (Tier 2) not built; only embedded-preview conversion is supported"
message. Behind a `raw` cargo feature to keep the default lean. A future project
wave, not PROJ-001.

---

## Stretch / PROJ-004+

Differentiators with a meaningful new surface (UI, color science, or native
encode). Worth doing, clearly later.

| Item | Value | Complexity | Enabling architecture in place |
|---|---|---|---|
| ratatui TUI live-preview editor → exports a recipe | "Experiment like an editor" with live preview, then save the tuned chain as a recipe — additive, not a rewrite | L | Recipe (de)serialization + registry; the editor just builds an op list and saves it |
| ICC color conversion (lcms2) | True color-managed conversion (MVP only preserves ICC, never converts) | L | Metadata/ICC container lane already preserves ICC (DEC-003); conversion adds an `Operation`/encode step |
| mozjpeg / turbojpeg native encode (feature) | Best-in-class JPEG size/quality | L | Codec policy already reserves native codecs behind off-by-default cargo features (DEC-004) |

---

## Notes

- **`crop` is the explicit lead** of the next wave (user-flagged; brief
  "Explicitly out of scope" → deferred geometry extras, `crop` first).
- WebP output is the highest-value *format* fast-follow and the natural
  headline for PROJ-002 alongside the geometry/effects work.
- Anything touching untrusted input in a future wave inherits the STAGE-006
  hardening baseline (decode limits, path/symlink safety, recipe validation,
  `cargo audit` in CI) — new `Operation`s are pure pixel transforms and add
  little new surface; network fetch (Picsum/Unsplash) and native codecs are
  the ones that would warrant fresh threat-model review and a DEC.
- **Input formats (RAW / HEIC)** — new *decode* paths, not new `Operation`s. RAW
  Tier 1 (embedded-preview) is the clean permissive win and the recommended first
  build; a small **ISOBMFF/box parser** is reusable across Canon CR3 previews AND a
  future HEIF container. Both are untrusted input → inherit the STAGE-006 hardening
  (decode limits, no-panic). HEIC's HEVC codec has no permissive path — stays a
  pre-convert/shell-out story. Full analysis + revisit triggers live in
  `guidance/license-watchlist.yaml`.
- **Permissive in-house `Display` sink (drop viuer + ansi_colours)** — S–M, near
  term. viuer pulls `ansi_colours` (LGPL-3.0-or-later), the only copyleft dep in
  the tree (optional `display` feature; accepted today via a documented
  `cargo-deny` exception, DEC-018). Replace with a thin permissive sink: emit the
  **Kitty graphics** + **iTerm2 inline** protocols directly (base64-PNG escape
  sequences), **`icy_sixel`** (MIT/Apache) for Sixel, and a **truecolor
  half-block** fallback (24-bit `▄`, no ANSI-256 quantization → no `ansi_colours`
  needed). Removes the last copyleft, stays dependency-light, makes the "100%
  permissive" claim literally true, and revisits DEC-011. `ratatui-image` (MIT,
  multi-protocol) is the right display lib for the *later* ratatui TUI editor, not
  for the one-shot `view`.
- **A "crustyimg in a deploy pipeline" benchmark** — M, post-launch companion to
  `BENCHMARKS.md` (not a rewrite of it). BENCHMARKS measures **single-image latency on a
  14-core desktop**, which is crustyimg's worst case: it's the configuration where sharp's
  multi-threading pays most (3–9×) and where our single-threaded design pays least. CI runners
  and build containers are small — typically low single-digit cores — so **the gap compresses
  toward the per-core result, where the two already trade wins 4–4**. Meanwhile the
  zero-dependency story is worth most exactly there: no libvips, no `node_modules`, no native
  addon to compile or platform-match. Three axes to measure, all cheap extensions of
  `scripts/bench-compare.py` (which already pins sharp to one thread):
  (1) **a core-count sweep** at 1/2/4 cores — the range pipelines actually run in;
  (2) **batch throughput** — N images, total wall-clock, each tool using the machine as it likes.
  This is the one that could change the story: crustyimg parallelizes **across files**
  (`apply --recipe`) while sharp parallelizes **within** an image, so on a multi-image job we may
  already saturate the cores. **Verify the batch path is genuinely parallel before leaning on
  this** — it's an assumption, not a measured fact;
  (3) **install / cold-start cost** — `npm i sharp` vs downloading one static binary, plus
  container image size. A real pipeline cost nobody measures, and one we'd win on merit.
  Sequence AFTER the LLM-free benchmark refresh (see the repo-tooling backlog) so re-running
  costs wall-clock instead of tokens. Same fairness bar as `BENCHMARKS.md`: state the machine,
  pin versions, publish losses.
- **Benchmark corpus expansion** — S–M, post-launch, and gated on the same refresh tooling.
  Today's corpus is 8 real photographs (0.7–47 MP), which exercises only the AVIF-photo path.
  Highest-value addition is **content diversity**: screenshots, UI, and flat graphics, where the
  engine's content-aware branch picks lossless WebP and AVIF is roughly a 4× regression. That
  branch is the actual differentiator and is currently untested in public. **Fairness trap to
  design around:** comparing crustyimg's automatic choice against a competitor *forced* to AVIF
  is a strawman a reader will call out — the honest claim is "correct format automatically" vs
  "you have to know which to pick", so the competitor must be shown doing the right thing.
  Then: a public/licensed corpus (so readers can check *our* cells, not just re-run the method)
  and thicker small/medium buckets (currently n=1 and n=2).
- **The recipe format as a first-class language, with a playground** — M–L, post-0.6.0, and
  the highest-leverage adoption item on this list. Recipes are *already* a small declarative
  language: versioned (`version = "1"`), named, an ordered `[[step]]` pipeline, parsed and
  validated by `Recipe::parse`. And the wasm surface already accepts **arbitrary** recipe TOML
  via `transform(input, recipe_toml, out_format)`. What's missing is not syntax — it's
  formalization and exposure:
  (1) **a formal op reference / schema** — every `op`, its params, types and defaults, as the
  source of truth (partly in `docs/cli-reference.md` today); optionally emit a **JSON Schema**
  so editors give autocomplete for free;
  (2) **a `validate(recipe_toml)` wasm export** that parses and validates *without running* and
  returns **structured** errors (which step, which field, what was wrong, did-you-mean);
  (3) **a playground in the browser tool** — edit a recipe, validate it live, run it on your own
  photo, copy the exact TOML into CI.
  **The point is that the browser would be running the CLI's actual validator compiled to wasm,
  not a JavaScript reimplementation** — so unlike every other online config playground, it
  cannot drift from the real tool. That supports a claim almost nobody can make honestly: *what
  you just watched run is exactly what your CI will do.*
  **Do NOT invent a new syntax.** TOML is parseable everywhere, diffable, familiar, already
  versioned, and free; a bespoke grammar means maintaining a parser twice and losing every
  editor that already understands TOML.
  **Probe first:** what do recipe parse/validate errors look like today — structured, or
  strings? That answer decides whether this is S or L, because **error quality is the entire
  feature**: "invalid recipe" is worthless, "step 2: unknown op `reisze` — did you mean
  `resize`?" is the product.
  Feeds the web-asset manifest and the SSG plugins, which consume recipes.
- **The browser demo as a real tool (not only a showcase)** — M, post-0.6.0, demand-driven.
  Driven by the maintainer's own use: dropping a whole Photos batch in and wanting it to be
  *good*, not just impressive. The competitive position is genuinely open — **Squoosh is
  archived** (unmaintained; won't start on current Node), TinyPNG/CloudConvert **upload your
  images**, ImageOptim needs a Mac install; none of them report a perceptual quality number.
  Candidate features, ordered cheapest-first: **presets** (`gallery.toml`/`product.toml` already
  exist and are unused by the demo), **batch + download-all** (the real workflow), a **byte
  budget** (⚠ an unsatisfiable `maxBytes` currently returns over-budget bytes *silently*, so the
  UI must self-check `bytes.length` and say so), and **remembered settings**. Note these serve
  *repeat use*, a different axis from demo-value — batch makes a better tool but not a better
  demo.
  ⚠ **Decide the territory question explicitly before building any of it.** The project brief
  scopes the demo as "a thin marketing artifact, NOT a web app" — a listed risk-to-thesis.
  Crossing that deliberately is fine; drifting across it one feature at a time is how you end up
  maintaining a web app you never chose while the CLI and library stall. Useful distinction: a
  **playground is an on-ramp** (you leave with a file to run elsewhere, so it strengthens the
  CLI), whereas **batch + download-all makes the demo a destination** that competes with it.
  Let launch traffic inform the call.
- **`eleventy-crustyimg` has a real dogfood testbed.** The roadmap already sequences Eleventy
  first among SSG integrations; the maintainer's own Eleventy photo blog is an actual site with
  actual photos and an actual build, which is what a plugin needs to be designed against instead
  of a toy fixture. Worth pairing the two when the manifest wave (#4) comes up.
- **Add an install-footprint comparison to `BENCHMARKS.md`** — S, post-launch, pairs naturally with
  the deploy-pipeline benchmark above (whose third axis is install cost). `BENCHMARKS.md` compares
  size, speed and quality of the *output*; it says nothing about what each tool costs to *install*,
  which is where the zero-dependency thesis is strongest and currently unevidenced. Ad-hoc numbers
  measured 2026-07-23 on arm64 macOS, **indicative only — re-measure through a harness rather than
  lifting these**: crustyimg **12.2 MB** as one static binary (~15.1 MB once AVIF is in the default
  build); `npm i sharp` → **28 MB** of `node_modules`, of which **17 MB is
  `@img/sharp-libvips-darwin-arm64`** and 9 MB a `wasm32` fallback you may never run; ImageMagick via
  brew → **31 MB** plus **~86 MB** across 17 transitive dependencies.
  **State the caveats or don't publish it:** it isn't apples-to-apples (ImageMagick's dependencies are
  shared libraries other formulae also use, so the *marginal* cost on a populated machine is lower),
  and it's one platform on one machine — the same caveat the rest of the doc already carries.
  **The shape is the story, more than the totals:** sharp's footprint is dominated by a prebuilt
  libvips matched to your platform, which is exactly the thing that breaks when a CI runner or an
  architecture changes. crustyimg's is one artifact with nothing to match. That's the benchmark's
  existing finding told from the other side — you pay some size and speed, and you get nothing to link
  and nothing to go wrong on a different runner.
- **Provenance that survives a screenshot (invisible watermark + C2PA)** — L, post-1.0,
  **research-gated**. The question: can you mark an image so its origin is still recoverable
  after a screenshot or a copy+paste? Mechanism decides the answer, and the split is sharp:
  a screenshot re-renders pixels and carries **no** metadata, and copy+paste usually transfers
  a bitmap only — so **EXIF, XMP and C2PA manifests all die to both**. Only a signal carried in
  the *pixels* survives.
  **The standard architecture is a pairing, not a choice.** C2PA / Content Credentials gives you
  a signed manifest that is rich but fragile; a **soft binding** — a robust invisible watermark
  or perceptual fingerprint — is tiny but durable, and carries an identifier that lets the
  manifest be recovered from a registry afterwards. Google's SynthID and Digimarc are the same
  shape. (Verify against current C2PA specs when this is picked up; the area moves.)
  **Why crustyimg is unusually well placed:** the `watermark` verb already exists
  (`--image`/`--text`), so this extends a surface rather than inventing one — and **SSIMULACRA2
  can measure whether the mark is genuinely imperceptible**, which most implementations cannot
  do for themselves. That turns "invisible" from a claim into a number, which is this project's
  whole posture.
  ⚠ **Probe the dependency before framing anything.** Does a permissive, pure-Rust robust
  watermarking crate exist? If it needs a C library, it breaks the zero-dependency thesis and
  belongs on `guidance/license-watchlist.yaml` with a revisit trigger instead. This decides
  feasibility, so it is a design-time probe, not a build task.
  ⚠ **Resolve the direction conflict deliberately.** `web` *strips* metadata by default, for
  privacy — the tool currently removes provenance on purpose. Adding it back in pixel form is a
  change of stance, and the privacy story would need restating honestly rather than quietly.
  **Accept the tradeoff triangle up front:** robustness, imperceptibility and payload capacity
  trade against each other. A mark that survives screenshot + rescale + recompression carries
  **tens of bits** — an identifier, not a story — and nothing is unbreakable under cropping,
  rotation or deliberate attack. Claim that honestly or not at all.

## Batch report flag (maintainer request, 2026-07-25)

A `--report[=path]` flag on the BATCH path that writes a summary of a multi-file run:
per-file input→output format / bytes / savings, plus (likely) timing and errors/skips,
aggregating the existing per-file `--json` audit report (SPEC-088, `optimize.explain/v1`)
into ONE batch artifact. Format TBD — JSON (machine) and/or a compact markdown/CSV table
(human). Natural fit for the **goal-1 CI / deploy-pipeline** use case: "what did this batch
actually do, and did anything fail or grow?" Reuses machinery already in place (the audit
report + the rayon batch path); the new work is aggregation + a writer + the flag surface.
Frame as its own spec post-launch. Consider whether it also wants an exit-code signal when
any file errored or grew (CI-friendly).

## Shell completions — install, complete paths, don't rot silently (maintainer report, 2026-07-26)

`crustyimg completions <shell>` has existed since SPEC-040 (DEC-039) and works, but the
surrounding story has three gaps — all three found by the maintainer hitting them in real use,
not by any gate. **Three separable defects; likely one S spec.**

1. **Nothing installs them.** The Homebrew formula ships no completion files (checked: no
   `share/zsh/site-functions/_crustyimg`, no `etc/bash_completion.d/`, no
   `share/fish/vendor_completions.d/`), there is no `completions/` directory in the repo, and
   neither the README nor `--help` mentions the subcommand. So a `brew install` user gets
   nothing unless they discover the verb and place the file by hand — which is what happened,
   into `~/.oh-my-zsh/plugins/brew/`, a directory `omz update` can overwrite. Fix is the
   standard `generate_completions_from_executable` in the formula, which also makes completions
   regenerate on upgrade — the real cure for (3).

2. **No `ValueHint` anywhere in `src/`** (verified: raw grep, zero hits, positive control
   passes). clap only emits a file-completion action for args carrying
   `ValueHint::FilePath`/`AnyPath`/`DirPath`; without it every path argument and path-valued
   flag generates the generic `_default` action instead of `_files` — confirmed in the real
   0.6.0 output (`':input:_default'`, `'*::inputs:_default'`, `-o`, `--name-template`).
   **Severity is shell-dependent, and measured, not assumed:** on zsh `_default` does reach
   filename completion (confirmed working on the maintainer's machine once the script was
   current), so zsh degrades gracefully. On **bash it is a hard failure** — clap registers
   `complete -F _crustyimg`, which replaces bash's default filename completion, so with no file
   action bash offers nothing and has no fallback. Fix is mechanical (`value_hint` on every path
   arg across the 14-verb surface) but per [[mechanical-sweeps-need-a-mechanical-check]] it needs
   a grep-backed sweep with a hit count, not a read-through. Natural mechanical check: assert the
   generated script contains no `_default` action for a path argument.

3. **A stale completion fails silently and confusingly — and STAGE-030 guaranteed a stale one.**
   The maintainer's installed script predated the surface freeze: it still offered `shrink`
   (removed, SPEC-086) and `copy-metadata` (consolidated to `meta copy`, SPEC-087) and had **no
   `web` case at all**. Because the script is `#compdef`-registered, zsh hands it the whole line;
   its `case $line[1] in` matched nothing, no `_arguments` spec ran, and the function returned
   having "handled" the command — so **zsh offered nothing and did not fall back to files**.
   Verbs surviving the freeze still completed, which is what makes the failure so confusing:
   "everything works except the flagship verb." The 20→14 hard cutover with no aliases means
   *every* pre-freeze install has exactly this breakage, and the 0.6.0 CHANGELOG never tells
   anyone to regenerate. Wants: a CHANGELOG/README note, and consider having the script assert
   its own version against the binary so staleness is loud instead of silent.

## Classifier regression — graphics promoted to lossy by the resize (code review, 2026-07-26)

⚠ **Launch-gating, engine-level, not yet framed as a spec.** Full findings:
[`docs/research/pr113-classifier-review-findings.md`](research/pr113-classifier-review-findings.md).

A max-effort review of the merged SPEC-105 commit (`54ba05e`) returned 15 findings, none
refuted. It posted nothing to PR #113, so that document is the only record — read it before
framing any classifier work.

The load-bearing one: **classification runs after the resize pipeline**
(`src/cli/optimize.rs:989` → `:1013`), so `--max` chooses the content class. Downscaling
averages hard edges into intermediate luma bins, which is exactly the signal the SPEC-105
entropy rule reads as "photograph" — and `web` downscales to 2048 by default. DEC-047's
calibration was measured at native size, so it does not describe the path most users take.
Reported (unverified, being re-derived): a 3840×2160 code-editor screenshot crosses the 4.0
threshold at `--max 2048` and ships a 358 KB lossy AVIF for a 111 KB lossless source —
larger than the input, and smeared, on the flagship verb.

Also confirmed structurally: cascade rule 6 is unreachable dead code (two constants now
inert, `clippy-fmt-clean` is a **blocking** constraint and the automated gate cannot see
this); `--profile docs` is a silent no-op for promoted images; DEC-047 states a rule reach
that is false for images ≤ 128 px; and the PR's headline calibration guard is a tautology
that stays green with the threshold moved to 5.5.

Suggested shape (the review's, not yet adopted): one design spec for classification
placement / scale-aware entropy — which likely **subsumes** the queued "scale-normalize the
flat/edge detector" item rather than layering on it — plus one evidence-integrity spec that
commits the boundary specimens DEC-047 cites but the repo does not contain, re-establishes
each diluted guard with a negative control, and corrects DEC-047's two false claims.

Session prompt for the re-derivation:
[`docs/research/pr113-rederivation-session-prompt.md`](research/pr113-rederivation-session-prompt.md).

## Carried forward from PROJ-008 — RESOLVED, re-homed into PROJ-010 (2026-07-26)

PROJ-008 closed `shipped` on 2026-07-25 leaving three stages that were **not** its thesis work
(wasm core → npm library → client-side demo) and were deliberately left in place until the next
project's thesis was chosen. The maintainer framed **PROJ-010 — post-launch correctness and
consolidation** on 2026-07-26 and re-homed them. The three were **not** treated alike:

- **STAGE-031 — engineering quality and code health.** **Not moved; closed in place as `shipped`.**
  Its three specs shipped during the wave (097 `src/cli/mod.rs` split, 098/099 dependency-pinning
  record + correction) and their files live in PROJ-008's `specs/done/`. Moving the stage would
  have relocated PROJ-008's shipped work and PR provenance into a project that has not started.
  Its one unframed follow-up (strict-JSON `escape_json`) went to **PROJ-010 STAGE-036**, the
  continuation, which also inherits the shelved-directive record and the byte-identity gate.
- **STAGE-032 → PROJ-010 STAGE-037** (`git mv`, content unchanged). SPEC-092 (`convert --to` plus
  social/archive recipes). Additive only; STAGE-030's freeze deferred it on purpose.
- **STAGE-033 → PROJ-010 STAGE-038** (`git mv`). SPEC-106 (shell completions) plus six
  repo-tooling chores. **SPEC-107 (hostile / edge input confirmation pass — LAUNCH-GATING) left
  for PROJ-010 STAGE-035**, a launch-gating stage of its own.

**The two launch-gating items are now PROJ-010 STAGE-034 (classifier regression) and STAGE-035
(SPEC-107).** Both should be sequenced before the Show HN.

## Open — bound classification cost by sampling the source (2026-07-28)

**Measured, not speculative.** SPEC-108 moved `Analysis::compute` onto the source image, which is
what makes classification independent of `--max`. `Analysis::compute` is an unconditional
O(pixels) scan, so on large inputs it now costs more than it did on the bounded post-resize buffer.

Measured by SPEC-108's verify on a 24 MP (6000×4000) input:

| path | branch | pre-change | delta |
|---|---|---|---|
| graphic → lossless (cheap encode) | 484–507 ms | 344–352 ms | **~140–150 ms, ≈40%** |
| photograph → AVIF (encode dominates) | 4282–4468 ms | 4234–4342 ms | inside noise |

Isolated cleanly: `decode_ms` (8–14 ms) and `encode_ms` (~6 ms) are near-identical on both builds,
so the whole gap sits in classify+resize. `web`'s help text was corrected in the same change, so
the repo no longer claims size-insensitivity on that path.

**The obvious fix is wrong.** Classifying the downscaled buffer again reinstates exactly the
18.5×-blow-up bug SPEC-108 fixed.

**The shape worth trying:** bound analysis cost by sampling the **source** under a fixed,
scale-independent rule (e.g. cap the scan at N pixels by striding). Because the sample is drawn
from the source rather than from a `--max`-dependent resize, SPEC-108's invariant survives — the
emitted features stay identical across `--max` values, which is what AC-2 asserts and what any
implementation here must keep proving.

**Open question that decides the design:** striding changes measured entropy, so the thresholds
would need re-checking against the committed boundary specimens
(`photo_entropy_floor.png` 4.5176, `dither_32color.png` 3.6414). If a strided sample moves either
past `PHOTO_ENTROPY_STRONG`, sampling is not free and the trade needs stating. **Measure that
before designing.**

## Open — establish the real entropy ceiling of dithers-of-photos (2026-07-26)

**Nobody's work yet. Both STAGE-034 specs forbid retuning thresholds, so this has no home.**

`PHOTO_ENTROPY_STRONG = 4.0` is safe only if no genuine graphic reaches it. DEC-047 has now
twice stated a ceiling for the hardest case — dithers of photographs — and been wrong twice:

| claim | source | refuted by |
|---|---|---|
| "none of those reach 4.0" | DEC-047, original | `dithered_graphic.png` measures **7.08 at `--max 256`** (SPEC-109) |
| "≤3.64 counting dithers-of-photos" | DEC-047, **as revised by SPEC-109** | the same recipe on the Canon frame measures **3.8396** (SPEC-109 verify) |

Measured so far, 32-level Floyd–Steinberg: **3.6414** (Fuji), **3.8396** (Canon). Two frames is
not a corpus. The margin to 4.0 is **0.16 bits**, not the 0.36 the record implied.

**Why it matters, and why SPEC-108 does not cover it.** SPEC-108 fixes *scale-dependence* by
classifying the source rather than the resize output. It does not move the threshold. So if any
dither of a photograph exceeds 4.0 **at native size**, it classifies `photograph` on its own
merits and ships lossy — a hard-edged graphic smeared, which is the harmful error direction
DEC-047 exists to prevent. Placement does not save that case.

**What the work is:** measure the recipe across the available photographic corpus (not two
frames), establish the actual distribution and its upper tail, and only then decide whether 4.0
holds, needs to move, or needs a second discriminator. Cheap — it is measurement, not design.

**Note the pattern, since it has now repeated inside its own fix:** a value measured from one
specimen restated as a property of the class. [[a-guards-advertised-reach-is-a-claim]]. Whatever
number this work produces, record it as *"the maximum observed across corpus X"*, never as
*"the ceiling"*.
## Open question — are the 317 historical `estimated_usd` entries inflated? (2026-07-26)

**Not urgent. Nothing has been run; this is a note, not a finding.**

DEC-083 replaced the flat `tokens_total × list rate` cost rule with component pricing, after
SPEC-109's build measured a **14× overstatement** (98.7% cache reads: $588 flat vs $43.21 by
component). That fixes the method going forward. It says nothing about what is already recorded.

**The corpus:** 317 non-zero `estimated_usd` entries across `projects/PROJ-*/specs/`, summing
**$897.98**.

**The question that decides everything:** did the `subagent_tokens` figure those entries were
derived from **count cache reads**? If yes, they are inflated on the same order as SPEC-109's
measurement and the real lifetime spend is a small fraction of $897.98. If it counted only
non-cached tokens, they may be approximately right. **This has not been checked.**

**Why it matters beyond bookkeeping:** spend-per-spec is a figure that reads as precise and
gets repeated. It is exactly the kind of number that ends up in a launch post, a README, or a
"what did this cost to build" note. Getting it wrong in public by an order of magnitude is
worse than not quoting it.

**Suggested order when picked up:**

1. Establish what `subagent_tokens` counts — from the harness, not by inference.
2. Only then decide between: restate (likely **impossible** — old cycles almost certainly did
   not preserve a per-component breakdown), annotate the series with a dated methodology
   divider, or leave it and stop quoting the aggregate.
3. Whatever is chosen, `just cost-audit` needs no change — it checks presence, not method, and
   will keep passing either way. That is not evidence the numbers agree.

## Classifier review findings NOT taken into PROJ-010 (2026-07-26)

Of the 15 findings in `docs/research/pr113-classifier-review-findings.md`, STAGE-034 took the
placement bug, the test-integrity cluster, and seven more (rule 6's dead code, the `DOC_ENTROPY_MAX`
band, rule 5's reachability, the `Icon` ordering call, `decide.rs:150`'s missing lossless fallback,
the `iso_luma` fixture artifact, and `--profile docs`). **One was deliberately left here:**

- **Luma entropy ignores alpha** (`src/analysis/mod.rs:248`). RGB under fully transparent pixels feeds
  the histogram, and the new rule made it class-deciding: a flat logo with dirty transparent-background
  RGB measures **6.25 → `photograph`**, while the same file at `--max 500` measures **1.04 →
  `graphic-logo`**, because the resize zeroes transparent RGB. Same asset, opposite buckets, depending
  on whether a resize ran.

  **Status: PLAUSIBLE, not confirmed.** The mechanism is verified by reading source; the re-derivation
  of 2026-07-25 lists this as **COULD NOT TEST** — the dirty-alpha case was never attempted. It is not
  spec-able yet.

  **First task is a specimen, not a fix:** obtain a logo exported with dirty alpha (common from
  Photoshop/Sketch), confirm the 6.25/1.04 split reproduces, and only then decide whether it is a
  STAGE-034 follow-up or its own spec. Per [[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]],
  drive the failure path before designing against it.

## Open question — split the roadmap into internal and public (maintainer, 2026-07-26)

`docs/roadmap.md`'s "Sequencing rationale" is written from inside the AI-agent-experiment framing,
in a public file. The maintainer wants the roadmap split into an internal and a public document
but **has not framed the shape yet**.

This is recorded as an **open question awaiting his framing** — deliberately *not* filed as a
decision, and not to be acted on unprompted.

## Open — `resize` resamples in sRGB, not linear light (2026-08-15; **MEASURED 2026-08-16 — premise holds, alpha half refuted**)

> **Homed on STAGE-046** (output fidelity on shipped verbs, 2026-08-15). The evidence stays
> here; the schedulable work lives there. Sequenced ahead of STAGE-041 by maintainer decision.


**Measured, not speculative.** `Resize::apply` converts to RGBA8 and hands
`PixelType::U8x4` straight to `fast_image_resize` with Lanczos3
(`src/operation/mod.rs:515`). A grep for `gamma`, `linear`, and `premultipl` across
`src/operation/mod.rs` and `src/image/mod.rs` returns **zero hits**.

So crustyimg resamples non-linear sRGB values as if they were linear. The visible effect is that
high-contrast edges darken on downscale — worst on thin bright features against dark backgrounds.
This is a quality defect in the **most-used operation** of a tool whose headline claim is
quality-per-byte, so it sits closer to the `source_format`-truthfulness class than to a
nice-to-have.

**~~Adjacent and probably the same spec: premultiplied alpha appears absent.~~ REFUTED
(2026-08-16, SPEC-120 / DEC-092).** The original note — *"Resizing non-premultiplied RGBA produces
halos around transparent edges; `fast_image_resize` ships `MulDiv` for exactly this and documents
the hazard. **Not confirmed** — only two files were grepped"* — was right to distrust itself.
`fast_image_resize` 6.0.0's `ResizeOptions::default()` sets `mul_div_alpha: true`, and
`ResizeOptions::new()` **is** `Default::default()`; `Resize::apply` overrides only the algorithm,
so it **already premultiplies**. Measured: max premultiplied-RGB error at the alpha edge is
**27/255 (mean 0.364/255)**, versus **68/255 (mean 18.34/255)** for the same code with
`use_alpha(false)`. **The fix spec must not carry the alpha half.** Note *why* the grep could not
have found this: the behaviour lives in the dependency's default, not in `src/`.

**The fix does NOT require a 16-bit pipeline.** Convert to linear `f32`/`u16` *inside*
`Resize::apply`, resample, convert back to 8-bit on the way out. The `Operation` pipeline stays
8-bit; `fast_image_resize` (locked at **6.0.0**, not the 5.x this note first said) already
supports `U16x4`/`F32x4` and `MulDiv`, so the backend is in place — SPEC-120's prototype drove
`F32x4` through it and it works. This is deliberately **separate** from the open "should the
pipeline preserve >8 bits" question (16-bit PNG/TIFF inputs are truncated today — every op calls `to_rgba8()`,
`src/operation/mod.rs:197,396,816,894`). Two projects, separately schedulable; this one is
contained and benefits every user including the core JPEG→WebP path.

**What makes it non-trivial — do not discover this during build:** fixing the resampling
**changes output bytes for every existing recipe**, which invalidates every PROJ-007 build
lockfile. A quality improvement becomes a breaking change needing its own DEC and a migration
story. Open sub-question: does the build cache key need a pixel-depth / colour-pipeline-version
component so old and new renders cannot collide in the cache?

**~~Open question that decides whether the premise holds:~~ ANSWERED — YES (2026-08-16,
SPEC-120).** The gate asked: *"does SSIMULACRA2 (DEC-019) score the linear-light output better
than the current output on a representative downscale? If it does not, the premise is wrong and
this should be closed rather than specced."* It does, on every case tried, by 15 to 164 points.
**The premise holds; spec the fix.** Numbers and method below.

**A second consumer raises the stakes (2026-08-15).** For `resize` this is a quality defect. For
any **grading** op — `.cube` LUT above, curves, exposure — it is a **correctness** defect: the
pipeline is 8-bit throughout (`to_rgba8()` at `src/operation/mod.rs:197,396,816,817`), so a grade
is quantized to 256 levels per channel and evaluated against the wrong transfer function. Worse,
such an op's own tests **cannot see it** — reference and candidate are wrong identically. So this
question **gates the LUT entry above**, and should be answered before it is specced, not during.
(It gates any future grading surface for the same reason.)

**Context:** gamma-correct scaling is a commercial product elsewhere — imazen's `zenresize`
(9.8k downloads) ships `AGPL-3.0-only OR LicenseRef-Imazen-Commercial`, as does every codec and
operation in that stack; only their interfaces (`zenpixels`, `zencodec`) are permissive. Fixing
this closes a gap against a paid competitor with a permissive implementation. Related caution for
any future "can we just use theirs" question: `crabmagick-core` (unrelated author) shipped 0.1.0
and 0.1.1 as MIT/Apache on 2026-07-06/07, then relicensed to AGPL-3.0-or-later at 0.1.2 the next
day after adopting that stack.

### MEASURED — the premise holds; spec the fix (2026-08-16, SPEC-120, DEC-092)

**Verdict: *premise holds, spec the fix.*** Both oracles agree, in the predicted direction, on
every case.

The gate could not be run in the form it was written: SSIMULACRA2 requires equal dimensions
(`src/cli/report.rs:329`), so *"score the downscale against its source"* **errors rather than
answering**. The runnable shape supplies a reference at the target size and scores both candidates
against it — and the reference is generated **outside this codebase**
(**ImageMagick 7.1.2-29 Q16-HDRI**, `-colorspace RGB -filter Lanczos … -colorspace sRGB`), because
a reference produced by the code under test cannot fail the code under test.

Reference = the independent linear-light downscale. Luminance is BT.709 relative luminance on
**linearized** values; negative = darker, the direction the premise predicts.

| case | source → target | today: mean signed luma err | as % of ref | today SSIMULACRA2 | linear-light prototype | Δ |
|------|-----------------|----------------------------:|------------:|------------------:|-----------------------:|----:|
| synthetic worst case (positive control) | 2048² → 256² (8×) | −0.104350 | **−88.07%** | **−63.85** | 100.00 | **+163.85** |
| `graphic_large.png` | 512² → 128² (4×) | −0.001386 | −0.44% | **70.45** | 100.00 | **+29.55** |
| `photo_forest_cc0.jpg` | 800×532 → 200×133 (4×) | −0.004920 | −2.63% | **84.45** | 99.41 | **+14.96** |

**The instrument was proven able to fire before any of this was believed.** SSIMULACRA2 eats 8-bit
sRGB and is tuned for compression artifacts, so whether it registers a systematic luminance shift
was itself unknown — and a null would have had two opposite readings (*premise false* vs *wrong
instrument*). The synthetic worst case settles it: an −88% physical luminance error registers as a
**163.85-point** swing. The metric can see gamma-incorrect resampling, so the realistic rows mean
what they say.

**The alpha half got its own oracle, and its own control.** SSIMULACRA2 never sees alpha
(`to_rgb8()`, `src/quality/mod.rs:68`), so it is structurally incapable of measuring a
transparent-edge halo. Method: a 1024² hard-edged opaque shape over a fully-transparent surround
carrying maximally contrasting RGB ("dirty alpha"), downscaled 8× to 128²; both sides resampled in
sRGB space so premultiplication is the only variable; the number is the **max per-channel
difference in premultiplied 8-bit RGB over the 6301-pixel band where either image has
`0 < alpha < 255`** — that is the visible composite error, and it is background-independent
because the two alphas agree (mean disagreement 0.42/255). **The number: 27/255 max, 0.364/255
mean** — and the same code with `use_alpha(false)` measures 68/255 max, 18.34/255 mean, which is
how we know the oracle can fire. There is no halo defect to fix.

**Carry this into the fix spec: the two metrics agree in direction but are not interchangeable.**
On `graphic_large.png` the *mean* luminance error is only −0.44% while the perceptual penalty is
29.5 points, because the error concentrates at edges — max local |luma err| **0.213** against a
mean absolute of 0.0023, ~90×. Mean luminance **understates** the defect on exactly the content
class the premise says is worst hit. A fix spec that gates on mean luminance alone would
under-report its own win.

**Caveat, stated rather than smoothed over:** the prototype scores ~100 against the reference
partly because it implements the *same* algorithm as the reference. The load-bearing measurement
is **crustyimg-today's score against a correct reference** (−63.85 / 70.45 / 84.45), not the exact
magnitude of the delta. A production linear-light resize will not necessarily score 100.

**Re-derive it rather than trusting it** — the harness is committed, runs offline apart from
`magick`, and produced byte-identical reports on two clean runs:

```sh
cargo build --release && cargo build --release --example spec120_linear_probe
python3 scripts/spec120_linear_light.py            # table
python3 scripts/spec120_linear_light.py --json     # machine-readable
```

It reports five controls every run, none of them assumed: the prototype's sRGB arm is
**pixel-identical** to the shipped `crustyimg resize --exact` (so the delta is the transfer
function and nothing else); ImageMagick's own sRGB-vs-linear resize moves by the same −88%/−0.44%/
−2.63%; crustyimg today agrees with ImageMagick's sRGB-space resize to a mean |luma err| of
0.0003 or better (so the gaps are gamma, not filter drift); and the two alpha controls above.

**Everything else in this entry stands** — the breaking-change consequence, the invalidated
PROJ-007 lockfiles, the open cache-key sub-question, and the grading-op stakes. What changed is
that the fix is now justified by measurement instead of plausibility, and it is **one** premise
rather than two.

---

## Open — workhorse items with no home (2026-08-15)

`docs/feature-set-triage-2026-08.md` opens by warning that it is *"a triage, not a backlog.
Nothing here is committed work."* Checked 2026-08-15 with a word-boundary grep and a positive
control (`crop`, which **is** homed at roadmap Wave 5 + the Geometry-extras table above): three of
its strongest items are homed **nowhere** — zero hits across `backlog.md`, `roadmap.md`,
`feature-exploration.md` and `moat.md`. Each is a crustyimg feature on its own merits and is
recorded here as one. *(They went unhomed because they were parked behind the then-open lab
question; that is history, not a dependency — see DEC-091 §Consequences.)*

### `.cube` LUT op — build the reader in-house

**Workhorse, not lab** (Fence A: a LUT path + strength generalizes across a batch). The feature
was never in question; only the dependency advice was, and the maintainer reversed that on
2026-08-10 in favour of in-housing.

**The differentiator is the reproducible build, not the LUT.** *"The LUT inside a reproducible
build — the file's hash becomes part of the cache key, so changing the grade invalidates exactly
the affected outputs and `build --check` catches an accidental grade change in review."* Nothing
else occupies that. It is also what unlocks §16 (brand consistency as a build gate), the one
commercial angle in the triage set.

**Cost, measured rather than estimated (2026-08-15):** `lut-cube` 0.2.0 — the closest real
comparable — is **329 lines** (`lib.rs` 53 + `cube.rs` 59 + `lut.rs` 217). The triage's "~100 to
parse, ~50 to interpolate" is optimistic by roughly 2×. **Budget 250–350 lines** for parse +
trilinear + typed errors, against the `src/metadata/tiff.rs` (718-line) in-housing precedent.

**Two corrections to the triage's dependency note**, both checked 2026-08-15:
- Its licence is **plain MIT** (`LICENSE`, Copyright (c) 2023 Yury Korolev). crates.io reports
  "non-standard" only because the manifest uses `license-file` rather than `license`. The
  practical conclusion survives — `cargo deny` still cannot read it without a clarification entry
  — but "the licence is unclear" should not be repeated as a fact.
- A **better, independent** reason to decline it: its only tests hardcode absolute paths into the
  author's local DaVinci Resolve install (`src/lib.rs:28,33`), so they cannot run on any other
  machine. That is a supply-chain quality signal the licence question distracted from.

⚠ **Blocked on the colour-space question below** — a `.cube` applied in an 8-bit, non-linear
pipeline is baked against the wrong transfer function. Settle that first.

### MCP server exposing crustyimg's measurements — and its stated gate is questionable

The triage calls this *"the strongest strategic idea in the set"* on the reframe that **crustyimg
is a measurement instrument, and measurement is what LLMs are worst at**. It notes the item needs
**no new capability**: `lint --format json`, `diff --json`, `info --json` and `build --check` all
ship today.

**The gate needs checking before it is inherited.** The triage marks §17 *"gated on §4 (lint in
wasm)"*, and §4 on a bundle-size measurement. But a **native** MCP server needs no wasm at all.
That gating only holds if the distribution plan is npm-via-wasm. **If it isn't, the strongest item
in the set is unblocked right now.** Ten minutes to settle; do that before scheduling either.

**Why it may outrank more lint rules:** if `lint` is under-adopted, the bottleneck is
distribution, not breadth. This adds a new *consumer class* for output that already exists,
rather than more output nobody consumes. See STAGE-014's replaced gate.

### Perceptual dedup lint rule

Triage verdict: *"best differentiation per unit of effort in the set"* — fits the existing `lint`
framework with no new command, and image bloat in git history is permanent. Re-probed 2026-08-15:
**`image_hasher` v3.1.1, MIT OR Apache-2.0, 821,062 downloads, updated 2026-02-21** — healthy, and
the correct pick over `img_hash` (last updated 2021-05-04, effectively abandoned).

⚠ A **new top-level dependency** → needs a `DEC-*` first (`no-new-top-level-deps-without-decision`).

---

## Open — the public API leaks two crates it does not re-export (2026-08-15)

**A published-library correctness issue today, independent of lab.** crustyimg ships to crates.io
as a library, and two crates appear in its **public API** without being re-exported:

- `Image::pixels(&self) -> &::image::DynamicImage` (`src/image/mod.rs:269`) and
  `Image::with_pixels` (`:321`)
- `OperationParams::from_map(BTreeMap<String, toml::Value>)` (`src/operation/mod.rs:48`)

So any consumer implementing an `Operation` must declare `image` and `toml` itself. **Measured
consequence:** a consumer writing the obvious `image = "0.25.10"` adds **ten features** to the
`image` crate — `dds`, `exr`, `ff`, `hdr`, `pnm`, `qoi`, `tga`, `rayon`, `default`,
`default-formats` — because cargo features are additive across the graph. And they are
**reachable**, not merely linked: decode dispatch is
`ImageReader::new(..).with_guessed_format()` (`src/image/mod.rs:521-522`).

**A consumer therefore silently widens crustyimg's accepted-input surface to six decoders
`tests/hostile_inputs.rs` has never fuzzed**, without adding a crate or tripping any gate.

**Fix:** `pub use ::image;` and `pub use ::toml;` in `src/lib.rs` — re-exports, not visibility
widenings — plus a test asserting the resolved feature set. Cheap, and it closes the hole for
every consumer, not just a future lab.

### Companion: a public-API contract test for the registry seam

**crustyimg makes a promise to library consumers in its own shipped rustdoc.**
`src/operation/registry.rs:60-62` tells them new operations register from outside *"without
touching the recipe parser (the whole point of the registry seam)"*. Nothing tests that promise.

The repo already tests exactly this class of claim: `tests/spec097_reexport_paths.rs` exists
solely so that every path that was `pub` at `crustyimg::cli::*` still resolves, and it fails to
compile if one is dropped, renamed, or narrowed. **This is that test for the `Operation` seam.**

The property was **verified 2026-08-15** by compiling an out-of-crate probe (implement
`Operation` → `register` → `from_toml` → `build_pipeline` → `run` → `from_ops`/`to_toml` →
`encode_to_bytes` → `quality::score` → `Analysis::compute`): `cargo check` exit 0, **zero `pub`
widenings required**, with an `E0603` negative control proving the probe can fail.

**Nothing keeps it true.** STAGE-042/043/045 all touch core paths, and a consumer only finds out
at `cargo build`. A committed version of that probe — with its negative control — turns the
rustdoc promise into a gate, and catches the feature-unification hole above at the same time.
Reproduction recipe: `docs/lab-plan-2026-08.md` §Appendix. (It also happens to protect DEC-088's
premise, but the contract stands on its own.)

---

## Open — the pixel core uses 4 of ~48 `imageops` functions (2026-08-15)

**Measured, not estimated.** `image::imageops` exposes ~48 public functions. crustyimg calls
**four**: `resize`, `overlay`, `crop_imm`, and one bare path. Everything below is **already
compiled into every default binary** and unused:

| group | available now, unused |
|---|---|
| tone / colour | `brighten`, `contrast`, `huerotate`, `grayscale`, `grayscale_with_type`, `unsharpen` |
| blur | `blur`, `fast_blur`, `blur_advanced` (+ `new_from_sigma` / `new_from_radius` kernels) |
| geometry | `crop`, `rotate90/180/270`, `flip_horizontal/vertical`, `tile`, `replace` |
| convolution | **`filter3x3(image, &[f32; 9])`** — arbitrary 3×3 kernels |
| palette | `dither`, `index_colors`, the `ColorMap` trait |
| sampling | `interpolate_bilinear`, `sample_bilinear`, `sample_nearest`, gradients |

**Why this matters for planning:** the *Effects catalog* and *Geometry extras* tables above are
costed as if each item is an implementation. Most are a **wrapper over a function already in the
binary** plus params, `params()` round-trip, and tests. Re-cost them before they are specced —
the work is registry wiring and test coverage, not pixel math.

**`tiny_skia` is likewise under-used.** Reached today only for SVG rasterization
(`src/image/svg.rs:55`, via resvg's re-export), it also ships linear/radial/**sweep** gradients,
patterns, path stroking with AA, and ~15 Porter-Duff blend modes (`Screen`, `Modulate`, `Plus`,
`Xor`, `SourceAtop`, …). Any compositing feature — duotone, glow, vignette, gradient map — has its
engine in the binary already.

⚠ **Scope guard.** Availability is not a reason to build. These pass DEC-091's two fences (their
params generalize; their output is a build artifact), which makes the fences the *wrong* filter
here — the operative test is `docs/territory.md`'s **"does this help produce a better artifact
automatically?"**. On that test `dither`/`index_colors` (smaller PNGs), `unsharpen` (recovers
downscale softening, measurable in SSIMULACRA2), `blur` (Wave 4 placeholders) and declared 3×3
kernels earn their place; a vignette or a film-grain filter does not. **Do not let "it's free"
become an effects backlog.**

⚠ **All of it is bounded by the 8-bit sRGB pipeline** (see the resampling entry above): blurs,
gradient maps and curves band at 256 levels/channel, and compositing in non-linear space makes
blends and glows subtly wrong.

---

## ⚠ Live defect — ops widen to RGBA and never narrow back (2026-08-15)

> **Homed on STAGE-046** (output fidelity on shipped verbs, 2026-08-15). The evidence stays
> here; the schedulable work lives there. Sequenced ahead of STAGE-041 by maintainer decision.


**Not a roadmap candidate — wrong today, on shipped verbs, including the flagship.** Same class as
D-1: a default path that hands the user a worse result than it received. Measured by driving
`target/debug/crustyimg` (built 2026-08-14, newer than all three source files) against
purpose-built PNGs and reading the output **IHDR** — a structural assertion, not a byte guess.

### The measurement

Source: 64×64 PNG, `bit_depth=8`, **`colour_type=2` (RGB, no alpha)**.

| verb | out `colour_type` | verdict |
|---|---|---|
| `convert --format png` | 2 (RGB) | ✅ correct |
| `optimize` | 2 (RGB) | ✅ correct |
| `auto-orient` | 2 (RGB) | ✅ correct |
| `resize --max 32` | **6 (RGBA)** | ❌ alpha invented |
| `thumbnail --size 32` | **6 (RGBA)** | ❌ alpha invented |
| `edit --invert` | **6 (RGBA)** | ❌ alpha invented |
| **`web`** | **6 (RGBA)** | ❌ **the flagship verb** |
| `watermark --text` | 6 (RGBA) | arguable — compositing genuinely needs alpha |

The clean verbs are exactly the ones that run **no `Operation`** (`convert`/`optimize` use the
decide/encode path; `AutoOrient` returns `img` unchanged or clones the variant). Every verb that
folds a real op through the pipeline comes out RGBA.

### Root cause — one call, three sites

`Operation::apply` implementations widen unconditionally and never narrow back:

- `Invert::apply` — `img.pixels().to_rgba8()` (`src/operation/mod.rs:197`)
- `Resize::apply` — `img.pixels().to_rgba8()` (`src/operation/mod.rs:396`)
- `Watermark::apply` — `src/operation/mod.rs:816-817`

`web` is `auto-orient → resize → optimize`, so **fixing `Resize` fixes the flagship.**

### Why it matters more than it looks

**It contradicts the product's core claim.** An all-opaque alpha channel is pure waste, and
crustyimg's whole thesis is quality-per-byte. Measured on a 512×512 representative PNG, same
pixels: RGB **377,132 B** vs RGBA **423,756 B** — **+46,624 B, +12.4%**, for a channel carrying no
information. `optimize` may re-decide a format and mask this on some paths, but `resize`,
`thumbnail` and `web` write it out.

### The same call also truncates 16-bit — one fix, two defects

`to_rgba8()` is *also* why 16-bit input is silently halved, and the two are one code change.
Measured on a 32×32 `bit_depth=16` RGB PNG:

| verb | out | |
|---|---|---|
| `convert` / `auto-orient` | **16-bit, RGB** | ✅ already preserved |
| `resize` / `edit --invert` | 8-bit, RGBA | ❌ halved **and** alpha-added |

**So "crustyimg is 8-bit internally" is not accurate as stated.** Decode preserves the
`DynamicImage` variant, `Identity` and `AutoOrient` preserve it, and the default encode path
(`img.pixels().write_to(..)`, `src/sink/mod.rs:718`) preserves it. **Only the three op bodies
above collapse it.** `Image` already wraps `DynamicImage`, which already has `ImageRgb16` /
`ImageRgba16` variants — **no type change is needed anywhere.** `fast_image_resize` 5.x already
ships `U16x4`/`F32x4` (see the resampling entry above).

### What the fix has to decide

- **Narrow-back policy.** Simplest correct rule: an op preserves the input's colour type and bit
  depth unless it genuinely needs more (compositing a translucent overlay). "Widen to work, narrow
  to write" is the shape.
- **Lossy targets are 8-bit** (JPEG, lossy WebP) — that downgrade should be **reported**, in the
  spirit of SPEC-090's honest size reporting, not silent.
- ⚠ **It changes output bytes for existing recipes**, so it invalidates PROJ-007 build lockfiles —
  the same migration cost the linear-light entry carries. **Sequence the two together**; paying
  that migration twice would be avoidable waste. (The linear-light entry is right that they are
  *technically* independent; this is about blast radius, not dependency.)
- **No new flag.** The correct behaviour is "preserve what you were given when the target format
  can hold it", which the user should not have to ask for.

---

## Metadata-driven text — the differentiated half of `watermark --text` (2026-08-15)

`src/text/mod.rs` (351 lines) renders **one line** of text — no newline handling, no wrapping, no
kerning (`render_text(font_bytes, text, size_px, color) -> RgbaImage`) — composited through the
`Watermark` op's gravity.

**The commodity part is compositing a string. The differentiated part is where the string comes
from.** Stamping capture date / camera / lens / exposure from the image's own EXIF is:

- **already supplied by the container lane** — `kamadak-exif` read ships today, so **zero new
  dependencies**;
- **Fence-A clean** — the *template* generalizes across a batch even though the *value* is
  per-image, exactly as the sink's existing `{stem}` name template does. That precedent is the
  design to copy: `{exif:Model}`, `{exif:DateTimeOriginal}`, `{width}`, `{stem}`;
- **in-territory** — it produces a delivery artifact automatically, which the effects catalog's
  stylistic filters do not.

Supporting work, in rough value order: multi-line + wrapping (the natural completion of the
existing primitive) · stroke/shadow for legibility over busy photos (`tiny_skia` can do it, already
linked) · a caption bar that extends the canvas, which needs `pad`/`extend` from the Geometry
extras table · contact-sheet cell labels.

⛔ **Out of scope, firm tier:** OG-card rendering. `docs/territory.md:104` — *"No HTML generation,
templating, routing, or OG-card rendering in crustyimg."* That is the separate web-content tool's
job, with the manifest as the seam. Worth stating explicitly because it is the most obvious
"text beyond watermark" idea and it is already excluded.

---

## Animated AVIF vs animated WebP — AVIF wins on all three axes (2026-08-16)

> The gating decision for STAGE-046's animated-*output* spec. Measured against the **reference
> implementations** (libavif 1.4.2 / aom 3.14.1, libwebp 1.6.0) used as oracles — **not** proposed
> for adoption. Status: **recommendation, unscheduled.**
>
> Consolidated into the draft at `docs/research/draft-spec-animated-avif-output.md` (AC-6, AC-7).

### 1. Size at matched quality — AVIF by ~6x

Same source (`Newtons_cradle`, 308,156 B GIF, 36 frames, 480×360). Every candidate scored with
**crustyimg's own oracle** (`crustyimg diff`, SSIMULACRA2) against the *same* reference: frame 0 of
the source GIF.

| encode | bytes | ssim2 | vs GIF |
|---|---:|---:|---:|
| **avif (aom) q30** | **27,309** | **86.9** | **11.3×** |
| avif q40 | 40,030 | 88.2 | 7.7× |
| avif q80 | 95,152 | 91.0 | 3.2× |
| webp q50 | 109,998 | 75.7 | 2.8× |
| webp q75 | 146,154 | 81.7 | 2.1× |
| **webp q80** (best measured) | **172,492** | **84.1** | 1.8× |

**AVIF q30 beats WebP's best point on both axes at once** — higher quality (86.9 vs 84.1) in
**6.3× fewer bytes**. AVIF q40 is 4.3× smaller *and* 4 points better. Animated WebP never reached
86.9 in the sweep; at q90 it was 317,816 B, **larger than the source GIF**.

**And the result transfers to the encoder crustyimg would actually use.** rav1e measured
27,564 B @ ssim2 86.7 — landing essentially on top of aom's q30 (27,309 B @ 86.9). The reference
encoder is not flattering AVIF here; rav1e is competitive on this content.

The comparison is **conservative toward AVIF**: its path carries an inherent 95.92 ceiling from the
hand-rolled YUV420 conversion feeding it (`crustyimg diff ref0 src0` = 95.92), while `gif2webp` does
its own, likely better, RGB→YUV internally.

### 2. Browser support — the axis WebP wins, and it no longer disqualifies AVIF

**Animated** AVIF (distinct from still AVIF, which landed earlier in each engine): Chrome **93+**,
Firefox **114+**, Safari **17.0+**; ~94–95% global coverage. Animated WebP is effectively universal
(~97%). So WebP is wider, but AVIF is past the threshold where it needs to be hidden behind a flag.
**Sourced from secondary trackers, not vendor release notes — worth one confirmation pass against
caniuse before it becomes a spec premise.**

### 3. Pure-Rust feasibility — AVIF wins this one too, which is the surprise

The intuition is that WebP is the easier build. It is not:

- **AVIF**: `rav1e` already encodes (in-tree, BSD-2) and `mp4-atom` already has `av01`/`av1C` plus an
  `avis` test. Both halves exist, permissively licensed.
- **WebP**: there is **no pure-Rust animated WebP encoder at all**. `image-webp` 0.2.4's encoder
  writes `VP8X` (`src/encoder.rs:722`) but emits no `ANIM`/`ANMF`, so animation would need libwebp
  (C, and `webp-animation` wraps exactly that) or a from-scratch VP8 encoder.

### Recommendation

**Animated AVIF as the primary output**, with the WebP-fallback question deferred rather than
answered — crustyimg already emits multiple formats behind a manifest (`responsive`/`build`), so
"AVIF plus a fallback" is an existing pattern, not new machinery. Shipping animated WebP *from pure
Rust* is the genuinely blocked path, and that is the opposite of the going-in assumption.

### Method — three wrong hypotheses before the real cause, all falsified by targeted tests

The first AVIF numbers were **invalid** and looked plausible: scores flat at ~56–57 while bytes more
than doubled. Recorded because the failure mode is subtle and will recur.

1. *"BT.709 vs BT.601 matrix mismatch."* Falsified — switching both directions to BT.601 moved the
   score by 0.05.
2. *"avifdec is returning the wrong frame."* Falsified — scoring the output against source frames
   0/5/17/35 gave 57.4/46.7/41.7/29.5, monotonically decreasing, so it *was* frame 0.
3. **The actual cause: `avifdec -i` reported `Range: Limited`.** The y4m carried full-range 0–255
   YUV; avifenc tagged the output limited-range 16–235, so the decoder expanded it and crushed
   contrast — a constant error invariant to quality, which is exactly the flat curve. Fixed by
   adding **`XCOLORRANGE=FULL`** to the y4m header.

**The control that made this findable**: a near-lossless encode (`-q 100`) must score *high*. It
scored **57.2**, which cannot be compression loss on a 322,981-byte encode of a 308,156-byte source.
After the fix the same control scored **96.5**. Without that control the flat curve would have been
written up as "AVIF is worse than WebP" — the exact opposite of the truth.
[[a-control-you-never-verified-applied-is-not-a-control]]

## Multi-frame format sweep — `format/animated-gif` is too narrow by FOUR, not one (2026-08-16)

> The mechanical sweep the animated-input defect entry called for. **Driven on the shipped 0.7.0
> binary**, with fixtures built independently of the code under test. Directly de-risks **SPEC-119**,
> and finds two loss cases **outside** its AC-5 scope.

### The mechanical claim, with its grep

crustyimg's resolved `image` feature set is `avif, bmp, gif, ico, jpeg, png, tiff, webp`
(`cargo tree -f '{p} FEATS[{f}]'`; `Cargo.toml:139` plus `avif` from the default feature).

`AnimationDecoder` is implemented by exactly **three** decoders in `image` 0.25.10
(`grep -rn "impl.*AnimationDecoder"`):

- `GifDecoder`  — `src/codecs/gif.rs:426`
- `ApngDecoder` — `src/codecs/png.rs:514`
- `WebPDecoder` — `src/codecs/webp/decoder.rs:104`

**But animation is not the only multi-image case**, and this is what the rule *name* cannot reach:

- **TIFF** — `TiffDecoder` exposes **no multi-page API** (`grep -n 'next_image\|more_images\|fn new'
  src/codecs/tiff.rs` → `fn new` only). Pages 2..N are unreachable *and* undetectable through
  `image`. Reaching them needs the `tiff` crate directly.
- **ICO** — `IcoDecoder::new` calls `best_entry(entries)` (`src/codecs/ico/decoder.rs:147,194`),
  scoring on `(bits_per_pixel, width × height)` and **discarding every other entry**.

### Driven: what actually survives a `convert`

Fixtures built independently ([[fixtures-from-the-code-under-test-cannot-fail]]) — two real Wikimedia
animations, and a hand-written TIFF/ICO whose structure was verified by walking the IFD chain and the
icon directory *before* conversion.

| source | contained | output | kept | **lost** | `lint` warns? |
|---|---|---|---|---|---|
| animated GIF | 44 frames | 400×400 png | frame 1 | **43 frames** | **yes** |
| **APNG** | 20 frames (`acTL` + 20 `fcTL`) | 100×100 png | frame 1 | **19 frames** | **no** |
| **multi-page TIFF** | 3 pages, greys 70/140/210 | 8×8 png, pixel = **70** | page 1 | **2 pages** | **no** |
| **multi-size ICO** | 16/32/64 = red/green/blue | **64×64** png, pixel = **(0,0,255)** | the 64px | **16px + 32px** | **no** |

**Every one exits 0 and says nothing.** The output pixel values are the proof, not the exit code: grey
`70` is page 1 of 3, and blue at 64×64 is `best_entry`'s pick out of three.

The GIF row doubles as the **positive control** — the linter demonstrably *can* warn, so the three
silent rows are narrowness, not a broken harness.

### What this means for SPEC-119

- **AC-5 is correctly scoped for animation** (GIF + APNG + animated WebP == the `AnimationDecoder`
  set, confirmed mechanically). No change needed there.
- **`lint` today covers 1 of those 3.** APNG and animated WebP decode multi-frame and are flagged by
  nothing — so AC-7's `fix:` repair must not be GIF-only either.
- **Two cases sit outside AC-5 entirely** and should be triaged, not silently inherited:
  - **Multi-page TIFF is the clearer gap** — scanned documents are routinely multi-page, and page 1
    is kept with no signal. Note the constraint: `image` gives no route to the other pages, so this
    is a *detect-and-warn* item, not a *convert-them-all* item, unless a new dependency is taken.
  - **Multi-size ICO is arguably correct behaviour** — an `.ico` is a container of sizes and picking
    one is the point. But the pick rule is undocumented and invisible, and "I converted my favicon
    and got only the 64px" is a legitimate surprise. Lowest priority of the four; **state it, do not
    necessarily fix it.**

### The rule name

`format/animated-gif` cannot describe any of this. The finding is **multi-image input is silently
flattened**, of which animated GIF is one instance of four. Renaming is a `lint` rule-id change and
therefore a compatibility surface (SARIF/JSON consumers) — flag it in SPEC-119's design rather than
doing it incidentally.

## Animated GIF → animated AVIF — measured, and it is a 9–11x win (2026-08-16)

> **Evidence for STAGE-046's animated-*output* spec** (backlog piece **(b)**), not for SPEC-119,
> which is scoped to stopping the data loss. Probe committed at
> `docs/probes/animated-gif-to-av1-probe.rs`. Status: **measurement, unscheduled.**
>
> **A drafted (unscheduled, unowned) spec consolidating this and the two entries above it is at
> `docs/research/draft-spec-animated-avif-output.md`** — `status: idea`, no SPEC number claimed, not
> attached to any stage.

### The whole path is pure Rust, patent-clear, and already in the tree

Driven end to end out-of-crate, exit 0: **real animated GIF → `image`'s `AnimationDecoder` →
`Image::from_parts` → crustyimg `Pipeline` (registered op, per frame) → `rav1e` AV1 → decoded back
with `re_rav1d` and frame-count verified.** No C, no ffmpeg, no new system dependency. **AV1 is
royalty-free by design**, so none of the AVC/HEVC patent problem in the assessment below applies.

### Measured on real Wikimedia animations, not synthetic fixtures

`Newtons_cradle_animation_book_2.gif` — 480×360, 36 frames, **GIF 308,156 B**. Quantizer swept at
fixed speed 6, then speed swept at fixed quantizer — **one variable at a time**:

| setting | AV1 bytes | ssim2 (frame 0) | vs GIF |
|---|---:|---:|---:|
| s6 q60 | 44,678 | 89.8 | 6.9× |
| **s6 q80** | **34,979** | **88.5** | **8.8×** |
| s6 q100 | 27,705 | 86.8 | 11.1× |
| s6 q120 | 21,324 | 83.9 | 14.5× |
| s6 q140 | 15,999 | 78.6 | 19.3× |

`Rotating_earth_(large).gif` — 400×400, 44 frames, **GIF 1,001,718 B** → 93,835 B at s6/q100
(**10.7×**), but ssim2 76.1: a harder, higher-motion source needs a lower quantizer. **Which is the
design conclusion — drive the existing quality search (`src/quality/mod.rs:255`) per sequence, not a
fixed quantizer.** The machinery already exists.

### Finding: speed 10 is a trap for animation. Do not inherit the still-image intuition.

At a fixed quantizer, holding everything else constant:

| speed | bytes | ssim2 |
|---|---:|---:|
| 1 | 24,110 | 86.9 |
| 4 | 25,427 | 86.6 |
| **6** (`AVIF_SPEED`, `src/sink/mod.rs:48`) | 27,705 | 86.8 |
| 8 | 27,563 | 86.1 |
| **10** | **38,061** | **84.4** |

**Speed 10 is 37% LARGER *and* lower quality than speed 6.** That inverts DEC-068's still-image
finding (speed 10 ≈ 3.6× faster for ~4% more bytes), and the reason is structural: speed 10 guts
motion estimation, and on a sequence inter-frame prediction is where the entire win lives — every
encode above produced exactly **1 keyframe** for 36 frames. crustyimg's existing `AVIF_SPEED = 6`
is already the right default here; the risk is a future session "optimising" animation with the
wasm speed knob (SPEC-079/DEC-068) and silently making output both bigger and worse.

**This was caught by a methodology error worth recording**: the first run varied quantizer *and*
speed together and produced a smaller file scoring *higher*, which is impossible if quantizer were
the only variable. Sweeping one variable at a time made it monotonic and explained it.
[[a-control-you-never-verified-applied-is-not-a-control]]

### Caveats — none change the conclusion, all change the spec

- **These are raw OBU bytes, not a container.** ISO-BMFF overhead is ~1.5–3 KB at these frame counts
  (`stsz` alone is 4 B × frames) — ~2–3% on the 94 KB payload, ~7–10% on the 28 KB one. Budget it.
- **ssim2 is frame 0 only — the keyframe, i.e. the optimistic end.** Inter frames will score lower.
  A real implementation must score across frames, and per [[a-self-referential-control-cannot-detect-a-broken-pipeline]]
  the *frame count* assertion stays separate from the *quality* assertion regardless.
- **Animated-AVIF browser support was not verified** in this session. It is the gating question for
  choosing AVIF over animated WebP, and it is a lookup, not a probe.
- `mp4-atom` (see the entry below) supplies the `av01`/`av1C` boxes and has a committed `avis`-brand
  test, so the muxing half has a permissive, pure-Rust candidate. Adoption needs a DEC.

### Why this matters for the defect

Today the tool turns this 1,001,718 B / 44-frame GIF into a **118 B static WebP** and reports
`72% smaller · ssim 100.0`. The honest alternative is ~94 KB carrying **all 44 frames** — a real
10× win against the GIF instead of a fabricated one against a single frame.

## Video asset lint — a container-only rule family, with a driven field audit (2026-08-16)

> Follows the DECLINED video-tool assessment below. **This is the part of the category that is not
> blocked by codec patents**, because it never decodes a frame — it parses ISO-BMFF box structure
> only. Status: **candidate, unscheduled.** Nothing committed.

### The idea

Extend `crustyimg lint` with a video-asset rule family. Every check below reads container metadata,
so it needs **no codec, no C, and no patent exposure** — the wall in the assessment below is
specifically on *decoding pixels you did not create*, and box parsing is not codec implementation.

- **`moov` after `mdat` (no faststart)** — playback cannot begin until the whole file downloads.
  The classic web-video defect; `ffmpeg -movflags +faststart` is the folk remedy.
- **an audio track on an autoplay-muted loop** — bytes nobody ever hears.
- **HEVC-only (`hvc1`/`hev1`)** — no Firefox support; needs a `<source>` fallback.
- **resolution far above the display slot**; container/extension mismatch; absurd GOP length.

**The faststart *fix* is also container-only**: reorder `moov` ahead of `mdat` and rewrite the
chunk-offset tables (`stco`/`co64`). It never touches the codec bitstream. `mp4-atom`'s own
`examples/info.rs` is the read half; its `WriteTo` trait is the other half.

### Field audit — driven 2026-08-16, and the findings are not boring

A throwaway ISO-BMFF walker (box structure only, HTTP ranged GETs of the first 256 KB) was pointed
at **18 production MP4s** discovered from the homepages of ~20 major tech companies.

| finding | count | consequence |
|---|---:|---|
| **missing faststart** | **6/18** | playback blocked until fully downloaded |
| carries an audio track | 5/18 | bytes shipped for muted loops |
| HEVC-only (`hvc1`) | 5/18 | Firefox cannot play it |
| **AV1 (`av01`)** | **0/18** | nobody ships the royalty-free codec |

Verified at byte level rather than trusted to the parser —
`docker.com/.../AgenticCompose-web-1080.mp4` reads `ftyp` → `free` → `mdat` with
`0x0039df35` = **3,792,181 bytes** of media before `moov`; its sibling `sbx-rev2-1.mp4` reads
`ftyp` → `moov` immediately. **Same company, same CDN path, both patterns** — which is the whole
argument for the rule: this is inconsistency between exports, not a considered choice, and it is
exactly what an automated check catches.

Two individual findings worth keeping:

- `videos.ctfassets.net/.../web-homepage-hero-1920x1200_final.mp4` is **actually 3840×2400**
  (`tkhd` raw `0x0F000000`). Scored first as a parser bug, then confirmed as a real mislabelling —
  the *filename* is wrong, not the parse.
- `linear.app/static/pwa.webm`, as it appears in page source, returns **HTTP 404** as `text/html`
  with `cache-control: max-age=21600`.
- The most concentrated example: **`webflow.com`'s home hero** — no faststart, HEVC-only, *and* an
  audio track. Three findings on one asset.

### Method notes, including a false positive of my own

- **Controls first.** The faststart detector was proven to discriminate on synthesised
  `ftyp/moov/mdat` vs `ftyp/mdat/moov` byte sequences **and** on a 64-bit `largesize` `mdat`, before
  any real asset was fetched. [[a-plausible-test-result-is-not-a-checked-one]]
- **Two "broken" results in the first run were my bug, not the assets'** — `\x1aE\xdf\xa3` is EBML
  magic, i.e. legitimate WebM that an ISO-BMFF-only parser cannot read. Recorded because a linter
  that reports "not a video" for every WebM would be worse than no linter.
- The `tkhd` dimension parse was off by 4 bytes on the first attempt (the version-dependent block);
  caught because a result disagreed with a filename by exactly 2×, then confirmed against raw hex.
- **The HEVC finding is softer than the count suggests**: an `hvc1` asset may have a `<source>`
  fallback this audit did not check. The honest rule output is *"confirm you have a fallback"*, not
  *"broken"*. Likewise "muted" is inferred from these being marketing loops, not read from the
  `<video>` tag.
- 18 assets from one afternoon is suggestive, not a survey.

### The open question is demand, not feasibility

Every company audited has a performance team and ships these defects anyway. That reads two ways —
nobody checks (a checker is valuable), or nobody cares (impact too small). The evidence leans to the
first for **faststart specifically**, because it is a visible startup delay rather than a few hundred
wasted KB, and because the same-CDN split above shows inconsistency rather than intent.

### Placement

**Workhorse, and specifically `lint` — not a new binary.** Fence A: the parameters (expected codec,
max resolution, faststart required) are invariant across a batch. Fence B: the output is a decision
a build acts on, with a CI exit code, which is exactly what `lint` already is. It extends a territory
claim already held (`docs/territory.md`'s source-file, no-URL, pre-deploy lint against Lighthouse's
deployed-URL shape) rather than opening a new front. **Adopting `mp4-atom` would need a DEC** under
`no-new-top-level-deps-without-decision`.

**Anti-goal, restated:** this rule family must never decode a frame. The moment it wants pixels it
has walked into the patent problem the assessment below declines.

## Video tool on crustyimg — assessed and DECLINED (2026-08-15)

> Full evidence: `docs/video-tool-assessment-2026-08.md`. Read-only ideation session; nothing built.

**Verdict: do not build it.** The crustyimg dependency is load-bearing for *video → images* and not
for *video → video* (the shared core is 4,145 of 28,920 src lines, and the registry's four ops
reduce to one usable op for video — `resize`). No wedge against ffmpeg survives: the permissive-licence
angle trades a known LGPL obligation for an unquantified AVC **patent** exposure a licence does not
shelter, and "safe on untrusted input" would get *weaker*, since crustyimg's claim is strong precisely
because it declines the unsafe formats. The non-audio value already ships — driven on the released
0.7.0 binary, `crustyimg web <frames-dir> --out-dir <out>` exits 0 on 8/8 frames, which is DEC-088
tier 1 and, because `compute_key` hashes input **bytes** (`src/build/cache.rs:245-252`), caches per
frame.

**Revisit only if all four of the assessment's §12 questions answer affirmatively** — the blocking
one is the AVC patent position for an independently-implemented decoder, which is a lawyer question,
not a probe. Pure-Rust H.264 decoders now exist (`rusty_h264-decoder` 0.10.0, BSD-2-Clause,
`forbid(unsafe_code)`, fuzzed) but are **seven weeks old** and short of full JVT conformance by their
own README.

### Two items handed to STAGE-046 (not new work — inputs to work already scheduled)

- **`mp4-atom` 0.15.0 is a cleared candidate for animated AVIF output.** MIT OR Apache-2.0 (both
  LICENSE files present), pure Rust with no `-sys` crate, 243,922 downloads, updated 2026-07-31.
  Ships the `av01` sample entry and `av1C` config box
  (`src/moov/trak/mdia/minf/stbl/stsd/av01.rs:21,87`), the full sample table, and a committed test
  decoding a real libavif animated AVIF (`avis` brand). crustyimg already has the AV1 encoder
  (rav1e) and decoder (re_rav1d) in-tree. **Belongs to the later animated-*output* spec, not
  SPEC-119** (which is scoped to stopping the data loss). Measured price of the muxing *driver* on
  top of a box library: **~1,000 lines** (`mp4` 0.14's `writer.rs` + `track.rs`) — compare against
  the in-house RIFF/ANMF route before choosing. Adoption needs a DEC per
  `no-new-top-level-deps-without-decision`.
- **SPEC-119's AC-6 was independently confirmed.** An out-of-crate probe reproduced the exact
  failure signature on AV1: 8 frames encoded, 3 dropped, output has 5 — frame-count oracle catches
  it, SSIMULACRA2 scores **100.0**. This adds no requirement (AC-6 already says the assertion must
  be structural); it is a second derivation agreeing from a different direction, plus a working
  template for the half AC-6 leaves open — that the decoder doing the counting should be one you
  did not write ([[verify-wasm-output-with-an-independent-decoder]]).

### One correction to a shipped-API note

`Image::from_parts` takes `image::ImageFormat`, and `crustyimg::image::ImageFormat` is **private**
(a `use`, not a `pub use` — `src/image/mod.rs:25`). So an out-of-crate consumer is *forced* to
declare `image` itself, which is exactly the naive dependency the lab plan's F2 measures as adding
six reachable decoders. This raises the priority of the recorded `pub use ::image;` /
`pub use ::toml;` fix from ergonomics to correctness-of-guidance. Re-exports, not visibility
widenings — the measured zero-widening result stands.

## ⚠ Live defect — animated input is silently flattened, and `lint` recommends the command that does it (2026-08-15)

> **Homed on STAGE-046** (output fidelity on shipped verbs, 2026-08-15). The evidence stays
> here; the schedulable work lives there. Sequenced ahead of STAGE-041 by maintainer decision.


**The most severe of the current defect set: silent data loss on a path the tool itself
recommends, reported as a success with a perfect quality score.** Same class as D-1, worse
consequence.

### Driven end to end

Fixture: a valid 4-frame looping GIF, 64×64, written with `image`'s own `GifEncoder`
(`encode_frames`) — the same encoder `src/lint/rules.rs:363-369` uses in its test.

```
$ crustyimg lint anim.gif
anim.gif
  warn format/animated-gif: animated GIF (a modern format encodes far smaller)
    fix: crustyimg convert --format webp anim.gif        <-- the tool's own advice

$ crustyimg convert anim.gif --format webp -o fixed.webp
$ crustyimg optimize anim.gif ; crustyimg web anim.gif
anim.gif: gif → webp · 423 → 118 B (72% smaller) · ssim 100.0
```

Output: **118 B, zero `ANMF` chunks, no `VP8X`** — a *static* WebP. **3 of 4 frames discarded.**
Exit 0, no warning, on `convert`, `optimize` **and `web`**.

### Why this is worse than a wasted channel

1. **The linter actively recommends the destructive command.** A user following the tool's own fix
   loses their animation.
2. **The loss is reported as a win.** "72% smaller" is true only because three-quarters of the
   content was thrown away.
3. **`ssim 100.0` certifies it.** The score compares decoded-source to output — and both are frame
   1 — so the perceptual oracle **structurally cannot see this failure**. It is not that the check
   is weak; it is measuring a quantity that is preserved by the bug
   ([[a-self-referential-control-cannot-detect-a-broken-pipeline]]). Any future test that asserts
   "score stayed high" will stay green through this defect forever.

### Root cause

**Frame decoding exists in exactly one place, and only to count:** `gif_is_animated`
(`src/lint/rules.rs:303-306`) constructs a `GifDecoder` and takes 2 frames purely to test `>= 2`.
The pixel pipeline never sees frame 2 — `Image::from_bytes` → `decode_with_format` →
`ImageReader` → **one** `DynamicImage`. So the linter *knows* the file is animated and the encoder
path does not.

### Two separable pieces of work — do not conflate them

- **(a) Stop the data loss — small, urgent, and independent of any new capability.** Detect
  multi-frame input on the pixel path and refuse (a typed error → exit 4, the `CodecNotBuilt`
  precedent) or warn loudly, instead of silently flattening. **Until animated output exists, the
  lint rule's `fix:` string is also wrong and must stop recommending a destructive command.**
- **(b) Animated GIF → animated WebP/AVIF** — triage §11. This is the real capability, and §11's
  framing should be updated: it is **not an enhancement, it is the repair** that lets the linter's
  advice become true.

  ⚠ **Correction to §11's dependency verdict (2026-08-15).** The triage verified `webp-animation`
  as *"✅ v0.10.0, MIT OR Apache-2.0"* — the licence is right, but it **wraps `libwebp-sys2`, a C
  dependency**, which the triage did not flag. That does not clear `pure-rust-codecs-default`;
  it would have to sit behind an off-by-default feature, exactly like the existing `webp-lossy`
  (DEC-022).

  **There is a pure-Rust route that avoids it entirely**, and it fits this repo's in-housing
  precedent:
  - **frame decode** — `image`'s `GifDecoder` + `AnimationDecoder` (already used, to count, at
    `src/lint/rules.rs:303-306`); animated WebP decode via `image-webp` 0.2.4 `extended.rs`
  - **per-frame transform** — the existing `Pipeline`, run once per frame (ops are pure, so this
    needs **no core change**)
  - **frame encode** — `image-webp` 0.2.4, already in the tree, pure Rust, lossless
  - **mux** — assemble `VP8X` / `ANIM` / `ANMF` RIFF chunks **in-house**. `image-webp`'s *encoder*
    has no animation support, so this is the only new code. RIFF is length-prefixed chunks;
    *estimated* 150–250 lines against the `src/metadata/tiff.rs` (718-line, with IFD offset
    patching) precedent — **label this an estimate and measure it at design.**

  **Lossless-only is not a compromise here:** a GIF source is palettized (≤256 colours by
  definition), which is exactly where lossless WebP wins big. The C dependency buys lossy frames,
  which GIF sources do not need.

  ⚠ **"WebP/AVIF" is not one item.** The 150–250 line estimate covers **animated WebP only** —
  RIFF is length-prefixed chunks. **Animated AVIF is a HEIF image sequence, materially harder, and
  has not been priced.** `rav1e` 0.8.1 is already in the tree (via `ravif`) and encodes multiple
  frames, so the *encoder* is not the gap — the container is. Price it separately; do not let the
  WebP number stand in for both.

  **The lint rule's own claim is unverified.** `format/animated-gif` tells the user "a modern
  format encodes far smaller" — but crustyimg cannot produce the animated format it recommends, so
  **this repo has never measured that win**. Shipping the capability is the first opportunity to;
  the spec should record the measured saving rather than inherit the claim.

### Scope check before either is specced

Animated GIF is not the only multi-frame input the tree can decode. **Mechanically sweep** every
format the `image` feature set enables for multi-frame support (animated WebP decode is in
`image`'s `webp`; APNG in `png`) and state the finding as a claim with its grep
([[mechanical-sweeps-need-a-mechanical-check]]). The `format/animated-gif` rule name may itself be
too narrow.

### Placement — this is workhorse, not `crustyimg-lab`

Under DEC-091: the params (quality, loop count, frame rate) generalize across a batch (**Fence A →
workhorse**), and the output is a delivery artifact a build consumes (**Fence B → workhorse**). It
also passes `docs/territory.md`'s test — animated GIF → animated WebP is one of the largest
byte wins in web asset delivery, which is exactly "a better artifact automatically".

**And more simply: a defect in crustyimg cannot be fixed in a different binary.** crustyimg already
accepts animated GIFs, already detects them, and already destroys them. What *would* be lab is the
per-image judgement around animation — choosing a cover frame, dropping frames by eye, previewing
a loop — none of which is the batch conversion.

## Open — RAW Tier-2 is no longer "multi-month"; three repo claims are now stale (2026-08-15)

**Measured, not speculative.** A design session priced Tier-2 RAW development against real
implementations and against a real Leica DNG. Three places in this repo now assert things the
measurements contradict. None is urgent; all will drift if nobody records them.

**1. `DEC-055` says Tier-2 needs "LGPL `rawler` … or a multi-month from-scratch effort."**
Both halves are now wrong for a DNG-first scope. `demosaic` 0.3.0 is MIT/Apache with **zero
dependencies** and `no_std`; its Malvar-He-Cutler implementation is **211 lines**, cited to the
ICASSP 2004 paper. A complete Sony pipeline (`rawkit` 0.1.0, MIT/Apache) is **2,205 lines of
src**, of which ~1,030 is TIFF machinery. A monochrome DNG needs no demosaic at all —
`PhotometricInterpretation: Linear Raw`, `SamplesPerPixel: 1` — so that path is ~550–700 lines.
DEC-055's Alternatives §B should be amended with the measured numbers, not deleted: the
conclusion was right for its time, the cost estimate was not.

**2. `guidance/license-watchlist.yaml` → `raw-full-demosaic` says "UNSURVEYED — no permissive
pure-Rust demosaic crate has been probed. This is the gap to close before the capability is
costed at all."** It is surveyed. `demosaic` 0.3.0 exists (MIT/Apache, zero deps, Bayer
bilinear/MHC/PPG/VNG/AHD + X-Trans Markesteijn/DHT + Quad-Bayer). ⚠ **But do not adopt it**:
`markesteijn_impl.rs:11` self-describes as *"Ported from LibRaw's `xtrans_interpolate(1)`"* —
LibRaw is LGPL-2.1/CDDL — while the crate ships `MIT OR Apache-2.0`. **`cargo deny check
licenses` reads the declared licence and passes it green.** That is a silent-green defect
against `no-agpl-default-deps`, and it is the reason the sibling project writes its demosaic
clean-room from the paper and keeps the crate as a dev-time oracle only.

**3. `docs/roadmap.md` places "RAW Tier-2" at 2.0+** under "Opt-in intelligence / new
frontends". The work is now framed and starting, so that row understates its status.

**What changed externally.** A separate permissively-licensed Rust library — **`irradiance`** —
is being built to do RAW *development* (sensor data in, pixels + metadata out; no I/O, no CLI, no
`image` dependency). crustyimg will consume it behind an off-by-default **`raw-develop`** cargo
feature; `crustyimg-lab` inherits it free through the shared `Operation` core. This is a
**dependency**, not a delegate — it participates in the lockfile and the build cache key, so
DEC-088's tier-3 objection does not apply. Its `develop_version` process-version field is what
keeps `build --frozen` meaningful while the library's algorithms are still free to improve.

**It also supplies the mechanism for a defect this file already records.**
`docs/backlog.md`'s "RAW loses 100% of EXIF … and RAW orientation is never read" notes that the
measured DNG's embedded previews carry **no APP1 at all**, so threading the preview's EXIF
forward cannot work — the container's IFD0 is the only source. A DNG container parser is exactly
what reads it.

**Not a gate on anything.** Recorded so the three stale claims get amended when RAW work
actually lands here (that is `irradiance`'s STAGE-004), rather than being discovered by someone
trusting DEC-055's cost estimate.
