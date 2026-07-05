# SPEC-045 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-045-<cycle>.md`.

## Instructions

- [x] design (2026-07-04) — Opus, main loop. Second STAGE-010 spec. Authored the spec
  (failing tests + implementation context) and **DEC-046** (in-house TIFF-IFD EXIF writer;
  amends DEC-029; drop `little_exif` → remove RUSTSEC-2026-0194/-0195 quick-xml + `brotli`).
  **Three design-time checks:** (1) confirmed no upstream shortcut — `little_exif 0.6.23` is
  latest and still pins vulnerable `quick-xml ^0.37` + `paste`; (2) corrected the backlog's
  premise — `paste` (-2024-0436) also comes via `rav1e`→`ravif`→`image` (avif) and
  `deny.toml` is `all-features=true`, so it is NOT removable here (maintainer accepted it as
  the 1 residual ignore); (3) **probe-validated the writer core** — `img-parts` `.exif()`
  returns the bare TIFF (`II*`), and a generic IFD parse→recurse-subIFD→re-serialize
  round-tripped a real JPEG (IFD0 strings + ExifIFD `ExposureTime`) byte-identical per
  `kamadak-exif`. Design + DEC-046 + STAGE-010 update to be pushed to `main` before build.
- [ ] build — Sonnet, prescriptive prompt. Write the `tiff` submodule (bounded panic-free
  parser + LE serializer), re-implement `set_tags`/`clean_gps` on it via `img-parts`, drop
  `little_exif`, delete `-0194`/`-0195`, re-base the test helpers, make the 8 new + existing
  tests pass; lean + full `deny` green (`-2024-0436` residual retained).
- [ ] verify — independent Explore subagent (Opus); emphasis on hardening (no-panic on
  malformed EXIF) + round-trip fidelity (sub-IFD + thumbnail) via kamadak-exif.
- [ ] ship.
