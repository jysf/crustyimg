---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-108
  type: decision
  confidence: 0.75
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-012
repo:
  id: crustyimg

created_at: 2026-10-30
supersedes: null
superseded_by: null

# The muxer will live beside the still-AVIF writer, and the dependency line is
# what `no-new-top-level-deps-without-decision` watches.
affected_scope:
  - "Cargo.toml"
  - "src/image/avif*.rs"
  - "src/image/animated*"

tags:
  - dependency
  - animated-output
  - avif
  - iso-bmff
---

# DEC-108: Adopt `mp4-atom` as the ISO-BMFF box writer for animated AVIF, on conditions the design session verifies

## Decision

**Animated AVIF output writes its ISO-BMFF boxes with `mp4-atom`** (MIT OR Apache-2.0,
`default-features = false`), behind the existing `avif` feature. crustyimg writes the muxing
driver on top: the sample table, timing and loop count. Adoption is **conditional**. The first
design session in PROJ-012's animated-output stage checks the four conditions under
Validation before any code depends on the crate. If any fails, this decision falls back to an
in-house box writer (Alternative B) and is amended to say so. It doesn't stay silently
half-true.

Maintainer ruling, 2026-10-30, in a planning session. It resolves `mp4-atom-adoption` in
`guidance/questions.yaml`.

## Context

`format/animated-gif` and `format/animated-input` flag an animation crustyimg cannot
re-encode, because it has no animated output format (PROJ-012's thesis). Animated AVIF is the
measured answer: a 308,156 B / 36-frame GIF went to 27,564 B at SSIMULACRA2 86.7 (11.2×), and
the best animated WebP point was 172,492 B at 84.1, so AVIF is 6.3× smaller at higher quality.
No pure-Rust animated WebP encoder exists, which is why the target is AVIF. `rav1e` (encode)
and `re_rav1d` (decode, the test oracle) are already in the tree.

An AVIF image sequence is an ISO-BMFF file (`ftyp` with the `avis` brand, `moov`/`trak`
with an `av01` sample entry and `av1C` config, a sample table, `mdat`). That container is
the missing piece. The question has been open since 2026-08-16 and blocked PROJ-012 from
being specced at all.

**What was verified, and when:**
- *2026-08-16, source-level, `mp4-atom` 0.15.0:* both LICENSE files present and read; 23,631
  source lines; pure Rust, no `-sys` crate; ships the `av01` sample entry and `av1C` box
  (`src/moov/trak/mdia/minf/stbl/stsd/av01.rs`) and the full sample table, with a committed
  test decoding a real libavif animated AVIF (`avis`).
- *2026-10-30, crates.io metadata only, `mp4-atom` 0.16.2 (2026-10-06):* licence field
  `MIT OR Apache-2.0`; 617,696 downloads (403,426 recent); repository
  `github.com/kixelated/mp4-atom`. Normal dependencies: `derive_more` 2, `num` 0.4, `pastey`
  0.2, `thiserror` 1, `tracing` 0.1; optional `bytes`, `serde`, `tokio`. ⚠ **The 0.16 source
  was not re-read.** Three minor releases landed since the August read.
- *Against this repo's `Cargo.lock` (2026-10-30):* it would add `derive_more`, `num`
  and `tracing` new, a second `thiserror` (1.x beside our 2.0.18), and a second `pastey`
  (0.2 beside our 0.1.1).

It gives boxes, not a muxer. Its README says it does the binary format "without validation
or interpretation", so the measured price of the muxing driver on top is about 1,000 lines
either way.

## Alternatives Considered

- **Option A: `mp4` (the older crate) or another muxer crate**
  - What it is: a higher-level MP4 writer.
  - Why rejected: the August probe priced a driver against `mp4` 0.14's writer and track code.
    Neither saved the bookkeeping, and `mp4-atom` is the one with `av01`/`av1C` and an `avis`
    test.

- **Option B: an in-house ISO-BMFF box writer**
  - What it is: write `ftyp`, `moov`/`trak`/`mdia`/`minf`/`stbl` (`stsd` with `av01`+`av1C`,
    `stts`, `stsc`, `stsz`, `stco`), `mdat`, and the AVIF still-image boxes, by hand. It's
    single-track, video-only, and write-only.
  - Why not chosen now: the repo's precedent prefers in-house small formats over **low-trust**
    crates (`little_exif` → `tiff.rs`, 718 lines). `mp4-atom` is not low-trust: it's
    permissive, widely used, and maintained. The box layer is not the hard part, and a
    write-only single-track subset is plausibly a few hundred lines. **This is the fallback**
    if a condition below fails, and it should be priced in lines at design time either way so
    the comparison is real.

- **Option C (chosen): `mp4-atom` with `default-features = false`, plus an in-house driver**
  - What it is: boxes from the crate; sample table, timing, loop count and frame packing are
    ours.
  - Why selected: it's the only probed crate with the AV1 sample entry and a real `avis` test,
    it carries a permissive licence, and its box types make the driver's output inspectable in
    tests.

## Consequences

- **Positive:** PROJ-012's animated-output stage can be specced. `lint`'s animated rules can
  name a fix the tool performs.
- **Negative:** three new crates in the default build (it sits behind `avif`, which is
  default-on per DEC-081), plus two duplicated majors (`thiserror`, `pastey`) until upstream
  moves. Binary growth is unmeasured.
- **Neutral:** no change to still-AVIF output, so no lockfile migration. `re_rav1d` remains
  the independent decoder for tests; an encoder's packet count is not evidence of frames.

## Validation

The first design session in PROJ-012's animated-output stage checks all four **before** any
spec depends on the crate, and records the results here as an amendment:

1. **Licence, from the files.** Read both LICENSE files in the 0.16.x tarball, not the
   metadata field (DEC-018). `just deny` passes with the crate added.
2. **The AV1 boxes still exist in 0.16.x.** `av01` and `av1C` are present, and the `avis`
   decode test still passes on the version we would pin.
3. **The dependency cost is stated.** With `cargo tree -i --all-features` and the build
   fingerprint, not `cargo tree -e features` alone (a disabled feature can swap a module),
   record every crate added and the stripped release-binary delta. If the delta is large
   enough to matter, price Option B in lines and compare.
4. **It builds at our MSRV** (1.90.0) and on the wasm32 leg, or animated output is declared
   native-only in the spec.

If any fails: amend this record to Option B, with the failing condition named.

## References

- `guidance/questions.yaml` → `mp4-atom-adoption` (resolved by this record)
- `docs/research/draft-spec-animated-avif-output.md`: measured numbers, the colour-range
  trap, the speed-10 trap
- `projects/PROJ-012-lint-can-fix-what-it-names/brief.md` and its animated-output stage
- DEC-018 (permissive licence policy), DEC-081 (AVIF encode default-on), DEC-088 (workhorse
  vs lab)
