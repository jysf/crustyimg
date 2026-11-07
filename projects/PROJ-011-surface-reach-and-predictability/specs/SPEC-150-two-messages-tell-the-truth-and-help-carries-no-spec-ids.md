---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-150
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
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
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

See `## Frame` → "Out of scope (for this spec specifically)"; design copies it here.

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
