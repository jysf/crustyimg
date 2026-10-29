---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-137
  type: bug                        # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-054
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-opus-5-5     # usually same Claude, different session
  created_at: 2026-10-28

references:
  decisions:
    - DEC-054
    - DEC-045
    - DEC-034
    - DEC-066
  constraints:
    - untrusted-input-hardening
    - pure-rust-codecs-default
    - every-public-fn-tested
    - clippy-fmt-clean
  related_specs:
    - SPEC-060
    - SPEC-119
    - SPEC-138

value_link: >
  Closes one of STAGE-054's two 0.8.1 silent-wrong-output bugs: SVG text that names a font
  family stops vanishing on the CLI and in the browser build.

# Self-reported AI cost per cycle. See AGENTS.md §4 and docs/cost-tracking.md.
cost:
  sessions:
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 3877270
      tokens_breakdown:
        input: 62
        output: 23174
        cache_creation: 70374
        cache_read: 3783661
      estimated_usd: 2.91
      duration_minutes: null
      recorded_at: 2026-10-28
      note: >
        Half of a shared frame session (SPEC-137 + SPEC-138, worktree docs/spec-137-138-frame).
        Whole session: 7,754,540 tokens, $5.82. Measured from the transcript
        (ae99d1a9-….jsonl, identified by content), deduped by `.message.id` (62 unique ids,
        all claude-opus-5-5), input/cache_creation/cache_read once per id, MAX output_tokens.
        Priced at Opus $5/$25 per MTok, cache_creation x1.25 ($6.25), cache_read x0.10 ($0.50).
        Measured before the commit/push turns, so it slightly under-counts.
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 6684492
      tokens_breakdown:
        input: 108
        output: 56022
        cache_creation: 146214
        cache_read: 6482148
      estimated_usd: 5.56
      duration_minutes: null
      recorded_at: 2026-10-28
      note: >
        Measured from the transcript (60602b56-….jsonl, identified by content), deduped by
        `.message.id` (54 unique ids, all claude-opus-5-5), input/cache_creation/cache_read once
        per id, MAX output_tokens. Priced at Opus $5/$25 per MTok, cache_creation x1.25,
        cache_read x0.10. Measured before the commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-137: SVG text in any font family renders in the bundled font

## Context

STAGE-054 backlog item "An SVG font family other than Go draws no text", filed by the PR #211
(resvg 0.49) verify on 2026-10-28. One of the two bugs that open 0.8.1 (PROJ-011).

DEC-054 turned SVG text on with only the bundled Go font loaded, and no system fonts, so that
text would never vanish silently ("rendering it to *nothing* is a silent-wrong-output footgun").
It set `usvg::Options::font_family = "Go"`, which only covers text that names **no** family. Text
that names any family other than exactly `Go`, including the generics `sans-serif`/`serif`,
resolves to no face and draws nothing. Most real SVGs name a family.

**What the user sees today** (`main` @ 3e6d39a, release build; also 0.8.0 from Homebrew and the
released 0.7.1 binary):

```
$ crustyimg convert logo.svg --format png -o logo.png     # <text font-family="sans-serif">Hello</text>
$ echo $?
0                                                         # no stderr; logo.png has no text at all
```

## Goal

SVG `<text>` in any `font-family` (generic, named, unknown, a list) renders in the bundled Go
font, on the CLI and in the browser build. An SVG that already rendered (no family, `Go`, or a
list containing `Go`) produces byte-identical output, and no system font is ever read.

## Frame

**Verdict: GO, [S].** Premise confirmed. The fix is confirmed by a throwaway probe build (reverted,
never committed): five `fontdb::Database::set_*_family("Go")` calls in `hardened_options()`.

### Premise check (driven 2026-10-28)

Release build of `main` @ 3e6d39a in a fresh `CARGO_TARGET_DIR`. Each SVG is
`<svg width="200" height="60"><text x="10" y="40" font-size="32" …>Hello</text></svg>`, run through
`crustyimg convert X.svg --format png -o X.png -y`. Ink is counted with ImageMagick, not crustyimg:
`magick X.png -alpha extract -threshold 0 -format '%[fx:mean*w*h]' info:`, i.e. pixels with
alpha > 0.

- **Negative control holds:** no family → **738** px of ink, exit 0.
- **Scenario confirmed:** `sans-serif` → **0** px, exit 0, empty stderr. The same on Homebrew 0.8.0
  (0 px) and on the downloaded 0.7.1 release asset (`crustyimg-aarch64-apple-darwin.tar.xz`, sha256
  OK): sans 0 px, no family 738 px. So the bug predates resvg 0.49 (#211).
- **Browser build: same code path, not driven.** `src/image/mod.rs:595` dispatches every SVG to
  `svg::decode_svg` → `hardened_options()` with no `cfg(target_arch)` split, and the wasm build
  keeps resvg's `text` feature (`Cargo.toml:243`). The fix covers it with no wasm-specific code.

### Family table

"Today" = `main`. "Fix" = the probe binary with the five generics set to Go. "==Go" = output PNG
byte-identical (`cmp`) to the `font-family="Go"` render.

| Form | Example | Today | Fix | How usvg 0.49 / fontdb 0.24 resolves it |
|---|---|---|---|---|
| none | (no attribute) | 738 | 738, **same bytes as main** | `Options::font_family` = "Go" |
| exact | `Go` | 738 | 738, **same bytes as main** | `Family::Name("Go")` matches |
| list with Go | `Helvetica, Go` | 738 | 738, **same bytes as main** | first match wins → Go |
| generic | `sans-serif`, `serif`, `monospace`, `cursive`, `fantasy` | 0 each | 738 each, ==Go | `Family::SansSerif` → `db.family_name()` → was "Arial"/"Times New Roman"/…, not loaded |
| named, installed on host | `Helvetica`, `Zapfino` | 0 | 738, ==Go | `Name` misses (system fonts never loaded) → fallback |
| named, unknown | `NoSuchFontXyz` | 0 | 738, ==Go | same |
| list without Go | `Helvetica, sans-serif` | 0 | 738, ==Go | every entry misses → fallback |
| quoted list | `'Helvetica Neue', Arial, sans-serif` | 0 | 738, ==Go | same |
| CSS property | `style="font-family: sans-serif"` | 0 | 738, ==Go | same parse |
| `<style>` element | `text{font-family:Arial}` | 0 | 738, ==Go | same |
| case | `go` | **0** | 738, ==Go | fontdb compares names **case-sensitively** (`family.0 == name`, fontdb `lib.rs:668`) → fallback |
| mixed `<tspan>` | `Arial` text, `Go` tspan | 359 (half the text) | 738, ==Go | per-span resolution |
| bold | `Go` + `font-weight="bold"` | 738, ==Go | 738, ==Go | nearest weight in the candidate set (`find_best_match`): the one Regular face. **No synthetic bold.** |
| italic | `Go` + `font-style="italic"` | 738, ==Go | 738, ==Go | same: Regular, upright. **No synthetic oblique.** |
| generic + bold | `sans-serif` + bold | 0 | 738, ==Go | generic → Go → nearest weight |
| glyphs Go lacks | `Go`, text `漢字` | 136 | 136 (unchanged) | face found; missing glyphs → `.notdef` (probably boxes). Out of scope. |

**Why five setters are enough (no custom resolver):** usvg's `default_font_selector`
(`usvg-0.49.0/src/text/mod.rs:99-153`) maps each CSS family to a `fontdb::Family` and then **always
appends `Family::Serif`** as a last resort (`:114`). `fontdb::Database::new()` sets serif to "Times
New Roman" (`fontdb-0.24.0/src/lib.rs:183`), which is never loaded. `set_serif_family("Go")`
therefore catches every unknown named family. The other four setters cover the generics named
directly. Every row above draws Go with the same five lines.

### The rule: maintainer question; recommendation below

**Recommend: every family falls back to Go, silently** (option 1), plus one sentence in the docs.
Evidence:
- Substitution is not lost output. The bug is text that vanishes, and every row above stops
  vanishing. Drawing a missing face in a substitute is what every SVG renderer does.
- A warning would fire on nearly every SVG with text, because real SVGs name Arial/Helvetica/
  Inter, which the bundled set will never have. A warning that fires on every input is noise.
- With the fix, usvg's own `log::warn!("No match for … font-family")` never fires (the query
  always matches through Serif → Go). A warning needs a custom `font_resolver.select_font` that
  records the unmatched family, a carrier on `Image` (the `animated_input` pattern, SPEC-119) and a
  print at each CLI decode site (`ANIMATED_INPUT_WARNING` is printed from 5 sites:
  `cli/build.rs:614`, `cli/optimize.rs:1658,1723`, `cli/ops.rs:373,484`). That is **[M]**, for a message about
  typeface choice.
- Option 3 ("refuse an SVG whose text cannot render") has nothing left to refuse after the fix.
  Every family resolves. The only residual loss is glyph coverage (CJK/emoji), which is a
  different predicate.
- `lint` has **no SVG rule at all** (`grep -i svg src/lint/` → no hits), and `info` reports a
  rasterized SVG as `png` (DEC-054's accepted wart). Neither is a natural home today. If the
  maintainer wants a signal, an opt-in `lint` rule ("this SVG names fonts crustyimg substitutes")
  is the better home than a decode-time warning. That would be a new backlog item, not this spec.

### Bytes: the check design must require

A test that renders the **already-working** forms (no family, `Go`, `Helvetica, Go`, `Go` + bold,
`Go` + italic) and compares each against a **SHA-256 pinned as a constant**, captured from `main`'s
binary before the change. Do not compute the hash with the code under test. Every one of those rows
was byte-identical to `main` in the probe.

**System-font negative control:** `font-family="Helvetica"` (installed on macOS) and `Arial`
(installed on macOS and Windows runners) must be **byte-identical to the `Go` render**. If a system
font were read, the bytes would differ. This only discriminates on a host where the named font is
installed, so design should name the font per CI OS (e.g. DejaVu Sans on Linux).

### Sizing: [S]

Code: 5 lines in `hardened_options()`, plus a doc comment. Verification: one table test, about 12
rows from the table above (ink > 0 by an independent count, or byte-equal to the Go render), plus
the pinned-hash identity rows (5), plus the system-font control (1 to 3 rows), plus a docs sentence.
About 18 assertions in one test file. The warning (option 2) would make it **[M]**; if the maintainer
picks it, cut the warning into its own spec rather than grow this one.

### Scope

**In:** the five `set_*_family` calls; the table test; the pinned-hash identity check; the system-font
negative control; one sentence in `docs/cli-reference.md` (SVG section) and the README saying SVG
text always renders in the bundled Go font, whatever family it names, with no bold or italic faces;
a CHANGELOG `Fixed` line.

**Out:** see `### Out of scope` below.

### Rough build file list

- `src/image/svg.rs`: `hardened_options()`, plus unit tests in its `#[cfg(test)]` module
- `tests/svg_font_family.rs` (new): the table, identity and system-font rows through the CLI
- `tests/fixtures/svg/`: small family-form SVGs, or generate them inline in the test
- `docs/cli-reference.md`, `README.md`: one sentence each
- `CHANGELOG.md`: `Fixed` under Unreleased

### Open maintainer questions

1. **Silent fallback, or a warning naming the family?** Recommend **silent** (above). A warning
   makes this [M] and goes in its own spec.
2. **Bold/italic:** accept "drawn in Go Regular, upright", or bundle Go Bold/Italic/Bold-Italic
   (`Go-Regular.ttf` is 148,672 B, so roughly that much again per face, unmeasured, on the CLI and on the
   size-gated wasm bundle)? Recommend **accept**, and document
   it. Bundling faces is a DEC-066 size purchase, not a bug fix.

### Maintainer rulings (2026-10-28, settled)

1. **Silent fallback.** Every family, generic or named, known or unknown, renders in the bundled Go
   font with no message. No warning in this spec; an opt-in `lint` rule for SVG fonts would be a
   separate, unscheduled item.
2. **Accept Go Regular.** Bold and italic text draws in Go Regular, upright, with no synthetic bold.
   Document it. No extra faces are bundled.

## Design

Designed 2026-10-28 on `main` @ `5e260b5`. Every number below was driven on a release build in a
fresh `CARGO_TARGET_DIR` (deleted afterwards), or read from the cited `file:line`. Ink is the
count of pixels with alpha > 0, measured with ImageMagick
(`magick X.png -alpha extract -threshold 0 -format '%[fx:round(mean*w*h)]' info:`), never with
crustyimg. Each SVG is `<svg xmlns="http://www.w3.org/2000/svg" width="200" height="60">BODY</svg>`
with `T` = `x="10" y="40" font-size="32"`, run as `convert X.svg --format png -o X.png -y`.

### What `main` does today (the RED baseline)

Every row exits 0 with **empty stderr**. "px" is the first 8 hex digits of the pixel hash
defined under Call 3 (SHA-256 of width LE u32, height LE u32 and the RGBA8 samples).

| Row | BODY | `main` ink | `main` px |
|---|---|---|---|
| none | `<text T>Hello</text>` | 738 | `6e4d6b08` |
| Go | `font-family="Go"` | 738 | `6e4d6b08` |
| list with Go | `"Helvetica, Go"` | 738 | `6e4d6b08` |
| Go + bold | `"Go"` + `font-weight="bold"` | 738 | `6e4d6b08` |
| Go + italic | `"Go"` + `font-style="italic"` | 738 | `6e4d6b08` |
| Go tspan in Go text | `font-family="Go">Hel<tspan font-family="Go">lo</tspan>` | 738 | `6e4d6b08` |
| `sans-serif`, `serif`, `monospace`, `cursive`, `fantasy` | each alone | **0** each | `bb918147` (blank) |
| `Helvetica` / `Arial` / `DejaVu Sans` | named | **0** each | blank |
| `NoSuchFontXyz` | unknown | **0** | blank |
| list without Go | `"Helvetica, sans-serif"` | **0** | blank |
| quoted list | `"'Helvetica Neue', Arial, sans-serif"` | **0** | blank |
| CSS property | `style="font-family: sans-serif"` | **0** | blank |
| `<style>` element | `<style>text{font-family:Arial}</style>` + no attribute | **0** | blank |
| lowercase | `"go"` | **0** | blank |
| inherited | `<g font-family="sans-serif"><text T>Hello</text></g>` | **0** | blank |
| generic + bold | `"sans-serif"` + bold | **0** | blank |
| mixed tspan | `font-family="Arial">Hel<tspan font-family="Go">lo</tspan>` | **253** (only "lo") | `6893dab3` |
| text-free (control) | `<text T></text>`, the same with `font-family="sans-serif"`, and `> </text>` | 0 each | blank |

Released binaries (official `aarch64-apple-darwin` assets, sha256 checked): **0.5.0** and
**0.6.0** draw `sans-serif` and `Helvetica` as 0 px (no family: 738), exit 0, empty stderr. The
frame drove 0.7.1, and Homebrew 0.8.0 matches `main` above. SVG input arrived in 0.5.0
(`CHANGELOG.md:417`), so **every release with SVG input has the bug.**

### Call 1: the code, and what each line actually does

**usvg 0.49's selector, re-read** (`usvg-0.49.0/src/text/mod.rs:99-114`):

```rust
pub fn default_font_selector() -> FontSelectionFn<'static> {
    Box::new(move |font, fontdb| {
        let mut name_list = Vec::new();
        for family in &font.families {
            name_list.push(match family {
                FontFamily::Serif => fontdb::Family::Serif,
                FontFamily::SansSerif => fontdb::Family::SansSerif,
                FontFamily::Cursive => fontdb::Family::Cursive,
                FontFamily::Fantasy => fontdb::Family::Fantasy,
                FontFamily::Monospace => fontdb::Family::Monospace,
                FontFamily::Named(s) => fontdb::Family::Name(s),
            });
        }

        // Use the default font as fallback.
        name_list.push(fontdb::Family::Serif);
```

`FontResolver::default()` uses this selector (`text/mod.rs:84-90`), and `Options::default()`
uses that resolver (`parser/options.rs:118`). `fontdb::Database::query` walks the families in
order and returns the first that has a face (`fontdb-0.24.0/src/lib.rs:663-679`), comparing
names case-sensitively (`family.0 == name`, `:669`). `Database::new()` sets serif to
`"Times New Roman"`, sans-serif to `"Arial"`, cursive to `"Comic Sans MS"`, fantasy to `"Impact"`
(`"Papyrus"` on macOS) and monospace to `"Courier New"` (`:183-193`). None of those is loaded. Text
with no family at all becomes `FontFamily::Named(opt.font_family)` (`parser/text.rs:464-466`).
That line is the only reason "no family" works today. A `font-family` value that fails to parse
takes the same path (`:452-462`).

**So the frame's premise holds, and is sharper than it said.** Four probe builds, all reverted:

| Variant | Generics | Named / unknown / `go` / `<style>` Arial / mixed tspan | Lists, `style=`, inherited, generic + bold | Working forms |
|---|---|---|---|---|
| `main` | 0 | 0 (tspan 253) | 0 | `6e4d6b08` |
| all five setters | 738, ==Go | 738, ==Go | 738, ==Go | `6e4d6b08`, unchanged |
| **`set_serif_family` only** | 738, ==Go | 738, ==Go | 738, ==Go | unchanged |
| the four others, no serif | 738 except `serif` (0) | **0** (tspan 253) | 738 (each list contains a generic) | unchanged |
| five setters, `opt.font_family` line removed | n/a | n/a | n/a | `none` still byte-identical |

"==Go" means the PNG file is byte-identical (`cmp`) to `main`'s `font-family="Go"` render.

- **`set_serif_family("Go")` alone fixes every row**, because usvg tries `serif` after every
  family. The other four setters change no output, ever, while usvg keeps appending `serif`.
- **Keep all five anyway** (orchestrator ruling 3). They make the database correct on its own
  terms: if a future usvg stops appending `serif`, `sans-serif` still resolves. Because they are
  invisible end-to-end, **the unit row `hardened_fontdb_resolves_each_generic_family_to_go` is
  where each one is observable.** It queries the database with that one family and no `serif`
  appended. Without it, removing a setter would be an undetectable (equivalent) mutant.
- **`opt.font_family` stays.** Once `serif` points at Go it is redundant (driven: removing it
  leaves `none` byte-identical). It still says directly what no-family text draws in, rather than
  "Times New Roman, then the last resort". It is pinned by the unit row
  `hardened_options_hold_only_the_bundled_face`.
- **No custom resolver.** Ruling 3's condition is met: the selector always appends `serif`, quoted
  above.

**The code, exactly** (`src/image/svg.rs`, `hardened_options()`, replacing `:189-193`):

```rust
    // Load ONLY the bundled font; never enumerate system fonts.
    let mut db = usvg::fontdb::Database::new();
    db.load_font_data(FONT_DATA.to_vec());
    // Text in any font family draws in the bundled font. The generic families
    // (`serif`, `sans-serif`, ...) default to fonts that are never loaded, so
    // each one points at Go. usvg also tries `serif` after every family a text
    // names, so the `serif` line is what turns a named or unknown family
    // (`Helvetica`, `Arial`, a typo) into Go instead of into no text at all.
    db.set_serif_family(FONT_FAMILY);
    db.set_sans_serif_family(FONT_FAMILY);
    db.set_monospace_family(FONT_FAMILY);
    db.set_cursive_family(FONT_FAMILY);
    db.set_fantasy_family(FONT_FAMILY);
    // Text that names no family at all.
    opt.font_family = FONT_FAMILY.to_string();
    opt.fontdb = Arc::new(db);
    opt
```

The doc comment on `hardened_options()` becomes:

```rust
/// Build the hardened [`usvg::Options`]: no filesystem resource resolution, all
/// external `href`s refused (data: URIs kept), and only the bundled Go font
/// loaded (no system fonts). Every font family a text can name, generic or
/// specific, known or not, resolves to that one font, so text never vanishes for
/// want of a face. Bold and italic draw upright in its Regular face.
```

In the module doc's `## Text (fonts)` paragraph (`svg.rs:42-50`), after "…text is on with a
guaranteed font.", add: "Every `font-family` resolves to it: the five generic families point at
Go, and usvg falls back to `serif` after any family it cannot find. Bold and italic have no face
of their own and draw upright in Go Regular."

### Call 2: no system font can be read, and the control per CI OS

**Structural fact, every OS:** the build compiles `fontdb` with **no features at all**. The release
build fingerprint (`target/release/.fingerprint/fontdb-*/lib-fontdb.json`) reads `"features": []`.
`load_system_fonts`, `load_fonts_dir` and `load_font_file` are `#[cfg(feature = "fs")]`
(`fontdb-0.24.0/src/lib.rs:401`), so **the binary contains no system-font loader.** The only faces
it can hold are bytes passed to `load_font_data`. (`cargo tree -e features` cannot show this
reliably; use the fingerprint.) The behavioural control below is the second line, and it only
discriminates where the font is installed.

| CI OS | Font, path | Checked how | Test behaviour if the file is absent |
|---|---|---|---|
| `macos-latest` | `Helvetica`, `/System/Library/Fonts/Helvetica.ttc` | present on this design host (Darwin 25.6). A core macOS system font. | **panic** (`expected … on macOS`): absent means the path is wrong, not a skip |
| `windows-latest` | `Arial`, `C:\Windows\Fonts\arial.ttf` | **not checked** (no Windows host). A core Windows font. | **panic**, same reason |
| `ubuntu-latest` | first present of `DejaVu Sans` (`/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`), `Liberation Sans` (`/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf`), `Noto Sans` (`/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf`) | **not confirmed.** The runner README (`images/ubuntu/Ubuntu2404-Readme.md`, image `20261026.354.1`) lists only `fonts-noto-color-emoji`. DejaVu may arrive as a dependency, unlisted. | `eprintln!("SKIP an_installed_system_font_is_never_read: no known font file on this host; checked: …")` and return. Linux then rests on the structural fact. `NotoColorEmoji` is deliberately not a candidate: it has no Latin glyphs, so it cannot tell Go from itself on "Hello". |

The row proves its own power before it asserts anything (**positive control**). In the test,
not in crustyimg: read the font file, `load_font_data` it into a fresh `fontdb::Database` that
holds only that face, render the same SVG through `resvg` directly, and assert (a) its alpha
channel has ink > 0 and (b) the alpha channel differs from the Go render's alpha channel. That
proves the family name matches the file and that the face draws different pixels from Go.
**Only then** assert that the CLI's render equals the pinned Go pixels. Building a positive
control from `resvg` is allowed here because it is the control, not the code under test.

**The skip must be seen.** libtest hides stderr of a passing test, so the build runs
`cargo test --test svg_font_family an_installed_system_font -- --nocapture` once locally, and
records in Build Completion which font ran or that it skipped. Verify does the same. CI cannot
show the Linux skip; that is accepted and stated.

### Call 3: cross-platform bytes: one pinned hash, proven across architectures

**Driven.** The official 0.8.0 `x86_64-unknown-linux-gnu` asset (sha256 checked), run in
`debian:bookworm-slim` under `docker --platform linux/amd64` on this arm64 Mac, against Homebrew
0.8.0 and `main`, both arm64 macOS. For none, Go, list with Go, Go + bold, Go + italic, unknown
family (blank) and mixed tspan, **the pixels and the PNG file bytes were identical** on both
architectures. The full pixel hash of every working form, from x86-64 and from arm64:

```
6e4d6b080052173623550a30e4e877658f5da28ddb711fbfad7bc46df2d46ad6
```

**Read.** tiny-skia 0.12 picks SIMD at **compile time** only (`cfg(target_feature = "avx")` etc.,
`wide/f32x8_t.rs:13`). `grep -rn is_x86_feature_detected` over its `src/` finds nothing, so a
runner's AVX2 hardware does not change the code. The release target is baseline x86-64 (SSE2):
there is no `target-cpu` in `.cargo/config.toml`, so CI's x86 runners execute the same SSE2 code as
the emulated binary. A solid text fill compiles to the **lowp** pipeline whenever every stage
has a lowp version (`pipeline/mod.rs:395-411`). Its arithmetic is u16 integer. The approximate
ops that do differ between SSE and NEON (`recip_fast` → `_mm_rcp_ps` / `vrecpeq_f32`,
`wide/f32x4_t.rs:353-361`) are used only by the highp color-dodge and color-burn blends
(`pipeline/highp.rs:459,469`). The test SVGs use neither, and no transforms (no trig). tiny-skia
has no `target_os` cfg, so Windows x86-64 runs the Linux x86-64 code. **Windows was not driven.**
Emulation is not silicon. The build PR's first CI run is the real three-OS check.

**Chosen: one pinned hash per form**, the same constant for every form, plus an in-run equality
assertion. Each fix row asserts `hash == GO_HELLO_PIXELS_SHA256` **and** byte-equality to the Go
render made in the same test run. The second assertion runs first, so a failure message
separates "the fix is wrong" (relative fails) from "this OS rasterizes differently" (only the
pin fails). **If CI fails only the pin on one OS, the build stops and reports it. It does not
loosen the pin.** That would make the cross-platform reading here wrong, which is a design fact.

**Hash pixels, not file bytes.** `GO_HELLO_PIXELS_SHA256` is SHA-256 over `width.to_le_bytes()`,
`height.to_le_bytes()` and `image::open(out).to_rgba8().as_raw()` (the shape of
`tests/watermark_damage.rs:139-151`). A `png` encoder bump would change file bytes without
changing a pixel. The constant was computed outside crustyimg (ImageMagick `rgba:` +
`struct.pack('<II',200,60)`), and **`working_family_forms_keep_their_pinned_bytes` must be GREEN
on `main` before the fix**. That proves the test's hash recipe matches the constant.

### Call 4: how ink is counted

`ink(png) = image::open(png).to_rgba8().pixels().filter(|p| p.0[3] > 0).count()`. The upstream
`image` decoder reads the PNG file; no crustyimg code is involved. It is crustyimg's dependency,
but not crustyimg's pipeline. Its two controls:
- **positive:** every working form counts exactly **738** (ImageMagick also says 738), asserted
  as `GO_HELLO_INK = 738`;
- **negative:** the text-free SVGs (`<text T></text>`, with `font-family="sans-serif"`, and a
  single space) count **0**, asserted. These are the same SVGs with the text removed.

### Call 5: the browser build gets a test row

`src/image/mod.rs:595-597` sends every SVG to `svg::decode_svg` → `hardened_options()` with no
`cfg` split, so the fix covers wasm by construction. **A row is still worth it**, for one reason:
the wasm build declares its own `resvg` (`Cargo.toml:243`), and a feature set is per target. A
different `fontdb` feature set, or a missing face, would show only in the VM (lesson: wasm32
compiles is not wasm32 runs). Cost: one `#[wasm_bindgen_test]` and two inline SVG constants in
`tests/wasm_roundtrip.rs`. The row uses a **named** family (`Helvetica`), not a generic, so the
`set_serif_family` mutation turns it red. `just wasm-test` is **not in CI** (no workflow runs it:
`grep -rn wasm-test .github/workflows` → nothing). The build and verify run it locally, and
Build Completion records the result.

### Call 6: DEC-054 is amended, DEC-107 is released

The rule belongs in DEC-054 (text on, bundled font only). Its `## Decision` says "the default
family is set to the bundled font", and that sentence is the bug. Its `affected_scope` already
covers `src/image/**`. The amendment is worded under Implementation Context. **DEC-107 is
released unused.**

### Call 7: the `build` cache

The `build` cache key folds in `env!("CARGO_PKG_VERSION")` (`src/build/cache.rs:249-269`). A
release built with 0.8.0 that cached a blank SVG output gets a new key at 0.8.1's version bump,
so no stale blank output survives an upgrade. **No `CACHE_SCHEMA_VERSION` bump.** Residual,
accepted: a `main` build made after this merges but before the bump still reports 0.8.0 and
would reuse 0.8.0's cached blank output for an unchanged SVG. That affects dev builds only.

### Call 8: docs and CHANGELOG (worded now, written at build)

`docs/cli-reference.md` has **no SVG section** (the frame and prompt assumed one). Its only SVG
text is the `optimize` passthrough note at `:161-169`. The sentence goes in `## Conventions`
(`:11-21`) as a new last bullet:

> - **SVG text always draws in the bundled Go font.** crustyimg never reads the fonts installed on
>   your system, so `<text>` in any `font-family` (`sans-serif`, `Arial`, a family you do not
>   have) draws in Go. Bold and italic draw upright in Go Regular, and characters Go does not
>   contain (CJK, emoji) do not draw correctly.

`README.md`: a new paragraph directly before `#### HEIC is opt-in and never in a released binary`
(`:82`):

> SVG input is rasterized at its own size. Its text always draws in the bundled Go font,
> whatever `font-family` it names: crustyimg never reads your system's fonts, and bold or italic
> text draws upright in Go Regular.

`CHANGELOG.md`, a `### Fixed` subsection under `## [Unreleased]`:

> - **SVG text in any font family draws again.** Every release with SVG input (0.5.0 through
>   0.8.0) drew no text at all for `<text>` whose `font-family` was anything but `Go`, including
>   `sans-serif`, `serif`, `Arial`, `Helvetica` and lists without `Go`. It exited 0 with no
>   warning. That text now draws in the bundled Go font, on the command line and in the browser
>   build. System fonts are still never read, and bold and italic draw upright in Go Regular.

The versions are named because they were driven: 0.5.0 and 0.6.0 here, 0.7.1 by the frame,
0.8.0 (Homebrew) here.

### Size: [S] holds

Code: 5 setter lines plus 2 comment edits. Tests: 7 functions in 3 files. The two table tests
hold 18 + 6 rows; there are 3 control rows, 1 system-font row, 2 unit tests and 1 wasm row. That is
more rows than the frame's estimate of about 18 assertions, but no new mechanism: one pinned
constant (not one per architecture), no fixture files. The cross-platform answer added nothing.
Not grown past S; nothing cut.

## Inputs

- **Files to read:** `src/image/svg.rs` (`hardened_options()` `:177-195`, module doc `:42-50`,
  test module `:230`); `src/image/mod.rs:588-598` (the one SVG dispatch, native and wasm);
  `tests/input_svg.rs` (CLI-driving shape: `BIN`, `tempfile`); `tests/watermark_damage.rs:139-151`
  (pixel-hash shape); `tests/wasm_roundtrip.rs:30-95,486-502` (wasm module, `transform`,
  `IDENTITY_RECIPE`); `decisions/DEC-054-…md`.
- **External APIs:** none. `usvg-0.49.0/src/text/mod.rs:84-153` and `fontdb-0.24.0/src/lib.rs:183,
  401,619-679` for the behaviour cited in `## Design`.
- **Related code paths:** `src/image/` only.

## Outputs

- **Files created:** `tests/svg_font_family.rs`: the CLI table, identity, control and
  system-font rows.
- **Files modified:**
  - `src/image/svg.rs`: five `set_*_family` lines, doc comment, module doc, two unit tests;
  - `tests/wasm_roundtrip.rs`: one wasm row;
  - `docs/cli-reference.md`, `README.md`: one passage each (Call 8);
  - `CHANGELOG.md`: `### Fixed` under `## [Unreleased]`;
  - `decisions/DEC-054-svg-rasterize-dependency-resvg-usvg-tiny-skia.md`: an amendment.
- **New exports:** none. **Database changes:** none. **No** `Cargo.toml`/`Cargo.lock` change
  (`image`, `sha2`, `resvg`, `tempfile` are already reachable from tests), no version bump, no
  `CACHE_SCHEMA_VERSION` change.

## Acceptance Criteria

Surfaces: **CLI** = `crustyimg convert X.svg --format png -o X.png -y` (every verb that reads an
SVG goes through the same `src/image/mod.rs:595` dispatch, so `convert` stands for `resize`,
`optimize`, `apply --recipe` and `build`); **browser** = the wasm `transform` export; **lib** =
`hardened_options()`. Inputs, `T`, ink and the pixel hash are defined in `## Design`.
`GO_HELLO_PIXELS_SHA256 = 6e4d6b080052173623550a30e4e877658f5da28ddb711fbfad7bc46df2d46ad6`,
`GO_HELLO_INK = 738`.

- [ ] **AC-1. CLI: text in any font family draws exactly the Go render.** Each of these 18 forms
      exits 0 with empty stderr, has ink == 738, is pixel-equal to the `font-family="Go"` render
      from the same run, and hashes to `GO_HELLO_PIXELS_SHA256`: `sans-serif`, `serif`,
      `monospace`, `cursive`, `fantasy`; `Helvetica`, `Arial`, `DejaVu Sans`; `NoSuchFontXyz`;
      `"Helvetica, sans-serif"`; `"'Helvetica Neue', Arial, sans-serif"`;
      `style="font-family: sans-serif"`; `<style>text{font-family:Arial}</style>`; `go`;
      inherited from `<g font-family="sans-serif">`; `sans-serif` + bold; `serif` + italic (not driven; `Go` + italic was, ==Go);
      mixed `Arial` text with a `Go` `<tspan>`. **RED on `main`:** ink 0 on 17 rows, 253 on the
      tspan row.
- [ ] **AC-2. CLI: the forms that already drew keep their exact pixels.** none, `Go`,
      `"Helvetica, Go"`, `Go` + bold, `Go` + italic, and `Go` text with a `Go` `<tspan>` each
      exit 0 with empty stderr, have ink == 738 and hash to `GO_HELLO_PIXELS_SHA256`, on every CI
      OS. **GREEN on `main`**, and stays green.
- [ ] **AC-3. CLI: bold and italic draw upright in Go Regular.** Named separately because it
      is documented behaviour: `Go` + bold, `Go` + italic (AC-2 rows), `sans-serif` + bold and
      `serif` + italic (AC-1 rows) all equal the Regular render. That is no synthetic bold and no
      synthetic oblique.
- [ ] **AC-4. CLI: the fix draws nothing where there is no text.** `<text T></text>`, the same
      with `font-family="sans-serif"`, and `font-family="sans-serif"> </text>` each exit 0 with
      ink == 0. **GREEN on `main`**, and stays green (the ink counter's negative control).
- [ ] **AC-5. CLI: an installed system font is never read.** On a host that has the OS's control
      font (Call 2 table), its positive control holds: resvg with only that font draws ink > 0
      in an alpha channel different from Go's. Then the CLI render of `font-family="<that
      family>"` hashes to `GO_HELLO_PIXELS_SHA256`. macOS and Windows: an absent file is a
      failure. Linux: an absent file is a printed skip. **RED on `main`** wherever it runs (the
      CLI render is blank).
- [ ] **AC-6. lib: the font database resolves each generic family to Go on its own, and holds
      only Go.** For each of `Serif`, `SansSerif`, `Monospace`, `Cursive`, `Fantasy`:
      `hardened_options().fontdb.query(&Query { families: &[f], ..Default::default() })` is
      `Some(id)`, and that face's families include `"Go"`. That is the generic alone, with no
      `Serif` appended. The database has exactly one face, and `opt.font_family == "Go"`.
      **RED on `main`:** all five queries are `None`. The one-face and `font_family` checks are
      **GREEN on `main`**.
- [ ] **AC-7. browser: text in a named family draws in the wasm build.** `transform(SVG_NAMED,
      IDENTITY_RECIPE, "png")` on `<text … font-family="Helvetica">Hi</text>` (no background)
      returns PNG bytes whose decoded alpha has ink > 0. The text-free twin draws ink == 0.
      **RED on `main`** (ink 0). It runs under `just wasm-test`, which is **not in CI**.
- [ ] **AC-8. Docs say the rule.** `docs/cli-reference.md` `## Conventions` and `README.md`
      carry the Call 8 passages: always Go, never system fonts, bold and italic upright in
      Regular.
- [ ] **AC-9. CHANGELOG.** `## [Unreleased]` → `### Fixed` carries the Call 8 line, naming 0.5.0
      through 0.8.0 and both surfaces.
- [ ] **AC-10. DEC-054 is amended** with the rule, the mechanism, the structural no-system-font
      fact and the accepted limits (Implementation Context). DEC-107 is not used.

## Failing Tests

Written during **design**, BEFORE build. **7 tests: 4 RED on `main`, 3 GREEN guards** that
must stay green. Every table test asserts its own row count, so a dropped row is red.

- **`tests/svg_font_family.rs`** (new, `#![cfg(not(target_arch = "wasm32"))]`; drives
  `env!("CARGO_BIN_EXE_crustyimg")` in a `tempfile::tempdir()`, SVGs generated inline from the
  `## Design` bodies, no fixture files)
  - `every_family_form_draws_the_go_render`: **AC-1, AC-3**. 18 rows, `assert_eq!(rows.len(),
    18)`. Per row, in this order: exit 0; stderr empty; ink == 738; pixels == the in-run `Go`
    render; hash == `GO_HELLO_PIXELS_SHA256`. The message names the row. **RED on `main`:** ink
    0 (the tspan row 253), hash `bb918147…`.
  - `working_family_forms_keep_their_pinned_bytes`: **AC-2, AC-3**. 6 rows, count asserted. Exit
    0, stderr empty, ink == 738, hash == `GO_HELLO_PIXELS_SHA256`. **GREEN on `main`** (driven:
    all six hash `6e4d6b08…` on arm64 `main` and on x86-64 0.8.0). **Confirm it green on `main`
    before writing the fix.** That proves the test's hash recipe matches the constant.
  - `text_free_svgs_draw_no_ink`: **AC-4**. 3 rows, count asserted. Exit 0, ink == 0.
    **GREEN on `main`.**
  - `an_installed_system_font_is_never_read`: **AC-5**. Per Call 2: pick the OS's font file;
    absent → macOS/Windows `panic!`, Linux `eprintln!` SKIP + return. Positive control through
    `resvg` with only that face loaded (ink > 0, alpha ≠ Go's alpha), then CLI render hash ==
    `GO_HELLO_PIXELS_SHA256`. **RED on `main`** (blank render) on macOS; on Windows by the same
    reasoning (not driven).
- **`src/image/svg.rs`** `#[cfg(test)] mod tests`
  - `hardened_fontdb_resolves_each_generic_family_to_go`: **AC-6**. Loop over the five
    `usvg::fontdb::Family` generics, one `query` each with `families: &[f]`, and assert `Some`
    and a `"Go"` family on the face, with a message naming `f`. **RED on `main`:** all five `None`.
  - `hardened_options_hold_only_the_bundled_face`: **AC-6**. `opt.fontdb.len() == 1` and
    `opt.font_family == "Go"`. **GREEN on `main`.**
- **`tests/wasm_roundtrip.rs`** (`mod wasm`, `#[wasm_bindgen_test]`)
  - `svg_text_in_a_named_family_draws_in_wasm`: **AC-7**. Two inline 40×30 SVG constants
    (text `Hi`, `font-size="10"`, `font-family="Helvetica"`, no background; and the same with the
    text removed) go through `transform(…, IDENTITY_RECIPE, "png")`, then `image::load_from_memory`
    → alpha ink. Assert > 0 and == 0. **RED on `main`** (0). Run with `just wasm-test`.

AC-8, AC-9 and AC-10 are documents. Verify reads them against Call 8 and the amendment text,
and there is no test for them.

### Negative controls for verify (one per independent condition)

Each is a source mutation on the build branch. Rebuild **clean** (fresh `CARGO_TARGET_DIR`, or
`cargo clean -p crustyimg`) and confirm the binary changed before reading a result.

1. **Remove `set_serif_family`.** → RED: `every_family_form_draws_the_go_render` rows `serif`,
   `serif` + italic, `Helvetica`, `Arial`, `DejaVu Sans`, `NoSuchFontXyz`, `<style>` Arial, `go`
   and mixed tspan; `an_installed_system_font_is_never_read` (where it runs);
   `hardened_fontdb_…[Serif]`; the wasm row. Rows that **stay green** are expected, because each
   one names a generic that still resolves: the other four generics, both lists, `style=`,
   inherited, `sans-serif` + bold. This was driven: the "four others, no serif" probe column.
2. **Remove `set_monospace_family`** (likewise any one of sans-serif, cursive, fantasy). → RED:
   `hardened_fontdb_resolves_each_generic_family_to_go` naming `Monospace`, **only**. Every CLI
   row stays green, because usvg's `serif` last resort covers it end-to-end (driven: the
   "`set_serif_family` only" probe is identical to all five). This mutation is the reason the
   unit test exists.
3. **Remove `opt.font_family = …`.** → RED: `hardened_options_hold_only_the_bundled_face` only.
   The CLI `none` row stays byte-identical (driven).
4. **Load a system font.** Add `db.load_font_data(std::fs::read("/System/Library/Fonts/Helvetica.ttc").unwrap())`
   (macOS, temporary). `load_system_fonts` will not compile, because `fontdb` has no `fs`
   feature, and that is itself the structural guard. → RED: `an_installed_system_font_is_never_read`
   (Helvetica now resolves to itself), `hardened_options_hold_only_the_bundled_face` (`len` 2),
   the AC-1 `Helvetica` and `"Helvetica, sans-serif"` rows, and the AC-2 `"Helvetica, Go"` row.
5. **Break the ink counter** (`p.0[3] > 0` → `true`). → RED: `text_free_svgs_draw_no_ink` and
   the ink assertion in every 738 row.
6. **Wrong pin.** Flip one hex digit of `GO_HELLO_PIXELS_SHA256`. → RED: every AC-1, AC-2 and
   AC-5 row, on the hash assertion. In AC-1 the in-run equality still passes, which proves the
   two checks are independent.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-054`: SVG text on, bundled font only, never system fonts. **Amend it** (text below);
  DEC-107 is released.
- `DEC-045`: the bundled BSD-3 Go font (`assets/fonts/Go-Regular.ttf`). No new face is bundled
  (maintainer ruling 2).
- `DEC-034`: decode caps. Untouched; the change is inside the options, before the caps.
- `DEC-066`: wasm size gate. No new bytes: five setter calls on an existing struct.
- `DEC-063` names `src/image/svg.rs` (the pixel budget in `check_caps`). Untouched.

**DEC-054 amendment, to append verbatim** (after `## Validation`, before `## References`):

```markdown
## Amendment (2026-10-28): every font family draws in the bundled font

"The default family is set to the bundled font" covered only text that names no family. Text
that named any other family, including `sans-serif` and `serif`, found no face and drew
nothing, exit 0, in every release with SVG input (0.5.0 to 0.8.0). That was the
silent-wrong-output this decision set out to prevent. The rule is now:

- **SVG text always draws in the bundled Go font, whatever `font-family` it names:** generic,
  named, installed on the host or not, a list, a CSS property, or a `<style>` rule.
- **How:** `hardened_options()` points fontdb's five generic families (`serif`, `sans-serif`,
  `monospace`, `cursive`, `fantasy`) at Go, and keeps `font_family = "Go"` for text with no
  family. usvg's default selector tries `serif` after every family a text names
  (`usvg-0.49.0/src/text/mod.rs:114`), so `serif` → Go catches every named or unknown family. No
  custom font resolver is needed. **Revisit on a usvg bump** if that fallback line changes:
  `tests/svg_font_family.rs` goes red on the named-family rows if it does.
- **Substitution is silent** (maintainer ruling, 2026-10-28). A warning would fire on nearly
  every real SVG, which names Arial or Helvetica. An opt-in `lint` rule is the place for that
  signal if it is ever wanted.
- **Accepted limits:** bold and italic draw upright in Go Regular, with no synthetic bold or
  oblique and no extra faces bundled (ruling, 2026-10-28; bundling faces would be a DEC-066
  purchase). Characters Go lacks (CJK, emoji) still do not draw correctly.
- **System fonts cannot be read**, structurally: `fontdb` builds with no features (fingerprint
  `"features": []`), so `load_system_fonts` and the other filesystem loaders are not compiled in.
  `tests/svg_font_family.rs` also checks it behaviourally with an installed font per CI OS.
```

### Constraints that apply

- `untrusted-input-hardening`: no new input surface; the hardening options are unchanged except
  the family table.
- `pure-rust-codecs-default`: no new dependency.
- `every-public-fn-tested`: no new public fn. `hardened_options()` gains two unit tests.
- `clippy-fmt-clean`: `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`,
  default and `--no-default-features`.

### Prior related work

- `SPEC-060` (shipped): SVG rasterize, DEC-054, the bundled-font text.
- `SPEC-119` (shipped): the `animated_input` carrier. It is the pattern a warning would have
  needed, and was ruled out.
- `PR #211` (merged): resvg 0.49. Its verify found this bug. The fix has the same shape on 0.47
  (the bug predates it).
- `SPEC-138`: the other 0.8.1 silent-output spec. Its build **also edits `CHANGELOG.md`,
  `README.md` and `docs/cli-reference.md`.** The two builds run one after the other. Whichever
  builds second rebases over the first's `## [Unreleased]` entry and keeps both `### Fixed`
  lines.

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- **A warning naming the substituted family.** Ruled out (maintainer ruling 1). An opt-in `lint`
  rule for SVG fonts would be a separate, unscheduled item, not filed by this spec.
- **Glyph coverage beyond Go** (CJK, emoji, most non-Latin scripts): measured 136 px of `.notdef`
  for `漢字` in `Go`, unchanged by the fix. It is the one-bundled-font limit DEC-054 chose. This spec
  only documents it.
- **Bundling Go Bold / Italic faces.** Ruled out (maintainer ruling 2): a DEC-066 size purchase.
- **Reading system fonts.** Still never: DEC-054. The negative control above guards it.
- **`info` reporting a rasterized SVG as `png`.** DEC-054's accepted wart, unrelated.
- **Running `just wasm-test` in CI.** It belongs on STAGE-053 (guards), and is not filed here.
- **Any `build`/CLI flag work.** That is SPEC-138.

## Notes for the Implementer

- **Order:** write `tests/svg_font_family.rs` and the two unit tests first. Run them on
  unmodified `main` and record the RED/GREEN split in Build Completion: 4 red, 3 green, with
  `working_family_forms_keep_their_pinned_bytes` green. Then apply the Call 1 code.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. Read `$?` from a redirected run, never through a pipe.
- The system-font row needs `resvg`/`usvg` in an integration test. Both are normal dependencies
  of the crate, so `use resvg::{tiny_skia, usvg};` works. Compare **alpha channels**: tiny-skia's
  premultiplied alpha equals straight alpha, so no demultiply is needed. Gate the per-OS path
  table with `cfg!(target_os = …)`.
- `fontdb::Query` derives `Default`. `Database::face(id)` gives `FaceInfo.families:
  Vec<(String, Language)>`.
- Do **not** enable any `fontdb` feature, or add `fontdb` as a direct dependency. Reach it as
  `usvg::fontdb`.
- Run `just wasm-test` and the clean full matrix (`just check` plus `--no-default-features`)
  before you open the draft PR. Record both in Build Completion, along with the
  `-- --nocapture` line for the system-font row.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids.

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
