---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-073
  type: story
  cycle: ship  # frame | design | build | verify | ship
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
    - cycle: build
      interface: claude-code
      tokens_total: 300000
      note: >
        ORDER-OF-MAGNITUDE ESTIMATE (build ran in the main loop, not a metered
        subagent). Single session, no rework: measured the size delta on the real
        release artifact (lean vs `--features avif`), wired the wasm surface, drove
        the encode in a real wasm VM (10/10 wasm tests green), ran the full native
        gate set, and emitted DEC-065. The one unplanned piece of work was the
        `optimize`/perceptual-search guard (the DEC-019 search needs a decoder that
        AVIF on wasm does not have).
    - cycle: verify
      interface: claude-code
      tokens_total: 200000
      note: >
        ORDER-OF-MAGNITUDE ESTIMATE (verify ran in the main loop, not a metered
        subagent). Fresh adversarial session, no rework: re-drove the 10 wasm tests,
        wrote + ran 19 adversarial probes of its own in the wasm VM (removed after),
        extracted the wasm-produced AVIF bytes and decoded them with two independent
        AV1 decoders, reproduced both size builds, and ran the full native gate set.
        Verdict CLEAN — no punch list.
  totals:
    tokens_total: 500000        # build 300k + verify 200k (design null, un-metered main loop)
    estimated_usd: 4.50         # ~500k @ ~$9/MTok — LABELLED ESTIMATE, not a meter read (§4)
    session_count: 3
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

- [x] The wasm surface encodes to AVIF: a `#[wasm_bindgen_test]` takes a PNG (+ optional recipe)
      and `transform(..., "avif")` returns bytes that are **valid AVIF** (ftyp/brand check, or a
      native decode in the assertion), in a real wasm VM (`just wasm-test`). No panic/abort.
- [x] AVIF **input** still returns the typed `CodecUnavailableOnTarget` error on wasm (decode
      unchanged) — no regression from SPEC-072.
- [x] The `.wasm` **size delta with `--features avif` is measured and recorded** (raw/gzip/brotli)
      against the 1.19 MB brotli baseline; DEC-065 states the number and the shipping strategy.
- [x] Native builds unaffected: `cargo build`, `cargo build --no-default-features`, `cargo test`,
      `cargo clippy` green; native AVIF encode (`--features avif`) still works on the real binary;
      `just deny` unchanged.
- [x] `just wasm-build` (and the with-AVIF variant) reproduce on the stable toolchain; DEC-065 committed.

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

- **Branch:** `feat/spec-073-avif-on-wasm`
- **PR:** #82
- **All acceptance criteria met?** Yes.
  - **AVIF encode in the wasm surface** — `transform(png, RESIZE_RECIPE, "avif")` returns bytes
    with a `ftyp` box and an `avif` major brand, asserted **inside a real wasm VM**
    (`just wasm-test`: 10/10 `#[wasm_bindgen_test]`s green, no panic/abort). The output is sniffed
    rather than decoded back — the wasm build has no AVIF decoder, which is the point.
  - **AVIF input still errors** — new `avif_input_still_errors_on_wasm` (alongside the SPEC-072
    test) proves that turning ENCODE on did not quietly turn DECODE on: still typed
    `CodecUnavailableOnTarget`, still no `--features` advice a browser user can't act on.
  - **Size delta measured** (release, post `wasm-opt`, same machine/toolchain as SPEC-072):
    | | lean | **with `avif`** | delta |
    |---|---|---|---|
    | raw | 4,496,577 B | 6,415,270 B | +1.83 MB (+42.7%) |
    | gzip | 1,716,575 B | 2,272,806 B | +0.53 MB (+32.4%) |
    | **brotli** | **1,248,818 B (1.19 MB)** | **1,594,482 B (1.52 MB)** | **+345,664 B (+27.7%)** |
    The lean column reproduces the SPEC-072 baseline byte-for-byte.
  - **Native unaffected** — `cargo build`, `cargo build --no-default-features`, `cargo test`
    (29 suites, 0 failures), `cargo test --features avif`, `cargo clippy --all-targets -D warnings`,
    `just deny` (advisories/bans/licenses/sources ok — **no new exception**; rav1e/ravif/av1-grain/
    av-scenechange are permissive pure Rust). **`Cargo.toml` has no dependency change at all** —
    only feature-doc comments — so the native feature matrix and released binary are byte-identical.
  - **Reproducible on stable** — `just wasm-build` (shipped, with AVIF) and
    `just --set _wasm_features "" wasm-build` (the lean comparison SPEC-074 needs). DEC-065 committed.
- **New decisions emitted:** **DEC-065** — AVIF **encode is IN** the wasm build (shipped artifact is
  built `--features avif`); **decode is deferred, not scheduled** (the browser already has an AVIF
  decoder — `createImageBitmap`; shipping a second one is the wrong trade at any size); **one
  artifact, no lazy AVIF chunk** (wasm modules don't share code, so a "chunk" is a second full
  1.52 MB engine — the AVIF user would pay 2.71 MB instead of 1.52 MB; the bundle's real problem is
  the 1.19 MB core, which is SPEC-074's).
- **Deviations from spec:** two, both additive.
  1. **`optimize(_, "avif")` needed a guard the spec didn't anticipate.** The perceptual quality
     search encodes a candidate and **decodes it back** to score it (DEC-019) — so it needs a
     DECODER, which AVIF on wasm does not have. `src/wasm.rs::optimize` decided lossy-vs-lossless on
     `supports_lossy_quality` (encode-knob only); with `avif` on, AVIF would have entered the search
     and failed on candidate #1's decode. Now it guards on `supports_perceptual_quality` (the same
     seam the CLI guards on) and encodes once at the encoder's default quality. Covered by a new
     test (`optimize_to_avif_encodes_without_the_perceptual_search`).
  2. **The `avif` cargo feature stayed the single gate** rather than making AVIF unconditional on
     wasm (`any(feature, target_arch)` + a wasm dep-table entry). Two reasons, argued in DEC-065:
     it would smear the cfg's meaning across ~8 sites in `sink`/`quality`, and it would **weld
     rav1e to the target** — deleting the lean comparison build SPEC-074 needs. The justfile
     remembers the flag so a human doesn't have to.
- **Follow-up work identified:**
  - **STAGE-027 (demo page) inherits two hard constraints:** (a) rav1e runs **serial** on wasm
    (`maybe-rayon`, no threads) — AVIF encode must go in a Web Worker with visible progress or the
    page will feel hung; (b) `.avif` **inputs** must be decoded by the page via `createImageBitmap`,
    since we return a typed error.
  - **STAGE-026 (npm packaging) / a wasm CI job must build through `just wasm-build`.** A bare
    `cargo build --target wasm32-…` silently ships an artifact whose headline call answers "codec
    not built". (The wasm CI job was already a SPEC-072 follow-up; this raises its stakes.)
  - **SPEC-074 inherits a sharpened brief:** 1.52 MB brotli, of which ~1.19 MB is engine and
    ~0.35 MB is rav1e. The levers are `ssimulacra2`, the resvg text stack, unused `image` codecs,
    and the `crustyimg-core` split — not the headline codec.
  - **`quality::supports_perceptual_quality`'s doc comment is stale on native.** It says AVIF has
    "no decoder built", which stopped being true at SPEC-058 (re_rav1d). The *predicate* may still
    be right (perceptual AVIF was never wired), but the stated reason is wrong on native and only
    accidentally right on wasm. Out of scope here; worth a cleanup spec.

### Build-phase reflection

1. **What was unclear in the spec that slowed you down?** — Nothing blocked me; the spec was
   unusually well-loaded (the probe result, the seam, the baseline, the exact files). The one thing
   it did not anticipate was the `optimize`/perceptual-search collision above: the spec framed the
   work as "let the format string reach the existing encode arm", which is true for `transform` but
   **not** for `optimize` — turning the `avif` feature on changed `supports_lossy_quality(Avif)` to
   `true`, which silently re-routed `optimize` into a search that cannot run without a decoder. A
   grep for `feature = "avif"` found it in a minute, but only because I went looking for what the
   feature flag *else* switches on. **Lesson worth carrying: enabling a feature flag on a new target
   changes every `cfg(feature)` site in the crate, not just the one you came for** — the sibling of
   the "IMAGE_EXTENSIONS exposes every decode caller" lesson, and it deserves the same reflex.
2. **Was there a constraint or decision that should have been listed but wasn't?** — DEC-019 (the
   perceptual search decodes candidates to score them). It is the reason the `optimize` guard exists,
   and it was not in the spec's "Decisions that apply" list. Listing it would have pointed straight
   at the collision.
3. **If you did this task again, what would you do differently?** — Measure the size delta *first*,
   before writing a line of code. I did the builds early by luck of ordering, but it should be
   deliberate: this spec's whole shape (one artifact vs. a split) hinged on one number, and every
   code decision downstream of it was cheap to make once the number existed and would have been
   expensive to unwind had it come out at, say, +2 MB brotli.

---

## Reflection (Ship)

*Appended during ship (2026-07-12). Shipped via PR #82 (squash `f027d79`, DEC-065); the clean
path — no out-of-scope commits to split, all three commits signed off (the SPEC-072 DCO lesson
landed).*

1. **What would I do differently next time?** — Nothing structural; this is the pattern working.
   The design-time probe (does `rav1e` compile to wasm?) answered the spec's central question
   before a line was written, so the build had no design surprises — its one deviation
   (`optimize`'s perceptual-search guard) wasn't a design miss but a **second-order consequence of
   flipping a cargo feature**: turning `avif` on for a new target lit up *every* `cfg(feature =
   "avif")` site, including `supports_lossy_quality(Avif) → true`, which re-routed `optimize` into a
   search that decodes candidates — which AVIF-on-wasm can't. The generalizable lesson (banked in
   memory): **enabling a feature for a new target flips every `cfg(feature)` site at once — audit
   the whole set, not just the arm you came for.** It's the `IMAGE_EXTENSIONS`-exposes-every-caller
   lesson in a new guise.
2. **Does any template, constraint, or decision need updating?** — DEC-065 records the durable
   calls: encode-in / decode-deferred (the browser's own `createImageBitmap` reads `.avif`, so
   shipping a second decoder is the wrong trade at any size), and **one artifact, no lazy AVIF
   chunk** (wasm modules don't share code → a "chunk" is a second full 1.52 MB engine; the AVIF
   user would pay 2.71 MB). The verify method itself is worth keeping: **a `ftyp`/magic-byte sniff
   is not proof of a valid file — decode the wasm-produced bytes with an independent decoder**
   (native re_rav1d + macOS `sips`, the browser's class); banked as its own memory. Also confirmed:
   verify must commit `-s` (done this time).
3. **Is there a follow-up spec I should write now before I forget?** — Filed to `docs/roadmap.md`:
   (a) **STAGE-027 inherits two hard constraints** — rav1e runs *serial* on wasm (no threads), so
   AVIF encode must run in a **Web Worker** with visible progress or the page feels hung; and
   `.avif` *inputs* must be decoded page-side via `createImageBitmap` (we return a typed error).
   (b) **A docs-cleanup follow-up** now owns two stale native doc strings that predate SPEC-058's
   native AVIF decode: `docs/api-contract.md:244` ("reading an `.avif` fails") and
   `quality::supports_perceptual_quality`'s doc comment ("no decoder built"). Neither is a SPEC-073
   defect; both are pre-existing and out of scope. (c) The **wasm CI job** (already a SPEC-072
   follow-up) gains stakes — a bare `cargo build --target wasm32` silently ships an artifact whose
   headline call answers "codec not built"; CI must build through `just wasm-build`. SPEC-074
   (bundle size) is next in STAGE-025 regardless, with a sharpened brief: of 1.52 MB brotli, ~1.19
   is engine and ~0.35 is rav1e — the levers are `ssimulacra2` / resvg text / unused `image` codecs
   / the `crustyimg-core` split, **not** the headline codec.
