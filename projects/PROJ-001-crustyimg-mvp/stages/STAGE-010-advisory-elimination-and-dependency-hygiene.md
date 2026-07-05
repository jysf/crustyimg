---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-010                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-04
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "Trust/quality of the shipped tool — a near-clean `cargo deny` (3 of 4 advisory ignores eliminated at the source; 1 upstream-blocked residual) for a credible 0.2.0."
  delivers:
    - "A 0.2.0 down to a single documented advisory ignore (paste/-2024-0436, upstream-blocked via rav1e/avif) from three"
    - "Removal of the unmaintained `ttf-parser` from the dependency tree (SPEC-044)"
    - "In-house EXIF-tag writer eliminating the `little_exif` → `quick-xml` vulnerabilities (+ `brotli`)"
    - "User-facing `--help` text free of internal stage/DEC jargon"
  explicitly_does_not:
    - "Add new user-facing image capabilities (that is PROJ-002)"
    - "Change the behavior of `watermark`, `set`, or `clean` as observed by users"
    - "Re-open the pure-Rust / permissive-license posture (DEC-004, DEC-018)"
---

# STAGE-010: advisory elimination and dependency hygiene

## What This Stage Is

The post-0.1.0 fast-follow arc that gets crustyimg to a **clean `cargo deny`** — zero
accepted advisory ignores — for a credible **0.2.0**. v0.1.0/v0.1.1 shipped with three
documented, low-risk `deny.toml` exceptions (DEC-042). This stage eliminates each one
**at its source** — swapping or replacing the dependency that drags it in — rather than
suppressing it, and it clears a small pile of UX debt (internal jargon in `--help`) found
during the release smoke-test. When this stage ships, `just deny` is green with no
`[advisories].ignore` entries and the user-facing surface reads for end users, not
maintainers. No new image capability is added; every change is behavior-preserving.

## Why Now

The MVP is shipped and public (v0.1.1 on crates.io, Homebrew, GitHub Releases). The three
advisory ignores were an explicit, agreed compromise to ship 0.1.0 on time (DEC-042), with
the plan to eliminate them as the *first* fast-follow. They are the concrete revisit
triggers DEC-042 recorded. Doing this now — before adding any PROJ-002 surface — keeps the
supply-chain story clean while the dependency footprint is still small and every advisory
is freshly assessed. STAGE-007 (release/distribution) proved the tag→cargo-dist pipeline
across two releases, so cutting a clean 0.2.0 at the end of this stage is low-risk.

## Success Criteria

- `just deny` (`cargo deny check advisories bans sources licenses`) passes with the
  `[advisories].ignore` list down to a **single documented residual** — `RUSTSEC-2024-0436`
  (`paste`, unmaintained build-time proc-macro, reached via `rav1e`→`ravif`→`image`/`avif`;
  no upstream fix, so it can't be eliminated at the source here). The other **three** are
  removed. (Corrected from the original "empty list" goal — see Design Notes: the paste
  residual.)
- `ttf-parser` no longer appears in `cargo tree` (RUSTSEC-2026-0192 eliminated — SPEC-044).
- `little_exif`, `quick-xml`, and `brotli` no longer appear in `cargo tree`
  (RUSTSEC-2026-0194/-0195 eliminated — SPEC-045); `set` / `clean --gps` still round-trip
  correctly on JPEG + PNG.
- `crustyimg <cmd> --help` contains no `STAGE-0XX` / `DEC-0XX` references or stale "stub"
  text.
- Every change is behavior-preserving: `watermark --text`, `set`, and `clean` produce
  output indistinguishable to users from v0.1.1 (rasterizer AA may differ sub-pixel).
- Both the lean (`--no-default-features`) and full builds compile and pass; CI green on
  main after each spec ships.

## Scope

### In scope
- **SPEC-044** — swap `ab_glyph` → `skrifa` + `zeno`, dropping `ttf-parser`; remove the
  `-0192` ignore.
- **SPEC-045 — in-house EXIF-tag writer** — replace `little_exif` with a binary TIFF-IFD
  writer for `set`/`clean --gps` via `img-parts`; removes the `-0194`/`-0195` ignores +
  `quick-xml` + `brotli`. (Does **not** remove `paste`/-2024-0436 — that also comes via
  `rav1e`/`avif`; see Design Notes. DEC-046.)
- **`--help` jargon cleanup** (PATCH or small spec) — strip stage/DEC refs + stale "stub"
  text from the clap doc-comments in `src/cli/mod.rs`.
- Update DEC-042 / `deny.toml` / `docs/backlog.md` as each exception is eliminated.

### Explicitly out of scope
- New image operations or user-facing capabilities (PROJ-002).
- Re-opening the pure-Rust / permissive-license posture (DEC-004, DEC-018) — all
  replacements stay pure-Rust and MIT/Apache.
- Richer typography (multi-line text, alignment, stroke) — a future spec if wanted.
- STAGE-007 #7 (dual lean/full release artifacts) — tracked separately.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-044 (shipped 2026-07-04, PR #49) — swapped `ab_glyph` → `skrifa`+`zeno`;
  dropped `ttf-parser`; removed the RUSTSEC-2026-0192 ignore (DEC-045).
- [x] SPEC-045 (shipped 2026-07-05, PR #50) — in-house TIFF-IFD EXIF writer replacing
  `little_exif`; removed the RUSTSEC-2026-0194/-0195 ignores + `quick-xml`/`brotli` (DEC-046).
  `paste`/-2024-0436 stays (rav1e/avif).
- [ ] (not yet written — PATCH candidate) — `--help` jargon cleanup in `src/cli/mod.rs`.

**Count:** 2 shipped / 0 active / 1 pending — `deny.toml` down to 1 residual ignore (paste).

## Design Notes

- **Eliminate, don't suppress.** Every item removes a `deny.toml` `[advisories].ignore`
  entry by removing the *dependency* that introduces the advisory — never by widening the
  ignore list or downgrading `deny` to `warn` (both rejected in DEC-042).
- **Behavior parity is the bar, not byte-identity.** These are dependency swaps under a
  stable public CLI. A different rasterizer (SPEC-044) or a hand-written IFD serializer
  will not produce byte-identical output; the contract is that users observe no functional
  change (legible text at the same anchor; the same tags set/removed).
- **The fontdue dead-end (recorded for posterity):** the original backlog plan named
  `fontdue` for SPEC-044's job. A design-time probe found fontdue 0.9.3 **still depends on
  `ttf-parser`** (v0.21.1), and RUSTSEC-2026-0192 is crate-wide (`patched = []`,
  `informational = "unmaintained"`) — so fontdue would *not* remove the ignore. The
  advisory's own recommended alternative, `skrifa` (Google `fontations`), is ttf-parser
  free; SPEC-044 retargets to `skrifa` + `zeno`. See DEC-045.
- Each spec pushes its design + DECs to `main` before the build branch, runs the lean +
  full `deny` gates in build and verify, and verifies the git index before the ship commit
  (standing practice).

## Dependencies

### Depends on
- STAGE-004 (shipped) — `watermark --text` (SPEC-030/DEC-032) and the metadata write lane
  (`set`/`clean`, SPEC-026/DEC-029) are the surfaces being re-implemented on new deps.
- STAGE-006 (shipped) — the security assessment that bounded these advisories (DEC-042).
- External: the `fontations` (`skrifa`/`read-fonts`) + `zeno` crates; `img-parts` (already
  a dep) for the EXIF-writer segment surgery.

### Enables
- A clean **0.2.0** release (`just release 0.2.0`) with no advisory exceptions.
- A scheduled `cargo-deny` advisory CI job (retro rec) that can run in `deny` mode with an
  empty ignore list.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
