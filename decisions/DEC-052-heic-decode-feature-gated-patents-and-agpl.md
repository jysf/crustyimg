---
insight:
  id: DEC-052
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-004
repo:
  id: crustyimg

created_at: 2026-07-07
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - src/image/**
  - src/source/**

tags:
  - codecs
  - dependencies
  - licensing
  - patents
  - heic
---

# DEC-052: HEIC/HEVC decode is feature-gated — never in the default binary (AGPL wall + HEVC patents)

## Decision

`.heic`/`.heif` **decode is gated behind an off-by-default `heic` cargo feature** and MUST
NOT appear in any distributed default binary (cargo-dist releases, Homebrew bottle, the
`cargo install` default). The `heic` feature uses the **libheif C path** (`libheif-rs` →
system/ dynamically-linked libheif, built decode-only via `WITH_LIBDE265=ON WITH_X265=OFF`)
where a system HEVC codec is present. This extends DEC-004 (pure-Rust default; native codecs
behind features) to HEIC specifically, and pins the rationale to **two independent blockers**,
either of which alone forces the gate:

1. **Copyright/license (the AGPL wall).** The mature pure-Rust HEIC decoders are AGPL —
   `imazen/heic` (AGPL-3.0 OR commercial) and `ente/heic_decoder` (AGPL-3.0). AGPL would
   relicense our statically-linked binary (constraint `no-agpl-default-deps`, DEC-018). They
   cannot be a default dep, and even as an opt-in they can't ship in a distributed artifact
   without AGPL-ing it.
2. **Patents (independent of the software license).** HEVC/H.265 is covered by the Access
   Advance patent pool. A copyright license — AGPL, commercial, MIT, *or* Apache — grants
   **zero** patent rights. This exposure attaches to **every** decode path equally: the
   libheif C feature, `imazen/heic`, and any future permissive pure-Rust decoder. Peers
   converge on the same posture: Firefox ships no software HEVC decoder (OS/hardware only),
   ffmpeg pushes the risk downstream (FFmpegKit retired Jan 2025 over exactly this), Fedora
   strips H.265, and ImageMagick/libvips/`image-rs` treat HEIC as an optional off-by-default
   component. Our release channels (Homebrew + GitHub Releases) are US-hosted, without the
   France-hosted-repo mitigation Fedora's ecosystem relies on.

**Key consequence:** "once a mature permissive pure-Rust HEVC decoder appears we can ship HEIC
in the default binary" is **false**. The patent blocker persists after the license blocker is
solved, so HEIC decode stays feature-gated until the patent question is *separately* resolved
(counsel, not code). Decode-only (never an HEVC *encoder* — that adds x265/GPL and encoder-side
patent tiers).

## Context

Framing the input-reach wave (AVIF/SVG/RAW/HEIC). A first-pass spike concluded "no permissive
pure-Rust HEVC decoder exists," which a second-opinion review corrected: permissive candidates
*do* exist but are immature and HEIC-unproven (`rust_h265` 0.1.0, `media-codec-h265` 0.1.1 —
both MIT OR Apache-2.0, tracked in `guidance/license-watchlist.yaml`). That correction does not
change this decision, because the patent blocker is orthogonal to the license and independently
forces the gate. Full evidence: `docs/research/heic-input-reach-spike.md`.

The default build keeps its "no system dependencies" promise and leads modern-format input with
**AVIF decode** (patent-*free* via the AV1 royalty-free grant; rav1d, BSD-2-Clause), **SVG**
(resvg, MIT), and **RAW embedded-preview** extraction (permissive, no codec). HEIC is the
honest opt-in.

## Alternatives Considered

- **Option A: HEIC decode on by default (pure-Rust once a decoder matures).**
  - Why rejected: the patent exposure remains regardless of a permissive code license; every
    peer gates it out of the default. Also no permissive decoder is HEIC-proven yet.
- **Option B: No HEIC support at all; tell users to pre-convert (`sips`/`heif-convert`).**
  - Why rejected: a meaningful slice of users have raw `.heic` on disk (AirDrop, export-original);
    a flat "can't read HEIC" is a visible gap. Invoking an external converter stays valid as a
    documented fallback, but an opt-in in-tool path is worth offering.
- **Option C (chosen): feature-gated `heic` (libheif decode-only), never default; permissive
  pure-Rust decoders tracked for a future opt-in but still patent-gated.**
  - Why selected: keeps the default binary clean, portable, and legally conservative; gives
    Mac/Linux users who accept the terms an in-tool path; matches the whole-industry posture.

## Consequences

- **Positive:** default binary stays pure-Rust, zero-system-dep, and free of both the AGPL and
  the HEVC-patent exposure. Honest positioning; no rug-pull risk.
- **Negative:** HEIC "just works" is not the default-CLI story; `heic` needs system libheif
  (the "is it installed" problem). `convert x.heic ...` without the feature exits 4 ("codec not
  built"), consistent with DEC-004's AVIF behavior.
- **Neutral:** the native apps (macOS/iOS) decode HEIC via the OS (ImageIO/PhotoKit,
  `objc2-image-io` — permissive), which sidesteps *both* blockers (Apple holds the platform
  patent license) — a separate path from the CLI, recorded in the private `business/` notes.

## Validation

Right if: default `cargo build`/release artifacts contain no HEVC decoder and no libheif, and
`--features heic` builds against a decode-only system libheif where present. Revisit only when
**all** hold: (a) a permissive decoder's license is confirmed [done: `rust_h265`,
`media-codec-h265` are MIT/Apache], (b) it is proven on a real iPhone grid/tiled/HDR corpus,
(c) performance is acceptable — **and** the HEVC patent question is separately cleared with
counsel. License-only maturity is not sufficient to un-gate.

## References

- Related decisions: DEC-004 (pure-Rust default + feature-gated native), DEC-018 (`no-agpl-default-deps`)
- Related constraints: `no-agpl-default-deps`, `pure-rust-codecs-default`, `untrusted-input-hardening`
- Watchlist: `guidance/license-watchlist.yaml` → `heic-heif-decode` (permissive candidates + triggers)
- Evidence: `docs/research/heic-input-reach-spike.md`
- libheif licensing: LGPL-3.0-or-later; decode-only = libde265 (LGPL), exclude x265 (GPL). Dynamic
  link / system lib is the clean path; static linking carries §4 relink obligations.
