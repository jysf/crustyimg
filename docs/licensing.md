# Licensing

`crustyimg` is dual-licensed under **MIT OR Apache-2.0**. Every dependency on the
default path carries a permissive license, mechanically enforced by
`cargo deny check licenses` (`just deny`, constraint `no-agpl-default-deps`,
DEC-018/DEC-037). The default binary is therefore freely embeddable, including in
closed-source and commercial products.

Two things that policy does **not** cover live here: an off-by-default feature that
links a copyleft *system* library, and the patent question that is orthogonal to
every software license.

## The `heic` feature and system libheif (LGPL)

`--features heic` (SPEC-062, DEC-052, DEC-056) adds HEIC/HEIF **decode** by linking
the system **libheif** C library. The Rust crates in that path — `libheif-rs` and
`libheif-sys` — are **MIT**, which is why `cargo deny` stays green with no new
exception: cargo-deny sees crates, and libheif itself is not one. The obligation is
real regardless, so it is recorded here rather than in `deny.toml`.

**libheif is LGPL-3.0-or-later.** If you *redistribute* a `--features heic` build,
the LGPL applies to libheif, not to crustyimg:

- **Attribution.** Ship libheif's copyright notice and a copy of the LGPL-3.0 text,
  and state that libheif is used and may be modified/replaced.
- **Relinking.** Dynamic/system linking — what this feature does, via `pkg-config`
  — satisfies the LGPL's relink requirement by construction: the user can swap
  their libheif. `libheif-rs`'s `embedded-libheif` feature would *statically* vendor
  it and pull in LGPLv3 §4's relink obligation, so **we do not use it** (DEC-056).
- **Decode only.** No HEVC *encoder* is built (that would add x265, which is GPL).

The feature is **never enabled in a distributed artifact** — not in cargo-dist
release builds, not in the Homebrew formula, not in `cargo install` defaults — so
none of this touches the binaries this project publishes. It exists for users who
build locally and accept the terms below.

## HEVC patents (independent of any software license)

HEVC/H.265 is covered by the Access Advance patent pool. **A copyright license —
MIT, Apache, LGPL, AGPL, or commercial — grants no patent rights.** That exposure
attaches to *every* HEIC decode path equally, so a future permissive pure-Rust HEVC
decoder would **not** make HEIC shippable by default; resolving it is a legal
question, not an engineering one (DEC-052).

This matches the wider ecosystem: Firefox ships no software HEVC decoder, Fedora
strips H.265, and ImageMagick/libvips/`image-rs` all treat HEIC as an optional,
off-by-default component.

Enabling `--features heic` is your decision, in your jurisdiction, for your use.

## Related

- `decisions/DEC-052` — why HEIC is feature-gated (AGPL wall + HEVC patents).
- `decisions/DEC-056` — the `libheif-rs` dependency, system-link, and CI approach.
- `decisions/DEC-018` — no AGPL default dependencies.
- `deny.toml` — the enforced license allow-list and its per-crate exceptions.
- `guidance/license-watchlist.yaml` — capabilities declined for license reasons.
