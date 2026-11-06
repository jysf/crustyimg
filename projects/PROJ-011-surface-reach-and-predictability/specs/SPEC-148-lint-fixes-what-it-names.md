---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-148
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-063
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null                # chosen at build
  created_at: 2026-11-05

references:
  decisions:
    - DEC-050
    - DEC-015
    - DEC-035
  constraints:
    - every-public-fn-tested
    - test-before-implementation
  related_specs:
    - SPEC-053
    - SPEC-119
    - SPEC-149

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-063's first success criterion: every `fix:` line `lint` prints, run verbatim, fixes the
  file, and `gps-metadata-leak` needs a coordinate.

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
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 4728482
      tokens_breakdown:
        input: 88
        output: 29533
        cache_creation: 110906
        cache_read: 4587955
      estimated_usd: 3.73
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Interactive frame session, worktree docs/spec-148-frame. Measured from transcript
        b9453114-2616-49f4-9f66-3ecf6897e33a.jsonl, found by the prompt's title, deduped by
        `.message.id` (44 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Opus $5 in / $25 out per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Taken before the commit/push exchanges, so it
        slightly under-counts the session.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-148: lint fixes what it names

## Context

STAGE-063 (PROJ-011's last stage) closes user-facing output that is wrong or misleading before
0.8.3. This spec takes its two `lint` items, both found on the `jysf/crustyimg-example` demo site
(2026-10-29) and carried from STAGE-054: the printed `fix:` commands do not fix the file, and
`privacy/gps-metadata-leak` fires on a GPS block with no coordinates. DEC-050 promises that every
finding "names a fix a user can run verbatim"; today none of them does.

## Goal

Today a user who copies a `fix:` line from `lint` gets the image dumped to their terminal and an
unchanged file (or exit 3, for `convert --format`), and a camera that writes an empty GPS block is
reported as leaking a location. When this ships, every `fix:` line run verbatim rewrites the file in
place and clears its finding, and the GPS rule fires only when a coordinate is present.

## Frame

**Verdict: GO.** Size **S**. Both premises hold on `main` (6198d77, `crustyimg 0.8.2` debug build),
and the drive found two more defects in the same fix lines (a malformed `convert --format` and
unquoted paths) that belong here because they are the same printed string.

### Premise check (driven, not read)

Fixtures built in a scratch dir with ImageMagick + `exiftool` 13 (independent of the code under
test); config enabling every rule, `max_bytes = 500000`, `max_intended_width = 1600`. Each printed
line was copied verbatim and run on a scratch copy, stdout redirected to a file.

| Rule | Printed fix (verbatim) | rc | stdout | File changed? | Cleared? |
|---|---|---|---|---|---|
| `privacy/gps-metadata-leak` | `crustyimg meta clean --gps gpscoord.jpg` | 0 | 1,694,401 B | no | no |
| `size/oversized-bytes` | `crustyimg optimize base.jpg` | 0 | 679,355 B | no | no |
| `dims/oversized-dimensions` | `crustyimg resize --max 1600 orient.jpg` | 0 | 297,818 B | no | no |
| `privacy/camera-metadata` (opt-in) | `crustyimg meta strip camera.jpg` | 0 | 1,694,313 B | no | no |
| `color/unexpected-icc` (opt-in) | `crustyimg meta strip icc.jpg` | 0 | 1,694,313 B | no | no |
| `color/wrong-colorspace` (CMYK) | `crustyimg convert --format cmyk.jpg` | **3** | 0 | no | no |
| `color/wrong-colorspace` (16-bit) | `crustyimg convert --format deep16.png` | **3** | 0 | no | no |
| `orient/orientation-not-baked` | `crustyimg auto-orient orient.jpg` | 0 | 468,070 B | no | no |

Rules with no fix line: `color/missing-icc`, `format/animated-gif`, `format/animated-input`,
`size/truncated-or-corrupt` (fix is `None` in source). The fix string is built in one place,
`Finding::fix_command` (`src/lint/mod.rs:152`: `format!("crustyimg {} {}", f, self.file.display())`),
and rendered three ways: human `fix:` line (`src/lint/report.rs:44`), JSON `"fix"`
(`report.rs:114`), SARIF message text (`report.rs:193`).

- **Every fix writes to stdout.** `build_sink` (`src/cli/common.rs:301`) defaults to
  `Sink::Stdout` when there is no `-o`/`--out-dir`; `Sink::Stdout` has no TTY check (only
  `Sink::Display` checks, `src/sink/mod.rs:551`). The TTY case was not driven: `script(1)` cannot
  allocate a pty in this sandbox. The code path is the same with or without a TTY.
- **`convert --format <file>` is malformed.** `--format` takes a value, so it consumes the file
  name and `convert` sees no input: exit 3, `input not found or unreadable`.
- **Unquoted paths.** `lint "my photo.jpg"` prints `fix: crustyimg meta clean --gps my photo.jpg`,
  which a shell splits into two inputs.

**(2) GPS.** `exiftool -GPSVersionID=2.3.0.0` gives a JPEG whose GPS IFD holds only
`GPSVersionID` (exiftool reports nothing else under `-GPS:all`). `lint` gives `error
privacy/gps-metadata-leak`, exit 7. Real coordinates (`GPSLatitude`/`Longitude` + Refs) also flag
(the positive control holds). Also flagged today, none of which holds a location: altitude +
timestamp + `GPSMapDatum` only; `GPSLatitudeRef` only. Latitude without longitude flags too.
The P1100 shape is **not** in the repo: the only JPEG fixtures with a GPS IFD
(`tests/fixtures/c2pa/signed_gps.jpg`, `tests/fixtures/optimize/exif_already_compressed.jpg`) both
carry coordinates. The spec adds the negative fixture (see Q4).

### Q1 — Reproduction
Above. Both reproduce on `main`.

### Q2 — Fix (1): carry `-o <f> -y` in the printed fix (recommended)
- **A TTY refusal does not fix the bug.** The verbatim command would then error on a terminal and
  still leave the file unfixed. In CI or a script (not a TTY, where `lint` mostly runs: the GitHub
  Action, the demo site's contributor guide) it would still pour the image into the log. A printed
  `-o <f> -y` is needed either way.
- **Blast radius of a TTY refusal:** every encoding verb defaulting to stdout (`build_sink` in
  `common.rs`, used from `ops.rs` and `optimize.rs`), plus the metadata lane. Redirects and pipes
  would be unaffected (not a TTY); the documented pipe forms use explicit `-o -`
  (`README.md:285-286`). The cost is small, but it is a behaviour change on every verb and does not
  serve this stage. Not recommended here. If wanted later, it is its own item.
- **The printed form is local to `lint`:** one function (`fix_command`) and the fragments in
  `src/lint/rules.rs` / `src/lint/mod.rs`. `docs/github-actions.md:118-121` already prints this
  form, so the docs move toward the code, not away from it.
- **Is `-o <f> -y` on the source safe?** Yes for correctness, not atomic. `Sink::File`
  (`sink/mod.rs:481-500`, `633-645`) encodes the whole output to memory first, then
  `truncate(true)`-opens the path and writes. Input-equals-output works (driven: all clearing rows
  below wrote in place). A crash or full disk mid-write leaves a truncated file; there is no
  temp-and-rename. The symlink guard rejects a symlinked destination even with `-y` (DEC-035).
  Format: `-o f.jpg` pins JPEG by extension and the file keeps its format. ⚠ `--format` beats the
  `-o` extension by design (DEC-015/099): `convert --format webp d.png -o d.png -y` wrote **WebP
  bytes into `d.png`**, exit 0. So a printed fix must never pair `--format X` with an `-o` of
  another extension.
- **Quoting: yes.** The path is printed twice now, so a space breaks it twice. Shell-quote the path
  when it contains anything outside a safe set (POSIX single-quote form). The design decides
  whether JSON/SARIF `fix` carries the quoted string too (recommended: yes, the same string, since
  the field is documented as "the full runnable command").

### Q3 — Does the corrected fix clear the finding?
Driven with `<fix> <f> -o <f> -y`, then `lint <f>` re-run on the result:

| Rule | Corrected fix | Changed | Cleared |
|---|---|---|---|
| gps-metadata-leak (coords) | `meta clean --gps f -o f -y` | yes | yes |
| gps-metadata-leak (version only) | same | yes | yes |
| oversized-bytes, 500 KB budget | `optimize f -o f -y` | yes (448.8 KB) | yes, **by luck** |
| oversized-bytes, 100 KB budget | `optimize f -o f -y` | yes (459,577 B) | **no** |
| oversized-bytes, 100 KB budget | `optimize f --max-size 100000 -o f -y` | yes (97,864 B) | yes |
| oversized-dimensions | `resize --max 1600 f -o f -y` | yes | yes |
| camera-metadata | `meta strip f -o f -y` | yes | yes |
| unexpected-icc | `meta strip f -o f -y` | yes | yes |
| wrong-colorspace (CMYK) | `convert --format jpeg f -o f -y` | yes (8-bit sRGB) | yes |
| wrong-colorspace (16-bit PNG) | `convert --format png f -o f -y` | yes | **no** (still 16-bit) |
| orientation-not-baked | `auto-orient f -o f -y` | yes | yes |

Two rules do not clear:
- **`size/oversized-bytes`: in scope.** The rule knows its budget (`target.byte_budget()`), so the
  fragment becomes `optimize --max-size <budget>` (driven: clears). The design checks `--max-size`
  semantics (unit, unreachable budget behaviour, format kept).
- **`color/wrong-colorspace`, 16-bit PNG: in scope as "stop printing a fix that does not work".**
  crustyimg has no bit-depth option; the only clearing command changes the format, so it cannot
  write in place under the same name. Recommended: CMYK keeps a fix (`convert --format jpeg`); the
  16-bit branch prints no command and says what to do in the message, as `format/animated-gif`
  does. A real depth-reduction fix is PROJ-012's (`color/wrong-colorspace` closure). The CMYK fix
  converts without ICC, so colour accuracy is also PROJ-012's (qcms).

### Q4 — Fix (2): the predicate
- **Today:** `scan_exif` (`src/lint/mod.rs:355`):
  `exif.fields().any(|f| f.tag.context() == exif::Context::Gps)`, i.e. any tag in the GPS IFD.
- **Recommended predicate:** flag when the GPS IFD holds a **position**: `GPSLatitude` or
  `GPSLongitude`, or `GPSDestLatitude` or `GPSDestLongitude` (`In::PRIMARY`). One coordinate is
  enough (a latitude alone narrows a location; driven: it flags today and should keep flagging).
  Not a leak on its own: `GPSVersionID`, the Ref tags, `GPSAltitude`/`Ref`,
  `GPSTimeStamp`/`GPSDateStamp`, `GPSMapDatum`, `GPSSatellites`, DOP/speed/track/direction.
  Open: `GPSAreaInformation` (free text, may hold a place name) and `GPSProcessingMethod`; see the
  maintainer questions.
- **Positive control:** `tests/fixtures/c2pa/signed_gps.jpg` (exiftool-written coordinates
  `50/1,29/1,1113/25`) must still flag. Negative fixture: a JPEG whose GPS IFD holds only
  `GPSVersionID`, hand-built bytes like the existing `gps_jpeg()` (not from the repo's own metadata
  writer: fixtures from the code under test cannot fail). ⚠ The existing unit fixture `gps_jpeg()`
  (`src/lint/mod.rs:633`) holds only `GPSLatitudeRef`. Under the new predicate it must stop firing,
  so `gps_rule_fires_on_gps_exif_and_is_clean_without` (`:730`) needs a fixture with a coordinate.
- **Other places that decide "has GPS":** `grep -rn 'Context::Gps\|GPSLatitude\|has_gps\|0x8825' src`
  finds only `lint` (`src/lint/mod.rs`) and the metadata writer (`src/metadata/tiff.rs:60`
  `GPS_PTR`; `tiff::remove_gps` used by `clean_gps`, `src/metadata/mod.rs:238`). `meta clean --gps`
  removes the whole GPS IFD whatever it holds, a superset of the new predicate, so the two agree in
  the direction that matters: clean always clears the finding (driven on the version-only file:
  cleared). No `info`, wasm or optimize path tests for GPS. `--keep-gps` is read by nothing
  (SPEC-149's item).
- **Message:** keep the text; with the new predicate it is true ("image carries GPS location
  metadata").

### Q5 — Release implications
- **Contract surfaces.** DEC-050 makes **rule ids** the stability surface; ids do not change. The
  `crustyimg.lint/v1` JSON `fix` field is documented as "the full runnable command"
  (`report.rs:79-80`, `docs/cli-reference.md:484-497` names the schema only). Changing its text
  fulfils that contract; no schema bump. The **finding set** shrinks for GPS-without-coordinates
  files: a CI job that failed on them now passes. That is user-visible.
- **The GitHub Action** (`crustyimg-action`, separate repo) renders SARIF/annotations whose message
  embeds the fix (`report.rs:193`). Annotation text changes; nothing parses it.
- **The demo site** (`jysf/crustyimg-example`): its README/contributor guide and the blog post
  (`docs/blog/2026-10-29-a-cat-portfolio-proves-the-github-actions.md:70-80`) describe both bugs as
  "being fixed". After 0.8.3, the demo's canary still fires: its fixtures use coordinates (checked
  at design, not here).
- **Proposed CHANGELOG text** (the Unreleased section is shared with PROJ-017; the ship or release
  session adds it, this spec does not):
  - *Fixed: `lint`'s suggested fixes now rewrite the file. Each `fix:` line (and the JSON `fix`
    field) ends in `-o <file> -y` and quotes paths with spaces; before, running one wrote the image
    to standard output and left the file unchanged. The `color/wrong-colorspace` fix names a format
    (CMYK JPEG) or, for 16-bit PNG, says what to do instead of printing a command that failed. The
    `size/oversized-bytes` fix passes the budget (`optimize --max-size <bytes>`).*
  - *Fixed: `privacy/gps-metadata-leak` no longer fires on a GPS block with no coordinates (some
    cameras, such as the Nikon P1100, write one into every JPEG). A file with a latitude or
    longitude is still flagged.*

### Q6 — Docs (outside the PROJ-017 files)
- `docs/github-actions.md:116-121`: the fix table already uses `-o <file> -y`. Align it with what
  `lint` prints: `size/oversized-bytes` shows `optimize <file> --profile preserve --max 1600 -o
  <file> -y` (not what lint prints; make it `optimize --max-size <bytes>`), `dims` shows `resize
  --max 1600` (fine). Add or drop `color/wrong-colorspace` per Q3. Line 115 "Each command rewrites
  the file in place" becomes true of `lint`'s own output too.
- `docs/blog/2026-10-29-…:77-80`: a dated post. Leave the text; optionally add a one-line
  "Fixed in 0.8.3" note (maintainer question).
- `src/lint/mod.rs:73-76` and `:92` doc comments (``fix` … the report renders as `crustyimg <fix>
  <file>``) and `report.rs:24`, `:79-80`: update to the new shape.
- `docs/cli-reference.md`: no `fix:` example and no GPS rule text. No change.
- `docs/recipes.md:101-108` and `skills/crustyimg/SKILL.md:42` mention `meta clean --gps` and the
  rule but are **PROJ-017's files**. Read-only check: neither shows a `fix:` line or describes the
  GPS predicate, so neither needs to change. If design finds otherwise, it names it for PROJ-017.
- `README.md`: no `fix:` line, no GPS-rule text. No change.

### Q7 — Size, verification cost, scope
**S.** Code: one formatter, about five rule fragments, one predicate (~40 lines of change). Tests:
one new negative GPS fixture plus updated unit fixtures. The cost is in the end-to-end proof.

**How verify proves "run verbatim, it clears":** an integration test that, for each rule with a
fix, builds an independent fixture, runs `lint --format json`, takes the `fix` string **as printed**,
runs it through `sh -c` (not argv assembled by the test: assembling argv would hide the quoting
and `--format` bugs) in a temp dir, then asserts (a) stdout is empty, (b) the file's bytes changed,
(c) `lint` on the file no longer reports that rule, (d) the file still decodes in its original
format. Traps that apply:
- **Trap 14:** the control is the old code put back verbatim. Restoring the old `fix_command` must
  turn the test red on (a)/(b); restoring the old `scan_exif` line must turn the version-only test
  red.
- **Trap 10 (formatted output is a derived value):** include a path with a space and a path with a
  `'`, where naive and quoted output differ.
- **Trap 13:** the red reason must be stated per row: an exit-3 `convert` red is not a stdout red.
- **Trap 11:** no `sh` row may skip silently; Windows CI has no `sh -c` with POSIX quoting, so the
  design states the platform gate and makes a skipped row fail where coverage is required (or uses
  a quoting form that works under both, decided at design).
- **Trap 12:** name each mutation's location (`fix_command`, each rule fragment, `scan_exif`) and
  every test expected red.
- Opt-in rules (`camera-metadata`, `unexpected-icc`) need a config enabling them, or the row
  exercises nothing (`a-harness-that-exercises-nothing-reports-green`).

Expected cost: frame ~$5, design ~$8–12, build ~$10–15 (Sonnet viable; the end-to-end harness is
the work), verify ~$8–12 (Opus, mutations).

**In scope:** `fix_command` appends ` -o <f> -y` with shell-quoted paths, in all three renderings;
`convert --format` fragments get a format (CMYK → `jpeg`) or become `None` (16-bit PNG, message
says what to do); `optimize` fragment carries `--max-size <budget>`; GPS predicate keyed on a
coordinate; fixtures and tests; the doc comments and `docs/github-actions.md` table.

**Out of scope:** listed below.

**Rough build file list:**
- `src/lint/mod.rs`: `fix_command`, `scan_exif` predicate, GPS rule doc, unit fixtures.
- `src/lint/rules.rs`: `OversizedBytes`, `WrongColorspace` fragments + unit tests.
- `src/lint/report.rs`: doc comments; renderer unit tests whose expected strings change.
- `tests/lint.rs` (and `tests/common/mod.rs` if a fixture helper lives there): end-to-end
  verbatim-fix harness, GPS negative/positive rows.
- `tests/fixtures/lint/` (new, or inline bytes): the version-only GPS JPEG.
- `docs/github-actions.md`: the fix table.
- None of the PROJ-017 files.

**Open maintainer questions (one line each, with recommendation):**
1. Printed fix vs TTY refusal? **Recommend the printed `-o <f> -y` only;** no TTY refusal in this
   spec.
2. Does `GPSAreaInformation` (free-text place name) alone count as a leak? **Recommend yes** (it
   can name the place; cheap to include).
3. 16-bit PNG: print no fix (message only) vs a fix that writes a sibling `.webp`? **Recommend no
   fix + message;** a format change under the same name is the WebP-in-`.png` trap.
4. Donate one P1100 JPEG (cropped tiny, serial stripped except the GPS IFD) as a real-camera
   fixture? **Recommend yes if easy;** the hand-built bytes are enough for the gate.
5. Add a "Fixed in 0.8.3" line to the 2026-10-29 blog post? **Recommend yes,** one line at ship.

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- A TTY refusal for binary stdout on the encoding verbs (declined at frame: does not fix the bug, changes every verb). Not filed; raise it as its own item if wanted.
- Atomic (temp-and-rename) writes in `Sink::File`. Pre-existing; not filed by this frame.
- A real bit-depth reduction fix and ICC-correct CMYK conversion for `color/wrong-colorspace`: PROJ-012 (`proposed`, ICC via `qcms`).
- Animated-input fixes (`format/animated-gif`, `format/animated-input`): PROJ-012.
- `--keep-gps` being read by nothing: SPEC-149.
- `optimize` to stdout choosing a 48 % larger JPEG than `optimize -o x.jpg`: filed on STAGE-063 as its own item.
- Any edit to `docs/recipes.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md` Unreleased, `src/operation/`, `src/recipe/` (PROJ-017's). CHANGELOG text is proposed in the Frame, Q5.
- The `--format` > `-o` extension precedence (DEC-015/099, by design; STAGE-037 holds the open question).

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
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
