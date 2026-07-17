---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-046
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-07-04
supersedes: null
superseded_by: null

affected_scope:
  - src/metadata/mod.rs
  - Cargo.toml
  - deny.toml

tags:
  - architecture
  - metadata
  - exif
  - dependencies
  - security
  - advisory
  - supply-chain
---

# DEC-046: In-house TIFF-IFD EXIF writer — retire `little_exif` (drop quick-xml)

## Decision

Replace **`little_exif`** — the tag-level EXIF writer behind `set` and `clean --gps`
(DEC-029) — with a **small in-house binary TIFF-IFD reader+writer** in `src/metadata/`.
The new writer operates on the **raw TIFF/EXIF block** that `img-parts` already exposes
(`ImageEXIF::exif()` / `set_exif()` — verified to be the bare TIFF starting at `II*\0` /
`MM\0*`, no `Exif\0\0` wrapper), so no XMP/XML is ever parsed. This drops `little_exif`
and, with it, **`quick-xml`** (`RUSTSEC-2026-0194`/`-0195`) and **`brotli`** from the
dependency tree, letting us delete the `-0194`/`-0195` `deny.toml` ignores.

`img-parts` stays (segment/chunk plumbing + `strip_all`/`copy_metadata`, unchanged).
`kamadak-exif` stays (the read side; also used in tests to assert round-trips). This
**amends DEC-029** (which chose `little_exif` for the write half); the read side and the
container-lane architecture (DEC-003) are unchanged.

## Context

crustyimg ships v0.1.x with three accepted `deny.toml` advisory ignores (DEC-042).
STAGE-010 eliminates them at the source. SPEC-044 removed `ttf-parser` (`-0192`). This
decision targets the two **real** (if unreached) vulnerabilities — `quick-xml`'s
quadratic-attr / NsReader memory-DoS (`-0194`/`-0195`) — which enter **only** via
`little_exif` (`cargo tree -i quick-xml` confirms the single path). `little_exif` uses
`quick-xml` for the XMP/XML path; crustyimg drives it for **binary EXIF only** and has
zero XMP in `src`, so the vuln was never reachable — but removing the dependency is
cleaner than carrying the ignore.

**No upstream shortcut (checked, per the SPEC-044 fontdue lesson):** `little_exif 0.6.23`
is the **latest** release and still pins `quick-xml ^0.37.5` (vulnerable) **and**
`paste ^1.0.15`. There is no version to bump to; the DEC-042 revisit trigger ("little_exif
bumps quick-xml ≥0.41 or is replaced") resolves to **replace**. No drop-in exists:
`kamadak-exif`/`nom-exif` are read-only; `img-parts` is segment-level only. So an in-house
tag-level writer is the path.

**`paste` (`RUSTSEC-2024-0436`) is NOT removed by this** — and the backlog claim that it
would be is corrected here. `paste` reaches the graph via **both** `little_exif` **and**
`rav1e` → `ravif` → `image` (the AVIF encoder). Because `deny.toml` sets
`[graph] all-features = true`, the `avif`-feature path keeps `paste` in the evaluated
graph even after `little_exif` is gone. `rav1e 0.8.1` is the latest (no fix). So
`-2024-0436` remains a **documented residual** ignore for 0.2.0 (an unmaintained,
build-time proc-macro — the lowest-risk of the four), with a revisit trigger of "rav1e
drops paste". Maintainer-accepted 2026-07-04.

**Design-time probes (on a real EXIF-bearing JPEG):** (1) `img-parts` `.exif()` returns
the raw TIFF block. (2) A minimal **generic IFD rewriter** — model each entry as
`(tag, type, count, value-bytes)`, recurse into the sub-IFD pointer tags (`ExifIFD 0x8769`,
`GPS 0x8825`, `Interop 0xA005`), then re-serialize with recomputed offsets — **round-trips
faithfully**: a JPEG carrying IFD0 strings **and** an ExifIFD sub-tag (`ExposureTime`)
came back byte-for-byte identical per `kamadak-exif`. This validates the writer's core.

## The writer (what gets built)

A `tiff` submodule with a bounded, panic-free parser + a normalizing serializer:
- **Parse** the TIFF header (`II`/`MM` + magic 42 + IFD0 offset) → a tree: IFD0, its
  sub-IFDs (via the pointer tags), IFD1 (thumbnail, via IFD0's next-IFD link). Every
  offset/length is **bounds-checked**; malformed/truncated/cyclic input → a typed
  `MetadataError`, never a panic (untrusted input, STAGE-006 / DEC-034/036/038). A
  recursion-depth cap guards pointer cycles.
- **Edit**: `set_tags` adds/replaces the three IFD0 ASCII tags (`ImageDescription 0x010E`,
  `Artist 0x013B`, `Copyright 0x8298`); `clean_gps` drops the `GPS 0x8825` pointer entry
  (orphaning the GPS IFD). All other tags/sub-IFDs pass through untouched.
- **Serialize** to a normalized **little-endian** TIFF (matches what `little_exif` emitted;
  consumers handle either order), relocating out-of-line values and the IFD1 thumbnail
  blob (`JPEGInterchangeFormat 0x0201` + `0x0202`) with recomputed offsets.
  > ⚠️ **Amended by [DEC-076](DEC-076-tiff-writer-preserves-input-byte-order.md)
  > (SPEC-093): normalizing was wrong and silently corrupted data.** Entry values are
  > carried through *verbatim in the input's byte order*, so re-labelling the header
  > `II` made every reader misread a big-endian block's numeric tags (Orientation
  > `6` → `1536`; GPS drifted to a plausible-but-wrong coordinate; the IFD1
  > thumbnail length to `504954880`, dangling the pointer). The writer now
  > **preserves the input's byte order**; only `minimal()` (no existing EXIF) is
  > little-endian.
- **Embed** via `img-parts` `set_exif(Some(tiff))` (JPEG APP1 + PNG native `eXIf`),
  preserving pixels exactly (`metadata-not-via-pixel-encode`). No-EXIF cases keep today's
  behavior: `set` synthesizes a minimal TIFF; `clean` is a byte-faithful no-op.

## Alternatives Considered

- **Bump `little_exif`** — impossible: 0.6.23 is latest and still pins vulnerable
  `quick-xml` + `paste`.
- **Keep `little_exif`, keep the ignores** — rejected: the 0.2.0 goal is to eliminate the
  real vulns at the source; the XML dep is pure liability for a binary-EXIF tool.
- **Another EXIF-writer crate** — none exists that is pure-Rust, permissive, and
  read+write at tag level (`kamadak-exif`/`nom-exif` read-only).
- **Full generic TIFF re-encode via `kamadak-exif` fields** — rejected: reconstructing a
  serializer from `kamadak`'s typed `Value`s means handling every value type and thumbnail
  relocation anyway; the opaque `(tag,type,count,bytes)` model is simpler and more faithful
  (unknown tags pass through verbatim).
- **Also strip `paste`** by narrowing `deny`'s feature graph — rejected: `all-features`
  is deliberate (catch copyleft/advisories behind flags, DEC-018); accept the residual.

## Consequences

- **Positive:** `quick-xml` (`-0194`/`-0195`) and `brotli` leave the tree; two ignores
  deleted; the metadata write lane is pure-Rust, permissive, and XML-free. `deny.toml`
  goes from 3 ignores → 1.
- **Negative:** we own EXIF serialization now — byte-order, offset, and thumbnail
  relocation logic that must be correct and hardened against malformed input. Mitigated by
  the probe-validated approach + round-trip tests (incl. a thumbnail case) + the STAGE-006
  no-panic bar. Output TIFF bytes differ from `little_exif`'s (normalized LE) but are
  semantically identical (asserted via `kamadak-exif`, not byte-compare).
- **Neutral:** `set`/`clean --gps`/`strip`/`copy-metadata` CLIs and exit codes unchanged;
  `img-parts` + `kamadak-exif` remain; `-2024-0436` stays documented until `rav1e` moves.

## Validation

Right if: after the swap `cargo tree` shows no `little_exif`/`quick-xml`/`brotli`;
`deny.toml` has no `-0194`/`-0195`; `just deny` passes (with `-2024-0436` still listed);
`set`/`clean --gps` round-trip through `kamadak-exif` with the target tags added/GPS
removed and all other tags (incl. an ExifIFD sub-tag and an IFD1 thumbnail) preserved;
malformed EXIF yields a typed error, not a panic; the lean build compiles. Revisit if:
`rav1e` drops `paste` (then delete `-2024-0436` too), or we need to write tag groups beyond
IFD0 strings + GPS removal (extend the writer in a new spec).

## References

- Related specs: SPEC-045 (this writer); SPEC-026/SPEC-027 (`strip`/`clean`/`set`, the
  behavior being preserved); SPEC-044 (the prior STAGE-010 advisory elimination)
- Related decisions: **DEC-029** (amended — `little_exif` write choice retired), DEC-003
  (container-lane metadata), DEC-042 (accepted the ignores this removes), DEC-004/DEC-018
  (pure-Rust / permissive), DEC-034/036/038 (input hardening the parser must honor)
- Constraints: `metadata-not-via-pixel-encode`, `no-new-top-level-deps-without-decision`,
  `pure-rust-codecs-default`
- Advisories: `RUSTSEC-2026-0194`/`-0195` (quick-xml, removed); `RUSTSEC-2024-0436`
  (paste — residual, via rav1e/avif; not removed)
- External: TIFF 6.0 / EXIF 2.3 IFD structure; `img-parts` `ImageEXIF`; `kamadak-exif`
