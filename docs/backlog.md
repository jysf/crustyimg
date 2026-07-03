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

## PROJ-002 — next wave after MVP

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
| **WebP output** | Biggest real web-size win; the headline fast-follow | M | `convert`/`shrink` encode path + codec policy (DEC-004); `image` already supports WebP |

---

## PROJ-003 — later

Higher complexity, a native/feature-gated dep, a new metric, or a broader
suite. Still additive on the same architecture.

| Item | Value | Complexity | Enabling architecture in place |
|---|---|---|---|
| **AVIF output (feature-gated)** | Best modern compression; slow pure-Rust encode behind a cargo feature | L | Codec policy already gates native/slow codecs behind off-by-default features (DEC-004) |
| `open` in external app | Hand off to Preview / Safari / Chrome / OS default | S | Sink abstraction (a non-rendering "open" sink) |
| `compare` (SSIM / PSNR) | "Did optimization hurt quality?" — quality measurement | M | Read-only inspect path (like `info`); two-image read |
| target-size / target-quality auto-tuning | "Smallest file ≥ SSIM threshold" — a real differentiator | L | Builds on `compare` metric + `shrink` encode loop |
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
`convert`/`shrink`/`thumbnail` all work). Bound it with the existing decode limits
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
