---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-020
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship  (shipped 2026-06-17, PR #23)
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

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
  decisions: [DEC-022, DEC-021, DEC-020, DEC-004, DEC-018, DEC-019, DEC-016, DEC-015, DEC-007, DEC-012]
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
  related_specs: [SPEC-019, SPEC-018, SPEC-017, SPEC-016, SPEC-014, SPEC-005]

value_link: "Delivers the headline WebP value — LOSSY WebP (typically smaller than JPEG at equal quality) behind an off-by-default `webp-lossy` feature. Because SPEC-019 already shipped the pure-Rust WebP DECODER, BOTH auto-quality searches drive WebP with the feature on: `-q`, `--max-size` AND the perceptual `--target`/`--ssim` (the AVIF contrast — AVIF has no decoder, so only `--max-size` works on it). The first C dependency (vendored libwebp via cc), strictly opt-in."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Design authored by the ORCHESTRATOR (Opus) directly. Verified the dep empirically (the discipline): `webp` 0.3.1 → `libwebp-sys` 0.9.6 builds via `cc` (VENDORED libwebp, clean build, no system lib install); licenses webp MIT/Apache + libwebp-sys MIT + vendored libwebp BSD-3 → `cargo deny check licenses` GREEN with NO new exception (verified with the feature enabled). Pinned `webp::Encoder::from_rgba(&[u8], w, h).encode(quality: f32) -> WebPMemory` (Deref<[u8]>); use from_rgba/from_rgb on to_rgba8()/to_rgb8() bytes (NOT the webp crate's `image` feature → avoids a duplicate image crate). Emitted DEC-022 (lossy WebP via libwebp behind off-by-default `webp-lossy`; first C dep, opt-in; BOTH searches drive WebP since the decoder exists)."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 220000     # ORDER-OF-MAGNITUDE estimate — build ran in the orchestrator main loop (background subagents can't get Bash here); M spec (dep + two encode arms + predicates + CI + tests across two builds)
      estimated_usd: 1.98      # ~220k @ Opus 4.8 list ($5/$25 per MTok, ~80/20 in/out) — order of magnitude
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Built in the main loop (background subagents can't get Bash here), tokens are a labeled order-of-magnitude estimate. Added the webp optional dep + webp-lossy feature, the sink lossy arm (lossy iff quality set, else lossless fall-through), the identical quality-search arm, BOTH LossyFormat predicates (rewritten as cfg-gated matches), the CI webp-lossy job, and tests. Default/avif/webp-lossy builds all green; just deny green with NO new exception. Skipped the proposed WEBP_DEFAULT_QUALITY const (would be dead code — lossy only runs when a quality is already set)."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 110000     # ORDER-OF-MAGNITUDE estimate — 2 focused read-only review subagents (Explore): cross-sync/selection + predicates/default-unchanged/single-image-library; the Agent results didn't surface token counts
      estimated_usd: 0.99      # ~110k @ Opus 4.8 list ($5/$25 per MTok, ~80/20 in/out) — order of magnitude
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Independent review with 2 focused Explore subagents (sink↔search cross-sync byte-identity + lossy-iff-quality selection; predicate-rewrite regression + default-build-unchanged + single-image-library). Both clean — no findings. + CI green incl. the webp-lossy job. Token total is a labeled estimate."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null       # orchestrator main-loop bookkeeping — legitimately null (AGENTS §4)
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-17
      notes: "Ship bookkeeping on main (merge dance + archive): orchestrator main-loop, not separately metered."
  totals:
    tokens_total: 330000     # sum of non-null sessions (build 220k + verify 110k); design/ship main-loop (null → 0)
    estimated_usd: 2.97
    session_count: 4
---

# SPEC-020: Lossy WebP encode behind a feature-gated `libwebp` codec

## Context

The headline WebP value and STAGE-008's last format spec. SPEC-019 shipped WebP as a
pure-Rust default (lossless encode + decode) but deferred **lossy** encode — pure-Rust
`image-webp` does not do it. This spec adds lossy WebP behind an **off-by-default
`webp-lossy` cargo feature** (DEC-022) via the `webp` crate (→ `libwebp-sys` →
**vendored libwebp**, built with `cc`).

- **Parent stage:** `STAGE-008`, the lossy-WebP layer on top of SPEC-019's WebP wiring.
- **Why feature-gated (DEC-004 / DEC-022):** libwebp is **C** — the project's first C
  dependency. `libwebp-sys` **vendors** it and compiles it with `cc` (a C compiler, no
  system-library install). Keeping it opt-in preserves the default build's pure-Rust,
  zero-C, zero-system-deps promise (even AVIF was pure-Rust); `--features webp-lossy`
  opts in.
- **The payoff — BOTH searches drive WebP (the AVIF contrast):** because SPEC-019
  already gave us a pure-Rust WebP DECODER, the perceptual search (`--target`/`--ssim`,
  SPEC-016) can decode WebP candidates to score them — so with this feature WebP
  supports **both** `supports_lossy_quality` (byte budget) AND
  `supports_perceptual_quality` (perceptual). AVIF supports only the former (no decoder,
  DEC-020). This is the format that finally exercises the full auto-quality story on a
  modern format.
- **Lossy vs lossless selection (DEC-022):** WebP encodes **lossy** when an effective
  quality is set (an explicit `-q`, or a quality chosen by an auto-quality search); with
  no quality it stays **lossless** (the SPEC-019 default). So `shrink` (default `-q` 80)
  writes lossy WebP with the feature; a bare `convert --format webp` stays lossless.
  Without the feature, `-q`/auto on WebP are ignored (lossless) — exactly SPEC-019.
- **Verified dependency reality (design-time, empirical):**
  - `webp` 0.3.1 → `libwebp-sys` 0.9.6: **builds via `cc` (vendored libwebp)**, clean,
    no system lib install.
  - **License:** webp MIT/Apache, libwebp-sys MIT, vendored libwebp BSD-3 — cargo-deny
    `all-features` is **green with NO new exception**.
  - **API:** `webp::Encoder::from_rgba(&[u8], w, h).encode(quality: f32) -> WebPMemory`
    (and `.encode_lossless()`); `WebPMemory: Deref<[u8]>`. Use `from_rgba`/`from_rgb` on
    `DynamicImage::to_rgba8()`/`to_rgb8()` bytes; do NOT enable the webp crate's `image`
    feature (avoids a second `image` crate).

## Goal

Add an off-by-default `webp-lossy` feature that wires the `webp` crate (libwebp) into
the Sink and the auto-quality search so WebP is a real LOSSY output (`convert --format
webp -q Q`, `shrink`/`convert -o x.webp` with `-q`/`--target`/`--ssim`/`--max-size`),
selected when a quality is set; keep the default (no-feature) build's WebP at lossless
only (SPEC-019, `-q` ignored); keep the license gate green with NO new exception; add a
`--features webp-lossy` CI job. Change no default-build behavior.

## Inputs

- **Files to read:**
  - `Cargo.toml` — `[dependencies]` (add `webp` as an OPTIONAL dep,
    `default-features = false`) + `[features]` (add `webp-lossy = ["dep:webp"]`).
  - `src/sink/mod.rs` — `encode_to_bytes` (add the WebP lossy arm behind the feature,
    selected when `quality` is `Some`; lossless fall-through otherwise),
    `AVIF_SPEED`/`AVIF_DEFAULT_QUALITY` consts as the pattern for `WEBP_DEFAULT_QUALITY`.
  - `src/quality/mod.rs` — `encode_candidate_bytes` (add the `#[cfg(feature =
    "webp-lossy")]` WebP arm, IDENTICAL to the sink lossy encode), `LossyFormat`
    (`supports_lossy_quality` AND `supports_perceptual_quality` add WebP under the
    feature — WebP has a decoder, so BOTH).
  - `src/cli/mod.rs` — `resolve_effective_quality` (no structural change: the predicates
    drive it; with `webp-lossy`, WebP enters both search arms). Confirm `convert --format
    webp -q` threads quality to the lossy encode.
  - `.github/workflows/ci.yml` — the AVIF job as the template for a `webp-lossy` job.
  - `docs/api-contract.md` — the WebP notes (lossy with the feature).
  - `decisions/DEC-022-*.md` (the governing decision) + DEC-021 (the foundation) +
    DEC-020 (the AVIF contrast / cross-sync contract) + DEC-019/DEC-016.
- **External APIs:** the `webp` crate (MIT/Apache) → `libwebp-sys` (vendored libwebp).
  `webp::Encoder::from_rgba(&[u8], w, h)` / `from_rgb(...)`, then `.encode(quality: f32)`
  → `WebPMemory` (`Deref<[u8]>` → `.to_vec()`). docs: https://docs.rs/webp
- **Related code paths:** `Cargo.toml`/`ci.yml` + `src/sink` + `src/quality` +
  `src/cli`. Do NOT modify `src/image`, `src/operation`, `src/pipeline`, `src/recipe`.

## Outputs

- **Files modified:**
  - **`Cargo.toml`** — add `webp = { version = "=0.3.1", optional = true,
    default-features = false }` to `[dependencies]` (the FIRST new top-level dep since
    the metric crate — authorized by DEC-022) and `webp-lossy = ["dep:webp"]` to
    `[features]` (off by default). Run `just deny` (green, NO new exception).
  - **`src/sink/mod.rs`** — `encode_to_bytes`: add a WebP branch. `#[cfg(feature =
    "webp-lossy")]` AND `quality == Some(q)` → lossy encode:
    `let rgba = img.pixels().to_rgba8(); let enc = webp::Encoder::from_rgba(rgba.as_raw(),
    w, h); Ok(enc.encode(q.clamp(1,100) as f32).to_vec())`. Otherwise (no feature, or
    `quality == None`) → fall through to the existing lossless `write_to` path. Add
    `const WEBP_DEFAULT_QUALITY: u8 = 80;` (used only if a default is needed — `convert`
    has none, `shrink` supplies 80). NO `CodecNotBuilt` (lossless WebP is always
    available, so WebP output never exits 4).
  - **`src/quality/mod.rs`** — `encode_candidate_bytes`: `#[cfg(feature = "webp-lossy")]
    ImageFormat::WebP => { ...from_rgba(reference.to_rgba8())...encode(q as f32)... }`
    IDENTICAL to the sink (cross-sync contract, now covering WebP too).
    `supports_lossy_quality` AND `supports_perceptual_quality`: `#[cfg(feature =
    "webp-lossy")]` add `ImageFormat::WebP` to BOTH (WebP has a pure-Rust decoder, so
    the perceptual search can score it).
  - **`src/cli/mod.rs`** — no structural change; `resolve_effective_quality`'s predicate
    guards now admit WebP for both searches under the feature. Confirm via tests.
  - **`.github/workflows/ci.yml`** — add a `webp-lossy` job (single ubuntu runner,
    mirror the `avif` job): `cargo build/test/clippy --all-targets --features webp-lossy`.
    `cc` is preinstalled on ubuntu runners (no extra step).
  - **`docs/api-contract.md`** — WebP entry: with `--features webp-lossy`, `-q`/
    `--target`/`--ssim`/`--max-size` produce/tune LOSSY WebP; a bare `convert --format
    webp` stays lossless; without the feature, WebP is lossless only (SPEC-019).
- **New decisions:** `DEC-022` (emitted in this design cycle).
- **One new top-level dependency** (`webp`, authorized by DEC-022). **No new
  `Operation`. No default-build behavior change.**

## Acceptance Criteria

Each maps to a test. Tests split into **default-build** (lossless WebP, SPEC-019
behavior unchanged) and **feature-build** (`#[cfg(feature = "webp-lossy")]`, run by the
CI `webp-lossy` job). WebP output is verified via `image::guess_format == WebP` AND a
real decode (the decoder is built by default since SPEC-019).

- [ ] **Default build:** `convert <png> --format webp -q 50 -o out.webp` → exit 0;
  output is WebP and LOSSLESS (`-q` ignored without the feature). → reuse
  `webp_quality_is_ignored` (SPEC-019).
- [ ] **Feature build:** WebP supports BOTH searches:
  `ImageFormat::WebP.supports_lossy_quality()` and `.supports_perceptual_quality()` are
  both `true`. → `webp_supports_lossy_and_perceptual` (cfg webp-lossy)
- [ ] **Feature build:** lossy quality knob works: encode q30 bytes < q90 bytes (both
  guess WebP). → `encode_webp_lossy_respects_quality` (cfg webp-lossy)
- [ ] **Feature build:** `convert <detailed png> --format webp -q 20` → exit 0; output
  WebP and SMALLER than the same image as lossless WebP (lossy actually engaged). →
  `convert_to_lossy_webp_is_smaller` (cfg webp-lossy)
- [ ] **Feature build:** perceptual search drives WebP (the AVIF contrast): `shrink
  <png> --target high -o out.webp` → exit 0; output WebP; NO "needs a decoder" warning.
  → `webp_target_high` (cfg webp-lossy)
- [ ] **Feature build:** `convert <png> --format webp --max-size 4KB -o out.webp` →
  exit 0; output WebP ≤ 4000 bytes. → `webp_max_size_fits` (cfg webp-lossy)
- [ ] **Both builds:** `just deny` green with NO new exception; the default suite + all
  5 gates stay green; no default-build behavior change. → CI + existing suites

## Failing Tests

Written during **design**. Default-build tests live in the normal modules/`tests/cli.rs`;
feature-build tests are `#[cfg(feature = "webp-lossy")]`. WebP output is verified with
`image::guess_format` (and decode round-trips, since the decoder is built).

- **`src/sink/mod.rs`** (UNIT, `#[cfg(feature = "webp-lossy")]`):
  - `encode_webp_lossy_respects_quality` — encode a detailed image to WebP at quality
    30 vs 90 (via `encode_to_bytes(.., WebP, Some(q))`); assert both `guess_format ==
    WebP` and q30 length < q90 length.
- **`src/quality/mod.rs`** (UNIT, `#[cfg(feature = "webp-lossy")]`):
  - `webp_supports_lossy_and_perceptual` — both predicates true for WebP.
  - `auto_under_size_webp_is_monotone` — `auto_under_size(img, WebP, small).quality <=
    auto_under_size(img, WebP, large).quality`.
  - `auto_quality_webp_succeeds` — `auto_quality(img, WebP, &SearchConfig::for_target(70.0))`
    returns Ok (the perceptual round-trip decodes WebP and scores — proving WebP, unlike
    AVIF, supports the perceptual search).
- **`tests/cli.rs`** (INTEGRATION, `#[cfg(feature = "webp-lossy")]`):
  - `convert_to_lossy_webp_is_smaller` — `convert <detailed png> --format webp -q 20`
    → exit 0; WebP; bytes < the same image encoded as lossless WebP
    (`common::webp_lossless` of the same source).
  - `webp_target_high` — `shrink <png> --target high -o out.webp` → exit 0; WebP; stderr
    has NO decoder-fallback warning (contrast with `avif_target_high`).
  - `webp_max_size_fits` — `convert <detailed png> --format webp --max-size 4KB` → exit
    0; WebP; `len <= 4000`.
- **DEFAULT build:** `webp_quality_is_ignored` (SPEC-019, already present) continues to
  pass — `-q` on WebP without the feature stays lossless.

## Implementation Context

### Decisions that apply
- **`DEC-022`** (emitted here) — lossy WebP via the `webp` crate (libwebp, vendored,
  `cc`) behind off-by-default `webp-lossy`; first C dep, opt-in; lossy-iff-quality
  selection; BOTH `LossyFormat` predicates for WebP (decoder exists); webp used as an
  encode-only sink (no second image crate).
- `DEC-021` — the WebP foundation (lossless + decode, default) this layers onto.
- `DEC-020` — AVIF: the perceptual-needs-a-decoder finding + the cross-sync contract
  (probe bytes must equal written bytes); WebP now ALSO obeys the cross-sync contract,
  and — having a decoder — supports the perceptual search AVIF cannot.
- `DEC-004` — heavy/native codecs behind off-by-default features (libwebp = C → gated).
- `DEC-018` — license gate; green with NO new exception (all permissive).
- `DEC-019` — the auto-quality search reused unchanged via the per-format encode arm +
  predicates.
- `DEC-016` — `-q` → encoder quality (WebP, with the feature); ignored without it
  (lossless). `DEC-015` — format precedence.

### Constraints that apply
- `pure-rust-codecs-default` — the DEFAULT build stays pure-Rust + zero C/system deps;
  `webp-lossy` is the opt-in exception (vendored C, `cc`).
- `no-agpl-default-deps` — webp/libwebp-sys/libwebp are permissive; `just deny` green,
  NO new exception.
- `no-new-top-level-deps-without-decision` — `webp` is authorized by DEC-022.
- `single-image-library` — `webp` is an encode-only codec binding fed raw `to_rgba8()`
  bytes; NOT the webp crate's `image` feature; one pixel library (`image`) remains.
- `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`
  (BOTH default and `--features webp-lossy`), `untrusted-input-hardening` (quality
  clamped 1..=100; libwebp bounds the encode).

### Prior related work
- `SPEC-019` (shipped, PR #22) — WebP lossless + decode (default); this adds lossy.
- `SPEC-018` (shipped, PR #21) — AVIF: the feature-gated-codec + cross-sync pattern;
  the perceptual-needs-a-decoder finding WebP now satisfies.
- `SPEC-017` (shipped) — the format-agnostic search WebP plugs into.

### Out of scope (create a new spec rather than expand)
- Animated WebP (the `webp` crate's AnimEncoder); WebP-specific tuning (method/effort
  knobs); a system-libwebp (non-vendored) link option.
- The `--max-size` dimension-reduction fallback (separate backlog item).

## Notes for the Implementer

- **The dep is opt-in and vendored.** `Cargo.toml`: `webp = { version = "=0.3.1",
  optional = true, default-features = false }` + `webp-lossy = ["dep:webp"]`. Confirm
  `cargo build --features webp-lossy` (compiles vendored libwebp via `cc` — verified in
  design) and `just deny` (green, NO new exception — do NOT touch deny.toml).
- **Encode (pin this):** `let rgba = img.pixels().to_rgba8(); let (w, h) =
  rgba.dimensions(); let enc = ::webp::Encoder::from_rgba(rgba.as_raw(), w, h); let mem
  = enc.encode(q.clamp(1, 100) as f32); Ok(mem.to_vec())`. Keep
  `encode_candidate_bytes`'s WebP arm IDENTICAL (same `from_rgba`, same clamp, same
  `q as f32`) — the byte-budget/perceptual guarantee depends on the probe matching the
  written bytes (the cross-sync contract, now covering WebP).
- **Lossy iff a quality is set.** In `encode_to_bytes`, the WebP lossy arm runs only
  when `quality.is_some()` AND the feature is on; otherwise fall through to the existing
  lossless `write_to` path. Do NOT default WebP to lossy — a bare `convert --format
  webp` must stay lossless (SPEC-019). `shrink` supplies `-q` 80, so it goes lossy.
- **BOTH predicates for WebP (the AVIF contrast).** Unlike AVIF, add WebP to
  `supports_perceptual_quality` too — the pure-Rust decoder (SPEC-019) lets the
  perceptual search decode WebP candidates. Confirm `auto_quality(.., WebP, ..)` works
  (it calls `load_from_memory` on WebP bytes, which the default decoder handles).
- **Verifying lossy WebP in tests:** `image::guess_format == WebP`; for "lossy actually
  engaged," compare against `common::webp_lossless(..)` of the same source (lossy q20 <
  lossless for a detailed image). Decode round-trips work (the decoder is built).
- **clippy/test must pass BOTH builds.** Locally run `cargo clippy --all-targets
  --features webp-lossy -- -D warnings` and `cargo test --features webp-lossy` plus the
  default gates. Keep test images small (≤160px) so encodes stay fast.
- **Commit incrementally** (dep + feature + deny green → sink lossy arm → quality arm +
  predicates → CLI confirm + feature tests → CI job → docs).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-020-lossy-webp-behind-a-feature-gated-libwebp`
- **PR (if applicable):** *(opened during build — see timeline)*
- **All acceptance criteria met?** yes — `encode_webp_lossy_respects_quality`,
  `webp_supports_lossy_and_perceptual`, `auto_under_size_webp_is_monotone`,
  `auto_quality_webp_succeeds` (perceptual round-trip), `convert_to_lossy_webp_is_smaller`,
  `webp_target_high` (no decoder warning), `webp_max_size_fits` all pass; the SPEC-019
  `webp_quality_is_ignored` still passes in the default build. Default + avif +
  webp-lossy builds all green; `just deny` green with NO new exception.
- **New decisions emitted:**
  - `DEC-022` — lossy WebP via libwebp (feature-gated) [authored in design].
- **Deviations from spec:**
  1. **Dropped the proposed `WEBP_DEFAULT_QUALITY` const.** The lossy arm only runs
     when `quality.is_some()`, so there is no site that needs a WebP default — adding
     the const would be dead code (clippy). `convert` has no default; `shrink` supplies
     80. (Minor; the spec listed it as "used only where a default is needed" — there
     is no such place.)
  2. **Rewrote the `LossyFormat` predicates as cfg-gated `match` arms** (instead of
     stacked `#[cfg]` blocks) so two features (avif, webp-lossy) compose cleanly.
     Behaviorally identical; just readable. Broadened the sink test's `detailed_image`
     helper gate to `any(avif, webp-lossy)`.
- **Follow-up work identified:**
  - None new. STAGE-008's formats are complete (JPEG/AVIF/WebP); the remaining backlog
    item is the `--max-size` dimension-reduction fallback. The jpegli-encode license
    question is parked in `guidance/license-watchlist.yaml` for a future JPEG spec.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing. The design's empirical dep probe (build via cc, deny green, pinned
   `from_rgba(...).encode(q)`) meant the build was mechanical. The one judgment call
   (no `WEBP_DEFAULT_QUALITY`) was obvious once writing the arm.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `single-image-library` was the one to watch, and feeding `to_rgba8()` bytes to
   `from_rgba` (not the webp crate's `image` feature) kept it satisfied, exactly as the
   spec specified.

3. **If you did this task again, what would you do differently?**
   — Nothing material. Splitting WebP into SPEC-019 (pure-Rust) + SPEC-020 (the C dep)
   paid off: this spec was a clean, isolated layer on top of working WebP wiring, and
   the first-C-dependency decision got its own DEC and verification rather than being
   buried in a larger change.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — Nothing material. Splitting WebP into SPEC-019 (pure-Rust foundation) and
   SPEC-020 (the libwebp C dep) was the right call — it isolated the project's first
   C dependency in its own DEC + verification + CI job, and made this spec a clean
   layer on already-working WebP wiring. The design-time `cc`-build + `just deny`
   probe meant zero surprises at build.

2. **Does any template, constraint, or decision need updating?**
   — No update needed, but worth noting the pattern is now established: a
   feature-gated native codec = its own DEC (license + build-cost + any constraint
   tension) + an empirical design probe (build, deny, pinned API) + a dedicated CI
   job. AVIF (DEC-020) and lossy WebP (DEC-022) both followed it; the next native
   codec should too. The `single-image-library` constraint held by feeding raw
   `to_rgba8()` bytes to an encode-only crate — a reusable tactic.

3. **Is there a follow-up spec I should write now before I forget?**
   — Not a new one — STAGE-008's formats are complete (JPEG/AVIF/WebP, lossless +
   lossy). The remaining stage item is the **`--max-size` dimension-reduction
   fallback** (already in the backlog); designing it next. The parked jpegli-encode
   license question stays in `guidance/license-watchlist.yaml` for a future JPEG spec.
