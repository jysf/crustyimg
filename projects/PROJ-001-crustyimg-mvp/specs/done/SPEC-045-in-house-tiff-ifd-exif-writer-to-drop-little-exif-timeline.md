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
- [x] verify (2026-07-05) — independent Explore subagent (Opus, 85526 tok, ~6 min).
  Adversarial hardening audit: every offset/length via `checked_add`/`checked_mul` +
  bounds-checked `get_slice`; `type_size*count` overflow guarded; depth-cap + HashSet cycle
  guard; the 4 crafted malformed inputs (short header, OOB IFD0/value offsets, self-cycle)
  all return `Err`, verified under `RUST_BACKTRACE=full`. Round-trip fidelity (ExifIFD
  sub-tag + IFD1 thumbnail + GPS-only removal) confirmed via kamadak-exif; pixels bit-equal
  (JPEG+PNG); no scope creep (API/CLI/strip/copy unchanged; deny only -0194/-0195 removed,
  -2024-0436 kept). All gates re-run incl. `--features avif`. VERDICT **PASS**, no defects.
  Orchestrator independently confirmed the checked arithmetic + tree/tests beforehand.
- [x] ship (2026-07-05) — Opus, main loop. Squash-merged PR #50 → `main` (cad7be9); all 19
  PR checks + main CI green. Recorded real cycle tokens; archived the spec; STAGE-010 backlog
  2/3. **Two more `deny.toml` ignores eliminated** (quick-xml -0194/-0195; `little_exif` +
  `quick-xml` + `brotli` out of the tree). `deny.toml` now 3 → 1 (paste residual only).
