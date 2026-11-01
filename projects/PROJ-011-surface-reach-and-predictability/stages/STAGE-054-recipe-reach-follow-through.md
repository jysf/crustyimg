---
stage:
  id: STAGE-054
  status: active
  priority: high
  target_complete: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-10-27
shipped_at: null

value_contribution:
  advances: >
    The predictability half of the thesis: what STAGE-050 made a recipe able to
    express now behaves the same on every path, and says so when it cannot.
  delivers:
    - "A `watermark` size or scale that is not a sensible number is refused on the CLI, as it already is in a recipe"
    - "A cached `build` reports the same per-input warnings as the build that produced it"
    - "One rule for how per-input lines name their input, and for which verbs honour `--name-template`"
  explicitly_does_not:
    - "Add new verbs or new recipe operations"
    - "Change which candidate `optimize` picks"
---

# STAGE-054: Recipe Reach Follow-Through

## What This Stage Is

The follow-up work STAGE-050 turned up while it shipped. A recipe can now watermark, pin a
format and validate its parameters; this stage closes the gaps around that: the CLI path that
still draws a nonsense mark at exit 0, a cached `build` that forgets its warnings, the value
half of the watermark ruling (`--min-size`), and the verbs and lines that still disagree with
each other. Every item was filed by a STAGE-050 spec and moved here at its Stage Ship
(2026-10-27).

## Why Now

STAGE-050's Stage Ship judged its criterion 4 ("a bad watermark is refused or skipped, never
silently dropped") only **partly** met: `watermark --size nan` still writes an unmarked image at
exit 0. 0.8.0 headlines that rule, so the maintainer ruled (2026-10-27) that the first item
ships **before 0.8.0**. The rest has no release deadline and must not hold the tag.

**0.8.0 released 2026-10-28** (`v0.8.0` → `139b381`). It carried SPEC-135 (this stage's first
item) and, at the maintainer's call, resvg 0.49 (PR #211, verified with a CHANGELOG-only punch
list). All five surfaces show 0.8.0: crates.io, the GitHub Release (15 assets), the Homebrew
tap, npm `crustyimg-wasm`, and jysf.org. Main's CI on the tagged commit passed all 13 jobs. The
remaining items here are 0.8.1 candidates.

## Success Criteria

- `watermark --size nan|inf` and `--scale nan|inf` are refused with a plain message, on one
  input and in a batch, before any input is read; a size or scale the CLI accepts, a recipe
  accepts too. (Narrowed 2026-10-27, maintainer: marks that draw nothing on *both* paths, such as
  `opacity = 0` or blank text, are a separate backlog item for 0.8.1.)
- A text mark's `size` has an upper bound, the same on the CLI and in a recipe.
- A second `build` over unchanged inputs prints the warnings the first one printed.

## Scope

### In scope
- The items in the Spec Backlog below.

### Explicitly out of scope
- SPEC-133's optional test gaps: moved to STAGE-053 (guards, not behaviour).
- Anything in PROJ-013. It is `proposed`, so `just backlog` cannot see it (HANDOFF trap 8).

## Spec Backlog

Ordered. ⚡ Only the first item gates 0.8.0.

- [x] **SPEC-135** (merged 2026-10-27, PR #207) — [S] ⚡ **Before 0.8.0: a `watermark` size or scale that is not a
  sensible number is refused, never silently drawn as nothing.** One spec for two filings that
  share the same validation code on the same verb (maintainer ruling 2026-10-27, after STAGE-050's
  Stage Ship judged criterion 4 only *partly* met because of the first one). Scope: refuse
  non-finite `--size` and `--scale` on the CLI verb (the recipe path already does, SPEC-131); put
  an upper bound on a text mark's `size`, on the CLI and in the recipe schema; and replace
  `--scale inf`'s `4294967295×4294967295 px mark` message. The two filings, moved verbatim:
  - [S] ⚡ **The `watermark` CLI verb silently drops the mark for
    `--size nan` / `--size inf`, and accepts `--scale nan`.** Filed by SPEC-131's verify
    (2026-10-26), driven on release builds before and after SPEC-131 (identical): `watermark
    --size nan` exits 0 and writes output byte-equal to `--opacity 0`. The CLI twin of what SPEC-131
    fixed for recipes; the CLI builds the op with `new_image`/`new_text`, bypassing the recipe
    schema. ⚠ **Check against SPEC-133 first:** its fit test may already refuse a non-finite mark
    size; if so, this closes with SPEC-133 and needs only a test row.
    **Checked 2026-10-26 (SPEC-133 build + verify): NOT closed.** `--size nan`, `--size inf` and
    `--scale nan` still exit 0 (the mark renders tiny and fits); only `--scale inf` is now refused,
    and its message reads `the 4294967295×4294967295 px mark` — fix that wording here too.
  - [S] **No upper bound on a text watermark's `size`.** Filed by SPEC-131's
    design (2026-10-24): `size = 100000.0` in a recipe is accepted and took 52 s of wall time over 3
    small inputs at exit 0 (the text raster is enormous); `watermark --size` shares it. A
    resource-limit question in DEC-036's family, not a malformed-recipe one, so SPEC-131 refuses
    only `size <= 0` and non-finite values. Open: the cap, and whether it lives in the schema.

- [ ] (not yet written) — [S] **A cached `build` replays no per-input warnings.** Filed by
  SPEC-133's design (2026-10-25, Call 4). A cache hit writes the stored bytes and returns before the
  pipeline (`src/cli/build.rs:531-540`), so three warnings print on the build that produced an
  output and never on a cached re-run: SPEC-133's "watermark skipped, image written without it",
  DEC-085's truncated JPEG, and SPEC-119's animated input (the last two only on the Decide plan's
  rebuild path today — the Preserve/Pinned plans never print them at all, `encode_one` drops the
  flags). The output stays correct and locked; what is lost is the *report*: a second `build`
  says `(N cached, 0 rebuilt)` over images that are unmarked or truncated. Fix shape: the cache
  entry carries the output's warnings (frame field after `ext`, `src/build/cache.rs:455-503`),
  `CACHE_SCHEMA_VERSION` 1 → 2 + new magic, and `build_one` replays them on a hit. A schema bump
  is only a one-time rebuild, so this is not tied to 0.8.0's lockfile migration. SPEC-133 ships
  the limitation documented (`docs/api-contract.md`, `build`) with a test pinning it, which this
  item flips on purpose.

- [ ] (not yet written) — [S] **`watermark --min-size`: the opt-in value rule, settable in a
  recipe.** Split out of SPEC-133 by its frame (2026-10-24); the orchestrator confirms the split.
  It is the second half of the 2026-08-23 ruling above ("Value rule — opt-in `--min-size`,
  settable in a recipe … only the user knows what is worth protecting"), which is settled; it is
  not a new idea. Why separate: it adds a CLI flag on `watermark` and a recipe key on the step, so
  it couples to SPEC-131's strict param schema (land it after SPEC-131 and the key goes straight
  into the schema), and it needs its own single-input rule, which the ruling does not state (below
  the floor on one explicit input: error like the damage rule, or write unmarked?). It reuses
  SPEC-133's batch *skipped* outcome, so build it after SPEC-133. Open for its spec: what "size"
  means (shorter edge? longer edge? area?) and whether `apply --recipe` gets a CLI override.

- [ ] (not yet written) — [S] **`responsive` accepts `--name-template` and silently ignores it.**
  Filed by SPEC-134's design (2026-10-25), driven on `main`: `responsive in.jpg --widths 32
  --out-dir rp --name-template '{stem}.png'` writes `rp/in-32w.jpg`, exit 0 — the flag is a global
  arg (`src/cli/mod.rs:80-82`) that `responsive` never reads. Open: refuse it (exit 2, the flag
  means nothing here) or honour it (it would need a `{w}` token). SPEC-134 makes the template pin
  the format on every other encoding verb, so `responsive` is now the one verb where it does
  nothing.

- [ ] (not yet written) — [S] **`watermark` prints absolute paths in batch lines; `apply` prints
  relative ones.** Filed by SPEC-133's re-verify (2026-10-26): `watermark two/` labels its skip
  warnings with absolute paths, `apply --recipe … two/` with the path as given. Pre-existing
  (identical at `3dd23ff`; `src/cli/ops.rs` unchanged by SPEC-133). Pick one labelling rule for
  every per-input line and use it in `run_pixel_op` too; check the error lines as well as the
  skip lines.
  The 0.8.0 release-notes session (2026-10-27) added that the same warning prints an absolute
  path for a directory input but a relative one for a glob, on `watermark` alone.

- [ ] (not yet written) — [S] **wasm `validate(recipe_toml)` export returning structured,
  per-step recipe errors.** Filed by SPEC-131's design (2026-10-24). Roadmap row #6's (recipe
  playground) prerequisite: no wasm export checks a recipe without an image today, and
  `transform` decodes its input before it builds the pipeline. Consumes SPEC-131's
  `OperationRegistry::param_specs` and its `recipe step N ('op'): …` errors; the natural home for
  "did you mean …?" key suggestions, which SPEC-131 declined.

- [ ] (not yet written) — [S] **Encoding verbs resolve their inputs twice.** Filed by SPEC-134's
  re-verify (2026-10-26). To decide the format pin before branching, `web`, `optimize` and
  `apply --recipe` call `resolved_input_count` (`src/cli/common.rs`), which runs `source::resolve`
  on every argument; the chosen branch (`run_pixel_op` or `run_optimize_autodecide`) then resolves
  the same `&[String]` again. Same function both times and stdin is never drained by the count, so
  it is correct today; the cost is a second directory walk and two passes that could drift.
  Refactor: resolve once and pass `&[Input]` into both branches. Code quality, not a bug; a
  candidate for the maintainer's queued Rust cleanup.

- [x] **SPEC-143** (merged 2026-10-31, PR #235) — [S] **A watermark with no visible pixels is drawn as nothing at exit 0.**
  Filed by SPEC-135's frame (2026-10-27), driven on a release build of `main` @ `4d6808b`: on
  **both** paths, `--text " "` / `text = " "` and `--size 0.01` / `size = 0.01` exit 0 with 0
  pixels changed, and `--opacity 0` / `opacity = 0.0` the same. Cause for the first two:
  `render_text` returns a 1×1 transparent canvas when no glyph is drawable
  (`src/text/mod.rs:194-197`), and nothing refuses a mark whose visible-pixel count (already
  computed at construction, `visible_pixels`) is 0. The same fallback is in the `v0.7.1` tag (not driven on the released binary). Fix shape:
  refuse a mark with 0 visible pixels at construction (invocation-level: exit 2 CLI, exit 1
  recipe). **Ruled 2026-10-30 (maintainer): `opacity = 0` is refused too**, with the same exit
  codes; to turn a watermark off, remove the step. Out of SPEC-135
  because both paths agree; it is what STAGE-054's first success criterion literally promises
  ("no CLI or recipe path draws an invisible mark at exit 0").

- [ ] (not yet written) — [S] **A text watermark's raster has no memory bound beyond `size`.**
  Filed by SPEC-135's frame (2026-10-27). Peak RSS grows with text length × size²: for an 11-char
  mark, 106 MiB at size 1024, 1,031 MiB at 4096, 5,887 MiB at 10 000, paid before any input is
  read. SPEC-135 caps `size` (one glyph); `--text` has no length limit on the CLI, and a recipe's
  text is bounded only by DEC-036's 64 KiB. Fix shape: bound the canvas area (or the string
  length) before allocating, in DEC-034/DEC-063's resource-limit family.

- [ ] (not yet written) — [S] **A missing `--font` reports "could not read image input".**
  Filed by SPEC-135's frame (2026-10-27): `watermark in.png --text hi --font /no/font.ttf` exits 3
  with `error: could not read image input`; the file is a font. The font read maps to
  `ImageError::Io` (`src/cli/ops.rs:1369`). Pre-existing. Name the font in the message.

- [ ] (not yet written) — [S] **`text::render_text` panics for a library caller that passes a huge
  size.** Filed by SPEC-135's design (2026-10-27; maintainer ruling, filed by the orchestrator).
  On `main`, `--text "© crustyimg" --size 100000` panics in `zeno` (`raster.rs:144`, exit 101),
  and so does released 0.7.1 (driven on the release asset). SPEC-135 caps `size <= 1024` on the
  CLI and in recipes (and so in wasm) before `render_text` runs, but the library function itself
  still has no bound: a direct caller of `Watermark::new_text` or `render_text` can still crash
  the process. Fix shape: refuse (return an error) inside `render_text` / `new_text` for a size
  past the cap, so the library cannot panic whichever caller reaches it
  (`untrusted-input-hardening`).

- [ ] (not yet written) — [S] **SPEC-135's two optional cleanups: DEC-106's account of the old
  cast, and the AC-4 table test stopping at its first failing row.** Filed by SPEC-135's
  re-verify (2026-10-27). (1) DEC-106 says the old cast "turned NaN and −inf into a 1×1 mark".
  The old code computed the height as `round(oh/ow).max(1)`, so a tall overlay became 1×N, not
  1×1; make the sentence accurate. (2) The AC-4 table test in `tests/watermark_cli_params.rs`
  stops at its first failing row. Collect every row's failure and assert once, so a mutation that
  breaks several rows shows all of them. Docs and test only, no behaviour change.

- [ ] (not yet written) — [S] **A recipe's `format = "bogus"` is refused as an "extension".**
  Filed by the 0.8.0 release-notes session (2026-10-27): it exits 4 with
  `unsupported output extension: bogus`, but the user wrote a recipe key, not a file extension.
  Name the key and the step, as other recipe load errors do (DEC-102), and check whether it
  should be refused at load (exit 1) instead of at encode (exit 4).

- [ ] (not yet written) — [S] **A recipe's `quality = 101` is accepted and treated as 100.**
  Filed by the 0.8.0 release-notes session (2026-10-27), consistent with `-q 101` on the CLI.
  `recipes/README.md` now documents 0–100, so the schema accepts a value the guide calls out of
  range. Decide: refuse it at load, as every other ranged recipe key is refused (DEC-102), and
  `-q` too; or document the clamp.

- [x] **SPEC-138** (merged 2026-10-30, PR #221) — [S] **`build --format` is accepted and silently ignored.**
  Filed by the 0.8.0 release-notes verify (2026-10-27), driven on `main`'s release binary:
  `build --format png` exits 0 and writes the target's own format (GIF, WebP or JPEG), with or
  without a name extension or a recipe `format`. `--format` is a **global** flag
  (`src/cli/mod.rs:88-90`), so `build --help` lists it as "Force output format", but
  `src/cli/build.rs` never reads it. `apply --recipe` honours it. Decide: make `build` honour it,
  above the name and the recipe's `format`, as `apply` does (the cache key must include it); or
  refuse it on `build` with exit 2. Then correct the docs, which 0.8.0 words as "on `build`, the
  target's name or the recipe's `format` sets the format". Check whether 0.7.1 ignores it too.

- [ ] (not yet written) — [S] **The browser's asset refusal shows a spec id to users.** Filed by
  the 0.8.0 release-notes verify (2026-10-27). The wasm `transform` error for a recipe that needs
  an image or font file ends with `… is out of scope (SPEC-128)` (`src/wasm.rs:237`).
  User-facing text carries no spec or DEC ids. Reword it to say what to do instead, and check
  every other user-facing string for the same: `grep -rn 'SPEC-\|DEC-' src/` outside comments.
  ⚠ **"Outside comments" misses `--help`.** clap builds help text from `///` doc comments, and
  `crustyimg --help` shows `build … (SPEC-063, DEC-057)` and `lint … (SPEC-050, DEC-050)` today
  (seen by the orchestrator on 0.7.1, 2026-10-28). Check every verb's `--help` output, not the
  source: loop over the verbs and grep the rendered help for `SPEC-` and `DEC-`.

- [x] **SPEC-137** (merged 2026-10-29, PR #217) — [S] **An SVG font family other than Go draws no text.** Filed by the
  PR #211 (resvg 0.49) verify, 2026-10-28. It's the same on 0.7.1, so #211 didn't cause it.
  `<text font-family="sans-serif">`, `serif`, `Helvetica` or `"Helvetica, sans-serif"` renders
  **zero ink**, exits 0, and prints no warning. Only no family, `Go`, or a list that contains
  `Go` renders. The bundled-font-only rule (no system fonts) holds: installed Helvetica and
  Zapfino are not read. Most real SVGs name a family, so their text silently vanishes. Likely
  fix: point `fontdb`'s generic families (`sans-serif`, `serif`, `monospace`, …) at Go in
  `hardened_options()` (`src/image/svg.rs`). If an unknown named family still draws nothing
  after that, fall back to Go too. Test with an independent check that ink is present. Keep a
  negative control proving no system font is read.

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

- [ ] (not yet written) — [S] **SPEC-138's verify cleanups, and `build --help` still lists the four
  refused flags.** Filed by the SPEC-138 ship (2026-10-30), one entry for what verify found.
  (1) FT-13 should assert its snapshot diff names a `.crustyimg/` path: today `snapshot()` could
  skip dotfiles and no test would notice a write there (mutation M-d, a gap; low risk, since a
  build that slips past the guard also writes its outputs). (2) `snapshot()` records files only,
  so a new empty directory is invisible to it. (3) The DEC-105 rule-2 line and the
  `docs/api-contract.md:626` line run past the wrap width. (4) `build --help` still lists
  `--format`, `-o`, `--out-dir` and `--name-template` although `build` now exits 2 on them; decide
  whether a `build`-specific help note is worth the clap work, or leave the docs as the answer.
  Tests and docs only, apart from (4).

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

- [ ] (not yet written) — [S] **A library caller of `Watermark::new_image` / `new_text` can still
  build a mark that draws nothing.** Filed by SPEC-143's frame (2026-10-30). SPEC-143 refuses a
  mark with no visible pixel after opacity on the CLI and in `from_params` (recipes, `build`, wasm).
  But the two public constructors are infallible (`-> Self`) in released 0.8.1
  (`src/operation/mod.rs:1442`, `:1470`), and `Refusal` is not `#[non_exhaustive]`. So a direct
  library caller that skips SPEC-143's predicate still composites nothing with `Ok`. Closing it
  changes the public API (fallible constructors, or a new `Refusal` variant that `apply` returns),
  so it waits for the next minor (0.9), not a patch. The same shape as the `render_text`
  library-panic entry above (`untrusted-input-hardening`).

- [ ] (not yet written) — [S] **No CI job runs the wasm round-trip tests.** Filed by SPEC-143's
  verify (2026-10-30). `tests/wasm_roundtrip.rs` (47 tests, including SPEC-143's FT-12) runs only
  under `just wasm-test`, which needs `wasm-bindgen-test-runner`; no workflow in
  `.github/workflows/` invokes it. So a wasm-only regression merges on green CI. Add a CI job that
  installs the pinned `wasm-bindgen-cli` and runs `just wasm-test` (and `wasm-test-slim`), gated on
  the code-paths filter like the native legs. Measure its runtime before making it required.

- [ ] (not yet written) — [S] **SPEC-143's verify leftovers: a boundary golden row that does not
  guard the boundary, and one unwrapped `--watch` line.** Filed by SPEC-143's ship (2026-10-31),
  from verify's notes. (1) In `tests/fixtures/watermark/cli_params.golden`, `faint_opacity`
  (`--opacity 0.00001`) and `faint_opacity_boundary` (`--opacity 0.0000077`) hash identically
  (`6f10ba3e…`), so the boundary row cannot tell a boundary shift from none; pick an opacity just
  either side of the 16-bit boundary that differs by hash, or drop the row. (2) `docs/api-contract.md`
  has one unwrapped line about `--watch` (the line verify named; not located at ship). Both are
  test or docs polish with no `src/` change.

- [ ] **SPEC-144** (framed 2026-10-31) — [M] **A tag push publishes every channel, on runners that
  exist.** cargo-dist puts the Apple Silicon build on `macos-14`, retired 2026-11-02 (0.8.2's run
  was cancelled twice in a brownout); npm is published by hand; crates.io and npm went out ~2 h
  before the binaries. Fix: `macos-15` override; npm (OIDC trusted publishing) and crates.io as
  cargo-dist publish jobs after `host`. Must land before the next tag. DEC-111 reserved.

- [ ] (not yet written) — [S] **The release pipeline runs on `ubuntu-22.04`, which retires
  2027-04-17.** Filed by SPEC-144's frame (2026-10-31), from actions/runner-images#14254
  (brownouts in March 2027). cargo-dist 0.32.0 through 0.33.1 default `plan`, `host`,
  `build-global-artifacts`, Homebrew, `announce` and the Linux build to it. The global jobs can
  move with `[dist.github-custom-runners] global = "ubuntu-24.04"`; the Linux build moving raises
  the released binary's glibc floor (2.35 → 2.39), a user-visible change that needs a ruling
  (or a container/older-glibc build). Released versions are unaffected. Must land before
  March 2027.

- [ ] (not yet written) — [S] **crates.io is still published with a stored token.** Filed from
  SPEC-144's ruling 4 (2026-11-01). crates.io supports trusted publishing (OIDC from GitHub
  Actions), which would drop `CARGO_REGISTRY_TOKEN`. That token expired once already (0.8.1's
  publish failed 403 until it was replaced). SPEC-144 keeps the token so each release changes one
  credential path. Do this after SPEC-144's first real tag has published both registries from CI.

**Count:** 4 shipped / 0 active / 24 pending

## Design Notes

- The first spec's refusal must reuse SPEC-131's recipe validation (`param_specs`), not add a
  second copy for the CLI, so the two paths cannot drift again.

## Dependencies

### Depends on
- STAGE-050 (shipped 2026-10-27): the watermark op, SPEC-131's param schema, SPEC-133's
  refusal and skip.

### Enables
- 0.8.0's release prep, once the first item ships. **Done: 0.8.0 released 2026-10-28.**
- Roadmap row #6, the recipe playground, through the wasm `validate` export.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
