# SPEC-074 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in a
  design-time twiggy size-attribution probe on the raw release cdylib: **no single whale** — mass
  clusters in the SVG text/font stack (usvg text + ttf_parser CFF/COLR + rustybuzz + unicode_bidi)
  and the raster-codec spread (zune_jpeg/png/tiff/image_webp/fdeflate); ssimulacra2 is NOT a top
  contributor. Method = feature-ablation brotli-diffing on the wasm-opt'd artifact + a size-tuned
  wasm profile. Capability-losing levers (drop SVG text, trim a codec) = explicit DEC-066 calls.
- [x] **build** (2026-07-12, branch `feat/spec-074-wasm-bundle-size`, PR #83) — ablation table built
  (16 real builds), **1,595,028 → 1,394,313 B brotli (−200,715, −12.6%): 1.52 → 1.33 MB**. DEC-066
  emitted. **Both of the design's two suggested "no-cost" levers turned out to COST, and were
  refused:** `opt-level="z"` makes the AVIF encoder **2.8× slower** (350→956 ms — rav1e is generic,
  so it monomorphizes into `ravif`; protecting it hands back 161 of the 165 KB), and `wasm-opt` was
  **silently failing validation** while wasm-pack shipped the unoptimized module at exit 0 — and
  measured, it *costs* 36 KB on the wire for 340 KB of raw it buys no speed with, so it is now OFF.
  Taken instead: fat LTO + codegen-units=1 + strip (−138 KB, free) and a wasm-only `image` trim of
  tiff/bmp/ico (−84 KB). **resvg `text` REFUSED at −287 KB** — dropping it doesn't degrade SVG text,
  it silently *deletes* it (built that artifact; `transform()` still returned Ok and the old test
  stayed green through the corruption). 2 new mutation-tested guardrails; wasm-test 12/12; native +
  lean + deny green; Cargo.lock untouched.
- [x] **verify** (2026-07-12, fresh adversarial session) — **CLEAN, ready to ship**, with **one
  correction to the record** (docs only, applied on this branch; no code touched, no re-measurement
  needed).

  **The numbers reproduce byte-exactly.** Rebuilt both endpoints from source on this machine:
  shipped `just wasm-build` = **1,394,313 B brotli** (confirmed twice, incl. a forced clean
  rebuild), and `origin/main`'s config rebuilt = **1,595,028 B** exactly. **−200,715 B (−12.6%)
  confirmed.** Two ablation rows re-measured independently and both landed on the table's number:
  `wasm-opt` ON in the shipped config = **1,430,801 B** exactly (+36,488 on the wire, −340 KB raw),
  and dropping resvg `text` = **−284,069 B** in the shipped config (DEC's −287,098 was measured
  against a different base — honest within 1%).

  **Both reversed "free" levers hold, and one is worse than the build said.** Timed the real
  artifact from Node on a 512×384 photo: shipped **AVIF encode 562 ms**; rebuilt with
  `opt-level = "z"`, same config otherwise, **1906 ms — 3.4× slower** (the build said 2.8×) for
  −219 KB. The SSIMULACRA2 search is genuinely live on wasm (`optimize`→jpeg 554 ms → 951 ms under
  `z`), so refusing to drop it is real too. **The shipped config is the fast one.** The shipped
  `.wasm` is valid and runs: **wasm-test 12/12 green**, and all demo conversions drive clean from
  Node (PNG→PNG/JPEG/WebP/AVIF, SVG→PNG, `optimize`); TIFF/BMP/ICO return a typed `Err` and the
  module survives all three (no trap).

  **The resvg-text guardrail has teeth — mutation-confirmed.** Dropping the `text` feature doesn't
  even compile (`svg.rs` uses `usvg::fontdb` unconditionally) — a *stronger* guard than the test,
  but one a dev would "fix" by cfg-ing the font code out, which is exactly when silence returns. Did
  that: `svg_text_renders_glyphs_in_wasm` goes **RED**, while the old dimensions-only
  `svg_rasterizes_in_wasm` stays **green straight through the corruption**. Precisely as claimed.

  **Native unaffected, driven not assumed.** `Cargo.lock` byte-identical to main; build /
  `--no-default-features` / test / clippy `-D warnings` / fmt / `just deny` all green; and the real
  native binary still converts `.tiff`/`.bmp`/`.ico` → PNG (16×12) and still renders SVG `<text>` as
  glyphs. The trim is wasm-only.

  **⚠ CORRECTED ON BRANCH — the build's headline "trap" does not reproduce.** DEC-066, the research
  doc, `Cargo.toml` and the spec all recorded that `wasm-opt` **"silently fails and wasm-pack ships
  the unoptimized module at exit 0"**, and inferred from it that SPEC-072/073's "post wasm-opt"
  numbers were never optimized. **Both halves are false.** On wasm-pack 0.15.0 / binaryen 130, all
  four invocation shapes (wasm-pack default, `wasm-opt = true`, a flag list without
  `--enable-simd`, the full list) fail **LOUD**: `Error: failed to execute 'wasm-opt': exited with
  exit status: 1`, recipe aborts **exit 1**. And under main's own config `wasm-opt` validates clean
  (0 validator errors) and **really runs** — it strips 1.6 MB of raw (8,015,811 → 6,414,690 B). So
  the 1.52 MB baseline was genuinely post-wasm-opt and the −12.6% delta stands. The durable hazard
  is narrower: on failure the **un-optimized module stays in `pkg/`**, so a caller that loses the
  exit code through a pipe (`just wasm-build | tail` — the likely origin of "exit 0") sees a
  plausible artifact and a size no optimizer touched. **Check the raw size moved; don't read an exit
  code through a pipe.** Fixed in all four places. *A wrong lesson is worse than no lesson — it was
  written as durable guidance for whoever re-enables wasm-opt, and it defamed a sound baseline.*

  **Footgun confirmed (not a SPEC-074 defect, but the ship must file it).** The size profile lives
  in the `just wasm-build` recipe's env vars, so building without them ships **1,503,817 B — +109 KB
  (+7.9%) heavier, silently.** Measured. A wasm CI job **must** go through `just wasm-build`.

  **Observation for STAGE-027 (pre-existing, not this spec):** `optimize(img, "webp")` on wasm
  returns **lossless** WebP (320 KB vs the 44 KB JPEG) — there is no lossy WebP encoder in the wasm
  feature set, so the lossy/perceptual path silently isn't available for WebP. Worth a look before
  the demo offers WebP as an output.
- [x] **ship** (2026-07-12) — **SHIPPED.** Clean path, both commits `-s`. Squash-merged **PR #83**
  (`506df80`, DEC-066). Cost totals (330k tok / $3.00 est, 3 sessions); ship reflection appended
  (the propagated-wrong-lesson process note); spec + timeline archived. **STAGE-025 CLOSED** (3/3
  specs shipped) — stage status → shipped + Stage-Level Reflection filled. Roadmap: filed the wasm
  CI-job footgun (+109 KB) + the lossless-WebP-on-wasm observation for STAGE-027. `just validate` +
  `just cost-audit` green. **The build's false wasm-opt lesson had reached the auto-memory too —
  corrected there this ship.**
