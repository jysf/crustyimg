---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-027
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-004
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-06-18

references:
  decisions: [DEC-003, DEC-029, DEC-007, DEC-015]
  constraints:
    - metadata-not-via-pixel-encode
    - clippy-fmt-clean
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
  related_specs: [SPEC-026]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-004's <capability>". Optional; null is acceptable.
value_link: >
  Adds container-lane `set` (write artist/copyright/description) — the
  attribution counterpart to SPEC-026's strip/clean, continuing STAGE-004's
  metadata lane with no pixel re-encode.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-18
      notes: >
        Main-loop orchestrator work, not separately metered. Authored the spec
        (Failing Tests + Implementation Context) on top of SPEC-026's metadata
        lane; ran a design-time probe confirming little_exif set-then-write
        preserves existing tags + the no-EXIF fresh-create fallback (real JPEG).
        No new dep / no new DEC (reuses DEC-029's crates).
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-027: `set` — write EXIF artist/copyright/description

## Context

The **third metadata-lane spec** of STAGE-004, building directly on the lane
SPEC-026 shipped (`strip` + `clean --gps`). Where `strip`/`clean` *remove*
metadata, `set` *writes* it: artist, copyright, and description tags — the
**attribution** half of the verifiable-privacy/attribution axis (`docs/moat.md`).
It is the natural counterpart to `clean` and reuses the same container lane:
edit container-level EXIF via `little_exif` **without re-decoding pixels**
(DEC-003, constraint `metadata-not-via-pixel-encode`).

`set` exists today only as a clap stub (`Commands::Set { inputs, artist,
copyright, description }`) returning `CliError::NotImplemented("set")`. The lane
infrastructure is already in place from SPEC-026: `run_metadata_lane` (a fan-out
that takes a `Fn(&[u8]) -> Result<Vec<u8>, MetadataError>` transform),
`Sink::write_bytes`, `metadata_output_ext`, the `MetadataError` enum, and the
`sniff`/`Lane`/`file_extension` helpers. This spec adds one transform function
and one handler — no new dependency, **no new DEC** (reuses DEC-029's pinned
`little_exif`).

Parent: `STAGE-004-compose-and-metadata`. Governing: **DEC-003** (dual-lane),
**DEC-029** (pinned crates). Builds on **SPEC-026** (PR #30).

## Goal

Wire `set <inputs…> [--artist S] [--copyright S] [--description S]` to write the
given EXIF tags into the container via `little_exif`, **preserving all other
metadata and the pixels exactly** (no re-encode), for JPEG + PNG. At least one
tag flag is required; unsupported formats exit cleanly (code 4).

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `Commands::Set` clap variant (already declared), the
    `NotImplemented("set")` dispatch arm, `run_metadata_lane`/`run_clean`/
    `run_strip` (the pattern to follow), `CliError`.
  - `src/metadata/mod.rs` — `strip_all`/`clean_gps`, `MetadataError`, the
    `sniff`/`Lane`/`file_extension` helpers, the `#[cfg(test)]` fixture helpers
    (`jpeg_with_exif`-style) — extend these.
  - `decisions/DEC-029-*.md` (its probe), `decisions/DEC-003-*.md`.
- **External crate (already pinned, DEC-029):** `little_exif` `=0.6.23` —
  `ExifTag::{Artist, Copyright, ImageDescription}` (all STRING, IFD0/GENERIC).
- **Related code paths:** `src/metadata/mod.rs`, `src/cli/mod.rs`,
  `tests/metadata.rs`.

## Outputs

- **Files modified:**
  - `src/metadata/mod.rs` — add:
    - `pub struct TagSet { pub artist: Option<String>, pub copyright: Option<String>, pub description: Option<String> }`
    - `pub fn set_tags(bytes: &[u8], tags: &TagSet) -> Result<Vec<u8>, MetadataError>`
  - `src/cli/mod.rs` — `run_set(inputs, artist, copyright, description, global)`;
    wire the `Commands::Set` dispatch arm to it. At least one tag → else
    `CliError::Usage` (exit 2). Delegates to `run_metadata_lane` with a closure
    `|bytes| metadata::set_tags(bytes, &tags)`.
  - `tests/metadata.rs` — extend with `set` integration tests.
  - `docs/api-contract.md` — flesh out the `set` entry (done at design).
- **New exports:** `crate::metadata::{TagSet, set_tags}`.
- **Database changes:** none.

## Command surface (PINNED)

```
crustyimg set <INPUTS...> [--artist NAME] [--copyright NOTICE] [--description TEXT]
```

- **At least one** of `--artist`/`--copyright`/`--description` is required.
  None → `CliError::Usage("set requires at least one of --artist/--copyright/--description")`
  (exit **2**).
- **Format coverage v1:** JPEG + PNG (same as SPEC-026). Other formats → exit **4**
  (per-input in a batch → exit 6). Format preserved; `-q`/`--format` ignored.
- **Semantics:** `set` writes the given tags, **overwriting** any existing value of
  the same tag and **preserving** every other tag and segment. On a file with no
  existing EXIF, it creates a fresh EXIF block carrying just the given tags.
- **Fan-out:** identical to `strip`/`clean` (reuses `run_metadata_lane`) — single
  input → stdout / `-o` / `--out-dir`; multiple → require `--out-dir`; per-input
  failure → exit 6; overwrite guarded by `-y`.

## Lane mechanics (PINNED — probe-verified)

`set_tags(bytes, tags)`:
1. `sniff(bytes)` → `Lane::Jpeg`/`Lane::Png`, else `MetadataError::UnsupportedFormat`
   (reuse the SPEC-026 helper). `ext = file_extension(lane)`.
2. Load existing metadata to preserve it:
   `let mut md = Metadata::new_from_vec(&bytes.to_vec(), ext).unwrap_or_else(|_| Metadata::new());`
   — the `Err` branch is the "No EXIF" / fresh-create fallback (probe-verified).
3. For each `Some` tag, `md.set_tag(ExifTag::Artist(s.clone()))` /
   `ExifTag::Copyright(..)` / `ExifTag::ImageDescription(..)`. `set_tag` overwrites
   the same tag and leaves others intact.
4. `let mut out = bytes.to_vec(); md.write_to_vec(&mut out, ext)?; Ok(out)` — map any
   `little_exif` write error to `MetadataError::Exif`.
- **Pixel invariant:** the compressed scan / IDAT is carried verbatim — decoded
  pixels are byte-identical (probe-confirmed for set-on-existing and set-on-empty).

## Acceptance Criteria

- [ ] `set --artist A --copyright C --description D` on a JPEG writes all three tags
  (re-parse reads them back); decoded pixels identical to input.
- [ ] `set` on a JPEG that already has Orientation + GPS leaves those intact (only
  the named tags change); decoded pixels identical.
- [ ] `set --copyright NEW` over a file whose Copyright was OLD → re-parse reads NEW.
- [ ] `set --artist A` on a JPEG with no EXIF creates the tag (fresh-create path).
- [ ] `set` works on PNG (writes EXIF to the container); decoded pixels identical.
- [ ] `set` with no tag flags → exit **2**.
- [ ] `set` on an unsupported format → exit **4** (single) / counts toward exit 6 in a batch.
- [ ] Fan-out: single → stdout/`-o`/`--out-dir`; multi → `--out-dir`; partial → exit 6;
  overwrite refused without `-y`.
- [ ] `cargo deny` green (no new dep); build stays pure-Rust.

## Failing Tests

Written during **design**, BEFORE build. Reuse SPEC-026's native fixture helpers
(`image` crate pixels + `little_exif` to seed EXIF); no ImageMagick.

- **`src/metadata/mod.rs` (unit, extend `#[cfg(test)] mod tests`)**
  - `"set_tags_writes_all_three"` — `set_tags` with artist+copyright+description →
    re-parse reads all three.
  - `"set_tags_preserves_existing_metadata"` — seed Orientation+GPS, `set_tags`
    artist → Orientation + GPS still present; artist added.
  - `"set_tags_overwrites_existing_tag"` — seed Copyright="OLD", `set_tags`
    copyright="NEW" → re-parse reads "NEW".
  - `"set_tags_on_no_exif_creates_them"` — plain JPEG (no EXIF), `set_tags` artist
    → artist present (fresh-create fallback).
  - `"set_tags_preserves_pixels"` — decode-equality input vs output.
  - `"set_tags_png"` — PNG input, `set_tags` copyright → re-parse reads it; pixels equal.
  - `"set_tags_unsupported_format_errors"` — BMP/GIF blob → `UnsupportedFormat`.
- **`tests/metadata.rs` (integration, extend)**
  - `"set_writes_tags_to_output"` — `set fixture.jpg --artist Jane --copyright 2026 -o out.jpg`
    → out has both tags; exit 0.
  - `"set_without_any_flag_exits_2"` — `set fixture.jpg` → exit 2.
  - `"set_preserves_other_metadata"` — fixture with Orientation, `set --copyright X`
    → Orientation survives; exit 0.
  - `"set_unsupported_format_exits_4"` — `set fixture.bmp --artist A` → exit 4.
  - `"set_multi_input_fanout_writes_all"` — two JPEGs + `--out-dir` + `--artist A` →
    two outputs both tagged; exit 0.
- **`tests/cli.rs`** — `"set"` is already in the subcommand-list tests (added in
  SPEC-007); confirm, add only if missing.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-003` — **governing.** Container lane; `set` never touches the pixel encode path.
- `DEC-029` — `little_exif` `=0.6.23` is already pinned; `set` uses its `set_tag` +
  `write_to_vec`. No new dep, no new DEC.
- `DEC-007` — typed `thiserror`; no `unwrap`/`expect`/`panic!` off test paths; map
  `little_exif` errors to `MetadataError::Exif`.
- `DEC-015` — the fan-out + exit-code semantics live in `run_metadata_lane` (reused as-is).

### Constraints that apply

- `metadata-not-via-pixel-encode` (blocking) — no decode/encode of pixels; asserted
  via decode-equality tests.
- `clippy-fmt-clean`, `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`.

### Prior related work

- `SPEC-026` (shipped, PR #30) — built the metadata lane this spec extends:
  `run_metadata_lane`, `Sink::write_bytes`, `MetadataError`, `sniff`/`Lane`/
  `file_extension`, native EXIF fixtures. **Read `src/metadata/mod.rs` and
  `run_clean`/`run_strip` first — `set` mirrors them.**
- `DEC-029`'s probe (and this spec's design-time probe) verified the exact
  `little_exif` set/preserve/fresh-create behavior on a real JPEG.

### Out of scope (for this spec specifically)

- `copy-metadata` (the last metadata-lane spec; transfers a whole container's
  metadata `--from`/`--to`).
- `watermark` (pixel-lane; separate STAGE-004 spec).
- Arbitrary/other EXIF tags beyond artist/copyright/description (e.g. datetime,
  camera fields), XMP/IPTC writing, and per-tag removal (that's `clean`'s job).
- WebP/TIFF (v1 metadata lane is JPEG + PNG).

## Notes for the Implementer

- **Mirror `run_clean`/`run_strip`** (`src/cli/mod.rs`) — `run_set` differs only in
  (a) the at-least-one-flag usage check and (b) passing a *capturing closure*
  `|bytes| crate::metadata::set_tags(bytes, &tags)` to `run_metadata_lane` (which
  already accepts `impl Fn(&[u8]) -> Result<Vec<u8>, MetadataError>`).
- **Load-then-set preserves** — always `Metadata::new_from_vec(..).unwrap_or_else(|_|
  Metadata::new())` so existing tags survive; do NOT start from `Metadata::new()`
  unconditionally (that would drop existing metadata). Probe-verified.
- `ExifTag::Artist`/`Copyright`/`ImageDescription` are all `String` constructors,
  IFD0/GENERIC group. `set_tag` overwrites same-tag, keeps others.
- Keep diagnostics on **stderr**; stdout stays clean for `-o -`.
- Build the `TagSet` from the owned `Option<String>` clap args; pass by reference to
  the closure so it lives across the fan-out.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
