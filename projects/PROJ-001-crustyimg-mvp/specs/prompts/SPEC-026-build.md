# SPEC-026 build prompt — metadata lane v1 (`strip` + `clean --gps`)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-026 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests + DEC-029. Make the
`## Failing Tests` pass with the smallest correct change. **The container lane must
never re-decode/re-encode pixels** (constraint `metadata-not-via-pixel-encode`).

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-026-metadata-lane-strip-and-clean-gps.md`
   — especially `## Command surface (PINNED)`, `## Lane mechanics (PINNED)`,
   `## Failing Tests`, `## Notes for the Implementer`.
2. `decisions/DEC-029-metadata-write-crates-img-parts-little-exif.md` — the
   **probe in `## Context` has the exact, verified API calls — mirror them**.
3. `decisions/DEC-003-metadata-dual-lane.md` — the governing two-lane decision.
4. `src/cli/mod.rs` — `run_pixel_op` (fan-out shape), `Commands::Strip`/`Clean`,
   `dispatch`, `CliError` + `code()`, `build_sink`, `GlobalArgs`, `Overwrite`.
5. `src/sink/mod.rs` — `Sink`, `SinkInput`, `safe_join`, `extension_for_format`,
   the overwrite guard (for the raw-bytes write path).

## What to build
- **`src/metadata/mod.rs`** (new; `pub mod metadata;` in `src/lib.rs`). Pure
  byte→byte API, format sniffed with `image::guess_format` (NO `Image::load`):
  - `pub fn strip_all(bytes: &[u8]) -> Result<Vec<u8>, MetadataError>`
  - `pub fn clean_gps(bytes: &[u8]) -> Result<Vec<u8>, MetadataError>`
  - `pub enum MetadataError { UnsupportedFormat(String), Container(String), Exif(String) }`
    (`thiserror`). JPEG + PNG only; else `UnsupportedFormat`.
  - `clean_gps`: catch `little_exif`'s `Err` containing "No EXIF" → `Ok(bytes.to_vec())`
    (no-op). Drop the GPS IFD via `get_ifd_mut(ExifTagGroup::GPS, 0)` + `remove_tag`.
- **`src/sink/mod.rs`** — add a raw-bytes write path (e.g. `Sink::write_bytes`) that
  writes container bytes verbatim; format preserved (ext = input extension); reuse
  `safe_join` + overwrite guard. Write failure → `SinkError` (exit 5).
- **`src/cli/mod.rs`** — `CliError::Metadata(#[from] MetadataError)`; `code()`:
  `UnsupportedFormat → 4`, `Container`/`Exif → 1`. Wire `Strip`/`Clean` dispatch
  arms → `run_strip`/`run_clean` over a shared `run_metadata_lane` fan-out that
  mirrors `run_pixel_op` BUT reads raw bytes (`std::fs::read` / stdin bytes) and
  transforms via the lane (no decode). `clean` without `--gps` → `CliError::Usage`
  (exit 2). Reuse the global `--out-dir`/`-y` flags (do NOT declare locals — the
  SPEC-024 collision lesson). File-read I/O after resolve → exit 3.

## Tests — every one in the spec's `## Failing Tests` must exist and pass
- 8 unit tests in `src/metadata/mod.rs`; 8 integration tests in `tests/metadata.rs`.
- Generate fixtures **natively** (no ImageMagick): `image` crate for pixels,
  `little_exif` to seed Orientation + Copyright + GPS (`jpeg_with_exif()` helper).
  Assert no-pixel-change via decode-equality (`to_rgba8()` buffers equal).
- Add `"strip"` + `"clean"` to BOTH subcommand-list tests in `tests/cli.rs` if they
  exist. **Confirm each named test exists** before claiming green.

## Gates (all must pass)
```
cargo fmt            # then `git add -u`
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check licenses   # img-parts + little_exif already pinned (DEC-029) — must stay green
```

## Git / PR
- Branch `feat/spec-026-metadata-lane` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before each commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(metadata): strip + clean --gps container lane (SPEC-026)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-003, DEC-029, DEC-015,
  DEC-004, DEC-007 / Constraints checked / New decisions — "No new DEC" unless the
  build forces one).
- Fill the spec's `## Build Completion` + the 3 reflection answers.

## Cost
Append a build session to `cost.sessions` (numerics null; orchestrator fills at ship):
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-18
  notes: "metadata lane v1: src/metadata strip_all/clean_gps + Sink::write_bytes + run_metadata_lane fan-out + CliError::Metadata + tests; container lane, no pixel re-encode"
```

## When done
`just advance-cycle SPEC-026 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
