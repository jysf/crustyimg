---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-063                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-011                      # parent project
repo:
  id: crustyimg

created_at: 2026-11-05
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    The predictability half of the thesis: a flag, a printed fix or an error message says what the
    tool will actually do, and the tool does it.
  delivers:
    - "Every `fix:` line `lint` prints, run verbatim, fixes the file; `gps-metadata-leak` needs a coordinate"
    - "No global flag is accepted and then ignored: each is honoured, refused or removed"
    - "Two misleading messages say what is true (an image-less folder, the browser's asset refusal)"
    - "0.8.3: the first release through SPEC-144's tag-driven pipeline, every channel verified"
  explicitly_does_not:
    - "Touch src/operation/, src/recipe/ or anything PROJ-017 owns"
    - "Add verbs, recipe operations or output formats"
---

# STAGE-063: Honest output and the 0.8.3 release

## What This Stage Is

PROJ-011's last stage. It closes the user-facing output that is wrong or misleading, then cuts
0.8.3. Seven backlog items moved here from STAGE-054 on 2026-11-05 (the maintainer approved the
triage); they group into three specs plus the release.

## Why Now

PROJ-011 has 36 unchecked items, accumulated because every spec's verify and ship files its
leftovers. Triage (2026-11-05): these seven are the only ones a user can hit as a wrong or
misleading result without touching PROJ-017's paths. 0.8.3 also exists to prove SPEC-144's
tag-driven pipeline (its AC-18 is open until a real tag), so it should carry real fixes.

## Success Criteria

- Run verbatim, every `fix:` line `lint` prints changes the file and clears its finding.
- `privacy/gps-metadata-leak` flags a file with real coordinates and does not flag a GPS block that
  holds only `GPSVersionID`.
- `--keep-gps`, `-v`/`--verbose` and `responsive --name-template` are each implemented, refused
  with exit 2, or removed with their docs, never accepted and ignored.
- 0.8.3 is published on all channels (crates.io, GitHub Release, Homebrew tap, npm, jysf.org) by a
  tag push, and each channel is checked.

## Scope

### In scope
- The seven items below, grouped as SPEC-148, SPEC-149 and SPEC-150, then the 0.8.3 release cycle.

### Explicitly out of scope
- Every other PROJ-011 item. Dispositions: recipe, watermark and `validate()` items go to PROJ-017
  or PROJ-013; the release-pipeline items wait for 0.8.3; guard and test-gap items stay in
  STAGE-053; STAGE-041 (launch content) is parked.

## Spec Backlog

Spec ids 148 to 150 are reserved by this commit (PROJ-017 holds 147).

- [ ] SPEC-148 (not yet framed) — **`lint` fixes what it names:** the `fix:` stdout item and the
  gps-leak item, framed together (both are `lint` rules). Complexity S.
- [ ] SPEC-149 (not yet framed) — **No flag is accepted and ignored:** `--keep-gps` and `-v`,
  `responsive --name-template`, and the missing-`--font` message. Needs a per-flag ruling
  (implement, refuse or remove); the frame asks the maintainer. Complexity S–M.
- [ ] SPEC-150 (not yet framed) — **Two messages tell the truth:** a folder with only
  subfolders, and the browser's asset refusal plus every user-facing `SPEC-`/`DEC-` id in rendered
  `--help`. Complexity S.
- [ ] (release, not a spec) — **Cut 0.8.3** through the tag-driven pipeline; drive and verify
  every channel. Writes `projects/PROJ-011-surface-reach-and-predictability/specs/prompts/RELEASE-0.8.3.md`.

**Count:** 0 shipped / 0 active / **4 pending** — 2026-11-05.

### Items carried from STAGE-054 (full text, unchanged)

- [ ] (not yet written) — [S] **`lint`'s suggested `fix:` commands write the image to stdout.**
  Filed by the orchestrator (2026-10-30), from the `jysf/crustyimg-example` demo site
  (2026-10-29). `lint` names a runnable fix for each finding, such as `meta clean --gps <f>`,
  `optimize <f>`, `auto-orient <f>` or a `resize`. Run exactly as printed, each writes the
  encoded image to **stdout**, even on a TTY, so the file is not fixed. The form that works is
  `-o <f> -y`. Not yet driven on 0.8.1. Decide whether the printed fix should carry
  `-o <f> -y`, or whether these verbs should refuse to write binary to a TTY. Then test that
  every printed `fix:` line, run verbatim in a shell, changes the file and clears the finding.

- [ ] (not yet written) — [S] **`privacy/gps-metadata-leak` fires on a GPS block with no
  coordinates.** Filed by the orchestrator (2026-10-30), from the `jysf/crustyimg-example` demo
  site (2026-10-29). A Nikon P1100 JPEG writes a GPS IFD holding only `GPSVersionID`, with no
  latitude or longitude, and `lint` flags it as a location leak. Not yet driven on 0.8.1. Decide
  whether the rule keys on coordinate tags (`GPSLatitude`/`GPSLongitude`) rather than on the IFD's
  presence, and keep a positive control: a file with real coordinates must still be flagged.

- [ ] (not yet written) — [S] **A folder that holds only subfolders reports "input not found or
  unreadable".** Filed by the orchestrator (2026-10-30), from the 0.8.1 release-notes verify. A
  directory input reads only its top-level images (documented in the skill since 0.8.1). For
  `lint d/` or `optimize d/` where `d/` has only subfolders, both exit 3 with "input not found
  or unreadable", which is misleading: the folder exists and is readable. Say instead that the
  folder has no images at its top level, and suggest a recursive glob (`'d/**/*'`). Keep exit 3
  unless a ruling changes it.

- [ ] (not yet written) — [S] **`responsive` accepts `--name-template` and silently ignores it.**
  Filed by SPEC-134's design (2026-10-25), driven on `main`: `responsive in.jpg --widths 32
  --out-dir rp --name-template '{stem}.png'` writes `rp/in-32w.jpg`, exit 0 — the flag is a global
  arg (`src/cli/mod.rs:80-82`) that `responsive` never reads. Open: refuse it (exit 2, the flag
  means nothing here) or honour it (it would need a `{w}` token). SPEC-134 makes the template pin
  the format on every other encoding verb, so `responsive` is now the one verb where it does
  nothing.

- [ ] (not yet written) — [S] **`--keep-gps` and `-v`/`--verbose` are read by no verb.** Filed by
  the SPEC-137/138 frame (2026-10-28), from `build`'s global-flag sweep. Both are `GlobalArgs`
  fields (`src/cli/mod.rs:98`, `:110`), but `grep -rn 'keep_gps\|verbose' src` finds them only in
  that struct and two test fixtures that set them (`src/cli/ops.rs:1600-1603`,
  `src/cli/common.rs:454-457`). Driven on `main`: `convert signed_gps.jpg --format jpeg --keep-gps`
  exits 0 and the output has **0 EXIF keys** (the source has `GPSLatitude 50/1,29/1,1113/25`).
  The pixel lane drops all metadata (`metadata-not-via-pixel-encode`), so the flag cannot do what
  `docs/cli-reference.md:21,38`, `README.md:193` and `CHANGELOG.md:711` promise. `-vv` changes no
  output either; no logger is installed, so usvg's own `log::warn!`s are never shown. Released
  versions not driven. Decide per flag: implement it, refuse it, or remove it and its docs. Sibling,
  minor: `build --strict` without `--check` exits 0 and does nothing (its help says "With
  `--check`").

- [ ] (not yet written) — [S] **A missing `--font` reports "could not read image input".**
  Filed by SPEC-135's frame (2026-10-27): `watermark in.png --text hi --font /no/font.ttf` exits 3
  with `error: could not read image input`; the file is a font. The font read maps to
  `ImageError::Io` (`src/cli/ops.rs:1369`). Pre-existing. Name the font in the message.

- [ ] (not yet written) — [S] **The browser's asset refusal shows a spec id to users.** Filed by
  the 0.8.0 release-notes verify (2026-10-27). The wasm `transform` error for a recipe that needs
  an image or font file ends with `… is out of scope (SPEC-128)` (`src/wasm.rs:237`).
  User-facing text carries no spec or DEC ids. Reword it to say what to do instead, and check
  every other user-facing string for the same: `grep -rn 'SPEC-\|DEC-' src/` outside comments.
  ⚠ **"Outside comments" misses `--help`.** clap builds help text from `///` doc comments, and
  `crustyimg --help` shows `build … (SPEC-063, DEC-057)` and `lint … (SPEC-050, DEC-050)` today
  (seen by the orchestrator on 0.7.1, 2026-10-28). Check every verb's `--help` output, not the
  source: loop over the verbs and grep the rendered help for `SPEC-` and `DEC-`.

## Design Notes

Item (1) of SPEC-148 needs a ruling: carry `-o <f> -y` in the printed fix, or make these verbs
refuse to write binary to a TTY. The frame should recommend, not leave it open.

## Dependencies

### Depends on
- SPEC-144 (shipped): the tag-driven release pipeline 0.8.3 will exercise.

### Enables
- The release-pipeline check tightening and crates.io OIDC (STAGE-054), which need a real tag first.
- Closing PROJ-011.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
