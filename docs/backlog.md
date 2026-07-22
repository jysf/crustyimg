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
| sobel / edges | Edge-detection effect | S–M | `Operation` trait (imageproc convolution) |

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
