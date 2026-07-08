# SPEC-061 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — RAW Tier-1 embedded-preview extraction as a default input; Failing Tests
  (extension routing, pick-largest-decodable-JPEG, skip false SOIs, bounded decode attempts,
  oversize→LimitsExceeded, no-preview→typed error, dir-source discovery, optimize→webp / convert→png)
  + full Implementation Context. Load-bearing item done in design: a firsthand probe of the repo's
  pinned `image` → **finding: a format-agnostic byte scan for the largest embedded JPEG (decode from
  each SOI, `image` tolerates trailing bytes, keep largest by pixels) covers TIFF-based RAW + CR3 +
  RAF with NO new dependency and NO IFD/ISOBMFF parsing — correcting the brief's "ISOBMFF glue for
  CR3" and the watchlist's "parse the TIFF/EXIF IFDs".** Approach → **DEC-055** (emit at build).
  Detection is extension-driven in `Image::load` (TIFF-based RAW is byte-ambiguous with `.tif`).
  Framing, 2026-07-08.
- [x] **build** — add `src/image/raw.rs` (JPEG-SOI scan → capped `image` decode of each candidate →
  largest wins → canonical `Image`, `source_format=Jpeg`), route by extension in `Image::load`, add RAW
  extensions to `IMAGE_EXTENSIONS`, synthetic fixture + tests + `fuzz/raw_preview`, DEC-055. No new dep /
  no deny change. Verify default + lean + `just deny` + clippy + fmt; check MSRV.
  **Done 2026-07-08** on `feat/spec-061-raw-preview`: 14 new RAW tests (11 unit + 3 integration), full
  suite 569 green; default+lean clippy/fmt/build clean; `just deny` UNCHANGED & green (no new crate);
  MSRV floor 1.90 unchanged (Cargo.toml/lock/deny untouched); DEC-055 emitted (decisions-audit 0 errors).
  PR opened for a separate verify session.
- [ ] **verify** — fresh session; re-run all gates independently, confirm hostile-input safety
  (bounded candidate decodes, cap-per-decode, false-SOI skip), no new dep, DEC-055 consistent.
- [ ] **ship** — merge PR, cost sessions + totals, ship reflection, archive to done/, update
  STAGE-018 backlog; carry `fuzz/raw_preview` as a pre-1.0 hardening gate (like `fuzz/avif_decode`).
