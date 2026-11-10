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
- A write that fails exits non-zero and leaves the previous file intact; `meta strip` leaves a JPEG
  rendering the same; `build` never overwrites a declared source, never records two outputs for one
  file, and never writes a lockfile its own `--check` refuses (the 2026-11-10 external review,
  SPEC-154 to SPEC-158).
- 0.8.3 is published on all channels (crates.io, GitHub Release, Homebrew tap, npm, jysf.org) by a
  tag push, and each channel is checked.

## Scope

### In scope
- The seven items below, grouped as SPEC-148, SPEC-149 and SPEC-150, then SPEC-154 to SPEC-158
  (the 2026-11-10 external code review: the release is held for them), then the 0.8.3 release cycle.

### Explicitly out of scope
- Every other PROJ-011 item. Dispositions: recipe, watermark and `validate()` items go to PROJ-017
  or PROJ-013; the release-pipeline items wait for 0.8.3; guard and test-gap items stay in
  STAGE-053; STAGE-041 (launch content) is parked.

## Spec Backlog

Spec ids 148 to 150 are reserved by this commit (PROJ-017 holds 147).

- [x] SPEC-148 (shipped 2026-11-06, S; PR #253, bb78fa1) — **`lint` fixes what it names:** the `fix:` stdout item and the
  gps-leak item, framed together (both are `lint` rules). The frame also found both `convert --format`
  fixes malformed (exit 3), unquoted paths, `optimize` ignoring the budget, and a 16-bit PNG fix that
  cannot clear; all in scope. Complexity S.
- [x] SPEC-149 (shipped 2026-11-06, M; PR #256, 302afba) — **No flag is accepted and ignored:** `--keep-gps` and `-v`,
  `responsive --name-template` (and `--format`, found by the frame), `build --strict` alone and `--check`/`--strict`/
  `--no-cache` off `build`, and the missing-`--font` (and `--image`) message. Recommended: refuse `--keep-gps`, a
  minimal `-v` logger, refuse the rest (done as recommended). Complexity M.
- [x] SPEC-150 (shipped 2026-11-07, M; PR #257, b8b2a04e1a7b4f6bea5d85487802ee9837134b28) — **Two messages tell the truth:** a folder with only
  subfolders (driven wider: any folder with no top-level image, 13 verbs, `build` targets too), and the browser's asset refusal plus every user-facing `SPEC-`/`DEC-` id in rendered
  `--help`, including `lint --help`'s "(each with a runnable fix)" line (SPEC-148's design asks SPEC-150 to reword it with its spec ids). Complexity S; framed M with fold-ins B, C
  (docs-only, pending ruling) and leftovers (1), (3), (5) ruled in, (4) out.

Spec ids 154 to 158 are reserved (2026-11-10) for the external code review's findings
(`docs/reviews/2026-11-10-external-code-review.md`, which holds the evidence and the fix each finding
asked for). PROJ-017 holds 153. The review recommended holding 0.8.3 for the four P1 findings; the
release line below waits for SPEC-154 to SPEC-157. Land SPEC-154 first and alone: it decides which
gates later PRs run. Its frame prompt is `specs/prompts/SPEC-154-frame.md`; the other four get theirs
when SPEC-154 is framed. A frame may re-size, split or merge them.

- [ ] **SPEC-154** (reserved 2026-11-10) — [S] **The CI path filter treats real build inputs as docs-only (review P2).**
  `scripts/ci-code-paths.sh` returned `false` for `recipes/web.toml`, `assets/fonts/Go-Regular.ttf`, `clippy.toml` and
  `deny.toml`; the recipes and the font are embedded at compile time (`include_str!`/`include_bytes!`), so an isolated
  edit skips the native build and test legs. `pages.yml`'s path lists lack `recipes/` and `assets/` too. Fix the
  classifier (consider inverting it to a docs-only allowlist, so an unknown path runs everything), add a self-test
  row per class and a mutation row per alternative, and align the Pages triggers. Frame first.
- [ ] **SPEC-155** (reserved 2026-11-10) — [M] **A write that fails must not report success or destroy the old file (review P1).**
  `Sink::File` and `Sink::Dir` truncate the destination, then `BufWriter::write_all` and drop with no `flush()`
  (`src/sink/mod.rs`, both the pixel and the byte sink), so a short write exits 0 and leaves a truncated file. Stage
  the bytes beside the destination, flush and sync, then rename; keep the symlink and overwrite guards ahead of
  it. Test with an injected failing writer. The rename-over-existing behaviour differs on Windows: verify on the
  full OS matrix.
- [ ] **SPEC-156** (reserved 2026-11-10) — [S] **`meta strip` changes how Adobe YCCK JPEGs render (review P1).**
  The JPEG strip loop removes `0xE1..=0xEF`, which includes APP14 (`0xEE`, Adobe), the marker that carries the
  colour transform. On `tests/fixtures/lint/cmyk_16x16.jpg`, `crustyimg diff` scored the stripped file 41.94
  SSIMULACRA2 against the original. Keep APP14; the regression compares decoded pixels as well as checking that
  metadata is gone. The sweeps in `src/metadata/mod.rs` that loop over the full range (tests near lines 630 and
  1712) need the same exception.
- [ ] **SPEC-157** (reserved 2026-11-10) — [M] **`build` destinations have one identity: no overwritten sources, no aliased collisions (review P1 x2).**
  A target with `source = "src/photo.png"` and `out = "src"` replaced the source and exited 0; targets writing
  `dist/result.png` and `alias/../dist/result.png` produced one file and a lockfile with two records. The
  collision key compares path spellings (`output_collision_key`, `check_output_injective`), and the late check
  compares the same strings after the files are written. Resolve destinations once, use that identity for
  planning, the sink and the lockfile, and reject a destination that overlaps any declared source, across
  targets, before writing. Also rule on symlinked output directories (DEC-061 accepts them; the review wrote
  outside the tree through one). Needs a design cycle and a DEC.
- [ ] **SPEC-158** (reserved 2026-11-10) — [S–M] **`build` integrity: a lockfile its own verifier refuses, and a hash of one read with an encode of another (review P2 x2).**
  A 22,000-image build wrote a 5.2 MB lockfile that `build --check` then refused (`LOCK_MAX_BYTES` is 4 MiB and is
  enforced only on read, `src/build/lock.rs`). Give writer and reader one contract, or refuse before publishing.
  And `cache_key_for` reads the file to hash it while `encode_one` reopens the path (`src/cli/common.rs`), so an edit
  between the two stores new pixels under the old key: decode the bytes that were hashed, including on the
  auto-optimize path. Same pass: record the source format before moving the image into the pipeline in `encode_one`
  instead of cloning the decoded pixel buffer. May split into two specs at frame.
- [ ] (not yet written) — [S] **Diagnose the `build_watch` timeouts (review verification note).** On the review's
  macOS arm64 run, 10 of 14 `build_watch` tests timed out waiting for a rebuild; the cause is unexplained. Rerun
  `cargo test --test build_watch` locally and in CI first. A spec only if it is a real bug.
- [ ] (not yet written) — [S] **Strict mode for watermark builds (review, product view).** Out of this stage: the
  watermark and recipe items go to PROJ-017 or PROJ-013. Filed here only so the review's point has an owner: a warm
  build hides the skipped-watermark warning, so "every image is marked or the build fails" is not expressible.
- [ ] (not yet written) — [S] **Verify's optional findings on SPEC-148 (2026-11-05).** (O-1) `optimize --max-size` with an
  unreachable budget exits 0 and rewrites the file larger (159 to 331 B) with a "best effort" note; decide whether
  that is a failure. (O-2) config `max_bytes = 0` prints `optimize --max-size 0`, which exits 2; config validation
  should reject 0. (O-3) FT-1 has no `$` or backtick row, so double-quote POSIX quoting would not be caught
  end to end (FT-4 and the goldens do catch it). (O-4) `common::jpeg_with_gps` is unused and its doc still calls it
  a GPS leak; it is in `tests/common/mod.rs`, so leave it until SPEC-147 merges. (O-5) the lint fixture README does
  not state the CMYK fixture's size (296 B).
- [ ] (release, not a spec) — **Cut 0.8.3** through the tag-driven pipeline; drive and verify
  every channel. Writes `projects/PROJ-011-surface-reach-and-predictability/specs/prompts/RELEASE-0.8.3.md`.
- [ ] (not yet written) — [S] **`optimize` to stdout picks a larger JPEG than `optimize -o x.jpg`.**
  Filed by the SPEC-148 frame (2026-11-05), driven on `main` (0.8.2 debug): the same 2000×1500
  JPEG gives 679,355 B with `optimize in.jpg > a.out` (stderr: `jpeg → jpeg · … (60% smaller)`) and
  459,577 B with `optimize in.jpg -o b.jpg` (no summary line), the same as `-o c.out --format jpeg`.
  Same output format, 48 % larger on the unpinned path, and the summary line prints on one path
  only. Find why the unpinned search lands on a worse JPEG than the pinned encode (`src/cli/optimize.rs`),
  and whether "never ship a larger file" / "smallest format" still holds there.

- [ ] (not yet written) — [S–M] **More global flags are accepted and ignored (finding A).** Filed by the SPEC-149
  frame (2026-11-06), from a driven 20-verb × 13-flag matrix on `main` (0.8.2 debug). Exit 0, output identical to
  not passing the flag: `meta strip|clean|set gps.jpg --format webp -o x.jpg` (writes JPEG) and `-q 5`;
  `resize gps.jpg -o a.jpg --out-dir od` (`od` never created); `info gps.jpg -o x.txt` (no file; also
  `lint`, `diff`, `view` take `-o`/`--out-dir`/`-y`); an unknown template token is written literally
  (`resize --out-dir t --name-template '{stem}-{w}w.{ext}'` → `t/gps-{w}w.jpg`). SPEC-149 builds its refusal as a
  per-verb table; this item extends it. Decide per combination: refuse (exit 2) or document.
- [ ] (→ folded into SPEC-150, frame 2026-11-06) — [S] **`optimize -q` and `web -q` blame flags the user did not pass (finding B).** Filed by
  the SPEC-149 frame (2026-11-06): `optimize gps.jpg -q 10 -o o.jpg` exits 2 with `-q/--quality cannot be combined
  with --target/--ssim/--max-size`, though none was given; the default `AutoQuality::Fast`
  (`src/cli/optimize.rs:837-838`, `:691-693`) trips `reject_quality_with_auto`. Exit 2 may be right; the message
  is not. May fold into SPEC-150.
- [ ] (→ folded into SPEC-150 as docs-only, pending ruling; frame 2026-11-06 drove ICC dropped too, and found DEC-003's
  amendment calling `meta copy` "JPEG and PNG" while the code refuses PNG) — [S] **DEC-003's pixel-lane preserve claim is false (finding C).** Filed by the SPEC-149
  frame (2026-11-06): DEC-003 says pixel-lane encodes keep ICC and copyright/artist and drop only GPS. Driven:
  `resize` of a JPEG with exiftool-written `Artist`, `Copyright` and GPS keeps 0 of them (ICC not driven).
  Either amend DEC-003 (and `AGENTS.md:503`, `SECURITY.md:50`) to "drops all metadata", or build the selective
  preserve, which is also what a real `--keep-gps` would need (`meta copy` restores both today, JPEG/PNG).
- [ ] (→ (1), (3), (5) folded into SPEC-150, frame 2026-11-06; (4) ruled out: clap's message is true and the exit is the same;
  (2) stays here) — [S] **Leftovers from SPEC-149's build and verify (2026-11-06).** (1) a `"` in a path prints
  unescaped in `could not read font file "a"b.ttf": …` (`src/cli/ops.rs`, `watermark_overlay`). (2) FT-1's `meta copy`
  row copies PNG to PNG so it exits 4 even without the flag, so it proves only that the guard runs first; the real
  workaround is covered by `tests/keep_gps_workaround.rs`. (3) the existing message `metadata lane does not support
  meta copy supports JPEG only in v1 yet` is garbled (SPEC-150). (4) `--keep-gps=false` gets clap's
  `unexpected value 'false'` (exit 2) instead of the R6 line; it is still refused. (5) rendered `--help` still carries
  spec/DEC ids for `--no-cache` (SPEC-064), `--check` (SPEC-066), the `--frozen`/`--locked` note and `--strict`
  (DEC-059), `--watch` (SPEC-067), and the `build` (SPEC-063, DEC-057), `lint` (SPEC-050, DEC-050) and `meta`
  (SPEC-087, SPEC-089) descriptions; SPEC-150 owns this sweep.

- [ ] (not yet written) — [S] **`docs/recipes.md:148` mirrors `lint`'s "each with a runnable fix" line.** Filed by the
  SPEC-150 frame (2026-11-06). SPEC-148 made `fix: null` real (stdin, 16-bit PNG, formats the metadata lane cannot write),
  so "each" is false; SPEC-150 rewords `lint --help` but cannot edit `docs/recipes.md`, which PROJ-017 owns. Reword it to
  match after PROJ-017's current spec merges.

- [ ] (not yet written) — [S] **Verify's optional findings on SPEC-150 (2026-11-06).** Seven small items, none blocking:
  (1) user docs still carry ids the SPEC/DEC rule did not cover: `docs/cli-reference.md:305` (STAGE-030) and
  `README.md:125` (`decisions/DEC-052`). (2) the hidden `--keep-gps` flag appears in every completion script (zsh 22
  lines, fish 23, bash 22), same on `main`. (3) `MANIFEST_DROPPED_WARNING` (`src/metadata/mod.rs`) warns as unused on
  wasm32, on `main` too; PR #240 (wasm clippy) may trip on it if it uses `-D warnings`. (4) a path containing `[`
  (folder `a[1]`) is treated as a glob and cannot be escaped (older behaviour). (5) no test sends a `\` path through
  the folder message or uses a `C:\` glob (only `folder_name(r"d\")` is unit-tested), and a single-quoted hint does
  not paste into `cmd.exe`. (6) `quoted()` is ambiguous when a `\` comes right before a `"` (contrived; `"` is
  illegal in Windows filenames). (7) `SourceError` and `CliError` are not `#[non_exhaustive]`, so the new variants
  break downstream exhaustive matches (the CHANGELOG says so); whether to add `#[non_exhaustive]` before 1.0 is a
  maintainer call.

**Count:** 3 shipped / 0 active / **16 pending** — 2026-11-10 (re-derived by grep of the backlog checkboxes above `### Items carried`: 19 in total, 3 ticked, 16 open; the seven new entries are SPEC-154 to SPEC-158 from the external code review, the `build_watch` diagnosis and the watermark strict-mode pointer).

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
