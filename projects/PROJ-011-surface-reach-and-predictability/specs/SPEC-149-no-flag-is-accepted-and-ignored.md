---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-149
  type: bug                        # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 8833593
      tokens_breakdown:
        input: 112
        output: 65420
        cache_creation: 186456
        cache_read: 8581605
      estimated_usd: 7.09
      duration_minutes: null
      recorded_at: 2026-11-06
      note: >
        Interactive design session, worktree docs/spec-149-design. Measured from transcript
        351dcaf8-e80b-466c-9734-be730a982243.jsonl, found by the prompt's title, deduped by
        `.message.id` (56 unique ids, all claude-opus-5-5): input/cache_creation/cache_read once
        per id, MAX output_tokens per id. Rates: Opus $5 in / $25 out per MTok, cache_creation
        x1.25 input, cache_read x0.10 input. Includes driving a scratch sketch of the guard table,
        logger and messages (two debug builds, a wasm-check, cargo deny). Taken before the
        commit/push exchanges, so it slightly under-counts the session.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 5878066
      tokens_breakdown:
        input: 90
        output: 32051
        cache_creation: 127235
        cache_read: 5718690
      estimated_usd: 2.67
      duration_minutes: null
      recorded_at: 2026-11-06
      note: >
        Measured from transcript 98b0a82e-e783-40ac-ac63-7db061c11337.jsonl, found by the
        prompt title, deduped by `.message.id` (45 unique ids, all Sonnet): input/cache_creation/
        cache_read once per id, MAX output_tokens. Rates: Sonnet $3 in / $15 out per MTok,
        cache_creation x1.25, cache_read x0.10. Taken before the final commit/push/PR
        exchanges, so it slightly under-counts the session.
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
`CARGO_TARGET_DIR`). **Rulings settled 2026-11-06 (see `## Design`)**; at frame they were pending from the maintainer (one per flag, listed at the end of this
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

## Design

Designed 2026-11-06 on `main` at `d62314e`, Opus 5.5, own `CARGO_TARGET_DIR`. Every number below
comes from a **sketch** of this design (guard table, logger, the two messages) built in a scratch
worktree and driven with the debug binary; the sketch is not committed. Rulings 1-6 of the design
prompt are binding and are not re-opened here.

### Call 1: one guard table, in its own file, replacing the `--watch` `if`

A new module **`src/cli/global_flags.rs`** holds a `const GLOBAL_FLAG_RULES: &[GlobalFlagRule]` and
`pub(super) fn refused_global_flags(cli: &Cli) -> Vec<&'static str>`. A row is
`{ given: fn(&GlobalArgs) -> bool, on: RefusedOn, message: &'static str }`, and
`RefusedOn` is `AllButBuild | Build | Responsive | All`. `dispatch` calls it **first**, in place of
today's `--watch` block (`src/cli/mod.rs`, the `if cli.global.watch && !matches!(…Build…)` block),
and returns `CliError::Usage(lines.join("\nerror: "))` when non-empty, exactly as
`refused_build_flags` does. Every matching row is reported, in table order; for `build`, the table
runs before `refused_build_flags`.

- **Why a scope, not a verb list.** `AllButBuild` and `All` are defined by exclusion, so a verb
  added later (SPEC-151's `pad`) is refused with no edit here. Only `build` and `responsive` are
  named, through `matches!(cli.command, Commands::Build { .. })` / `Commands::Responsive { .. }`.
- **Why its own file.** `src/cli/mod.rs` is shared with SPEC-151. Its hunks shrink to: two `mod`
  lines, the dispatch block swap, one `logger::init` line, help strings, one `CliError` variant and
  its `code()` arm. No hunk touches the `match &cli.command` arms SPEC-151 adds to.
- **The `--watch` message is moved byte-for-byte** into the table's first row, so
  `tests/cli.rs` (`contains("--watch is only valid with `build`")`) and `build_watch.rs` stay green.
- **Why not clap.** `requires = "check"` on `--strict` covers only `build --strict` alone, prints
  clap's "required arguments were not provided", and says nothing off `build`. Moving
  `--check`/`--strict`/`--no-cache` into `Commands::Build` breaks the documented "before or after
  the subcommand" form (`crustyimg --check build`) and gives clap's bare `unexpected argument` with
  no reason; it cannot express `--keep-gps`'s pointer at all. One mechanism, the table, covers all
  eight rows with a reason each. clap still owns parse errors.
- **`--help`.** `--keep-gps` gets `hide = true` (ruling 1) but stays in `GlobalArgs`, so
  `resize --keep-gps` reaches the guard and prints the table's message, not clap's. `--check`,
  `--strict`, `--no-cache` stay listed (they are real on `build`, and a global arg cannot be hidden
  per subcommand) with the help text prefixed "`build` only:" like `--watch`. `responsive`'s
  `about` gains one sentence naming its refusals.

No other verb defines its own `--check`, `--strict` or `--no-cache` (`grep -n 'long' src/cli/mod.rs`
inside `Commands`/`MetaCommand`: none; `lint` has `--no-config`, a different flag), so there is no
local `--check` to protect.

### The table: verb × flag × message (driven on the sketch)

Each line prints as `error: <message>`; exit 2 for all. `<V>` is any of the 19 non-`build` leaf
commands: `view info diff resize thumbnail convert web optimize responsive auto-orient watermark
edit apply lint completions meta strip|clean|copy|set`.

| Row | Flag given | Refused on | Exact message |
|---|---|---|---|
| R1 | `--watch` | `<V>` | ``--watch is only valid with `build`: there is no rebuild loop to run for this subcommand`` (unchanged) |
| R2 | `--check` / `--frozen` / `--locked` | `<V>` | ``--check (or --frozen/--locked) is only valid with `build`: there is no build lockfile to check for this subcommand`` |
| R3 | `--strict` | `<V>` | ``--strict is only valid with `build --check`: there is no build lockfile to check for this subcommand`` |
| R4 | `--strict` without `--check` | `build` | ``--strict only applies with --check: run `build --check --strict`, or drop --strict`` |
| R5 | `--no-cache` | `<V>` | ``--no-cache is only valid with `build`: there is no build cache for this subcommand`` |
| R6 | `--keep-gps` | every command, `build` included | ``--keep-gps is not supported yet: pixel-lane encodes drop all metadata, GPS included; to keep it, copy the metadata back after the encode with `crustyimg meta copy --from <in> --to <out> -y` (JPEG and PNG)`` |
| R7 | `--name-template` | `responsive` | `responsive does not take --name-template: its output names are fixed as {stem}-{w}w.{ext}` |
| R8 | `--format` | `responsive` | `responsive does not take --format: choose its output formats with --formats (e.g. --formats webp,jpeg)` |

R6 says "not supported yet", never "does not exist" or "removed" (ruling 1). On the sketch (with
earlier wording of R2-R6, same rows): **19 leaves × 7 flags (`--keep-gps --check --frozen --locked
--strict --no-cache --watch`) = 133 rows, plus `build --keep-gps` and `build --strict` = 135 rows,
all exit 2, all with an unchanged `ls -lTR` snapshot.** `responsive --name-template '{stem}.png'`,
`--format webp`, and both together: exit 2, nothing written, both lines in R7, R8 order. Positive
controls on the same binary: `build`, `build --check`, `--frozen`, `--locked`, `--no-cache` exit 0
(`--no-cache` rewrites `bo/`); `build --watch` started its watcher (killed at the timeout, exit 143).

**Not refused (positive controls; each is read by the verb):** `-o`, `--out-dir`, `--name-template`,
`--format`, `-q`, `-j`, `-y`, `-Q`, `-v` on the pixel verbs; `--check`, `--frozen`, `--locked`,
`--strict` with `--check`, `--no-cache`, `--watch` on `build`; `--formats` on `responsive`. The
out-of-scope oddities of finding A (`meta … --format`, `info -o`) stay as they are; none blocks a
ruling.

### Call 2: the logger

- **Module:** `src/cli/logger.rs` (~45 lines with docs). `src/cli/` is already
  `#[cfg(not(target_arch = "wasm32"))]` (`src/lib.rs:77`), so the logger is never compiled for wasm.
  Named `logger`, not `log`, so `log::` paths inside `src/cli/` still mean the crate.
- **Dependency:** `log = "0.4.32"` in `Cargo.toml`'s
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` table, next to `clap_complete`.
  Driven: `Cargo.lock` gains one line (`"log",` in crustyimg's own `dependencies`), package count
  **327 → 327**; `cargo tree -i log -e normal --depth 1` lists `crustyimg` beside av-scenechange,
  avif-parse, fontdb, mio, notify, rav1e, usvg…; with `--no-default-features` it is still there;
  `cargo tree --target wasm32-unknown-unknown -i log` shows no `crustyimg` edge; `cargo deny check`:
  `advisories ok, bans ok, licenses ok, sources ok`; `just wasm-check`: exit 0 (its one warning,
  `MANIFEST_DROPPED_WARNING` never used, is on `main` already).
- **Level mapping** (`fn level_for(verbose: u8, quiet: bool) -> LevelFilter`): `-Q` → `Off` whatever
  the count; 0 → `Off`; 1 → `Warn` (error + warn records); ≥ 2 → `Debug` (adds info and debug).
  `trace` is never shown. `-v -v`, `-vv` and `--verbose --verbose` are the same count (clap `Count`);
  `-vvv` = `-vv`.
- **Install:** in `run()`, the line after `Cli::parse()` and **before** `dispatch`:
  `logger::init(cli.global.verbose, cli.global.quiet);`. At `Off` it installs nothing (today's
  behaviour exactly). Installed once per process, so `build --watch` keeps it for every cycle, and
  rayon workers (`-j`) log through the same global logger.
- **Line format:** `<level>: [<target>] <message>`, level spelled `error`/`warning`/`info`/`debug`
  (crustyimg's own warnings already print `warning: `; the bracketed target says which library spoke).

**Driven on the sketch** with the fixture below (`tests/fixtures/svg/log_levels_40x30.svg`: a rect,
an `<image>` with no `href`, and a `<text font-variation-settings="'wght' 700">`):

```
$ crustyimg -v convert log_levels_40x30.svg --format png -o c.png
warning: [usvg::parser::image] Image lacks the 'xlink:href' attribute. Skipped.
$ crustyimg -vv info log_levels_40x30.svg
warning: [usvg::parser::image] Image lacks the 'xlink:href' attribute. Skipped.
debug: [usvg::parser::text] Found font-variation-settings: ''wght' 700'
```

No `-v`: stderr empty (0 lines). `-v -Q` and `-Q -v` and `-vv -Q`: 0 lines. `-v` shows no `debug:`
line. The output file is byte-identical with and without `-v` (`shasum` equal). Per verb, `-v` gives
1 warning line on `convert`, `info`, `resize`, `thumbnail`, `optimize`, `web`, `lint`, `responsive`;
**`diff a.svg a.svg` gives 2** (it decodes both inputs: the maximum), and `resize a.svg b.svg
--out-dir d -j 4` gives 2 (one per input, from worker threads). The minimum is the default: 0.

**A second, lean-safe source.** `tests/fixtures/avif/photo_128.avif` (AVIF decode is native and
unconditional, DEC-053): `-vv info` prints 5 `debug: [avif_parse] …` lines (first:
`debug: [avif_parse] BoxHeader { name: hdlr, size: 33, offset: 8 } (skipped)`); `-v` prints 0. This
row runs on the lean leg, where the SVG rows are compiled out (`svg` feature).

### Call 3: the font and overlay messages (ruling 4)

A new `CliError::WatermarkAssetUnreadable { what: &'static str, path: String, source: std::io::Error }`,
`#[error("could not read {what} \"{path}\": {source}")]`, `code()` → **3**. In `watermark_overlay`
(`src/cli/ops.rs`):
- font: `Some(path) => Some(std::fs::read(path).map_err(ImageError::Io)?)` becomes a `map_err` to
  the variant with `what: "font file"`;
- overlay: `let overlay = Image::load(image)?;` becomes `std::fs::read(image)` mapped to the variant
  with `what: "watermark image"`, then `Image::decode_path(image, &bytes)?`, so a read failure names
  the file and a decode failure keeps its own error and code.

Driven: `--font /no/font.ttf` → exit 3,
`error: could not read font file "/no/font.ttf": No such file or directory (os error 2)`;
`--font .` → exit 3, `… ".": Is a directory (os error 21)`; `--image /no/logo.png` → exit 3,
`error: could not read watermark image "/no/logo.png": No such file or directory (os error 2)`;
`--image .` → exit 3; `--image bad.png` (text bytes) → exit 4,
`error: unsupported or undetectable image format` (unchanged). Nothing written in each.

### Call 4: DECs (ruling 5)

No new DEC: the guard table and the logger are mechanisms, not decisions. This design commit adds an
amendment to **DEC-003** and **DEC-024**, and adds `src/cli/global_flags.rs` to each one's
`affected_scope` (block-list form; DEC-003 also gains `src/cli/mod.rs`). DEC ids were checked after
the last rebase: highest is DEC-113 (PROJ-017); none taken.

### Call 5: `guidance/ideas.yaml`

This design commit adds `responsive-name-template` beside `manifest-placeholders-favicon` (the
`{hash}` token idea): status `proposed`, likely home PROJ-014, gated on template-token validation on
every verb (finding A) and a ruling on a literal extension against DEC-105 when `responsive` writes
several formats.

## Inputs

- **Files to read:** `src/cli/mod.rs` (`GlobalArgs`, `run`, `refused_build_flags`, `dispatch`),
  `src/cli/ops.rs` (`watermark_overlay`), `tests/build_refuses_output_flags.rs` (the snapshot and
  deadline helpers to copy, not import), `tests/cli.rs` (the `--watch` refusal test),
  `tests/build_lock.rs::cross_env_hash_tolerated_unless_strict`,
  `tests/build_cache.rs::no_cache_flag_bypasses_store`, `tests/build_watch.rs::watch_rejects_verify_modes`.
- **External:** `log` 0.4.32 (`Log`, `set_logger`, `set_max_level`), clap 4.6 `CommandFactory`.

## Outputs

- **Files created:**
  - `src/cli/global_flags.rs` — the guard table and `refused_global_flags`, plus unit tests.
  - `src/cli/logger.rs` — the stderr logger, `level_for`, `init`, plus unit tests.
  - `tests/cli_flags_honest.rs` — FT-1 to FT-10.
  - `tests/fixtures/svg/log_levels_40x30.svg` — the `-v`/`-vv` fixture (text below).
- **Files modified** (⚠ = shared; smallest hunk):
  - `Cargo.toml` — one dependency line plus a two-line comment in the native table.
  - `Cargo.lock` — the one `"log",` line (from `cargo build`, never hand-edited).
  - ⚠ `src/cli/mod.rs` — `mod global_flags;` and `mod logger;` (two lines after `mod common;`);
    the `--watch` block in `dispatch` → the `refused_global_flags` call (−12/+7); `logger::init`
    in `run` (+2); help text of `verbose`, `quiet`, `keep_gps` (+`hide = true`), `no_cache`,
    `check`, `strict`; one sentence in `Responsive`'s doc comment; the `WatermarkAssetUnreadable`
    variant (+12) and its `code()` arm (+1).
  - ⚠ `src/cli/ops.rs` — the two lines in `watermark_overlay` (≈ +12/−2).
  - Docs per the sweep table below (⚠ on each shared file).
- **New exports:** none (`global_flags`, `logger` are private modules; `CliError` gains one variant).
- **PROJ-017-owned paths touched: none.** Not `docs/recipes.md`, `skills/crustyimg/SKILL.md`,
  `CHANGELOG.md`, `tests/common/mod.rs`, `src/operation/`, `src/recipe/`, or `projects/PROJ-017-*`.

Fixture text, `tests/fixtures/svg/log_levels_40x30.svg`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30" viewBox="0 0 40 30">
  <rect x="0" y="0" width="40" height="30" fill="#3366cc"/>
  <image x="0" y="0" width="10" height="10"/>
  <text x="4" y="24" font-family="Go" font-size="10" fill="#ffffff" font-variation-settings="'wght' 700">Hi</text>
</svg>
```

## Acceptance Criteria

- [ ] **AC-1 (build-only flags off `build`).** On every leaf command except `build`, derived from
  `Cli::command()` (meta's subcommands included), each of `--check`, `--frozen`, `--locked`,
  `--strict`, `--no-cache`, `--watch` exits 2 with exactly its R1/R2/R3/R5 line on stderr, and
  writes nothing. A leaf with no test argv fails the test by name. → FT-1, FT-2.
- [ ] **AC-2 (`--keep-gps`).** On every leaf command, `build` included, `--keep-gps` exits 2 with
  exactly the R6 line, which names `meta copy`, and writes nothing; `--keep-gps` is absent from
  `crustyimg --help` and `crustyimg resize --help` and still parses (the R6 line, not clap's
  `unexpected argument`). → FT-1, FT-3.
- [ ] **AC-3 (`build --strict`).** `build --strict` without `--check` exits 2 with exactly R4 and
  writes nothing (no lockfile, no `.crustyimg/`, no output); `build --watch --strict` exits 2 with
  R4 within the deadline (the watcher never starts). `build --check --strict` on a fresh lockfile
  exits 0, and `build --no-cache` exits 0 and writes. → FT-4, plus the existing positive controls
  listed under FT-4.
- [ ] **AC-4 (`responsive`).** `responsive --name-template T` exits 2 with R7, `responsive --format F`
  exits 2 with R8, both flags print R7 then R8; each writes nothing (`--out-dir` not created).
  `responsive --formats webp` still writes. → FT-5.
- [ ] **AC-5 (flags a verb reads are not refused).** `resize` with `--out-dir`, `--name-template`,
  `--format`, `-q`, `-j`, `-y`, `-Q`, `-v` exits 0 and writes the named file. → FT-6.
- [ ] **AC-6 (`-v` shows warnings).** With `-v`, the R-fixture's warning line appears on stderr; without
  `-v` stderr is empty; with `-v -Q` and `-Q -v` it is empty; `-v` shows no `debug:` line; `-vv`
  and `-v -v` show the warning **and** the debug line; the output file is byte-identical with and
  without `-v`. → FT-7 (svg), FT-8 (avif, every leg), logger unit tests.
- [ ] **AC-7 (every record reaches the logger).** `diff` of the fixture against itself shows the
  warning line exactly twice; `resize` of two copies with `--out-dir -j 4` shows it exactly twice.
  → FT-7.
- [ ] **AC-8 (font).** A missing `--font` and a directory `--font` exit 3 with
  `error: could not read font file "<path>": ` + the OS error, never `could not read image input`;
  nothing written. → FT-9.
- [ ] **AC-9 (overlay).** A missing `--image` and a directory `--image` exit 3 with
  `error: could not read watermark image "<path>": ` + the OS error; a non-image `--image` keeps exit 4
  and `unsupported or undetectable image format`; nothing written. → FT-10.
- [ ] **AC-10 (dependency).** `log` is a direct, native-only dependency; `Cargo.lock`'s package count is
  unchanged (327); `cargo deny check` and `just wasm-check` pass. → the build quotes
  `cargo tree -i log -e normal --depth 1`, `grep -c '^name = ' Cargo.lock` on `main` and the branch,
  `cargo deny check`'s last line and `just wasm-check`'s exit.
- [ ] **AC-11 (docs and help).** Each row of the sweep table below is applied verbatim, PROJ-017-owned
  rows excepted (their text goes to the orchestrator). → the build lists `git diff --stat` per file
  and `grep -rn -e '--keep-gps' README.md docs SECURITY.md AGENTS.md` shows only refused-form lines.

## Failing Tests

All in **`tests/cli_flags_honest.rs`** (new) unless named otherwise. They drive the real binary
(`env!("CARGO_BIN_EXE_crustyimg")`) in a `TempDir`, with `current_dir` set to it and **relative
paths** in argv (so a Windows path never needs escaping in an expected message). Each command's
`Output` is assigned to a variable first, then asserted (trap 15). A **snapshot** is the map of every
file under the temp dir (relative path → SHA-256), taken before and after; copy the helper from
`tests/build_refuses_output_flags.rs` (do not add to `tests/common/mod.rs`, which PROJ-017 owns).
Messages are `const` strings copied from the R-table, full line, `error: ` prefix and `\n` included,
compared with `assert_eq!` against the whole stderr (a refusal prints nothing else).

- **FT-1 `every_leaf_command_refuses_build_only_flags_and_keep_gps`** — walks
  `crustyimg::cli::Cli::command()` (`clap::CommandFactory`): for each subcommand, if it has
  subcommands, recurse (that is `meta`), else it is a leaf path such as `["meta", "strip"]`. Skip
  only a leaf named `help`, by name (clap adds it at build time; the test does not call `.build()`,
  so expect none). For each leaf, look up `fn minimal_args(path: &[&str]) -> Option<Vec<&str>>`;
  `None` → `panic!("no test argv for `{}`: add one to minimal_args", path.join(" "))`. Asserts:
  - the number of leaves seen is **20** and equals the number of `minimal_args` entries (a stale
    entry or an empty walk is red); prints `saw 20 leaf commands` (visible with `--nocapture`).
  - **Baseline per leaf:** `minimal_args` alone exits with a code other than 2 and its stderr
    contains none of the R-lines (so the argv is valid and the refusal below is the guard's).
    For `resize`, `convert`, `web`, `optimize`, `watermark`, `edit`, `apply`, `thumbnail`,
    `auto-orient`, `meta strip|clean|set` the baseline also changes the snapshot (the snapshot can
    see a write).
  - For every leaf but `build`, each of `--check --frozen --locked --strict --no-cache --watch
    --keep-gps` appended: exit **2**, stderr == the row's line, snapshot unchanged. (19 × 7 = 133.)
  - For `build`: `--keep-gps` → exit 2, R6, snapshot unchanged. (`build` is never given `--watch`
    here: that starts a watcher; FT-4 covers it with a deadline.)
  - The `minimal_args` table (each in a temp dir holding `in.png` 32×32, `r.toml` = resize max 16,
    and `crustyimg.build.toml` with one target `source = "in.png"`, `recipe = "r.toml"`,
    `out = "bo"`): `view in.png` · `info in.png` · `diff in.png in.png` ·
    `resize in.png --max 8 -o o.png` · `thumbnail in.png -o o.png` ·
    `convert in.png --format jpeg -o o.jpg` · `web in.png -o o.png` · `optimize in.png -o o.png` ·
    `responsive in.png --widths 8 --out-dir rd` · `auto-orient in.png -o o.png` ·
    `watermark in.png --text hi -o o.png` · `edit in.png --invert -o o.png` ·
    `apply --recipe r.toml in.png -o o.png` · `build` · `lint in.png` · `completions zsh` ·
    `meta strip in.png -o m.png` · `meta clean in.png --gps -o m.png` ·
    `meta copy --from in.png --to t.png` (with `t.png` a copy) · `meta set in.png --artist a -o m.png`.
- **FT-2 `refused_global_flags_reports_every_row_in_order`** (unit test in
  `src/cli/global_flags.rs`, `Cli::try_parse_from`, no I/O): `resize a --max 8 --check --no-cache
  --keep-gps` → `[R2, R5, R6]`; `meta strip a --strict` → `[R3]`; `build --strict` → `[R4]`;
  `build --check --strict` → `[]`; `build --no-cache --check` → `[]`; `build --watch --strict` →
  `[R4]`; `responsive a --widths 8 --format webp --name-template x` → `[R7, R8]`;
  `resize a --max 8 --name-template x --format png` → `[]`; `resize a --max 8` → `[]`.
- **FT-3 `keep_gps_is_hidden_from_help_but_still_parsed`** — `crustyimg --help` and
  `crustyimg resize --help`: exit 0, stdout does not contain `--keep-gps`; and
  `crustyimg resize in.png --max 8 -o o.png --keep-gps` stderr == R6 (not clap's
  `unexpected argument '--keep-gps'`).
- **FT-4 `build_refuses_strict_without_check_and_keeps_its_own_flags`** — in a temp project:
  `build --strict` → exit 2, stderr == R4, snapshot unchanged (no `crustyimg.build.lock`, no
  `.crustyimg/`, no `bo/`). `build --watch --strict` spawned with the `DEADLINE`/`TICK` loop from
  `build_refuses_output_flags.rs` → exits by itself within 20 s, exit 2, stderr == R4, snapshot
  unchanged; a run still alive at the deadline is killed and the row fails as "watcher started".
  Then `build` (exit 0, writes the lockfile), `build --check --strict` → exit 0 and no R4 line,
  `build --no-cache` → exit 0 and no R-line. (What `--no-cache` *does* stays proven by
  `no_cache_flag_bypasses_store`, which asserts `(0, 2)` and no `.crustyimg/`.) Existing positive controls that must stay green:
  `tests/build_lock.rs::cross_env_hash_tolerated_unless_strict` (`--check --strict` still exits 7 on
  variance), `tests/build_cache.rs::no_cache_flag_bypasses_store`,
  `tests/build_watch.rs::watch_rejects_verify_modes`, and the `--watch` refusal test in
  `tests/cli.rs` (the `contains("--watch is only valid with `build`")` assertion).
- **FT-5 `responsive_refuses_name_template_and_format`** — `responsive in.png --widths 8 --out-dir rd`
  plus `--name-template '{stem}.png'` → exit 2, stderr == R7; plus `--format webp` → exit 2, R8; plus
  both → exit 2, stderr == R7 line then R8 line; each: snapshot unchanged and `rd` does not exist.
  Control: `--formats webp` → exit 0 and `rd/in-8w.webp` exists (RIFF magic).
- **FT-6 `flags_a_verb_reads_are_not_refused`** — `resize in.png --max 8` with each of:
  `--out-dir d --name-template '{stem}_s.png'` (→ `d/in_s.png`), `--format jpeg -o o.jpg`,
  `-q 40 --format jpeg -o q.jpg`, `-j 2 --out-dir j`, `-y -o in2.png` (over an existing file),
  `-Q -o qq.png`, `-v -o v.png`: each exit 0 and the named file exists. None prints an R-line.
- **FT-7 `verbose_shows_dependency_warnings`** (`#[cfg(feature = "svg")]`, the same gate as
  `tests/input_svg.rs`) — with the fixture copied in as `n.svg` (and `m.svg` for two inputs).
  `W` = `warning: [usvg::parser::image] Image lacks the 'xlink:href' attribute. Skipped.`,
  `D` = `debug: [usvg::parser::text] Found font-variation-settings: ''wght' 700'`. Rows (count lines
  of stderr exactly equal to the line, not `contains`):
  - `convert n.svg --format png -o a.png`: 0 × W, stderr empty.
  - `-v convert … -o b.png`: 1 × W, 0 × D, and `b.png` bytes == `a.png` bytes.
  - `-vv` and `-v -v`: 1 × W, 1 × D.
  - `-v -Q`, `-Q -v`, `-vv -Q`: stderr empty.
  - `-v diff n.svg n.svg`: **2** × W.
  - `-v -j 4 resize n.svg m.svg --max 20 --out-dir jd`: **2** × W.
  A row that cannot see its line fails; nothing in this test skips at runtime.
- **FT-8 `very_verbose_reaches_the_avif_parser`** (no feature gate; runs on the lean leg) —
  `info photo_128.avif` (copied from `tests/fixtures/avif/`): no flag → stderr empty; `-v` → empty;
  `-vv` → at least one line starting `debug: [avif_parse] `; `-vv -Q` → empty.
- **FT-9 `missing_font_names_the_file`** — `watermark in.png --text hi --font no-font.ttf -o o.png`:
  exit 3; stderr starts with `error: could not read font file "no-font.ttf": ` and does not contain
  `could not read image input`; snapshot unchanged. Same with `--font fontdir` (an existing
  directory): exit 3, the same prefix with `"fontdir"`. (Only the prefix is asserted: the OS error
  text differs on Windows.)
- **FT-10 `missing_overlay_names_the_file`** — `watermark in.png --image no-logo.png -o o.png` and
  `--image logodir`: exit 3, stderr starts `error: could not read watermark image "<path>": `, no
  `could not read image input`; snapshot unchanged. Control: `--image bad.png` (bytes `junk\n`) →
  exit 4, stderr == `error: unsupported or undetectable image format\n`.
- **Unit tests, `src/cli/logger.rs`:** `level_for(0,false)=Off`, `(1,false)=Warn`, `(2,false)=Debug`,
  `(3,false)=Debug`, `(1,true)=Off`, `(2,true)=Off`, `(0,true)=Off` (`level_for_quiet_wins`,
  `level_for_counts`).
- **Unit test, `src/cli/mod.rs` tests:** `CliError::WatermarkAssetUnreadable{..}.code() == 3` and its
  `Display` for path `a b.ttf` is `could not read font file "a b.ttf": <io error>`.

### Mutation table (trap 12: location, and every check expected red, for its reason)

Run each on the branch, record red/green per check, quote one failure message per row, revert.
"Reason" is what the failure message must show (trap 13).

| # | Mutation (exact location) | Expected red (reason) | Expected green |
|---|---|---|---|
| M1 | Delete the `refused_global_flags` call and its `if` in `dispatch` (`src/cli/mod.rs`) | FT-1 every flag row (exit 0 quoted, e.g. `info --check`: got 0); FT-3 R6 row (exit 0); FT-4 `--strict` row (exit 0, lockfile written) and `--watch --strict` (watcher started, killed at deadline); FT-5 all three (exit 0, `rd/` written); `tests/cli.rs` `--watch` refusal test (exit 0) | FT-2 (calls the function directly), FT-6..FT-10 |
| M2 | One verb dropped: in `refused_global_flags`, `RefusedOn::AllButBuild => !is_build` → `!is_build && !matches!(cli.command, Commands::Lint { .. })` (`src/cli/global_flags.rs`) | FT-1 `lint` rows for the six R1/R2/R3/R5 flags only (lint's own exit, quoted) | FT-1 `lint --keep-gps` (R6 is `All`), every other leaf; FT-2 (names no `lint`) |
| M3 | `-Q` precedence dropped: in `level_for`, match on `(false, verbose)` instead of `(quiet, verbose)` (`src/cli/logger.rs`) | `level_for_quiet_wins`; FT-7 `-v -Q`, `-Q -v`, `-vv -Q` (W present); FT-8 `-vv -Q` (debug lines present) | FT-7 other rows, FT-8 other rows |
| M4 | Logger installed after the verb: move `logger::init(…)` below `dispatch(&cli)` in `run()` (`src/cli/mod.rs`) | FT-7 `-v`, `-vv`, `-v -v`, `diff`, `-j` rows (0 × W); FT-8 `-vv` (no debug line) | logger unit tests (they never install); FT-7 quiet and default rows |
| M5 | R6 message without `meta copy` (`src/cli/global_flags.rs`, the R6 row) | FT-1 every `--keep-gps` row (20, message mismatch, exit 2); FT-3 parse row (FT-2 compares the code's own constant, so it stays green: expected) | everything else |
| M6 | Font line restored verbatim: `Some(path) => Some(std::fs::read(path).map_err(ImageError::Io)?),` (`src/cli/ops.rs`, `watermark_overlay`) | FT-9 both rows (stderr `error: could not read image input`, exit 3) | FT-10 |
| M6b | Overlay line restored verbatim: `let overlay = Image::load(image)?;` | FT-10 both rows (`could not read image input`) | FT-10 `bad.png` control, FT-9 |
| M7 | `responsive` refuses only one flag: delete the R8 row | FT-5 `--format` row (exit 0, `rd/in-8w.webp` written) and the both-flags row (R7 only); FT-2 responsive row | FT-5 `--name-template` row, control |
| M8 | `build --strict` alone allowed: delete the R4 row | FT-4 `--strict` row (exit 0, lockfile written) and `--watch --strict` row (watcher started); FT-2 `build --strict` and `build --watch --strict` | FT-1, FT-4 positive rows |
| M9 | `--no-cache` refused on `build`: R5's `on` → `RefusedOn::All` | FT-4 `build --no-cache` (exit 2, R5 quoted); FT-2 `build --no-cache --check`; `tests/build_cache.rs::no_cache_flag_bypasses_store` (exit 2) | FT-1 (non-build rows unchanged) |
| M10 | `hide = true` removed from `keep_gps` (`src/cli/mod.rs`) | FT-3 help rows (`--keep-gps` in stdout) | FT-1 |
| M11 | Coverage list made stale: delete the `lint` entry from `minimal_args` (`tests/cli_flags_honest.rs`) | FT-1 panics `no test argv for `lint`` | — |
| M12 | Empty walk: replace the subcommand walk with an empty `Vec` (`tests/cli_flags_honest.rs`) | FT-1 (`saw 0 leaf commands`, expected 20) | — |

Trap 16: no CI or environment-variable change in this spec. Trap 11: the only gate is the
compile-time `svg` feature on FT-7; the lean leg runs FT-8 for the logger; nothing skips at runtime
and nothing branches on `cfg(windows)` (OS error texts are not asserted).

## Implementation Context

### Decisions that apply

- `DEC-003` — the drop-GPS / `--keep-gps` promise; amended by this design (R6 refuses the flag).
- `DEC-024` — "`--keep-gps` remains a no-op until the container lane exists"; amended likewise.
- `DEC-012` — clap derive; global flags stay `global = true`; the guard runs after parse.
- `DEC-007` — exit codes: refusals 2, an unreadable `--font`/`--image` 3.
- `DEC-059` — `--check`/`--strict` semantics; R3/R4 enforce "With `--check`" from its help text.
- `DEC-105` — a name's extension pins the format; why `responsive` refuses a template rather than
  half-honouring one (the ideas entry carries the open question).
- SPEC-071's `--watch` guard — the precedent; its message is moved, unchanged, into R1.

### Constraints that apply

- `ergonomic-defaults` — a flag is honoured or refused with a reason, never silently ignored.
- `metadata-not-via-pixel-encode` — why `--keep-gps` cannot be honoured on the pixel lane; R6 points
  at the container lane (`meta copy`).
- `test-before-implementation` — FT-1..FT-10 fail on `main` (see baseline below).

### Prior related work

- SPEC-071 (`--watch` off `build`), SPEC-134 / DEC-105 (`--name-template` pins format; `build`'s
  `refused_build_flags`), SPEC-128 (recipe asset messages: `could not read 'font' file "…"`),
  SPEC-138 (`build` refuses output flags; the snapshot/deadline test helpers).

### Expected baseline on `main` (quote each RED row's message in Build Completion; trap 13)

FT-1: every flag row red with **exit 0** (or the verb's own code: `lint` 7, `view` 5 without a TTY),
never 2 — the likely wrong-reason red is a bad `minimal_args` argv (baseline exits 2 with a clap
message), which the baseline assertion catches first. FT-3: help rows red (`--keep-gps` listed).
FT-4: `--strict` exit 0; `--watch --strict` killed at the deadline. FT-5: exit 0, `rd/in-8w.png`
written. FT-7/FT-8: no warning/debug lines (no logger). FT-9/FT-10: `could not read image input`.
FT-6 green on `main` (a positive control, by design).

### Docs sweep (exact text; ⚠ shared, keep the hunk to the line; ⛔ PROJ-017-owned: do not edit)

| File:line (on `d62314e`) | Replace with |
|---|---|
| ⚠ `docs/cli-reference.md:21` | `- **Metadata is dropped on pixel-lane encodes,** GPS included. \`--keep-gps\` is refused (exit \`2\`); to keep metadata, run \`crustyimg meta copy --from <in> --to <out> -y\` after the encode (JPEG and PNG).` |
| ⚠ `docs/cli-reference.md:40` (`-Q` row) | append: ` Also silences \`-v\`.` |
| ⚠ `docs/cli-reference.md:41` | `\| \`--verbose\` \| \`-v\` \| Show warnings from the SVG renderer and codecs on stderr; \`-vv\` adds debug detail. Silent by default; \`-Q\` wins. \|` |
| ⚠ `docs/cli-reference.md:42` | `\| \`--keep-gps\` \| \| Refused (exit \`2\`): pixel-lane encodes drop all metadata. Use \`meta copy\` after the encode (JPEG and PNG). Hidden from \`--help\`. \|` |
| ⚠ `docs/cli-reference.md` after line 42 | new row: `\| \`--check\` / \`--strict\` / \`--no-cache\` / \`--watch\` \| \| \`build\` only (see Project builds); refused with exit \`2\` on every other command, and \`--strict\` is refused without \`--check\`. \|` |
| ⚠ `docs/cli-reference.md:204` (responsive, after "Single input.") | ` Output names are fixed as \`{stem}-{w}w.{ext}\`: \`--name-template\` and \`--format\` are refused (exit \`2\`); choose formats with \`--formats\`.` |
| ⚠ `docs/cli-reference.md:425` (first prose line after the `--check --strict` example block) | insert before it, as its own paragraph: `\`--strict\` without \`--check\` is refused (exit \`2\`).` |
| ⚠ `docs/api-contract.md:36` | `\| \`--verbose\` \| \`-v\` \| Dependency log records to stderr: \`-v\` warnings, \`-vv\` adds info and debug. Nothing without it; \`-Q\` wins. \|` |
| ⚠ `docs/api-contract.md:37` (`-Q` row) | `\| \`--quiet\` \| \`-Q\` \| Suppress non-error output, including \`-v\` log records. \|` |
| ⚠ `docs/api-contract.md:39` | `\| \`--keep-gps\` \| \| Refused on every command (exit 2): pixel-lane encodes drop all metadata; the message names \`meta copy\`. Hidden from \`--help\`. \|` |
| ⚠ `docs/api-contract.md:592-593` | replace `Not yet covered: \`responsive\` accepts \`--name-template\` and ignores it.` with `\`responsive\` refuses \`--name-template\` and \`--format\` (exit 2): its names are fixed \`{stem}-{width}w.{ext}\`.` |
| ⚠ `docs/api-contract.md:192` (responsive, after "Single input (no glob/batch in v1).") | ` Names are fixed; \`--name-template\` and \`--format\` are refused (exit 2).` |
| ⚠ `README.md:199` | `> Pixel-lane encodes drop all metadata, GPS included. To keep it, run \`crustyimg meta copy --from in.jpg --to out.jpg -y\` after the encode (JPEG and PNG).` |
| ⚠ `README.md:293` | `-v/--verbose` · `--keep-gps` → `` `-v/--verbose` (dependency warnings to stderr; `-vv` for more) `` and drop `· \`--keep-gps\`` |
| `docs/USAGE.md:164` | `Current safety behavior: pixel-lane encodes drop all metadata, GPS included (\`--keep-gps\` is refused; use \`meta copy\` after the encode to keep it);` |
| ⚠ `docs/data-model.md:122` | `\| GPS \| **dropped** (\`--keep-gps\` is refused, exit 2; \`meta copy\` restores it on JPEG/PNG) \| \`meta clean --gps\` removes \|` |
| `docs/moat.md:96-97` | `a default drop-GPS policy on pixel-lane encodes (\`--keep-gps\` to opt out)` → `a pixel lane that drops GPS with every other tag (\`meta copy\` puts metadata back on JPEG and PNG)` |
| `SECURITY.md:49-50` | `**drops GPS** on pixel-lane encodes unless \`--keep-gps\`;` → `**drops GPS** (with all other metadata) on pixel-lane encodes; \`--keep-gps\` is refused (exit 2);` |
| `SECURITY.md:88` | `default drop-GPS on pixel-lane encodes (\`--keep-gps\` to opt out)` → `pixel-lane encodes drop all metadata, GPS included (\`--keep-gps\` refused)` |
| ⚠ `AGENTS.md:503` | `ICC + copyright/artist; drop GPS unless \`--keep-gps\` (DEC-003).` → `ICC + copyright/artist; drop GPS (\`--keep-gps\` is refused, SPEC-149; DEC-003).` (The ICC/copyright half is finding C; not fixed here.) |
| ⛔ `docs/recipes.md:123` | proposed: `- **Note:** \`optimize\` drops all metadata, GPS included (privacy-safe web prep); \`--keep-gps\` is refused. To keep it: \`crustyimg meta copy --from orig.jpg --to web.jpg -y\`.` |
| ⛔ `skills/crustyimg/SKILL.md:67-68` | proposed: `- **Metadata, GPS included, is dropped** on any command that re-encodes pixels; \`--keep-gps\` is refused (exit 2). If the user asks to keep location data, run \`crustyimg meta copy --from <in> --to <out> -y\` after the encode (JPEG and PNG).` |
| `docs/research/*.md`, `docs/sessions/*.md`, released `CHANGELOG.md` entries | unchanged (historical). |

Help strings in `src/cli/mod.rs` (⚠):
- `verbose`: `/// Show warnings from the SVG renderer and codecs on stderr; \`-vv\` adds debug detail. \`-Q\` silences it.`
- `quiet`: `/// Suppress non-error output, including \`-v\` logging.`
- `keep_gps`: `/// Refused (exit 2): pixel-lane encodes drop all metadata; use \`meta copy\` after the encode.` + `hide = true`
- `no_cache`, `check`: prefix the first line with `` `build` only: ``; `strict`: replace `With \`--check\`:` by `` `build --check` only: ``.
- `Responsive` doc comment: append `Output names are fixed (\`{stem}-{w}w.{ext}\`); \`--name-template\` and \`--format\` are refused: choose formats with \`--formats\`.`

### Proposed CHANGELOG text (⛔ PROJ-017-owned section: the orchestrator places it)

> ### Changed
> - **`--keep-gps` is refused (exit 2).** It never kept GPS: pixel-lane encodes drop all metadata.
>   To keep it, copy it back after the encode: `crustyimg meta copy --from in.jpg --to out.jpg -y`
>   (JPEG and PNG). The flag is hidden from `--help`.
> - **`responsive` refuses `--name-template` and `--format` (exit 2)** instead of ignoring them. Its
>   names are fixed (`{stem}-{w}w.{ext}`); choose formats with `--formats`.
> - **`--check` (`--frozen`, `--locked`), `--strict` and `--no-cache` are refused (exit 2) on every
>   command but `build`**, as `--watch` already was, and `build --strict` without `--check` is refused.
>
> ### Added
> - **`-v` prints warnings from the SVG renderer and codecs on stderr**, such as
>   `warning: [usvg::parser::image] Image lacks the 'xlink:href' attribute. Skipped.`; `-vv` adds
>   debug detail. Without `-v` nothing changes; `-Q` wins.
>
> ### Fixed
> - **A missing or unreadable `watermark --font` or `--image` names the file** (exit 3) instead of
>   saying "could not read image input".

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- Implementing `--keep-gps` (copy-then-clean through the container lane) or DEC-003's selective
  preserve: a feature with its own DEC (filed on PROJ-013 STAGE-052; STAGE-063 finding C).
- Honouring a `responsive` name template and validating unknown template tokens on every verb
  (`guidance/ideas.yaml` `responsive-name-template`; finding A).
- The other accepted-and-ignored combinations: `meta strip|clean|set` with `--format`/`-q`, `-o`
  together with `--out-dir`, output flags on read-only verbs (STAGE-063 finding A).
- The `optimize`/`web` `-q` refusal naming flags the user did not pass (STAGE-063 finding B; may fold
  into SPEC-150).
- The wasm asset refusal's wording and spec ids in `--help` (SPEC-150).
- A corrupt `--image` naming its file (it keeps exit 4 and the decode/format message; not ruling 4).
- Logging from crustyimg's own code (`log::` calls in `src/`): the logger only surfaces dependencies.
- Anything under `src/operation/`, `src/recipe/`, or `projects/PROJ-017-geometry-as-recipe-steps/`.

## Notes for the Implementer

- Build on a branch, **draft PR**, own `CARGO_TARGET_DIR`; checkpoint-commit after the tests are
  written (red) and after they pass. `git commit -s` on every commit.
- Write FT-1..FT-10 first and record the `main` baseline per row (exit code and first stderr line)
  before any `src/` change.
- `refused_build_flags` stays as it is; `global_flags` does not absorb it (its `build` rows test
  `Option::is_some()` on output flags and have their own unit test). Order for `build`: table first.
- Keep the R-lines as `const`s in `global_flags.rs` and copy them as literals into the test (do not
  import them: a test reading the code's own constant cannot catch a changed message).
- `clap::CommandFactory` must be in scope in the test (`use clap::CommandFactory;`); `clap` is a
  normal dependency, so integration tests can use it.
- Run `cargo test --no-default-features` too: FT-7 compiles out, FT-8 must still run and pass.
- Run `just wasm-check` and `cargo deny check` and quote both.
- If a rebase conflicts in a PROJ-017 file, stop and report.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-149-no-flag-accepted-and-ignored`
- **PR (if applicable):** draft, see the report
- **All acceptance criteria met?** yes (AC-1..AC-11; the Windows leg is quoted from the PR's CI)
- **New decisions emitted:** none
- **Files this diff touches:**
  - `src/cli/global_flags.rs` (new) — the 8-row guard table, `refused_global_flags`, FT-2 unit test
  - `src/cli/logger.rs` (new) — stderr logger, `level_for`, `init`, 2 unit tests
  - `src/cli/mod.rs` — two `mod` lines, `--watch` block swapped for the table, `logger::init` in `run`, help strings (`keep_gps` hidden), `Responsive` doc sentence, `WatermarkAssetUnreadable` variant + `code()` arm + unit test
  - `src/cli/ops.rs` — `watermark_overlay`: font and overlay reads name the file
  - `Cargo.toml`, `Cargo.lock` — `log = "0.4.32"` native-only; lock gains one `"log",` line (327 packages before and after)
  - `tests/cli_flags_honest.rs` (new) — FT-1, FT-3..FT-10 (FT-2 is the unit test)
  - `tests/fixtures/svg/log_levels_40x30.svg` (new) — the `-v` fixture
  - `tests/build_refuses_output_flags.rs` — see Deviations
  - `README.md`, `docs/cli-reference.md`, `docs/USAGE.md`, `docs/moat.md`, `SECURITY.md`, `AGENTS.md` — the Docs sweep rows
- **Deferred to ship (SPEC-151, PR #255, still a draft at my last rebase):** `docs/api-contract.md` lines 36, 37, 39, 192, 591-593 and `docs/data-model.md:122`. Also the orchestrator-placed text: `CHANGELOG.md`, `docs/recipes.md:123`, `skills/crustyimg/SKILL.md:67-68`. `guidance/ideas.yaml` `responsive-name-template` was already on `main`.
- **Evidence:**
  - RED on `main` (before any `src/` change): FT-3 `` `--help` lists --keep-gps ``; FT-4 `build --strict: exit code` left 0, right 2; FT-5 `responsive ... --name-template {stem}.png: exit code` left 0, right 2; FT-7 `-v: stderr` W count left 0, right 1; FT-8 `-vv: no avif_parse debug line; stderr:` (empty); FT-9/FT-10 `error: could not read image input`; FT-1 `view --check: exit code` left 5, right 2 (`saw 20 leaf commands`); FT-6 green (control). GREEN after: 9/9 (8/9 on `--no-default-features`, FT-7 compiled out).
  - Real `-vv` line from the fixture: `warning: [usvg::parser::image] Image lacks the 'xlink:href' attribute. Skipped.` then `debug: [usvg::parser::text] Found font-variation-settings: ''wght' 700'`.
  - `cargo tree -i log -e normal --depth 1`: crustyimg listed beside av-scenechange, avif-parse, fontdb, notify, rav1e, resvg, usvg...; `grep -c '^name = ' Cargo.lock` 327 on `main` and on the branch; `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`; `just wasm-check` exit 0.
  - Gates: `cargo fmt --check` 0, `cargo clippy --all-targets -- -D warnings` 0, `just check` 0, lean `cargo test --no-default-features --test cli_flags_honest` ok.
  - Local mutations (scratch worktree, each diff non-empty, reverted to an empty diff). Red checks, with the quoted line:
    - M1: FT-1 `view --check: exit code` left 5; FT-3 `resize --keep-gps: exit code`; FT-4 `build --strict: exit code` (built, exit 0); FT-5 `responsive ... --name-template: exit code`; `tests/cli.rs::watch_on_non_build_subcommand_is_usage_error` got Some(0). FT-2 green.
    - M2: FT-1 only, `lint --check: exit code`. FT-2 green.
    - M3: `level_for_quiet_wins`; FT-7 `["-v", "-Q"]: -Q must win` (W line present); FT-8 `-vv -Q: -Q must win` (debug lines present).
    - M4: FT-7 `-v: stderr` (0 x W); FT-8 `-vv: no avif_parse debug line`.
    - M5: FT-1 `view --keep-gps: stderr` (message lacks `meta copy`); FT-3 `resize --keep-gps: stderr`. FT-2 green, as predicted.
    - M6: FT-9 `error: could not read image input`. M6b: FT-10 same line.
    - M7: FT-5 `responsive ... --format webp: exit code`; FT-2.
    - M8: FT-4 `build --strict: exit code`; FT-2.
    - M9: FT-4 `build --no-cache` (R5 quoted); FT-2; `no_cache_flag_bypasses_store` (`build should exit 0, got Some(2)`).
    - M10: FT-3 `` `--help` lists --keep-gps ``. M11: FT-1 `no test argv for `lint``. M12: FT-1 `saw 0 leaf commands, expected 20`.
  - Windows leg: not runnable here; quoted from the draft PR's CI in the report.
- **Deviations from spec:**
  - `tests/build_refuses_output_flags.rs` (outside the spec's file list): its positive control ran `build --strict` expecting exit 0, which is the behaviour this spec removes. It now runs a plain `build` then `build --check --strict`.
  - `minimal_args` for `watermark` adds `--size 8`: at the default size the mark covers 27% of the 32x32 fixture and the 5% limit refuses it with exit 2, so the spec's argv was a usage error.
  - FT-2 is a unit test in `src/cli/global_flags.rs`; the test file's tests are FT-1, FT-3..FT-10.
- **Follow-up work identified:** none new.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The `watermark` argv in `minimal_args` hit the 5% coverage refusal. On `main` the walk fails at `view` first, so the bad argv only showed once the guard worked: baselines should be checked on a build where the guard is already in.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — `tests/build_refuses_output_flags.rs` asserted `build --strict` exits 0; the file list did not name it.

3. **If you did this task again, what would you do differently?**
   — Run every `minimal_args` baseline standalone against `main` before writing the refusal loop. No design call turned out wrong.

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
