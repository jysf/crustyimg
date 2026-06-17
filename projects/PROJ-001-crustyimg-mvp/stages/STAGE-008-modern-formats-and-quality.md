---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-008                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-16
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Turns crustyimg from "good at the routine transforms" into something
    *different* — it advances the project thesis ("routine image prep, faster
    and more repeatable than a GUI or ImageMagick incantations") into the
    frontier paradigm: tell the tool the OUTCOME you want — a visual quality or
    a file-size budget, plus a modern output format — and let it find the
    encoder settings. "Set the look, not the quality number."
  delivers:
    - "`shrink --target visually-lossless` / `--ssim <N>` — perceptual auto-quality: binary-search the JPEG quality against an SSIMULACRA2 score (the differentiator flagship)"
    - "`--max-size <KB>` byte-budget on `shrink`/`convert` — hit a file size, not a quality number"
    - "AVIF output behind an off-by-default `avif` cargo feature (`ravif`), exit 4 without it (DEC-004)"
    - "WebP output — lossless via the pure-Rust `image`/`image-webp` backend (default-able); lossy WebP only behind a feature-gated libwebp, or deferred with a documented reason"
  explicitly_does_not:
    - Add an AGPL/GPL default dependency (DEC-018 / `no-agpl-default-deps`) — rules out gifski / jpegli-rs / zenwebp / imagequant as defaults
    - Build animation / GIF, responsive `<picture>`/srcset sets, or blurhash (a later stage; soft-depends on the WebP/AVIF landed here)
    - Build the watermark / EXIF-metadata-edit container lane (the original STAGE-004 — re-sequenced behind this differentiator wave)
    - Publish cross-tool benchmarks (the SSIMULACRA2 metric added here is the FOUNDATION for that later suite, but the benchmark publication is its own work)
---

# STAGE-008: modern formats and quality

> **Sequencing note (2026-06-16).** The PROJ-001 roadmap was re-prioritized this
> day. This stage was created with the next free numeric id (STAGE-008) but runs
> in **execution position right after STAGE-003** — ahead of the lower-numbered
> `proposed` stages (STAGE-004 compose-and-metadata, STAGE-005 batch-and-recipes,
> STAGE-006 hardening, STAGE-007 release), which stay `proposed` and slide later.
> **Numeric id ≠ execution order** here; drive order by stage `status`/`priority`,
> not by number. Rationale and the full menu are in
> `docs/sessions/2026-06-16-roadmap-and-stage-004-decision-handoff.md` (Option B,
> user-chosen) and `docs/sessions/2026-06-16-stage-004-formats-quality-build-handoff.md`.

## What This Stage Is

The stage that makes crustyimg's compression *outcome-driven* and gives it
modern output formats. Two threads, one theme — "tell me the result, not the
knob":

1. **Perceptual quality.** The flagship `shrink --target visually-lossless`
   (and `--ssim <N>`) binary-searches the JPEG encoder quality, decoding each
   candidate and scoring it against the original with **SSIMULACRA2**, stopping
   at the lowest quality whose score clears the target — so a user asks for a
   *look*, not a magic number. Its sibling `--max-size <KB>` does the same for a
   file-size budget. The SSIMULACRA2 metric this introduces is also the
   foundation for the future benchmark suite and a `diff`/visual-regression
   command.
2. **Modern formats.** AVIF output (feature-gated `ravif`, pure-Rust but heavy →
   off by default, exit 4 without the feature per DEC-004) and WebP output
   (pure-Rust *lossless* via the `image` backend, default-able; lossy WebP only
   behind a feature-gated libwebp or deferred with a documented reason). The
   auto-quality search loop is encoder-agnostic, so it generalizes onto AVIF/WebP
   once they land.

When this stage ships, "make this image web-good" stops meaning "guess a quality
number and a format" and starts meaning "give me visually-lossless AVIF under
200 KB" — a category claim, not a feature.

## Why Now

- It is the **differentiator** and the cleanest build: the flagship rides the
  already-shipped `shrink` + DEC-016 quality-encode path and needs only **one
  permissive pure-Rust dependency** (`ssimulacra2`, BSD-2-Clause) — no native
  deps, default-buildable.
- It is the **dependency root** for the next waves: animation (GIF), responsive
  `<picture>`/srcset sets, and the `optimize` one-button command all soft-depend
  on the modern formats and the perceptual metric landed here.
- The competitor/CLI layer is **stale** (squoosh-cli unmaintained, rimage has no
  quality-targeting), and the frontier component layer just made
  perceptual-target encoding viable in pure Rust — so there is a real, open gap
  and no race pressure to cut corners.
- The **license gate is now live** (DEC-018): the codec landscape is constrained
  to permissive crates, which is exactly why the flagship (permissive metric on
  the default JPEG path) leads and the heavier/feature-gated formats follow.

## Success Criteria

- `shrink photo.jpg --target visually-lossless -o out.jpg` produces a JPEG that
  SSIMULACRA2-scores at/above the visually-lossless threshold and is smaller than
  a max-quality encode — chosen automatically in a capped, sub-second search.
- A lower `--ssim` / lower `--target` yields a smaller output than a higher one,
  on the same input (monotone target → size).
- `shrink`/`convert --max-size 200KB` produces an output ≤ the budget (or the
  closest feasible best-effort, reported), with no manual quality guessing.
- `convert photo.jpg --format avif` exits 4 without `--features avif`, and with
  it produces a valid AVIF; a `--features avif` CI job is green.
- WebP output works for the lossless case on the default (pure-Rust) build; the
  lossy-WebP decision is recorded in a DEC.
- `cargo deny check licenses` stays green throughout (no AGPL/GPL default dep);
  every new top-level dep carries a DEC and a `just deny` run.
- 3-OS CI stays green; every command remains one short, ergonomic invocation.

## Scope

### In scope
- Perceptual auto-quality (`--target`/`--ssim`) on `shrink`, JPEG target for v1,
  via SSIMULACRA2 (`ssimulacra2` crate) + a capped binary search. **(SPEC-016)**
- `--max-size <KB>` byte budget on `shrink`/`convert`, reusing the search
  machinery. **(SPEC-017)**
- AVIF output behind an off-by-default `avif` feature (`ravif`), incl. the
  auto-quality loop, plus a `--features avif` CI job. **(SPEC-018)**
- WebP output: pure-Rust lossless (default-able); lossy WebP decision in a DEC.
  **(SPEC-019)**

### Explicitly out of scope
- Animation / GIF, responsive sets, `<picture>`/srcset, blurhash — a later stage
  (soft-depends on the WebP/AVIF this delivers).
- Watermark + the EXIF/metadata edit container lane (the original STAGE-004) —
  re-sequenced behind this wave.
- Parallel batch + recipes (STAGE-005); cross-tool benchmark *publication*
  (later; this stage lays the metric foundation).
- Any AGPL/GPL default dependency (DEC-018). Lossy WebP via AGPL `zenwebp` is
  blocked; gifski/jpegli-rs/imagequant are out as defaults.

## Spec Backlog

Ordered by recommended execution (flagship first); drive by status, not number.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-016 (shipped 2026-06-16, PR #18) — **perceptual auto-quality (FLAGSHIP):** `shrink --target visually-lossless` / `--ssim <N>` — binary-search the JPEG quality against an SSIMULACRA2 score; lowest quality clearing the target; capped iterations; default JPEG path; one permissive dep (`ssimulacra2`, DEC-019). New `src/quality/` module (metric + generic scorer-injected search reused by SPEC-017+); opt-in; unmet-target best-effort + warning
- [x] SPEC-017 (shipped 2026-06-16, PR #20) — **`--max-size <SIZE>` byte budget** on `shrink`/`convert`: binary-search the quality for the highest output ≤ the budget (the SPEC-016 search inverted, via a shared `search_threshold` core); **quality-only v1** — lossless/infeasible → best-effort + warning; no new dep, no new DEC (DEC-019 dual). Also made the search **format-agnostic** (`LossyFormat` trait + `encode_candidate_bytes`) so AVIF/WebP slot in via one encode arm
- [ ] (not yet written) SPEC-NNN — **`--max-size` dimension-reduction fallback** (the deferred half of SPEC-017): when quality alone can't hit the budget (a lossless output, or even min quality too big), progressively downscale dimensions until it fits; makes `--max-size` work for PNG and very-small budgets. Reuses the shipped `Resize` op + the byte-budget search
- [x] SPEC-018 (shipped 2026-06-17, PR #21) — **AVIF output (feature-gated `avif`):** `ravif` via `image/avif` behind an off-by-default `avif` feature (exit 4 without it, `CodecNotBuilt`/`ensure_codec_built`, DEC-004); `--features avif` CI job; **DEC-020** (pure-Rust/no-nasm + a scoped `libfuzzer-sys` deny exception). **Build-time correction:** only the `--max-size` byte budget drives AVIF for free; the perceptual `--target`/`--ssim` search needs an AVIF decoder (deferred) → split the `LossyFormat` seam, graceful fallback. Output-only v1 (decode + `--speed` deferred)
- [x] SPEC-019 (shipped 2026-06-17, PR #22) — **WebP lossless output + WebP decode (input), pure-Rust DEFAULT:** added `webp` to the image default features so `.webp` reads as INPUT everywhere and `convert --format webp` / `-o x.webp` write LOSSLESS WebP (smaller than PNG). All pure-Rust, no system deps, `just deny` green with NO new exception; **DEC-021**. No quality knob → `-q`/auto ignored like PNG (not a `LossyFormat`). The WebP foundation; lossy is SPEC-020. Build had zero deviations (design's empirical probe held: lossless encode via write_to + decode via ImageReader, no code).
- [ ] (not yet written) SPEC-020 — **Lossy WebP encode (feature-gated `webp-lossy`):** wire `libwebp` (a C system dep — the project's first) behind an off-by-default `webp-lossy` feature so `convert --format webp -q Q` and the auto-quality searches (`--target`/`--ssim`/`--max-size`) drive WebP (BOTH searches — the pure-Rust decoder already exists, so perceptual can score WebP round-trips). Needs its own DEC (DEC-022: the C-dep decision, CI C-toolchain, `single-image-library` tension) + dep verification. The smaller-than-JPEG story. Layers onto SPEC-019's WebP wiring

**Count:** 4 shipped / 0 active / 2 pending (dimension-reduction fallback, SPEC-020 lossy WebP)

> **STAGE-008 follow-ups identified during SPEC-018 build/verify (write as specs when picked up):**
> - **AVIF decode** (`dav1d`/`avif-native`, a C dep) behind its own feature — unblocks `.avif` INPUT *and* perceptual AVIF (`--target`/`--ssim`).
> - **AVIF `--speed` knob** (thread speed through the sink encode + the search probe so probed and written bytes agree).
> - **Up-front codec check for `shrink`/`resize`/`thumbnail`/`auto-orient`** — multi-input forced to an unbuilt AVIF currently exits 6 (partial-batch) rather than a single exit 4 like `convert` (single-input is already exit 4). Low-severity consistency gap (verify finding, PR #21).

> The flagship (SPEC-016) is the differentiator AND the most self-contained
> (default JPEG, one permissive dep), so it leads. (SPEC-017) reuses its search;
> (SPEC-018)/(SPEC-019) add formats behind features. Adjust if a modern format
> should land first, but SPEC-016 is the "why I'd switch" demo with the cleanest
> build.

## Design Notes

- **License discipline is load-bearing here** (DEC-018 / `no-agpl-default-deps`):
  every new top-level dep needs its own DEC *and* a `just deny` run after adding,
  to confirm the gate stays green (it catches a copyleft transitive dep
  immediately). The full gate set is now `cargo build` · `cargo test` ·
  `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check` ·
  `cargo deny check licenses`.
- **The search loop is encoder-agnostic.** SPEC-016 builds it for JPEG; SPEC-017
  reuses it for the size budget; SPEC-018/019 generalize it onto AVIF/WebP. Keep
  the metric + search in their own module (`src/quality/`) so format wiring stays
  separate from the search policy.
- **Decode-once still holds (DEC-002).** The original image is decoded once; the
  quality search re-encodes/decodes *candidates in memory* (capped iteration
  count, no per-candidate disk round-trips).
- **Feature-gating policy (DEC-004):** heavy/native codecs (AVIF via
  rav1e; lossy WebP via libwebp) are off-by-default cargo features; the default
  binary stays pure-Rust, zero system deps, and exits 4 for an unbuilt codec
  (current `convert`/`shrink` behavior).
- Weighty decisions get their own `DEC-*`: **DEC-019** (ssimulacra2 + the
  metric/threshold/search-loop policy, SPEC-016), **DEC-020** (adopt `ravif`;
  revisit DEC-004 AVIF-gating, SPEC-018), and a WebP/lossy-WebP DEC (SPEC-019).

## Dependencies

### Depends on
- STAGE-003 — `shrink` + the DEC-016 quality-encode path (`-q` →
  `JpegEncoder::new_with_quality`, threaded `run_pixel_op → Sink::write →
  encode_to_bytes`), the `run_pixel_op` fan-out, and DEC-015 format precedence.
- STAGE-001 — the `Image`/decode-once pipeline + the encoding Sink.
- DEC-018 — the live license gate (`cargo deny check licenses`) that constrains
  which codec/metric crates are permissible.
- External: `ssimulacra2` (BSD-2, SPEC-016); later `ravif` (BSD-3, feature-gated,
  SPEC-018) and the `image`/`image-webp` WebP backend (SPEC-019).

### Enables
- The animation / responsive-set / `optimize` / `diff` waves (modern formats +
  the perceptual metric are their prerequisites).
- The benchmark suite — SSIMULACRA2 is the equal-quality basis every size/speed
  claim must be held against (roadmap handoff §Benchmarking).

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
