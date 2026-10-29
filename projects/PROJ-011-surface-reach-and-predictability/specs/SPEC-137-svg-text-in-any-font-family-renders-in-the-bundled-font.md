---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-137
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
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

- **A warning naming the substituted family.** Only if the maintainer picks option 2: its own spec
  ([M], `Image` carrier + 5 CLI print sites), or an opt-in `lint` rule. Not filed until ruled.
- **Glyph coverage beyond Go** (CJK, emoji, most non-Latin scripts): measured 136 px of `.notdef`
  for `漢字` in `Go`, unchanged by the fix. It is the one-bundled-font limit DEC-054 chose. This spec
  only documents it.
- **Bundling Go Bold / Italic faces.** A DEC-066 size purchase, not a bug fix (maintainer question 2).
- **Reading system fonts.** Still never: DEC-054. The negative control above guards it.
- **`info` reporting a rasterized SVG as `png`.** DEC-054's accepted wart, unrelated.
- **Any `build`/CLI flag work.** That is SPEC-138.

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
