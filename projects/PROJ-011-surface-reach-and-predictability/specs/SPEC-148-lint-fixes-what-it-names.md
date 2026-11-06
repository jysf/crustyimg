---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-148
  type: bug                        # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 5471597
      tokens_breakdown:
        input: 78
        output: 51102
        cache_creation: 163183
        cache_read: 5257234
      estimated_usd: 4.93
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Interactive design session, worktree docs/spec-148-design. Measured from transcript
        a2da7e84-7289-4179-90be-0d27019c54f9.jsonl, found by the prompt's title, deduped by
        `.message.id` (39 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Opus $5 in / $25 out per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Taken before the commit/push exchanges, so it
        slightly under-counts the session.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 7991620
      tokens_breakdown:
        input: 102
        output: 53241
        cache_creation: 203694
        cache_read: 7734583
      estimated_usd: 3.88
      duration_minutes: null
      recorded_at: 2026-11-05
      note: >
        Build session, worktree feat-spec-148-lint-fixes-what-it-names. Measured from transcript
        ae3eb629-68d7-432d-a847-6b379d9e14af.jsonl, found by the prompt's title, deduped by
        `.message.id` (47 unique ids, all claude-sonnet-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Sonnet $3 in / $15 out per MTok, cache_creation
        x1.25, cache_read x0.10. Taken before the PR and final-report exchanges, so it slightly
        under-counts the session.
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

## Design

Designed 2026-11-05 (Opus 5.5) on `main` at `1eba31f`. Every fix below was driven with `sh -c` on a
scratch copy against a debug build of that commit before it went into this spec (see "Driven at
design"). The orchestrator's rulings 1–9 are taken as settled; this section records the calls the
design owned.

### Call 1 — one formatter, quoting inside it

`Finding::fix_command` stays the single source for all three renderings (human `fix:` line,
`report.rs:44`; JSON `fix`, `report.rs:114`; SARIF message, `report.rs:193`). The renderers do not
change. Rules keep emitting a **fragment** (`Option<String>`: the verb and its flags, no binary
name, no file). The formatter owns everything that depends on the file: the quoting, the `./`
guard, `-o <file> -y`, and the decision that there is no runnable command.

New shape (in `src/lint/mod.rs`, next to `Finding`):

```rust
/// Which shell's quoting a printed fix uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shell {
    Posix,
    PowerShell,
}

/// The shell a user on this platform most likely pastes into: PowerShell on
/// Windows (the default terminal shell and the GitHub Actions default), sh/bash/zsh
/// everywhere else.
const HOST_SHELL: Shell = if cfg!(windows) { Shell::PowerShell } else { Shell::Posix };

/// Quote one path so `shell` reads it as exactly one argument. `None` when the
/// path is not valid UTF-8 (no printable command can name it).
fn shell_arg(path: &Path, shell: Shell) -> Option<String> { … }

impl Finding {
    /// The full runnable fix: `crustyimg <fragment> <file> -o <file> -y`, with
    /// the file quoted for this platform's shell. `None` when the rule names no
    /// command, the input was stdin, or the path cannot be printed.
    pub fn fix_command(&self) -> Option<String> {
        self.fix_command_for(HOST_SHELL)
    }

    fn fix_command_for(&self, shell: Shell) -> Option<String> {
        if self.from_stdin {
            return None;
        }
        let fragment = self.fix.as_deref()?;
        let file = shell_arg(&self.file, shell)?;
        Some(format!("crustyimg {fragment} {file} -o {file} -y"))
    }
}
```

`shell_arg` rules, in order:
1. `path.to_str()`; `None` (no command) if the path is not UTF-8. Printing it lossily would name a
   different file.
2. If the string starts with `-`, prefix `./`. Quoting does not stop clap reading `-x.jpg` as a
   flag; `./-x.jpg` is the file on every OS (Rust accepts `/` on Windows).
3. If every char is in the safe set, print it bare. Safe set: ASCII alphanumerics and `_ - . /`;
   for `PowerShell` also `\` and `:` (so `img\a.jpg` and `C:\x\a.jpg` stay bare and also run in
   `cmd`). The safe set is deliberately small: `,` makes an array in PowerShell, `=`, `~`, `@`, `$`
   and spaces mean something in one shell or another.
4. Otherwise quote:
   - `Posix`: `'` + replace every `'` with `'\''` + `'`.
   - `PowerShell`: `'` + double every `'` **and** every `‘ ’ ‚ ‛` (U+2018–U+201B; PowerShell
     treats these as single quotes too) + `'`.

`fix_command_for` and `shell_arg` are private and take `Shell` explicitly, so the unit tests cover
both quoters on every OS. Only `HOST_SHELL` is chosen by `cfg!`.

**How a rule says "no command":** the fragment is `None`, as today. There is no empty-string
path: the formatter does not filter `Some("")`. A unit test instead asserts that no rule emits an
empty or whitespace fragment (FT-6), so an empty string fails a test rather than being silently
dropped.

**Stdin** (a design call the frame did not raise): `lint -` builds a target whose path is the stem
`stdin`, so today it prints `fix: crustyimg meta clean --gps stdin`, which names a file that does
not exist. A stdin finding now has no command. `Finding` gets a private `from_stdin: bool`
(default `false`), and `run_lint` sets it with a private `Finding::on_stdin(self)` when the input
is `Input::Stdin`. The input variant decides, not the name, so a real file called `stdin` still gets
its fix. Driven: the pipe form (`crustyimg <frag> - -o -`) is not a safe substitute, because
`optimize - -o -` switched a JPEG to AVIF and made it 58 % larger.

### Call 2 — which rules print a fix, and the exact line

The file is `a.jpg` (or `a.png`) for the bare rows, `my photo.jpg` and `it's.jpg` for the quoting
rows. "Before" is what `main` prints today (frame, driven).

| Rule | Fragment (after) | Input the fragment needs | Rule knows it? | Before | After (POSIX; Windows differs only for `'`) |
|---|---|---|---|---|---|
| `privacy/gps-metadata-leak` | `meta clean --gps` | none | — | `crustyimg meta clean --gps a.jpg` | `crustyimg meta clean --gps a.jpg -o a.jpg -y` |
| `privacy/camera-metadata` (opt-in) | `meta strip` | none | — | `crustyimg meta strip a.jpg` | `crustyimg meta strip a.jpg -o a.jpg -y` |
| `color/unexpected-icc` (opt-in) | `meta strip` | none | — | `crustyimg meta strip a.jpg` | `crustyimg meta strip a.jpg -o a.jpg -y` |
| `size/oversized-bytes` | `optimize --max-size <budget>` | the byte budget, in bytes | yes: `target.byte_budget()`, the value the rule compares against | `crustyimg optimize a.jpg` | `crustyimg optimize --max-size 100000 a.jpg -o a.jpg -y` |
| `dims/oversized-dimensions` | `resize --max <W>` (unchanged) | the intended width | yes: `target.intended_width()` | `crustyimg resize --max 1600 a.jpg` | `crustyimg resize --max 1600 a.jpg -o a.jpg -y` |
| `orient/orientation-not-baked` | `auto-orient` | none | — | `crustyimg auto-orient a.jpg` | `crustyimg auto-orient a.jpg -o a.jpg -y` |
| `color/wrong-colorspace`, CMYK JPEG | `convert --format jpeg` | none (the rule only fires on JPEG content) | — | `crustyimg convert --format a.jpg` (exit 3) | `crustyimg convert --format jpeg a.jpg -o a.jpg -y` |
| `color/wrong-colorspace`, ≥16-bit | **none** (message changes, see Call 3) | — | — | `crustyimg convert --format a.png` (exit 3) | *(no `fix:` line)* |
| `color/missing-icc` (opt-in) | none (unchanged) | — | — | *(none)* | *(none)* |
| `format/animated-gif`, `format/animated-input` | none (unchanged; PROJ-012) | — | — | *(none)* | *(none)* |
| `size/truncated-or-corrupt` | none (unchanged) | — | — | *(none)* | *(none)* |
| any rule, path `my photo.jpg` | — | — | — | `crustyimg meta clean --gps my photo.jpg` | `crustyimg meta clean --gps 'my photo.jpg' -o 'my photo.jpg' -y` (same on Windows) |
| any rule, path `it's.jpg` | — | — | — | `crustyimg meta clean --gps it's.jpg` | POSIX `crustyimg meta clean --gps 'it'\''s.jpg' -o 'it'\''s.jpg' -y`; PowerShell `… 'it''s.jpg' -o 'it''s.jpg' -y` |
| any rule, path `-x.jpg` | — | — | — | `crustyimg meta clean --gps -x.jpg` | `crustyimg meta clean --gps ./-x.jpg -o ./-x.jpg -y` |
| any rule, input `-` (stdin) | — | — | — | `crustyimg meta clean --gps stdin` | *(no `fix:` line; JSON `null`)* |

Flags go before the file (`optimize --max-size N a.jpg`); driven, clap accepts that order on every
verb above. `--max-size` takes a bare integer as bytes (`parse_size`, `src/cli/optimize.rs:511`),
which is exactly the budget's unit, so no rounding and no `KB` conversion.

**Format is kept** on every row: no fragment except CMYK names `--format`, so `-o a.<ext>` picks
the source's own extension; a file with no extension keeps its content format (driven). The CMYK
fragment pairs `--format jpeg` with JPEG content. For a CMYK JPEG whose name does not end in
`.jpg`/`.jpeg`, the bytes stay JPEG; that is the property ruling 4 protects.

### Call 3 — the no-fix path and the JSON/SARIF consequence

The ≥16-bit branch of `WrongColorspace` returns fragment `None` and the message becomes:

`{N}-bit channels are needless for the web; crustyimg cannot reduce bit depth, so re-export it at 8 bits per channel`

(`{N}` is `info.bit_depth`, as today.) The CMYK branch keeps its message.

- **Human:** no `fix:` line (the renderer already skips `None`).
- **JSON `crustyimg.lint/v1`:** `"fix":null`. `null` is already in the documented schema ("the full
  runnable command … or `null`", `report.rs:79-80`) and is what `missing-icc` and the animated
  rules emit today. **No schema bump** for an absent fix, and none for a present one (its text
  becomes what the field always promised).
- **SARIF:** `message.text` is the message alone, with no `— fix:` suffix (the existing `None` arm,
  `report.rs:195`).
- **Who notices:** a consumer that *parses* the `fix` text (splitting on spaces, taking the last
  word as the file) breaks on the new trailing `-o <file> -y` and on quoted paths. Nothing in this
  repo or the Action does; `crustyimg-action` passes the SARIF through. A consumer that runs the
  string through a shell, which is what the field is documented for, now gets a working command.
  The finding set changes too: 16-bit PNGs keep their finding but lose the fix; GPS blocks without
  a location lose the finding (Call 5).

### Call 4 — how each printed fix is proved, on all three OSes

The headline test (`tests/lint.rs`, FT-1) is table-driven. Each row builds a fixture independently
of the code under test, writes it into a fresh `TempDir`, and runs `lint --format json --select
<rule> <name>` with **`current_dir` set to the temp dir and the file named relatively**, so the
printed path is the name as typed (space, `'`, leading `-`). It then reads the `fix` string with
`serde_json` (already a dev-dependency) and runs **that string, unmodified**, through the host
shell in the same directory:

- Unix: `sh -c <fix>`.
- Windows: `powershell -NoProfile -NonInteractive -Command "<fix>; exit $LASTEXITCODE"`. That is
  Windows PowerShell 5.1, which every `windows-latest` image ships. The fix is the PowerShell-quoted
  form there, so the test runs exactly what a Windows user is shown.

For both, `PATH` is the test binary's directory prepended to the inherited `PATH` (with
`std::env::join_paths`). Before any row runs, the harness asks the same shell where `crustyimg`
resolves (`command -v crustyimg` / `(Get-Command crustyimg).Source`) and asserts it is
`env!("CARGO_BIN_EXE_crustyimg")` (both sides canonicalized). A Homebrew `crustyimg` earlier on
`PATH` would otherwise run the released binary and report the same version.

**Trap 11:** no row skips on any OS. If the shell cannot be spawned, the test **fails** with
`cannot run the printed fix: no <sh|powershell> on PATH: <err>`. Both shells exist on every CI
image, so no CI environment variable gate is needed and no CI step changes (trap 16 does not apply).
The one test-only override, `CRUSTYIMG_TEST_FIX_SHELL` (M-13), is read only by the FT-1 helper and is
never set in CI. The test
runs in the existing `Test` step (`cargo test --verbose`) on the 3-OS `check` matrix. The PowerShell
quoter itself is unit-tested on every OS (FT-4), so the macOS/Linux legs also check the Windows
strings. Only the Windows leg runs them in a real PowerShell.

Each row asserts, then **records** rather than panics (trap 15), with one counter per failure kind:

| Kind | Check |
|---|---|
| `not_reported_before` | the `--select`ed rule is in the pre-fix findings (the row exercises something) |
| `fix_shape` | `fix` is a string that starts with `crustyimg ` and ends with ` -y` |
| `shell_failed` | shell exit status 0 |
| `stdout_nonempty` | the shell's stdout is empty |
| `unchanged` | the file's bytes differ from the fixture's |
| `still_reported` | `lint --format json --select <rule> <name>` afterwards reports no finding for that rule |
| `format_changed` | `image::guess_format` of the result equals that of the fixture, and the result decodes with `image::load_from_memory` |

After all rows run, the test panics if any counter is non-zero. The panic message prints every
counter (`shell_failed=2 stdout_nonempty=9 …`) and, per failing row, its name, kinds, the fix string,
and the first 300 bytes of stderr. So a red run states its reason (trap 13), and a row that fails two
ways shows both.

Rows (FT-1): `gps-coords` (`leak.jpg`), `gps-area-info` (`area.jpg`, `GPSAreaInformation` only),
`camera-metadata` (`cam.jpg`, Make), `unexpected-icc` (`icc.jpg`, 5000-byte profile),
`oversized-bytes-jpeg`, `oversized-bytes-png`, `oversized-dimensions` (`wide.png`, intended width
16), `orientation` (`rot.jpg`, Orientation 6), `cmyk` (`cmyk.jpg`), `space` (`my photo.jpg`, GPS),
`quote` (`it's.jpg`, GPS), `dash` (`-x.jpg`, GPS; lint invoked as `lint … -- -x.jpg`), `subdir`
(`sub/leak.jpg` reached by linting the relative dir `sub`, so Windows prints `sub\leak.jpg` and
exercises the backslash-safe branch).

`oversized-bytes-jpeg` must show that the budget matters. Before the lint, the test runs plain
`optimize <copy> -o <copy> -y` (argv, not the shell) on a second copy, measures that size `S1`, and
sets the row's budget to `S1 * 3 / 4`. It asserts `S1 * 3 / 4 < fixture size` and writes the
`[[budget]]` config into the temp dir. After the fix it also asserts the **plain-optimized copy is
still reported** by `lint` (`budget_control` kind). Driven at design: plain `optimize` left a 1.07 MB
JPEG at 459–679 KB against a 100 KB budget; `--max-size 100000` gave 97,523 B.

### Call 5 — GPS predicate (ruling 7, as code)

`scan_exif` (`src/lint/mod.rs:355`):

```rust
// A location is a coordinate (where the photo was taken, or the
// destination it names) or a free-text place name. A block holding only
// GPSVersionID, Ref tags, altitude, timestamps or map datum says nothing
// about where (some cameras write one into every JPEG).
const GPS_LOCATION_TAGS: &[Tag] = &[
    Tag::GPSLatitude,
    Tag::GPSLongitude,
    Tag::GPSDestLatitude,
    Tag::GPSDestLongitude,
    Tag::GPSAreaInformation,
];
let has_gps = exif.fields().any(|f| GPS_LOCATION_TAGS.contains(&f.tag));
```

Any IFD (not only `In::PRIMARY`): a location in a thumbnail's GPS IFD is still a leak, and
`meta clean --gps` removes every GPS block, so it still clears the finding. The `ExifFacts::has_gps` doc and
`LintTarget::has_gps_exif` doc become "carries a GPS location (a coordinate or a place name)".
The rule's message is unchanged. Tag names checked against `kamadak-exif 0.6.1` (`tag.rs:639–717`).

### Driven at design (debug build of `1eba31f`, scratch dir, stdout redirected)

| Command (via `sh -c` unless noted) | rc | stdout | Result |
|---|---|---|---|
| `crustyimg optimize --max-size 100000 t.jpg -o t.jpg -y` (1,066,050 B JPEG) | 0 | 0 | 97,523 B JPEG, lint clean |
| `crustyimg optimize --max-size 100000 t.png -o t.png -y` (1.85 MB 8-bit PNG) | 0 | 0 | 98,699 B **PNG**, downscaled to 276×207 with a stderr warning; lint clean |
| `crustyimg optimize --max-size 10 u.jpg -o u.jpg -y` (unreachable) | 0 | 0 | 643 B best effort + warning; finding stays |
| same on `u.JPG` / no extension | 0 | 0 | JPEG kept |
| `crustyimg meta clean --gps 'my photo.jpg' -o 'my photo.jpg' -y` | 0 | 0 | cleared |
| `crustyimg meta clean --gps 'it'\''s.jpg' -o 'it'\''s.jpg' -y` | 0 | 0 | cleared |
| `crustyimg meta clean --gps ./-dash.jpg -o ./-dash.jpg -y` | 0 | — | cleared |
| `crustyimg convert --format jpeg cmyk.jpg -o cmyk.jpg -y` | 0 | 0 | cleared, JPEG |
| `crustyimg meta clean --gps ny.jpg -o ny.jpg` (no `-y`) | **5** | 0 | `output file already exists (use --yes to overwrite)` |
| `cat g.jpg \| crustyimg lint -` | 7 | — | prints `fix: crustyimg meta clean --gps stdin` |
| `crustyimg optimize --max-size 100000 - -o - < g.jpg` | 0 | 95,275 B | **AVIF**, 58 % larger: why stdin gets no command |

Not driven: a real PowerShell (no Windows host here); the Windows leg of FT-1 is the first run.
`--max-size` with a lossless format downscales to hit the budget. That is `optimize`'s documented
behaviour, and the fix inherits it. An unreachable budget leaves the finding standing after a
best-effort write, which is outside this spec.

Ruling 9's question: the frame's 459,577 B figure is the `-o <f> -y` path (Q3 table), and
679,355 B is the stdout path. The printed fix uses `-o`, so the STAGE-063 discrepancy item does not
change what this spec's fix does.

## Inputs

- **Files to read:** `src/lint/mod.rs` (`Finding`, `fix_command`, `scan_exif`, `run_lint`, unit
  fixtures), `src/lint/rules.rs` (`OversizedBytes`, `WrongColorspace`, unit tests),
  `src/lint/report.rs` (three renderers, goldens), `tests/lint.rs`, `tests/common/mod.rs`
  (`jpeg_with_gps`, `wrap_with_gps_app1`, `jpeg_with_icc`, `png_16bit`, `detailed_jpeg_at_quality`,
  `detailed_png`), `tests/adoption_glue.rs:76` (lints `jpeg_with_gps`, expects exit 7),
  `docs/github-actions.md:111-125`.
- **Related code paths:** `src/cli/optimize.rs:511` (`parse_size`), `src/sink/mod.rs` (`-o` with
  `-y`, symlink guard), `src/source/mod.rs` (`Input::Stdin`, stem `stdin`).

## Outputs

- **Files created:** `tests/fixtures/lint/cmyk_16x16.jpg` (≈300 B, made by ImageMagick, independent
  of crustyimg) and `tests/fixtures/lint/README.md` (one line per fixture: the exact command that
  made it).
- **Files modified:** see the build file list under Implementation Context.
- **New exports:** none. `Shell`, `HOST_SHELL`, `shell_arg`, `fix_command_for`, `from_stdin` and
  `on_stdin` are private to `src/lint/mod.rs`. `Finding::fix_command`'s signature is unchanged.
- **Database changes:** none.

## Acceptance Criteria

- [ ] **AC-1 One shape, three renderings.** Every finding with a fragment prints
  `crustyimg <fragment> <file> -o <file> -y`, with `<file>` from `shell_arg`. The human `fix:` line, the
  JSON `fix` string and the SARIF `— fix:` suffix carry the identical string. *(FT-2, FT-3)*
- [ ] **AC-2 Each printed fix, run verbatim, fixes its finding.** For every FT-1 row, on Linux,
  macOS and Windows: shell exit 0, empty stdout, the file's bytes changed, `lint` no longer reports
  that rule, and the result decodes in the fixture's format. One row per fix-printing rule:
  gps (coordinates), gps (area information), camera-metadata, unexpected-icc, oversized-bytes
  (JPEG and PNG), oversized-dimensions, orientation-not-baked, wrong-colorspace CMYK. Plus the paths
  `my photo.jpg`, `it's.jpg`, `-x.jpg` and `sub/leak.jpg`. *(FT-1)*
- [ ] **AC-3 Quoting.** Safe-set paths print bare. Other paths are quoted so the host shell reads
  one argument: POSIX single quotes on Unix, PowerShell single quotes on Windows, with the exact
  escapes in Call 1. A path starting with `-` gets `./`. A non-UTF-8 path gets no command. *(FT-4,
  FT-5; e2e in FT-1)*
- [ ] **AC-4 The budget fix carries the budget.** `size/oversized-bytes` prints
  `optimize --max-size <budget>` with the rule's budget in bytes. On FT-1's JPEG row, plain
  `optimize` does not clear the finding and the printed fix does. *(FT-1 `oversized-bytes-jpeg` +
  `budget_control`, FT-7)*
- [ ] **AC-5 CMYK keeps a fix that keeps the format.** The CMYK branch prints
  `convert --format jpeg`; the result is a JPEG and `color/wrong-colorspace` no longer fires.
  *(FT-1 `cmyk`, FT-7)*
- [ ] **AC-6 ≥16-bit prints words, no command.** The finding stays (warn). Its message says crustyimg
  cannot reduce bit depth and to re-export at 8 bits per channel. Human output has no `fix:` line,
  JSON has `"fix":null` (not `""`), and the SARIF text has no `fix:`. No rule emits an empty or
  whitespace fragment. *(FT-6, FT-8)*
- [ ] **AC-7 Stdin prints no command.** `lint -` with a GPS JPEG on stdin still reports the finding,
  with `"fix":null` and no `fix:` line. A real file named `stdin` still gets its fix. *(FT-9, FT-5)*
- [ ] **AC-8 GPS fires on a location only.** The finding is reported for each of `GPSLatitude`,
  `GPSLongitude`, `GPSDestLatitude`, `GPSDestLongitude` and `GPSAreaInformation` on its own, and for
  `tests/fixtures/c2pa/signed_gps.jpg`. It is not reported for a GPS IFD holding only
  `GPSVersionID` (the P1100 shape), only `GPSLatitudeRef`, or only `GPSAltitude` + `GPSAltitudeRef`
  + `GPSTimeStamp` + `GPSDateStamp` + `GPSMapDatum`. *(FT-10 unit, FT-11 integration)*
- [ ] **AC-9 Existing GPS tests still mean "a leak".** `tests/common::jpeg_with_gps` and
  `src/lint/mod.rs::gps_jpeg` carry a coordinate. Every existing test that lints them, including
  `tests/adoption_glue.rs`'s exit-7 check, passes unchanged in its assertions. *(existing tests,
  FT-10)*
- [ ] **AC-10 Docs say what `lint` prints.** The `docs/github-actions.md` §5 table matches Call 2's
  "after" column (with `<file>`/`<budget>`/`<width>` placeholders), adds the `color/wrong-colorspace`
  rows, and says how quoting works on Windows. The doc comments at `src/lint/mod.rs:73-77`, `:92-93`,
  `:150-151`, `:285`, `:338`, `:538-539`, `rules.rs:76-77`, `:134-135`, and `report.rs:22-26`,
  `:78-80` describe the new shape. *(FT-12, read at verify)*
- [ ] **AC-11 The harness proves what it ran.** On every OS the shell resolves `crustyimg` to the
  test binary. A missing shell fails the test instead of skipping it. The failure summary counts each
  failure kind in FT-1's table. *(FT-1 preamble, mutation M-13)*

## Failing Tests

Written at design; all must be red on `main` before build changes code, except where noted, with
the reason quoted in Build Completion (trap 13).

- **`tests/lint.rs`** (new section `// ── SPEC-148: printed fixes run verbatim ──`)
  - **FT-1** `every_printed_fix_run_verbatim_fixes_its_finding`. The harness and rows in Call 4.
    Red on `main`: under the shell, the gps/camera/icc/bytes/dims/orientation rows give
    `stdout_nonempty` + `unchanged` + `still_reported`. `cmyk` gives `shell_failed` (exit 3) +
    `unchanged`. `space`/`quote` give `shell_failed`. `fix_shape` fails on every row (no ` -y`).
    `gps-area-info` passes the GPS steps on `main` (the old predicate fires on it). Quote the
    summary line.
  - **FT-2** `human_json_and_sarif_carry_the_same_fix_string`. One GPS file in a temp dir, linted
    three ways. The `fix:` line's text, the JSON `fix` and the text after `— fix: ` in the SARIF
    `message.text` are byte-equal, and equal to `crustyimg meta clean --gps leak.jpg -o leak.jpg -y`.
    Red on `main`: the expected string.
  - **FT-8** `a_16_bit_png_prints_words_and_no_command_in_every_format`. `png_16bit` is linted
    human, JSON and SARIF. The finding is present in all three. Human: no line contains `fix:`.
    JSON: `findings[0]["fix"].is_null()`; asserting `== ""` would not catch the empty-string mutation.
    SARIF: the text contains `cannot reduce bit depth` and not `fix:`. Red on `main`: a
    `convert --format` fix is printed.
  - **FT-9** `stdin_input_prints_no_command`. `lint - --format json` with `jpeg_with_gps` piped in:
    the finding is present, `fix` is `null`, and the human run has no `fix:` line. Red on `main`:
    `crustyimg meta clean --gps stdin`.
  - **FT-11** `gps_rule_fires_on_a_location_and_not_on_an_empty_block`. Integration rows over
    `jpeg_with_gps_tags` (below) plus `signed_gps.jpg`, linted with `--select
    privacy/gps-metadata-leak`, asserting exit 7 or exit 0 per AC-8. Red on `main`: the
    version-only, ref-only and altitude/timestamp/datum rows exit 7.
  - **FT-12** `the_github_actions_doc_fix_table_matches_what_lint_prints`. Reads
    `docs/github-actions.md` with `include_str!`, normalized to `\n` (the CRLF lesson). Asserts the
    §5 table contains `crustyimg optimize --max-size <budget> <file> -o <file> -y`,
    `crustyimg convert --format jpeg <file> -o <file> -y`, and none of
    `--profile preserve --max 1600`. Red on `main`: the old optimize row.
- **`src/lint/mod.rs`** (unit tests)
  - **FT-3** `fix_command_shape_is_fragment_file_dash_o_file_dash_y`.
    `Finding::new("img/leak.jpg", …, Some("meta clean --gps"))` gives
    `fix_command_for(Posix) == Some("crustyimg meta clean --gps img/leak.jpg -o img/leak.jpg -y")`.
    The same holds for `PowerShell`. A `None` fragment gives `None`.
  - **FT-4** `shell_arg_quotes_for_each_shell`. Exact strings, both shells, on every OS:
    `a.jpg` → bare/bare; `img/a.jpg` → bare/bare; `img\a.jpg` → `'img\a.jpg'` / bare;
    `C:\x\a.jpg` → quoted / bare; `my photo.jpg` → `'my photo.jpg'` / `'my photo.jpg'`;
    `it's.jpg` → `'it'\''s.jpg'` / `'it''s.jpg'`; `it’s.jpg` (U+2019) → `'it’s.jpg'` /
    `'it’’s.jpg'`; `a,b.jpg` and `$x.jpg` → quoted in both; `-x.jpg` → `./-x.jpg` both.
    Trap 10: the `'` and `’` rows are the ones where naive quoting and correct quoting differ.
  - **FT-5** `fix_command_none_for_stdin_and_non_utf8`. A finding passed through `on_stdin()` gives
    `None`. A finding on a real path named `stdin` gives `Some(…stdin -o stdin -y)`. On `#[cfg(unix)]`,
    a path from `OsStr::from_bytes(b"\xff.jpg")` gives `None`.
  - **FT-10** `gps_predicate_keys_on_a_location_tag`. Replaces
    `gps_rule_fires_on_gps_exif_and_is_clean_without`. A local hand-built `gps_jpeg_with(&[entries])`
    (generalizing today's `gps_jpeg`) feeds each AC-8 row to `GpsMetadataLeak.check`. `gps_jpeg()`
    becomes lat + lon + refs. Keep the `f.fix() == Some("meta clean --gps")` assertion.
- **`src/lint/rules.rs`** (unit tests)
  - **FT-6** `no_rule_emits_an_empty_fix_fragment`. For every rule's firing fixture in this module,
    `fix()` is `None` or a non-empty, non-whitespace string that does not start with `crustyimg`
    and contains no path.
  - **FT-7** `budget_and_colorspace_fragments_name_their_inputs`. `OversizedBytes` with budget
    `123457` → `Some("optimize --max-size 123457")` (an odd number, so a `KB`-rounding formatter
    differs). `WrongColorspace` on the crafted CMYK fixture → `Some("convert --format jpeg")`, and on
    `png16()` → `None` with a message containing `cannot reduce bit depth`. Update
    `wrong_colorspace_fires_on_16bit_clean_on_8bit` and `every_rule_carries_a_runnable_or_noted_fix`
    to match.
- **`src/lint/report.rs`** (existing goldens, updated; they are the hash guard)
  - `json_golden_is_an_exact_byte_stable_string`, `human_render_…`, the SARIF golden: the synthetic
    fragments become `meta clean --gps` and `optimize --max-size 500000`. Add a third synthetic
    finding on `img/my photo.jpg` (quoted the same in both shells, so the golden stays cross-platform).
    Expected strings are written out in full: `…"fix":"crustyimg meta clean --gps img/leak.jpg -o
    img/leak.jpg -y"…`.

**Fixture helpers** (`tests/common/mod.rs`, hand-built bytes, not from crustyimg's metadata writer):
- `jpeg_with_gps_tags(w, h, entries: &[(u16 tag, u16 type, u32 count, Vec<u8> value)])` builds a
  little-endian TIFF with IFD0 → GPSInfo pointer → a GPS IFD holding exactly `entries`. Values over
  4 bytes go in a data area after the IFD, with offsets computed, not hard-coded. Rows: version-only
  `(0x0000, BYTE, 4, [2,3,0,0])`; lat-ref only; lat + lat-ref; lon + lon-ref; dest-lat; dest-lon;
  area-info `(0x001C, UNDEFINED, n, b"ASCII\0\0\0Kyiv")`; altitude/alt-ref/timestamp/datestamp/map-datum.
- `jpeg_with_gps(w, h)` becomes `jpeg_with_gps_tags` with lat-ref N, lat `50/1,29/1,1113/25`,
  lon-ref E, lon `30/1,31/1,0/1`. Update its doc block. `wrap_with_gps_app1` goes or is rebuilt on the
  new builder.
- `jpeg_with_make` and `jpeg_with_icc_len(base, len)` (an APP2 profile of `len` filler bytes, for
  the 5000-byte row) are added if `tests/common` lacks them.
- CMYK: `include_bytes!("fixtures/lint/cmyk_16x16.jpg")`, made with
  `magick -size 16x16 xc:'#c04020' -colorspace CMYK -quality 90 cmyk_16x16.jpg` (driven: 296 B,
  fires `color/wrong-colorspace`). `image` cannot encode CMYK.

### Mutation table (trap 12)

Apply each at the named location only, run `cargo test --test lint --test adoption_glue` and
`cargo test --lib lint`, and record which tests went red **and the failure text** (trap 13). FT-1
reds are named by counter.

| # | Mutation | Location | Expected red (reason) |
|---|---|---|---|
| M-1 | **Old formatter verbatim** (`format!("crustyimg {} {}", f, self.file.display())`, stdin check removed) | `Finding::fix_command_for` | FT-1: `fix_shape` on every row; `stdout_nonempty` + `unchanged` + `still_reported` on gps-coords, gps-area-info, camera, icc, both oversized-bytes, dims, orientation, cmyk (new fragment, so it writes to stdout), subdir; `shell_failed` + `unchanged` on `space`, `quote` (the shell splits or mis-parses the path) and `dash` (clap reads `-x.jpg` as a flag). FT-2, FT-3, FT-5 (stdin row), FT-9, report goldens (string mismatch). |
| M-1b | **Old fragments verbatim** (`"optimize"`, both `convert --format`) together with M-1 | `rules.rs` | additionally FT-1 `cmyk`: `shell_failed` (exit 3); FT-7; FT-8 (fix printed for 16-bit). |
| M-2 | Drop ` -y` | `fix_command_for` | FT-1 `fix_shape` all rows + `shell_failed` (exit 5 "output file already exists") + `unchanged`; FT-2, FT-3, goldens (hash). |
| M-3 | Drop `-o {file}` | `fix_command_for` | FT-1 `stdout_nonempty` + `unchanged` all rows; FT-2, FT-3, goldens. |
| M-4 | Drop quoting (`shell_arg` returns the raw string) | `shell_arg` | FT-1 `shell_failed` on `space`, `quote`; `dash` only if the `./` step is also removed (M-4b); FT-4; report golden (space row). Other FT-1 rows stay green. |
| M-4b | Drop the `./` prefix | `shell_arg` | FT-1 `dash` (`shell_failed`, clap reads a flag); FT-4 `-x.jpg` row. |
| M-5 | Swap the CMYK pairing (`convert --format webp`) | `WrongColorspace` CMYK branch | FT-1 `cmyk`: `format_changed` (WebP bytes in `cmyk.jpg`, exit 0); FT-7. |
| M-6 | Remove the budget (`"optimize"`) | `OversizedBytes` | FT-1 `oversized-bytes-jpeg`: `still_reported`; FT-7. `oversized-bytes-png` may stay green (plain optimize of the PNG fixture may already clear); the build records which. |
| M-7 | GPS predicate widened back (`f.tag.context() == exif::Context::Gps`, the old line verbatim) | `scan_exif` | FT-10 version-only, ref-only, altitude rows; FT-11 same rows (exit 7). |
| M-8 | GPS predicate narrowed to `Tag::GPSLatitude` only | `GPS_LOCATION_TAGS` | FT-10 + FT-11 lon-only, dest-lat, dest-lon, area-info rows; FT-1 `gps-area-info` (`not_reported_before`). |
| M-9 | No-fix emits empty: ≥16-bit branch returns `Some(String::new())` | `WrongColorspace` 16-bit branch | FT-6; FT-8 (JSON `fix` is a string, human has `fix:`, SARIF has `fix:`). |
| M-10 | Formatter returns `Some(String::new())` when the fragment is `None` | `fix_command_for` | FT-8 (JSON `""` is not null; human prints `fix: `), FT-9, FT-3 (`None` row), and existing `"fix":null` assertions (`report.rs` `clean_outcome…`, animated-gif tests). |
| M-11 | Stdin check removed | `fix_command_for` | FT-5, FT-9. |
| M-12 | `HOST_SHELL` forced to `Posix` on Windows | `HOST_SHELL` | Windows leg only: FT-1 `quote` (`'it'\''s.jpg'` mis-parses in PowerShell, `shell_failed`). `subdir` stays green (`'sub\leak.jpg'` is valid PowerShell too). macOS/Linux legs green, as expected. The build cannot run this locally; verify reads the Windows leg of a throwaway CI push or records it as unrun. |
| M-13 | Harness: the shell spawn error is turned into an early `return` | FT-1 helper | FT-1 must still fail on a host without the shell. Prove it by running FT-1 once with `PATH` stripped of the shell's directory (`env PATH=/nonexistent cargo test …` is wrong because it also hides cargo; use a test-only env override of the shell name such as `CRUSTYIMG_TEST_FIX_SHELL=nosuchshell`, read **only** by the test helper). It must go red with `cannot run the printed fix`. Without the override, the mutation is invisible, so this row is a check on the harness, not on the product. |
| M-14 | Formatter prints the budget in KB (`--max-size {}KB`, budget/1000) | `OversizedBytes` | FT-7 (`123457` row); FT-1 `oversized-bytes-jpeg` if rounding takes the budget over (record either way). |

M-1's row is the trap-14 control: the old code put back verbatim. The FT-1 table entry for it is
written per row by the build from the actual run, not copied from here.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Build file list (none is a PROJ-017 path)

- `src/lint/mod.rs`: `Shell`, `HOST_SHELL`, `shell_arg`, `fix_command(_for)`, `from_stdin` +
  `on_stdin`, `run_lint` marks stdin, `scan_exif` predicate, doc comments, unit tests FT-3/4/5/10,
  `gps_jpeg` fixture.
- `src/lint/rules.rs`: `OversizedBytes` and `WrongColorspace` fragments, 16-bit message, doc
  comments, unit tests FT-6/FT-7 + updated existing tests.
- `src/lint/report.rs`: doc comments, golden updates (no renderer logic change).
- `tests/lint.rs`: FT-1, FT-2, FT-8, FT-9, FT-11, FT-12.
- `tests/common/mod.rs`: `jpeg_with_gps_tags`, `jpeg_with_gps` with a coordinate, ICC/Make helpers
  if missing.
- `tests/fixtures/lint/cmyk_16x16.jpg`, `tests/fixtures/lint/README.md` (new).
- `docs/github-actions.md`: §5 table and one quoting sentence.

Checked against the PROJ-017 list: no file under `src/operation/`, `src/recipe/`,
`docs/recipes.md`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`,
`skills/crustyimg/SKILL.md`, `CHANGELOG.md`, or `projects/PROJ-017-…/`. `tests/adoption_glue.rs`
is **read, not edited**: it must stay green through the `jpeg_with_gps` change.

Not edited, on purpose: `src/cli/mod.rs`. `lint --help` says "(each with a runnable fix)" and
carries `(SPEC-050, DEC-050)`. That line is SPEC-150's (spec ids in `--help`); "each" was already
false for no-fix rules. SPEC-150 should reword it. `docs/cli-reference.md` and `README.md` show
no `fix:` line (frame, re-checked).

### Decisions that apply

- `DEC-050`: the rule catalog. Rule ids are the stability surface, and each finding "names a fix a
  user can run verbatim". This spec makes that true; ids and the `crustyimg.lint/v1` schema id do
  not change. **No new DEC.** The quoting choice (PowerShell on Windows) is recorded here and in
  `docs/github-actions.md`; it is an output format detail under DEC-050, not a new decision.
- `DEC-015` / DEC-099: `--format` beats the `-o` extension. That is why only the CMYK fragment names
  `--format`, and only with JPEG content.
- `DEC-035`: the symlink guard refuses a symlinked `-o` even with `-y`. A fix on a symlinked asset
  exits non-zero and writes nothing. That is acceptable and out of scope.
- `DEC-003` (metadata lane): `meta clean --gps` and `meta strip` do not re-encode pixels.
- `DEC-051` (GitHub Actions on-ramp): `docs/github-actions.md` is its page.

### Constraints that apply

- `every-public-fn-tested`: `fix_command` is covered by FT-2/FT-3; the new private fns by FT-4/FT-5.
- `test-before-implementation`: FT-1 to FT-12 go in first and are run red on `main`. Quote each red
  reason in Build Completion (trap 13).
- `no-new-top-level-deps-without-decision`: none added (`serde_json` and `tempfile` are already
  dev-deps).

### Prior related work

- `SPEC-050` (shipped): `lint`, the `Finding`/fragment model. `SPEC-053`: the rule catalog.
- `SPEC-119` (shipped): set the precedent that a rule with no correct command prints none
  (`format/animated-gif`).
- `SPEC-149` (not framed): `--keep-gps`. `SPEC-150`: `--help` spec ids.

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- A TTY refusal for binary stdout on the encoding verbs (declined at frame: does not fix the bug, changes every verb). Not filed; raise it as its own item if wanted.
- Atomic (temp-and-rename) writes in `Sink::File`. Pre-existing; not filed by this frame.
- A real bit-depth reduction fix and ICC-correct CMYK conversion for `color/wrong-colorspace`: PROJ-012 (`proposed`, ICC via `qcms`).
- Animated-input fixes (`format/animated-gif`, `format/animated-input`): PROJ-012.
- `--keep-gps` being read by nothing: SPEC-149.
- `optimize` to stdout choosing a 48 % larger JPEG than `optimize -o x.jpg`: filed on STAGE-063 as its own item.
- Any edit to `docs/recipes.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md` Unreleased, `src/operation/`, `src/recipe/` (PROJ-017's). CHANGELOG text is proposed in the Frame, Q5, amended below.
- The `--format` > `-o` extension precedence (DEC-015/099, by design; STAGE-037 holds the open question).
- `cmd.exe` quoting. The Windows fix is PowerShell syntax. In `cmd`, a fix runs as printed only when
  the path needed no quoting, which is the common case since `\` and `:` are safe there.
- A file whose content does not match its extension (a PNG named `.jpg`): `-o` follows the name, so
  the fix rewrites it as the named format. Pre-existing `-o` behaviour.
- `optimize --max-size` with an unreachable budget writes a best-effort file and the finding stays;
  with a lossless format it downscales. `optimize`'s behaviour, inherited.
- A stdin fix (pipe form). Stdin prints no command (Call 1).
- `lint --help`'s "(each with a runnable fix)" and its spec ids: SPEC-150.

### Proposed CHANGELOG text (for the ship/release session; replaces the Frame's Q5 draft)

- *Fixed: `lint`'s suggested fixes now rewrite the file. Each `fix:` line (and the JSON `fix` field)
  ends in `-o <file> -y` and quotes paths that need it (POSIX quoting; PowerShell quoting on
  Windows). Before, running one wrote the image to standard output and left the file unchanged. The
  `size/oversized-bytes` fix passes the budget (`optimize --max-size <bytes>`). The
  `color/wrong-colorspace` fix names the format for a CMYK JPEG (`convert --format jpeg`). For
  16-bit images it prints no command and says to re-export at 8 bits per channel, since crustyimg
  cannot reduce bit depth. Findings on standard input no longer print a fix naming a file called
  `stdin`.*
- *Fixed: `privacy/gps-metadata-leak` fires only when the GPS block holds a location (a latitude,
  longitude, destination coordinate or place name). Some cameras, such as the Nikon P1100, write a
  GPS block with only a version number into every JPEG; those are no longer flagged.*

## Notes for the Implementer

- **Build on Sonnet.** The code is mechanical once Call 1 is followed. The work is the FT-1 harness
  and the hand-built TIFF builder. Checkpoint: commit and push after the failing tests are in (red),
  again after the code, again after docs (long-build lesson).
- Run every red test on `main` **before** changing `src/` and quote the failure text per FT.
- `lint` prints the path as given. FT-1 must run `lint` with `current_dir(temp)` and a relative
  name, or every row prints an absolute temp path (often safe-set on macOS/Linux) and the quoting
  rows test nothing.
- Use `serde_json::Value` to read `fix`. Do not regex the JSON: an escaped `\"` in a quoted path would
  fool it.
- On Windows `Command::new("powershell")` resolves `powershell.exe`. Pass the fix plus
  `; exit $LASTEXITCODE` as one `-Command` argument. Do not wrap it in extra quotes; Rust's Windows
  argument escaping handles the space.
- Byte-compare the result against the fixture **bytes in memory**, not a re-read of a file you have
  already overwritten.
- `cargo fmt` after incremental commits; `git commit -s` on every commit (DCO).
- Use your own `CARGO_TARGET_DIR`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** feat/spec-148-lint-fixes-what-it-names
- **PR (if applicable):** draft, see the PR list for the branch
- **All acceptance criteria met?** yes (AC-2's Windows run is left to the PR's windows-latest leg)
- **New decisions emitted:** none
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`):
  - `src/lint/mod.rs` — `Shell`, `HOST_SHELL`, `shell_arg`, `fix_command(_for)`, `from_stdin`/`on_stdin`, `run_lint` marks stdin, `scan_exif` GPS predicate, doc comments, unit tests FT-3/4/5/10
  - `src/lint/rules.rs` — `OversizedBytes` budget fragment, CMYK `convert --format jpeg`, 16-bit prints words and no command, unit tests FT-6/7
  - `src/lint/report.rs` — doc comments, goldens updated plus a third finding on `img/my photo.jpg`
  - `tests/lint.rs` — FT-1 harness and 13 rows, FT-2, FT-8, FT-9, FT-11, FT-12, local EXIF/ICC/CMYK fixture helpers
  - `tests/adoption_glue.rs` — its own coordinate-bearing GPS fixture (the shared one is no longer a location)
  - `tests/fixtures/lint/cmyk_16x16.jpg`, `tests/fixtures/lint/README.md` — new ImageMagick CMYK fixture and its command
  - `docs/github-actions.md` — section 5 fix table and a quoting sentence
- **Deviations from spec:**
  - `tests/common/mod.rs` NOT edited (claimed by SPEC-147): the GPS builder, `jpeg_with_gps` (with a coordinate), Make and ICC helpers live in `tests/lint.rs`; `adoption_glue.rs` has its own fixture. The shared `common::jpeg_with_gps` is now used by nothing in lint tests.
  - FT-12 checks the section 5 slice of the doc, not the whole file: a later section legitimately shows `optimize ... --profile preserve --max 1600` for batch use.
  - The `subdir` row does not print `sub\leak.jpg`: `lint sub` prints the canonicalized absolute path, so the row does not exercise the backslash-safe branch (FT-4 covers it at unit level).

#### RED on main, then GREEN
Red run used main's formatter behind the new API (stub, commit 4304607). Lines quoted:
- FT-1: `printed fixes do not fix their findings: fix_shape=13 shell_failed=4 stdout_nonempty=9 still_reported=13 unchanged=13`; e.g. `row cmyk ... fix: "crustyimg convert --format cmyk.jpg" stderr: error: input not found or unreadable:`, `row space ... error: input not found or unreadable: my`, `row quote ... unexpected EOF while looking for matching`, `row dash ... error: unexpected argument '-x' found`.
- FT-2: `left: "crustyimg meta clean --gps leak.jpg"  right: "crustyimg meta clean --gps leak.jpg -o leak.jpg -y"`
- FT-8: `human output must have no fix line: ... fix: crustyimg convert --format deep.png`
- FT-9: `stdin fix must be null, got "crustyimg meta clean --gps stdin"`
- FT-11: `version only (P1100 shape): expected exit 0 (no location)`; FT-12: `docs/github-actions.md must show crustyimg optimize --max-size <budget> <file> -o <file> -y`
- Unit: FT-3 `left: Some("crustyimg meta clean --gps img/leak.jpg")`; FT-4 `POSIX quoting of "img\\a.jpg"`; FT-5 `left: Some("crustyimg meta clean --gps stdin") right: None`; FT-7 `left: Some("optimize") right: Some("optimize --max-size 123457")` and 16-bit `left: Some("convert --format") right: None`; FT-10 `version only (the P1100 shape): no location, so no finding`; three goldens red. FT-6 passes on main (no empty fragment existed): its red is M-9.
- GREEN after the change: `cargo test --lib lint` 39 passed; `--test lint` 32 passed; `adoption_glue` 4 passed; `just check` passed; lean (`--no-default-features`) lint tests passed.

#### Local mutations (scratch worktree, each reverted: `git diff` empty)
- M-1 old formatter: FT-1 `fix_shape=13 shell_failed=3 stdout_nonempty=10 still_reported=13 unchanged=13`; FT-2, FT-3, FT-5, FT-9, three goldens.
- M-1b + old fragments: FT-1 `fix_shape=13 shell_failed=4 stdout_nonempty=9 still_reported=13 unchanged=13` (cmyk: `input not found or unreadable`); FT-7, FT-8, FT-2, FT-3, FT-5, FT-9, goldens.
- M-2 drop ` -y`: FT-1 `fix_shape=13 shell_failed=13 still_reported=13 unchanged=13` (`output file already exists (use --yes to overwrite)`); FT-2, FT-3, goldens.
- M-3 drop `-o`: FT-1 `stdout_nonempty=13 still_reported=13 unchanged=13`; FT-2, FT-3, goldens.
- M-4 no quoting: FT-1 `shell_failed=2 still_reported=2 unchanged=2` (space, quote); FT-4; goldens (space row). Other rows green.
- M-4b no `./`: FT-1 `shell_failed=1` (dash: `unexpected argument '-x' found`); FT-4.
- M-5 webp pairing: FT-1 `format_changed=1`; FT-7.
- M-6 no budget: FT-1 `still_reported=2 unchanged=1`: both oversized-bytes rows, so the PNG row is red too (`kept the 96951 B source unchanged`); FT-7.
- M-7 old GPS line: FT-10, FT-11. M-8 latitude only: FT-10, FT-11, FT-1 `fix_shape=1 not_reported_before=1 shell_failed=1` (gps-area-info).
- M-9 16-bit `Some("")`: FT-6, FT-7, FT-8, `wrong_colorspace_fires_on_16bit...`.
- M-10 `Some("")` for no fragment: FT-3 (`None` row), FT-8, existing `json_escapes_quotes_and_backslashes`. FT-9 stays GREEN (stdin returns `None` before this path), unlike the table.
- M-11 stdin check removed: FT-5, FT-9.
- M-12 (Windows forced to POSIX quoting): not run, needs the Windows runner; left to the draft PR's windows-latest leg (no local effect on macOS).
- M-13: with `CRUSTYIMG_TEST_FIX_SHELL=nosuchshell` FT-1 goes red with `cannot run the printed fix: no nosuchshell on PATH`. With the spawn error turned into a fake success it still goes red at the preamble (`the shell cannot find crustyimg`), so the harness cannot pass without a shell, though the message differs from the unmutated one.
- M-14 KB rounding: FT-7 only; FT-1 stays green (1000-byte rounding keeps the budget reachable).

#### Hash-mismatch mutation (trap 13)
The three report goldens are the hash guard: M-1, M-2, M-3 turn all three red, M-4 turns them red on the `my photo.jpg` row.

#### Windows
Not run here. The first PowerShell run of FT-1 is the draft PR's windows-latest leg.

- **Follow-up work identified:** none new; `lint --help` "(each with a runnable fix)" is SPEC-150's.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Whether unit tests may reference not-yet-existing private API for the red-on-main run; I put main's formatter behind the new signatures to get honest failures. Also `lint <dir>` prints an absolute path, so the `subdir` row cannot test the backslash branch.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The claim on `tests/common/mod.rs` was added late; the spec's "update the shared `jpeg_with_gps`" instructions conflicted with it.

3. **If you did this task again, what would you do differently?**
   — Start the scratch mutation worktree's target dir copy earlier; and make the `subdir` row lint a relative path to a file, not a directory.

No design call turned out wrong.

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
