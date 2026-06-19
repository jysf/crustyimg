# SPEC-028 build prompt — `copy-metadata` (transfer EXIF+ICC SRC→DST)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-028 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests + DEC-030. This is the LAST
metadata-lane command and reuses the SPEC-026/027 infrastructure. Make the spec's
`## Failing Tests` pass with the smallest correct change, then open a PR and STOP.
**No pixel re-decode/encode of DST** (constraint `metadata-not-via-pixel-encode`).

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-028-copy-metadata-from-one-image-to-another.md`
   — `## Command surface (PINNED)`, `## Lane mechanics (PINNED)`, `## Failing Tests`,
   `## Notes for the Implementer`.
2. `decisions/DEC-030-copy-metadata-jpeg-only-png-exif-crate-mismatch.md` — the
   probe + the JPEG-only rationale. **Mirror its verified snippet.**
3. `src/metadata/mod.rs` — `MetadataError`, `sniff`/`Lane`/`file_extension`, fixtures.
   You ADD `copy_metadata` here.
4. `src/cli/mod.rs` — `run_strip`/`run_set` (handler patterns), `read_raw_bytes`,
   `metadata_output_ext`, `Sink`/`Overwrite`, the `Commands::CopyMetadata { from, to }`
   variant + its `NotImplemented` arm.
5. `src/sink/mod.rs` — `Sink::write_bytes`, `Sink::File`/`Stdout`.

## What to build (no new dep, no new DEC beyond DEC-030)
- `src/metadata/mod.rs`:
  - `pub fn copy_metadata(from: &[u8], to: &[u8]) -> Result<Vec<u8>, MetadataError>` —
    `sniff(from)` AND `sniff(to)` must be `Lane::Jpeg`, else
    `MetadataError::UnsupportedFormat("copy-metadata supports JPEG only in v1")`;
    `Jpeg::from_bytes` both (parse err → `MetadataError::Container`);
    `dst.set_exif(src.exif()); dst.set_icc_profile(src.icc_profile());`
    (import `img_parts::{ImageEXIF, ImageICC}` for the trait methods);
    `dst.encoder().write_to(&mut out)` → `Ok(out)`.
- `src/cli/mod.rs`:
  - `run_copy_metadata(from, to, global)` — NOT a fan-out. `std::fs::read` each of
    `from`/`to` (read error → exit 3). `copy_metadata(&from_bytes, &to_bytes)`. Build
    the output `Sink`: `-o PATH`→`Sink::File`, `-o -`→`Sink::Stdout`, else default
    `Sink::File { path: <to> }` (in-place). Overwrite from `-y`
    (`Overwrite::Allow`/`Forbid`); in-place with no `-y` → `Sink::write_bytes` returns
    the AlreadyExists error → exit 5. Use ext from `metadata_output_ext` or `"jpg"`.
  - Wire the `Commands::CopyMetadata { from, to }` dispatch arm to `run_copy_metadata`.

## Tests — every named test in the spec's `## Failing Tests` must exist + pass
- 6 unit tests in `src/metadata/mod.rs`; 4 integration in `tests/metadata.rs`.
- Reuse SPEC-026/027 native fixtures; seed SRC EXIF via `little_exif`, seed an ICC
  blob via `img-parts` `set_icc_profile`. Assert no-DST-pixel-change via
  decode-equality. **Confirm each named test exists** before claiming green.

## Gates (all must pass)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check licenses   # no new dep — must stay green
```

## Git / PR
- Branch `feat/spec-028-copy-metadata` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(metadata): copy-metadata transfer EXIF+ICC (SPEC-028)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-003, DEC-029, DEC-030,
  DEC-015, DEC-007 / Constraints checked / New decisions — "No new DEC" — DEC-030 was
  emitted at design).
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; orchestrator fills at ship).

## Cost
```
- cycle: build
  agent: claude-opus-4-8
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-18
  notes: "copy-metadata: metadata::copy_metadata (img-parts ImageEXIF/ImageICC transfer) + run_copy_metadata (two inputs, single fixed output, in-place behind -y); JPEG-only (DEC-030); no pixel re-encode; no new dep"
```
(Use the agent id of the session that actually runs the build.)

## When done
`just advance-cycle SPEC-028 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
