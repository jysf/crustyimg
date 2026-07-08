# Upstream contribution — pure-Rust AVIF decode in `image-rs` (issue #2621)

> A tracked **ecosystem contribution** (Track B / Reach & adoption). Not a crustyimg `src/` change —
> the work lands in the `image-rs/image` repo. It runs **in parallel** to PROJ-009's in-tree AVIF
> decode and does not block it. Status: **proposed** (not started). Snapshot: 2026-07-07.

## Why we're doing this

Two payoffs, both on-brand for crustyimg ("assemble the pure-Rust imaging frontier"):

1. **Ecosystem goodwill + adoption.** Landing pure-Rust AVIF decode in `image` benefits every
   Rust image tool, not just us — exactly the kind of upstream citizenship that builds the
   community crustyimg lives in (Track B, `docs/roadmap.md`).
2. **A future migration for crustyimg.** PROJ-009 ships AVIF decode in-tree via `re_rav1d`
   (BSD-2) + `avif-parse` (MPL-2.0) + glue. That's the right call to ship now, but `re_rav1d` is
   rerun's self-described "messy" fork with an uncertain maintenance future. If `image` gains a
   pure-Rust AVIF decode path, crustyimg can later **drop its direct `re_rav1d` dep + hand-written
   glue** and decode AVIF through the `image` crate it already depends on. See
   `guidance/license-watchlist.yaml` → `avif-decode`.

## The opportunity (why it's cheap)

- **[image-rs/image#2621](https://github.com/image-rs/image/issues/2621)** — "enable re_rav1d by
  default for avif decoding" — is **open, unassigned, no linked PR, no timeline**. The blocker is
  **maintainer prioritization, not difficulty.**
- The issue author attached a **working proof-of-concept**: add `re_rav1d` as an optional dep,
  feature-gate it, and swap the decoder import — trivial because `re_rav1d`'s API is a `dav1d-rs`
  fork, so it's largely a drop-in for the existing (C) `dav1d` path.
- Today `image` decodes AVIF only via the `avif-native` feature → **system/native dav1d (C)**
  (latest `image` is 0.25.x; pure-Rust decode has **not** landed).

## Approach (proposed — coordinate before coding)

1. **Talk to the maintainers first.** Comment on #2621, confirm they'd accept a pure-Rust decode
   backend and how they want it gated (e.g. an `avif` / `avif-rust` feature vs the existing
   `avif-native`), before opening a PR. The blocker is prioritization — a well-scoped offer helps.
2. **Wire `re_rav1d` behind a feature** mirroring the PoC: optional dep, feature flag, decoder
   dispatch. Keep the existing `avif-native` (dav1d/C) path for those who want asm speed.
3. **Build the no-asm path by default** for the pure-Rust feature (zero build-tool deps).
4. **Container:** confirm whether `image` wants `avif-parse` (MPL-2.0) for the ISOBMFF/MIAF layer
   or its existing container handling; align licenses (MPL is file-level copyleft, fine).
5. **Tests:** real-file corpus (libavif samples, alpha, 10-bit, wide-gamut), and a decision on
   grid/tiled AVIF (support vs reject-cleanly).

**Higher-leverage variant (optional):** help upstream a **native Rust API into `rav1d`** itself
(`memorysafety/rav1d`) — rerun has signalled they're unsure whether to keep maintaining their fork.
A first-class rav1d Rust API de-risks the whole ecosystem's dependency (and ours) more than the
`image` wiring alone. Bigger effort; gauge appetite before committing.

## Scope / non-goals

- **In:** an upstream `image-rs/image` PR (+ discussion) enabling pure-Rust AVIF decode; optionally
  a `rav1d` Rust-API contribution.
- **Out:** any crustyimg `src/` change (that's PROJ-009). The crustyimg-side **migration** off
  `re_rav1d` to `image`'s built-in decode is a *future* PROJ-009 follow-up, gated on this landing in
  a released `image` version.

## Status & tracking

- **Status:** proposed / not started.
- **Blocks nothing** (PROJ-009 ships in-tree regardless).
- **Done when:** pure-Rust AVIF decode is merged + released in `image`; then file the crustyimg
  migration follow-up.
- **Risks:** maintainer acceptance + timeline are outside our control; `re_rav1d` fork maintenance;
  coordinate rather than drop an unsolicited PR.

## Pointers
- `guidance/license-watchlist.yaml` → `avif-decode` (the full decoder landscape + why re_rav1d).
- `projects/PROJ-009-input-reach/` (the in-tree AVIF decode this de-risks).
- `docs/roadmap.md` Track B (this is an ecosystem/adoption contribution).
