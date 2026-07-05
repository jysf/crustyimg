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
- [x] build (2026-07-05) — Sonnet, prescriptive prompt. Wrote `src/metadata/tiff.rs`: a
  bounded, panic-free binary TIFF-IFD parser (bounds-checked every offset/length; `HashSet`-
  based cycle guard + `MAX_IFD_DEPTH = 8` cap on sub-IFD/next-IFD recursion) + a normalizing
  little-endian serializer (sub-IFDs, out-of-line values, and the IFD1 thumbnail blob all
  relocated with patched offsets). Re-implemented `set_tags`/`clean_gps` in
  `src/metadata/mod.rs` on `tiff` + `img-parts` `ImageEXIF::exif()`/`set_exif()`; public API
  (`TagSet`, `MetadataError`, fn signatures) unchanged, `run_set`/`run_clean` compile
  unchanged. Removed `little_exif` from `Cargo.toml`; deleted the `-0194`/`-0195` `deny.toml`
  entries and corrected the `-2024-0436` comment (paste now only via rav1e/avif). Re-based
  all `little_exif`-seeded test fixtures (`src/metadata/mod.rs` unit tests AND
  `tests/metadata.rs` integration tests, which also used `little_exif` and were not
  mentioned in the spec's file list) onto hand-assembled TIFF + `img-parts` seeding; added
  all 8 spec `## Failing Tests`, asserting semantics via `kamadak-exif` (`exif` crate), incl.
  a `Tag`-qualified (not bare tag-number) GPS-vs-generic comparison to avoid a
  context-collision false-positive. All gates green: `cargo test` 431 passed (16 suites),
  `cargo clippy --all-targets -D warnings` clean, `cargo fmt --check` clean, `cargo build
  --no-default-features` and `--features avif` both compile, `cargo deny check advisories
  bans sources licenses` green (advisories/bans/licenses/sources all ok). `cargo tree`:
  `little_exif`/`quick-xml`/`brotli` all 0 occurrences; `cargo tree -i paste` shows only the
  `rav1e→ravif→image` path (absent entirely without `--features avif`), confirming DEC-046.
- [ ] verify — independent Explore subagent (Opus); emphasis on hardening (no-panic on
  malformed EXIF) + round-trip fidelity (sub-IFD + thumbnail) via kamadak-exif.
- [ ] ship.
