# SPEC-073 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in a
  design-time probe: `cargo build --lib --target wasm32-unknown-unknown --features avif` compiled
  clean (exit 0) — **rav1e 0.8.1 + ravif 0.13.0 build to wasm32**, so AVIF *encode* is achievable
  (the "convert to AVIF in-browser" headline); AVIF *decode* (re_rav1d) stays gated (SPEC-072).
  Spec = wire encode into the wasm surface + measure the size delta + DEC-065 (encode in, decode
  deferred). Failing tests specified.
- [x] **build** (2026-07-12, branch `feat/spec-073-avif-on-wasm`, PR #82) — AVIF encode RUNS in the
  wasm VM: `transform(png, recipe, "avif")` returns ftyp/`avif`-branded bytes, 10/10
  `#[wasm_bindgen_test]`s green under `just wasm-test`. Size measured on the release artifact:
  **1.19 → 1.52 MB brotli (+345 KB, +27.7%)** → strategy = **one artifact, `avif` ON** (a lazy
  chunk would re-link the whole engine: 2.71 MB for the AVIF user). **DEC-065** emitted (encode in
  / decode deferred — the browser's own `createImageBitmap` reads `.avif`). `optimize(_, "avif")`
  skips the perceptual search (it needs a decoder). NO `Cargo.toml` dep change; native + lean +
  clippy + deny green. Ready for verify.
- [x] **verify** (2026-07-12, fresh adversarial session) — **CLEAN.** Every claim re-earned on the
  real artifact, plus 19 adversarial probes of my own (written, run in the wasm VM, removed).
  - **The AVIF is real AVIF, not a brand sniff.** The build asserted `ftyp`/`avif` on 4 bytes; I
    smuggled the wasm-VM-produced bytes out as hex and decoded them with **two independent AV1
    decoders** — our native `re_rav1d` (`info` → `64x48, format avif, rgb8`) and **macOS's own
    system decoder** (`sips` → `pixelWidth 64, pixelHeight 48, format avif`), the same class of
    decoder the browser would use. 515 bytes, decodes clean. `just wasm-test` 10/10 green.
  - **Decode stayed gated, through EVERY entry point.** The build tested `transform`; I also probed
    `optimize(avif, "png")`, `optimize(avif, "auto")`, `info(avif)`, and AVIF-in/AVIF-out — all
    typed `CodecUnavailableOnTarget`, none advising `--features`, zero panics.
  - **The `optimize` guard holds.** `optimize(_, "avif")` encodes once and never enters the
    perceptual search, on the explicit AND the `auto` path. Confirmed in source: the guard is
    `!fmt.supports_perceptual_quality()`, which excludes AVIF *even with the feature on* — so it
    catches the auto path too, not just the one the test drives. `decide.rs` already has tests
    asserting AVIF is absent from the perceptual shortlist.
  - **19/19 adversarial probes returned typed `Err` or sane `Ok` — zero panics, zero module
    aborts.** A forged PNG *declaring* 20000×20000 (400M px) is refused by the DEC-034/063 pixel
    cap before rav1e allocates, through all three entry points; empty/garbage/truncated input;
    bogus `out_format` (incl. `""`, `../../etc/passwd`, `AVIF\0`); bogus recipe TOML; and rav1e's
    edge geometry — 1×1, odd 7×13 dims, semi-transparent and fully-transparent alpha — all encode
    to valid AVIF without a trap.
  - **Size delta reproduced** independently, both builds: brotli lean **1,248,423 B** → avif
    **1,595,028 B** = **+346,605 B (+27.8%)**, within 0.03% of the recorded +345,664 (+27.7%);
    raw/gzip percentages match exactly. The 1.19 → 1.52 MB headline stands.
  - **Native unaffected.** `cargo build`, `cargo build --no-default-features` (LEAN, run
    explicitly), `cargo test` (0 failures across every suite), `cargo test --features avif`
    (`native_avif_encode_still_works` + `native_avif_still_decodes` green),
    `cargo clippy --all-targets -D warnings` clean, `just deny` **advisories/bans/licenses/sources
    ok — no new exception**. `Cargo.toml` diff vs merge-base is **comments only** and `Cargo.lock`
    is **untouched**, so the dep-change claim is true at the diff level, not just asserted.
  - **DEC-065 is well-formed** and matches what landed: one artifact with `avif` ON (not a split),
    decode deferred-not-scheduled, `createImageBitmap` as the escape hatch. Its two load-bearing
    code claims were checked against source, not taken on trust.
  - **Follow-up (pre-existing, NOT a SPEC-073 defect):** `docs/api-contract.md:244` still says
    "**AVIF input (decode) is not supported** … reading an `.avif` fails". That line dates to
    SPEC-019 (PR #22, 2026-06-17) and went stale when **SPEC-058** shipped native AVIF decode — the
    native binary demonstrably reads `.avif` today. Untouched by this branch; worth folding into
    the cleanup spec that already owns `supports_perceptual_quality`'s stale doc comment.
- [x] **ship** (2026-07-12) — **SHIPPED.** Clean path (no out-of-scope commits to split; all 3
  commits signed off — SPEC-072's DCO lesson landed). Squash-merged **PR #82** (`f027d79`, DEC-065).
  Cost totals filled (500k tok / $4.50, labelled estimates, 3 sessions); ship reflection appended;
  spec + timeline archived. STAGE-025 backlog: SPEC-073 shipped (2 shipped / 0 active / 1 pending —
  only SPEC-074 size left). Follow-ups → `docs/roadmap.md` (STAGE-027 Web-Worker + createImageBitmap
  constraints; a docs-cleanup for 2 stale native AVIF-decode doc strings; wasm CI job stakes).
  `just validate` + `just cost-audit` green.
