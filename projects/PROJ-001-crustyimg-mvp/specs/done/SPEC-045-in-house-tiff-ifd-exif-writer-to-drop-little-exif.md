---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-045
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-010
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5     # build cycle runs on Sonnet (prescriptive prompt)
  created_at: 2026-07-04

references:
  decisions: [DEC-046, DEC-029, DEC-042, DEC-003, DEC-034, DEC-036, DEC-038]
  constraints: [metadata-not-via-pixel-encode, no-new-top-level-deps-without-decision, pure-rust-codecs-default]
  related_specs: [SPEC-026, SPEC-027, SPEC-044]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-010's <capability>". Optional; null is acceptable.
value_link: "Drops `little_exif` → removes quick-xml (RUSTSEC-2026-0194/-0195) so STAGE-010 can delete two more deny ignores toward a near-clean 0.2.0."

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
      recorded_at: 2026-07-04
      notes: >
        Main-loop orchestrator, not separately metered. Three design-time checks: (1) no
        upstream shortcut — little_exif 0.6.23 latest, still pins vulnerable quick-xml ^0.37
        + paste; (2) corrected the backlog — paste (-2024-0436) also via rav1e/ravif/avif
        and deny is all-features=true, so NOT removable here (maintainer accepted the 1
        residual); (3) probe-validated the writer core — img-parts .exif() returns bare TIFF,
        and a generic IFD parse→recurse-subIFD→re-serialize round-tripped a real JPEG
        (IFD0 + ExifIFD) byte-identical per kamadak-exif. Authored DEC-046, the spec, the
        build prompt (with the probe skeleton embedded).
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 233602
      estimated_usd: 1.27
      duration_minutes: 113
      recorded_at: 2026-07-05
      notes: >
        Real metered subagent on Sonnet. subagent_tokens=233602, duration_ms=6805535.
        estimated_usd at Sonnet list (~$3/$15 per MTok, ~80/20). Wrote src/metadata/tiff.rs
        (bounds-checked, panic-free parser + normalizing LE serializer; HashSet cycle guard +
        MAX_IFD_DEPTH=8; relocates sub-IFDs/out-of-line values/IFD1 thumbnail). Re-implemented
        set_tags/clean_gps on tiff + img-parts; dropped little_exif; deleted -0194/-0195,
        corrected the -2024-0436 comment. Caught + rebased tests/metadata.rs (also imported
        little_exif; not in the spec's file list). 8 new tests + suite green; cargo tree
        little_exif/quick-xml/brotli=0. PR #50.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 85526
      estimated_usd: 0.76
      duration_minutes: 6
      recorded_at: 2026-07-05
      notes: >
        Real metered independent Explore subagent on Opus. subagent_tokens=85526,
        duration_ms=377547. Adversarial hardening audit (integer overflow on type_size*count,
        bounds on every read, depth/cycle guards), round-trip fidelity (sub-IFD + thumbnail +
        GPS-only removal via kamadak-exif), pixel preservation, no scope creep; re-ran all
        gates incl. --features avif. VERDICT PASS, no defects. Orchestrator independently
        confirmed the checked_mul/checked_add arithmetic + tree/tests before merge.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-05
      notes: >
        Main-loop orchestrator: squash-merged PR #50 (cad7be9), ran the ship bookkeeping
        (cost, timeline, STAGE-010 backlog 2/3, archive), confirmed CI green on main. Two
        more deny ignores eliminated (quick-xml -0194/-0195); deny.toml now 3→1.
  totals:
    tokens_total: 319128
    estimated_usd: 2.03
    session_count: 4
---

# SPEC-045: in-house TIFF-IFD EXIF writer to drop little_exif

## Context

Second spec of **STAGE-010** (advisory elimination). `set` and `clean --gps` write EXIF
via **`little_exif`** (DEC-029), which pulls **`quick-xml`** (`RUSTSEC-2026-0194`/`-0195`,
memory-DoS in the XML reader) and **`brotli`** into the tree. crustyimg drives `little_exif`
for **binary EXIF only** (zero XMP/XML in `src`), so the vuln is unreached — but v0.1.x
carries a documented `deny.toml` ignore for it (DEC-042). This spec removes `little_exif`
at the source so those two ignores can be deleted.

**No upstream shortcut** (checked): `little_exif 0.6.23` is the latest and still pins
vulnerable `quick-xml ^0.37` + `paste`. **No drop-in exists** (`kamadak-exif`/`nom-exif`
are read-only; `img-parts` is segment-level). So we write a small **in-house binary
TIFF-IFD reader+writer**. **DEC-046** records the decision and the probe results.

**Scope correction (important):** dropping `little_exif` removes `-0194`/`-0195` (quick-xml
is *only* via `little_exif`) but **NOT `-2024-0436` (`paste`)** — `paste` also arrives via
`rav1e` → `ravif` → `image` (the `avif` feature), and `deny.toml` uses
`[graph] all-features = true`, so that path stays. `-2024-0436` remains a documented
residual ignore (maintainer-accepted). After this spec, `deny.toml` has **1** ignore, not 0.

## Goal

Replace `little_exif` in `src/metadata/mod.rs` with an in-house TIFF-IFD writer that
implements `set` (add/replace IFD0 `Artist`/`Copyright`/`ImageDescription`) and
`clean --gps` (remove the GPS IFD) on the raw TIFF block `img-parts` exposes — preserving
all other tags and the pixels exactly — and delete the `RUSTSEC-2026-0194`/`-0195` ignores.

## Inputs

- **Files to read:**
  - `src/metadata/mod.rs` — the module. `set_tags` (lines ~199–224) and `clean_gps`
    (~141–177) are the `little_exif` users to replace; `strip_all` and `copy_metadata`
    (img-parts only) stay untouched. The `#[cfg(test)]` helpers seed EXIF via `little_exif`
    and must be re-based on the new writer / `img-parts`.
  - `decisions/DEC-046-in-house-tiff-ifd-exif-writer.md` — the decision, probe results, the
    writer design, and the `paste` residual.
  - `decisions/DEC-029-*` (little_exif) and `DEC-003` (container lane) — the amended /
    surrounding decisions.
  - `Cargo.toml` (`little_exif` line ~52) and `deny.toml` (`-0194`/`-0195` block).
- **External crates (already deps):** `img-parts 0.4.0` (`ImageEXIF::exif()` returns the
  raw TIFF `II*\0`/`MM\0*`; `set_exif(Some(Bytes))` re-embeds — JPEG APP1 + PNG `eXIf`).
  `kamadak-exif 0.6.1` (read side + test assertions). TIFF 6.0 / EXIF 2.3 IFD layout.
- **Related code paths:** `src/metadata/`, `src/cli/mod.rs` (`run_set`/`run_clean` — the
  callers; **must not change**).

## Outputs

- **Files modified:**
  - `src/metadata/mod.rs` — add a `tiff` submodule (or sibling file `src/metadata/tiff.rs`):
    a bounded, panic-free IFD parser + a normalizing LE serializer. Re-implement `set_tags`
    and `clean_gps` on it via `img-parts` `exif()`/`set_exif()`. Keep the public fn
    signatures, `TagSet`, and `MetadataError` variants **identical**. Re-base the test
    helpers off the new writer / `img-parts` (no `little_exif`).
  - `Cargo.toml` — remove `little_exif = "=0.6.23"` (+ update its comment).
  - `deny.toml` — delete the `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` entries + comment.
    **Keep** `RUSTSEC-2024-0436` (paste, residual via rav1e) — and tighten its comment to
    drop the now-false "via little_exif" clause (paste is now only via rav1e/avif).
- **New exports:** none outside `src/metadata` (the `tiff` submodule is crate-internal).
- **Database changes:** none.

## Acceptance Criteria

- [x] `cargo tree` shows **no `little_exif`**, **no `quick-xml`**, **no `brotli`**.
- [x] `deny.toml` has no `-0194`/`-0195`; `just deny` passes (with `-2024-0436` still
      present and its comment corrected). `cargo tree -i paste` still shows the `rav1e` path.
- [x] `set` adds `Artist`/`Copyright`/`ImageDescription` to IFD0 and **preserves** every
      other tag — incl. an **ExifIFD sub-tag** and an **IFD1 thumbnail** — verified by
      `kamadak-exif` reading the output.
- [x] `clean --gps` removes **all** GPS tags and **preserves** every non-GPS tag; a file
      with no EXIF is a byte-identical no-op; a file with no GPS is unchanged (non-GPS tags
      intact).
- [x] Both `set` and `clean` **preserve pixels exactly** (decode-equality, as the existing
      tests assert) for JPEG **and** PNG.
- [x] **Malformed/truncated/cyclic EXIF** yields a typed `MetadataError`, **never a panic**
      (parser is fully bounds-checked; recursion-depth capped).
- [x] Public API unchanged (`set_tags`, `clean_gps`, `TagSet`, `MetadataError`);
      `run_set`/`run_clean` in `src/cli/mod.rs` compile unchanged. All existing metadata
      tests pass (helpers re-based off `little_exif`); lean build + clippy + fmt clean.

## Failing Tests

Written during **design**, BEFORE build. Add to `src/metadata/mod.rs` tests (adapt existing
helpers to seed via the new writer / `img-parts`, not `little_exif`).

- **`src/metadata/mod.rs` (tests)**
  - `"set_preserves_exififd_subtag"` — seed a JPEG whose EXIF has an IFD0 tag **and** an
    ExifIFD sub-tag (e.g. `ExposureTime`); `set_tags(.. artist ..)`; read back with
    `kamadak-exif`: `Artist` present **and** the ExifIFD sub-tag still present with the same
    value. (Locks sub-IFD preservation — the probe-proven core.)
  - `"set_preserves_ifd1_thumbnail"` — seed a JPEG with an IFD1 thumbnail; `set_tags`; the
    output still contains a readable thumbnail (IFD1 `JPEGInterchangeFormat` blob intact).
  - `"set_overwrites_existing_tag"` — set `Copyright` twice with different values; only the
    latest survives (no duplicate IFD0 entry).
  - `"set_on_no_exif_creates_minimal"` — `set_tags` on a JPEG **and** a PNG with no EXIF
    produces a file whose EXIF reads back exactly the set tag(s).
  - `"clean_gps_removes_only_gps"` — seed EXIF with GPS + non-GPS (Orientation/Copyright);
    `clean_gps`; GPS tags gone, non-GPS tags preserved (JPEG + PNG).
  - `"clean_gps_no_exif_is_noop"` — `clean_gps` on a no-EXIF file returns the input bytes
    unchanged.
  - `"set_and_clean_preserve_pixels"` — decode the output and assert pixel-equality with the
    input decode, for JPEG + PNG (extends the existing decode-equality tests).
  - `"malformed_exif_errors_not_panics"` — feed the writer a truncated/garbage TIFF block
    (bad IFD offset, out-of-bounds value offset, self-referential sub-IFD pointer) and
    assert a `MetadataError` is returned (the test must not panic/abort).

> The dependency outcome — no `little_exif`/`quick-xml`/`brotli`, `-0194`/`-0195` deleted,
> `just deny` green — is a **gate** (verified via `cargo tree` + `just deny`), not a unit
> test.

## Implementation Context

*Read this section (and DEC-046) before building.*

### The writer (probe-validated approach — build on this)

A design probe implemented a minimal generic rewriter and round-tripped a real JPEG
(IFD0 strings + an ExifIFD `ExposureTime`) **byte-identical** per `kamadak-exif`. Model:

```
Entry { tag: u16, ty: u16, count: u32, value: Vec<u8>, sub: Option<Ifd> }
Ifd   { entries: Vec<Entry> }         // + optional next-IFD (IFD1)
```

- **Parse** (bounds-checked; every index guarded, return `MetadataError::Exif` on any OOB):
  header = `II`/`MM` + magic `42` + IFD0 offset. For each 12-byte entry read
  `(tag, ty, count)`; `vlen = type_size(ty) * count`; value is inline (`vlen ≤ 4`, in the
  entry's value field) or at the 4-byte offset. For the **pointer tags** `ExifIFD 0x8769`,
  `GPS 0x8825`, `Interop 0xA005`, the value is a LONG offset → recurse into a sub-IFD
  (**cap recursion depth**, e.g. ≤ 8, to kill cycles). Follow IFD0's next-IFD link to
  **IFD1** (thumbnail); in IFD1, treat `JPEGInterchangeFormat 0x0201` (offset) + `0x0202`
  (length) as a relocatable **thumbnail blob**.
- **Edit**: `set` → in IFD0, replace the entry for each target tag or append a new ASCII
  (type 2) entry whose value is the UTF-8 bytes + a NUL terminator (`count = len+1`);
  `clean_gps` → drop the IFD0 `0x8825` entry (do not emit the GPS sub-IFD).
- **Serialize** normalized **little-endian**: emit header, then IFD0 directory, then
  sub-IFDs and out-of-line values appended after, patching each entry's 4-byte
  offset field; keep offsets even (pad odd blobs). Emit IFD1 + relocate its thumbnail blob.
  (The probe's `ser()` is a working skeleton — extend it with IFD1/thumbnail + the edits.)
- **Embed**: `let mut j = Jpeg::from_bytes(..)?; j.set_exif(Some(Bytes::from(tiff)));
  j.encoder().write_to(&mut out)?;` — and the `Png` equivalent (native `eXIf`). `set_exif`
  takes the bare TIFF (no `Exif\0\0`). No-EXIF `set`: build a minimal TIFF (header + IFD0
  with just the target tags). No-EXIF `clean`: return input unchanged.

### Decisions that apply
- `DEC-046` — this writer; drop `little_exif`; remove `-0194`/`-0195`; keep `-2024-0436`.
- `DEC-029` — **amended**: the `little_exif` write choice is retired; the read side
  (`kamadak-exif`) and container-lane split stay.
- `DEC-003` — container lane: edit raw container bytes, never re-encode pixels.
- `DEC-034`/`036`/`038` — input hardening: the parser must be bounds-checked and
  panic-free on untrusted/malformed EXIF (no `unwrap`/`expect`/index-panic on the byte path).

### Constraints that apply
- `metadata-not-via-pixel-encode` — the pixels/compressed scan pass through `img-parts`
  verbatim; only the EXIF block changes.
- `no-new-top-level-deps-without-decision` — this **removes** a dep (net −1) and adds none.
- `pure-rust-codecs-default` — no new codec; pure-Rust.

### Prior related work
- `SPEC-026`/`SPEC-027` (shipped) — `strip`/`clean`/`set`; the behavior contract to
  preserve.
- `SPEC-044` (shipped) — the prior STAGE-010 advisory elimination; same push-design-first /
  lean-build / deny-in-verify discipline applies.

### Out of scope (for this spec)
- Removing `-2024-0436` (paste) — impossible here (rav1e/avif path); stays documented.
- XMP/IPTC/ICC editing, or writing tags beyond IFD0 strings + GPS removal — extend later.
- `copy_metadata` and `strip_all` — untouched (img-parts only).
- PNG `copy-metadata` (still deferred, DEC-030).
- Big/little-endian *output* choice — always emit LE (matches prior behavior).

## Notes for the Implementer

- **Behavior parity via `kamadak-exif`, not byte-compare.** Our TIFF bytes won't match
  `little_exif`'s; assert semantics (tags present/absent, values, thumbnail readable).
- **Hardening is a first-class requirement**, not a nice-to-have: the EXIF block is
  attacker-influenced. Every offset/length read is bounds-checked; sub-IFD recursion is
  depth-capped; a bad block returns `MetadataError::Exif(..)`. The
  `malformed_exif_errors_not_panics` test is the gate for this.
- Re-base the test helpers (`jpeg_with_exif`, the PNG variants) to seed EXIF **without**
  `little_exif` — build the seed TIFF with the new writer, or hand-assemble a tiny TIFF and
  embed via `img-parts`. This is required (the crate is gone).
- Run **both** builds + full deny: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --check`, `cargo build --no-default-features`,
  `cargo deny check advisories bans sources licenses` (green with `-0194`/`-0195` removed),
  and confirm `cargo tree` has no `little_exif`/`quick-xml`/`brotli`. Commit `Cargo.lock`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-045-exif-writer`
- **PR (if applicable):** (opened against `main`; see PR URL in the build report)
- **All acceptance criteria met?** yes
- **New decisions emitted:** none (DEC-046 from design covers this build)
- **Deviations from spec:**
  - `tests/metadata.rs` (a black-box CLI integration test file) also seeded EXIF via
    `little_exif` and was not named in the spec's file list (`## Inputs` mentions only
    `src/metadata/mod.rs`). Re-based it the same way: hand-assembled little-endian TIFF
    bytes embedded via `img-parts` `set_exif`, read back via `kamadak-exif`. Necessary —
    the crate is gone, so this file would not compile otherwise.
  - Added one extra test beyond the spec's 8 (`set_overwrites_existing_tag_no_duplicate`)
    that asserts "no duplicate IFD0 entry" directly on the parsed `tiff::Ifd` (the spec's
    `set_overwrites_existing_tag` only required the *value* to be the latest; the no-
    duplicate half is called out explicitly in the spec's prose, so it gets its own
    assertion rather than being implicit in a value check) and one PNG variant of
    `clean_gps_removes_only_gps` (the spec's acceptance criteria explicitly ask for GPS
    removal to be verified "incl... for JPEG and PNG" pixel preservation; a PNG GPS-removal
    case was added alongside for parity).
- **Follow-up work identified:**
  - None — the writer's scope (IFD0 ASCII tags + GPS removal, sub-IFD/thumbnail
    preservation) fully covers `set`/`clean --gps`'s current behavior. Future tag types
    (beyond IFD0 strings) or IFD1 write support would need a new spec per DEC-046's
    "Revisit if" clause.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec's `## Inputs` file list named only `src/metadata/mod.rs` as needing
   `little_exif` test-helper rebasing, but `tests/metadata.rs` (a separate integration-test
   crate target) also imported `little_exif` directly and would not compile once the dep
   was removed. A `grep -rn little_exif` across the whole tree up front (not just the
   spec-named files) would have caught this before writing any code.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Not really a gap in DEC-046/the spec itself — the probe-validated skeleton (parse →
   recurse sub-IFD → serialize) was accurate and the design's IFD1/thumbnail extension
   guidance was sufficient to implement without surprises. The one wrinkle (kamadak-exif's
   `Tag` being `(Context, u16)`, so GPS tag 0x0001 and a hypothetical generic 0x0001 are
   distinct `Tag`s) is a kamadak-exif API detail, not something the spec needed to flag.

3. **If you did this task again, what would you do differently?**
   — Grep the whole repo (not just the spec's named files) for the crate being removed
   before starting, to size the full rebasing surface up front instead of discovering the
   second test file mid-build.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — The two highest-leverage moves both came from *checking the real state before trusting
   the plan*: (a) confirming little_exif has no fixed release (so in-house was truly
   required), and (b) probing the **full feature graph** (`deny` is `all-features=true`),
   which revealed paste survives via rav1e regardless — a premise the backlog got wrong,
   same as the fontdue case. Generalized rule for advisory work: run `cargo tree -i <dep>`
   AND check the `[graph]` feature config before claiming a "drops dep X" outcome. The build
   agent's catch of `tests/metadata.rs` (a little_exif user not in my file list) also argues
   for grepping the *whole repo* for a dep before writing the spec's file list.

2. **Does any template, constraint, or decision need updating?**
   — DEC-029's write-side choice is now amended by DEC-046 (recorded in both). No template
   change. Worth a possible constraint someday: "an advisory ignore may only be *claimed*
   removable after verifying the dep has no other path in the all-features graph" — the
   recurring lesson of this stage. `--help` still leaks `(STAGE-004)` etc. — the STAGE-010
   jargon-cleanup PATCH remains.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. STAGE-010's remaining item is the `--help` jargon cleanup (a PATCH, not a
   spec). After that, 0.2.0 is ready to cut (`just release 0.2.0`) with a single documented
   `paste` residual ignore.
