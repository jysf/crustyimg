---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-028
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
  decisions: [DEC-003, DEC-029, DEC-030, DEC-007, DEC-015]
  constraints:
    - metadata-not-via-pixel-encode
    - clippy-fmt-clean
    - every-public-fn-tested
    - no-unwrap-on-recoverable-paths
  related_specs: [SPEC-026, SPEC-027]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-004's <capability>". Optional; null is acceptable.
value_link: >
  Completes the container-lane metadata commands with `copy-metadata`
  (transfer EXIF+ICC SRC→DST), the last metadata piece of STAGE-004.

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
        (Failing Tests + Implementation Context); ran a design-time probe that
        verified JPEG EXIF+ICC transfer via img-parts traits (pixels identical)
        AND surfaced the PNG cross-crate EXIF-chunk mismatch → emitted DEC-030
        (copy-metadata JPEG-only v1). No new dep.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-028: `copy-metadata` — transfer EXIF + ICC from one image to another

## Context

The **last metadata-lane spec** of STAGE-004 — it completes the container-lane
command set (`strip`/`clean`/`set`/`copy-metadata`). `copy-metadata --from SRC
--to DST` grafts SRC's container metadata onto DST **without re-decoding DST's
pixels** (DEC-003). It is the only metadata command that reads **two** inputs
(SRC = metadata donor, DST = pixel recipient), so it does NOT use the
single-stream `run_metadata_lane` fan-out — it has its own one-output handler.

A design-time probe (recorded in **DEC-030**) verified JPEG EXIF+ICC transfer via
`img-parts`' `ImageEXIF`/`ImageICC` traits with byte-identical DST pixels, and
surfaced that **PNG copy is not viable in v1**: `little_exif` writes PNG EXIF as a
`zTXt` "Raw profile type exif" chunk while `img-parts` uses the native `eXIf`
chunk, so they can't read each other's PNG EXIF. **Therefore copy-metadata is
JPEG-only in v1** (DEC-030); non-JPEG inputs exit 4.

`copy-metadata` exists today only as a clap stub (`Commands::CopyMetadata { from,
to }`) returning `CliError::NotImplemented`. Reuses `MetadataError`, the `sniff`
helper, and `Sink::write_bytes` from SPEC-026. No new dependency.

Parent: `STAGE-004-compose-and-metadata`. Governing: **DEC-003**, **DEC-029**
(pinned crates), **DEC-030** (JPEG-only scope). Builds on **SPEC-026/027**.

## Goal

Wire `copy-metadata --from SRC --to DST` to copy SRC's **EXIF + ICC** onto DST's
JPEG container via `img-parts` traits, preserving DST's pixels exactly (no
re-encode), and write the result. JPEG only; non-JPEG → exit 4.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `Commands::CopyMetadata { from, to }` (already declared),
    the `NotImplemented("copy-metadata")` dispatch arm, `run_strip`/`run_set`
    (handler patterns), `read_raw_bytes`, `metadata_output_ext`, `Sink`/`Overwrite`,
    `CliError`.
  - `src/metadata/mod.rs` — `MetadataError`, `sniff`/`Lane`, `file_extension`, the
    `#[cfg(test)]` fixtures — extend these.
  - `src/sink/mod.rs` — `Sink::write_bytes`, `Sink::File`/`Stdout`.
  - `decisions/DEC-030-*.md` (the probe + JPEG-only rationale), `DEC-003`, `DEC-029`.
- **External crate (pinned, DEC-029):** `img-parts` `=0.4.0` —
  `img_parts::{ImageEXIF, ImageICC}` traits (`exif()`/`set_exif`,
  `icc_profile()`/`set_icc_profile`) on `img_parts::jpeg::Jpeg`.
- **Related code paths:** `src/metadata/mod.rs`, `src/cli/mod.rs`,
  `tests/metadata.rs`.

## Outputs

- **Files modified:**
  - `src/metadata/mod.rs` — add
    `pub fn copy_metadata(from: &[u8], to: &[u8]) -> Result<Vec<u8>, MetadataError>`.
  - `src/cli/mod.rs` — `run_copy_metadata(from, to, global)`; wire the
    `Commands::CopyMetadata` dispatch arm.
  - `tests/metadata.rs` — `copy-metadata` integration tests.
  - `docs/api-contract.md` — flesh out the `copy-metadata` entry (done at design).
- **New exports:** `crate::metadata::copy_metadata`.
- **Database changes:** none.

## Command surface (PINNED)

```
crustyimg copy-metadata --from SRC --to DST [-o OUT] [-y]
```

- **Format coverage v1: JPEG only** (DEC-030). If SRC or DST is not JPEG →
  `MetadataError::UnsupportedFormat` (exit **4**) with a clear message naming the
  JPEG-only limitation.
- **What is copied:** SRC's **EXIF (APP1) + ICC (APP2)**. DST's existing EXIF/ICC are
  **replaced** by SRC's (a true "make DST's metadata match SRC's"). If SRC has no
  EXIF/ICC, DST's corresponding metadata is cleared (`set_exif(None)` /
  `set_icc_profile(None)`). XMP/IPTC are NOT transferred (out of scope).
- **Output target:**
  - `-o PATH` (or `-o -` for stdout) → write the grafted result there; **DST is left
    untouched** (read-only pixel source).
  - **Default (no `-o`)** → write the result **back to DST in place**. Because DST
    already exists, this is an overwrite → refused without `-y` (exit **5**); with
    `-y` it overwrites DST. (copy-metadata is the one command whose default output is
    a file, justified by the `--to` naming.)
- **Pixels:** DST's compressed scan is carried verbatim — decoded output pixels are
  byte-identical to DST's (probe-verified). SRC's pixels are irrelevant/ignored.
- This is a **single fixed output**, not a fan-out: no `--out-dir`, no globs on
  `--from`/`--to` (each is one path). `-q`/`--format` ignored.

## Lane mechanics (PINNED — probe-verified, DEC-030)

`copy_metadata(from, to)`:
1. `sniff(from)` and `sniff(to)` — both must be `Lane::Jpeg`, else
   `MetadataError::UnsupportedFormat("copy-metadata supports JPEG only in v1")`.
2. `let src = Jpeg::from_bytes(Bytes::from(from.to_vec()))` (map parse err →
   `MetadataError::Container`); `let mut dst = Jpeg::from_bytes(Bytes::from(
   to.to_vec()))?`.
3. `dst.set_exif(src.exif()); dst.set_icc_profile(src.icc_profile());` (the
   `ImageEXIF`/`ImageICC` traits; `Option<Bytes>` flows straight through — `None`
   clears).
4. `let mut out = Vec::new(); dst.encoder().write_to(&mut out).map_err(..Container)?;
   Ok(out)`.
- **Pixel invariant:** `dst.encoder()` re-serializes container segments + DST's
  original entropy/scan verbatim — no pixel re-encode.

## Acceptance Criteria

- [ ] `copy-metadata --from a.jpg --to b.jpg -o out.jpg` where `a` has EXIF
  (e.g. Copyright) → `out` carries `a`'s EXIF; `out`'s decoded pixels == `b`'s.
- [ ] ICC transfers: `a` with an ICC profile → `out` has that ICC profile.
- [ ] DST's prior metadata is replaced: `b` had its own Copyright="B" and `a` has
  Copyright="A" → `out` reads "A".
- [ ] SRC with no EXIF/ICC → `out` has DST's metadata cleared (no EXIF/ICC).
- [ ] Default (no `-o`) writes back to DST: refused without `-y` (exit **5**); with
  `-y`, DST is updated in place and its pixels unchanged.
- [ ] Non-JPEG `--from` or `--to` (PNG/BMP) → exit **4** with the JPEG-only message.
- [ ] No pixel re-encode (decode-equality; never constructs an `Image`).
- [ ] `cargo deny` green (no new dep).

## Failing Tests

Written during **design**, BEFORE build. Reuse SPEC-026/027 native fixtures (image
crate + `little_exif` to seed SRC EXIF; `img-parts` `set_icc_profile` to seed an ICC
blob); no ImageMagick.

- **`src/metadata/mod.rs` (unit, extend `#[cfg(test)] mod tests`)**
  - `"copy_metadata_transfers_exif"` — SRC with Copyright, DST without →
    `copy_metadata` → re-parse DST-out reads SRC's Copyright.
  - `"copy_metadata_transfers_icc"` — SRC with an injected ICC blob → out's
    `Jpeg::icc_profile()` equals it.
  - `"copy_metadata_preserves_recipient_pixels"` — decode-equality: out vs DST.
  - `"copy_metadata_replaces_recipient_metadata"` — DST had Copyright="B", SRC
    "A" → out reads "A".
  - `"copy_metadata_src_without_metadata_clears_dst"` — SRC no EXIF/ICC, DST had
    some → out has none (EXIF + ICC both `None`).
  - `"copy_metadata_unsupported_format_errors"` — a PNG (or BMP) as `from` or `to`
    → `MetadataError::UnsupportedFormat`.
- **`tests/metadata.rs` (integration, extend)**
  - `"copy_metadata_to_explicit_output"` — `copy-metadata --from src.jpg --to
    dst.jpg -o out.jpg` → out has src's EXIF; dst unchanged on disk; exit 0.
  - `"copy_metadata_in_place_requires_yes"` — no `-o`, no `-y` → exit **5**; with
    `-y` → exit 0 and dst.jpg now carries src's EXIF.
  - `"copy_metadata_png_exits_4"` — `--to` a PNG → exit 4.
  - `"copy_metadata_preserves_pixels_e2e"` — after an in-place copy, dst's decoded
    pixels are unchanged.
- **`tests/cli.rs`** — `"copy-metadata"` is already in the subcommand-list tests
  (SPEC-007); confirm, add only if missing.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-003` — **governing.** Container lane; copy never touches the pixel encode path.
- `DEC-029` — `img-parts` `=0.4.0` already pinned; this uses its `ImageEXIF`/
  `ImageICC` traits. No new dep, no new DEC beyond DEC-030.
- `DEC-030` — **JPEG-only v1** (PNG EXIF cross-crate chunk mismatch); the probe +
  rationale live here. Mirror its verified snippet.
- `DEC-007` — typed `thiserror`; map `img-parts` errors to `MetadataError::Container`;
  no `unwrap`/`expect`/`panic!` off test paths.
- `DEC-015` — overwrite-guard semantics (exit 5) reused via `Sink::write_bytes` +
  `Overwrite`.

### Constraints that apply

- `metadata-not-via-pixel-encode` (blocking) — no decode/encode of DST pixels;
  asserted via decode-equality.
- `clippy-fmt-clean`, `every-public-fn-tested`, `no-unwrap-on-recoverable-paths`.

### Prior related work

- `SPEC-026` (PR #30) — the lane: `MetadataError`, `sniff`/`Lane`/`file_extension`,
  `Sink::write_bytes`, `read_raw_bytes`, native fixtures. **Reuse them.**
- `SPEC-027` (PR #31) — `set`/`run_set` handler shape; `copy-metadata`'s handler is
  similar but reads two inputs and has a single fixed output (no fan-out).

### Out of scope (for this spec specifically)

- **PNG / WebP / TIFF** copy-metadata (DEC-030 — JPEG only; PNG needs the `eXIf`↔
  `zTXt` bridge, a tracked follow-up).
- **XMP / IPTC** transfer (only EXIF + ICC, what `img-parts` exposes via traits).
- Merging metadata (this REPLACES DST's EXIF/ICC with SRC's, it does not merge).
- `watermark` (pixel-lane; the remaining STAGE-004 spec).

## Notes for the Implementer

- **Mirror DEC-030's probe** — `Jpeg::from_bytes` / `dst.set_exif(src.exif())` /
  `dst.set_icc_profile(src.icc_profile())` / `dst.encoder().write_to(&mut out)`.
  Import `img_parts::{ImageEXIF, ImageICC}` (the trait methods need the traits in
  scope) plus `img_parts::jpeg::Jpeg` and `img_parts::Bytes`.
- **`run_copy_metadata` is NOT a fan-out** — `--from`/`--to` are single paths
  (`std::fs::read` each; map read error → exit 3 like the lane does). Build the output
  `Sink`: `-o PATH`→`Sink::File`, `-o -`→`Sink::Stdout`, else `Sink::File { path: DST
  }`. Overwrite from `-y` (`Overwrite::Allow`/`Forbid`). Call `Sink::write_bytes` with
  the output extension (`metadata_output_ext` on the DST input, or just `"jpg"`).
- **Do not glob** `--from`/`--to`; they are not run through `source::resolve`'s glob
  fan-out — treat each as a literal path (a missing path → exit 3).
- Keep diagnostics on **stderr**; stdout clean for `-o -`.
- The JPEG-only check should produce a message that names the limitation so users
  aren't confused why PNG fails (DEC-030).

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
