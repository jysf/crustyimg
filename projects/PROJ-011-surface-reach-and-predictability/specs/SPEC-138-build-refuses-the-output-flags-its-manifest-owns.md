---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-138
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
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-138: build refuses the output flags its manifest owns

> Title follows the frame's recommendation (refuse). If the maintainer rules "honour", design
> renames it.

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

- **Honouring `--format` on `build`.** Only if the maintainer rules honour: then its own [M] spec, and
  this one still refuses the other three.
- **A per-target `quality` key in the manifest, and `-q` on `build`.** `-q` is honoured today; whether
  it should be is not this bug.
- **`--keep-gps` and `-v` being read by no verb, and `--strict` without `--check`.** These flags do
  nothing on any verb. A different bug, filed in STAGE-054's backlog.
- **Hiding globals from `build --help`.** clap shows `global = true` flags on every subcommand. The
  refusal message replaces the misleading help line; restructuring `GlobalArgs` is out.
- **Sweeping other verbs' global-flag handling.** Not driven, by the frame prompt's rule.
- **SVG font fallback.** That is SPEC-137.

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
