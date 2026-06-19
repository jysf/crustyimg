# SPEC-027 build prompt — `set` (write EXIF artist/copyright/description)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-027 in the `crustyimg`
repo. The architect (Opus) wrote the spec + failing tests. This extends the metadata
container lane SPEC-026 shipped — **most of the infrastructure already exists**. Make
the `## Failing Tests` pass with the smallest correct change. **No pixel
re-decode/encode** (constraint `metadata-not-via-pixel-encode`).

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-027-set-exif-tags-artist-copyright-description.md`
   — especially `## Command surface (PINNED)`, `## Lane mechanics (PINNED)`,
   `## Failing Tests`, `## Notes for the Implementer`.
2. `src/metadata/mod.rs` — `strip_all`/`clean_gps`, `MetadataError`, `sniff`/`Lane`/
   `file_extension`, the `#[cfg(test)]` fixture helpers. You ADD `TagSet` + `set_tags` here.
3. `src/cli/mod.rs` — `run_metadata_lane` (takes `impl Fn(&[u8]) -> Result<Vec<u8>, MetadataError>`),
   `run_clean`/`run_strip` (the pattern), the `Commands::Set` clap variant +
   `NotImplemented("set")` dispatch arm, `CliError::Usage`.

## What to build (no new dependency, no new DEC)
- `src/metadata/mod.rs`:
  - `pub struct TagSet { pub artist: Option<String>, pub copyright: Option<String>, pub description: Option<String> }`
  - `pub fn set_tags(bytes: &[u8], tags: &TagSet) -> Result<Vec<u8>, MetadataError>` —
    `sniff` → JPEG/PNG else `UnsupportedFormat`; `Metadata::new_from_vec(&bytes.to_vec(), ext).unwrap_or_else(|_| Metadata::new())`
    (the Err branch is the no-EXIF fresh-create fallback — PRESERVES existing tags
    otherwise); `set_tag(ExifTag::Artist/Copyright/ImageDescription(..))` for each
    `Some`; `write_to_vec` into a clone; map write errors → `MetadataError::Exif`.
- `src/cli/mod.rs`:
  - `run_set(inputs, artist, copyright, description, global)` — if all three `None`
    → `CliError::Usage("set requires at least one of --artist/--copyright/--description")`
    (exit 2); build a `TagSet`; `run_metadata_lane(inputs, global, |b| metadata::set_tags(b, &tags))`.
  - Wire the `Commands::Set { .. }` dispatch arm to `run_set`.

## Tests — every one in the spec's `## Failing Tests` must exist + pass
- 7 unit tests in `src/metadata/mod.rs`; 5 integration tests in `tests/metadata.rs`.
- Reuse SPEC-026's native fixtures (image crate + little_exif seed). Assert
  no-pixel-change via decode-equality. **Confirm each named test exists** before green.

## Gates (all must pass)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check licenses   # no new dep — must stay green
```

## Git / PR
- Branch `feat/spec-027-set-exif-tags` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked
  `reports/daily|weekly/*.md`.
- PR title: `feat(metadata): set artist/copyright/description tags (SPEC-027)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-003, DEC-029, DEC-015,
  DEC-007 / Constraints checked / New decisions — "No new DEC").
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
  notes: "set command: metadata::TagSet + set_tags + run_set reusing run_metadata_lane; container lane, no pixel re-encode; no new dep"
```
(Use the agent id of the session that actually runs the build.)

## When done
`just advance-cycle SPEC-027 verify`, open the PR, and **stop** — the orchestrator
pauses for the user before any merge.
