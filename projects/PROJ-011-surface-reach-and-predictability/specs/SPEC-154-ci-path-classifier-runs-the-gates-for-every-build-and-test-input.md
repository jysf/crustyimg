---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-154
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
  implementer: claude-opus-4-7     # usually same Claude, different session
  created_at: 2026-11-22

references:
  decisions: []
  constraints: []
  related_specs: [SPEC-155, SPEC-156, SPEC-157, SPEC-158]
  reviews: [docs/reviews/2026-11-10-external-code-review.md]
  files: [scripts/ci-code-paths.sh, .github/workflows/ci.yml, .github/workflows/pages.yml]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling STAGE-063's honest-output fixes: every PR that changes a build or test input runs the gates that judge it"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-154: CI path classifier runs the gates for every build and test input

## Context

STAGE-063 (PROJ-011) holds five specs from the external code review of 2026-11-10
(`docs/reviews/2026-11-10-external-code-review.md`, the P2 row "The CI path filter classifies real
build inputs as docs-only"). This one lands **first and alone**: SPEC-155 to SPEC-158 change
`src/sink/`, `src/metadata/` and `src/cli/build.rs` and must run through the full gates.

The `changes` job in `.github/workflows/ci.yml` pipes the diff's file names through
`scripts/ci-code-paths.sh`. The script holds one deny-style regex, `CODE_PATHS`: a path that
matches it is code; **anything unlisted is docs-only**. When every path is docs-only, the three
required `build / test / clippy / fmt` legs still report success but skip Build, Test, Clippy and
Rustfmt, and seven advisory jobs skip outright. A skipped required check reads as passed, so the
PR merges green. Files that are compiled into the binary or read by tests fall into the unlisted
set today (20 tracked paths, below). `pages.yml` has the same gap in its own `paths:` lists.

## Goal

A PR or push to `main` whose diff touches any file that can change a build, test, lint or
dependency-audit result runs the full gates, and a diff that touches only docs and process files
still skips them. Each rule the classifier depends on has a self-test row that goes red when the
rule is removed or widened.

## Frame

*Frame cycle, 2026-11-22, Opus. Probes ran in the session scratchpad and are not committed.*

### Verdict: GO, one spec, [S]

The premise holds, and the review's four paths are only part of it: 20 tracked paths in 9 classes
are real inputs the classifier calls docs-only. One of the four (`deny.toml`) is wrong in letter
but harmless today. No split.

### Premise check (on `main` at `ff3c3fc`)

```
$ for p in recipes/web.toml assets/fonts/Go-Regular.ttf clippy.toml deny.toml; do
    printf '%s -> ' $p; printf '%s\n' $p | scripts/ci-code-paths.sh; done
recipes/web.toml -> false
assets/fonts/Go-Regular.ttf -> false
clippy.toml -> false
deny.toml -> false
```

What a `recipes/web.toml`-only PR does, end to end:

- `changes` sets `code=false` (ci.yml's "docs/specs only: heavy jobs will skip").
- `check` (3 OS legs) runs but every step after the toolchain install has
  `if: needs.changes.outputs.code == 'true'`: no `cargo build`, `cargo test`, `clippy` or `fmt`.
- Skipped outright (`if:` at job level): `ddphotos-parity`, `wasm-roundtrip`, `avif`,
  `webp-lossy`, `heic` (2 legs), `lean`, `msrv`.
- Still run (not gated on `changes`): `supply-chain` (cargo-deny), `cost-data`, `metadata`.
- `pages.yml` does not trigger (its `paths:` lack `recipes/`).
- Branch protection (read via `gh api repos/{owner}/{repo}/branches/main/protection/required_status_checks`):
  the required contexts are exactly the three `build / test / clippy / fmt (<os>)` legs, `strict: true`.
  Those legs report success with their steps skipped, so **the PR is mergeable green with no build
  or test run**. The rulesets endpoint returned `[]`. A push to `main` uses the same classifier
  (diff `before..sha`), so a direct push is skipped the same way.
- `tests/recipe_strict_params.rs` and `tests/build.rs` `include_str!`/`include_bytes!` the file,
  and `src/recipe/bundled.rs` embeds it in the binary. A typo in it ships.

The review's `deny.toml` claim is half right: `supply-chain` runs cargo-deny on every PR whatever
`changes` says, so a `deny.toml` edit is already checked (advisory, not required). It is still a
dependency-audit input and belongs on the code side.

### The full path table

Commands (all on `main` at `ff3c3fc`, `/usr/bin/grep` and `/usr/bin/git` to avoid rtk truncation):

```
# compile-time includes
/usr/bin/grep -rnoE 'include_(str|bytes)!\([^)]*\)' src tests benches examples fuzz build.rs
/usr/bin/grep -rlE 'CARGO_MANIFEST_DIR' src tests benches examples fuzz
/usr/bin/grep -rnE 'concat!\(' src tests benches
# runtime reads by tests (string literals naming non-code roots), then each CARGO_MANIFEST_DIR file by hand
cd tests; /usr/bin/grep -nE '"(\.\./)?(docs|skills|recipes|README|CHANGELOG|demo|integrations|examples|assets|scripts|npm|guidance|decisions|projects|fuzz|bench|\.github|Cargo|justfile|AGENTS|CONTRIBUTING|RELEASING|SECURITY|BENCHMARKS|dist-workspace|deny|clippy|\.cargo|\.claude-plugin|\.pre-commit)[^"]*"' *.rs
# top level
/usr/bin/git ls-files | awk -F/ '{print (NF>1)? $1"/" : $1}' | sort | uniq -c
```

Scope of the sweep: `src/`, `tests/` (Rust, plus the `.mjs`/`.py` harnesses), `benches/`,
`examples/`, `fuzz/`; there is no `build.rs`. Runtime reads built from `format!`/`join` with a
variable were read by hand in each of the 19 `CARGO_MANIFEST_DIR` files; all resolve under
`tests/`, `src/`, `recipes/`, `bench/` or `demo/`. `concat!` builds strings only, never paths.

**Wrong today: real inputs the classifier calls docs-only (20 paths, 9 classes)**

- `recipes/{gallery,product,web}.toml` (3): `include_str!` in `src/recipe/bundled.rs` and
  `tests/recipe_strict_params.rs`; `include_bytes!` in `tests/build.rs`; runtime reads in
  `tests/recipe_corpus_output.rs` and `tests/recipe_round_trip.rs`.
- `assets/fonts/Go-Regular.ttf` (1): `include_bytes!` in `src/text/mod.rs` and `src/image/svg.rs`;
  runtime read in `tests/watermark.rs`.
- `clippy.toml` (1): configures the clippy gate (`src/lib.rs:44` cites it).
- `deny.toml` (1): cargo-deny policy; harmless today because `supply-chain` is ungated.
- `docs/github-actions.md` (1): `include_str!` in `tests/lint.rs`. **A test input under `docs/`.**
- `docs/data-model.md` (1): read by `tests/docs_ops.rs`. **Also under `docs/`.**
- `README.md` (1) and `.pre-commit-hooks.yaml` (1): read by `tests/adoption_glue.rs`.
- `demo/index.html`, `demo/README.md`, `demo/demo.js`, `demo/worker.js` (4): read by
  `tests/demo_copy.rs` and `tests/help_has_no_internal_ids.rs` (every `demo/*.js|html`).
  `pages.yml` covers `demo/**` for the browser smoke, but `cargo test` in ci.yml is skipped.
- `bench/corpus/*` (6): `tests/audit_bench.rs` reads the directory and its `README.md`.

**Right today by design:** `src/`, `tests/`, `examples/`, `benches/`, `Cargo.toml`, `Cargo.lock`,
`.cargo/`, `.github/workflows/`, `justfile`, `scripts/`, `integrations/ddphotos/` (code); `docs/`
(except the two above), `projects/`, `decisions/`, `guidance/`, `feedback/`, `reports/` (docs).

**Right today by accident** (docs-only verdict is harmless only because no ci.yml job reads them
now; nothing in the classifier says so): `fuzz/` (detached workspace, no CI job builds it; the
per-PR guard is `tests/fuzz_regressions.rs`), `dist-workspace.toml` and `npm/` (release inputs;
`publish-crates.yml`/`publish-npm.yml` trigger on them), `skills/`, `.claude-plugin/`, the root
`*.md` other than `README.md`, `LICENSE-*`, `.repo-context.yaml`, `.variant`, `.gitignore`,
`.github/TEMPLATE_README.md`, `gitignore_files_maybe/`, and the unread files in `demo/` (css,
icons, manifest), `assets/fonts/` (licence, README) and `recipes/README.md`. `rust-toolchain*` and
`rustfmt.toml` do not exist; the regex names `rust-toolchain` already. `integrations/` holds only
`ddphotos/`. Anchor check: `docs/.cargo/config.toml` is `false` today because `^` pins `\.cargo/`
to the root.

### Design: recommend the docs-only allow-list

Recommendation: **invert it.** List what is provably docs-only; anything else, including a new
top-level directory nobody has classified, runs the full gates. Plus a short exception list for
docs-shaped files a test reads. Probe shape (scratch, illustrative, not the build's text):

```
DOCS='^(docs/|projects/|decisions/|guidance/|feedback/|reports/|gitignore_files_maybe/|skills/|\.claude-plugin/|[^/]+\.md$|LICENSE-[A-Z]+$|\.repo-context\.yaml$|\.variant$|\.gitignore$|\.github/TEMPLATE_README\.md$)'
EXCEPT='^(README\.md|docs/github-actions\.md|docs/data-model\.md)$'   # docs-shaped test inputs
code = any path matches EXCEPT, or does not match DOCS
```

Evidence, from running both over all 1,289 tracked paths:

- The allow-list never calls a path docs-only that today's regex calls code (0 such paths).
  39 paths flip from docs-only to code: the 20 real inputs above, plus 19 harmless over-inclusions
  (`fuzz/` 5, `npm/` 2, `dist-workspace.toml` 1, unread `demo/` files 8, `assets/fonts/LICENSE-Go`
  and `README.md` 2, `recipes/README.md` 1).
- **It fails safe under mutation.** Deleting an alternative from the allow-list makes CI run
  *more*; deleting one from the deny-list makes it run *less*, silently. The dangerous edits on
  the allow-list are widenings (dropping `^`, `[^/]+\.md$` → `.*\.md$`, adding a code directory),
  and the self-test rows below catch each one.
- Cost, replayed over the last 600 commits on `main` (back to 2026-07-23): 514 were docs-only
  under today's regex; **12 (2.3%)** flip to a full run. 10 of the 12 touched `README.md`,
  `docs/github-actions.md`, `docs/data-model.md` or `.pre-commit-hooks.yaml`, which a fixed
  deny-list must also classify as code. Only 2 are allow-list overhead (`recipes/README.md`,
  `demo/demo.css`), about 15 runner-minutes each. The real cost is the `README.md` exception:
  README-only edits were 6 of the 12.
- The weak point is the same for both designs: a new test that reads a docs path (as
  `tests/lint.rs` reads `docs/github-actions.md`) makes it an input silently. Design should add a
  **derived self-test row**: every `include_str!`/`include_bytes!` target in `src/` and `tests/`
  must classify `true` (mechanical; the probe resolved all of them). Runtime reads built with
  `format!`/`join` cannot be derived mechanically and stay hand-listed.

The deny-list, fixed, would need `recipes/`, `assets/`, `bench/`, `demo/`, `clippy\.toml`,
`deny\.toml`, `\.pre-commit-hooks\.yaml$`, `README\.md$`, `docs/github-actions\.md$`,
`docs/data-model\.md$`: ten alternatives that each fail unsafe when removed, and still no
protection for the next new top-level directory.

### Self-test and mutation rows

Keep the existing seven rows (all keep their verdicts under the allow-list; `docs/.cargo/config.toml`
stays `false` because `docs/` matches first). Rows driven in the scratch probe, all passing:

- `true`: `recipes/web.toml`, `assets/fonts/Go-Regular.ttf`, `clippy.toml`, `deny.toml`,
  `README.md`, `docs/github-actions.md`, `docs/data-model.md`, `.pre-commit-hooks.yaml`,
  `demo/index.html`, `bench/corpus/README.md`, `fuzz/Cargo.toml`, `newdir/x.txt` (an unknown
  top-level directory), `src/docs/mod.rs` and `tests/fixtures/x.md` (anchor and root-only `*.md`
  guards).
- `false` (negative controls): `docs/x.md`, `docs/.cargo/config.toml`, `docs/recipes/web.toml`,
  one path under each of `projects/`, `decisions/`, `guidance/`, `feedback/`, `reports/`,
  `skills/`, `.claude-plugin/`, `gitignore_files_maybe/`, plus `CHANGELOG.md`, `LICENSE-MIT`,
  `.gitignore`, `.github/TEMPLATE_README.md`, `.repo-context.yaml`, `.variant`.

Mutation rows, each driven in the probe; every one turned the self-test red:

- Delete each of the 15 `DOCS` alternatives in turn: 1 FAIL row each (3 for `docs/`).
- Delete each of the 3 `EXCEPT` alternatives in turn: 1 FAIL row each.
- Drop the leading `^`: FAIL on `bench/corpus/README.md`, `src/docs/mod.rs`, `tests/fixtures/x.md`.
- Widen `[^/]+\.md$` to `.*\.md$`: FAIL on `bench/corpus/README.md`, `tests/fixtures/x.md`.

Design should fix the mutation list in the spec and have build record each mutation's red output
(AGENTS §15). A mutation driver can live in the scratch dir; it need not ship.

### Pages and the other workflows

- `pages.yml` builds the wasm demo through `just demo-build`. `src/recipe/` and `src/text/` are
  compiled for `wasm32` (no `cfg` gate in `src/lib.rs`), so the bundle embeds `recipes/*.toml` and
  `assets/fonts/Go-Regular.ttf`. **Add `recipes/**` and `assets/**`** to both `paths:` lists.
  `.cargo/config.toml` holds only the wasm *test* runner, which pages does not use; leave it out.
- `publish-crates.yml` and `publish-npm.yml`: their `pull_request.paths` are release-plumbing
  triggers (release workflows, `dist-workspace.toml`, `npm/**`, the guard scripts). They do not
  list `src/**` either, on purpose: they check the pipeline, not the content. **Leave them.**
- `release.yml` (tags), `scheduled-audit.yml` (cron), `dco.yml` (every PR): no path filters.
- Drift: GitHub's parser has no YAML anchors, so pages' two lists stay duplicated. Recommend a
  cheap check in the classifier script, run from the ungated `metadata` job: (a) the
  `push.paths` and `pull_request.paths` lists in `pages.yml` are identical; (b) every pages path
  prefix classifies `true`. Design picks grep/awk or Ruby (preinstalled, already used by
  `validate-frontmatter.sh`).

### What only a PR can prove

Locally: the script, `--self-test`, the mutations and the replay. Only Actions can show that
`changes` emits `code=true` for a lone `recipes/web.toml` diff and that the `check` legs then run
Build/Test, and that `pages.yml` triggers on it. The build PR itself touches `scripts/` and
`.github/workflows/`, which are code under both designs, so **it cannot prove the classification**.

Smallest proof, after the build PR merges: a **draft PR touching only `recipes/web.toml`** (a
comment line), checked once by the maintainer, then closed unmerged. Expected: the `changes` log
prints `code paths touched`, the three `check` legs run `cargo test`, `pages` runs. The negative
side needs no extra PR: the next docs-only push to `main` (any frame commit) should still print
`docs/specs only`. No polling (AGENTS §16 habit 6).

### Sizing: [S], confirmed

Code: one script (regex inverted, an exception list, about 35 self-test rows, the derived
include-target row, the pages drift check), `pages.yml` (four lines), and comments in `ci.yml`.
Verification: 2 designs already compared at frame; 18 deletion mutations and 2 widening
mutations; one draft PR after merge. No Rust changes, no tests under `tests/`.

### Scope

In: the classifier and its self-test; `pages.yml`'s two lists; the drift check; the comments
that describe the classifier (the script header, the `changes` job comment in `ci.yml`).

Docs today: neither `AGENTS.md` nor `docs/development.md` mentions the classifier (grep for
`ci-code-paths`, `docs-only`, `heavy jobs` found only `ci.yml`). After: the script header states
the allow-list rule and the exception rule; the `changes` comment in `ci.yml` says "anything not
provably docs-only runs the gates". One short paragraph in `docs/development.md` on "which PRs
skip the Rust gates" is optional (maintainer question 3).

### Rough build file list

- `scripts/ci-code-paths.sh` — invert to the allow-list, exception list, rows, derived row, drift check
- `.github/workflows/pages.yml` — `recipes/**`, `assets/**` in both lists
- `.github/workflows/ci.yml` — `changes` job comment; call the drift check from `metadata` if it lands there
- `docs/development.md` — optional paragraph

### Open maintainer questions

1. Allow-list or fixed deny-list? **Recommend the allow-list**: it fails safe, it covers the next
   new directory, and it costs 2 of 600 commits over a fixed deny-list.
2. `README.md` as a code path: it is a test input (`tests/adoption_glue.rs`), so README-only PRs
   would run the full gates (6 of 600 commits). **Recommend yes**; the alternative is a green
   README PR that breaks the next code PR on `main`.
3. A `docs/development.md` paragraph? **Recommend yes, three sentences**: contributors otherwise
   cannot tell why a docs PR ran 15 minutes of CI.

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

- Gating semantics: which jobs are required, which are gated on `changes`, and the `check`
  always-run pattern stay as they are. `supply-chain`, `cost-data` and `metadata` stay ungated.
- `publish-crates.yml` and `publish-npm.yml` path lists (release-plumbing triggers, on purpose).
- Building `fuzz/` in CI (detached workspace; `tests/fuzz_regressions.rs` is the per-PR guard).
- Any Rust source or test under `tests/`; any change to branch protection.
- SPEC-155 to SPEC-158, which land after this one.

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
