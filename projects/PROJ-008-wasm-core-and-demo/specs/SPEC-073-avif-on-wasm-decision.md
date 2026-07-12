---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-073
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L

project:
  id: PROJ-008
  stage: STAGE-025
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # separate build session
  created_at: 2026-07-12

references:
  decisions:
    - DEC-064    # the wasm target-cfg boundary SPEC-072 established (this extends it for AVIF)
    - DEC-053    # re_rav1d AVIF decode — the crate that can't compile to wasm32
    - DEC-020    # AVIF output behind the off-by-default `avif` feature (image/avif → ravif → rav1e)
    - DEC-004    # pure-Rust default (rav1e/ravif are pure-Rust, no nasm on this path)
  constraints:
    - pure-rust-codecs-default
    - untrusted-input-hardening
  related_specs:
    - SPEC-072   # the wasm build seam this extends (shipped)
    - SPEC-058   # AVIF decode (re_rav1d) — the gated-out direction
    - SPEC-074   # bundle size — this spec hands it the AVIF size delta

value_link: >
  Resolves the AVIF-on-wasm question STAGE-025 was organized around: proves and wires the
  "convert to AVIF in-browser" headline (rav1e encode), and records the decode asymmetry as a
  DEC — so the demo's compelling moment is real and honestly scoped.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      note: >
        framed build-ready in the orchestrator main loop; grounded in a design-time
        probe (2026-07-12) that compiled `--features avif` to wasm32 — rav1e 0.8.1 +
        ravif 0.13.0 built clean (exit 0), proving AVIF encode is achievable on wasm.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-073: AVIF-on-wasm decision (encode in, decode deferred) + DEC

## Context

Second spec of STAGE-025. SPEC-072 shipped the wasm build seam with AVIF **decode** gated out
(`re_rav1d` can't compile to bare wasm32 — libc POSIX types + threads). The open question the
whole stage was organized around: what is the AVIF story in the browser? A **design-time probe
(2026-07-12)** answered the encode half — `cargo build --lib --target wasm32-unknown-unknown
--features avif` compiled clean (exit 0, 23s): **`rav1e` 0.8.1 and `ravif` 0.13.0 build to
wasm32.** So the asymmetry is now proven and load-bearing:

- **AVIF encode (rav1e/ravif, the `avif` feature): COMPILES on wasm32** → "drop a PNG, get a
  tiny AVIF in-browser" — the demo's compelling headline — is achievable.
- **AVIF decode (re_rav1d): does NOT compile on wasm32** (SPEC-072) → reading `.avif` *inputs*
  in-wasm is not available; the browser's native `createImageBitmap` is the demo escape hatch.

This spec makes that a **decision + DEC-065**, wires AVIF encode into the wasm surface (so the
headline has teeth), and **measures the `.wasm` size delta** rav1e adds — the decisive input for
SPEC-074, and for whether encode ships default-on or via a size-managed path.

## Goal

Wire AVIF **encode** into the wasm surface (a PNG/JPEG → valid `.avif` round-trip in the browser),
record the encode-in/decode-deferred decision as **DEC-065**, and measure + record the `.wasm`
size delta rav1e adds — choosing the encode shipping strategy from that measured number.

## Inputs

- **Files to read:**
  - `src/wasm.rs` — the shipped `transform`/`optimize` surface (SPEC-072) to extend for `avif` out.
  - `src/sink/mod.rs` — `encode_to_bytes` + the `#[cfg(feature = "avif")]` AVIF encode arm
    (~:611) and the `CodecNotBuilt`/off arm (~:628); the wasm surface must reach the on arm.
  - `Cargo.toml` — the `avif = ["image/avif"]` feature (:222); the wasm target dep tables (DEC-064).
  - `justfile` — `wasm-build`/`wasm-check`/`wasm-size` (:97-121) — the recipes to teach `--features avif`.
  - `docs/research/proj-008-wasm-build.md` — the SPEC-072 size baseline (1.19 MB brotli) to diff against.
- **Related code paths:** `src/image/sniff.rs` (AVIF detection; decode still returns the typed
  `CodecUnavailableOnTarget` on wasm — unchanged).

## Outputs

- **Files created/modified:**
  - `decisions/DEC-065-*.md` — the AVIF-on-wasm scope decision (see below).
  - `src/wasm.rs` — accept `out_format = "avif"` in `transform`/`optimize`; encode via the
    existing `avif`-feature path. AVIF *input* still returns the typed error (decode unchanged).
  - `Cargo.toml` / `justfile` — enable the `avif` feature for the wasm build (per the chosen
    shipping strategy); `wasm-build`/`wasm-size` report the with-AVIF size.
  - `docs/research/proj-008-wasm-build.md` — append the AVIF size delta + the run.
  - `tests/wasm_roundtrip.rs` — a `#[wasm_bindgen_test]` PNG → AVIF encode test asserting the
    output is valid AVIF (sniff the `ftyp`/brand, or decode natively in the test assertion).
- **New decision — DEC-065 (draft the build finalizes):**
  - AVIF **encode is supported** on wasm (rav1e compiles); AVIF **decode is not** (re_rav1d),
    deferred — reading `.avif` inputs is a demo-side `createImageBitmap` concern (STAGE-027), not
    an in-wasm capability. The typed `CodecUnavailableOnTarget` on the decode path stands.
  - The **encode shipping strategy**, chosen from the measured size delta (see Acceptance): if
    rav1e's delta keeps the artifact within a sane first-load budget, ship `avif` on in the wasm
    build; if it blows the budget, ship a **size-managed path** (a separate avif-enabled wasm
    artifact / lazy-loaded chunk, or opt-in) rather than bloating the default demo bundle. Record
    the number and the choice; this is the concrete input SPEC-074 optimizes.

## Acceptance Criteria

- [ ] The wasm surface encodes to AVIF: a `#[wasm_bindgen_test]` takes a PNG (+ optional recipe)
      and `transform(..., "avif")` returns bytes that are **valid AVIF** (ftyp/brand check, or a
      native decode in the assertion), in a real wasm VM (`just wasm-test`). No panic/abort.
- [ ] AVIF **input** still returns the typed `CodecUnavailableOnTarget` error on wasm (decode
      unchanged) — no regression from SPEC-072.
- [ ] The `.wasm` **size delta with `--features avif` is measured and recorded** (raw/gzip/brotli)
      against the 1.19 MB brotli baseline; DEC-065 states the number and the shipping strategy.
- [ ] Native builds unaffected: `cargo build`, `cargo build --no-default-features`, `cargo test`,
      `cargo clippy` green; native AVIF encode (`--features avif`) still works on the real binary;
      `just deny` unchanged.
- [ ] `just wasm-build` (and the with-AVIF variant) reproduce on the stable toolchain; DEC-065 committed.

## Failing Tests

Written now (design), before build.

- **`tests/wasm_roundtrip.rs`**, `#[wasm_bindgen_test]`:
  - `"transform_png_to_avif_is_valid_avif"` — PNG in, `transform(png, recipe, "avif")` → bytes
    whose header sniffs as AVIF (`ftyp` + an `avif`/`avis` brand via `image::sniff`/`is_avif`),
    length > 0. Asserts rav1e encode runs to completion in the wasm VM.
  - `"avif_input_still_errors_on_wasm"` — an AVIF fixture as *input* → `Err`
    (`CodecUnavailableOnTarget`), no panic. Guards the decode direction stays gated.
- **Native guard** (`#[cfg(not(target_arch = "wasm32"))]`): `"native_avif_encode_still_works"` —
  assert `--features avif` encode still produces valid AVIF natively (reference the existing avif
  encode test if one covers it, rather than duplicating).

## Implementation Context

### Decisions that apply
- `DEC-064` — the `cfg(target_arch = "wasm32")` boundary + target dep tables; this spec turns the
  `avif` feature **on** for the wasm build (or a variant), staying within that boundary.
- `DEC-020` — AVIF output is the off-by-default `avif` feature (`image/avif` → `ravif` → `rav1e`,
  pure-Rust, no nasm). The probe confirmed this whole chain compiles to wasm32.
- `DEC-053` — re_rav1d decode; the direction that stays gated (do not attempt to restore it here).

### Constraints that apply
- `pure-rust-codecs-default` — rav1e/ravif are pure-Rust; no C/nasm enters the wasm build.
- `untrusted-input-hardening` — **on wasm a panic aborts the module / crashes the page** (the
  SPEC-072 lesson): the AVIF encode path must return a typed `Err` on any failure, never panic.
  Carry the decode caps (DEC-034/063) — they already gate dimensions before encode.

### Prior related work
- `SPEC-072` (shipped, PR #80, DEC-064) — the wasm surface + `just wasm-*` recipes + size baseline.
- `SPEC-058` (AVIF decode) — the gated-out direction; `src/image/sniff.rs` is the shared detector.

### Out of scope (for this spec)
- **Restoring AVIF decode on wasm** (porting/shimming `re_rav1d`, or `wasm32-wasi`) — explicitly
  deferred by DEC-065; its own spec only if ever pulled. Reading `.avif` inputs in the demo is a
  STAGE-027 `createImageBitmap` concern, not this spec.
- **Bundle-size *optimization*** — SPEC-074. This spec only *measures* the AVIF delta and picks a
  shipping strategy; it does not shrink rav1e.
- npm packaging (STAGE-026), the demo page (STAGE-027).

## Notes for the Implementer

- **The probe proved compile, not runtime or size.** rav1e uses `maybe-rayon` (it compiled); on
  wasm with no threads it runs serial — fine, just slower. **Drive the real encode in the wasm VM**
  (the wave's earned-verdict rule) and **measure the release size** — don't assume.
- **rav1e is a large encoder.** Expect a meaningful `.wasm` size jump; that number *is* the point
  of the size acceptance criterion and drives the default-on-vs-size-managed call. If it's large,
  prefer a size-managed path over silently bloating the demo bundle — and say so in DEC-065.
- Reuse the shipped seam: `src/wasm.rs` already routes to `sink::encode_to_bytes`; the `avif` arm
  exists behind `#[cfg(feature = "avif")]`. This is mostly "turn the feature on for wasm + let the
  format string reach the arm + test", not new encode logic.
- Next DEC id is **DEC-065**. `docs/api-contract.md`/README may mention AVIF as native-only — sync
  if you change the wasm story (the SPEC-071 doc-drift lesson).

---

## Build Completion

*Filled in at the end of the build cycle.*

- **Branch:**
- **PR:**
- **All acceptance criteria met?**
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** —
2. **Was there a constraint or decision that should have been listed but wasn't?** —
3. **If you did this task again, what would you do differently?** —

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
