---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-154
  type: bug                        # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
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
  files: [scripts/ci-code-paths.sh, .github/workflows/ci.yml, .github/workflows/pages.yml, docs/development.md]

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
  sessions:
    - cycle: frame
      date: 2026-11-22
      interface: claude-code
      agent: claude-opus-5-5
      rate_anchors: "Opus $5 in / $25 out per MTok; cache_creation x1.25, cache_read x0.10"
      tokens_total: 3422377
      tokens_breakdown:
        input: 64
        cache_creation: 110091
        cache_read: 3282348
        output: 29874
      estimated_usd: 3.08
      duration_minutes: null
      note: "Measured from the session JSONL (dedup by message.id, MAX output) after the frame push, before this cost commit; 32 API messages."
    - cycle: design
      date: 2026-11-22
      interface: claude-code
      agent: claude-opus-5-5
      rate_anchors: "Opus $5 in / $25 out per MTok; cache_creation x1.25, cache_read x0.10"
      tokens_total: 6788928
      tokens_breakdown:
        input: 106
        cache_creation: 155888
        cache_read: 6575269
        output: 57665
      estimated_usd: 5.70
      duration_minutes: null
      note: "Measured from the session JSONL (dedup by message.id, MAX output) after the design push, before this cost commit; 53 API messages, no subagents."
    - cycle: build
      date: 2026-11-22
      interface: claude-code
      agent: claude-sonnet-5-5
      rate_anchors: "Sonnet $3 in / $15 out per MTok; cache_creation x1.25, cache_read x0.10"
      tokens_total: 3172394
      tokens_breakdown:
        input: 56
        cache_creation: 110416
        cache_read: 3043597
        output: 18325
      estimated_usd: 1.60
      duration_minutes: null
      note: "Re-measured at ship from the session JSONL (dedup by message.id, MAX output), including the CI-wait and final turns; 27 API messages. Was 1.37 over 23 messages at the build push."
    - cycle: verify
      date: 2026-11-22
      interface: claude-code
      agent: claude-opus-5-5
      rate_anchors: "Opus $5 in / $25 out per MTok; cache_creation x1.25, cache_read x0.10"
      tokens_total: 5698923
      tokens_breakdown:
        input: 88
        cache_creation: 137623
        cache_read: 5522468
        output: 38744
      estimated_usd: 4.59
      duration_minutes: null
      note: "Re-measured at ship from the session JSONL (dedup by message.id, MAX output); 44 API messages. Was 4.13, the figure handed to the punch list."
    - cycle: punchlist
      date: 2026-11-22
      interface: claude-code
      agent: claude-sonnet-5-5
      rate_anchors: "Sonnet $3 in / $15 out per MTok; cache_creation x1.25, cache_read x0.10"
      tokens_total: 2670057
      tokens_breakdown:
        input: 64
        cache_creation: 111593
        cache_read: 2542163
        output: 16237
      estimated_usd: 1.43
      duration_minutes: null
      note: "Sum of two sessions started from the same prompt, measured at ship from their JSONLs: 0.24 (4 messages, 572079f6) and 1.19 (27 messages, 532948e1; it found the head had moved and stopped without changes). Was 0.75 over 17 messages."
    - cycle: ship
      date: 2026-11-23
      interface: claude-code
      agent: claude-sonnet-5-5
      rate_anchors: "Sonnet $3 in / $15 out per MTok; cache_creation x1.25, cache_read x0.10"
      tokens_total: 2302759
      tokens_breakdown:
        input: 56
        cache_creation: 74084
        cache_read: 2219045
        output: 9574
      estimated_usd: 1.09
      duration_minutes: null
      note: "Measured from this session's JSONL (dedup by message.id, MAX output) before the final commit, push and docs-only CI read; those turns are not counted."
  totals:
    tokens_total: 24055438
    estimated_usd: 17.49
    session_count: 6
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

### Maintainer rulings (2026-11-22)

All three recommendations accepted:
1. **The docs-only allow-list,** not a fixed deny-list. Anything not provably docs-only runs the
   gates, with a short exception list for docs-shaped files a test reads.
2. **`README.md` is a code path.** README-only PRs run the full gates.
3. **`docs/development.md` gets a short paragraph** (about three sentences) on which PRs skip the
   Rust gates and why. It moves from optional to in scope.

The frame's extra self-test row (every `include_str!`/`include_bytes!` target in `src/` and
`tests/` classifies as code) and the pages drift check stay in scope. The post-merge draft PR
touching only `recipes/web.toml` is the maintainer's step at ship, checked once, closed unmerged.

## Design

*Design cycle, 2026-11-22, Opus. Every row below was driven against the real `scripts/ci-code-paths.sh`
on a never-pushed prototype branch (`proto/spec-154`, a worktree off `origin/main` at `b7f07af`), not
the frame's probe. The prototype is removed; its script is reproduced verbatim under Implementation
Context.*

### Design corrections to the frame

1. **Replay count holds; its breakdown does not.** Re-run with the final patterns over the last 600
   first-parent commits on `main` (`/usr/bin/git rev-list --first-parent -n 600 origin/main`,
   `ecb4fde` 2026-07-24 to `b7f07af` 2026-11-22): **517** docs-only under today's script (the frame's
   514 was on an earlier window), **12 flip** to a full run, **0 flip the other way**. The 12 and
   the path that flips each:
   `3f7fb6c` docs/data-model.md · `318bb2c` .pre-commit-hooks.yaml docs/github-actions.md README.md ·
   `3d2cf16` same three · `0e49ca0` docs/github-actions.md · `587e2e6` docs/github-actions.md README.md ·
   `ab61f52` .pre-commit-hooks.yaml README.md · `9927613` README.md recipes/README.md · `6e1b769`
   README.md · `777253e` recipes/README.md · `8e0d19c` README.md · `08b367d` demo/demo.css ·
   `eb2b5b8` README.md recipes/README.md.
   - `README.md` is in **8** of the 12 (sum of the rows above) and the only flipping path in **2**
     (`6e1b769`, `8e0d19c`). The frame's "README-only edits were 6 of the 12" does not reproduce.
   - The frame's "2 are allow-list overhead" (`777253e` recipes/README.md, `08b367d` demo/demo.css)
     is overhead only against a deny-list naming single files. The frame's own fixed deny-list names
     `recipes/` and `demo/` whole, so it flips the same 12. Against that list the allow-list costs
     **0** extra commits in this window.
2. **Pages: the reason for `recipes/**` is the smoke test, not the wasm bundle.** `src/recipe/bundled.rs`
   compiles for wasm32 (`src/lib.rs:63` `pub mod recipe;` has no `cfg`), but its only caller is
   `src/cli/common.rs:217`, and `cli` is native-only (`src/lib.rs:76-77`
   `#[cfg(not(target_arch = "wasm32"))] pub mod cli;`). What makes `pages` depend on
   `recipes/web.toml` is `tests/demo_smoke.mjs:853`
   `const webToml = readFileSync(join(repoRoot, "recipes", "web.toml"), "utf8");`, which asserts
   `demo/demo.js`'s inlined `WEB_RECIPE` is byte-identical to the file (`demo/demo.js:25-28`). The
   font claim holds: `src/text/mod.rs:26`
   `pub const DEFAULT_FONT: &[u8] = include_bytes!("../../assets/fonts/Go-Regular.ttf");` is used by
   `src/operation/mod.rs:1705` (core, compiled and reached on wasm32). The fix is unchanged.
3. **The derived row needs two more guards the frame did not name**, both driven: an include-scan
   floor (with the scan disabled, no path row goes red, because the hand rows already cover every
   docs-shaped include target), and a canonical-path check (a resolver that does not collapse `..`
   leaves every target under `src/` or `tests/`, which always classify as code, so the row would
   pass vacuously; HANDOFF trap 10).
4. **Empty input changes verdict.** Today an empty list prints `false` (nothing matches). The
   allow-list prints `true` for an empty list: nothing was proved docs-only. Blank lines are ignored
   so `echo "$files"` of an empty diff (one blank line) is an empty list. This only fires on a diff
   with no files; it fails safe.
5. Everything else in the frame was confirmed as stated: no `build.rs`; every include is a literal
   `include_(str|bytes)!("…")` (no `concat!`/`env!` form); the two runtime docs reads; `supply-chain`
   ungated; `pages.yml` holds the only `paths:` lists that need the addition.

### What was driven (evidence)

- **Premise on the real script (`main` at `b7f07af`).** `printf '%s\n' <path> | scripts/ci-code-paths.sh`
  printed `false` for each of `recipes/web.toml recipes/gallery.toml recipes/product.toml
  assets/fonts/Go-Regular.ttf clippy.toml deny.toml docs/github-actions.md docs/data-model.md README.md
  .pre-commit-hooks.yaml demo/index.html demo/README.md demo/demo.js demo/worker.js
  bench/corpus/README.md` (15 paths); positive control `src/lib.rs` printed `true`.
- **Test-input readers (the exception list holds only really-read paths).**
  - `README.md`: `tests/adoption_glue.rs:95` `let r = read("README.md");` (`read` joins the manifest
    dir, `tests/adoption_glue.rs:66`).
  - `docs/github-actions.md`: `tests/lint.rs:1445`
    `let full = include_str!("../docs/github-actions.md").replace("\r\n", "\n");`
  - `docs/data-model.md`: `tests/docs_ops.rs:68` and `:143`
    `std::fs::read_to_string(Path::new(ROOT).join("docs/data-model.md"))`
  - Newly-code paths outside `DOCS` (no exception needed, listed for the record):
    `.pre-commit-hooks.yaml` `tests/adoption_glue.rs:72` `let y = read(".pre-commit-hooks.yaml");`;
    `demo/README.md` `tests/demo_copy.rs:66` `let readme = read("demo/README.md");`;
    `bench/corpus/README.md` `tests/audit_bench.rs:570` `let readme = corpus.join("README.md");`;
    `recipes/*.toml` and `assets/fonts/Go-Regular.ttf` (the include list below); `clippy.toml`,
    `deny.toml`, `fuzz/` are tool inputs with no `src/`/`tests/` reader, code because not in `DOCS`.
- **Include forms.** `ls build.rs` → `No such file or directory`.
  `/usr/bin/grep -rnE 'include_(str|bytes)!' src tests benches examples` → 79 lines. Non-literal forms:
  `/usr/bin/grep -rnE 'include_(str|bytes)!\((concat|env)' src tests benches examples` → rc 1 (none);
  multi-line `include_(str|bytes)!\(\s*$` → rc 1 (none). Every other hit not matching
  `include_(str|bytes)!\("[^"]+"\)` is a comment or a string literal:
  `src/recipe/bundled.rs:6`, `:37`, `src/cli/build.rs:1566`, `src/text/mod.rs:14`,
  `tests/wasm_roundtrip.rs:39` (comments), `tests/help_has_no_internal_ids.rs:604`
  `const MACRO: &str = "include_str!(\"";` and `:630` (string literals, the scan's regex cannot match
  them: `(` is followed by `\`), and `tests/help_has_no_internal_ids.rs:601` (a `///` doc comment
  holding `include_str!("…")`, skipped by the comment rule; mutant C below shows it would otherwise
  resolve to `tests/…`). All literal paths are relative to the including file; the script resolves
  `dirname(file)/literal` and collapses `.`/`..` in awk. Resolved: **35 distinct targets** (`sort -u`
  of the resolved list; by root: `tests/` 28, `recipes/` 3, `assets/` 1, `docs/` 1, `Cargo.toml` 1,
  `justfile` 1; 28+3+1+1+1+1 = 35). The 7 outside `tests/`: `assets/fonts/Go-Regular.ttf`,
  `Cargo.toml`, `docs/github-actions.md`, `justfile`, `recipes/gallery.toml`, `recipes/product.toml`,
  `recipes/web.toml`. Forms the scan cannot resolve (`include_*!(concat!(…))`, `include_*!(env!(…))`)
  are **refused**: a self-test check fails on any, naming the site.
- **Runtime reads under a docs-only root.**
  ```
  cd tests; /usr/bin/grep -rnE '"(\.\./)?(docs|README|CHANGELOG|AGENTS|CONTRIBUTING|RELEASING|SECURITY|BENCHMARKS|projects|decisions|guidance|feedback|reports|skills|\.claude-plugin|gitignore_files_maybe|LICENSE|\.repo-context|\.variant|\.gitignore|\.github/TEMPLATE)[^"]*"' --include='*.rs' --include='*.mjs' --include='*.py' . | /usr/bin/grep -v '^\./fixtures'
  /usr/bin/grep -rnE '(read_to_string|fs::read|Path::new|join)\("(\.\./)*(docs|README|CHANGELOG|projects|decisions|guidance|skills)' src
  /usr/bin/grep -rnE 'join\("(docs|skills|projects|decisions|guidance|feedback|reports|\.claude-plugin)' src tests benches examples
  /usr/bin/grep -rnE '"[A-Za-z0-9_./-]*\.md"' tests src benches examples --include='*.rs' --include='*.mjs' --include='*.py' | /usr/bin/grep -vE 'fixtures/|write\(&dir'
  ```
  Hits under a `DOCS` root: `tests/docs_ops.rs:68`/`:143` (→ EXCEPT), `tests/adoption_glue.rs:95`
  (→ EXCEPT), `tests/lint.rs:1445` (→ EXCEPT, and the derived row). Shown not to be exceptions:
  `tests/hostile_inputs.rs:121` (`if name == "README.md"` skips a fixture-dir file name),
  `tests/lint.rs:127` (`write(&dir, "README.md", …)` writes a temp file), `tests/cli.rs:5880`
  (`run(Some("docs"), "docs")`, a CLI argument), `tests/audit_bench.rs:570` and
  `tests/demo_copy.rs:66` (under `bench/`, `demo/`: code already). `src/`: no hit. Workflows:
  `/usr/bin/grep -nE '(docs|projects|decisions|guidance|skills|feedback|reports)/|README|CHANGELOG' .github/workflows/*.yml`
  → only the `docs/specs only` echo in `ci.yml`. The scripts that read `projects/`/`decisions/`
  (`cost-audit.sh`, `validate-frontmatter.sh`) run in the ungated `cost-data` and `metadata` jobs,
  so they run on a docs-only PR anyway.
- **Where the self-test runs.** `.github/workflows/ci.yml:56` `scripts/ci-code-paths.sh --self-test`,
  a plain command in the `changes` job's `Classify changed paths` step under `set -euo pipefail`
  (`:45`). `changes` (`:31-35`) has no `needs:` and no `if:`, and `on:` is `push: branches: [main]`
  plus every `pull_request` (`:9-11`), so it runs on every PR and every push to `main`, docs-only
  ones included. It checks out with `fetch-depth: 0` (`:38-40`), so the tree the derived checks read
  is there. **After the change it stays in `changes`, the one call.** The new derived and pages checks
  are inside `--self-test`, so they run there too. No new job, no change to `metadata`.
- **cargo-deny runs on every PR.** `.github/workflows/ci.yml:219-229`:
  `supply-chain:` / `name: supply-chain policy (cargo-deny)` / `runs-on: ubuntu-latest`, no `needs:`
  and no `if:`, step `uses: EmbarkStudios/cargo-deny-action@v2` with
  `command: check advisories bans sources licenses`.
- **Pages' lists are the only ones that need it.** `/usr/bin/grep -n -A12 'paths' .github/workflows/*.yml`:
  `pages.yml:27` and `:39` (the two lists), `publish-crates.yml:15` and `publish-npm.yml:15`
  (release-plumbing triggers, out of scope), nothing else (`ci.yml`, `dco.yml`, `release.yml`,
  `scheduled-audit.yml` have no `paths:`).
- **CHANGELOG precedent.** `/usr/bin/grep -n -i 'ci\b\|workflow' CHANGELOG.md` lists only user-facing
  CI features (the Actions, `lint` exit codes, the scheduled audit). The commit that introduced the
  docs-only skip (`7827d80`, #165) and SPEC-146's `517acea` (#245) changed no `CHANGELOG.md`
  (`git show --stat` → 0 hits each). **No CHANGELOG line.**

## Inputs

- **Files to read:** `scripts/ci-code-paths.sh` (rewritten), `.github/workflows/ci.yml:21-75` (the
  `changes` job), `.github/workflows/pages.yml:22-48` (the two lists), `docs/development.md:44-50`.
- **External APIs:** none.
- **Related code paths:** the include sites and runtime readers quoted under Design.

## Outputs

- **Files created:** none.
- **Files modified:**
  - `scripts/ci-code-paths.sh` — `CODE_PATHS` deny-list replaced by `DOCS` + `EXCEPT`; `classify`
    rewritten; `include_targets`, `unresolvable_includes`, `pages_paths` added; self-test rows,
    `check` helper, derived include checks and pages checks added; header rewritten.
  - `.github/workflows/pages.yml` — `"recipes/**"` and `"assets/**"` in both `paths:` lists, with a
    two-line comment on the push list.
  - `.github/workflows/ci.yml` — the `changes` job comment (`:22-26`) and the step comment (`:54-55`)
    rewritten. No control-flow change; both echo lines unchanged.
  - `docs/development.md` — a new `## Which PRs skip the Rust gates` section between `## Reports`
    and `## Key discipline in this variant`.
- **New exports:** none. **Database changes:** none.

## Acceptance Criteria

Each AC names its checks (FT ids below). Every check names what turns it red (the mutation table).

- [ ] **AC-1 Docs-only paths still skip.** Each of the 15 `DOCS` alternatives has one row that
  classifies `false`, and a list of only such paths (with a blank line in it) classifies `false`.
  — FT-D1 to FT-D17, FT-L2; mutations M1 to M15.
- [ ] **AC-2 Every build and test input runs the gates.** `recipes/web.toml`,
  `assets/fonts/Go-Regular.ttf`, `clippy.toml`, `deny.toml`, `.pre-commit-hooks.yaml`,
  `demo/index.html`, `bench/corpus/README.md`, `fuzz/Cargo.toml` classify `true`; the classic code
  roots (`integrations/ddphotos/…`, `src/`, `.cargo/`, `.github/workflows/`) still do. — FT-C1 to
  FT-C12; baseline (12 of them red on `main`'s classifier).
- [ ] **AC-3 The three docs-shaped test inputs are exceptions.** `README.md`, `docs/github-actions.md`,
  `docs/data-model.md` classify `true`, each from `EXCEPT`. — FT-E1, FT-E2, FT-E3; mutations M16,
  M17, M18.
- [ ] **AC-4 An unclassified path runs the gates.** A new top-level directory (`newdir/x.txt`)
  classifies `true`. — FT-U1.
- [ ] **AC-5 `DOCS` is anchored and root-only for Markdown.** `src/docs/mod.rs` and
  `tests/fixtures/x.md` classify `true`; `docs/.cargo/config.toml` and `docs/recipes/web.toml`
  classify `false`. — FT-U2, FT-U3, FT-D2, FT-D3; mutations M19 (drop `^`), M20 (`.*\.md$`), M21
  (`src/` added).
- [ ] **AC-6 List semantics.** An empty list prints `true`; a docs-only list with a blank line prints
  `false`; a docs path plus one code path prints `true`. — FT-L1, FT-L2, FT-L3.
- [ ] **AC-7 Every included file classifies as code (derived).** The self-test scans every literal
  `include_str!`/`include_bytes!` in `src/`, `tests/`, `benches/`, `examples/`, `build.rs`
  (skipping comment lines), resolves each against the including file's folder, and fails a row when
  a target (a) is not a canonical repository path, (b) does not exist, or (c) classifies `false`.
  — FT-I2 (35 rows today); mutations M17, M23, M24, M25, M26, M27.
- [ ] **AC-8 The include scan cannot silently find nothing.** The scan must find at least
  `INCLUDE_FLOOR=35` files (the design-time count). — FT-I1; mutation M22.
- [ ] **AC-9 An include whose path is built with `concat!` or `env!` is refused.** Any
  `include_(str|bytes)!(concat!` or `include_(str|bytes)!(env!` under the scanned roots fails the
  self-test, naming `file:line`. Other forms are not scanned: a raw string, a space before the
  parenthesis, a multi-line `include_str!(`, and `include!`. — FT-I3; mutation M28.
- [ ] **AC-10 Pages runs for the demo's inputs.** `pages.yml`'s push list names `recipes/**` and
  `assets/**`. — FT-P2, FT-P3; mutation M31.
- [ ] **AC-11 Pages' two lists cannot drift.** `push.paths` and `pull_request.paths` are identical and
  not empty. — FT-P1; mutation M29.
- [ ] **AC-12 Every pages path is a code path.** Each pages entry, with `**`/`*` replaced by `x`,
  classifies `true`. — FT-P4 (10 rows); mutation M30.
- [ ] **AC-13 The self-test exits non-zero on any wrong verdict, and runs on every PR.** The final line
  is `ci-code-paths self-test: <rows> rows, <m> mismatched, <c> crashed, <j> junk, <f> failed` and the
  exit code is 0 only when all four are 0; it runs from the `changes` job, unchanged. — the negative
  control (M32, rc=1); the green run (rc=0); the build PR's own `changes` log.
- [ ] **AC-14 Every message is the text below, verbatim.** One check per message: see "Every message
  the script prints".
- [ ] **AC-15 The comments state the rule.** The script header, the `changes` job comment and the
  step comment in `ci.yml` are the verbatim text below and carry no spec, stage or DEC id.
  — `/usr/bin/grep -nE '(SPEC|STAGE|DEC|PROJ)-[0-9]' scripts/ci-code-paths.sh` → no hit; `ci.yml`
  hunks diffed against the text below.
- [ ] **AC-16 `docs/development.md` says which PRs skip the gates.** The section below, verbatim, at
  the place named. — diff against the text below.
- [ ] **AC-17 No other change.** No Rust source, no file under `tests/`, no CHANGELOG line, no version
  bump; `git diff --name-only main` lists exactly the 4 files under Outputs.
- [x] **AC-18 (ship) The classification is proved on Actions.** (corrected at ship) A draft PR touching only
  `recipes/gallery.toml` shows the code line, three `check` legs running `cargo test`, and `pages`
  running; it is closed unmerged. The next docs-only push to `main` prints the docs-only line.
  Met 2026-11-23: draft PR #267 (one comment line at the end of `recipes/gallery.toml`). Its `detect changed
  paths` log printed `ci-code-paths self-test: 88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed` then
  `-> code paths touched: running the full gates`; the `Test` step was `success` on ubuntu, macos and windows;
  `build + browser smoke` (pages) was `success`. Closed unmerged, branch deleted. The docs-only line is quoted in the ship
  report, from the `changes` job of the ship commit's push to `main`.

## Failing Tests

The tests are the rows and checks of `scripts/ci-code-paths.sh --self-test`. Ids are for this spec
only; the script prints paths and labels, not ids.

- **`scripts/ci-code-paths.sh --self-test`, path rows (`expect`)**
  - FT-C1..C4 `expect true` `integrations/ddphotos/edit/edit.js`, `src/lib.rs`, `.cargo/config.toml`,
    `.github/workflows/ci.yml` (kept from today).
  - FT-C5..C12 `expect true` `recipes/web.toml`, `assets/fonts/Go-Regular.ttf`, `clippy.toml`,
    `deny.toml`, `.pre-commit-hooks.yaml`, `demo/index.html`, `bench/corpus/README.md`,
    `fuzz/Cargo.toml`.
  - FT-E1..E3 `expect true` `README.md`, `docs/github-actions.md`, `docs/data-model.md`.
  - FT-U1 `expect true newdir/x.txt`; FT-U2 `expect true src/docs/mod.rs`; FT-U3
    `expect true tests/fixtures/x.md`.
  - FT-D1..D17 `expect false` `docs/x.md`, `docs/.cargo/config.toml`, `docs/recipes/web.toml`,
    `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-x.md`, `decisions/DEC-001-x.md`,
    `guidance/constraints.yaml`, `feedback/x.md`, `reports/x.md`, `gitignore_files_maybe/x.txt`,
    `skills/crustyimg/SKILL.md`, `.claude-plugin/plugin.json`, `CHANGELOG.md`, `LICENSE-MIT`,
    `.repo-context.yaml`, `.variant`, `.gitignore`, `.github/TEMPLATE_README.md`.
  - Count: 4 + 8 + 3 + 3 + 17 = **35 path rows**.
- **List checks (`check`)**: FT-L1 empty list → `true`; FT-L2 `docs/x.md`, blank, `CHANGELOG.md` →
  `false`; FT-L3 `docs/x.md`, `src/lib.rs` → `true`. **3 rows.**
- **Derived include checks**: FT-I1 the floor (1 row); FT-I2 one row per resolved target (35 today);
  FT-I3 no `concat!`/`env!` include (1 row). **37 rows.**
- **Pages checks**: FT-P1 the two lists identical and not empty (1); FT-P2 `recipes/**` listed (1);
  FT-P3 `assets/**` listed (1); FT-P4 each push-list entry classifies as code (10 today: the 8 now
  listed + 2 added). **13 rows.**
- **Total: 35 + 3 + 37 + 13 = 88 rows**, matching the prototype's last line
  `ci-code-paths self-test: 88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed` (rc=0).

### Baseline the build records before changing `classify` (trap 13)

Put the new self-test, derived checks and pages checks in first with `classify` still today's
`CODE_PATHS` one-liner and `pages.yml` unchanged except for the two new entries, and run
`bash scripts/ci-code-paths.sh --self-test`. Driven at design: **rc=1**, last line
`ci-code-paths self-test: 88 rows, 12 mismatched, 0 crashed, 0 junk, 9 failed`, and exactly these 21
FAIL lines (quote them in Build Completion):

```
FAIL recipes/web.toml -> false, expected true
FAIL assets/fonts/Go-Regular.ttf -> false, expected true
FAIL clippy.toml -> false, expected true
FAIL deny.toml -> false, expected true
FAIL .pre-commit-hooks.yaml -> false, expected true
FAIL demo/index.html -> false, expected true
FAIL bench/corpus/README.md -> false, expected true
FAIL fuzz/Cargo.toml -> false, expected true
FAIL README.md -> false, expected true
FAIL docs/github-actions.md -> false, expected true
FAIL docs/data-model.md -> false, expected true
FAIL newdir/x.txt -> false, expected true
FAIL empty list -> false, expected true
FAIL include target assets/fonts/Go-Regular.ttf classifies as docs-only; add it to EXCEPT
FAIL include target docs/github-actions.md classifies as docs-only; add it to EXCEPT
FAIL include target recipes/gallery.toml classifies as docs-only; add it to EXCEPT
FAIL include target recipes/product.toml classifies as docs-only; add it to EXCEPT
FAIL include target recipes/web.toml classifies as docs-only; add it to EXCEPT
FAIL pages.yml path demo/** classifies as code
FAIL pages.yml path recipes/** classifies as code
FAIL pages.yml path assets/** classifies as code
```

(`demo/**` is red on `main` too: today's classifier calls a path pages already builds docs-only.) And
before adding `recipes/**`/`assets/**` to `pages.yml`, with the new `classify`: rc=1, exactly
`FAIL pages.yml lists recipes/**` and `FAIL pages.yml lists assets/**`
(`86 rows, 0 mismatched, 0 crashed, 0 junk, 2 failed`).

### Mutation table (trap 12): apply each alone to the final script, record every red line

All **driven at design** on the prototype, each from the green final script, red set read off the
full self-test run. "Edit" is the exact change. "Red" lists every FAIL line (paths shown; the full
text of each is in "Every message"). Every row exited **rc=1**.

- **M1** `DOCS`: delete `docs/|` → red: `docs/x.md`, `docs/.cargo/config.toml`, `docs/recipes/web.toml`
  (`-> true, expected false`), `docs-only list with a blank line -> true, expected false`. *driven*
- **M2** delete `projects/|` → red: `projects/PROJ-017-…/SPEC-142-x.md`. *driven*
- **M3** delete `decisions/|` → red: `decisions/DEC-001-x.md`. *driven*
- **M4** delete `guidance/|` → red: `guidance/constraints.yaml`. *driven*
- **M5** delete `feedback/|` → red: `feedback/x.md`. *driven*
- **M6** delete `reports/|` → red: `reports/x.md`. *driven*
- **M7** delete `gitignore_files_maybe/|` → red: `gitignore_files_maybe/x.txt`. *driven*
- **M8** delete `skills/|` → red: `skills/crustyimg/SKILL.md`. *driven*
- **M9** delete `\.claude-plugin/|` → red: `.claude-plugin/plugin.json`. *driven*
- **M10** delete `[^/]+\.md$|` → red: `CHANGELOG.md`, `docs-only list with a blank line`. *driven*
- **M11** delete `LICENSE-[A-Z]+$|` → red: `LICENSE-MIT`. *driven*
- **M12** delete `\.repo-context\.yaml$|` → red: `.repo-context.yaml`. *driven*
- **M13** delete `\.variant$|` → red: `.variant`. *driven*
- **M14** delete `\.gitignore$|` → red: `.gitignore`. *driven*
- **M15** delete `|\.github/TEMPLATE_README\.md$` → red: `.github/TEMPLATE_README.md`. *driven*
- **M16** `EXCEPT`: delete `README\.md|` → red: `README.md -> false, expected true`. *driven*
- **M17** delete `docs/github-actions\.md|` → red: `docs/github-actions.md -> false, expected true`
  **and** `include target docs/github-actions.md classifies as docs-only; add it to EXCEPT`. *driven*
- **M18** delete `|docs/data-model\.md` → red: `docs/data-model.md -> false, expected true` (runtime
  read: the derived row cannot see it, which is why it is hand-listed). *driven*
- **M19** `DOCS='^(` → `DOCS='(` (drop the leading `^`) → red: `bench/corpus/README.md`,
  `src/docs/mod.rs`, `tests/fixtures/x.md` (`-> false, expected true`). *driven*
- **M20** in the `DOCS=` line, `[^/]+\.md$` → `.*\.md$` → red: `bench/corpus/README.md`,
  `tests/fixtures/x.md`. *driven* (first attempt hit the comment line and stayed green; re-driven on
  the `DOCS=` line only. The build applies it to the `DOCS=` line.)
- **M21** `DOCS='^(docs/` → `DOCS='^(src/|docs/` → red: `src/lib.rs`, `src/docs/mod.rs`
  (`-> false, expected true`), `docs plus one code path -> false, expected true`,
  `pages.yml path src/** classifies as code`. *driven*
- **M22** first line of `include_targets()` body: insert `  return 0` → red:
  `include scan found 0 files, expected at least 35 (lower INCLUDE_FLOOR only if an include was removed on purpose)`
  (53 rows: the 35 per-target rows vanish, which is why the floor exists). *driven*
- **M23** in `include_targets`' awk, replace
  `if (part[i] == "..") { if (depth > 0) depth--; continue }` with a comment (no `..` collapse) →
  red: 23 lines `include target <file's folder>/../… is not a repository path (the scan did not collapse . or ..)`,
  one per distinct uncollapsed target reached through `..`. *driven*
- **M24** in the awk, `dir = file; sub(/\/[^\/]*$/, "", dir)` → `dir = "."` (resolve against the repo
  root) → red: 25 lines `include target fixtures/… does not exist (the scan resolved a path wrong)`.
  *driven*
- **M25** in the awk, delete `if (text ~ /^[ \t]*\/\//) next` (no comment skip) → red:
  `include target tests/… does not exist (the scan resolved a path wrong)` (from the `///` line at
  `tests/help_has_no_internal_ids.rs:601`; 89 rows). *driven*
- **M26** append `const _X: &str = include_str!("../docs/x.md");` to a scratch copy of `src/lib.rs` →
  red: `include target docs/x.md does not exist (the scan resolved a path wrong)`. *driven* (the
  prompt's negative control; `docs/x.md` does not exist, so it is red on existence.)
- **M27** append `const _X: &str = include_str!("../docs/development.md");` to `src/lib.rs` → red:
  `include target docs/development.md classifies as docs-only; add it to EXCEPT`. *driven* (the
  classification half of the derived row on a real file.)
- **M28** append
  `const _X: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/development.md"));` to
  `src/lib.rs` → red: `no include_str!/include_bytes! builds its path with concat!/env!: src/lib.rs:101:const _X: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/development.md"));`
  (the line number is wherever the append lands). *driven*
- **M29** `pages.yml`: delete `      - "recipes/**"` from the `pull_request` list only → red:
  `pages.yml push.paths and pull_request.paths are identical and not empty`. *driven*
- **M30** `pages.yml`: add `      - "docs/**"` after `      - "demo/**"` in both lists → red:
  `pages.yml path docs/** classifies as code`. *driven*
- **M31** `pages.yml`: delete `recipes/**` and `assets/**` from both lists → red:
  `pages.yml lists recipes/**`, `pages.yml lists assets/**`. *driven*
- **M32 (negative control for the whole self-test)** `  expect true src/lib.rs` →
  `  expect false src/lib.rs` → red: `src/lib.rs -> true, expected false`; last line
  `88 rows, 1 mismatched, 0 crashed, 0 junk, 0 failed`; **rc=1**. The green run before it: rc=0. *driven*

Scratch mutants M26-M28 edit `src/lib.rs` in the build's worktree; revert with
`git checkout src/lib.rs` after each and show `git status --short src` empty before committing.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### The script, verbatim (driven at design; copy it)

`scripts/ci-code-paths.sh` becomes exactly this (227 lines; `git diff --numstat` on the prototype:
`188 14`). Pieces and where they sit in the new file:

- header comment `:2-12` (the rule and the exception rule);
- `DOCS` with a comment per alternative `:15-24`;
- `EXCEPT` with its comment `:26-32`;
- `classify` `:34-46` (blank lines dropped; empty → `true`; EXCEPT hit → `true`; any non-`DOCS` line →
  `true`; else `false`);
- `include_targets` `:48-78` (grep + awk resolver);
- `unresolvable_includes` `:80-87`;
- `pages_paths` `:89-98` (awk over `.github/workflows/pages.yml`);
- `--self-test` `:100-225`: `cd` to the repo root, `expect` (unchanged), `check`, the 35 path rows, 3
  list checks, the derived checks with `INCLUDE_FLOOR=35`, the pages checks, the summary line;
- the classifier call `:227`.

```bash
#!/usr/bin/env bash
# Classify changed paths for the `changes` job in .github/workflows/ci.yml.
#
# Reads file names (one per line) on stdin and prints `false` only when every one
# of them is provably docs-only, otherwise `true`. Anything not provably docs-only
# runs the gates: a path outside the DOCS list below, including a new top-level
# directory nobody has classified yet, counts as code. A docs-shaped file that a
# build or a test reads goes in EXCEPT, which wins over DOCS. Blank lines are
# ignored; an empty list prints `true`, because nothing was proved docs-only.
#
#   git diff --name-only base...HEAD | scripts/ci-code-paths.sh
#   scripts/ci-code-paths.sh --self-test
set -euo pipefail

# Docs and process files: nothing compiles them and no gated job reads them. Every
# alternative is anchored at the repository root.
#   docs/ projects/ decisions/ guidance/ feedback/ reports/  prose, specs, ADRs, process records
#   gitignore_files_maybe/                                   parked notes, read by nothing
#   skills/ .claude-plugin/                                  agent-skill text and its plugin manifest
#   [^/]+\.md$                                               a Markdown file at the root only
#   LICENSE-[A-Z]+$                                          licence texts
#   .repo-context.yaml .variant .gitignore                   repo metadata; no build reads them
#   .github/TEMPLATE_README.md                               template prose, not a workflow
DOCS='^(docs/|projects/|decisions/|guidance/|feedback/|reports/|gitignore_files_maybe/|skills/|\.claude-plugin/|[^/]+\.md$|LICENSE-[A-Z]+$|\.repo-context\.yaml$|\.variant$|\.gitignore$|\.github/TEMPLATE_README\.md$)'

# Docs-shaped files that a test reads, so a change to one must run the gates:
#   README.md               tests/adoption_glue.rs reads it
#   docs/github-actions.md  tests/lint.rs includes it
#   docs/data-model.md      tests/docs_ops.rs reads it
# A file a build or a test includes with include_str!/include_bytes! is checked by
# the self-test below; a file read at run time must be listed here by hand.
EXCEPT='^(README\.md|docs/github-actions\.md|docs/data-model\.md)$'

classify() {
  local paths
  paths=$(grep -v '^$' || true)
  if [ -z "$paths" ]; then
    echo true
  elif printf '%s\n' "$paths" | grep -qE "$EXCEPT"; then
    echo true
  elif printf '%s\n' "$paths" | grep -qvE "$DOCS"; then
    echo true
  else
    echo false
  fi
}

# Every file named by a literal include_str!/include_bytes! under src/, tests/,
# benches/, examples/ or build.rs, as a repository path, one per line. Comment lines
# are skipped. The path is resolved against the including file's folder, as rustc does.
include_targets() {
  local roots=() r
  for r in src tests benches examples build.rs; do
    if [ -e "$r" ]; then roots+=("$r"); fi
  done
  grep -rnE --include='*.rs' 'include_(str|bytes)!\("' "${roots[@]}" | awk '
    {
      file = $0; sub(/:.*/, "", file)
      text = $0; sub(/^[^:]*:[0-9]+:/, "", text)
      if (text ~ /^[ \t]*\/\//) next
      dir = file; sub(/\/[^\/]*$/, "", dir)
      if (dir == file) dir = "."
      while (match(text, /include_(str|bytes)!\("[^"]+"\)/)) {
        lit = substr(text, RSTART, RLENGTH)
        text = substr(text, RSTART + RLENGTH)
        sub(/^include_(str|bytes)!\("/, "", lit); sub(/"\)$/, "", lit)
        n = split(dir "/" lit, part, "/"); depth = 0
        for (i = 1; i <= n; i++) {
          if (part[i] == "" || part[i] == ".") continue
          if (part[i] == "..") { if (depth > 0) depth--; continue }
          out[++depth] = part[i]
        }
        path = out[1]
        for (i = 2; i <= depth; i++) path = path "/" out[i]
        print path
      }
    }' | sort -u
}

# Include forms the scan above cannot resolve: a path built with concat!/env!.
unresolvable_includes() {
  local roots=() r
  for r in src tests benches examples build.rs; do
    if [ -e "$r" ]; then roots+=("$r"); fi
  done
  grep -rnE --include='*.rs' 'include_(str|bytes)!\((concat|env)!' "${roots[@]}" || true
}

# One of pages.yml's two `paths:` lists (push or pull_request), one entry per line.
pages_paths() {
  awk -v want="$1" '
    /^  [a-z_]+:/ { section = $1; sub(/:$/, "", section); inpaths = 0; next }
    section == want && /^    paths:/ { inpaths = 1; next }
    inpaths && /^      - / { e = $0; sub(/^      - /, "", e); gsub(/"/, "", e); print e; next }
    inpaths && /^ *#/ { next }
    inpaths { inpaths = 0 }
  ' .github/workflows/pages.yml
}

if [ "${1:-}" = "--self-test" ]; then
  # The derived checks read the tree; run them from the repository root.
  cd "$(dirname "$0")/.."
  rows=0 mismatched=0 crashed=0 junk=0 failed=0
  expect() {
    local want="$1" path="$2" got rc=0
    rows=$((rows + 1))
    # Assign first, read the exit code after: `set -e` must not end the run on a crash.
    got=$(printf '%s\n' "$path" | classify) || rc=$?
    if [ "$rc" -ne 0 ]; then
      echo "FAIL $path -> classifier exited $rc"
      crashed=$((crashed + 1))
    elif [ "$got" != "true" ] && [ "$got" != "false" ]; then
      echo "FAIL $path -> printed '$got', expected true or false"
      junk=$((junk + 1))
    elif [ "$got" = "$want" ]; then
      echo "PASS $path -> $got"
    else
      echo "FAIL $path -> $got, expected $want"
      mismatched=$((mismatched + 1))
    fi
  }
  # Check rows that are not a single path: print PASS/FAIL with a reason.
  check() {
    local ok="$1" label="$2"
    rows=$((rows + 1))
    if [ "$ok" = "yes" ]; then
      echo "PASS $label"
    else
      echo "FAIL $label"
      failed=$((failed + 1))
    fi
  }

  # Code: the classic roots.
  expect true integrations/ddphotos/edit/edit.js
  expect true src/lib.rs
  expect true .cargo/config.toml
  expect true .github/workflows/ci.yml
  # Code: build and test inputs outside the classic roots.
  expect true recipes/web.toml
  expect true assets/fonts/Go-Regular.ttf
  expect true clippy.toml
  expect true deny.toml
  expect true .pre-commit-hooks.yaml
  expect true demo/index.html
  expect true bench/corpus/README.md
  expect true fuzz/Cargo.toml
  # Code: the docs-shaped test inputs in EXCEPT.
  expect true README.md
  expect true docs/github-actions.md
  expect true docs/data-model.md
  # Code: a top-level directory nobody has classified yet.
  expect true newdir/x.txt
  # Code: DOCS is anchored at the root, and only a root-level *.md is docs.
  expect true src/docs/mod.rs
  expect true tests/fixtures/x.md
  # Docs-only: one path per DOCS alternative.
  expect false docs/x.md
  expect false docs/.cargo/config.toml
  expect false docs/recipes/web.toml
  expect false projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-142-x.md
  expect false decisions/DEC-001-x.md
  expect false guidance/constraints.yaml
  expect false feedback/x.md
  expect false reports/x.md
  expect false gitignore_files_maybe/x.txt
  expect false skills/crustyimg/SKILL.md
  expect false .claude-plugin/plugin.json
  expect false CHANGELOG.md
  expect false LICENSE-MIT
  expect false .repo-context.yaml
  expect false .variant
  expect false .gitignore
  expect false .github/TEMPLATE_README.md

  # Lists, not single paths.
  got=$(printf '' | classify)
  check "$([ "$got" = true ] && echo yes || echo no)" "empty list -> $got, expected true"
  got=$(printf 'docs/x.md\n\nCHANGELOG.md\n' | classify)
  check "$([ "$got" = false ] && echo yes || echo no)" "docs-only list with a blank line -> $got, expected false"
  got=$(printf 'docs/x.md\nsrc/lib.rs\n' | classify)
  check "$([ "$got" = true ] && echo yes || echo no)" "docs plus one code path -> $got, expected true"

  # Every file a build or a test includes must classify as code.
  INCLUDE_FLOOR=35
  targets=$(include_targets)
  count=$(printf '%s\n' "$targets" | grep -c . || true)
  check "$([ "$count" -ge "$INCLUDE_FLOOR" ] && echo yes || echo no)" \
    "include scan found $count files, expected at least $INCLUDE_FLOOR (lower INCLUDE_FLOOR only if an include was removed on purpose)"
  while IFS= read -r t; do
    [ -n "$t" ] || continue
    if printf '%s\n' "$t" | grep -qE '(^|/)\.\.?(/|$)'; then
      check no "include target $t is not a repository path (the scan did not collapse . or ..)"
    elif [ ! -f "$t" ]; then
      check no "include target $t does not exist (the scan resolved a path wrong)"
    elif [ "$(printf '%s\n' "$t" | classify)" != true ]; then
      check no "include target $t classifies as docs-only; add it to EXCEPT"
    else
      check yes "include target $t -> true"
    fi
  done <<< "$targets"
  bad=$(unresolvable_includes)
  check "$([ -z "$bad" ] && echo yes || echo no)" \
    "no include_str!/include_bytes! builds its path with concat!/env!${bad:+: $bad}"

  # pages.yml's two lists must match, list the demo's inputs, and be code paths.
  push_paths=$(pages_paths push)
  pr_paths=$(pages_paths pull_request)
  check "$([ -n "$push_paths" ] && [ "$push_paths" = "$pr_paths" ] && echo yes || echo no)" \
    "pages.yml push.paths and pull_request.paths are identical and not empty"
  for need in 'recipes/**' 'assets/**'; do
    check "$(printf '%s\n' "$push_paths" | grep -qxF "$need" && echo yes || echo no)" \
      "pages.yml lists $need"
  done
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    sample=$(printf '%s' "$p" | sed 's/\*\*/x/g; s/\*/x/g')
    check "$([ "$(printf '%s\n' "$sample" | classify)" = true ] && echo yes || echo no)" \
      "pages.yml path $p classifies as code"
  done <<< "$push_paths"

  echo "ci-code-paths self-test: $rows rows, $mismatched mismatched, $crashed crashed, $junk junk, $failed failed"
  [ $((mismatched + crashed + junk + failed)) -eq 0 ]
  exit
fi

classify
```

**Tools: bash + grep -E + awk, no Ruby.** The classifier already is grep; the include resolver is
one awk program; the pages reader is one awk program. Ruby (`validate-frontmatter.sh`) would parse
the YAML properly, but the lists are flat `      - "x"` lines and the drift check only needs them as
text. Flags used: `grep -E -q -v -c -r -n -x -F --include`, awk `match`/`sub`/`gsub`/`split`/`substr`
with ERE only (no intervals, no gawk extensions), `sed 's/…/…/g'`, `sort -u`; all behave the same on
BSD and GNU grep and on mawk (Ubuntu's default awk). The script avoids bash 4 features (it ran under
macOS bash 3.2). **Driven locally only under BSD grep / BSD awk / bash 3.2** (Docker was down); the
GNU/mawk run of record is the build PR's own `changes` job on `ubuntu-latest`: the build quotes its
`ci-code-paths self-test: 88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed` line in Build Completion.
If that leg disagrees with the local run, stop and report.

**`INCLUDE_FLOOR=35`** is the design-time count of distinct resolved targets (derivation under Design).
A spec that adds includes needs no change; a spec that removes one lowers the floor in the same PR,
as the failure message says. If SPEC-155 to SPEC-157 change the count before this builds, re-derive
it with the script's own `include_targets | grep -c .` and record the number.

### The other three files, verbatim (the prototype's diff)

```diff
diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml
index 1699f90..3f40e71 100644
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -19,11 +19,13 @@ concurrency:
   cancel-in-progress: ${{ github.event_name == 'pull_request' }}

 jobs:
-  # Does this push touch anything the Rust gates can actually judge? A docs- or
-  # spec-only change ran the full three-OS matrix plus avif/webp-lossy/heic/lean/
-  # msrv -- roughly 15 minutes of compute, twice over for the two heic legs, to
-  # prove nothing about a markdown edit. Bookkeeping PRs paid that price four
-  # times in one day.
+  # Does this push touch anything the Rust gates can judge? Anything not provably
+  # docs-only runs the gates. scripts/ci-code-paths.sh lists the docs-only paths;
+  # any other path, including a new top-level directory nobody has classified,
+  # counts as code. A docs-shaped file that a build or a test reads (README.md,
+  # for one) is listed as an exception and runs the gates too. A docs-only change
+  # skips about 15 minutes of compute across the three-OS matrix and the
+  # avif/webp-lossy/heic/lean/msrv jobs.
   #
   # The three `check` jobs are the REQUIRED status checks, so they always run and
   # always report -- they just skip their expensive steps when no code changed.
@@ -51,8 +53,10 @@ jobs:
             files=$(git diff --name-only "${{ github.event.before }}" "${{ github.sha }}" 2>/dev/null || echo "src/")
           fi
           echo "changed files:"; echo "$files" | sed 's/^/  /'
-          # Anything that can affect a build, a test, or a lint result. The regex lives
-          # in the script so its self-test checks the one the workflow uses.
+          # The lists live in the script so its self-test checks the ones the workflow
+          # uses. The self-test also checks that every file a build or a test includes
+          # classifies as code and that pages.yml's two path lists match. It runs here
+          # because this job runs on every push and pull request, docs-only ones too.
           scripts/ci-code-paths.sh --self-test
           # Assign first: a failing command substitution inside `[ ... ]` does not stop a
           # `bash -e` step, so a crashing classifier would read as "docs only" and skip
diff --git a/.github/workflows/pages.yml b/.github/workflows/pages.yml
index d68a2c7..841a117 100644
--- a/.github/workflows/pages.yml
+++ b/.github/workflows/pages.yml
@@ -34,6 +34,10 @@ on:
       - "Cargo.toml"
       - "Cargo.lock"
       - "justfile"
+      # The demo smoke reads recipes/web.toml (tests/demo_smoke.mjs) and the wasm
+      # engine embeds assets/fonts/Go-Regular.ttf (src/text/mod.rs).
+      - "recipes/**"
+      - "assets/**"
       - ".github/workflows/pages.yml"
   pull_request:
     paths:
@@ -44,6 +48,8 @@ on:
       - "Cargo.toml"
       - "Cargo.lock"
       - "justfile"
+      - "recipes/**"
+      - "assets/**"
       - ".github/workflows/pages.yml"
   workflow_dispatch:

diff --git a/docs/development.md b/docs/development.md
index 136e786..f58362c 100644
--- a/docs/development.md
+++ b/docs/development.md
@@ -47,6 +47,17 @@ just report-weekly                 # Generate this week's weekly report
 `reports/daily/` and `reports/weekly/` from spec front-matter and the git log. Reports
 are stand-alone artifacts — re-running overwrites, so they're always a current snapshot.

+## Which PRs skip the Rust gates
+
+CI skips the Rust build, test, clippy and fmt steps only when every changed file is
+provably docs-only: `scripts/ci-code-paths.sh` lists those paths (`docs/`, `projects/`,
+`decisions/`, root-level Markdown and a few more). Anything else runs the full gates,
+because a file that looks like docs can still be compiled in or read by a test:
+`README.md`, `docs/github-actions.md` and `docs/data-model.md` are listed as exceptions
+for that reason. The script's self-test fails if a build or a test includes a file the
+list calls docs-only with a plain string literal in `include_str!` or `include_bytes!`.
+A file a test reads at run time, or includes any other way, must be added to the
+exception list by hand. A new top-level directory runs the gates until someone adds it
+to the docs-only list.
+
 ## Key discipline in this variant

 Because Claude plays every role, context contamination is the biggest risk. Four habits
```

### Every message the script prints (one check per message)

The `changes` job's two log lines (in `ci.yml`, unchanged, AC-14):
- `-> code paths touched: running the full gates` — checked at ship (AC-18, the draft PR).
- `-> docs/specs only: heavy jobs will skip` — checked at ship (the next docs-only push).

Self-test lines (`<…>` is filled in):
- `PASS <path> -> <verdict>` — the green run.
- `FAIL <path> -> <got>, expected <want>` — M1 to M21, M32, baseline.
- `FAIL <path> -> classifier exited <rc>` and `FAIL <path> -> printed '<got>', expected true or false`
  — kept from today, unchanged; their guard is the existing `ci.yml:60-64` assign-then-validate. No
  new mutation: they are not touched by this spec.
- `PASS|FAIL empty list -> <got>, expected true` — FT-L1; red in the baseline.
- `PASS|FAIL docs-only list with a blank line -> <got>, expected false` — FT-L2; M1, M10.
- `PASS|FAIL docs plus one code path -> <got>, expected true` — FT-L3; M21.
- `PASS|FAIL include scan found <n> files, expected at least 35 (lower INCLUDE_FLOOR only if an include was removed on purpose)` — FT-I1; M22.
- `FAIL include target <t> is not a repository path (the scan did not collapse . or ..)` — FT-I2; M23.
- `FAIL include target <t> does not exist (the scan resolved a path wrong)` — FT-I2; M24, M25, M26.
- `FAIL include target <t> classifies as docs-only; add it to EXCEPT` — FT-I2; M17, M27, baseline.
- `PASS include target <t> -> true` — FT-I2 green.
- `PASS|FAIL no include_str!/include_bytes! builds its path with concat!/env![: <grep -rn lines>]` — FT-I3; M28.
- `PASS|FAIL pages.yml push.paths and pull_request.paths are identical and not empty` — FT-P1; M29.
- `PASS|FAIL pages.yml lists recipes/**`, `PASS|FAIL pages.yml lists assets/**` — FT-P2, FT-P3; M31.
- `PASS|FAIL pages.yml path <entry> classifies as code` — FT-P4; M21 (`src/**`), M30, baseline.
- `ci-code-paths self-test: <rows> rows, <m> mismatched, <c> crashed, <j> junk, <f> failed` — every run.

### Comments, verbatim

- Script header: `:2-12` of the script above.
- `ci.yml` `changes` job comment and step comment: the `ci.yml` hunks above.
- `pages.yml`: the two-line comment above `"recipes/**"` in the push list.
No spec, stage, DEC or PROJ id in any of them (AC-15).

### `docs/development.md`

The `## Which PRs skip the Rust gates` section in the diff above, after `## Reports`
(`docs/development.md:44`) and before `## Key discipline in this variant`.

### CHANGELOG

None (precedent under Design: #165 and #245 added no line for CI-only changes).

### Verification the build runs

1. Baseline (above), quoted.
2. The final script: `bash scripts/ci-code-paths.sh --self-test; echo rc=$?` (redirect to a file and
   read `$?`, do not pipe) → 88 rows, rc=0.
3. Every mutation M1 to M32, one at a time from the green script, each red set quoted. A driver can
   live in the scratchpad; it is not committed.
4. Spot verdicts through stdin as `ci.yml` calls it:
   `printf 'recipes/web.toml\n' | scripts/ci-code-paths.sh` → `true`;
   `printf 'docs/x.md\nCHANGELOG.md\n' | scripts/ci-code-paths.sh` → `false`;
   `echo "" | scripts/ci-code-paths.sh` → `true`.
5. The replay (optional; design ran it): 12 flips, 0 reverse, over the 600 commits named under Design.
6. `ruby -ryaml -e 'YAML.load_file(".github/workflows/ci.yml"); YAML.load_file(".github/workflows/pages.yml")'`.
7. `/usr/bin/grep -nE '(SPEC|STAGE|DEC|PROJ)-[0-9]' scripts/ci-code-paths.sh` → no hit.
8. `git diff --name-only origin/main` → exactly the 4 files.
9. On the PR: the `changes` job's log shows the 88-row line (the GNU run) and
   `-> code paths touched: running the full gates` (the PR touches `scripts/` and `.github/`).

No `cargo` run is needed (no Rust change), but CI runs the full gates on the build PR anyway.
**Size: S, holds.** 4 files, +216/−21 lines (`git diff --stat` on the prototype: 11+6+11+188 added,
7+0+0+14 removed); 88 self-test rows; 32 mutations, all driven at design. No Rust, no `tests/` file.
Checkpoint: push after the baseline commit and after the final script.

### What the build cannot prove, and ship's step

The build PR touches `scripts/` and `.github/workflows/`, which are code under both the old and the
new script, so it cannot show the classification change on Actions. **At ship, after merge:**

- The maintainer opens a **draft PR touching only `recipes/gallery.toml`** (one added comment line
  at the end of the file, e.g. `# classifier check`), and checks **once**, when they next look.
  Not `recipes/web.toml`: `tests/demo_smoke.mjs:853-857` asserts the demo's inlined recipe is
  byte-identical to it, so a comment there turns `pages` red. Nothing byte-compares `gallery.toml`
  (its readers parse it, re-emit it without comments, or check its header for forbidden ids), so
  the three `cargo test` legs and `pages` stay green:
  - the `detect changed paths` (`changes`) log prints `-> code paths touched: running the full gates`;
  - each of the three `build / test / clippy / fmt (<os>)` legs ran its `Test` step (`cargo test`),
    not skipped;
  - the `pages` workflow ran (`build + browser smoke`).
  Then close it **unmerged** and delete its branch.
- The next docs-only push to `main` (any design or frame commit) should print
  `-> docs/specs only: heavy jobs will skip` in `changes`.
No polling and no wait loops (AGENTS §16 habit 6): the maintainer checks in.

### Overlap with other open work (from their file lists, 2026-11-22)

`gh pr list --state open --json number,title,files` and `git branch -r`:
- **#240** `ci: run clippy on the wasm32 library build` touches `.github/workflows/ci.yml` (also
  `justfile`, `src/cli/ops.rs`, `src/metadata/mod.rs`). Shared file: `ci.yml`. Its hunks are
  `@@ -87,6 +87,7 @@` and `@@ -97,6 +98,10 @@` (the `ddphotos-parity`/`wasm-roundtrip` area); this
  spec's are `:19-29` and `:51-58`. No hunk overlap.
- **#237** heifer swap touches `.github/workflows/ci.yml` (hunks `@@ -221,20 +221,17 @@`,
  `@@ -245,18 +242,6 @@`, the `heic` job) and `CHANGELOG.md` (this spec does not touch it). No hunk
  overlap.
- **#229** (`docs/roadmap.md`), **#226** (`AGENTS.md`, `src/cli/mod.rs`): no shared file.
- Branches: no `feat/spec-15[5-8]*` branch is pushed (`origin/feat/spec-150…`, `-151-pad`, `-152-trim`,
  `-153-…`, `-159-pad-transparent` only); none touches a file this spec touches by its name, and none
  is open as a PR.
- If #237 or #240 merges first, the rebase is textual only; re-run the self-test after it (the
  pages and include checks read the tree, not `ci.yml`).

### Decisions that apply

- None. No DEC is needed: this is CI plumbing inside the existing gating model.

### Constraints that apply

- `clippy-fmt-clean` — the gates this classifier decides to run; unchanged.
- `one-spec-per-pr` — the build PR carries this spec only.
- `test-before-implementation` — the baseline commit (new rows, old `classify`) lands red first.

### Prior related work

- `#165` (`7827d80`) introduced the docs-only skip and the deny-list.
- `SPEC-146` (#245, `517acea`) and `SPEC-142` (#234) extended `CODE_PATHS`/the self-test.

### Out of scope (for this spec specifically)

- Gating semantics: which jobs are required, which are gated on `changes`, and the `check`
  always-run pattern stay as they are. `supply-chain`, `cost-data` and `metadata` stay ungated.
- `publish-crates.yml` and `publish-npm.yml` path lists (release-plumbing triggers, on purpose).
- Building `fuzz/` in CI (detached workspace; `tests/fuzz_regressions.rs` is the per-PR guard).
- Any Rust source or test under `tests/`; any change to branch protection.
- SPEC-155 to SPEC-158, which land after this one.

## Notes for the Implementer

- Copy the script verbatim; do not re-derive the patterns. If a row fails on Linux that passed here,
  that is a GNU/BSD difference to report, not to patch around silently.
- Mutation M20 must edit the `DOCS=` line, not the comment above it that holds the same text.
- The include resolver must collapse `..` against the including file's folder (M23, M24 prove both).
- `printf '%s\n' "$t" | grep -qE '(^|/)\.\.?(/|$)'` is the canonical-path test; keep the `?` (it
  catches `./` as well as `../`).
- `[ -n "$t" ] || continue` guards the here-string's trailing empty line.
- Commit with `git commit -s` (DCO).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-154-ci-allow-list`
- **PR (if applicable):** #266 (draft)
- **All acceptance criteria met?** yes, except AC-18 (ship's step, after merge). AC-15's id grep has two
  fixture-path hits, see Deviations.
- **New decisions emitted:** none
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`; 4 files, plus this spec
  file's Build Completion and cost entry, which land in the final commit):
  - `scripts/ci-code-paths.sh` — deny-list replaced by `DOCS` + `EXCEPT`; `include_targets`,
    `unresolvable_includes`, `pages_paths`, 88-row self-test
  - `.github/workflows/pages.yml` — `recipes/**` and `assets/**` in both lists, two-line comment
  - `.github/workflows/ci.yml` — `changes` job comment and step comment rewritten; no control-flow change
  - `docs/development.md` — new `## Which PRs skip the Rust gates` section
- **Deviations from spec:**
  - AC-15 / verification step 7 says the id grep (`(SPEC|STAGE|DEC|PROJ)-[0-9]`) finds no hit. It finds
    two: `scripts/ci-code-paths.sh:161` (`projects/PROJ-017-…/SPEC-142-x.md`) and `:162`
    (`decisions/DEC-001-x.md`). Both are the spec's own verbatim self-test fixture paths, not comments or
    messages; the script is byte-identical to the spec's text, so I kept them.
  - No GNU-tools local run: no GNU grep or gawk installed (brew has only `bash`), no Docker. The
    Linux run of record is the PR's `changes` job (below).
- **Follow-up work identified:** none.

### Evidence

- **Step 0:** rebased onto `origin/main` (already current); spec at `cycle: build`; `cbdf66b` in history;
  `CODE_PATHS` one-liner present; `include_targets | grep -c .` = 35 = the floor.
- **Baseline 1** (new rows, old `classify`, `pages.yml` with the two new entries): rc=1,
  `ci-code-paths self-test: 88 rows, 12 mismatched, 0 crashed, 0 junk, 9 failed`, 21 FAIL lines;
  `diff` against the spec's 21-line list: identical.
- **Baseline 2** (final `classify`, `pages.yml` without the two entries): rc=1,
  `86 rows, 0 mismatched, 0 crashed, 0 junk, 2 failed`, exactly `FAIL pages.yml lists recipes/**` and
  `FAIL pages.yml lists assets/**`.
- **Script byte diff** against the spec's extracted block (227 lines): no differences. The three
  other files were applied with `git apply` of the spec's diff, cleanly.
- **Green run:** rc=0, `88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed`.
- **Spot verdicts** (stdin): `recipes/web.toml` → `true`; `docs/x.md`+`CHANGELOG.md` → `false`; empty
  line → `true`.
- **YAML load** of `ci.yml` and `pages.yml` (ruby): ok.
- **`git diff --name-only origin/main`:** exactly the 4 files.
- **Mutations**, each applied alone to a scratch copy of the tree (script, `src/lib.rs`, `pages.yml`
  copied fresh each time, the build worktree never touched; `git status` clean after), diff against
  green checked non-empty, every run rc=1. M20 edited the `DOCS=` line. **All 32 red sets equal the
  design table; no cell was wrong.**
  - M1 `docs/x.md`, `docs/.cargo/config.toml`, `docs/recipes/web.toml`, `docs-only list with a blank line`
  - M2 `projects/PROJ-017-…/SPEC-142-x.md`; M3 `decisions/DEC-001-x.md`; M4 `guidance/constraints.yaml`;
    M5 `feedback/x.md`; M6 `reports/x.md`; M7 `gitignore_files_maybe/x.txt`; M8 `skills/crustyimg/SKILL.md`;
    M9 `.claude-plugin/plugin.json`
  - M10 `CHANGELOG.md`, `docs-only list with a blank line`; M11 `LICENSE-MIT`; M12 `.repo-context.yaml`;
    M13 `.variant`; M14 `.gitignore`; M15 `.github/TEMPLATE_README.md`
  - M16 `README.md`; M17 `docs/github-actions.md` and `include target docs/github-actions.md classifies as docs-only; add it to EXCEPT`; M18 `docs/data-model.md`
  - M19 `bench/corpus/README.md`, `src/docs/mod.rs`, `tests/fixtures/x.md`; M20 `bench/corpus/README.md`, `tests/fixtures/x.md`
  - M21 `src/lib.rs`, `src/docs/mod.rs`, `docs plus one code path`, `pages.yml path src/** classifies as code`
  - M22 `include scan found 0 files, expected at least 35 (…)`, 53 rows
  - M23 23 `is not a repository path` lines; M24 25 `does not exist` lines; M25 1 line
    `include target tests/… does not exist` (the `///` comment at `tests/help_has_no_internal_ids.rs:601`), 89 rows
  - M26 `include target docs/x.md does not exist`; M27 `include target docs/development.md classifies as docs-only; add it to EXCEPT`
  - M28 `no include_str!/include_bytes! builds its path with concat!/env!: src/lib.rs:101:…`
  - M29 `pages.yml push.paths and pull_request.paths are identical and not empty`; M30 `pages.yml path docs/** classifies as code`; M31 `pages.yml lists recipes/**`, `pages.yml lists assets/**`
  - M32 `src/lib.rs -> true, expected false`; `88 rows, 1 mismatched, 0 crashed, 0 junk, 0 failed`
- **Overlap after the last rebase:** `gh pr list` shows #240 and #237 (`ci.yml`, other hunks), #229, #226;
  unchanged from the spec. Neither merged.
- **Linux self-test (PR #266 `changes` job, ubuntu-latest, GNU grep/awk):**
  `ci-code-paths self-test: 88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed`, then
  `-> code paths touched: running the full gates`. No GNU/BSD difference.
- **CI legs at head `e20126b`:** all pass (3 `check` legs, avif, webp-lossy, heic x2, lean, msrv,
  ddphotos parity, wasm round-trip, pages smoke, supply-chain, DCO, front-matter, detect changed paths).
  `cost-capture audit` shows `pending` with every step already `success` (a stale status; this commit's
  push re-runs it).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocking. The AC-15 id grep claim ("no hit") contradicts the verbatim script's fixture paths.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. No design call turned out wrong; all 32 mutation cells and both baselines reproduced exactly.
3. **If you did this task again, what would you do differently?**
   — Drive the mutations from a script from the start (done here), and not leave a `--watch` on a check
   whose status can stay stale.

### Punch list (verify)

- **Ship's step** now adds the comment line to `recipes/gallery.toml`, at the end of the file.
  Chosen by this grep over `tests/`, `scripts/`, `demo/`, `integrations/`, `src/`:
  - `tests/demo_smoke.mjs:853-857` reads `recipes/web.toml` and asserts the inlined demo recipe is
    byte-identical: `web.toml` is compared, so a comment there turns `pages` red.
  - `gallery.toml` is read by `src/recipe/bundled.rs:46` (`include_str!`; parse test, and a check
    that the leading `#` header lines hold no forbidden ids), `tests/recipe_strict_params.rs:173`
    (parse), `tests/recipe_round_trip.rs:303` (re-emit without comments) and
    `tests/recipe_corpus_output.rs:269` (parse). None compares bytes or comments. A line at the end
    of the file is outside the header, so even the header check does not see it.
  - `printf 'recipes/gallery.toml\n' | scripts/ci-code-paths.sh` → `true`; `pages.yml` triggers on
    `recipes/**`.
- **`docs/development.md`** now reads: "The script's self-test fails if a build or a test includes a
  file the list calls docs-only with a plain string literal in `include_str!` or `include_bytes!`.
  A file a test reads at run time, or includes any other way, must be added to the exception list by
  hand." The verbatim copy above matches it byte for byte. Driven by appending one line to
  `src/lib.rs`, running `--self-test`, restoring:
  - `include_str!(r"../docs/development.md")` → rc=0, 88 rows (not scanned, as the sentence says);
  - `include_str! ("../docs/development.md")` → rc=0, 88 rows (not scanned);
  - `include_str!("../docs/development.md")` → rc=1, `FAIL include target docs/development.md
    classifies as docs-only; add it to EXCEPT`;
  - `include_bytes!("../docs/development.md")` → rc=1, same FAIL line.
- **AC-9** now reads: "An include whose path is built with `concat!` or `env!` is refused", and
  says a raw string, a space before the parenthesis, a multi-line `include_str!(` and `include!` are
  not scanned.
- **Self-test:** `bash scripts/ci-code-paths.sh --self-test > out.txt 2>&1` → rc=0,
  `ci-code-paths self-test: 88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed`.
  `git diff 2a70156 -- scripts/` is empty.
- **Not changed, for the orchestrator:** AC-18 and the frame's "What only a PR can prove" still say
  `recipes/web.toml`; the script header sentence "A file a build or a test includes with
  include_str!/include_bytes! is checked by the self-test below" claims more than the plain form.
- **CI snapshot:** see the PR (filled in the report, not here: the push has to come first).

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — The classifier was right whatever the frame's numbers said: the frame priced an extra run at 2 and
   named a wrong reason for `pages`, and design corrected both (0 extra runs; `recipes/**` for the
   smoke's byte comparison). Design could run only BSD tools, so the build PR's own `changes` job, which
   prints the self-test, became the Linux run of record (88 rows, 0 failed); that worked because the
   job prints it. The proof edit chosen at design, `recipes/web.toml`, would have turned the pages smoke
   red because `tests/demo_smoke.mjs` byte-compares it; no one grepped the readers of the file being
   edited. Verify caught it and the proof used `recipes/gallery.toml`, which stayed green on all legs.
   Two punch-list sessions ran from one prompt; the second stopped correctly at the moved head, but
   cost $1.19 to find that out.

2. **Does any template, constraint, or decision need updating?**
   — Three proposals, none applied. (a) "A proof edit touches a file nothing byte-compares; grep its
   readers first": not in AGENTS §15-16, HANDOFF or `guidance/`. (b) Verify and punch-list prompts use
   `git checkout --detach origin/<branch>`, since auto mode blocks `git reset --hard`: only the SPEC-154
   punch-list prompt does; SPEC-150, SPEC-151 and SPEC-154's verify prompts still say `reset --hard`.
   (c) "A self-test that can only check written rows needs an independent whole-tree pass at verify":
   not stated anywhere; verify's 1,303-path comparison is the precedent.

3. **Is there a follow-up spec I should write now before I forget?**
   — None new. The rename hole (with the script header's over-claiming sentence) is in STAGE-063, and
   quoted non-ASCII paths and the include floor's drift are in `guidance/ideas.yaml`; the evidence
   names no gap those miss.
