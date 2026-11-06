---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-149
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
  implementer: null     # usually same Claude, different session
  created_at: 2026-11-06

references:
  decisions:
    - DEC-003   # dual-lane metadata; promises --keep-gps and a selective preserve the pixel lane does not do
    - DEC-024   # "--keep-gps remains a no-op until the container lane exists"
    - DEC-105   # an output name's extension pins the format (bears on a responsive template)
    - DEC-059   # --check / --strict semantics
    - DEC-007   # exit codes
  constraints:
    - metadata-not-via-pixel-encode
    - ergonomic-defaults
    - test-before-implementation
  related_specs:
    - SPEC-071   # refused --watch on non-build verbs (exit 2): the precedent
    - SPEC-134   # --name-template pins the format on every encoding verb but responsive
    - SPEC-128   # recipe asset resolution names an unreadable font
    - SPEC-150   # wasm asset-refusal wording; the -q message finding may fold there
    - SPEC-151   # PROJ-017, may add a verb to src/cli/mod.rs

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: >
  Delivers STAGE-063's "no global flag is accepted and then ignored": each of --keep-gps, -v,
  responsive --name-template/--format and build-only flags is honoured or refused with exit 2.

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
      tokens_total: 6086051
      tokens_breakdown:
        input: 102
        output: 45300
        cache_creation: 132597
        cache_read: 5908052
      estimated_usd: 4.92
      duration_minutes: null
      recorded_at: 2026-11-06
      note: >
        Interactive frame session, worktree docs/spec-149-frame. Measured from transcript
        e8c72342-f9e0-44fb-8b57-266ca2ce5595.jsonl, found by the prompt's title, deduped by
        `.message.id` (51 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Opus $5 in / $25 out per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Taken before the commit/push exchanges, so it
        slightly under-counts the session.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-149: No flag is accepted and ignored

## Context

STAGE-063 is PROJ-011's last stage: user-facing output that is wrong or misleading, then 0.8.3.
Its success criterion: `--keep-gps`, `-v`/`--verbose` and `responsive --name-template` are each
implemented, refused with exit 2, or removed with their docs, never accepted and ignored. This spec
takes those three, the sibling `build --strict` without `--check`, and the missing-`--font`
message (STAGE-063 backlog, items carried from STAGE-054). SPEC-148 shipped the `lint` items;
SPEC-150 takes the two misleading messages.

Precedent already in the code: `--watch` on any verb but `build` exits 2 (SPEC-071,
`src/cli/mod.rs` `dispatch`), and `build` refuses `-o`, `--out-dir`, `--name-template` and
`--format` (`refused_build_flags`). This spec extends that pattern.

## Goal

Today a user can pass `--keep-gps`, `-v`/`-vv`, `responsive --name-template`/`--format`,
`build --strict` alone, or `--check`/`--strict`/`--no-cache` to any verb but `build`, and get exit 0
with output identical to not passing it; a missing `--font` says "could not read image input".
After this spec, each such flag either changes the output as documented or exits 2 with a message
saying why, and a missing font or overlay names its file.

## Frame

**Verdict: GO.** Size **M**. Framed 2026-11-06 on `main` at `00e6961` (0.8.2 debug build, own
`CARGO_TARGET_DIR`). **Rulings pending** from the maintainer (one per flag, listed at the end of this
section); the recommendations below are what design should assume if the answers match.

### Premise check (driven, not read)

GPS fixtures: `tests/fixtures/ddphotos/photo_rot.jpg` stripped with `exiftool -all=`, a PNG made from
it with `crustyimg convert`, then **exiftool 13.55** wrote `GPSLatitude=50.4881 N`,
`GPSLongitude=30.5 E`, `Make=TestCam` (and `Artist`/`Copyright` for the preserve check) into both.
The repo's own writer was not used, so the fixtures do not come from the code under test.

| # | Command | Exit | What the user sees |
|---|---|---|---|
| 1 | `convert gps.jpg --format jpeg --keep-gps -o out.jpg` | 0 | silent; `exiftool -EXIF:all out.jpg` → **0 keys** |
| 1 | `convert gps.png --format png --keep-gps -o out.png` | 0 | silent; **0 EXIF keys** |
| 1 | `optimize gps.jpg --keep-gps -o o.jpg` | 0 | **0 EXIF keys** |
| 2 | every verb with `-v` and with `-vv` | same as without | byte-identical stdout, stderr and files (matrix below) |
| 3 | `responsive gps.jpg --widths 100 --out-dir rp --name-template '{stem}.png'` | 0 | writes `rp/gps-100w.jpg` (matrix: identical to no flag) |
| 3+ | `responsive gps.jpg --widths 100 --out-dir rp --format webp` | 0 | writes `rp/gps-100w.jpg`: **`--format` is ignored too** (new) |
| 4 | `watermark gps.jpg --text hi --font /no/font.ttf -o wm.jpg` | 3 | `error: could not read image input` |
| 4+ | `watermark gps.jpg --text hi --font .` (a directory) | 3 | same message |
| 4+ | `watermark gps.jpg --image /no/logo.png -o wi.jpg` | 3 | **same message, no path** (new, same function) |
| 4 ctl | `watermark /no/in.jpg --text hi` | 3 | `error: input not found or unreadable: /no/in.jpg` (primary input is named) |
| 4 ctl | `apply --recipe wr.toml gps.jpg` with `font = "/no/font.ttf"` | 1 | `error: recipe step 0 ('watermark'): could not read 'font' file "/no/font.ttf": No such file or directory (os error 2)` |
| 5 | `build --strict` (manifest present, no `--check`) | 0 | identical to `build` |

**The global-flag sweep.** `GlobalArgs` has 14 fields. Reads outside the struct and test fixtures
(`grep -rnE '(global|g|args)\.<field>\b' src`): output 16, out_dir 14, name_template 6, jobs 2,
format 11, quality 13, quiet 18, yes 6, no_cache 1, check 2, strict 1, watch 4, **verbose 0,
keep_gps 0**. So every field except two is read somewhere, but a field read by one verb is still
accepted and ignored by the others. A driven matrix (`matrix.sh` in the frame session's scratch dir:
each verb's baseline vs the baseline plus one flag; exit, stdout hash, stderr hash, hash of every file
written) on 20 verb forms × 13 flags:

| verb (base exit) | -v | -vv | --keep-gps | --no-cache | --check | --strict | -j 2 | -q 10 | --format webp | --name-template | --out-dir | -Q | -y |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| resize, thumbnail, auto-orient, watermark, edit, apply (0) | same | same | same | same | same | same | same | CHANGED | CHANGED | same* | same* | same | same |
| convert (0) | same | same | same | same | same | same | same | same (PNG) | exit2 | same* | same* | same | same |
| web, optimize (0) | same | same | same | same | same | same | same | exit2 | CHANGED/same | same* | same* | same | same |
| responsive (0) | same | same | same | same | same | same | same | CHANGED | **same** | **same** | exit2 | same | same |
| meta strip / clean / set (0) | same | same | same | same | same | same | same | **same** | **same** | same* | same* | same | same |
| info, diff (0); lint (7); view (5, no TTY); meta copy | same | same | same | same | same | same | same | same | same/exit2 (lint) | same | same | same/CHANGED (lint) | same |
| build (0) | same | same | same | same | exit7 | **same** | same | CHANGED | exit2 | exit2 | exit2 | CHANGED | same |
| completions (0) | all same |

`same*`: `-o` was also given, and `-o` wins silently (see out-of-spec finding A). `-j`, `-y`, `-Q`
being "same" on a single input, a fresh output path or a silent verb is expected, not a finding.

**Accepted and ignored, in this spec:** `--keep-gps` (every verb), `-v`/`-vv` (every verb),
`responsive --name-template` and `responsive --format`, `build --strict` without `--check`,
`--check`/`--strict`/`--no-cache` on the 18 non-`build` verb forms (exit 0, same output; the
`--watch` guard covers only `--watch`).
**Accepted and ignored, out of spec** (filed, finding A): `meta strip|clean|set --format webp -o x.jpg`
and `-q` (exit 0, JPEG written); `-o` together with `--out-dir`; read-only verbs taking `-o`/`--out-dir`
(`info gps.jpg -o x.txt` exits 0 and writes nothing); unknown template tokens written literally.

### Answers

**1. Reproduction:** above. Nothing has moved since the backlog items were filed, except that the
line numbers did (`--keep-gps` docs are now `docs/cli-reference.md:21,42`, `README.md:199,293`,
`CHANGELOG.md:777,794`).

**2. Per-flag rulings.**

*`--keep-gps`* — the pixel lane drops **all** metadata: driven, `resize` of a JPEG carrying
`Artist`, `Copyright` and GPS keeps 0 of them. So DEC-003's "keep orientation + ICC + copyright/artist;
drop GPS unless `--keep-gps`" is not what ships, and DEC-024 already called the flag "a no-op until the
container lane exists". The container lane now exists, and `meta copy --from gps.jpg --to r.jpg -y`
restores `Artist` and `GPSLatitude` onto a resized JPEG (driven, exit 0). The options:
- **Implement (copy-then-clean):** after each pixel encode, graft the source's metadata with the
  container lane, reset `Orientation` (already baked into pixels), keep GPS. That is a new feature: it
  touches the write path of 11 verbs (resize, thumbnail, convert, web, optimize, responsive,
  auto-orient, watermark, edit, apply, build), works only where source and output are both JPEG or PNG
  (`meta` refuses other pairs with exit 4), and makes `--keep-gps` keep *more* than the default
  (everything, not only GPS) unless DEC-003's selective preserve is built first. M to L alone, needs
  a DEC. Not this spec.
- **Refuse (exit 2) on every verb, hidden from `--help`:** one guard beside the `--watch` guard,
  message saying pixel-lane encodes drop all metadata and naming `meta copy --from <in> --to <out> -y`
  (JPEG/PNG). Breaks any script that passes it (exit 0 → 2); such a script never got GPS anyway.
  `grep` of `jysf/setup-crustyimg`, `jysf/crustyimg-action` and `jysf/crustyimg-example` (shallow
  clones): **no caller**. The skill says "pass `--keep-gps` only if the user asks"
  (`skills/crustyimg/SKILL.md:67`).
- **Remove:** clap's generic `unexpected argument '--keep-gps'` (exit 2). Same break, no pointer to
  the workaround.
- **Recommend: refuse, hidden**, and file implementation as a feature under the DEC-003 drift item
  (finding C).

*`-v`/`--verbose`* — crustyimg has 0 `log::` calls and 53 `eprintln!`s; no logger is installed, so
the `log` facade discards everything. `log 0.4.32` is already in the tree (via usvg, resvg, fontdb,
rav1e, notify, avif-parse), so a direct dependency adds no crate. Warn sites in the registry sources:
usvg 0.49.0 **56** (e.g. `"Image has an invalid size. Skipped."`), resvg 9, fontdb 4, rav1e 5 warn /
52 info / 12 debug, notify 1.
- **Implement (minimal):** a ~30-line stderr `log::Log` (no `env_logger`): `-v` shows `warn`, `-vv`
  adds `info`/`debug`; `-Q` wins. Default (no `-v`) stays silent, so no output changes for anyone not
  passing it. Value: SVG inputs that usvg silently simplifies become explainable. S.
- **Refuse:** contrary to every CLI convention; not considered.
- **Remove:** breaks habitual `-v` (clap exit 2) for a flag `docs/api-contract.md:36` lists.
- **Recommend: implement minimal.** Undriven: no warn line can be shown today (no logger); design
  must drive an SVG that triggers a known usvg `warn!` and quote the line.

*`responsive --name-template` (and `--format`)* — the template code does not validate tokens: on
`resize`, `--name-template '{stem}-{w}w.{ext}'` writes a file literally named `gps-{w}w.jpg`
(exit 0). Honouring needs a token set (`{stem}`, `{w}`, `{ext}`, maybe `{h}`), a rule that `{w}` is
required (else every width collides), a ruling on a literal extension (DEC-105 pins one format, but
`responsive` writes a list), snippet `srcset` names from the template, and the token validation
the other verbs lack. M alone. Refusing is a few lines in `run_responsive`
(`src/cli/optimize.rs:1904`) or the dispatch guard: say the names are fixed `{stem}-{w}w.{ext}` and
point `--format` users at `--formats`. `docs/api-contract.md:591-593` already documents "accepts
`--name-template` and ignores it". **Recommend: refuse both (exit 2)**; a template for `responsive` goes
to `guidance/ideas.yaml` beside the `{hash}` token idea (line 108).

*`build --strict` without `--check`* — help already says "With `--check`". Refusing is one line beside
the `--watch` guard (`--strict` with `--watch` is covered, since `--watch` refuses `--check`).
Documenting leaves a flag accepted and ignored. **Recommend: refuse (exit 2)**, and in the same guard
refuse `--check`/`--strict`/`--no-cache` on every verb but `build`, as SPEC-071 did for `--watch`.

**3. The `--font` message.** `src/cli/ops.rs:1446`:
`Some(path) => Some(std::fs::read(path).map_err(ImageError::Io)?)`, which renders as
`CliError::Image(ImageError::Io)` → "could not read image input", exit 3. The `--image` overlay in the
same function (`watermark_overlay`, `src/cli/ops.rs:1433`, `Image::load(image)?`) has the same
message. Proposed: `error: could not read font file "/no/font.ttf": No such file or directory (os error 2)`
and `error: could not read watermark image "/no/logo.png": …`, matching the recipe path's wording; keep
exit 3 unless the maintainer prefers the recipe path's 1 (design decides, one line). Readers of a font:
- `watermark --text … --font PATH` (CLI): the only broken one.
- recipe `watermark` step `font = "…"` via `resolve_recipe_assets` (`src/cli/common.rs:237-246`),
  used by `apply` (`src/cli/optimize.rs:65`) and `build` (`src/cli/build.rs:324`): already names the
  file (exit 1). Separate code; not shared with the CLI path.
- wasm: refuses every asset-bearing step (`src/wasm.rs:235-240`); its wording is SPEC-150's.
- `src/image/svg.rs` loads only the bundled Go font; `edit` has no `--font`. No other reader.

**4. Compatibility.** Documented contracts: `docs/api-contract.md:36` (`-v`), `:39` (`--keep-gps`),
`:591-593` (responsive ignores the template, already admitted); `skills/crustyimg/SKILL.md:67`
(`--keep-gps`). Neither Action has an input for any of these flags, and no Action or demo-site file
passes them. Refusals are user-visible; proposed CHANGELOG text (do not edit until build):

> ### Changed
> - **`--keep-gps` is now refused (exit 2).** It never kept GPS: pixel-lane encodes drop all
>   metadata. To keep location data, copy it back after the encode:
>   `crustyimg meta copy --from in.jpg --to out.jpg -y` (JPEG and PNG).
> - **`responsive` now refuses `--name-template` and `--format` (exit 2)** instead of ignoring them.
>   Its names are fixed (`{stem}-{w}w.{ext}`); choose formats with `--formats`.
> - **`--check`, `--strict` and `--no-cache` are refused (exit 2) on every command but `build`,**
>   and `build --strict` without `--check` is refused, as `--watch` already was.
>
> ### Added
> - **`-v` shows warnings from the SVG renderer and codecs on stderr; `-vv` shows more.** `-Q` wins.
>
> ### Fixed
> - **A missing or unreadable `watermark --font` or `--image` names the file** instead of saying
>   "could not read image input".

**5. Docs that change** (⚠ = a file PROJ-017 also edits or plans to; coordinate before build):
- `--keep-gps`: `src/cli/mod.rs:107-110` (help, hide) ⚠; `docs/cli-reference.md:21,42` ⚠;
  `docs/api-contract.md:39` ⚠; `README.md:199,293` ⚠; `docs/recipes.md:123` ⚠ (PROJ-017-owned);
  `skills/crustyimg/SKILL.md:67` ⚠ (PROJ-017-owned); `docs/USAGE.md:164`; `docs/data-model.md:122` ⚠;
  `docs/moat.md:96`; `SECURITY.md:50,88`; `AGENTS.md:503` (glossary) ⚠. DEC-003 / DEC-024 need an
  amendment line at design (no DEC written at frame). Released CHANGELOG entries (`:777`, `:794`) stay.
- `-v`: `src/cli/mod.rs:95-98` ⚠ (help becomes true); `docs/cli-reference.md:41` ⚠;
  `docs/api-contract.md:36` ⚠; `README.md:293` ⚠.
- `responsive`: `docs/api-contract.md:591-593` ⚠; `docs/cli-reference.md:199-206` ⚠; the verb's
  `about` in `src/cli/mod.rs` ⚠.
- `--check`/`--strict`/`--no-cache`: `src/cli/mod.rs:112-135` help strings ⚠ ("`build` only");
  `docs/cli-reference.md:422-429` ⚠.
- font: none (the message is the fix).
- `CHANGELOG.md` Unreleased ⚠ (PROJ-017-owned section).

**6. Size and scope.** **M**: five small code sites (one dispatch guard extended into a per-verb
table, a logger, two `watermark_overlay` messages, a `hide`), about 14 doc files, and a per-verb test
matrix. Not a split while the recommendations hold. **If the maintainer rules "implement" for
`--keep-gps` or "honour" for `responsive --name-template`, SPLIT that flag into its own spec** (each
is M with a DEC); this spec keeps the rest.

Verification cost (traps 11-17 apply), about $5-8 for verify:
- The refusal test iterates the verbs from `Cli::command().get_subcommands()` (and `meta`'s
  subcommands), never a hand list, so a verb PROJ-017 adds (SPEC-151 may add `pad`) is covered; it
  asserts exit 2 **and** the message for each flag on each verb, and that nothing was written.
- Mutation rows name their location (trap 12): delete the guard → every refused-flag row red with an
  exit-0 message; restore `std::fs::read(path).map_err(ImageError::Io)?` verbatim → the font row red
  (trap 14); drop `-Q` precedence → the quiet row red.
- `-v` is proven to change output by an SVG fixture that triggers a named usvg `warn!`: stderr has
  the line with `-v`, lacks it without, lacks it with `-v -Q`. A row that cannot show the line is a
  failure, not a skip (trap 11).
- A positive control per honoured flag: `build --check --strict` still fails on variance;
  `build --no-cache` still rebuilds.

### Scope

**In:** refuse `--keep-gps` (hidden); minimal `-v`/`-vv` logger; refuse `responsive --name-template`
and `--format`; refuse `build --strict` without `--check`; refuse `--check`/`--strict`/`--no-cache`
off `build`; name the file for a missing `watermark --font` and `--image`; the docs above.

**Out:** see `### Out of scope` below.

**Rough build file list** (⚠ = overlaps PROJ-017):
- `src/cli/mod.rs` ⚠ (GlobalArgs help/hide; dispatch guard; SPEC-151 may add a verb arm here)
- `src/cli/ops.rs` ⚠ (font and overlay messages; PROJ-017 references this file)
- `src/cli/optimize.rs` ⚠ (`run_responsive` refusal, if not in the dispatch guard)
- `src/main.rs` or `src/cli/` (logger install), `Cargo.toml` (direct `log = "0.4"`)
- `tests/cli_flags_honest.rs` (new; no overlap; do not add helpers to `tests/common/mod.rs` ⚠)
- one SVG fixture under `tests/fixtures/svg/` for the `-v` row
- docs listed in answer 5 (⚠ as marked), `guidance/ideas.yaml`

### Questions only the maintainer can answer

1. **`--keep-gps`:** refuse (exit 2, hidden, message names `meta copy`) **(recommended)**, remove, or
   implement? Refuse: scripts passing it fail loudly, with the workaround in the message. Remove: same
   break, clap's generic message. Implement: a new M-L feature with a DEC; split out of this spec.
2. **`-v`/`--verbose`:** implement a minimal stderr logger **(recommended)**, or remove? Implement:
   `-v` starts showing usvg/codec warnings, nothing changes without it. Remove: every script passing
   `-v` exits 2.
3. **`responsive --name-template` and `--format`:** refuse both (exit 2) **(recommended)**, or honour
   with a `{w}` token? Refuse: fixed names, `--formats` named in the message. Honour: a token set and
   validation design; split into its own M spec.
4. **`build --strict` alone (and `--check`/`--strict`/`--no-cache` on other verbs):** refuse
   (exit 2) **(recommended)**, or document? Refuse: matches the `--watch` precedent. Document: the
   flags stay accepted and ignored, failing the stage criterion.
5. **Missing `--font`/`--image` exit code:** keep 3 with the file named **(recommended)**, or 1 like
   the recipe path? Keep 3: only the message changes. 1: consistent with `apply`, but an exit-code change
   to note.

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

- Implementing `--keep-gps` (copy-then-clean through the container lane) or DEC-003's selective
  preserve: a feature with its own DEC (STAGE-063 finding C).
- Honouring a `responsive` name template and validating unknown template tokens on every verb
  (`guidance/ideas.yaml`; finding A).
- The other accepted-and-ignored combinations: `meta strip|clean|set` with `--format`/`-q`, `-o`
  together with `--out-dir`, output flags on read-only verbs (STAGE-063 finding A).
- The `optimize`/`web` `-q` refusal naming flags the user did not pass (STAGE-063 finding B; may fold
  into SPEC-150).
- The wasm asset refusal's wording and spec ids in `--help` (SPEC-150).
- Anything under `src/operation/`, `src/recipe/`, or `projects/PROJ-017-geometry-as-recipe-steps/`.

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
