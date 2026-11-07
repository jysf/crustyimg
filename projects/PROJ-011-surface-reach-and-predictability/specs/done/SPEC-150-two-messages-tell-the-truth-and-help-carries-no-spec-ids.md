---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-150
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-063
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: null
  created_at: 2026-11-06

references:
  decisions:
    - DEC-007   # exit codes: an unreadable/absent input is exit 3
    - DEC-003   # metadata dual lane; its pixel-lane preserve claim is false (fold-in 8)
    - DEC-010   # directory sources are non-recursive (top level only)
    - DEC-030   # meta copy is JPEG only
    - DEC-015   # the shared CLI fan-out and api-contract
    - DEC-050   # lint (its help text loses the id)
    - DEC-057   # build overwrites its outputs (its help text loses the id)
    - DEC-059   # --frozen/--locked/--strict (their help text loses the id)
  constraints: []
  related_specs:
    - SPEC-148  # lint's fix lines; `fix: null` cases make "(each with a runnable fix)" false
    - SPEC-149  # the guard table and the Cli::command() walk this spec's guard copies
    - SPEC-128  # the wasm asset refusal this spec rewords

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: >
  Delivers STAGE-063's "two misleading messages say what is true": a folder with no images
  says so, and no rendered help page, completion script or browser message shows an internal
  spec or DEC id.

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
      tokens_total: 5243487
      tokens_breakdown:
        input: 88
        output: 42243
        cache_creation: 133456
        cache_read: 5067700
      estimated_usd: 4.42
      duration_minutes: null
      recorded_at: 2026-11-06
      notes: "Measured from the transcript before the commit/push turns (44 deduped message ids; Opus $5/$25, cache_creation x1.25, cache_read x0.10)."
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 10565790
      tokens_breakdown:
        input: 126
        output: 86506
        cache_creation: 203264
        cache_read: 10275894
      estimated_usd: 8.57
      duration_minutes: null
      recorded_at: 2026-11-06
      notes: "Measured from the transcript before the commit/push turns (63 deduped message ids; Opus $5/$25, cache_creation x1.25, cache_read x0.10). Includes a throwaway sketch build + drive, reverted."
    - cycle: build
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 21442186
      tokens_breakdown:
        input: 212
        output: 98391
        cache_creation: 244333
        cache_read: 21099250
      estimated_usd: 14.54
      duration_minutes: null
      recorded_at: 2026-11-06
      notes: "Measured from the transcript before the final bookkeeping/CI-read turns (106 deduped message ids, MAX output per id; Opus $5/$25, cache_creation x1.25, cache_read x0.10). Includes the 25-row mutation run and two full just check runs."
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 8979718
      tokens_breakdown:
        input: 120
        output: 51414
        cache_creation: 180529
        cache_read: 8747655
      estimated_usd: 6.79
      duration_minutes: null
      recorded_at: 2026-11-06
      notes: "Punch list at 5996c6b (R1, R2). Figures from the timeline's verify record (deduped by message id, MAX output; Opus $5/$25, cache_creation x1.25, cache_read x0.10); re-priced here: 0.0006 + 1.2854 + 1.1283 + 4.3738 = 6.79. Entry added at ship: the verify cycle had not been written into cost.sessions."
    - cycle: fix
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 1983916
      tokens_breakdown:
        input: 48
        output: 8846
        cache_creation: 67038
        cache_read: 1907984
      estimated_usd: 0.96
      duration_minutes: null
      recorded_at: 2026-11-06
      notes: "Measured from the transcript before the bookkeeping turns (24 deduped message ids, MAX output per id; Sonnet $3/$15, cache_creation x1.25, cache_read x0.10)."
    - cycle: ship
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 1606982
      tokens_breakdown:
        input: 36
        output: 6045
        cache_creation: 77401
        cache_read: 1523500
      estimated_usd: 0.84
      duration_minutes: null
      recorded_at: 2026-11-07
      notes: "Measured from the transcript before the final edit, commit and push turns (18 deduped message ids, MAX output per id; Sonnet $3/$15, cache_creation x1.25, cache_read x0.10), so a slight under-count."
  totals:
    tokens_total: 49822079
    estimated_usd: 36.12
    session_count: 6
---

# SPEC-150: Two messages tell the truth, and --help carries no spec ids

## Context

STAGE-063 is PROJ-011's last stage; SPEC-148 (`lint` fixes what it names) and SPEC-149 (no flag
is accepted and ignored) shipped. This is the stage's third and last spec before the 0.8.3
release cycle. It carries two backlog items moved from STAGE-054 (a folder with only subfolders
reports "input not found or unreadable"; the browser's asset refusal shows `SPEC-128`) and
SPEC-149's leftover (5): rendered `--help` still shows spec/DEC ids. The frame rules on five
candidate fold-ins from the same stage (findings B and C, and SPEC-149's leftovers 1, 3 and 4).

## Goal

**Today** every one of the 13 folder-taking verbs driven (`build` targets included) answers an empty folder, a folder of subfolders and a folder of non-images with
`error: input not found or unreadable: <dir>` (exit 3), though the folder exists and was read;
and all 44 rendered help pages (22 commands × `-h`/`--help`) show internal ids such as
`(SPEC-064)` and `(DEC-059)`, as do four of five completion scripts and the browser build.
**After**, a folder with no images says that and how to reach subfolders (exit 3 kept), and a
test that walks the real command tree keeps every help page, completion script and user-facing
source string free of `SPEC-`/`DEC-` ids.

## Frame

**Verdict: GO, one spec, complexity M** (2026-11-06, on `main` at `4fc30ca`, `crustyimg 0.8.2`,
debug profile, own `CARGO_TARGET_DIR`). Rulings pending: see "Open maintainer questions".

### Premise check (driven)

Every premise holds. Item 1 is wider than filed: it is not only "only subfolders" but any folder
whose top level holds no image (empty, subfolders only, non-images only), on every
input-resolving verb, and `build` repeats it per target. The wasm refusal is real
(`src/wasm.rs:236-240`, id on line 239). The help sweep finds the 8 known doc comments and no
others; the completion scripts are a surface the backlog did not name.

### 1. Reproduced on `main`

Fixture shapes: `sub/` (only `sub/inner/a.png`), `txt/` (only `notes.txt`), `empty/`, and a
control `img/` (one 16×16 PNG). Every cell below is `error: input not found or unreadable: <dir>`,
exit 3, stdout empty, unless noted.

| verb (flags) | `sub/` | `txt/` | `empty/` | `img/` (control) |
|---|---|---|---|---|
| `resize --max 8 -o r.png -y` | 3 | 3 | 3 | 0 |
| `resize --max 8 --out-dir od` | 3 | 3 | 3 | 0 |
| `resize --max 8` (stdout) | 3 | 3 | 3 | 0, 143 B |
| `convert --format jpeg --out-dir` | 3 | 3 | 3 | 0 |
| `thumbnail --out-dir` | 3 | 3 | 3 | 0 |
| `web --out-dir` | 3 | 3 | 3 | 0 |
| `optimize --out-dir` / `-o` | 3 | 3 | 3 | 0 |
| `auto-orient --out-dir` / `edit --invert --out-dir` | 3 | 3 | 3 | 0 |
| `watermark --text hi --out-dir` | 3 | 3 | 3 | 2 (mark does not fit, true) |
| `apply --recipe r.toml --out-dir` | 3 | 3 | 3 | 0 |
| `meta strip --out-dir` | 3 | 3 | 3 | 0 |
| `info` / `lint` | 3 | 3 | 3 | 0 |
| `build` (target `source = "<dir>"`) | 3 ¹ | 3 ¹ | 3 ¹ | 0 |
| `responsive --widths 8 --out-dir` | 3 ² | 3 ² | 3 ² | **3 ²** |
| `diff <dir> a.png` | — | — | — | **3 ²** |

¹ `error: target #0 (recipe r.toml, out bo_sub) error: input not found or unreadable: sub`.
² `error: could not read image input`: `responsive` and `diff` take one file, and say a readable
folder could not be read, even when it holds an image.

Controls: a missing path (`resize nope`) gives the same line, which is true there. The suggested
glob works: `resize 'sub/**/*' --max 8 --out-dir gl` exits 0, `lint 'sub/**/*'` exits 0. A glob
that matches only non-images (`resize 'txt/*'`) also says "not found or unreadable: txt/*". A
`chmod 000` folder says the same, which is true. **What is false:** "not found or unreadable" for
a folder that exists and was listed. **What is true:** the folder has no image at its top level
(folders are not searched; `'d/**/*'` searches them).

Cause: `resolve_dir` returns an empty `Vec` for a folder with no images
(`src/source/mod.rs:255`, documented), and each caller turns "nothing resolved" into
`SourceError::NotFound(arg)`: `src/cli/ops.rs:42`, `src/cli/build.rs:333`,
`src/cli/optimize.rs:146` and `:1619`, `src/cli/report.rs:227` and `:505`. A glob that matches
no image returns `NotFound` from `src/source/mod.rs:247`. The `responsive`/`diff` line is
`ImageError::Io`'s display (`src/error.rs`), reached by reading a directory as a file.

The browser refusal (`src/wasm.rs:236-240`, a `format!`):
`recipe step {index} ('{op}') needs its '{key}' asset resolved from a file, which the wasm
surface cannot do (no filesystem) — supplying assets over this boundary is out of scope
(SPEC-128)`. Besides the id, "wasm surface" and "boundary" are internal words to a browser user.

### 2. The id sweep (measurement)

**Help pages.** Command (scratch script, walks the binary, recursing on each page's
`Commands:` list, so it sees the real tree):

```sh
walk() { for flag in -h --help; do "$B" "$@" $flag | grep -cE '(SPEC|DEC)-[0-9]'; done
  for s in $("$B" "$@" --help | awk '/^Commands:/{c=1;next} c&&/^[^ ]/{c=0} c&&/^  [a-z]/{print $1}' | grep -v '^help$'); do walk "$@" "$s"; done; }
```

Result: **44 pages** (root, 20 leaves, `meta`; × `-h`/`--help`, matching
`tests/cli_flags_honest.rs`'s `LEAF_COUNT = 20`), **44 of 44 carry ids, 207 id-bearing lines,
17 distinct rendered lines**. By page: root `-h` 6 / `--help` 7; `build` 5 / 6; `lint` 5 / 6;
`meta` 4 / 6; every other verb, `meta strip|clean|copy|set` included, 4 / 5 (the global flags).
They come from **8 doc comments** in `src/cli/mod.rs`: the global flags `--no-cache` (`:114-115`,
SPEC-064), `--check` (`:119-120`, SPEC-066) and its `--frozen`/`--locked` paragraph (`:122-124`,
DEC-059, `--help` only), `--strict` (`:133-135`, DEC-059), `--watch` (`:139-143`, SPEC-067), and
the descriptions of `meta` (`:444-447`, SPEC-087, SPEC-089, DEC-017; `meta --help` only),
`build` (`:491`, SPEC-063, DEC-057) and `lint` (`:499-500`, SPEC-050, DEC-050).

**Completion scripts** (`completions <shell>`), which embed the same help: bash 0, zsh 136,
fish 92, powershell 136, elvish 136 id-bearing lines. Fixing the 8 doc comments fixes these.
`--version` (`crustyimg 0.8.2`): 0.

**Source strings.** The prompt's grep `grep -rn '"[^"]*\(SPEC\|DEC\)-[0-9]' src` gives 20 lines,
all comments or test assertions, and **misses** the wasm hit (the id sits on a continuation
line of a multi-line literal). A non-comment sweep
(`grep -rnE '(SPEC|DEC)-[0-9]' src | grep -vE '^[^:]+:[0-9]+:\s*//'`) gives 7 lines:
`src/wasm.rs:239` (user-facing) and six test-assertion messages (`src/operation/registry.rs:657`,
`src/operation/mod.rs:2563`, `src/cli/build.rs:1515`, `src/image/avif.rs:698`, `:960`,
`src/build/cache.rs:945`), which no user sees. No `#[error(...)]` attribute carries an id.

**Comments:** 1072 comment lines in `src/` carry an id, 718 of them `///`/`//!` doc comments.
Only the 8 above render. The maintainer's "comments plain, no spec refs" preference would apply to
all of them; that is a finding, not scope (question 3).

**Browser demo** (`demo/`, user-visible text only): `demo/demo.js:434` ("…it has no AVIF decoder,
DEC-065)", shown in the score panel; the claim itself is true: `re_rav1d` is native-only per
`Cargo.toml:159`) and `demo/index.html:163` ("as the CLI (DEC-069, resolved)", a visible
paragraph). The other ~40 demo hits are JS/HTML comments. The wasm RAW messages
(`src/wasm.rs:624`, `:630`) and the missing-format message (`:215`) carry no id and say nothing
false.

**User docs:** `docs/cli-reference.md` has 6 id lines (`:40` DEC-085, `:302`, `:311-312`
SPEC-110/111, `:343` DEC-057, `:447` SPEC-088/DEC-074); `:312` also says a divergence "lands in
SPEC-111", a forward reference to work that has since shipped. `README.md`, `skills/crustyimg/SKILL.md`,
`docs/github-actions.md` and `docs/recipes.md`: 0. `SECURITY.md` cites ids as references for
researchers (out of scope). `docs/USAGE.md` and `GETTING_STARTED.md` are framework docs.

### 3. The guard

One test file, `tests/help_has_no_internal_ids.rs`, in-process (no binary spawn), three checks:

1. **Help walk:** recurse over `crustyimg::cli::Cli::command()` (as `leaf_paths()` in
   `tests/cli_flags_honest.rs:221` does, but visiting the root and every non-leaf too), call
   `render_help()` **and** `render_long_help()` on each, and fail on `(SPEC|DEC)-\d+`, naming the
   command path and line. Assert the number of pages visited is at least 44 and print it, so a
   walk that visits nothing cannot pass (trap: a harness that exercises nothing reports green).
2. **Completions:** run `clap_complete::generate` for every `Shell` into a buffer and apply the
   same check.
3. **Source strings:** scan every `src/**/*.rs` file up to its first `#[cfg(test)]` line, drop
   `//` comment lines, and fail on an id inside the rest. This sees runtime-formatted strings
   (the wasm `format!`), `#[error]` attributes and `JsError` text, which no help walk can. Build the
   search pattern so the test cannot match its own literal (`concat!("SPEC", "-")`), and
   normalise CRLF before splitting (the `include_str!` self-check lesson).

**Mutation rows verify should run** (each names its location, per trap 12):
(a) put `(SPEC-064)` back on `--no-cache` (`src/cli/mod.rs:115`): checks 1 and 2 red;
(b) add an id to the `meta` long description only: check 1 red **via `render_long_help` only**
(proves both forms are walked); (c) add an id to `meta set`'s about: red (proves recursion);
(d) put `(SPEC-128)` back in the wasm `format!` (`src/wasm.rs:239`): check 3 red;
(e) make the walk skip every subcommand: the page-count assert red.
The id-free fixes need their own tests for item 1 and the fold-ins; that is design's job.

**Where an id could still hide:** a string assembled from pieces (`"SPEC" + "-064"`); text in a
dependency; anything after a file's first `#[cfg(test)]` that is not a test (the scan assumes
tests sit at the bottom); `demo/*.js`/`*.html` (not Rust; comments and visible text mix, so a
scan is not cheap: fix the two lines and leave them unguarded, or add a narrow check on string
literals only); and docs. Hidden args (`--keep-gps`, `hide = true`) do not render, so they are
not user-visible. `after_help`, `before_help` and `ValueEnum` variant help all render into
`render_long_help`, so the walk sees them.

### 4. Replacement text

Rule for the rest: drop the parenthetical id. If the id stood for a reason, say the reason in
words; never replace an id with a doc path.

| Where | Proposed text |
|---|---|
| `build` (`:491`) | `Run a declared build: every `[[target]]` in a build manifest.` |
| `lint` (`:499-500`) | `Lint an image asset tree: report problems, most with a runnable `fix:` command, and exit with a CI-friendly code. Read-only: never writes an image.` |
| `meta` long (`:444-447`) | `` `meta` groups the metadata commands; none of them re-decodes pixels. `auto-orient` is not here: it bakes orientation into the pixels, so it is an image command.`` |
| `--check` (`:119-120`) | `` `build` only: check the build against the committed `crustyimg.build.lock` instead of refreshing it: exit 7 on drift, and never modify the lockfile.`` |
| `--frozen`/`--locked` (`:122-124`) | `` `--frozen` and `--locked` are aliases of `--check`: crustyimg never uses the network, so cargo's "don't update" and "don't hit the registry" mean the same thing here.`` |
| `--no-cache` (`:114-115`) | `` `build` only: bypass the cache: read no entries, write none, rebuild every input. The cache is on by default; this is the opt-out.`` |
| `--strict` (`:133-135`) | `` `build --check` only: fail, instead of only noting it, when output bytes differ because the toolchain or CPU differs. For a pinned toolchain and architecture that want byte-identical output.`` |
| `--watch` (`:139-143`) | `` `build` only: rebuild whenever a source, recipe or the manifest changes. Debounced (an editor's burst of saves is one rebuild); the build's own outputs, `.crustyimg` and the lockfile never trigger it. Ctrl-C exits. Incompatible with `--check`/`--frozen`/`--locked`. A rebuild under `--watch` does not rewrite the committed lockfile.`` |
| wasm refusal (`src/wasm.rs:236-240`) | `recipe step {index} ('{op}') needs a {key} file, and the browser has no filesystem to read it from, so file-based steps are out of scope here: remove the step, or run this recipe with the crustyimg CLI` (keeps "filesystem" and "out of scope", which `tests/wasm_roundtrip.rs:283`, a PROJ-017 file, asserts) |
| `demo/demo.js:434` | drop `, DEC-065`: `…the one output the engine can't score itself (it has no AVIF decoder).` |
| `demo/index.html:163` | drop `(DEC-069, resolved)` |
| folder with no images (new) | `error: no images in folder 'sub': only its top level is read; for subfolders use a glob such as 'sub/**/*'` (exit 3) |
| glob matching no image (new) | `error: no images matched 'txt/*'` (exit 3); a missing path keeps `input not found or unreadable: <arg>` |
| single-file verb given a folder | `error: 'img' is a folder; responsive takes one image file` (exit 3) |

`lint`'s line: SPEC-148 made `fix: null` real for stdin input, 16-bit PNG and formats the metadata
lane cannot write, so "each with" is false and "most with" is true. The `--strict` text must be
checked against what `--strict` compares (the design reads `src/build/lock.rs`).

### 5. Compatibility and docs

No golden test, Action input or `SKILL.md` quotes any help line or the folder message
(`grep -rlF` of each string over `README.md docs skills integrations npm demo tests .github`).
Tests that assert on changed text: `tests/wasm_roundtrip.rs:283` (**PROJ-017-owned**; the
proposed wording keeps its two tokens, so no edit) and `tests/cli_flags_honest.rs`'s
`OLD_MESSAGE = "could not read image input"` (asserts its *absence* for fonts; unaffected).
No test asserts `input not found or unreadable` (grep: 0 hits in `tests/`).

Docs that mirror a changed line:
- `docs/recipes.md:148` "each with a runnable fix" — **PROJ-017-owned**: do not edit; hand to the
  PROJ-017 thread or a later sweep.
- `docs/cli-reference.md:418` (the `--frozen`/`--locked` alias note, no id) and its 6 id lines.
- `skills/crustyimg/SKILL.md` (**PROJ-017-owned**) documents "a directory reads only its top-level
  images" — still true; no edit needed.

Proposed CHANGELOG (`Fixed`; **the Unreleased section is PROJ-017-owned**: coordinate the edit):
> - A folder with no images at its top level now says so and suggests a `'dir/**/*'` glob,
>   instead of "input not found or unreadable"; `build` targets too. `responsive` and `diff`
>   given a folder say it is a folder.
> - `--help`, the shell completions and the browser build no longer show internal spec or
>   decision ids, and `lint --help` no longer promises a fix for every finding.
> - `optimize -q`/`web -q` name the real conflict; `meta copy` and the other metadata commands
>   say plainly which formats they support; a path with a `"` is quoted correctly in the
>   font/watermark-image error. *(each only if its fold-in is ruled in)*

### 6. Size and scope

**M.** Two strands (item 1 changes a source error and six callers; items 2–3 are text plus one
guard file), each S, plus three S fold-ins. They share the theme and the verification shape
(drive the message, assert the text), so **no split**. If the maintainer rules fold-in 8 in
with a DEC amendment, it stays M.

Verification cost: native `cargo test` (one new guard file, new message rows), the lean
`--no-default-features` leg, and the wasm leg (`scripts/wasm-ci-check.sh`) for the refusal text.
Estimate: design ~$5, build ~$10, verify ~$7.

#### Fold-in rulings

| # | Item | Ruling | Reason |
|---|---|---|---|
| 4 | `optimize -q 10` / `web -q` blame `--target/--ssim/--max-size` (driven: exit 2, though none was passed; `src/cli/optimize.rs:490-502`, called at `:692` and `:838`) | **IN** | A message that names flags the user did not pass is this spec's theme; one function, not a PROJ-017 path. Keep exit 2. |
| 5 | `metadata lane does not support meta copy supports JPEG only in v1 yet` (`src/metadata/mod.rs:39` wraps `:319`'s sentence). Siblings built the same way: `does not support Gif yet`, `… WebP yet` (Rust enum names via `{other:?}`, `:69`), and `… unknown yet` (`:70`), on `meta strip|clean|set|copy` | **IN** | Same theme, one file outside PROJ-017's paths; fix the template and its four producers together. |
| 6 | `--keep-gps=false` gets clap's `unexpected value 'false'` (exit 2) | **OUT** | clap's message is true (the hidden flag takes no value) and the exit is the same; making a hidden placeholder take a value adds parser surface for nothing. Stays in STAGE-063's leftovers item. |
| 7 | `could not read font file "a"b.ttf"` (also `watermark image "m"k.png"`; `src/cli/mod.rs:~640`, `what` + `path`) | **IN** | Same file as the help text. There is no shared path-display helper in `src/` (grep for `shell_quote`/`fn quote`: 0); use Rust's escaped form (`{path:?}`-style) for this one message. Other messages print paths bare. |
| 8 | DEC-003's preserve claim | **IN as docs-only, pending ruling (question 4)** | Not "probably nothing": DEC-003's SPEC-149 amendment says the ICC/copyright half "is not corrected here", `AGENTS.md:503` still says "keep ICC + copyright/artist", `docs/architecture.md:150` says "keeps orientation + ICC + copyright". Driven: `resize` and `convert` of `tests/fixtures/c2pa/signed_gps.jpg` (ICC `sRGB IEC61966-2.1`) keep **no** ICC; SPEC-149's frame drove Artist/Copyright dropped. The amendment also says `meta copy` works for "JPEG and PNG"; the code refuses PNG (driven: PNG→PNG exit 4). |
| — | O-1/O-2 (`optimize` unreachable budget, `max_bytes = 0`) and `optimize`-to-stdout size | **OUT** | They are `optimize` behaviour, not messages. They stay where they are in STAGE-063's backlog (verify's optional findings on SPEC-148, and the stdout-size item). |

### Rough build file list

- `src/source/mod.rs` — a "no images" variant (or arg), keep exit 3 in `src/cli/mod.rs`'s map.
- `src/cli/ops.rs`, `src/cli/build.rs`, `src/cli/optimize.rs`, `src/cli/report.rs` — the six
  empty-result callers; `src/error.rs` or the single-file read path for the folder case.
- `src/cli/mod.rs` — 8 help doc comments; the asset-read message (fold-in 7); exit-code map.
- `src/cli/optimize.rs` — `reject_quality_with_auto` (fold-in 4).
- `src/metadata/mod.rs` — the `UnsupportedFormat` template and its producers (fold-in 5).
- `src/wasm.rs` — the refusal text.
- `demo/demo.js`, `demo/index.html` — two visible strings.
- `docs/cli-reference.md` — the id lines and any mirrored help text.
- `tests/help_has_no_internal_ids.rs` (new) and new message tests.
- Fold-in 8: `AGENTS.md`, `docs/architecture.md`, `decisions/DEC-003-metadata-dual-lane.md`.
- `CHANGELOG.md` Unreleased — **PROJ-017-owned section, coordinate**.

**PROJ-017 overlaps:** `CHANGELOG.md` Unreleased (edit needed, coordinate);
`tests/wasm_roundtrip.rs:283` (asserts the refusal's words; wording keeps them, no edit);
`docs/recipes.md:148` (mirrors `lint`'s line; do not edit); `skills/crustyimg/SKILL.md` (no edit
needed); `scripts/wasm-ci-check.sh` (run, not edited). `src/cli/mod.rs`, `src/wasm.rs` and
`docs/cli-reference.md` are not on PROJ-017's list but are shared hot files: rebase before build.
The six id-bearing test assertions in `src/operation/` (PROJ-017) stay as they are; the guard skips
test modules so it never needs them changed.

### Out of scope (for this spec specifically)

- Spec/DEC ids in code comments (1072 lines), unless question 3 rules otherwise.
- `SECURITY.md`'s ids (references for security researchers).
- `--keep-gps=false` (fold-in 6), O-1/O-2 and the `optimize`-to-stdout size item.
- Building a selective metadata preserve (PROJ-013 STAGE-052 holds the real `--keep-gps`).
- Searching subfolders: directory inputs stay non-recursive (DEC-010).
- Any edit under `src/operation/`, `src/recipe/`, `docs/recipes.md`, `skills/crustyimg/SKILL.md`,
  `tests/common/mod.rs`, `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`,
  `tests/wasm_roundtrip.rs`, `scripts/wasm-ci-check.sh`, or `projects/PROJ-017-*`.

### Open maintainer questions

1. **A folder with no images: error or quiet no-op?** Recommend **error, exit 3 kept**, with the
   new message. Consequence: CI that globs an empty folder still fails loudly, as today; only the
   words change.
2. **Fold-ins 4, 5 and 7 in?** Recommend **all three in**. Consequence: about +1 hour of build
   and three more message rows to verify; leaving them means 0.8.3 ships three known wrong
   messages.
3. **Do spec/DEC ids in code comments count?** Recommend **no** for this spec (1072 lines;
   a mechanical sweep with no user benefit), and file it as its own chore if wanted.
   Consequence: the guard scans only non-comment source lines.
4. **Fold-in 8 (DEC-003): amend now to "pixel-lane encodes drop all metadata"?** Recommend
   **yes, docs-only** (`AGENTS.md:503`, `docs/architecture.md:150`, a DEC-003 amendment, and its
   "JPEG and PNG" corrected to JPEG only). Consequence: the design writes a DEC amendment (no new
   DEC id), and 0.8.3 stops shipping a glossary that contradicts `SECURITY.md`.
5. **`docs/cli-reference.md`'s ids** (6 lines, one a stale "lands in SPEC-111"): in? Recommend
   **yes**. Consequence: one user doc fewer with internal ids; ~10 lines.

## Acceptance Criteria

Each AC names its checks (FT ids below). Every check names what turns it red.

- [ ] **AC-1 A folder with no images says so (exit 3), from one place.** A readable folder whose top level holds no
  image (`sub/` holding only `sub/inner/a.png`; `txt/` holding only `notes.txt`; `empty/`) makes every verb that
  resolves inputs print exactly
  `error: no images in folder '<dir>': only its top level is read; to include subfolders, use a glob such as '<dir>/**/*'`
  and exit 3, writing nothing. "Every verb" = each of the 8 callers that used to build `NotFound` from an empty
  result: `view`, the pixel fan-out (`resize`, `convert`, `thumbnail`, `auto-orient`, `edit`, `watermark`, format-pinned
  `optimize --format png`), the metadata fan-out (`meta strip|clean|set`), `build` (per target, prefixed by
  `error: target #N (recipe R, out O)`), `apply` (plain recipe), the auto-decide fan-out (`optimize --out-dir`,
  `web --out-dir`), `info`, `lint`. The text comes from one variant, `SourceError::NoImagesInFolder`, built in one
  function, `source::resolve_all`; no caller builds a source error itself. — FT-M1, FT-M2, FT-U1, FT-U2.
- [ ] **AC-2 The folder name in the message is derived, not echoed.** `sub/` (trailing separator) prints `'sub'`
  and the hint `'sub/**/*'`, never `'sub/'` or `'sub//**/*'`. — FT-M3 (the derived-vs-naive row, trap 10), FT-U1.
- [ ] **AC-3 A glob that matches no image says so (exit 3).** `'txt/*'` and `'nope/*'` print
  `error: no images matched '<glob>'`, exit 3. — FT-M4, FT-U1.
- [ ] **AC-4 The true old message stays where it is true.** A missing path (`nope`) keeps
  `error: input not found or unreadable: nope` (exit 3). A folder that exists but cannot be listed (`chmod 000`,
  Unix) keeps it too, on the fan-out verbs **and on `lint`**, which skips unresolvable arguments: the wrong-cause
  trap (a helper that re-checks `is_dir()` after the fact calls an unreadable folder "no images"; driven: `lint locked`
  reaches the empty-result branch). — FT-M5, FT-U2.
- [ ] **AC-5 Several arguments: behaviour unchanged, message names the first cause.** `resize a.png empty/` still
  exits 0 (an empty folder among others is not an error, as on `main`). When nothing resolves, the error is the
  first argument's cause in argument order (`resize sub/ empty/` names `'sub'`; `lint nope sub` names `nope` with
  the not-found text; `lint sub nope` names `'sub'` with the folder text). `lint 'txt/*' a.png` still exits 0. —
  FT-M6, FT-U2.
- [ ] **AC-6 A single-file verb given a folder says it is a folder (exit 3).** `responsive --widths 8 --out-dir od img`,
  `diff img a.png`, `diff a.png img`, `meta copy --from img --to a.png -o o.jpg` and `meta copy --from a.jpg --to img`
  print `error: '<arg>' is a folder, not an image file`, exit 3, even though `img/` holds an image. A missing file
  keeps `error: could not read image input` (exit 3). — FT-M7.
- [ ] **AC-7 `-q` names the real conflict (exit 2 kept).** `optimize -q 10 a.png -o o.png` →
  `error: -q/--quality cannot be used with optimize, which chooses the quality itself; to encode at a fixed quality, use \`crustyimg convert --format <FORMAT> -q N\``;
  `web -q 10 …` the same with `web`. `optimize -q 10 --target high …` and `--ssim 80` →
  `error: -q/--quality cannot be combined with --target or --ssim, which choose the quality to reach a score`;
  `optimize -q 10 --max-size 2k …` and `convert --format jpeg -q 10 --max-size 2k …` →
  `error: -q/--quality cannot be combined with --max-size, which chooses the quality to fit the size`. All exit 2.
  The advice is true: the printed command, with `<FORMAT>` → `jpeg` and `N` → `10`, run on `a.png`, exits 0 and writes a non-empty file (the test runs the printed text). — FT-M8.
- [ ] **AC-8 The `meta` format messages read as sentences.** `meta strip|clean --gps|set` on a GIF/WebP →
  `error: the meta commands support JPEG and PNG only, and this file is GIF` (`… is WEBP`); on unrecognised bytes →
  `… and this file's format is not recognised`; `meta copy` with a non-JPEG side →
  `error: meta copy supports JPEG only, and the --from file is PNG` (or `the --to file is GIF`, or
  `the --from file's format is not recognised`). All exit 4. No message contains `does not support`, `yet`,
  `in v1`, or a Rust enum spelling (`Gif`, `WebP`). In a batch the per-input line keeps its `error: <path>: ` prefix
  and the run exits 6. — FT-M9, FT-U4.
- [ ] **AC-9 A path with `"` is quoted unambiguously.** `watermark --text hi --font 'a"b.ttf'` →
  `error: could not read font file "a\"b.ttf": …`; `--image 'nope"x.png'` → `… watermark image "nope\"x.png": …`
  (exit 3); a recipe step `image = 'm"q.png'` → `error: recipe step 0 ('watermark'): could not read 'image' file "m\"q.png": …`
  (exit 1). A backslash is **not** doubled (`C:\dir\f.ttf` prints as typed: Windows paths stay readable). Both
  messages call one helper. — FT-M10, FT-U3.
- [ ] **AC-10 No rendered help page shows an internal id.** Every page the user can print with `-h` or `--help`
  (root, `meta`, every leaf; 44 on `main`) carries no `SPEC-<digit>`/`DEC-<digit>`. The walk renders pages through
  the parser (`try_get_matches_from`), as the binary does, and asserts each is a help display. — FT-G1.
- [ ] **AC-11 The walk cannot silently shrink.** The set of command paths the walk visits equals the set derived
  independently from the rendered pages' `Commands:` sections, and that set includes a nested path (`meta set`).
  — FT-G2.
- [ ] **AC-12 No completion script shows an id.** `clap_complete::generate` for every `Shell::value_variants()`
  (5 on `main`) carries no id. — FT-G3.
- [ ] **AC-13 No user-facing string literal in production source shows an id.** Every string literal in
  `src/**/*.rs` outside `#[cfg(test)]` items carries no id; comments are out of scope (ruling 3). This covers the
  wasm refusal, `#[error]` attributes and `JsError` text. The scanner's hard cases are tested on their own. — FT-G4,
  FT-G5.
- [ ] **AC-14 The browser refusal is plain.** The wasm refusal reads
  `recipe step {index} ('{op}') reads its '{key}' from a file, and crustyimg-wasm has no filesystem access, so steps that read files are out of scope here: remove the step, or run the recipe with the crustyimg CLI`
  and contains neither an id nor "wasm surface"/"boundary". — FT-G4 (id), FT-W1 (existing
  `tests/wasm_roundtrip.rs:270-283` asserts `watermark`, `image`, `filesystem`, `out of scope`; run by
  `just wasm-check`), FT-U5 (wording, native).
- [ ] **AC-15 The demo page and embedded text show no id.** `demo/*.js` and `demo/*.html` with comments removed, and
  every file the binary embeds with `include_str!`, carry no id. — FT-G6.
- [ ] **AC-16 The help text stays true.** `lint`'s description says a fix is given "where one exists" (SPEC-148 made
  `fix: null` real); `--strict` says it compares lockfiles written on a different platform (target triple), which is
  what `src/build/lock.rs::diff` keys on. — FT-G1 does not prove truth, so: FT-M11 (`lint --help` contains
  `where one exists` and not `each with`; `--strict` long help contains `different platform`).
- [ ] **AC-17 Docs match.** `docs/cli-reference.md` carries no id (6 lines today) and its `:312` sentence no longer
  points at SPEC-111 (an unrelated spec; the `edit`/`apply` divergence is still open); its exit-3 row and
  `docs/api-contract.md:58` name the new causes; `AGENTS.md:501-503` and `docs/architecture.md:148-151` say pixel
  encodes drop all metadata; DEC-003 carries the SPEC-150 amendment (committed by design). — FT-D1 (a grep the build
  runs and pastes; docs have no green, so the build quotes each changed line).

## Failing Tests

Written during **design**. Native tests run in `cargo test`; FT-W1 runs in `just wasm-check`. Every binary-driven row
assigns the command's output first, then asserts exit code, then exact stderr (trap 15). Fixtures are built in a
`tempfile::TempDir` with `image` (16×16 PNG) and literal bytes (a 1×1 GIF, `b"not an image at all"` named
`junk.jpg`); no fixture comes from crustyimg itself. No row sets an environment variable.

- **`tests/messages_tell_the_truth.rs`** (new, native only: `#![cfg(not(target_arch = "wasm32"))]`)
  - `"a folder with no top-level image says so on every verb"` (FT-M1) — table: 17 verb rows × 3 shapes (`sub`,
    `txt`, `empty`) = 51 rows. Rows: `view`; `resize --max 8 --out-dir od`; `resize --max 8` (stdout);
    `convert --format jpeg --out-dir od`; `thumbnail --out-dir od`; `auto-orient --out-dir od`;
    `edit --invert --out-dir od`; `watermark --text hi --out-dir od`; `optimize --format png --out-dir od` (an
    explicit format pin → pixel fan-out; `-o o.png` is avoided because its pin depends on the resolved count); `optimize --out-dir od`; `web --out-dir od`; `apply --recipe r.toml --out-dir od` (`r.toml` = one
    `invert` step); `meta strip --out-dir od`; `meta clean --gps --out-dir od`; `meta set --artist x --out-dir od`;
    `info`; `lint`; and two `build` rows (FT-M2). Asserts per row: exit 3; stderr == the AC-1 line exactly (one
    line); stdout empty; **nothing written**: a SHA-256 snapshot of the whole temp tree (the
    `tests/cli_flags_honest.rs` pattern, copied, not imported) is identical before and after, and `od`/`o.png` do not
    exist. The failure message names the verb, shape and the stderr it got.
  - `"a build target whose source is a folder with no images says so and writes nothing"` (FT-M2) —
    `build b.toml` with `version = 1`, one target `source = "<shape>"`: exit 3, stderr ==
    `error: target #0 (recipe r.toml, out bo)\nerror: <AC-1 line>\n`; `bo/` not created. A two-target manifest
    (target 0 `source = "img"`, target 1 `source = "sub"`): exit 3, stderr names `target #1`, and **neither** `bo0/`
    nor `bo1/` exists (every target is validated before any write; driven on the sketch).
  - `"the folder name is derived from the argument"` (FT-M3) — `resize --max 8 --out-dir od sub/`: stderr names
    `'sub'` and `'sub/**/*'`; asserts it does **not** contain `'sub/'` or `sub//`. Then
    `resize 'sub/**/*' --max 8 --out-dir gl` exits 0 and writes `gl/a.png` (the hint works, driven).
  - `"a glob that matches no image says so"` (FT-M4) — `resize --max 8 --out-dir od 'txt/*'` and `'nope/*'`, and
    `lint 'txt/*'`: exit 3, stderr == `error: no images matched '<glob>'`.
  - `"a missing path and an unreadable folder keep the not-found message"` (FT-M5) — `resize --max 8 --out-dir od
    nope`, `lint nope`, `info nope`: exit 3, stderr == `error: input not found or unreadable: nope`. Unix only
    (`#[cfg(unix)]`, a separate `#[test]` so its absence on Windows is visible in the test list, not a printed skip):
    `chmod 000 locked`; the row **first asserts `std::fs::read_dir(locked)` fails** and panics with
    "precondition: running as root? an unreadable folder cannot be made here" if not; then `resize … locked`,
    `lint locked` and `lint locked sub` all print `error: input not found or unreadable: locked`, exit 3; restores
    the mode before asserting.
  - `"several arguments keep their behaviour and name the first cause"` (FT-M6) — `resize --max 8 a.png empty`
    exit 0 (stdout non-empty); `lint 'txt/*' a.png` exit 0; `resize --max 8 --out-dir od sub empty` → names
    `'sub'`; `lint nope sub` → `error: input not found or unreadable: nope`; `lint sub nope` → the folder line for
    `'sub'`. All failing rows exit 3.
  - `"a single-file verb given a folder says it is a folder"` (FT-M7) — the five AC-6 commands (with `img/` holding
    `img/a.png`, so a reader that resolved the folder would succeed): exit 3, stderr == `error: '<arg>' is a folder,
    not an image file`; `responsive` writes nothing. Controls: `responsive --widths 8 --out-dir od nope.png` and
    `diff nope.png a.png` keep `error: could not read image input`, exit 3.
  - `"-q names the conflict the user actually has"` (FT-M8) — the six AC-7 commands: exit 2, stderr == the exact
    line. Plus the command the message prints, with `<FORMAT>` → `jpeg` and `N` → `10`, run on `a.png`: exit 0, non-empty output (the advice works as printed).
  - `"meta format messages are sentences"` (FT-M9) — `meta strip -o o.gif a.gif`, `meta clean --gps -o o.webp
    a.webp` (`a.webp` = literal bytes `RIFF\x1a\0\0\0WEBPVP8L` + padding: `sniff` reads magic bytes only, so no
    WebP codec is needed, lean build included), `meta set --artist x junk.jpg` (stdout), `meta copy --from a.png --to b.jpg
    -o o.jpg`, `meta copy --from b.jpg --to a.gif -o o.gif`, `meta copy --from junk.jpg --to b.jpg -o o.jpg`
    (`b.jpg` a JPEG made with `image`): exit 4, stderr == the exact AC-8 line; each asserts the absence of
    `does not support`, ` yet`, `in v1`, `Gif`, `WebP`. Batch row: `meta strip --out-dir od a.gif a.png` exits 6,
    stderr's first line == `error: a.gif: the meta commands support JPEG and PNG only, and this file is GIF`.
  - `"a path with a double quote is quoted unambiguously"` (FT-M10) — the three AC-9 commands (paths that do not
    exist; no file with `"` is created, so the row runs on Windows, where `"` is not a legal filename character):
    stderr starts with the exact quoted prefix; a fourth row with `--font 'C:\dir\f.ttf'` asserts the stderr contains
    `"C:\dir\f.ttf"` (one backslash each). Exits 3, 3, 1, 3.
  - `"help text that changed is true"` (FT-M11) — `lint --help` contains `where one exists`, not `each with`;
    `build --help` (the `--strict` long help) contains `different platform`.
- **`tests/help_has_no_internal_ids.rs`** (new, native only)
  - `"no help page shows an internal id"` (FT-G1) — walk `Cli::command()` recursively (`get_subcommands`, skip
    `help`), collecting every command path including the root and `meta`. For each path × `["-h", "--help"]`:
    `let err = Cli::command().try_get_matches_from(["crustyimg", ..path, flag]).unwrap_err();` assert
    `err.kind() == ErrorKind::DisplayHelp`, then `let page = err.render().to_string();` normalise `\r\n` → `\n`,
    and collect every line with an id (`SPEC-`/`DEC-` followed by an ASCII digit). Fail listing `path flag: line`
    for every hit. ⚠ **Not `render_help()` on the sub-`Command`**: driven on `main`, that sees **18** id lines of
    207, because global flags are only propagated into a subcommand when clap builds the parsed command; the parsed
    page sees all 207 (equal to the frame's binary walk).
  - `"the help walk visits every page the user can reach"` (FT-G2) — derive the expected paths **from the rendered
    text**: render the root's `--help` through `try_get_matches_from`, take each line of its `Commands:` section
    (`^  <name>` until the next unindented line; skip `help`), and recurse on each the same way. Assert the walk's
    path set (minus `is_hide_set()` subcommands, none on `main`) **equals** the derived set; assert the derived set
    contains a path of length 2 (`meta set`); `println!` the page count (44 on `main`) for the build log, but the
    equality is the check (a printed count is invisible in CI, trap 11).
  - `"no completion script shows an internal id"` (FT-G3) — for every `clap_complete::Shell::value_variants()`:
    `generate(shell, &mut Cli::command(), "crustyimg", &mut buf)`, assert the script is non-empty and contains
    `crustyimg`, normalise CRLF, fail on any id line naming the shell. Assert the shells seen ==
    `value_variants().len()`. On `main`: bash 0, zsh 136, fish 92, powershell 136, elvish 136.
  - `"no production string literal shows an internal id"` (FT-G4) — list `src/**/*.rs` recursively, sorted (44
    files on `main`); assert ≥ 1 file and that `src/wasm.rs` and `src/cli/mod.rs` are in the list (a scan of an
    empty or wrong dir must not pass). For each file run `production_literals(text)` (below) and fail on any literal
    containing an id, naming `file:line` and the literal. On `main` it finds exactly **1** of 1191 literals:
    `src/wasm.rs:237` (the refusal; the id sits on a continuation line, which the frame's `grep` missed). With the
    test-item skip disabled it finds 7 of 4279: the 6 test assertions the frame listed plus the refusal.
  - `"the literal scanner handles the hard cases"` (FT-G5) — `production_literals` on fixture snippets, each
    built with `concat!` so this file cannot match its own pattern: `"x // ID"` (an id after `//` inside a string)
    → 1 hit; `// "ID"` → 0; `r#"a // ID ""#` → 1; `let q = '"'; // ID` (a `"` char literal before a comment) → 0;
    `/* "ID" */` and nested `/* /* */ "ID" */` → 0; `"\"ID"` → 1; `"line \⏎   (ID)"` (continuation) → 1;
    `#[cfg(test)] mod t { "ID" } fn after() { "ID2" }` → exactly 1 (`ID2`: code after a test module is scanned);
    `fn f<'a>(x: &'a str) { "ID" }` (lifetimes) → 1; `#[error("bad (ID)")]` → 1; `br"ID"` → 1;
    `b'"'` then `"ID"` → 1. (All 11 non-`b'` cases were run on a probe at design; all matched.)
  - `"the demo page and embedded text show no internal id"` (FT-G6) — `demo/*.js` and `demo/*.html`, with `/* … */`,
    `<!-- … -->` and `//`-to-end-of-line (when `//` is at line start or after whitespace, so `https://` survives)
    removed: no id line. And every path named by an `include_str!("…")` in `src/**/*.rs` (derived by scanning, not
    listed; 3 recipe TOMLs on `main`), read whole: no id line. Assert the demo glob found ≥ 2 files and the
    `include_str!` scan found ≥ 1. On `main` it finds exactly 2: `demo/demo.js:434`, `demo/index.html:163`.
- **`src/source/mod.rs`** (existing `#[cfg(test)] mod tests`)
  - `"resolve_all explains an empty result by its cause"` (FT-U1) — temp dirs: empty dir → `NoImagesInFolder("d")`;
    `"d/"` → `NoImagesInFolder("d")` (trimmed); `"d/*.png"` with no match → `NoImagesMatched`; missing → `NotFound`;
    a dir with one PNG → `Ok(len 1)`. Display of `NoImagesInFolder("sub")` == the AC-1 text.
  - `"resolve_all skips or fails per OnEmpty and names the first cause"` (FT-U2) — `[img, empty]` Fail → Ok(1);
    `[sub, empty]` Fail → `NoImagesInFolder("sub")`; `[missing, sub]` Skip → `NotFound("missing")`;
    `[sub, missing]` Skip → `NoImagesInFolder("sub")`; `[bad[glob]` (invalid pattern, if `glob` rejects it) Skip →
    `InvalidPattern` still propagates. `#[cfg(unix)]`: a `chmod 000` dir under Skip → `NotFound` (precondition
    asserted as in FT-M5).
- **`src/cli/mod.rs`** (existing tests module)
  - `"new source and folder errors exit 3"` (FT-U3a) — `CliError::Source(NoImagesInFolder(..)).code() == 3`,
    `NoImagesMatched` → 3, `InputIsAFolder` → 3.
  - `"quoted escapes double quotes only"` (FT-U3) — `quoted("a\"b") == "\"a\\\"b\""`, `quoted(r"C:\d\f") ==
    r#""C:\d\f""#`, `quoted("plain") == "\"plain\""`.
- **`src/metadata/mod.rs`** (existing tests module)
  - `"unsupported-format messages are sentences"` (FT-U4) — `strip_all(gif_bytes)` Display == the GIF sentence;
    `strip_all(b"junk")` == the not-recognised sentence; `copy_metadata(png, jpeg)` == `… the --from file is PNG`;
    `copy_metadata(jpeg, gif)` == `… the --to file is GIF`. The existing `matches!(…, UnsupportedFormat(_))` asserts
    stay.
- **`src/wasm.rs`** — the refusal is compiled for wasm32 only, so its wording is pinned by FT-W1 (existing,
  PROJ-017-owned, unedited) and, natively, by FT-U5: the build moves the sentence into a `pub(crate) fn
  asset_refusal(index, op, key) -> String` in a module compiled on both targets **only if** that is a ≤ 10-line
  change; otherwise FT-U5 is dropped and FT-W1 + FT-G4 carry AC-14 (record which in Build Completion).
- **FT-D1** (docs) — the build runs
  `grep -nE '(SPEC|DEC)-[0-9]' docs/cli-reference.md; grep -n 'SPEC-111\|keep ICC\|ICC + copyright' docs/cli-reference.md AGENTS.md docs/architecture.md`
  and pastes the (empty) output, plus each changed line, in Build Completion.

### Baseline the build records before changing code (trap 13)

Run the new test files on unchanged `main` code and quote each RED row's failure message. Expected wrong-reason
risk: FT-M1's `view` rows could fail on "not a terminal" instead of the message (driven: `view sub` resolves
before the tty check, so it prints the folder line; quote it). FT-G1 must be red with 207 hit lines across 44 pages;
FT-G3 red on 4 shells (bash green, it carries no help text); FT-G4 red on `src/wasm.rs:237` only; FT-G6 red on the 2
demo lines; FT-G2 and FT-G5 green on `main` (they test the harness, not the fix).

### Mutation table (trap 12): apply each alone, record every test red

| # | Mutation (exact location) | Expected red | Expected green (proves independence) |
|---|---|---|---|
| 1 | `src/cli/ops.rs` `run_pixel_op`: restore the old loop + `if all.is_empty() { return Err(…NotFound(inputs.join(" "))) }` verbatim | FT-M1 rows `resize` ×2, `convert`, `thumbnail`, `auto-orient`, `edit`, `watermark`, `optimize --format png` (8 rows × 3 shapes), FT-M3 | FT-M1 `view`, `meta *`, `apply`, `optimize --out-dir`, `web`, `info`, `lint`; FT-M2 |
| 2 | `src/cli/ops.rs` `run_metadata_lane`: same restore | FT-M1 `meta strip|clean|set` ×3 | all other FT-M1 rows |
| 3 | `src/cli/build.rs` prepare-target: restore `NotFound(target.source.as_slice().join(" "))` | FT-M2 (both) | FT-M1 |
| 4 | `src/cli/optimize.rs` `run_apply` step 5: restore | FT-M1 `apply` ×3 | others |
| 5 | `src/cli/optimize.rs` `run_optimize_autodecide`: restore | FT-M1 `optimize --out-dir`, `web` ×3 | `optimize --format png` rows (pinned path, mutation 1's) |
| 6 | `src/cli/report.rs` `run_info` / `run_view` in `ops.rs`: restore `resolve` + `.ok_or(NotFound(input))` | FT-M1 `info` (resp. `view`) ×3 | others |
| 7 | `src/cli/report.rs` `run_lint`: restore the old match-and-join | FT-M1 `lint` ×3 (old: `input not found or unreadable: sub`); FT-M6 `lint nope sub` (old joins `nope, sub`), `lint sub nope`, and `lint 'txt/*' a.png` (old arm does not skip `NoImagesMatched`: exit 3, not 0); FT-M5 Unix `lint locked sub` (old: `locked, sub`) | FT-M4 `lint 'txt/*'` (the glob error now propagates with the new text either way), FT-M5 `lint nope`, `lint locked` (old text is the same) |
| 8 | `source::resolve_all`: build the empty-folder cause as `NotFound(arg)` (the helper returns the missing-path text) | all 51 FT-M1 rows, FT-M2, FT-M3, FT-M6 folder rows, FT-U1, FT-U2 | FT-M4, FT-M5 (proves the controls do not depend on the helper) |
| 9 | `source::resolve_all`: classify after the fact with `Path::new(arg).is_dir()` instead of `Ok(empty)` | FT-M5 Unix rows (`lint locked`, `lint locked sub`), FT-U2 Unix row | FT-M1 (proves the wrong-cause row is the only one that sees it) |
| 10 | `resolve_all`: drop the trailing-separator trim | FT-M3, FT-U1 `"d/"` row | FT-M1 |
| 11 | `src/cli/mod.rs:115`: put `(SPEC-064)` back on `--no-cache` | FT-G1 (every page: it is global), FT-G3 zsh/fish/powershell/elvish | FT-G4 (doc comments are not literals), FT-G3 bash |
| 12 | `meta set`'s doc comment (nested leaf): append `(DEC-017)` | FT-G1 on `meta set -h`/`--help` **and** `meta -h`/`--help` (the `Commands:` list shows each leaf's short help), FT-G3 (4 shells) | FT-G1 root and every other leaf |
| 13 | `meta`'s long description (second paragraph): add `(SPEC-087)` | FT-G1 on `meta --help` **only** (proves both forms are walked; `-h` stays clean) | FT-G3 (long about is not in completions; record if it is) |
| 14 | A `ValueEnum` variant doc (e.g. `QualityTarget::High`): add `(DEC-023)` | FT-G1 on `optimize --help`, FT-G3 zsh/fish (they carry possible-value help; record the shells that go red) | FT-G4. No completion-only text source was found: the frame traced every completion id line on `main` to the 8 doc comments that also render in help |
| 15 | `src/wasm.rs` refusal `format!`: add ` (SPEC-128)` | FT-G4 (`src/wasm.rs`), FT-W1 stays green (its tokens remain) | FT-G1, FT-G3 |
| 16 | `src/cli/mod.rs` `Lint` doc comment: restore `(each with a runnable fix)` | FT-M11 | FT-G1 (no id), proving FT-M11 is the truth check |
| 17 | `reject_quality_with_auto`: restore the old body verbatim | FT-M8 rows 1–2 (`optimize`, `web`), rows for `--max-size` (old lists `--target/--ssim`) and `--target` | FT-M8 `convert -q` exit-0 row |
| 18 | `quoted()`: return `format!("\"{path}\"")` (escaping removed) | FT-M10 rows 1–3, FT-U3 | FT-M10 backslash row |
| 19 | `quoted()`: use `format!("{path:?}")` | FT-M10 backslash row, FT-U3 `C:\d\f` | — |
| 20 | FT-G1's walk visits only the root (return before recursing) | FT-G2 (walk set ≠ derived set) | FT-G1 still red on `main`'s root page; green after fix — which is why FT-G2 exists |
| 21 | FT-G2's derivation parses nothing (wrong section header) | FT-G2 (derived set lacks `meta set`) | — |
| 22 | `production_literals`: stop skipping `#[cfg(test)]` items | FT-G4 red on the 6 test assertions (5 files, 2 PROJ-017-owned: do not edit them, revert) | — |
| 23 | `production_literals`: treat `//` inside a string as a comment | FT-G5 row 1 and continuation row | — |
| 24 | `src/metadata/mod.rs` `sniff`: restore `format!("{other:?}")` under `#[error("{0}")]` | FT-M9 rows, FT-U4 | FT-M7 |
| 25 | `refuse_folder`: removed from `run_diff` only | FT-M7 `diff` rows (old `could not read image input`) | FT-M7 `responsive`, `meta copy` rows |

Each red row's failure message must name its cause (trap 13): the FT-M rows print the stderr they got; FT-G rows print
`path flag: line` / `file:line`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Design calls (settled)

1. **One place for an empty result: `source::resolve_all`.** New in `src/source/mod.rs`:
   ```rust
   pub enum OnEmpty { Fail, Skip }   // Skip only for lint
   pub fn resolve_all<S: AsRef<str>>(args: &[S], reader: &mut impl Read, on_empty: OnEmpty)
       -> Result<Vec<Input>, SourceError>
   ```
   It calls `resolve` per argument in order. `Ok(empty)` comes **only** from the directory branch (stdin and a single
   file yield one input; a glob errors on empty), so `Ok(empty)` *is* the typed "readable folder, no top-level image"
   cause: record `NoImagesInFolder(trimmed arg)` as the first cause, keep going. Under `Skip`, `Err(NotFound | NoImagesMatched)`
   is recorded as a cause and skipped (lint's old behaviour); any other error, and every error under `Fail`,
   propagates at once (exactly as the `?` in today's loops). If nothing resolved, return the first recorded cause.
   Trim trailing `/` and `\` from the folder name; if that leaves `""` (the argument was `/`), use the argument.
   Never `unwrap`: an empty `args` slice (unreachable through the CLI) returns `NotFound(String::new())`.
   **Why not re-check the path in a helper after the fact:** `lint` swallows `NotFound` from an unreadable folder, so
   a helper that asks `is_dir()` afterwards calls a `chmod 000` folder "no images" (driven: `lint locked` reaches
   that branch). **Why not make `resolve` fail on an empty folder:** `resize a.png empty/` exits 0 today; the ruling
   is wording only.
2. **New variants** (`SourceError`, exit 3 each, explicit arms in `CliError::code`, whose match is exhaustive per
   variant so a missing arm is a compile error):
   `#[error("no images in folder '{0}': only its top level is read; to include subfolders, use a glob such as '{0}/**/*'")] NoImagesInFolder(String)`;
   `#[error("no images matched '{0}'")] NoImagesMatched(String)`, returned by `resolve_glob` on an empty match
   instead of `NotFound`. `NotFound` keeps its text and now means only a missing/unreadable path.
3. **The 8 callers** (the frame listed 6; it missed `ops.rs:346` `run_pixel_op` and `ops.rs:779`
   `run_metadata_lane`, the two fan-outs most verbs use): `ops.rs:42` (view), `:346`, `:779`, `build.rs:333`,
   `optimize.rs:146` (`run_apply`, plain recipe), `:1619` (`run_optimize_autodecide`), `report.rs:227` (info),
   `:505` (lint, `Skip`). Each becomes one `source::resolve_all(…)?` call; delete the loop and the
   `is_empty` check. `view`/`info` keep their `.next()` with the existing `ok_or` (now unreachable; no `unwrap`).
   `common.rs:346` `resolved_input_count` is not an error path; leave it on `resolve`. Remove the then-unused
   `SourceError` imports (clippy).
4. **Single-file verbs:** one helper `pub(super) fn refuse_folder(arg: &str) -> Result<(), CliError>` in
   `src/cli/common.rs`, returning new `CliError::InputIsAFolder(String)`
   (`#[error("'{0}' is a folder, not an image file")]`, exit 3). Called at the top of the read in `run_responsive`
   (once), `run_diff` (a, then b), `run_copy_metadata` (from, then to). Not in `Image::load`: `ImageError`'s
   messages are path-free by design (`src/error.rs` doc), and the CLI owns the argument name. Checking `is_dir()`
   first also makes Windows (where reading a folder is "access denied") print the same line.
5. **`-q`:** `reject_quality_with_auto(auto, global, verb: &str)` in `src/cli/optimize.rs:490`. When `-q` is given:
   `None` → Ok; `Some(Fast)` → the AC-7 "cannot be used with {verb}" line (the default `optimize` and every `web`
   trip this: `optimize_auto_config` returns `Fast` with no search flag, `run_web` hard-codes it);
   `Some(Perceptual(_))` → the `--target or --ssim` line; `Some(SizeBudget(_))` → the `--max-size` line (fixes
   `convert`, which has only `--max-size`, being told about `--target/--ssim`). Callers pass `"convert"`,
   `"optimize"`, `"web"`. **Exit 2 stays:** both flags are valid alone; together they are a usage conflict
   (DEC-007's 2), and `tests/cli.rs:3288` and scripts key on it.
6. **`meta` messages:** `MetadataError::UnsupportedFormat` keeps its shape; its template becomes `#[error("{0}")]`
   and each producer writes the whole sentence. `sniff` (`:65-71`): the AC-8 "meta commands … this file is {NAME}"
   / "not recognised" lines, `NAME = format!("{other:?}").to_ascii_uppercase()` (same rule as `ops.rs::format_name`).
   `copy_metadata` (`:317`): check `from` then `to` with `guess_format`, naming `--from`/`--to`. "The meta commands"
   is true: `sniff`'s error is reached only from `strip_all`/`clean_gps`/`set_tags`/`copy_metadata`, called only by
   `meta strip|clean|set|copy` (`has_manifest` maps it to `false`).
7. **Quoting:** one `fn quoted(path: &str) -> String` in `src/cli/mod.rs` that wraps in `"` and escapes `"` only.
   Used by `WatermarkAssetUnreadable` (`:640`) and `RecipeAssetUnreadable` (`:624`), the only two messages that wrap a
   path in `"` (grep of `\"{` over `src/` outside JSON/HTML writers). `{path:?}` was rejected: it doubles every
   backslash in a Windows path.
8. **Help text** (`src/cli/mod.rs`; rule for any other: drop the parenthetical id; if it stood for a reason, say the
   reason in words; never replace an id with a doc path):

   | Line | New text |
   |---|---|
   | `--no-cache` `:114-115` | `` `build` only: bypass the cache: read no entries, write none, rebuild every input. The cache is on by default; this is the opt-out.`` |
   | `--check` `:119-120` | `` `build` only: check the build against the committed `crustyimg.build.lock` instead of refreshing it: exit 7 on drift, and never modify the lockfile.`` |
   | `--frozen`/`--locked` `:122-124` | `` `--frozen` and `--locked` are aliases of `--check`: crustyimg never uses the network, so cargo's "don't update" and "don't hit the registry" mean the same thing here.`` |
   | `--strict` `:133-135` | `` `build --check` only: fail, instead of only noting it, when an output's bytes differ from a lockfile written on a different platform (target triple). For a team on one pinned toolchain that wants byte-identical output everywhere.`` |
   | `--watch` `:139-143` | `` `build` only: rebuild whenever a source, recipe or the manifest changes. Debounced (an editor's burst of saves is one rebuild); the build's own outputs, `.crustyimg` and the lockfile never trigger it. Ctrl-C exits. Incompatible with `--check`/`--frozen`/`--locked`. A rebuild under `--watch` does not rewrite the committed lockfile.`` |
   | `meta` long `:444-447` | `` `meta` groups the commands that edit metadata without re-decoding pixels. `auto-orient` is not here: it bakes orientation into the pixels, so it is an image command.`` |
   | `build` `:491` | `` Run a declared build: every `[[target]]` in a build manifest.`` |
   | `lint` `:499-500` | `Lint an image asset tree: report problems, with a runnable fix where one exists, and exit with a CI-friendly code. Read-only: never writes an image.` |

   Keep each doc comment's line structure (the first sentence is the short help).
9. **Browser refusal** (`src/wasm.rs:236-240`): AC-14's sentence. Keeps `watermark`/`image` (from `{op}`/`{key}`)
   and `filesystem`/`out of scope`, which `tests/wasm_roundtrip.rs:270-283` asserts; true in a browser and in Node
   (the npm package is `crustyimg-wasm`, a user-facing name). Also reword the code comment above it only if it is a
   one-line touch (comments are out of scope; leave it otherwise).
10. **Demo:** `demo/demo.js:434` → `"the one output the engine can't score itself (it has no AVIF decoder)."`;
    `demo/index.html:163` → `as the CLI. Bytes are not guaranteed identical — this build is a`.
11. **The guard's scanner** (`production_literals(text) -> Vec<(line, String)>` in the test file, ~60 lines): a
    character lexer over CRLF-normalised text with states code / line comment / block comment (nested) / string
    (`\` escapes the next char, newlines counted) / raw string (`r#*"` … `"#*`, also after `b`). A `'` starts a char
    literal if the next char is `\` (skip to the closing `'`) or the char after next is `'`; otherwise it is a
    lifetime. `#[cfg(test)]` arms a flag; the next `{` at code level starts a skipped region until its matching
    `}`; a `;` first disarms it (an attribute on a non-block item: its literals are scanned, conservatively).
    The scan resumes after the test module, so code after it is checked. Collect literal contents outside skipped
    regions. Id test: `SPEC-`/`DEC-` + ASCII digit, prefixes written `concat!("SP", "EC-")`, `concat!("DE", "C-")`.
    A probe of exactly this design ran on `main` (numbers in FT-G4/FT-G5).
12. **Where an id could still hide** (stated, not guarded): a string assembled at runtime from pieces
    (`format!("{}-{}", "SPEC", 64)`); text inside a dependency (clap's own strings); a test-only item not written as a
    `#[cfg(test)]` block (scanned, so a false red, not a miss); docs other than `cli-reference.md` (`SECURITY.md` keeps
    its ids by ruling); comments (ruling 3). Hidden args do not render. `after_help`, `before_help` and possible-value
    help all render into the parsed pages, so FT-G1 sees them.

### Docs (exact changes; ⚠ = PROJ-017-owned, do NOT edit, hand to the orchestrator)

- `docs/cli-reference.md:40` drop ` (DEC-085)`; `:302` drop ` (SPEC-110)`; `:311-312` replace "This divergence is
  introduced by SPEC-110 (before it, `edit` never baked either); flagged, not fixed — lands in SPEC-111." with
  "This divergence is known and not yet fixed."; `:343` drop ` (DEC-057)`; `:447` drop ` (SPEC-088, DEC-074)`;
  `:53` exit-3 row → `Input not found or unreadable, or a folder or glob with no images, or a folder where one image
  file is expected.`; mirror the `--strict` and `lint` help sentences wherever this file restates them (the build
  greps `strict` and `lint` sections and quotes what it changed).
- `docs/api-contract.md:58` → `| \`3\` | Input not found or unreadable, or a folder or glob with no images, or a folder where one image file is expected. |`
- `AGENTS.md:501-503` (glossary "Default-preserve policy") → `On pixel-lane encodes: **bake** orientation into
  pixels (SPEC-110, amending DEC-003) and drop all other metadata, ICC and copyright/artist included (DEC-003,
  amended by SPEC-150). \`--keep-gps\` is refused (SPEC-149); \`meta copy\` (JPEG only) copies metadata back.`
  (AGENTS.md is agent-facing; ids are fine there.)
- `docs/architecture.md:148-151` → `Read via \`kamadak-exif\` (read-only); edit via \`img-parts\` (EXIF/ICC
  segments) and \`little_exif\` (tag write). Pixel-lane encodes drop all metadata (orientation is baked into the
  pixels first); only the \`meta\` commands keep or edit it (DEC-003).`
- `decisions/DEC-003-metadata-dual-lane.md` — **done in the design commit**: the SPEC-149 amendment's "(JPEG and
  PNG)" corrected to "(JPEG only …)", and a SPEC-150 amendment added.
- ⚠ `docs/recipes.md:148` "each with a runnable fix" → "with a runnable fix where one exists" (PROJ-017-owned).
- ⚠ `CHANGELOG.md` Unreleased (PROJ-017-owned): the proposed `Fixed` text is in the design report; do not write it.

### Files the build touches (none PROJ-017-owned; shared files marked, smallest hunk)

- `src/source/mod.rs` — 2 variants, `NotFound`→`NoImagesMatched` at `:247`, `OnEmpty` + `resolve_all`, unit tests.
- `src/cli/mod.rs` (**shared**) — 2 exit arms + `InputIsAFolder` variant/arm, 8 doc comments, 2 `#[error]` lines,
  `quoted()`, unit tests. Doc-comment hunks only touch the listed lines.
- `src/cli/ops.rs`, `src/cli/build.rs`, `src/cli/report.rs`, `src/cli/optimize.rs` — the 8 callers; `refuse_folder`
  calls; `reject_quality_with_auto` and its 3 callers.
- `src/cli/common.rs` — `refuse_folder`.
- `src/metadata/mod.rs` — template + `sniff` + `copy_metadata` check; unit tests.
- `src/wasm.rs` (**shared**) — the refusal sentence only (+ FT-U5 helper only if ≤ 10 lines).
- `demo/demo.js`, `demo/index.html` — one string each.
- `docs/cli-reference.md` (**shared**), `docs/api-contract.md`, `docs/architecture.md`, `AGENTS.md` (**shared**) —
  the lines above.
- `tests/messages_tell_the_truth.rs`, `tests/help_has_no_internal_ids.rs` — new.
- Not touched: `src/operation/**`, `src/recipe/**`, `tests/common/mod.rs`, `tests/wasm_roundtrip.rs`,
  `tests/recipe_corpus_output.rs`, `tests/fixtures/recipe_corpus/`, `scripts/wasm-ci-check.sh`, `docs/recipes.md`,
  `recipes/README.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`, `projects/PROJ-017-*`.

### Verification the build runs

`cargo fmt --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test` (native, all); the baseline and the
mutation table above; `cargo build --no-default-features` **and** `cargo test --no-default-features --test
help_has_no_internal_ids --test messages_tell_the_truth` (the lean leg; every fixture is a PNG/JPEG made with `image`'s default
codecs or literal magic bytes, so no row needs a feature gate; if one does, name the gate in Build Completion); `just wasm-check` (FT-W1). Own `CARGO_TARGET_DIR`. Push a checkpoint commit after the source
change, after the guard file, and after the message tests.

**Size: M.** If verification pushes past M, the cut is FT-G6's `include_str!` half and FT-U5 (both are second nets);
report before cutting.

### Decisions that apply

- `DEC-007` — exit codes: unreadable/absent input 3, usage 2, unsupported format 4; the new causes are 3.
- `DEC-010` — directory sources are non-recursive; the message states it and points at the glob.
- `DEC-003` — the dual lane; amended here (pixel encodes drop all metadata; `meta copy` JPEG only).
- `DEC-030` — `meta copy` is JPEG only; the new message says so.
- `DEC-050`, `DEC-057`, `DEC-059` — what the removed ids in `lint`, `build` and `--frozen`/`--strict` help stood for;
  the new sentences keep their behaviour statements.
- `DEC-015` — the shared CLI fan-out (`run_pixel_op`) and `docs/api-contract.md`.

### Constraints that apply

- `no-unwrap-on-recoverable-paths` — `resolve_all`'s empty-args fallback, `view`/`info`'s kept `ok_or`.
- `clippy-fmt-clean`, `every-public-fn-tested` (`resolve_all`, `OnEmpty`), `test-before-implementation`.
- `single-image-library` — `src/source` still never decodes; `refuse_folder` only stats.

### Prior related work

- `SPEC-149` (shipped) — `tests/cli_flags_honest.rs`: the command-tree walk and SHA-256 tree snapshot this spec copies.
- `SPEC-148` (shipped) — `fix: null` cases that make "each with a runnable fix" false.
- `SPEC-128` (shipped) — the wasm asset refusal and `tests/wasm_roundtrip.rs`'s dedicated-refusal assertion.

### Out of scope (for this spec specifically)

As in `## Frame` → "Out of scope", plus: making an empty folder a no-op (ruling 1); changing multi-argument
behaviour; `resolved_input_count`; ids in `SECURITY.md`, code comments and test assertions.

## Notes for the Implementer

- The build's `meta strip` batch row prints `error: a.gif: …` then `error: 1 of 2 inputs failed`; assert the first
  line only.
- `build`'s error is two `error:` lines (the anyhow chain); FT-M2 asserts both.
- Every literal in the new test files that contains an id is built with `concat!`, so a future scan of `tests/`
  cannot match the guard itself.
- Comments in new code stay plain: no spec or DEC ids (the maintainer's comment rule; the guard does not check it).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-150-two-messages-tell-the-truth`
- **PR (if applicable):** #257 (draft)
- **All acceptance criteria met?** yes (AC-1 to AC-17). AC-14's native wording check FT-U5 was dropped by the spec's
  own rule (see Deviations); AC-14 is carried by FT-W1 (green, 53/53 wasm tests) and FT-G4.
- **New decisions emitted:** none. DEC-003's SPEC-150 amendment was committed by design (`613bf23`); confirmed, not
  re-amended.
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`, 18 files):
  - `AGENTS.md` — glossary "Default-preserve policy": pixel encodes drop all metadata, ICC and copyright included.
  - `demo/demo.js` — `:434` drops `, DEC-065`.
  - `demo/index.html` — `:163` drops `(DEC-069, resolved)`.
  - `docs/api-contract.md` — exit-3 row names the new causes.
  - `docs/architecture.md` — metadata-lane glossary: pixel encodes drop all metadata.
  - `docs/cli-reference.md` — 5 id lines cleaned (`:40`, `:302`, `:343`, `:447`; `:311-312` now "This divergence is
    known and not yet fixed."); exit-3 row names the new causes.
  - `src/cli/build.rs` — build target sources resolve through `resolve_all`; unused `SourceError` import removed.
  - `src/cli/common.rs` — new `refuse_folder`.
  - `src/cli/mod.rs` — 8 doc comments (no ids; `lint` "where one exists"; `--strict` "different platform"),
    `CliError::InputIsAFolder` + 3 exit arms, `quoted()` used by both asset messages, 2 unit tests.
  - `src/cli/ops.rs` — `view`, `run_pixel_op`, `run_metadata_lane` use `resolve_all`; `meta copy` calls
    `refuse_folder` on both sides.
  - `src/cli/optimize.rs` — `run_apply` and `run_optimize_autodecide` use `resolve_all`; `responsive` calls
    `refuse_folder`; `reject_quality_with_auto(.., verb)` names the real conflict, 3 callers pass their verb.
  - `src/cli/report.rs` — `info` and `lint` (`OnEmpty::Skip`) use `resolve_all`; `diff` calls `refuse_folder` on a, b.
  - `src/metadata/mod.rs` — `UnsupportedFormat` is `#[error("{0}")]`; `sniff` and `copy_metadata` write sentences
    (`format_name`, `require_jpeg_for_copy`); FT-U4.
  - `src/source/mod.rs` — `NoImagesInFolder`, `NoImagesMatched` (glob empty match), `OnEmpty`, `resolve_all`,
    `folder_name`; FT-U1, FT-U2 and a Unix unreadable-folder unit test.
  - `src/wasm.rs` — the refusal sentence only (AC-14 wording).
  - `tests/help_has_no_internal_ids.rs` — new (FT-G1 to FT-G6).
  - `tests/messages_tell_the_truth.rs` — new (FT-M1 to FT-M11).
  - `tests/source.rs` — `empty_glob_match_is_not_found_error` renamed and now expects `NoImagesMatched` (deviation 1).
- **RED on `main`, then GREEN** (baseline log quoted; every row green on `7ccc055`):
  - FT-M1: `51 of 51 rows wrong`, e.g. `` `view sub` (sub): stderr; stderr: "error: input not found or unreadable: sub\n" ``
    (the `view` rows fail on the message, not on "not a terminal").
  - FT-M2: `left: "error: target #0 (recipe r.toml, out bo)\nerror: input not found or unreadable: sub\n"`.
  - FT-M3: `` `resize --max 8 --out-dir od sub/`: wrong stderr left: "error: input not found or unreadable: sub/\n" ``.
  - FT-M4: `left: "error: input not found or unreadable: txt/*\n"`.
  - FT-M5 missing-path rows: green on `main` (the old line is true there, as designed). FT-M5 Unix: red,
    `` `lint locked sub`: left: "error: input not found or unreadable: locked, sub\n" ``.
  - FT-M6: `` `resize --max 8 --out-dir od sub empty`: left: "error: input not found or unreadable: sub empty\n" ``.
  - FT-M7: `` `responsive --widths 8 --out-dir od img`: left: "error: could not read image input\n" ``.
  - FT-M8: `left: "error: -q/--quality cannot be combined with --target/--ssim/--max-size (they auto-tune quality)\n"`.
  - FT-M9: `left: "error: metadata lane does not support Gif yet\n"`.
  - FT-M10: `got "error: could not read font file \"a\"b.ttf\": No such file or directory (os error 2)\n"`.
  - FT-M11: `lint --help: Lint an image asset tree: report problems (each with a runnable fix) … (SPEC-050, DEC-050).`
  - FT-G1: `207 id lines over 44 pages` (22 command paths). FT-G2: green, `22 paths derived from the rendered text, 44 pages`.
  - FT-G3: `zsh: 136`, `fish: 92`, `powershell: 136`, `elvish: 136` id lines; bash 0 (green).
  - FT-G4: `literal scan: 44 files, 1191 literals, 1 with an id` → `src/wasm.rs:237`. FT-G5: green.
  - FT-G6: `demo/demo.js:434`, `demo/index.html:163` (3 demo files, 3 embedded files).
  - After: FT-G1 `22 command paths, 44 pages, 0 id lines`; FT-G4 `44 files, 1201 literals, 0 with an id`; FT-G6 0.
- **FT-D1** (docs grep, run on `7ccc055`): `grep -nE '(SPEC|DEC)-[0-9]' docs/cli-reference.md` → no output (exit 1);
  `grep -n 'SPEC-111\|keep ICC\|ICC + copyright' docs/cli-reference.md AGENTS.md docs/architecture.md` → no output
  (exit 1). Changed lines: `cli-reference.md:53` "| `3` | Input not found or unreadable, or a folder or glob with no
  images, or a folder where one image file is expected. |"; `:311` "bakes, the replay doesn't. This divergence is
  known and not yet fixed."; `api-contract.md:58` the same exit-3 text; `AGENTS.md:501-504` and
  `architecture.md:148-151` as the spec's Docs list. `cli-reference.md` restates neither the old `lint` sentence
  ("each with" grep: 0) nor the old `--strict` help; its `:432` paragraph already explains the same-`arch-os` rule
  truthfully, so it is unchanged.
- **Mutations (all 25, scratch worktree, own target dir; each applied alone, `git diff` non-empty, reverted to an
  empty diff; the restored tree re-ran green).** Where a test function stops at its first red row, the masked rows
  were driven by hand against the mutated binary ("driven").

  | # | Red (quoted) | Stayed green |
  |---|---|---|
  | 1 | FT-M1 24 rows (`resize` ×2, `convert`, `thumbnail`, `auto-orient`, `edit`, `watermark`, `optimize --format png` × 3 shapes): `"error: input not found or unreadable: sub\n"`; FT-M3 `"…: sub/\n"`; FT-M6 `"…: sub empty\n"` (same fan-out; not in the table) | other FT-M1 rows, FT-M2 |
  | 2 | FT-M1 9 rows (`meta strip\|clean\|set` × 3) | other 42 rows |
  | 3 | FT-M2 `"error: target #0 (recipe r.toml, out bo)\nerror: input not found or unreadable: sub\n"` | FT-M1 |
  | 4 | FT-M1 `apply` × 3 | others |
  | 5 | FT-M1 6 rows (`optimize --out-dir`, `web` × 3) | `optimize --format png` rows |
  | 6 | info: FT-M1 `info` × 3; view: FT-M1 `view` × 3 (two separate runs) | others |
  | 7 | FT-M1 `lint` × 3; FT-M6 `lint 'txt/*' a.png: … left: Some(3) right: Some(0)`, driven: `lint nope sub` → `…: nope, sub`, `lint sub nope` → `…: sub, nope`; FT-M5 Unix `lint locked sub` → `"…: locked, sub\n"` | FT-M4, FT-M5 `lint nope`, `lint locked` |
  | 8 | all 51 FT-M1 rows, FT-M2, FT-M3, FT-M6, FT-U1 `empty folder: Err(NotFound(…/empty))`, FT-U2 `[sub, empty]: Err(NotFound(…/sub))` | FT-M4, FT-M5 |
  | 9 | FT-M5 Unix `` `lint locked`: left: "error: no images in folder 'locked': …" ``; Unix unit test `unreadable folder: Err(NoImagesInFolder(…/locked))` | FT-M1, FT-M5 `resize locked` |
  | 10 | FT-M3 `"…no images in folder 'sub/': … glob such as 'sub//**/*'\n"`; FT-U1 `left: "…/empty/" right: "…/empty"` | FT-M1 |
  | 11 | FT-G1 `44 id lines over 44 pages`; FT-G3 zsh/fish/powershell/elvish 22 each | FT-G4, FT-G3 bash |
  | 12 | FT-G1 4 lines: `meta -h`, `meta --help`, `meta set -h`, `meta set --help`; FT-G3 4 shells, 3 each | root and other leaves |
  | 13 | FT-G1 1 line, `meta --help` only | FT-G3 (long about not in completions) |
  | 14 | FT-G1 `optimize --help: - high: … (DEC-023)`; FT-G3 zsh, fish (1 each) | FT-G4, FT-G3 powershell/elvish/bash |
  | 15 | FT-G4 `src/wasm.rs:237` | FT-G1, FT-G3, FT-W1 (`just wasm-test` under the mutation: 53 passed) |
  | 16 | FT-M11 `lint --help: … (each with a runnable fix), …` | FT-G1 |
  | 17 | FT-M8 row 1 `left: "…cannot be combined with --target/--ssim/--max-size (they auto-tune quality)\n"`; driven: `web`, `--target`, `--ssim`, `--max-size`, `convert --max-size` all print that old line (exit 2) | `convert -q 10` exit 0 (driven) |
  | 18 | FT-M10 `` got "error: could not read font file \"a\"b.ttf\": …" ``; FT-U3 `left: "\"a\"b\""`; driven: overlay and recipe rows unescaped | backslash row (driven, on a build proven mutated) |
  | 19 | FT-M10 `the backslashes changed: … "C:\\dir\\f.ttf"`; FT-U3 `left: "\"C:\\\\d\\\\f\""` | — |
  | 20 | FT-G2 `the walk and the rendered Commands: sections disagree left: {[]}` | — |
  | 21 | FT-G2 `the rendered pages list no nested command (e.g. \`meta set\`): {[]}` | — |
  | 22 | FT-G4 `4334 literals, 6 with an id`: `src/build/cache.rs:944`, `src/cli/build.rs:1511`, `src/image/avif.rs:698`, `:960`, `src/operation/mod.rs:2563`, `src/operation/registry.rs:657` (none edited); also FT-G5 `test module then code: want only "DEC-065", got ["SPEC-128", "DEC-065"]` | — |
  | 23 | FT-G5 `an id after // inside a string: want 1, got 0` | FT-G5 continuation row (see design call notes) |
  | 24 | FT-M9 `left: "error: Gif\n"`; FT-U4 `left: "Gif"` | FT-M7 |
  | 25 | FT-M7 `` `diff img a.png`: left: "error: could not read image input\n" ``; driven: `diff a.png img` same | `responsive`, `meta copy` rows (driven) |
- **Gates** (own `CARGO_TARGET_DIR`, exit codes read directly): `cargo fmt --check` 0; `cargo clippy --all-targets
  -- -D warnings` 0; `cargo clippy --all-targets --no-default-features -- -D warnings` 0; `just check` 0 (1260
  passed, 0 failed, 60 test binaries); `cargo build --no-default-features` 0; `cargo test --no-default-features
  --test help_has_no_internal_ids --test messages_tell_the_truth` 0 (6 + 12 passed); `just wasm-check` 0; `just
  wasm-test` 0 (53 passed, `transform_refuses_asset_bearing_recipe` ok). After the final rebase: the two new files
  and `tests/cli.rs` 0 (145 + 6 + 12 passed). Toolchain: cargo 1.99.1 (Homebrew), rustup 1.29.0.
- **Windows leg:** cannot run here; see the PR's `windows-latest` result (recorded in the report).
- **Deviations from spec:**
  1. `tests/source.rs` (not in the spec's file list): `empty_glob_match_is_not_found_error` asserted
     `SourceError::NotFound` for an empty glob, the exact behaviour design call 2 changes. Found by `just check`
     (the only failure). Renamed `empty_glob_match_is_no_images_matched_error`, asserts `NoImagesMatched`. The
     frame's compatibility sweep grepped message text, not the variant, so it missed this.
  2. FT-U5 dropped, as the spec allows: the only modules compiled on both targets outside PROJ-017's paths are
     `lib.rs` and `sink`, and a helper there is unused on native (a dead-code allow, not a clean ≤ 10-line move).
     AC-14 is carried by FT-W1 + FT-G4.
  3. Mutations 3, 4 and 5 restore the old code verbatim and also put `SourceError` back in that file's import (the
     build removed it as unused). Mutation 9 is the after-the-fact classifier: `Ok(empty)` recorded as `NotFound`,
     then the first cause re-asked with `Path::is_dir()`. Mutation 20 returns after visiting the root.
  4. FT-M6, FT-M7, FT-M8 and FT-M10 assert their rows in sequence, so a mutation shows the first red row; the
     masked rows were driven against the mutated binary (table above).
- **Follow-up work identified:**
  - (orchestrator) `docs/recipes.md:148` "each with a runnable fix" → "with a runnable fix where one exists", and
    the CHANGELOG `Fixed` text (both PROJ-017-owned; the text is in this spec's Frame §5 and the PR body).
  - **Out-of-spec finding:** `just wasm-check` warns `constant MANIFEST_DROPPED_WARNING is never used`
    (`src/metadata/mod.rs:154`, a `pub(crate)` constant only the native CLI reads). Present on `main` before this
    change; released 0.8.2 builds the same way (a warning, not a behaviour bug). Open PR #240 adds clippy on the
    wasm32 library build; if it uses `-D warnings` it will trip on this. No CHANGELOG or DEC line needed; worth a
    `#[cfg(not(target_arch = "wasm32"))]` on the constant in whichever PR lands #240.

### Fix round (verify punch list at `5996c6b`)

- **R1:** the `-q` message now names `crustyimg convert --format <FORMAT> -q N` (the spec's AC-7 wording was wrong:
  `convert -q 10` alone exits 2, "required arguments were not provided: --format"). FT-M8 pins the new sentence and
  now runs the printed command with `<FORMAT>`→`jpeg`, `N`→`10`: exit 0, non-empty output.
- **R2:** the glob suggestion goes through `lint::shell_arg` (new `pub(crate) quote_for_host_shell` wrapper; the
  function itself stays private). `it's` now prints `'it'\''s/**/*'`; ordinary names stay `'sub/**/*'`. New unix-only
  FT-M3 row runs the printed glob in `sh -c`. Red with the old quoting: ``sh: -c: line 0: unexpected EOF while
  looking for matching `''``.
- **Optional:** FT-M rows for `optimize --out-dir` and `web` with `nope` + a real image (exit 3, nothing written).
  The `OnEmpty::Skip` mutant goes red: `optimize --out-dir od nope a.png: want exit 3, got Some(0)`.
- Code commit `e0e1e3a`.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Little. The design was exact enough to write the tests before reading most callers. One expectation was wrong:
   mutation 23 lists FT-G5's continuation row as red, but that fixture holds no `//`, so a lexer that treats `//`
   in a string as a comment cannot change it; only row 1 goes red. Mutation 22 also turns FT-G5's test-module row
   red, which the table did not list (a stronger result, not a weaker one).

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `tests/source.rs` should have been in the file list: it pins the empty-glob variant that design call 2
   changes. A compatibility sweep for a changed error needs to grep the variant name as well as its message text.

3. **If you did this task again, what would you do differently?**
   — Write table tests that collect every row's failure before asserting (as FT-M1 does) for FT-M6/7/8/10 too, so
   each mutation's full red set shows up in one run instead of being driven by hand.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Run every command a message tells the user to run, at design, before the sentence goes into the spec. The
   `-q` message named `crustyimg convert -q N`; AC-7 carried it, the build copied it into code and into FT-M8's six
   rows, and those rows compared the message with a literal copy of itself, so CI was green on a command that exits 2
   without `--format`. Verify found it. SPEC-149 taught the same lesson one spec earlier and it sat in the build
   prompt, which is too late: the sentence is written at design, so the run belongs there. Second thing: a mutation
   the table did not list (`optimize --out-dir` `Fail` to `Skip`) turned nothing red until the fix round added
   mixed-argument rows (`nope` plus a real image). When a verb has a fan-out mode, the table needs a row for the
   mixed case, not only the all-bad case.

2. **Does any template, constraint, or decision need updating?**
   — No decision. DEC-003's amendment was made at design and held. Template side, two small things: (a) the
   design template's "message that names a command" note (from SPEC-148 and SPEC-149) should say the command is
   run at design and the output pasted, and a test for such a message takes the command out of the message
   and runs it, as the fixed FT-M8 does; (b) a compatibility sweep for a changed error greps the variant name
   as well as the message text (the build's list missed `tests/source.rs`, which pinned `NotFound` for an empty
   glob). Verify found what CI green could not for the second spec in a row. A draft PR cannot merge until
   "ready for review" is clicked; that is process, not a finding about the code.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. Verify's optional findings are filed as one entry on STAGE-063: ids still in two user docs
   (`docs/cli-reference.md:305`, `README.md:125`), the hidden `--keep-gps` in every completion script, the wasm32
   `MANIFEST_DROPPED_WARNING` warning (PR #240 may trip on it), a `[` in a path read as a glob, no `\` or `C:\`
   test through the folder message and a single-quoted hint that does not paste into `cmd.exe`, an ambiguous
   `quoted()` before `"`, and `SourceError`/`CliError` not being `#[non_exhaustive]` (a maintainer call before 1.0).
   One more design fact: mutation 23's second row cannot go red because that fixture holds no `//`.

## Ship

Shipped 2026-11-07. Code is on `main` as `b8b2a04` (PR #257). Verify punch list at `5996c6b` (R1: the `-q`
message named a command that exits 2; R2: the subfolder glob hint could not be pasted for a folder named `it's`);
fix round `e0e1e3a`, head `0deb220`, CI green; the orchestrator read the diff, no re-verify. Cost: frame $4.42,
design $8.57, build $14.54, verify $6.79, fix $0.96, ship $0.84; total $36.12, 49,822,079 tokens.

### Behaviour changes in 0.8.3 (user-visible)

All of these change what a user sees; none changes an exit code except where noted. A folder with no top-level image
still exits 3, a folder given where one file is expected (`responsive`, `diff`, `meta copy`) exits 3, and the `-q`
refusal still exits 2. New text on stderr; stdout and output files are unchanged (verify: 22 previously passing
invocations, 0 differences against `main`'s binary). `--help` and the shell completions lose their internal ids.
The two public error enums gain variants (`SourceError::NoImagesInFolder`, `NoImagesMatched`;
`CliError::InputIsAFolder`) and neither is `#[non_exhaustive]`, so a downstream exhaustive `match` stops compiling.

### Proposed CHANGELOG entry for 0.8.3 (SPEC-148 + SPEC-149 + SPEC-150; not written to `CHANGELOG.md`)

The release prompt can paste this as the 0.8.3 block. SPEC-148 and SPEC-149 text is copied from their archived
specs; the SPEC-150 lines are new.

```
### Breaking

- `--keep-gps` is refused (exit 2). It never kept GPS: pixel-lane encodes drop all metadata. To keep
  it, copy it back after the encode with `crustyimg meta copy --from in.jpg --to out.jpg -y` (JPEG
  only). The flag no longer shows in `--help`.
- `responsive` refuses `--name-template` and `--format` (exit 2) instead of ignoring them. Its file
  names are fixed (`{stem}-{w}w.{ext}`); choose formats with `--formats`.
- `--check` (`--frozen`, `--locked`), `--strict` and `--no-cache` are refused (exit 2) on every command
  but `build`, as `--watch` already was. `build --strict` without `--check` is refused too.
- The public error enums `SourceError` and `CliError` gain variants (`NoImagesInFolder`,
  `NoImagesMatched`, `InputIsAFolder`). Neither is `#[non_exhaustive]`, so a library user with an
  exhaustive `match` on either must add arms.

### Changed

- `lint`'s printed `fix:` line (and the JSON `fix` field) is now a command that rewrites the file in
  place: it ends in `-o <file> -y`, and paths with spaces, quotes or a leading dash are quoted for your
  shell (PowerShell on Windows). Before, running one wrote the image to your terminal and left the file
  unchanged.
- Some findings now print no command, and the JSON `fix` is `null`: input from standard input, 16-bit
  PNG, and WebP, TIFF and AVIF files where the fix would be a metadata edit crustyimg cannot write yet
  (the message says so). This is a visible change if you read the `fix` field.
- `privacy/gps-metadata-leak` now fires on a coordinate (latitude, longitude, a destination coordinate)
  or a place name, not on any GPS block. A block holding only a version number, which some cameras
  write into every JPEG, no longer flags, so a CI job that failed on those files now passes.
- A folder with no image at its top level now says so and how to include subfolders (a glob such as
  `'dir/**/*'`), instead of "input not found or unreadable"; the exit code stays 3. This covers every
  command that takes a folder, and `build` targets. An empty glob match says `no images matched '<glob>'`.
  `responsive`, `diff` and `meta copy` given a folder say it is a folder, not an image file.
- When several inputs are given and none yields an image, `lint` and the other commands name the first
  input's cause, instead of one blanket message.
- `-q` with `optimize`, `web` and the other commands that choose the quality themselves now names the
  real conflict and a command you can run: `crustyimg convert --format <FORMAT> -q N`. `meta` on a file
  the metadata lane cannot handle, and `meta copy` on a non-JPEG, now say plainly which formats they
  support.
- `--help`, the shell completions and the browser build no longer show internal spec or decision ids,
  and `lint --help` no longer promises a fix for every finding.

### Added

- `-v` prints warnings from the SVG renderer and codecs on stderr, such as
  `warning: [usvg::parser::image] Image lacks the 'xlink:href' attribute. Skipped.`; `-vv` adds debug
  detail. Without `-v` nothing changes, and `-Q` wins.

### Fixed

- The `size/oversized-bytes` fix passes your budget (`optimize --max-size <bytes>`), so running it
  clears the finding; plain `optimize` often did not.
- The CMYK fix is `convert --format jpeg`; it used to print `convert --format <file>`, which failed with
  exit 3.
- A missing or unreadable `watermark --font` or `--image` names the file (exit 3) instead of saying
  "could not read image input"; a `"` in that path is quoted correctly.
```

### Proposed text for the other PROJ-017-adjacent files (not written)

- `docs/recipes.md:148`: replace "each with a runnable fix" with "with a runnable fix where one exists":
  `` `crustyimg lint assets/ --select privacy` → `privacy/gps-metadata-leak` and
  `privacy/camera-metadata`, with a runnable fix where one exists; exit `7` on an error-severity finding. ``
  (The `docs/recipes.md:145` GPS note and the `skills/crustyimg/SKILL.md:67-68` and `:145` lines proposed by
  SPEC-149's ship still stand and are separate from this.)
- `skills/crustyimg/SKILL.md:56-59` says "(a folder of only subfolders exits 3)". Still true, but incomplete now
  that the error names the cause. Proposed: `A directory works as an input too, but reads only the images
  directly inside it, not its subfolders (a folder with no image at its top level exits 3 and says so); use
  'assets/**/*' for a whole tree.` `skills/crustyimg/SKILL.md:332` (exit 3 "Input not found, or the glob matched
  nothing") needs "or a folder with no images" added. Neither restates the old `-q` text (grep: 0 hits for
  "cannot be combined" in `skills/`, `docs/`, `README.md` outside `cli-reference.md`'s `--watch` line).
