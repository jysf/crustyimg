# Licensing

`crustyimg` is dual-licensed under **MIT OR Apache-2.0**. Every dependency on the
default path carries a permissive license, mechanically enforced by
`cargo deny check licenses` (`just deny`, constraint `no-agpl-default-deps`,
DEC-018/DEC-037). The default binary is therefore freely embeddable, including in
closed-source and commercial products.

The one thing that policy does **not** cover lives here: the patent question, which
is orthogonal to every software license. (The `heic` feature used to link a copyleft
*system* library too; it now decodes through a permissive, pure-Rust crate — see
below.)

## The `heic` feature and the pure-Rust `heifer` decoder

`--features heic` (SPEC-062, DEC-052) adds HEIC/HEIF **decode** through
[`heifer`](https://crates.io/crates/heifer), a pure-Rust decoder licensed
**MIT OR Apache-2.0** and written with `#![forbid(unsafe_code)]`. It carries no C
dependency: no system library, no `pkg-config`/`vcpkg`, no bindgen, and it builds
for `wasm32-unknown-unknown` and `musl`. `cargo deny` stays green with no new
exception, and there is no copyleft obligation to record.

This replaces the earlier `libheif-rs` → `libheif-sys` → system libheif C library
path (DEC-056). That path pulled in an **LGPL-3.0-or-later** C library, whose
relink/attribution terms applied to any redistributed `--features heic` build. With
`heifer` those terms are gone: the HEIC decode dependency is permissive end to end,
and `cargo tree | grep libheif` is now empty.

The feature is still **never enabled in a distributed artifact** — not in cargo-dist
release builds, not in the Homebrew formula, not in `cargo install` defaults — but now
for the patent reason below alone, not for a license reason. It exists for users who
build locally and accept the terms below.

## HEVC patents (independent of any software license)

HEVC/H.265 is covered by the Access Advance patent pool. **A copyright license —
MIT, Apache, LGPL, AGPL, or commercial — grants no patent rights.** That exposure
attaches to *every* HEIC decode path equally, so even a permissive pure-Rust HEVC
decoder does **not** make HEIC shippable by default; resolving it is a legal
question, not an engineering one (DEC-052).

This matches the wider ecosystem: Firefox ships no software HEVC decoder, Fedora
strips H.265, and ImageMagick/libvips/`image-rs` all treat HEIC as an optional,
off-by-default component.

Enabling `--features heic` is your decision, in your jurisdiction, for your use.

## Related

- `decisions/DEC-052` — why HEIC is feature-gated (HEVC patents; the AGPL-decoder
  blocker was resolved by the permissive, pure-Rust `heifer` decoder).
- `decisions/DEC-056` — the previous `libheif-rs` system-link and CI approach, now
  superseded by the pure-Rust `heifer` dependency.
- `decisions/DEC-018` — no AGPL default dependencies.
- `deny.toml` — the enforced license allow-list and its per-crate exceptions.
- `guidance/license-watchlist.yaml` — capabilities declined for license reasons.
