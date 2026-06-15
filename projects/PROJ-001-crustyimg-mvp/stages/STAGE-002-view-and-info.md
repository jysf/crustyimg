---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-002                     # stable, zero-padded within the project
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-14
shipped_at: 2026-06-15

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    Turns the STAGE-001 skeleton into something a user actually runs: the
    first real, read-only commands. Proves the source → load → sink path
    end to end (terminal display + structured inspection) without yet
    mutating any pixels.
  delivers:
    - "`view` — display an image in the terminal via viuer, fit-to-terminal by default"
    - "`info` — dimensions, format, byte size, color type, bit depth, alpha, ICC/EXIF presence"
    - "EXIF tag dump (`info --exif`, read-only via kamadak-exif) and machine-readable `info --json`"
  explicitly_does_not:
    - Modify pixels or write any output image (read-only stage)
    - Write or edit metadata (read-only; the write lane is STAGE-004)
    - Implement transforms, watermarking, recipes, or batch
---

# STAGE-002: view and info

## What This Stage Is

The first stage that puts real, user-facing commands on the binary — both
read-only. `view` renders an image directly in the terminal (viuer),
fitting to the terminal by default with optional `--width`/`--height`
sizing, and refuses on a non-tty so pipes stay clean. `info` inspects an
image and reports its dimensions, format, byte size, color type, bit
depth, alpha, and ICC/EXIF presence; `--exif` dumps the EXIF tags (read
only, via `kamadak-exif`) and `--json` emits machine-readable output to
stdout for scripting. Together they exercise the Source → load → Sink path
the foundation laid down, on real images, without touching pixels.

## Why Now

These are the lowest-risk real commands: read-only, no encode, no pixel
mutation, no metadata writes. They validate that the STAGE-001 abstractions
(Source resolution, canonical `Image` load, the display Sink) actually work
on real files before any transform depends on them — and they give the user
something runnable immediately. `info --json` also establishes the
structured-output convention later commands reuse.

## Success Criteria

- `crustyimg view photo.jpg` displays the image in a tty and exits 0; on a
  non-tty it refuses with a clear error (display requires a terminal).
- `--width`/`--height` constrain the rendered size; default fits the terminal.
- `crustyimg info photo.jpg` prints dimensions, format, byte size, color
  type, bit depth, alpha, and ICC/EXIF presence.
- `info --exif` dumps EXIF tags read-only; `info --json` emits valid,
  machine-readable JSON on stdout with all diagnostics on stderr.

## Scope

### In scope
- `view` command: terminal rendering via viuer, tty check, `--width`/`--height`/fit-to-terminal sizing.
- `info` command: core image facts + ICC/EXIF presence detection.
- EXIF read (`--exif`) via `kamadak-exif` (read-only) and `--json` structured output.

### Explicitly out of scope
- Any pixel transform, encode, or output-image write (STAGE-003+).
- Writing/editing metadata (STAGE-004 container lane; this is read-only).
- `compare`/SSIM, histogram, dominant-color (post-MVP, see docs/backlog.md).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-008 (shipped 2026-06-15, PR #8) — `view` command: terminal display via viuer, fit-to-terminal default + `--width`/`--height`, non-tty refusal (exit 5)
- [x] SPEC-009 (shipped 2026-06-15, PR #9) — `info` command: dimensions/format/file-size/color-type/bit-depth/alpha/ICC+EXIF-presence, `--exif` read (kamadak-exif, DEC-013), `--json` structured output

**Count:** 2 shipped / 0 active / 0 pending

## Design Notes

- Pixel library is `image` 0.25 wrapped by the canonical `Image` (DEC-002);
  terminal rendering goes through the display Sink built in STAGE-001
  (viuer 0.9). `view` is the smoke-stub command in STAGE-001 and becomes
  real here.
- EXIF is **read-only** here via `kamadak-exif` (it cannot write); writing
  tags is the container lane in STAGE-004 (DEC-003). Keep `info` on the read
  path only — do not pull in the write crates.
- `--json` sets the structured-output convention: machine output to stdout,
  all human/diagnostic text to stderr (AGENTS.md §11), so `info --json | jq`
  stays clean.

## Dependencies

### Depends on
- STAGE-001 — canonical `Image` + load, Source resolution, display Sink, clap dispatch.
- External: `viuer` (display), `kamadak-exif` (EXIF read).

### Enables
- STAGE-003 — transforms reuse the same load + Sink path on real images.
- Confidence that the foundation works end to end before mutating pixels.

## Stage-Level Reflection

*Filled at ship, 2026-06-15.*

- **Did we deliver the outcome in "What This Stage Is"?** Yes. Both read-only
  commands are real and tested end-to-end on real images: `view` renders via the
  viuer display Sink (fit-to-terminal default, `--width`/`--height`, non-tty
  refusal at exit 5, behind the `display` feature); `info` reports
  dimensions/format/file-size/color-type/bit-depth/alpha/ICC+EXIF presence, dumps
  EXIF read-only (`--exif`, kamadak-exif), and emits machine-readable `--json`
  (stdout-only, `jq`-clean). The Source → load → Sink path is proven on real files
  with no pixel mutation, exactly as scoped. 111 tests green, 3-OS CI.
- **How many specs did it actually take?** 2 (SPEC-008 `view`, SPEC-009 `info`) —
  exactly as planned. SPEC-008 was complexity S, SPEC-009 M.
- **What changed between starting and shipping?** One new top-level dependency
  entered the graph — `kamadak-exif` (DEC-013), added always-on (not feature-gated
  like viuer) because it's pure-Rust and EXIF read is core to `info`. The `--json`
  output also established the structured-output convention (machine→stdout,
  diagnostics→stderr) that STAGE-003+ commands reuse.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Orchestration process lesson (not a doc change): push the design commit to
    `origin/main` BEFORE the build agent branches, or the build PR's squash folds
    the design artifacts in and local `main` needs a reset. Applied from SPEC-009
    onward; kept in orchestrator memory.
  - AGENTS §5 pre-names crate majors; the actual pins live in `Cargo.toml`
    (viuer `=0.11.0`, kamadak-exif `=0.6.1`). Optional polish to sync §5 patches;
    not required.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - The display/feature-gate vs always-on call is now a clear, reusable pattern:
    feature-gate a dep only when its transitive tree is heavy (viuer/DEC-011);
    add pure-Rust lightweight deps always-on (kamadak-exif/DEC-013). Both DECs
    record the reasoning.
  - `--json` hand-rolled emitter worked but is the natural breakpoint: if a second
    command needs JSON, promote `serde_json` to a runtime dep via a small DEC
    rather than hand-rolling a second emitter (noted in SPEC-009 reflection).
