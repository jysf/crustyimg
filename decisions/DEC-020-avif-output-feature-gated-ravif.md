---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-020
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-16
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - deny.toml
  - .github/workflows/ci.yml
  - src/sink/**
  - src/quality/**

tags:
  - avif
  - codec
  - ravif
  - feature-gate
  - license
  - modern-formats
---

# DEC-020: AVIF output via a feature-gated `ravif` codec (off by default)

## Decision

crustyimg gains **AVIF output** behind an **off-by-default `avif` cargo feature**
(`avif = ["image/avif"]`), which pulls the `image` crate's pure-Rust AVIF encoder
**`ravif`** (→ `rav1e`). The **default build is unchanged** — pure-Rust core
formats only, AVIF output **exits 4** ("codec not built", DEC-004). A `--features
avif` build encodes AVIF in `convert`/`shrink` (and via `-o x.avif`), with `-q`
→ AVIF quality and — thanks to the SPEC-017 format-agnostic search — the
**`--max-size` byte budget (SPEC-017)** driving the AVIF quality knob for free.

> **Build-time correction (SPEC-018).** The original draft claimed the
> *perceptual* `--target`/`--ssim` search (SPEC-016) would also drive AVIF for
> free. It does **not**: that search must DECODE each candidate to score the
> round-trip with SSIMULACRA2, and AVIF **decode is not built** (output-only v1 —
> point 3 below). Only the **byte-budget** search (`--max-size`, encode-only)
> works on AVIF today. A perceptual target on an AVIF output degrades gracefully
> (writes AVIF at the encoder default + a warning). Perceptual AVIF defers WITH
> AVIF decode. The quality seam is therefore split into two predicates:
> `supports_lossy_quality` (byte budget) and `supports_perceptual_quality`
> (perceptual).

Specifics:
1. **Feature-gated even though it is pure-Rust.** `ravif`/`rav1e` build with **no
   `nasm` and no system libraries** (verified empirically in design — a clean build
   with nasm absent), so AVIF does NOT compromise the "zero system deps" promise.
   It stays gated anyway for **compile time, binary size (~18 MB / ~431k SLoC), and
   encode speed** (AV1 is slow at high quality). This **revisits DEC-004's**
   AVIF-gating rationale — the original reason was "native dep"; the real reason is
   now build cost — and the verdict is **KEEP it feature-gated**.
2. **License gate stays green via a SCOPED exception.** The shipped `--features
   avif` tree is fully permissive (`ravif` BSD-3, `rav1e` BSD-2/MIT, …). cargo-deny's
   `all-features = true` evaluation flags exactly one crate — **`libfuzzer-sys`
   (`(MIT OR Apache-2.0) AND NCSA`)** — a **fuzz-only** transitive of `rav1e` that is
   **NOT in the real build** (`cargo tree -e normal --features avif` shows no
   libfuzzer-sys). NCSA is a permissive (OSI/FSF-approved, BSD-style) license. We add
   a **scoped per-crate exception** `{ name = "libfuzzer-sys", allow = ["NCSA"] }`
   in `deny.toml` (mirroring the `ansi_colours` LGPL exception, DEC-018) rather than
   a blanket `NCSA` allow — keep the surface minimal.
3. **AVIF OUTPUT only (v1).** Decoding an `.avif` INPUT needs `dav1d` (C, the
   `image` `avif-native` feature) — a system dep that contradicts the pure-Rust
   promise — so AVIF **decode is deferred**. Reading an `.avif` fails as today;
   `convert in.jpg --format avif` works (with the feature).
4. **Fixed encode speed in v1.** AVIF is encoded at a balanced fixed `rav1e` speed
   (`AVIF_SPEED = 6`); `-q` controls quality (1–100), default 80. A per-invocation
   **`--speed` knob is deferred** (it would need threading through both the sink
   encode and the search's `encode_candidate_bytes` so the probed and written bytes
   agree).

## Context

STAGE-008 is "Modern Formats & Quality." SPEC-016/017 shipped the perceptual-target
and byte-budget quality core, and SPEC-017 deliberately made the auto-quality search
**format-agnostic** (`LossyFormat::supports_lossy_quality` + a per-format
`encode_candidate_bytes` arm). AVIF is the first modern OUTPUT format to plug into
that seam: it is typically **30–50% smaller than JPEG at equal quality**, and once
its encode arm exists, the **byte-budget** search works on it with no search changes
(the encode-only probe). The perceptual search needs a decoder it does not have —
see the build-time correction above.

The codec choice was constrained by DEC-018 (no AGPL/GPL defaults) and DEC-004
(pure-Rust default, native codecs feature-gated). The research/verification
(2026-06-15/16, empirically confirmed this design) found `ravif` is the right pick:
pure-Rust, BSD-3, mature for still images, and — importantly — it builds without
nasm. The handoff pre-scoped this as "SPEC-018 → adopt ravif; keep feature-gated."

## Alternatives Considered

- **AVIF on by default.**
  - Why rejected: `rav1e` is ~18 MB / ~431k SLoC and slow to compile + slow to
    encode at high quality. It would bloat the default binary and CI compile, and
    slow `convert`/`shrink` for users who never touch AVIF. Gating keeps the common
    path lean; AVIF is opt-in for those who want it.

- **A C AVIF library (`libavif`/`avif-native`/`dav1d`) for encode and/or decode.**
  - Why rejected for the default: a system dependency contradicts "zero system deps."
    `ravif` gives pure-Rust ENCODE with no nasm, which is strictly better for our
    promise. (Decode would need `dav1d` (C) — hence decode is deferred, not done via
    a system lib.)

- **Add `NCSA` to the global `deny.toml` allow list.**
  - Why rejected: too broad. The NCSA crate (`libfuzzer-sys`) is fuzz-only and not
    even shipped; a scoped per-crate exception expresses exactly that and keeps the
    allowlist tight (the same discipline as the `ansi_colours` exception).

- **Defer AVIF until a `--speed` knob and AVIF decode are also done.**
  - Why rejected: AVIF OUTPUT with a sensible fixed speed is the high-value 80%;
    `--speed` and decode are independent fast-follows. Shipping output now unblocks
    the modern-format story and the benchmark "N% smaller at equal quality" claim.

## Consequences

- **Positive:** the first modern output format; AVIF is dramatically smaller than
  JPEG at equal quality, and the perceptual/byte-budget targeting works on it for
  free (SPEC-017's payoff). The default build is untouched (lean, pure-Rust, exit 4
  for AVIF). The feature build is ALSO pure-Rust (no nasm) — a strong story. The
  license gate stays honest via a tight scoped exception.
- **Negative:** a `--features avif` build is much slower to compile and the binary
  is large; AVIF encode is slow at high quality/low speed. AVIF INPUT (decode) is
  not supported (output-only). AVIF quality numbers are not comparable to JPEG's
  (use the perceptual/byte targets to ask for an outcome, not a `-q` number).
- **Neutral:** a new `SinkError::CodecNotBuilt` variant makes the unbuilt-codec exit
  4 specific ("rebuild with --features avif") instead of a generic "unsupported
  extension." A new CI `avif` job builds/tests the feature (single ubuntu runner,
  no nasm).

## Validation

Right if: the default build is byte-unchanged and `convert --format avif` exits 4
with a "--features avif" hint; a `--features avif` build produces valid AVIF
(`image::guess_format == Avif`) for `convert`/`shrink`, honors `-q`, lets
`--max-size` drive AVIF (and a `--target`/`--ssim` perceptual request on AVIF
falls back to the encoder default with a warning, since decode is deferred);
`cargo deny check licenses` is green
with only the scoped `libfuzzer-sys` exception; the `--features avif` build needs no
nasm/system libs. Revisit if: a `--speed` knob is wanted (thread speed through the
encode + the search probe); AVIF decode is needed (then weigh `dav1d`/`avif-native`
— a system dep — behind its own feature); rav1e's tree changes its license posture;
or AVIF encode speed becomes a UX problem (raise the default `AVIF_SPEED`).

## References
- Related specs: SPEC-018 (this — AVIF output), SPEC-017 (the format-agnostic search
  AVIF plugs into), SPEC-014 (`convert` + exit-4-up-front for an unbuilt codec).
- Related decisions: DEC-004 (codec policy / feature-gating — this revisits its AVIF
  rationale), DEC-018 (the license gate + the scoped-exception pattern), DEC-019 (the
  auto-quality search reused unchanged), DEC-016 (`-q` → encoder quality), DEC-015
  (format precedence).
- Related constraints: `pure-rust-codecs-default`, `no-agpl-default-deps`,
  `no-new-top-level-deps-without-decision`, `single-image-library`.
- External docs: https://docs.rs/ravif , https://docs.rs/image/0.25.10/image/codecs/avif/struct.AvifEncoder.html ,
  NCSA license: https://spdx.org/licenses/NCSA.html
