---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-019
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-008
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet 4.6, fresh session
  created_at: 2026-06-17

references:
  decisions: [DEC-021, DEC-004, DEC-018, DEC-016, DEC-015, DEC-002, DEC-007, DEC-012]
  constraints:
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
    - single-image-library
    - ergonomic-defaults
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - test-before-implementation
    - untrusted-input-hardening
  related_specs: [SPEC-020, SPEC-018, SPEC-017, SPEC-014, SPEC-005, SPEC-002]

value_link: "Adds WebP as a default, pure-Rust format — `.webp` becomes a readable INPUT everywhere (view/info/resize/convert), and `convert --format webp` / `-o x.webp` produce LOSSLESS WebP (smaller than PNG). All pure-Rust, zero system deps, `just deny` green with no new exception (DEC-021). Lossy WebP (the smaller-than-JPEG story; needs libwebp, a C dep) is SPEC-020, which layers onto this WebP wiring."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Design authored by the ORCHESTRATOR (Opus) directly. Verified empirically (the 'pin the dep in design' discipline, extended to the decode path): `image` 0.25.10 `webp` feature → `image-webp` 0.2.4 (MIT/Apache) builds PURE-RUST with no nasm/system deps; it DECODES lossy+lossless but ENCODES lossless only (image docs: 'for lossy, use libwebp'); `cargo deny check licenses` is GREEN with NO new exception when `webp` is enabled. `DynamicImage::write_to(_, WebP)` routes to `WebPEncoder::new_lossless`, so lossless encode needs no special sink arm. Emitted DEC-021 (WebP lossless+decode as a pure-Rust default; lossy deferred to SPEC-020/DEC-022)."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 130000     # ORDER-OF-MAGNITUDE estimate — build ran in the orchestrator main loop (background subagents can't get Bash here); small spec (one feature line + one match arm + tests)
      estimated_usd: 1.15      # ~130k @ Opus 4.8 list ($5/$25 per MTok, ~80/20 in/out) — order of magnitude
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Built in the main loop (background subagents can't get Bash here), tokens are a labeled order-of-magnitude estimate. Added `webp` to the default image features + the format_from_extension arm; lossless encode + decode came for free (write_to / ImageReader). Tests: unit + 4 integration (lossless round-trip, .webp input, shrink→webp, -q ignored). Dropped the webp branch from convert_unbuilt_codec_exits_4. Default + avif builds green; just deny green with NO new exception."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-019: WebP lossless output + WebP decode (input), pure-Rust default

## Context

STAGE-008's second modern format. WebP is the **inverse of AVIF (SPEC-018)**: AVIF
has a pure-Rust *encoder* but no decoder (output-only); WebP has a pure-Rust
*decoder* (lossy + lossless) but only a *lossless* encoder. So this spec ships, all
in pure Rust and **on by default** (DEC-021):

- **WebP INPUT (decode):** `.webp` reads everywhere — `view`, `info`, `resize`,
  `thumbnail`, `convert in.webp --format png`, etc. (the decode path is feature-driven
  via `image`/`image-webp`; once the feature is on, `Image::from_bytes` handles it).
- **WebP LOSSLESS OUTPUT (encode):** `convert --format webp` / `-o x.webp` produce a
  lossless WebP (VP8L), typically smaller than PNG.

**Verified at design time (empirical, DEC-021):** `image` 0.25.10's `webp` feature →
`image-webp` 0.2.4 (`MIT OR Apache-2.0`) builds **pure-Rust, no nasm/system libs**;
`cargo deny check licenses` stays **green with NO new exception**; `image-webp`
**decodes lossy+lossless** but **encodes lossless only** (the `image` docs say to use
`libwebp` for lossy). `DynamicImage::write_to(_, ImageFormat::WebP)` already routes
to `WebPEncoder::new_lossless`, so lossless encode needs **no special sink arm** —
only extension recognition.

- **Parent stage:** `STAGE-008` (modern formats & quality), the WebP foundation
  spec (the format after AVIF).
- **Why DEFAULT, not feature-gated (DEC-004):** WebP decode + lossless encode are
  pure-Rust, small, and permissive — the DEC-004 "ship by default" category. This
  **changes the default build**: `convert --format webp` flips from exit 4 to a
  successful lossless encode, and `.webp` input now decodes.
- **No quality knob (DEC-016):** lossless WebP has no 0–100 quality parameter, so
  `-q` is **ignored** for WebP output (like PNG), and the auto-quality searches
  (`--target`/`--ssim`, `--max-size`) do **not** drive it. WebP is **not** added to
  `LossyFormat` — `supports_lossy_quality(WebP)` / `supports_perceptual_quality(WebP)`
  stay `false`. (They turn on with the lossy encoder in **SPEC-020**.)
- **Lossy WebP is OUT OF SCOPE (SPEC-020):** it needs `libwebp` (a C system dep) — a
  different risk profile (first C dep, new top-level crate, CI C-toolchain). SPEC-020
  layers it behind an off-by-default `webp-lossy` feature.

## Goal

Add `webp` to the default `image` feature set so `.webp` decodes as input and
lossless WebP is a real output (`convert --format webp`, `shrink`/`convert -o
x.webp`); recognize the `webp` extension in the sink; keep `-q`/auto-quality ignored
for (lossless) WebP output (DEC-016, like PNG); keep `just deny` green with **no new
exception**. No new top-level dependency; no `Operation` change; lossy WebP deferred.

## Inputs

- **Files to read:**
  - `Cargo.toml` — the `image` dep line (add `"webp"` to its `features` list).
  - `src/sink/mod.rs` — `format_from_extension` (recognize `webp`),
    `extension_for_format` (already returns `"webp"` — no change), `encode_to_bytes`
    (the default `write_to` path already handles lossless WebP — **no new arm**).
  - `src/image/mod.rs` — `decode_with_format` (no change; webp decode is automatic
    once the feature is on — the tests confirm `.webp` input works).
  - `src/cli/mod.rs` — `resolve_effective_quality` (no change; WebP is not a
    `LossyFormat`, so `-q`/auto are ignored with the existing lossless handling),
    `format_label` (fallback already yields `"webp"`).
  - `tests/cli.rs` — the existing `convert_unbuilt_codec_exits_4` (drop the webp
    branch; webp is now a built default codec — keep the avif branch).
  - `tests/common/mod.rs` — add a `webp_lossless(w, h)` fixture (encode via
    `ImageFormat::WebP`) for the decode-input test.
  - `docs/api-contract.md` — the `convert` entry + the format table.
  - `decisions/DEC-021-*.md` (the governing decision).
- **External APIs:** the `image` crate's `webp` feature → `image-webp` (MIT/Apache,
  pure-Rust). Encode is reached through `DynamicImage::write_to(w, ImageFormat::WebP)`
  (→ `WebPEncoder::new_lossless`); decode through the existing `ImageReader`
  path. docs: https://docs.rs/image/0.25.10/image/codecs/webp/struct.WebPEncoder.html
- **Related code paths:** `Cargo.toml` + `src/sink` + tests + docs. Do NOT modify
  `src/operation`, `src/pipeline`, `src/recipe`, `src/quality` (WebP has no quality
  knob in this spec).

## Outputs

- **Files modified:**
  - **`Cargo.toml`** — add `"webp"` to the `image` dep's `features` list (it becomes
    part of the DEFAULT build; no new top-level dep — it is the `image` crate's own
    feature). Run `just deny` (must stay green with NO new exception).
  - **`src/sink/mod.rs`** — `format_from_extension`: add `"webp" => Ok(ImageFormat::WebP)`
    so `--format webp` / `.webp` resolve. `encode_to_bytes` needs NO WebP arm: the
    existing `img.pixels().write_to(&mut cursor, format)` path encodes lossless WebP
    (verified). (`extension_for_format` already returns `"webp"`.)
  - **`.github/workflows/ci.yml`** — NO new job: WebP is in the default build, so the
    existing matrix already builds/tests it.
  - **`docs/api-contract.md`** — document WebP as a supported format: readable INPUT
    (lossy + lossless) and lossless OUTPUT (`convert --format webp` / `-o x.webp`);
    `-q`/`--max-size`/`--target` ignored for WebP output (lossless, like PNG, DEC-016);
    note lossy WebP is `--features webp-lossy` (SPEC-020).
- **New decisions:** `DEC-021` (emitted in this design cycle).
- **No new top-level dependency. No new `Operation`. No `src/quality` change.**

## Acceptance Criteria

Each maps to a test. All run in the DEFAULT build (WebP is default).

- [ ] `convert <png> --format webp -o out.webp` → exit 0; output is a valid WebP
  (`image::guess_format == WebP`) and decodes back to the same dimensions and (since
  it is lossless) the same pixels. → `convert_to_webp_produces_lossless_webp`
- [ ] `.webp` is a readable INPUT: `convert <webp-fixture> --format png -o out.png`
  → exit 0; output is PNG and decodes to the source dimensions. → `webp_input_decodes`
- [ ] `shrink <jpg> -o out.webp` → exit 0; output is WebP. → `shrink_to_webp_output`
- [ ] `-q` is ignored for WebP output (no quality knob, DEC-016): `convert <png>
  --format webp -q 50 -o out.webp` → exit 0; output is WebP (still lossless). →
  `webp_quality_is_ignored`
- [ ] `format_from_extension(Path::new("x.webp")) == Ok(ImageFormat::WebP)`. →
  `format_from_extension_recognizes_webp`
- [ ] `just deny` is green with NO new exception; the default suite + all 5 gates
  stay green; `convert --format webp` no longer exits 4. → `convert_unbuilt_codec_exits_4`
  (webp branch dropped; avif branch kept) + CI

## Failing Tests

Written during **design**. WebP is a default format, so all of these run under the
normal `cargo test` (no feature gate). Verifying WebP output uses
`image::guess_format` AND a real decode round-trip (the decoder is built).

- **`src/sink/mod.rs`** (UNIT):
  - `format_from_extension_recognizes_webp` — `format_from_extension(Path::new("x.webp"))
    == Ok(ImageFormat::WebP)`.
- **`tests/common/mod.rs`** (FIXTURE, not a test): add
  `pub fn webp_lossless(w: u32, h: u32) -> Vec<u8>` — encode a small structured image
  to WebP bytes via `DynamicImage::write_to(_, ImageFormat::WebP)` (lossless).
- **`tests/cli.rs`** (INTEGRATION):
  - `convert_to_webp_produces_lossless_webp` — `convert <png> --format webp -o
    out.webp` → exit 0; `image::guess_format(&bytes) == WebP`; re-decode the output
    and assert dimensions match the source AND the decoded pixels equal the source
    pixels (lossless round-trip; use a small solid or low-color image so equality is
    exact).
  - `webp_input_decodes` — write `common::webp_lossless(..)` to a `.webp` file;
    `convert <that.webp> --format png -o out.png` → exit 0; output is PNG and decodes
    to the source dimensions. (Proves `.webp` INPUT works.)
  - `shrink_to_webp_output` — `shrink <jpg> -o out.webp` → exit 0; `guess_format ==
    WebP`.
  - `webp_quality_is_ignored` — `convert <png> --format webp -q 50 -o out.webp` →
    exit 0; `guess_format == WebP` (the `-q` is ignored for lossless WebP, DEC-016;
    no error, no warning required to be fatal).
  - `convert_unbuilt_codec_exits_4` (EDIT existing) — REMOVE the webp branch (webp is
    now a built default codec and would succeed); KEEP the avif branch (still
    feature-gated, exit 4 without `--features avif`). Update the doc comment.

## Implementation Context

### Decisions that apply
- **`DEC-021`** (emitted here) — WebP lossless output + decode as a pure-Rust DEFAULT
  format (add `webp` to the image features); no quality knob (so not a `LossyFormat`);
  lossy WebP deferred to SPEC-020/`webp-lossy`.
- `DEC-004` — pure-Rust codecs ship by default; WebP (decode + lossless encode) is
  exactly that, so it is default (not gated). The unbuilt-codec exit-4 path no longer
  applies to webp.
- `DEC-018` — the license gate; `just deny` must stay green WITH NO new exception
  (image-webp is permissive). Run it after adding the feature.
- `DEC-016` — `-q` applies to lossy formats (JPEG; AVIF with its feature); it is
  **ignored** for lossless formats — WebP joins PNG/GIF/BMP/TIFF/ICO here.
- `DEC-015` — format precedence (`--format` > `-o` ext > preserve); `--format webp`
  / `.webp` now resolve to `ImageFormat::WebP`.
- `DEC-002` — decode-once; reading a `.webp` decodes once via the normal pipeline.

### Constraints that apply
- `pure-rust-codecs-default` — WebP decode + lossless encode are pure-Rust, no nasm,
  no system libs; this is the constraint's exemplar (and it stays the default).
- `no-agpl-default-deps` — image-webp is MIT/Apache; `just deny` green, NO new
  exception.
- `no-new-top-level-deps-without-decision` — WebP arrives via the existing `image`
  dep's feature; DEC-021 covers it; no new top-level crate.
- `single-image-library` — WebP encode/decode is the `image` crate's own backend; no
  second pixel library.
- `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`,
  `untrusted-input-hardening` (decoding an untrusted `.webp` goes through the same
  hardened decode path as every other input; `image-webp` bounds its work).

### Prior related work
- `SPEC-018` (shipped, PR #21) — AVIF output (the opposite capability profile:
  encoder but no decoder). WebP reuses the same sink format-recognition seam.
- `SPEC-014` (shipped) — `convert` + exit-4 for an unbuilt codec; WebP no longer
  hits it (it is built by default now).
- `SPEC-005` (shipped) — the `Sink`/`encode_to_bytes` path WebP plugs into via
  `write_to`.
- `SPEC-002` (shipped) — the `Image` decode-once load path that gains `.webp` input.

### Out of scope (create a new spec rather than expand)
- **Lossy WebP encode** (`SPEC-020`) — needs `libwebp` (C); off-by-default
  `webp-lossy` feature; makes `-q`/`--target`/`--ssim`/`--max-size` drive WebP (both
  searches, since the decoder already exists). The headline smaller-than-JPEG story.
- Animated WebP; WebP-specific metadata; lossless WebP tuning knobs (effort/method).

## Notes for the Implementer

- **WebP arrives via the `image` feature, not a new crate.** `Cargo.toml`: add
  `"webp"` to the existing `image = { ..., features = [...] }` list. Confirm with
  `cargo build` (pure-Rust — no nasm) and `just deny` (green, NO new exception).
- **No sink encode arm needed.** `encode_to_bytes`'s existing
  `img.pixels().write_to(&mut cursor, format)` path encodes lossless WebP when
  `format == ImageFormat::WebP` (verified in design). You ONLY add the `webp` arm to
  `format_from_extension` so the format resolves. Do NOT add a WebP branch like the
  JPEG/AVIF quality arms — WebP has no quality knob here.
- **`-q` is ignored for WebP (DEC-016).** Because WebP is not added to `LossyFormat`,
  `resolve_effective_quality` already ignores `-q`/auto for it (the lossless path) and
  `--max-size`/`--target` emit the existing "needs a lossy-quality format" / encoder
  default behavior. No `src/cli` or `src/quality` change is required — confirm via the
  tests, don't add code.
- **`.webp` input is automatic.** `decode_with_format` uses `ImageReader::
  with_guessed_format().decode()`, which gains WebP once the feature is on. The
  `webp_input_decodes` test is the proof; no `src/image` change.
- **The existing `convert_unbuilt_codec_exits_4` test will FAIL** until you drop its
  webp branch — webp now succeeds (lossless). Keep the avif branch (still gated).
- **Lossless round-trip equality:** for `convert_to_webp_produces_lossless_webp`, use
  a SMALL image with few colors (e.g. a solid or 2-color RgbImage) so the decoded
  pixels exactly equal the source — that is what proves losslessness. Avoid a noisy
  gradient (still lossless, but keep the fixture cheap).
- **Commit incrementally** (feature + deny green → format_from_extension + unit test →
  integration tests + drop the webp exit-4 branch → docs).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-019-webp-lossless-output-and-decode`
- **PR (if applicable):** *(opened during build — see timeline)*
- **All acceptance criteria met?** yes — `format_from_extension_recognizes_webp`,
  `convert_to_webp_produces_lossless_webp` (guess_format==WebP + bit-exact
  round-trip), `webp_input_decodes`, `shrink_to_webp_output`, `webp_quality_is_ignored`
  all pass; `convert_unbuilt_codec_exits_4` webp branch dropped (avif kept). Default
  suite + all 5 gates green; the `avif` feature build also green; `just deny` green
  with NO new exception.
- **New decisions emitted:**
  - `DEC-021` — WebP lossless + decode as a pure-Rust default [authored in design].
- **Deviations from spec:**
  - None. The spec's prediction held exactly: only `Cargo.toml` (one feature) +
    `format_from_extension` (one arm) needed changing; lossless encode (via
    `write_to`) and `.webp` decode (via `ImageReader`) required no code.
- **Follow-up work identified:**
  - **SPEC-020 (lossy WebP via `webp-lossy`/libwebp)** — already in the STAGE-008
    backlog; it makes `-q`/`--target`/`--ssim`/`--max-size` drive WebP (both searches,
    since the pure-Rust decoder already exists).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing. The design's empirical verification (write_to routes to the lossless
   encoder; decode is feature-driven) meant the build was almost purely additive — a
   feature flag, an extension arm, and tests.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. Splitting lossy (SPEC-020) out kept this spec free of the `single-image-library`
   / C-dep tensions, so the applicable constraints were exactly the listed ones.

3. **If you did this task again, what would you do differently?**
   — Nothing material. The design-time `write_to(_, WebP)` spike is what made this a
   one-line-of-real-code spec; doing that probe in design (not build) was the win.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
