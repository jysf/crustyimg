---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-018
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
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
  created_at: 2026-06-16

references:
  decisions: [DEC-020, DEC-004, DEC-018, DEC-019, DEC-016, DEC-015, DEC-007, DEC-012]
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
  related_specs: [SPEC-017, SPEC-016, SPEC-014, SPEC-005, SPEC-013]

value_link: "Delivers the first MODERN output format — `convert --format avif` / `shrink -o x.avif` produce AVIF (typically 30-50% smaller than JPEG at equal quality), behind an off-by-default `avif` feature. The format-agnostic search (SPEC-017) makes auto-quality AND `--max-size` work on AVIF for free — the differentiator's payoff."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-16
      notes: "Design authored by the ORCHESTRATOR (Opus) directly. Verified empirically (the SPEC-016 'pin the dep in design' discipline): `image/avif` → `ravif` builds PURE-RUST with no nasm/system deps (clean build, nasm absent); the dep tree is permissive in the shipped binary — the only non-allowlisted license (NCSA via `libfuzzer-sys`) is a FUZZ-only transitive of `rav1e` NOT present in `--features avif` (cargo-deny all-features pulls it). Pinned `AvifEncoder::new_with_speed_quality(w, speed 1-10, quality 1-100)`. Emitted DEC-020 (adopt ravif feature-gated; the deny exception; output-only v1)."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-018: AVIF output behind a feature-gated `ravif` codec

## Context

STAGE-008's first MODERN output format. AVIF (AV1 still-image) is typically
**30–50% smaller than JPEG at equal quality** — the headline reason to adopt a
modern format. `convert --format avif` already **exits 4** today (the codec isn't
built, DEC-004); this spec makes it real behind an **off-by-default `avif` cargo
feature** so the default binary stays pure-Rust + zero-system-deps, and a
`--features avif` build gains AVIF output.

- **Parent stage:** `STAGE-008` (modern formats & quality), spec #3 of the
  recommended order — the first format addition after the quality core (SPEC-016/017).
- **Why feature-gated (DEC-004 / DEC-020):** AVIF encode is pure-Rust via
  **`ravif`** (the `image` crate's `avif` feature), but `ravif` pulls **`rav1e`**
  (~18 MB, ~431k SLoC, slow at high quality). Keeping it behind an opt-in feature
  preserves the default build's fast compile, small binary, and instant install.
- **The payoff of SPEC-017's generalization:** because the auto-quality search is
  now **format-agnostic** (`LossyFormat::supports_lossy_quality` + the per-format
  `encode_candidate_bytes` arm), `--target`/`--ssim` (SPEC-016) AND `--max-size`
  (SPEC-017) work on AVIF the moment its encode arm is added — **no search changes**.
- **Scope: AVIF OUTPUT only.** Decoding an `.avif` INPUT needs `dav1d` (C, the
  `avif-native` feature) and is **deferred** (documented). So `convert in.avif
  --format png` is out of scope; `convert in.jpg --format avif` is in.
- **Verified dependency reality (design-time, empirical):**
  - `avif = ["image/avif"]` → `ravif` 0.13 → `rav1e` 0.8: **builds pure-Rust, no
    `nasm`, no system libs** (confirmed by a clean build with nasm absent).
  - **License:** the shipped `--features avif` tree is fully permissive. cargo-deny
    `all-features` flags exactly one crate — **`libfuzzer-sys` (`(MIT OR Apache-2.0)
    AND NCSA`)** — a **fuzz-only** transitive of `rav1e` **not present** in
    `cargo tree -e normal --features avif`. NCSA is permissive (OSI/FSF approved).
    Fix: a **scoped `deny.toml` exception** for `libfuzzer-sys` (mirrors the
    `ansi_colours` LGPL exception precedent, DEC-018), NOT a blanket NCSA allow.

The api-contract already documents `convert --format avif` → exit 4 without the
feature; this spec makes the with-feature path real.

## Goal

Add an off-by-default `avif` cargo feature that wires `ravif` (via `image/avif`)
into the Sink and the auto-quality search so AVIF is a real output format
(`convert --format avif`, `shrink`/`convert -o x.avif`, with `-q`/`--target`/
`--ssim`/`--max-size`); keep AVIF output **exit 4** in the default (no-feature)
build (DEC-004); keep the license gate green via a scoped `libfuzzer-sys`
exception; add a `--features avif` CI job. Change no default-build behavior.

## Inputs

- **Files to read:**
  - `src/sink/mod.rs` — `encode_to_bytes` (add the AVIF encode arm; without the
    feature, AVIF → a typed exit-4 error), `format_from_extension` /
    `extension_for_format` (recognize `avif`), `SinkError` (+ its exit-code mapping
    via `CliError::code()`).
  - `src/quality/mod.rs` — `encode_candidate_bytes` (add the AVIF arm behind the
    feature), `LossyFormat::supports_lossy_quality` (AVIF → true behind the
    feature), the `ImageFormat` match arms.
  - `src/cli/mod.rs` — `resolve_format` / `output_format_for` (so `--format avif`
    and an `.avif` extension resolve), `run_convert` (exit 4 up-front for an
    unbuilt codec — DEC-004), `CliError::code()` (the exit-4 mapping).
  - `Cargo.toml` — the `[features]` block (add `avif = ["image/avif"]`); the
    `image` dep line.
  - `deny.toml` — add the scoped `libfuzzer-sys` exception.
  - `.github/workflows/ci.yml` — the existing matrix + cargo-deny job (add a
    `--features avif` job).
  - `docs/api-contract.md` — the `convert` entry (AVIF now real with the feature).
  - `decisions/DEC-004-*.md` (codec policy this extends) + `DEC-018` (the gate) +
    the new `DEC-020`.
- **External APIs:** the `image` crate's `avif` feature → `ravif` (BSD-3, pure-Rust).
  `image::codecs::avif::AvifEncoder::new_with_speed_quality(w, speed: u8 /*1-10,
  1=slowest/best*/, quality: u8 /*1-100, 100=best*/)`, then
  `img.pixels().write_with_encoder(encoder)`. docs:
  https://docs.rs/image/0.25.10/image/codecs/avif/struct.AvifEncoder.html
- **Related code paths:** `Cargo.toml`/`deny.toml`/`ci.yml` + `src/sink` +
  `src/quality` + `src/cli`. Do NOT modify `src/image`, `src/operation`,
  `src/pipeline`, `src/recipe`, `src/source`.

## Outputs

- **Files modified:**
  - **`Cargo.toml`** — add `avif = ["image/avif"]` to `[features]` (off by default).
    No new TOP-LEVEL dep line (the codec comes via the existing `image` dep's
    feature) — but it IS a new transitive codec, so it carries **DEC-020** and a
    `just deny` run.
  - **`deny.toml`** — add a scoped exception to the `exceptions` list:
    `{ name = "libfuzzer-sys", allow = ["NCSA"] }` with a comment explaining it is
    a fuzz-only transitive of `rav1e` (via `ravif`/`image` avif), permissive
    (NCSA), and NOT present in the shipped `--features avif` binary (verified via
    `cargo tree -e normal`). Do NOT add `NCSA` to the global `allow` list.
  - **`src/sink/mod.rs`**:
    - `format_from_extension`: add `"avif" => Ok(ImageFormat::Avif)` (recognize the
      extension regardless of feature, so the format resolves and the error is a
      clear "codec not built", not "unsupported extension").
    - `encode_to_bytes`: add an AVIF branch. `#[cfg(feature = "avif")]` →
      `AvifEncoder::new_with_speed_quality(&mut cursor, AVIF_SPEED, quality.unwrap_or(AVIF_DEFAULT_QUALITY))`
      + `img.pixels().write_with_encoder(encoder)` (map errors to
      `SinkError::Encode`). `#[cfg(not(feature = "avif"))]` → return a NEW
      `SinkError::CodecNotBuilt { codec: "avif", feature: "avif" }` (a clear
      message: `avif support not built; rebuild with --features avif`).
    - NEW `SinkError::CodecNotBuilt { codec: &'static str, feature: &'static str }`
      (`#[error("{codec} support is not built; rebuild with --features {feature}")]`)
      — map it to **exit 4** in `CliError::code()` (alongside `UnsupportedExtension`
      / `UnknownFormat`).
    - `const AVIF_SPEED: u8 = 6;` and `const AVIF_DEFAULT_QUALITY: u8 = 80;` (tunable
      constants — see Notes; speed is fixed in v1, `--speed` is deferred).
  - **`src/quality/mod.rs`** — `encode_candidate_bytes`: add an
    `#[cfg(feature = "avif")] ImageFormat::Avif => { ... AvifEncoder ... }` arm
    (mirroring the sink's AVIF encode at the SAME `AVIF_SPEED`/quality — the
    cross-sync contract now covers AVIF too); `LossyFormat::supports_lossy_quality`:
    `#[cfg(feature = "avif")]` add `| ImageFormat::Avif` to the `matches!`. So
    auto-quality + `--max-size` drive AVIF quality with the feature on.
  - **`src/cli/mod.rs`** — confirm `resolve_format("avif")` → `ImageFormat::Avif`
    (via `format_from_extension`) so `--format avif` resolves; `run_convert`
    already resolves the format up front and the unbuilt-codec error surfaces as
    exit 4 (now via `CodecNotBuilt`). No new `CliError` variant (the `Sink` arm
    carries it); extend `exit_code_mapping_is_total` for `CodecNotBuilt → 4`.
  - **`.github/workflows/ci.yml`** — add a `avif` job (single ubuntu runner, like
    the cargo-deny job): `cargo build --features avif` + `cargo test --features avif`
    + `cargo clippy --all-targets --features avif -- -D warnings`. No `nasm` step
    needed (verified pure-Rust). Mirror the existing job style.
  - **`docs/api-contract.md`** — update the `convert` entry: with `--features avif`,
    `--format avif` produces AVIF (`-q`/`--target`/`--ssim`/`--max-size` all apply);
    without it, exit 4 (DEC-004). Note AVIF INPUT (decode) is not supported.
- **New decisions:** `DEC-020` (emitted in this design cycle) — adopt `ravif` via a
  feature-gated `avif`; keep AVIF feature-gated (revisit DEC-004); the
  `libfuzzer-sys`/NCSA deny exception; AVIF output-only v1; fixed speed + deferred
  `--speed`.
- **No new top-level dependency** (the codec is the `image` crate's `avif`
  feature). **No new `Operation`. No default-build behavior change.**

## Acceptance Criteria

Each maps to a test. Tests split into **default-build** (run always) and
**feature-build** (`#[cfg(feature = "avif")]`, run by the CI `avif` job).

- [ ] **Default build:** `convert <png> --format avif -o out.avif` → exit **4**
  (codec not built), stderr mentions `--features avif`. → `convert_avif_without_feature_exits_4`
- [ ] **Default build:** `exit_code_mapping_is_total` covers
  `SinkError::CodecNotBuilt → 4`. → (extend existing test)
- [ ] **Feature build:** `convert <png> --format avif -o out.avif` (`--features
  avif`) → exit 0; output is a valid AVIF (`image::guess_format` == `Avif`). →
  `convert_to_avif_produces_avif` (cfg avif)
- [ ] **Feature build:** `shrink <jpg> -o out.avif -q 50` → exit 0; output AVIF. →
  `shrink_to_avif_output` (cfg avif)
- [ ] **Feature build:** `convert <png> --format avif --target high` → exit 0;
  output AVIF (auto-quality drove AVIF). → `avif_target_high` (cfg avif)
- [ ] **Feature build:** `convert <png> --format avif --max-size 4KB` → exit 0;
  output AVIF ≤ 4KB (byte budget drove AVIF). → `avif_max_size_fits` (cfg avif)
- [ ] **Feature build:** unit — `quality::auto_under_size(img, ImageFormat::Avif, ..)`
  succeeds and `ImageFormat::Avif.supports_lossy_quality()` is `true`. → `avif_lossy_*` (cfg avif)
- [ ] **Both builds:** `just deny` is green (the scoped `libfuzzer-sys` exception). → CI
- [ ] **Both builds:** the default suite + all 5 gates stay green; no default-build
  behavior changes. → existing suites

## Failing Tests

Written during **design**. Default-build tests live in the normal modules/`tests/cli.rs`;
feature-build tests are `#[cfg(feature = "avif")]` (compiled + run only under
`cargo test --features avif`). Verifying AVIF output uses `image::guess_format`
(magic-byte detection — works WITHOUT the decoder).

- **`src/sink/mod.rs`** (UNIT):
  - `format_from_extension_recognizes_avif` — `format_from_extension(Path::new("x.avif"))
    == Ok(ImageFormat::Avif)`. Default build.
  - `#[cfg(feature = "avif")] encode_avif_respects_quality` — encode a detailed image
    to AVIF at quality 30 vs 90; assert both `guess_format == Avif` and the q30 byte
    length < q90 (quality knob works).
- **`src/quality/mod.rs`** (UNIT, `#[cfg(feature = "avif")]`):
  - `avif_supports_lossy_quality` — `ImageFormat::Avif.supports_lossy_quality()` is true.
  - `auto_under_size_avif_is_monotone` — `auto_under_size(img, ImageFormat::Avif,
    small)?.quality <= auto_under_size(img, ImageFormat::Avif, large)?.quality`.
- **`src/cli/mod.rs`** (UNIT): extend `exit_code_mapping_is_total` with
  `CliError::Sink(SinkError::CodecNotBuilt { codec: "avif", feature: "avif" }).code() == 4`.
- **`tests/cli.rs`** (INTEGRATION):
  - `convert_avif_without_feature_exits_4` — DEFAULT build: `convert <png> --format
    avif -o out.avif` → exit 4; stderr contains `avif` and `--features avif`. (This
    runs in the normal suite — the default binary has no avif feature.)
  - `#[cfg(feature = "avif")] convert_to_avif_produces_avif` — `convert <png>
    --format avif -o out.avif` → exit 0; `image::guess_format(&bytes) == Avif`.
  - `#[cfg(feature = "avif")] shrink_to_avif_output` — `shrink <jpg> -o out.avif
    -q 50` → exit 0; AVIF output.
  - `#[cfg(feature = "avif")] avif_target_high` — `convert <png> --format avif
    --target high -o out.avif` → exit 0; AVIF output (auto-quality on AVIF).
  - `#[cfg(feature = "avif")] avif_max_size_fits` — `convert <detailed png> --format
    avif --max-size 4KB -o out.avif` → exit 0; AVIF, `len <= 4000`.

## Implementation Context

### Decisions that apply
- **`DEC-020`** (emitted here) — adopt `ravif` via the off-by-default `avif`
  feature; keep AVIF feature-gated (compile-time/binary-size/encode-speed) even
  though it is pure-Rust; the `libfuzzer-sys`/NCSA scoped deny exception; AVIF
  **output-only** v1 (decode needs `avif-native`/dav1d, deferred); fixed encode
  speed (`AVIF_SPEED = 6`) with `--speed` deferred.
- `DEC-004` — pure-Rust codecs by default; native/heavy codecs behind off-by-default
  features, exit 4 when not built. This is the canonical case; `CodecNotBuilt → 4`.
- `DEC-018` — the license gate; the scoped `libfuzzer-sys` exception keeps it green
  without a blanket NCSA allow. Run `just deny` (the config is `all-features = true`).
- `DEC-019` — the auto-quality search; AVIF reuses it unchanged via the per-format
  `encode_candidate_bytes` arm + `supports_lossy_quality` (SPEC-017's generalization).
- `DEC-016` — `-q` → encoder quality; for AVIF, `-q` → AVIF quality (1-100).
- `DEC-015` — format precedence (`--format` > `-o` ext > preserve); `--format avif`
  / `.avif` ext now resolve to `ImageFormat::Avif`.

### Constraints that apply
- `pure-rust-codecs-default` — the default build stays pure-Rust + zero system deps;
  AVIF is opt-in and ALSO pure-Rust (no nasm), so even the feature build needs no
  system libs (a nice property; still gated for size/compile/speed).
- `no-agpl-default-deps` — `ravif`/`rav1e` are permissive (BSD); the only NCSA crate
  is fuzz-only and not shipped (scoped exception). `just deny` green.
- `no-new-top-level-deps-without-decision` — DEC-020 covers the codec; it arrives via
  the `image` feature, not a new top-level line.
- `single-image-library` — AVIF encode is the `image` crate's own encoder; no second
  pixel library.
- `ergonomic-defaults` — `convert photo.jpg --format avif` is one short command;
  sensible default quality/speed.
- `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`
  (note: clippy must pass BOTH default and `--features avif`),
  `untrusted-input-hardening` (quality clamped; the AVIF encoder bounds work).

### Prior related work
- `SPEC-017` (shipped, PR #20) — made the search format-agnostic
  (`encode_candidate_bytes`/`supports_lossy_quality`); THIS spec adds the AVIF arm
  the design was built for.
- `SPEC-014` (shipped) — `convert` + the exit-4-up-front for an unbuilt codec
  (DEC-004); `CodecNotBuilt` makes that error specific.
- `SPEC-005` (shipped) — the `Sink`/`encode_to_bytes` this extends.

### Out of scope (create a new spec rather than expand)
- **AVIF input / decode** (`convert in.avif --format png`) — needs `dav1d` (C, the
  `avif-native` feature); deferred. Reading an `.avif` fails as today.
- **A `--speed` knob** — v1 uses the fixed `AVIF_SPEED` constant; threading a
  per-invocation speed through `encode_to_bytes` + `encode_candidate_bytes` (so the
  search and the write agree) is a tracked fast-follow.
- WebP (SPEC-019) and the `--max-size` dimension fallback (separate backlog items).
- Animated AVIF; AVIF-specific metadata.

## Notes for the Implementer

- **The codec arrives via `image`'s feature, not a new crate line.** `Cargo.toml`:
  `avif = ["image/avif"]` under `[features]`. Then `image::codecs::avif::AvifEncoder`
  is available under `#[cfg(feature = "avif")]`. Confirm with `cargo build
  --features avif` (it builds pure-Rust — no nasm — verified in design).
- **`just deny` will FAIL until the exception is added** — it flags `libfuzzer-sys`
  (`(MIT OR Apache-2.0) AND NCSA`). Add the SCOPED exception
  `{ name = "libfuzzer-sys", allow = ["NCSA"] }` to `deny.toml`'s `exceptions`
  (NOT a global NCSA allow). It is a fuzz-only transitive of `rav1e` not in the
  shipped binary (`cargo tree -e normal --features avif` shows no libfuzzer-sys).
- **Exit 4 without the feature is the contract (DEC-004).** Recognize `avif` as a
  format (so it's not "unsupported extension"), but the `not(feature)` encode arm
  returns `CodecNotBuilt` → exit 4 with a helpful "rebuild with --features avif".
  `run_convert` resolves the format up front, so a multi-input convert to avif
  without the feature is a single exit 4 (not partial-batch) — same as today.
- **AVIF encode (pin this):** `let enc = ::image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut cursor, AVIF_SPEED, q); img.pixels().write_with_encoder(enc).map_err(|e| SinkError::Encode(e.to_string()))?;`
  where `q = quality.unwrap_or(AVIF_DEFAULT_QUALITY)` for the sink, and the search
  passes its candidate quality. Keep `encode_candidate_bytes`'s AVIF arm IDENTICAL
  (same speed) to the sink — the byte-budget/perceptual guarantee depends on the
  probe matching the written bytes (the cross-sync contract, now covering AVIF).
- **Verifying AVIF in tests WITHOUT a decoder:** use `image::guess_format(&bytes)`
  (magic-byte `ftyp`/`avif` detection) — it does NOT need `avif-native`. Do NOT try
  to `image::load_from_memory` an AVIF (decode isn't built). For the detailed
  fixtures reuse `common::detailed_png`/`detailed_jpeg`.
- **Defaults rationale:** `AVIF_SPEED = 6` (a balanced rav1e speed — encode stays
  reasonably fast; 1 would be very slow); `AVIF_DEFAULT_QUALITY = 80`. Both tunable;
  `--speed` is the deferred knob. Note AVIF quality numbers are NOT comparable to
  JPEG's (AVIF q80 ≠ JPEG q80 visually) — that's expected; the perceptual/byte
  targets are the format-independent way to ask for an outcome.
- **clippy/test must pass BOTH builds.** The CI `avif` job runs them with
  `--features avif`; locally run `cargo clippy --all-targets --features avif -- -D
  warnings` and `cargo test --features avif` in addition to the default gates.
  Speed: `AVIF_SPEED = 6` keeps test encodes fast; if a test is slow, use small
  fixtures (≤ 160px).
- **Commit incrementally** (feature + deny green → sink encode + CodecNotBuilt →
  quality arm + supports_lossy_quality → CLI/exit-4 test → CI job).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-020` — adopt ravif (feature-gated AVIF) [authored in design]
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
