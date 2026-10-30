---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-138
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
    - DEC-057
    - DEC-058
    - DEC-101
    - DEC-105
    - DEC-087
    - DEC-099
  constraints:
    - ergonomic-defaults
    - every-public-fn-tested
    - clippy-fmt-clean
  related_specs:
    - SPEC-063
    - SPEC-064
    - SPEC-066
    - SPEC-134
    - SPEC-137

value_link: >
  Closes one of STAGE-054's two 0.8.1 silent-wrong-output bugs: `build` stops accepting output
  flags it never reads, so a user is never told "ok" while getting a format or location they
  did not ask for.

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
      tokens_total: 3904083
      tokens_breakdown:
        input: 72
        output: 41801
        cache_creation: 115167
        cache_read: 3747043
      estimated_usd: 3.64
      duration_minutes: null
      recorded_at: 2026-10-28
      note: >
        Worktree docs/spec-138-design. Measured from the transcript identified by content
        ("SPEC-138 — DESIGN prompt"), deduped by `.message.id` (36 unique ids, all
        claude-opus-5-5), input/cache_creation/cache_read once per id, MAX output_tokens.
        Priced at Opus $5/$25 per MTok, cache_creation x1.25, cache_read x0.10. Measured before
        the commit/push turns, so it slightly under-counts. Excludes the two release builds
        (main + key mutant), which cost machine time, not tokens.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 7776381
      tokens_breakdown:
        input: 134
        output: 32140
        cache_creation: 127133
        cache_read: 7616974
      estimated_usd: 3.24
      duration_minutes: null
      recorded_at: 2026-10-29
      note: >
        Worktree feat-spec-138-build-refuses-output-flags. Re-measured at ship from the full
        transcript (8813cf27-....jsonl, identified by content "BUILD prompt: "), deduped by
        `.message.id` (60 unique ids, all claude-sonnet-5-5), input/cache_creation/cache_read once
        per id, MAX output_tokens. Priced at Sonnet $3/$15 per MTok, cache_creation x1.25,
        cache_read x0.10. Replaces the earlier 5,060,494 / $2.28, which was taken before the final
        commit/PR turns. Excludes machine time for the release builds and the three-leg matrix.
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 8602182
      tokens_breakdown:
        input: 126
        output: 38128
        cache_creation: 165765
        cache_read: 8398163
      estimated_usd: 6.19
      duration_minutes: null
      recorded_at: 2026-10-29
      note: >
        Worktree verify-spec-138. Measured from the transcript (5199f3ee-....jsonl, identified by
        content "VERIFY prompt: "), deduped by `.message.id` (59 unique ids, all claude-opus-5-5),
        input/cache_creation/cache_read once per id, MAX output_tokens. Priced at Opus $5/$25 per
        MTok, cache_creation x1.25, cache_read x0.10.
    - cycle: ship
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 670030
      tokens_breakdown:
        input: 18
        output: 4923
        cache_creation: 52195
        cache_read: 612894
      estimated_usd: 0.45
      duration_minutes: null
      recorded_at: 2026-10-30
      note: >
        Worktree docs-spec-138-ship. Measured from the transcript (22a3922c-....jsonl, identified
        by content "SHIP prompt: "), deduped by `.message.id` (9 unique ids, all
        claude-sonnet-5-5), same method and Sonnet pricing as build. Measured before the final
        commit/push turns, so it slightly under-counts.
  totals:
    tokens_total: 24829946
    estimated_usd: 16.43
    session_count: 5
---

# SPEC-138: build refuses the output flags its manifest owns

## Context

STAGE-054 backlog item "`build --format` is accepted and silently ignored", filed by the 0.8.0
release-notes verify on 2026-10-27. One of the two bugs that open 0.8.1 (PROJ-011).

`--format` is a **global** flag (`src/cli/mod.rs:88-90`), so `build --help` lists it as "Force
output format". `src/cli/build.rs` never reads `global.format`. DEC-057 already says `-o`,
`--out-dir` and `--name-template` "are ignored by `build`: the manifest declares them". Nothing
tells the user. The sweep below shows that ignoring is not limited to `--format`.

**What the user sees today** (`main` @ 3e6d39a, release build):

```
$ crustyimg build --format png        # three targets: name "{stem}.webp" / recipe format="gif" / neither
$ echo $?
0                                      # no stderr; out/ holds a.webp, a.gif, a.jpg, and no PNG
$ crustyimg build -o x.png --out-dir other --name-template '{stem}.png'
$ echo $?
0                                      # same three files, same bytes; no x.png, no other/
```

## Goal

`build` refuses, with exit 2 and before any output is written, each global flag that would set
the output format, path or name: `--format`, `-o`, `--out-dir`, `--name-template`. The message
names the flag and says where the manifest sets that instead. A `build` without those flags is
byte-for-byte unchanged (same outputs, cache keys and lockfile).

## Frame

**Verdict: GO, [S]**, on the recommended rule (refuse). Honouring `--format` would be [M]; see
below.

### Premise check (driven 2026-10-28)

Release build of `main` @ 3e6d39a, fresh `CARGO_TARGET_DIR`. Fixture: `src/a.jpg` (64×48 JPEG,
made by ImageMagick) and a three-target manifest: **ext** = `name = "{stem}.webp"` with a v1 recipe,
**recipefmt** = a v2 recipe with `format = "gif"`, **neither** = a v1 recipe with no name. Formats
read with `magick identify`, not crustyimg.

| Run | exit | ext | recipefmt | neither |
|---|---|---|---|---|
| `build` (baseline, control) | 0 | WEBP | GIF | JPEG |
| `build --format png` (cache warm) | 0 | WEBP | GIF | JPEG |
| `build --format png --no-cache` | 0 | WEBP | GIF | JPEG, **SHA-256 identical to baseline** |
| control: `apply --recipe fmt.toml a.jpg -o ctl.out --format png` | 0 | | **PNG** | |
| **0.7.1** release asset, plain target, `build --format png --no-cache` | 0 | | | JPEG |

Confirmed on `main` for all three target kinds, and on 0.7.1 for the plain target. 0.7.1 has no
recipe `format` (v2 recipes are 0.8.0), so the recipefmt target was refused there at parse.
`--format bogus` also exits 0 on `build` today: the value is never even validated.

### Global-flag sweep on `build` (`GlobalArgs`, `src/cli/mod.rs:71-144`)

Each run uses `--no-cache` and compares all three outputs' SHA-256 to the baseline.

| Flag | Read in `build`? | Driven result | Verdict |
|---|---|---|---|
| `--format` | no | exit 0, same bytes; `bogus` also exit 0 | **ignored silently**: the bug |
| `-o` / `--output` | no (DEC-057: "ignored") | exit 0, same bytes, no `x.png` written | **ignored silently** |
| `--out-dir` | no (DEC-057) | exit 0, same bytes, `other/` not created | **ignored silently** |
| `--name-template` | no (DEC-057) | exit 0, same bytes (`{stem}.png` asked) | **ignored silently** |
| `-q` / `--quality` | yes (`build.rs:559`, over `recipe.quality`) | bytes differ (neither: 718 → 673 B) | honoured |
| `-j` / `--jobs` | yes (`build.rs:819`) | same bytes (pool size) | honoured |
| `-Q` / `--quiet` | yes | 0 stdout lines | honoured |
| `--no-cache`, `--check`/`--frozen`/`--locked`, `--watch` | yes | (build's own flags) | honoured |
| `--strict` | only with `--check` | alone: exit 0, no effect | documented as "With `--check`"; minor, filed |
| `-y` / `--yes` | no | exit 0, same bytes | no-op **by design**: a build overwrites its own outputs (DEC-057) |
| `--keep-gps` | **no verb reads it** | | dead on every verb, filed (below) |
| `-v` / `--verbose` | **no verb reads it** | | dead on every verb, filed (below) |
| `--json` | not a global (per-verb flag) | exit 2, clap "unexpected argument" | refused |

**Recommendation:** the four output flags (`--format`, `-o`, `--out-dir`, `--name-template`) are the
same bug: a flag that asks for an output the manifest owns, accepted at exit 0 and silently ignored.
They belong **in this spec**. Under "refuse" they cost one guard and four test rows, not four fixes.
`--keep-gps`, `-v` and `--strict` are a different bug (flags that do nothing on any verb), so they
go to a backlog entry.

### Honour or refuse: maintainer question; recommendation **refuse**

- **`build`'s promise is that the manifest declares the build.** DEC-057 already moved `-o`,
  `--out-dir` and `--name-template` into the manifest. Honouring `--format` makes one manifest give
  two outputs and two lockfiles. The lockfile pins `{path, key, hash}` per output
  (`src/build/lock.rs:150-161`); with the default `{stem}.{ext}`, `--format png` renames every
  output, so `build --check` after a `build --format png` (or the reverse) reports drift and exits
  7, with nothing in the committed manifest to explain why.
- **The format already has two manifest homes:** the target's `name` extension (DEC-087/DEC-105)
  and the recipe's `format` (DEC-099). A user who wants PNG writes `name = "{stem}.png"`. The
  refusal message can say exactly that.
- **Honouring costs [M], refusing [S].** Honour = a new top rung in both plan builders
  (`plain_target_plan`, `build.rs:88`, and the terminal-`optimize` plan at `:60-71`), validation of
  the value, and a matrix: 3 target kinds × 2 plan kinds × cache hit/miss, distinct cache keys, the
  `--check` drift row, and `--watch`. The key would follow on its own, since `target_recipe_hash`
  (`build.rs:137-160`) already folds in the format plan (DEC-101), but each of those rows still
  needs a test. Refuse = one guard in `src/cli/mod.rs`'s dispatch, next to the existing `--watch`
  guard (`:867-873`), covering `run_build` and `run_build_watching` from one site.
- **Against refusing:** `-q` is already a CLI override on `build` (driven: bytes change), so
  "manifest owns everything" is not absolute. Also, a script that passes `--out-dir` to every verb
  would go from exit 0 to exit 2. Both are weaker than a flag that silently does nothing: exit 0
  with the wrong format is the worse failure.
- **Under refuse, `--check`/`--frozen`:** the guard fires first, so it is exit 2 before any lockfile
  read, and a lockfile cannot change. **Under honour** (for completeness): `--format` changes keys,
  and with `{ext}` names it changes paths, so `--check` exits 7 whenever the flag differs from the
  run that wrote the lock.

**Proposed message shape** (design owns the wording, which must not carry spec/DEC ids):
`error: build does not take --format: a build's output format is set in the manifest; give the
target a name with an extension (name = "{stem}.png") or set format in its recipe` → exit 2.
`-o`/`--out-dir`/`--name-template` → "… set the target's `out` / `name` in the manifest".

### Bytes: the check design must require

The guard touches only CLI dispatch. Require: (a) **no diff** under `src/build/` or in
`src/cli/build.rs`'s plan and key code (a `git diff --stat` line in Build Completion); and (b) a
**golden lockfile** test. Build a fixture project (the three targets above) with `main`'s binary
**at the same crate version**, commit its `crustyimg.build.lock` (which holds every output's key and
hash), and assert that the post-change `build` writes an identical lock. ⚠ The crate version is in
every key (DEC-058 #2), so capture the golden **before** any 0.8.1 version bump, or regenerate it in
the bump commit.

### Sizing: [S]

Code: one guard in `src/cli/mod.rs` (~15 lines), message text, and the build section of the
docs. Test matrix: 4 flags × refuse (exit 2 + message names the flag + **nothing written**, incl. no
`.crustyimg/` or lockfile change) = 4 rows; `--watch` + one flag = 1 row (same guard, proves the
watch entry is covered); `--check` + `--format` = 1 row; the golden-lockfile identity row; a
positive control that `apply` still honours all four flags = 1 row. **About 8 rows, one test file.**
If the maintainer rules **honour**, it becomes [M]. Cut: honour `--format` only, still refuse
`-o`/`--out-dir`/`--name-template` here, and make the `--format` rung its own spec.

### Scope

**In:** refuse `--format`, `-o`, `--out-dir`, `--name-template` on `build` and `build --watch` (exit
2, nothing written); the message; the golden-lockfile check; `docs/cli-reference.md` (the build
section, and the global-flag table's `--format` row, which says nothing about `build`); the README
global-flags line if it implies `build`; correcting 0.8.0's wording ("`--format` has no effect") in
`CHANGELOG.md`'s new entry; amending DEC-057's "`-o`/`--out-dir`/`--name-template` are ignored by
`build`" (Neutral) to "refused".

**Out:** see `### Out of scope` below.

### Rough build file list

- `src/cli/mod.rs`: the guard in the dispatch, next to the `--watch` guard (`:867`)
- `tests/build_refuses_output_flags.rs` (new): the ~8 rows
- `tests/fixtures/build_golden/` (new): fixture project + golden `crustyimg.build.lock`
- `docs/cli-reference.md`, `README.md` (if it implies `build` honours globals), `CHANGELOG.md`
- `decisions/DEC-057-…md`: amend the Neutral bullet (no new DEC expected; DEC-108 may be released)

### Open maintainer questions

1. **Honour or refuse `build --format`?** Recommend **refuse** (exit 2): it keeps the manifest the
   single source, and the format already has two manifest homes.
2. **Refuse all four output flags, or only `--format`?** Recommend **all four**. `-o`, `--out-dir`
   and `--name-template` are the same silent ignore, and one guard covers them. Cost: a script that
   passes them to `build` goes from 0 to exit 2.

### Maintainer rulings (2026-10-28, settled)

1. **Refuse.** `build --format` exits 2 with a message saying where the format is set in the
   manifest. It is not honoured.
2. **All four.** `--format`, `-o`, `--out-dir` and `--name-template` are refused on `build` and
   `build --watch`, through one guard.

Added after design (2026-10-28):

3. **The message names `-o/--output`** (Call 2), whichever spelling was typed. It follows the
   existing `-q/--quality` message (`src/cli/optimize.rs:497`). No argv re-parse.
4. **No committed golden lockfile.** Call 6's two-binary `compare-builds.sh` plus the empty
   `git diff --stat` of `src/build/` and `src/cli/build.rs` replace it.
5. **DEC-057's stale "output format follows the source" bullet** is fixed in the same amendment
   (Call 8).
6. **All 15 tests ship, at [S].** FT-6 stays.

## Design

Driven 2026-10-28 on a release build of `main` @ `7e7a3f0` (`crustyimg 0.8.0`), fresh
`CARGO_TARGET_DIR`, each run in a pristine copy of the frame's three-target project (**ext** =
`name = "{stem}.webp"`, **recipefmt** = a v2 recipe with `format = "gif"`, **neither** = a v1
recipe, no name). Formats read with `magick identify`.

### What `main` does today (the RED baseline every refusal row cites)

| Run on `main` | exit | stderr | written |
|---|---|---|---|
| `build` (control) | 0 | the 4 progress lines (`target #0: …` … `built 3 targets, 3 outputs …`) | 3 outputs + 3 `.crustyimg/` entries + lockfile = **7 files** |
| `build --format png` / `--format bogus` / `--format jpeg` / `--format ''` | 0 | same 4 progress lines | 7 files |
| `build -o x.png` / `--output x.png` / `--output=x.png` / `-ox.png` / `-o ''` | 0 | same | 7 files, no `x.png` |
| `build --out-dir other` / `--out-dir .` | 0 | same | 7 files, no `other/` |
| `build --name-template '{stem}.png'` / `'{stem}.{ext}'` | 0 | same | 7 files |
| `build --format png -o x.png --out-dir other --name-template '{stem}.png'` | 0 | same | 7 files |
| `--format png build` (flag before the verb) | 0 | same | 7 files |
| no manifest, `build --format png` | **3** | `error: could not read build manifest crustyimg.build.toml: No such file or directory (os error 2)` | nothing |
| malformed manifest (`bogus = 3`), `build --format png` | **2** | `error: could not parse build manifest: TOML parse error at line 2, column 1` | nothing |
| built, recipe edited, `build --check --format png` | **7** | the drift report | lockfile SHA-256 unchanged |
| `build --watch --format png` | — | progress lines | **still running after 4 s** (watching) |
| `build --watch --check --format png` | 2 | `error: --watch cannot be combined with --check/--frozen/--locked: …` | nothing |
| `build -q 50 -j 2 -Q --no-cache -y` (honoured flags, control) | 0 | empty | 3 outputs + lockfile |
| `build --strict` (control) | 0 | progress lines | 7 files |
| `apply --recipe v1.toml src/a.jpg -o x.png` (control) | 0 | | `x.png` is PNG |
| `apply … -o f.out --format png` (control: `--format` alone decides) | 0 | | `f.out` is PNG |
| `apply … --out-dir d` (control) | 0 | | `d/a.jpg` |
| `apply … --out-dir t --name-template '{stem}_w.png'` (control) | 0 | | `t/a_w.png`, PNG |

Every refused flag, in every spelling and position, with any value, exits 0 on `main` and
writes the same 7 files. Two of the error-order rows (malformed manifest, `--watch --check`)
**already exit 2 on `main`**, so the exit code cannot tell the refusal apart from the other
error there: those rows assert the **stderr text**.

### Call 1: how the guard knows a flag was given

All four are `Option<String>` with **no clap default and no env binding**
(`src/cli/mod.rs:73-90`): `output` (`short = 'o', long`), `out_dir`, `name_template`, `format`,
each `#[arg(long, global = true)]` and nothing else. clap's `env` feature is **not enabled**
(`Cargo.toml:167`: `clap = { version = "4.6.1", features = ["derive"] }`), so no `Arg::env` can
bind them. The name template's `{stem}.{ext}` default is applied downstream, in
`src/cli/common.rs:293` (`.unwrap_or_else(|| "{stem}.{ext}".to_owned())`), not by clap.

So **`Option::is_some()` is exact**: `Some` means the user typed the flag, whatever the value
(`--name-template '{stem}.{ext}'`, `-o ''` and `--format ''` all parse to `Some`, driven above).
`ArgMatches::value_source` (clap_builder 4.6, `ValueSource::{DefaultValue, EnvVariable,
CommandLine}`) would give the same answer and needs the raw `ArgMatches`, which `dispatch` does
not have (`run` calls `Cli::parse()`, `:845`). Not used.

**Guard against a future default:** the unit test parses `["crustyimg", "build"]` and asserts
the four fields are `None`. If someone later adds `default_value` to one of them, that field
becomes `Some` on a bare `build`, the guard refuses every build, and this unit test plus FT-13
go red. That is the failure we want.

### Call 2: which spelling the message names

clap 4.6 does not record which spelling of a flag matched (`value_source` only says
`CommandLine`). Recovering it means re-scanning `std::env::args`, which must then understand
`-ox.png`, `--output=x.png`, clusters like `-Qo x.png`, and a value that looks like a flag. Not
worth a parser. **The `-o` message names both spellings, `-o/--output`**, which always includes
the one typed and matches how `--help` lists it (`-o, --output`). The other three have one
spelling each.

### Call 3: the messages (exact)

One shape for all four: `build does not take <FLAG>: <what the manifest owns>; <where to set
it> instead`. `run` prints each `CliError` as `error: {e}` (`:848`). Manifest key names checked
against the parser, `src/build/mod.rs:173-182` (`Target { source, recipe, out, name }`), and the
recipe's top-level `format` (DEC-099).

| Flag | stderr line |
|---|---|
| `-o` / `--output` | `error: build does not take -o/--output: the manifest says where a build writes; set the target's out and name instead` |
| `--out-dir` | `error: build does not take --out-dir: the manifest says where a build writes; set the target's out instead` |
| `--name-template` | `error: build does not take --name-template: the manifest names a build's outputs; set the target's name instead` |
| `--format` | `error: build does not take --format: the manifest sets a build's output format; give the target's name an extension (name = "{stem}.png") or set format in its recipe instead` |

These are the **entire** stderr: nothing else is printed, because nothing else runs.

### Call 4: several refused flags at once: name all of them

A user who passes `-o` and `--format` and is told only about `-o` drops it, re-runs, and is
refused again. **Every given refused flag gets its own line, one exit 2.** The lines come in a
**fixed order** (`-o/--output`, `--out-dir`, `--name-template`, `--format`: `GlobalArgs`'
declaration order), whatever the command-line order, so the output is deterministic and
testable. Only the flags actually given appear.

Mechanism (recommended, not binding): a private pure function in `src/cli/mod.rs`,
`fn refused_build_flags(global: &GlobalArgs) -> Vec<&'static str>`, returning each message
without the `error: ` prefix; the guard returns
`CliError::Usage(lines.join("\nerror: "))` when the list is non-empty, so `run`'s single
`error: ` prefix starts the first line and the join starts the rest. No new `CliError` variant
and no change to `code()`.

### Call 5: order against other errors

The guard sits in `dispatch` (`src/cli/mod.rs:861`), **after** the existing non-`build`
`--watch` guard (`:867-873`) and **before** the `match`, applying only to `Commands::Build`.
Nothing about the manifest, lockfile, recipe or sources has run yet, and both
`run_build` and `run_build_watching` (`:889-893`) sit behind it, so the `--watch` × `--check`
check inside `run_build_watching` is also later. Rows that prove it:

| Other error | `main` | After | Row |
|---|---|---|---|
| missing manifest | exit 3 | exit 2 + refusal line, no "could not read" | FT-8 |
| malformed manifest | exit 2, "could not parse" | exit 2, refusal line only, no "could not parse" | FT-9 |
| `--check` / `--frozen` against a stale lockfile | exit 7 | exit 2, lockfile and outputs unchanged | FT-10 |
| `--watch --check` | exit 2, "--watch cannot be combined" | exit 2, refusal line only | FT-12 |

`apply --watch --format png` still hits the older non-`build` `--watch` guard first; that is
unchanged and not tested here.

### Call 6: byte identity, and the version-bump trap

**Chosen: option (c)**, a one-time Build Completion comparison plus a structural no-diff check.
**No committed golden lockfile.**

Why not (a) or (b): every key is a SHA-256 that folds in `env!("CARGO_PKG_VERSION")`
(`src/build/cache.rs:249`), so a key cannot be "normalised": the version is inside the hash, not
beside it. Comparing only the output `hash` fields would survive the bump but miss a key-only
change (shown below: the mutant changes one key and **no** output byte). A committed golden that
the 0.8.1 bump commit must regenerate is a trap that fires on a release day, for a change that
touches no key code.

The check, run by the build and **re-run by verify**:

1. **Structural:** `git diff --stat origin/main...HEAD -- src/build/ src/cli/build.rs` prints
   nothing. Paste the (empty) output in Build Completion.
2. **Two binaries, one fixture:** build `main`'s merge-base and the PR head in separate
   `CARGO_TARGET_DIR`s (both are `0.8.0`, so keys are comparable), then run
   `compare-builds.sh BASE_BIN NEW_BIN REPO_ROOT` (verbatim below). It builds the
   three-target fixture from `tests/fixtures/optimize/already_compressed.jpg` with each binary
   in its own fresh directory, checks the fixture really made 3 outputs, and compares the
   lockfile with `cmp` and every output's SHA-256. Exit 0 prints
   `IDENTICAL: lockfile and 3 outputs`. Paste its output in Build Completion.

**Proven both ways at design** (2026-10-28):
- control, `main` vs `main`: exit 0, `IDENTICAL: lockfile and 3 outputs`;
- **mutation:** one line, `hasher.update(b"x");`, added after `hasher.update(toml.as_bytes());`
  in the `OutputFormatPlan::Preserve` arm of `target_recipe_hash` (`src/cli/build.rs:150-152`):
  exit 1, `LOCKFILE DIFFERS`, exactly one line changed (the plain target's `key =`). The output
  bytes were identical, so an output-hash-only check would have stayed green.

```sh
#!/bin/sh
# compare-builds.sh BASE_BIN NEW_BIN REPO_ROOT
# Runs the same three-target build with two binaries in two fresh directories and
# compares the lockfile byte-for-byte and every output's SHA-256. Exit 0 = identical.
set -eu
BASE=$1; NEW=$2; REPO=$3
W=$(mktemp -d)
mk() {
  d=$W/$1; mkdir -p "$d/src"
  cp "$REPO/tests/fixtures/optimize/already_compressed.jpg" "$d/src/a.jpg"
  printf 'version = "1"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = 32\n' > "$d/v1.toml"
  printf 'version = "2"\nformat = "gif"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = 32\n' > "$d/v2gif.toml"
  printf 'version = 1\n\n[[target]]\nsource = "src/*.jpg"\nrecipe = "v1.toml"\nout = "out/ext"\nname = "{stem}.webp"\n\n[[target]]\nsource = "src/*.jpg"\nrecipe = "v2gif.toml"\nout = "out/recipefmt"\n\n[[target]]\nsource = "src/*.jpg"\nrecipe = "v1.toml"\nout = "out/neither"\n' > "$d/crustyimg.build.toml"
  echo "$d"
}
B=$(mk base); N=$(mk new)
(cd "$B" && "$BASE" build -Q)
(cd "$N" && "$NEW" build -Q)
# Positive control: the fixture really built three outputs of three formats.
[ "$(find "$B/out" -type f | wc -l | tr -d ' ')" = 3 ] || { echo "FIXTURE: expected 3 outputs"; exit 3; }
status=0
cmp -s "$B/crustyimg.build.lock" "$N/crustyimg.build.lock" || { echo "LOCKFILE DIFFERS"; diff "$B/crustyimg.build.lock" "$N/crustyimg.build.lock" || true; status=1; }
(cd "$B" && find out -type f | sort | xargs shasum -a 256) > "$W/base.sha"
(cd "$N" && find out -type f | sort | xargs shasum -a 256) > "$W/new.sha"
cmp -s "$W/base.sha" "$W/new.sha" || { echo "OUTPUT BYTES DIFFER"; diff "$W/base.sha" "$W/new.sha" || true; status=1; }
[ $status = 0 ] && echo "IDENTICAL: lockfile and 3 outputs"
rm -rf "$W"
exit $status
```

Keep the script in the session scratchpad, not the repo. Running it once costs two release
builds; that is the price of not carrying a golden that breaks at every bump.

### Call 7: CHANGELOG, `Breaking`, not `Changed`

A script that passes any of these four flags to `build` goes from exit 0 to exit 2. 0.8.0
already set the precedent: "a recipe step with a key its op does not take … is now refused"
went under `### Breaking` (`CHANGELOG.md:42`). Same kind of change, same heading. The 0.8.0
section's "`--format` has no effect on `build`" (`:34`) and "(it does not apply to `build`)"
(`:57`) stay as released history; the new entry supersedes them. Exact text, under
`## [Unreleased]`:

```markdown
### Breaking

- **`build` now refuses `--format`, `-o`/`--output`, `--out-dir` and `--name-template`**
  (exit `2`, nothing written), including with `--watch` and `--check`. A build's outputs are
  declared in its manifest, and these flags used to be accepted and silently ignored:
  `build --format png` exited `0` and wrote no PNG. Set the format with the target's `name`
  extension (`name = "{stem}.png"`) or the recipe's `format`, and the location with the
  target's `out` and `name`. Drop these flags from any script that passes them to `build`.
```

### Call 8: the DEC: amend DEC-057; **DEC-108 is released**

DEC-057 already governs `src/cli/mod.rs` (`affected_scope`, line 27) and already holds the rule
being changed (Neutral, line 135: "`-o`/`--out-dir`/`--name-template` are ignored by `build` —
the manifest declares them"). No new decision is made: the manifest still owns outputs; only the
response to a stray flag changes from ignore to refuse. Amendments:

- **DEC-057, Neutral (line 135)** → "`-o`/`--output`, `--out-dir`, `--name-template` and
  `--format` are **refused** by `build` and `build --watch` (exit 2, before the manifest is
  read): the manifest declares each target's `out`, `name` and, through the name's extension or
  the recipe's `format`, its output format. *(Amended 2026-10-28: through 0.8.0 they were
  accepted and silently ignored.)*"
- **DEC-057, Neutral (line 137-138)**, "Output format follows the source format … A per-target
  `format`/`quality` override is deliberate future scope": append "*(Since then the target's
  `name` extension and the recipe's `format` pin it: DEC-087, DEC-099, DEC-105.)*" Same bullet
  list, stale since 0.8.0, and the refusal message now points users at exactly those two keys.
- **DEC-105, rule 2** (line 65): append "On `build`, `--format` is refused (exit 2), so the
  chain there starts at the target's `name` extension." Rule 2 currently reads as if `--format`
  tops the chain on every verb.
- **DEC-087**: no change. Its rejected option "a new `--format`/manifest-level format key" is
  still rejected.
- **DEC-099** (line 60) and `docs/api-contract.md:626` already say "`build` has neither
  `--format`/`-o`": still true. Only the api-contract line gets a clause (below).

## Inputs

- **Files to read:** `src/cli/mod.rs:65-144` (`GlobalArgs`), `:841-893` (`run`, `dispatch`, the
  `--watch` guard, the `Build` arm); `src/build/mod.rs:170-200` (manifest key names);
  `tests/build_watch.rs:80-130` (the `ChildGuard` / `spawn_watch` / `build_once` harness to
  copy); `decisions/DEC-057-*.md`, `DEC-105-*.md`.
- **External APIs:** none.
- **Related code paths:** `src/cli/build.rs` (must not change), `src/build/` (must not change).

## Outputs

- **Files created:** `tests/build_refuses_output_flags.rs`: FT-1 to FT-14.
- **Files modified:**
  - `src/cli/mod.rs`: the guard in `dispatch`, `refused_build_flags`, and FT-15 in its
    `#[cfg(test)]` module.
  - `docs/cli-reference.md`: the build section and four rows of the global-options table.
  - `docs/api-contract.md:626`: one clause.
  - `CHANGELOG.md`: the `### Breaking` entry under `[Unreleased]`.
  - `decisions/DEC-057-build-manifest-format-and-build-semantics.md`: two Neutral bullets.
  - `decisions/DEC-105-an-output-names-extension-pins-the-format-on-every-verb.md`: rule 2.
- **Not modified:** `README.md`. Checked 2026-10-28: its flag lines (`:178-250`) describe the
  batch verbs, and its `build` paragraph (`:268`) names no flag. The builder re-greps
  (`grep -n 'build' README.md`) and records the result.
- **New exports:** none (`refused_build_flags` is private).
- **Database changes:** none. No version bump, no `CACHE_SCHEMA_VERSION` change, no lockfile
  format change.

## Acceptance Criteria

Surfaces: `build`, `build --watch`, `build --check`, `apply`. "Nothing written" means a
snapshot of every file under the project directory (path → SHA-256, `.crustyimg/` and the
lockfile included) is identical before and after the run.

- [ ] **AC-1 (`build`, `--format`).** `build --format png` exits 2, its stderr is exactly the
  `--format` line from Call 3, and nothing is written. (FT-1)
- [ ] **AC-2 (`build`, `-o`).** `build -o x.png`, `build --output x.png`,
  `build --output=x.png` and `build -ox.png` each exit 2 with exactly the `-o/--output` line;
  nothing is written and `x.png` does not exist. (FT-2)
- [ ] **AC-3 (`build`, `--out-dir`).** `build --out-dir other` exits 2 with exactly the
  `--out-dir` line; nothing is written and `other/` does not exist. (FT-3)
- [ ] **AC-4 (`build`, `--name-template`).** `build --name-template '{stem}.png'` exits 2 with
  exactly the `--name-template` line; nothing is written. (FT-4)
- [ ] **AC-5 (`build`, any value).** A refused flag is refused whatever its value, including the
  value `build` would have used anyway or an empty one: `--name-template '{stem}.{ext}'`,
  `-o ''`, `--out-dir .`, `--format jpeg` (the plain target's own format), `--format ''` and
  `--format bogus` each exit 2 with that flag's line, nothing written. (FT-5)
- [ ] **AC-6 (`build`, position).** A refused flag before the verb is refused the same way:
  `--format png build` and `--out-dir other build` each exit 2 with that flag's line. (FT-6)
- [ ] **AC-7 (`build`, several).** `build --format png --name-template '{stem}.png' -o x.png
  --out-dir other` exits 2 and its stderr is exactly the four lines in the order `-o/--output`,
  `--out-dir`, `--name-template`, `--format`. `build --format png --out-dir other` prints exactly
  the `--out-dir` and `--format` lines, in that order. Nothing is written. (FT-7)
- [ ] **AC-8 (`build`, before a missing manifest).** With no manifest, `build --format png`
  exits 2 (not 3), stderr is exactly the `--format` line. (FT-8)
- [ ] **AC-9 (`build`, before a malformed manifest).** With a manifest that fails to parse,
  `build --out-dir other` exits 2 with exactly the `--out-dir` line, and stderr does not contain
  `could not parse build manifest`. (FT-9)
- [ ] **AC-10 (`build --check`, before the lockfile).** After a clean `build`, with a recipe
  edited so the lockfile is stale, `build --check --format png` and `build --frozen -o x.png`
  each exit 2 (not 7) with that flag's line, and the lockfile, the outputs and `.crustyimg/` are
  unchanged (snapshot). (FT-10)
- [ ] **AC-11 (`build --watch`).** `build --watch --format png` and
  `build --watch --name-template '{stem}.png'` each exit 2 with that flag's line within the
  test's deadline, never start watching, and write nothing. (FT-11)
- [ ] **AC-12 (`build --watch --check`).** `build --watch --check -o x.png` exits 2 with exactly
  the `-o/--output` line, and stderr does not contain `--watch cannot be combined`. (FT-12)
- [ ] **AC-13 (`build`, honoured flags unchanged).** `build -q 50 -j 2 -Q --no-cache -y` and
  `build --strict` each exit 0 and write the three outputs; then `build --check` exits 0. A bare
  `build` changes the snapshot (the helper sees writes). (FT-13)
- [ ] **AC-14 (`apply`, the four flags still work).** On `apply --recipe`: `-o x.png` writes a
  PNG; `--format png -o f.out` writes a PNG (the name does not pin, so `--format` decided);
  `--out-dir d` writes `d/<stem>.jpg`; `--out-dir t --name-template '{stem}_w.png'` writes
  `t/<stem>_w.png` as a PNG. Formats read by magic bytes, not by extension. (FT-14)
- [ ] **AC-15 (guard input).** `refused_build_flags` returns nothing for a bare `build`, and
  exactly the given flags, in the fixed order, for each subset tested. (FT-15)
- [ ] **AC-16 (`build`, bytes unchanged).** `git diff --stat origin/main...HEAD -- src/build/
  src/cli/build.rs` is empty, and `compare-builds.sh` against the merge-base binary prints
  `IDENTICAL: lockfile and 3 outputs`. Both outputs are pasted in Build Completion. (Call 6; not
  a `cargo test`.)
- [ ] **AC-17 (docs).** `docs/cli-reference.md`: the global-options rows for `--output`,
  `--out-dir`, `--name-template` and `--format` each end with "Refused by `build` (exit `2`)."
  and the `build [FILE]` section gains the paragraph in Notes; `docs/api-contract.md:626` gains
  the clause in Notes; `CHANGELOG.md` gains Call 7's entry verbatim; DEC-057 and DEC-105 carry
  Call 8's amendments. No user-facing text names a spec or DEC id except inside `decisions/`.

## Failing Tests

All in **`tests/build_refuses_output_flags.rs`** except FT-15. Each refusal row is **RED on
`main`** with the measured result given (see the Design table). FT-13 and FT-14 are controls and
are **green on `main`**; they must stay green.

Shared fixture: the three-target project (ext / recipefmt / neither) built from a PNG written
with the `image` crate (as `tests/build_watch.rs:55` does), sources under `src/`, outputs under
`out/`. Shared helpers: `snapshot(root) -> BTreeMap<PathBuf, [u8; 32]>` over every file under
the root, and `assert_refused(out, expected_stderr)` asserting `code == Some(2)` and
`stderr == expected` **exactly** (on `main` stderr holds the progress lines, so the equality
fails even where the code is 2).

- **FT-1** `"build refuses --format and writes nothing"`: asserts exit 2, stderr ==
  the `--format` line + `\n`, snapshot unchanged. *`main`: exit 0, 7 files written.*
- **FT-2** `"build refuses -o in every spelling"`: loops `-o x.png`, `--output x.png`,
  `--output=x.png`, `-ox.png`; each: exit 2, the `-o/--output` line, snapshot unchanged,
  `x.png` absent. *`main`: exit 0, 7 files, no `x.png`.*
- **FT-3** `"build refuses --out-dir"`: exit 2, the `--out-dir` line, snapshot unchanged,
  `other/` absent. *`main`: exit 0, 7 files.*
- **FT-4** `"build refuses --name-template"`: exit 2, the `--name-template` line, snapshot
  unchanged. *`main`: exit 0, 7 files.*
- **FT-5** `"build refuses a flag given its default, its own or an empty value"`: loops
  `--name-template {stem}.{ext}`, `-o ''`, `--out-dir .`, `--format jpeg`, `--format ''`,
  `--format bogus`; each: exit 2, that flag's line, snapshot unchanged. *`main`: exit 0 on every
  one (`--format ''` and `--out-dir .` driven too).*
- **FT-6** `"build refuses a flag given before the verb"`: `--format png build`,
  `--out-dir other build`; exit 2, that flag's line. *`main`: exit 0.*
- **FT-7** `"build names every refused flag in a fixed order"`: the four-flag run (flags given
  in the order `--format`, `--name-template`, `-o`, `--out-dir`) → stderr == the four lines in
  Call 3's order; the two-flag run `--format png --out-dir other` → exactly the `--out-dir` then
  `--format` lines; snapshot unchanged. *`main`: exit 0, 7 files.*
- **FT-8** `"the refusal comes before a missing manifest"`: no `crustyimg.build.toml`;
  `build --format png` → exit 2, the `--format` line. *`main`: exit 3, "could not read build
  manifest".*
- **FT-9** `"the refusal comes before a malformed manifest"`: manifest `version = 1\nbogus = 3\n`;
  `build --out-dir other` → exit 2, stderr == the `--out-dir` line (and asserts it does not
  contain `could not parse build manifest`). *`main`: exit 2 with the parse error, so RED on the
  stderr equality.*
- **FT-10** `"the refusal comes before a stale lockfile check"`: `build` (assert exit 0), edit
  the v1 recipe's width, snapshot, then `build --check --format png` and `build --frozen -o x.png`;
  each: exit 2, that flag's line, snapshot equal to the pre-run one. Control inside the test: a
  plain `build --check` on the same state exits 7 (proves the lockfile really is stale, so the 2
  is the guard's). *`main`: exit 7.*
- **FT-11** `"build --watch refuses and never starts watching"`: for `--format png` and
  `--name-template {stem}.png`: spawn `build --watch <flag>` with stdout/stderr to files outside
  the project, poll `child.try_wait()` every 50 ms up to a **20 s deadline**; on timeout, kill
  the child (a `ChildGuard` like `tests/build_watch.rs:83` also kills on panic) and fail with
  "build --watch with a refused flag is still running: it started watching". On exit: code 2,
  stderr == that flag's line, snapshot unchanged. **It cannot hang:** the deadline is the test's,
  not the child's, so with the guard missing it fails in 20 s. *`main`: still running after 4 s,
  so this fails at the deadline.*
- **FT-12** `"the refusal comes before the --watch --check conflict"`: `build --watch --check
  -o x.png` run to completion **under the same 20 s deadline harness** → exit 2, stderr == the
  `-o/--output` line (asserts it does not contain `--watch cannot be combined`). *`main`: exit 2
  with the `--watch cannot be combined` message, so RED on the stderr equality.*
- **FT-13** `"build still honours its own flags"` (control, green on `main`): fresh project,
  snapshot, `build -q 50 -j 2 -Q --no-cache -y` → exit 0, `out/ext/a.webp`,
  `out/recipefmt/a.gif`, `out/neither/a.png` exist and the snapshot **changed** (proves the
  snapshot helper sees writes, so the "unchanged" assertions elsewhere are not vacuous); then
  `build --strict` → exit 0; then `build --check` → exit 0.
- **FT-14** `"apply still honours the four output flags"` (control, green on `main`): a **JPEG**
  source `src/a.jpg` written with the `image` crate (so no expected format equals the source's)
  and `v1.toml`; each `apply --recipe v1.toml src/a.jpg …` exits 0 and: `-o x.png` → `x.png`
  starts with the PNG magic; `-o f.out --format png` → `f.out` starts with the PNG magic;
  `--out-dir d` → `d/a.jpg` exists; `--out-dir t --name-template {stem}_w.png` → `t/a_w.png`
  starts with the PNG magic.
- **FT-15** (`src/cli/mod.rs`, `#[cfg(test)]`) `"refused_build_flags lists only the given
  flags, in order"`: `Cli::try_parse_from` for `["crustyimg","build"]` → empty, and the four
  fields are `None`; `build --format png` → one entry; all four in reverse order → four entries
  in Call 3's order; `build -q 50 --no-cache` → empty. *`main`: does not compile (the function
  does not exist).*

### Mutation checks (for verify to run)

Each mutation is one edit to the guard or its position; rebuild, then run
`cargo test --test build_refuses_output_flags` and FT-15. Revert, rebuild before the next one
(a reverted source does not rebuild the binary on its own).

| Mutation | Must go red |
|---|---|
| drop `out_dir` from `refused_build_flags` | FT-3, FT-5 (`--out-dir .`), FT-6 (`--out-dir other build`), FT-7, FT-9, FT-15 |
| drop `output` | FT-2, FT-5 (`-o ''`), FT-7, FT-10 (`--frozen -o`), FT-12, FT-15 |
| drop `name_template` | FT-4, FT-5 (`{stem}.{ext}`), FT-7, FT-11 (second flag), FT-15 |
| drop `format` | FT-1, FT-5, FT-6, FT-7, FT-8, FT-10, FT-11, FT-15 |
| move the guard into `run_build` only (not the watch path) | FT-11, FT-12 |
| move the guard after the manifest is loaded | FT-8 (exit 3), FT-9 (parse message) and FT-12 (the `--watch`/`--check` conflict check runs before the manifest loads) |
| print only the first refused flag | FT-7 |
| make the guard also refuse `quality` | FT-13, FT-15 |
| `hasher.update(b"x");` in `target_recipe_hash`'s `Preserve` arm | `compare-builds.sh` (AC-16), exit 1 `LOCKFILE DIFFERS` |

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-057`: the manifest owns a build's outputs; this spec amends its Neutral bullet from
  "ignored" to "refused", and it already scopes `src/cli/mod.rs`.
- `DEC-105`: the name extension pins the format on every verb; rule 2 gets a `build` clause.
- `DEC-087`, `DEC-099`: the two manifest homes of a build's format (the name extension, the
  recipe's `format`), which the `--format` message points to.
- `DEC-058`: every cache key folds in the crate version, which is why there is no golden
  lockfile (Call 6).
- `DEC-101`: the format plan is in the key; untouched, proven by AC-16.
- `DEC-015`: exit 2 is a usage error (`CliError::Usage`, `src/cli/mod.rs:821`).

### Constraints that apply

- `ergonomic-defaults`: a flag that does nothing must say so, not exit 0.
- `every-public-fn-tested`: nothing new is public; FT-15 covers the private helper anyway.
- `clippy-fmt-clean`: `just check`, plus the lean (`--no-default-features`) build.

### Prior related work

- `SPEC-071` fix 4: the non-`build` `--watch` guard at `src/cli/mod.rs:867`, the same shape.
- `SPEC-063` (manifest), `SPEC-064` (cache), `SPEC-066` (lockfile/`--check`), `SPEC-067`
  (`--watch`): the code paths the guard sits in front of.
- `SPEC-134`: DEC-105, the name extension as format pin.
- `SPEC-137`: designed in parallel; touches `CHANGELOG.md`, `README.md`,
  `docs/cli-reference.md`. **The two builds run one after the other; whichever builds second
  rebases over the first's `[Unreleased]` CHANGELOG entry and keeps both.**

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- **Honouring `--format` on `build`.** Ruled out by the maintainer (refuse).
- **A per-target `quality` key in the manifest, and `-q` on `build`.** `-q` is honoured today;
  whether it should be is not this bug.
- **`--keep-gps` and `-v` being read by no verb, and `--strict` without `--check`.** A different
  bug, filed in STAGE-054's backlog.
- **Hiding globals from `build --help`, or editing their help text.** clap shows
  `global = true` flags on every subcommand; the refusal message replaces the misleading help
  line. Restructuring `GlobalArgs` is out.
- **Naming the exact spelling typed (`-o` vs `--output`).** Needs an argv re-parser (Call 2).
- **A committed golden lockfile.** Replaced by Call 6's comparison.
- **Sweeping other verbs' global-flag handling.**
- **SVG font fallback.** That is SPEC-137.

## Notes for the Implementer

- **The guard**, in `dispatch` after the non-`build` `--watch` guard and before the `match`:
  `if matches!(cli.command, Commands::Build { .. }) { let refused = refused_build_flags(&cli.global); if !refused.is_empty() { return Err(CliError::Usage(refused.join("\nerror: "))); } }`.
  Comment it behaviour-first ("`build` takes its outputs from the manifest, so a global flag
  that would set an output's path, name or format is refused rather than ignored"), no spec ids.
- **The `--format` message contains `{stem}`.** If built with `format!`, escape the braces
  (`{{stem}}`); a `&'static str` literal needs nothing. FT-1's exact-equality catches a slip.
- **FT-14's formats are read by magic bytes** (`\x89PNG`), not by extension, and the `-o f.out
  --format png` row is the one that proves `--format` itself works on `apply` (`.out` pins
  nothing). Use a JPEG source so no expected format equals the source's.
- **FT-11/FT-12 must not use `.output()`** (it waits forever if the child watches). Spawn and
  poll `try_wait` under the deadline; the existing `watch_rejects_verify_modes`
  (`tests/build_watch.rs:315`) uses `.output()` and would hang if its guard were missing; do not
  copy that part.
- **`build`'s progress lines go to stderr** (driven): that is why exact stderr equality is a
  sharper RED than `contains`, and why FT-13's `-Q` row is the quiet one.
- **Docs text.** `docs/cli-reference.md`, in `### build [FILE]` after the "overwrites its own
  declared outputs" paragraph: "`build` takes no `-o`/`--output`, `--out-dir`,
  `--name-template` or `--format`: each target's `out` and `name` say where it writes, and the
  name's extension or the recipe's `format` sets the output format. Passing one of those flags
  is a usage error (exit `2`) and nothing is built." `docs/api-contract.md:626`: "(`build` has
  no `--format`/`-o` of its own, and refuses them with exit 2 — DEC-098 — …)".
- **Byte-identity run.** Build the merge-base in a separate worktree and `CARGO_TARGET_DIR`,
  the head in another, both `--release`; run Call 6's script; paste both outputs. Delete the
  target dirs after.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout
  to a file and read `$?`, not a pipe's.
- **Push a WIP commit once it compiles**, before the full matrix.

## Size

Frame said [S], ~8 rows. Designed: **14 integration tests + 1 unit test**. The growth is the
default/empty-value row (FT-5), the before-the-verb row (FT-6), the several-flags row (FT-7),
splitting the error-order rows by error (FT-8 to FT-10, FT-12), and FT-15. Most are short loops
over one helper; the code is still one guard (~15 lines) and one private function. **Kept at
[S]; no cut proposed.** If the build wants a cut, FT-6 is the cheapest to drop (flag position is
clap's global propagation, not this guard), but it is a cheap guard against a future
`global = false`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-138-build-refuses-output-flags`
- **PR (if applicable):** draft, see the PR description
- **All acceptance criteria met?** yes (AC-1 to AC-15 by FT-1 to FT-15; AC-16 and AC-17 below)
- **New decisions emitted:** none (DEC-057 and DEC-105 amended)
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`):
  - `src/cli/mod.rs`: the guard in `dispatch`, private `refused_build_flags`, FT-15 in `mod tests`
  - `tests/build_refuses_output_flags.rs`: new, FT-1 to FT-14
  - `docs/cli-reference.md`: four global-option rows end "Refused by `build` (exit `2`)."; paragraph in `build [FILE]`
  - `docs/api-contract.md`: one clause on line 626
  - `CHANGELOG.md`: `### Breaking` entry under `[Unreleased]`
  - `decisions/DEC-057-build-manifest-format-and-build-semantics.md`: two Neutral bullets amended
  - `decisions/DEC-105-an-output-names-extension-pins-the-format-on-every-verb.md`: rule 2 clause
- **Commit 1:** `277391d`; `git show 277391d -- src/` is one hunk (`@@ -1517,4 +1517,59 @@`),
  inside `mod tests` (opens at line 1069). No other `src/` change.
- **RED baseline on `main`** (integration tests run against unmodified `src/`; FT-15 does not
  compile, as the spec says): FT-1, 2, 3, 4, 5, 6, 7: exit 0 (and 7 files written);
  FT-8: exit 3; FT-9: exit 2 with "could not parse build manifest"; FT-10: exit 7;
  FT-11: still running at the 20 s deadline; FT-12: exit 2 with "--watch cannot be combined".
  FT-13 and FT-14 GREEN. 12 RED, 2 GREEN, each for the spec's reason.
- **Mutations** (clean rebuild each; source restored, `git status` clean for `src/`):
  - drop `out_dir`: FT-3, FT-5, FT-6, FT-7, FT-9, FT-15
  - drop `output`: FT-2, FT-5, FT-7, FT-10, FT-12, FT-15
  - drop `name_template`: FT-4, FT-5, FT-7, FT-11, FT-15
  - drop `format`: FT-1, FT-5, FT-6, FT-7, FT-8, FT-10, FT-11, FT-15
  - guard skipped on the watch path: FT-11, FT-12
  - guard moved into `run_build` after the manifest loads: FT-8, FT-9, plus FT-11 and FT-12
    (superset: `run_build` is not on the watch path)
  - print only the first refused flag: FT-7
  - also refuse `quality`: FT-13, FT-15
  - `hasher.update(b"x")` in the `Preserve` arm: `compare-builds.sh` exit 1, `LOCKFILE DIFFERS`,
    one `key =` line; restored build prints `IDENTICAL`.
- **Byte identity:** `git diff --stat origin/main...HEAD -- src/build/ src/cli/build.rs` is empty.
  `compare-builds.sh` (merge-base release binary vs head): `IDENTICAL: lockfile and 3 outputs`.
- **Matrix** (fresh `CARGO_TARGET_DIR` each, sequential; `fmt --check` clean): default 1115 pass,
  `--no-default-features` 1053 pass, `--features webp-lossy` 1120 pass; clippy `-D warnings`
  clean on all three. `decisions-audit.sh`: warnings only, all existing scope overlaps.
- **README.md:** `grep -n 'build' README.md` shows no flag line that names these four; no change.
- **Deviations from spec:** none.
- **Follow-up work identified:** none.
- **Corrected at ship** (verify found these): the mutation-table rows for "after the manifest is loaded" (add FT-12) and "also refuse `quality`" (add FT-15), and the hunk header (`+1517,59`, not `+1517,55`).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing; the exact messages and the mutation table made it mechanical.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — Nothing material. No design call turned out wrong.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Make the design's mutation table name *where* each mutation is applied, not just what it
   does: "move the guard after the manifest loads" was worded for both paths, the build applied it
   in `run_build` only (off the watch path) and got FT-11 and FT-12 red as well, and verify applying
   it on both paths got FT-8, 9 and 12 red with FT-11 green. Both were valid reads of one row, and
   only verify's run caught that FT-12 depends on the conflict check preceding the manifest load.
   Also, ask at design whether each watch-path test still reaches the watcher in the lean leg: it
   does not (`watch` is not compiled in), so FT-11 passes there through its stderr assertion alone,
   and only verify could say so.

2. **Does any template, constraint, or decision need updating?**
   — Proposed, not applied: add one line to the design template's mutation-table guidance, "name
   the code location (path) where each mutation is applied, and list every test expected red,
   including tests on the other path". No constraint or DEC change; DEC-057 and DEC-105 were
   already amended in the build.

3. **Is there a follow-up spec I should write now before I forget?**
   — Yes, one small backlog entry in STAGE-054 grouping verify's cleanups 1 to 3 (close mutation
   M-d in FT-13, make `snapshot()` see empty directories, re-wrap two long lines) with the
   `build --help` listing of the four refused flags. Not one entry per cleanup: all are tests or
   docs and none changes behaviour.
