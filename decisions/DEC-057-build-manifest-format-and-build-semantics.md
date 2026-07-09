---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-057
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-08
supersedes: null
superseded_by: null

affected_scope:
  - src/build/**
  - src/cli/mod.rs
  - src/lib.rs

tags:
  - build
  - manifest
  - toml
  - reproducibility
  - cli-semantics
---

# DEC-057: The `crustyimg.build.toml` manifest format and `build` command semantics

## Decision

A **build** is declared in its own versioned TOML file (`crustyimg.build.toml` by
default) whose `[[target]]`s each bind sources (a glob/dir/path, or a list) to a **recipe
file**, an output directory, and an optional name template; `crustyimg build [FILE]`
validates every target up front, then runs them all through the shipped per-input
`apply` worker — and, unlike `apply`, **overwrites its own declared outputs without
`--yes`**, because a build owns its output tree and must be re-runnable.

```toml
version = 1

[[target]]
source = "assets/**/*.png"      # or ["a/*.png", "b/"]
recipe = "recipes/web.toml"     # a path to a recipe TOML (DEC-005)
out    = "dist/img"             # output directory (auto-created, PATCH-001)
name   = "{stem}_web.{ext}"     # optional; default "{stem}.{ext}"
```

## Context

crustyimg could already run a recipe over inputs ad hoc (`apply --recipe web.toml
assets/*.png --out-dir dist`). PROJ-007 ("a Makefile for images, verifiable") turns that
into a declared, cached, checkable **build**. SPEC-063 is the skeleton that STAGE-021
(content-addressed cache), STAGE-022 (lockfile + `--check`), and STAGE-023 (`--watch`)
all extend, so the manifest format and the command's semantics are a **stable contract**
those stages will key on — the DEC-005 (recipes) analog for builds.

Constraints in play: no new dependency (serde/toml are shipped); `untrusted-input-hardening`
(the manifest is config input); `ergonomic-defaults` (one short `crustyimg build`);
DEC-006 (rayon, no async); DEC-007 (typed errors, exit-code mapping only at the CLI
boundary); DEC-015 (partial-batch exit 6).

## Alternatives Considered

- **Option A: extend the recipe format with source/output keys**
  - What it is: let a recipe TOML optionally carry `source`/`out`, so one file
    declares both the transform and where to apply it.
  - Why rejected: it destroys what makes recipes valuable. A recipe is *portable and
    input-agnostic* — "tune once, replay across many" (DEC-005). Binding it to one
    project's directory layout makes it unshareable, and versioning one file for two
    audiences (transform authors, build owners) couples changes that should move
    independently. Keep the transform reusable; make the **binding** the new artifact.

- **Option B: declare targets via CLI flags / a `[build]` section in `Cargo.toml`-style config**
  - What it is: `crustyimg build --source 'a/*.png' --recipe r.toml --out dist`, or hide
    the build inside an existing project config file.
  - Why rejected: flags can't express N targets, which is the whole point (an asset tree
    has many source×recipe mappings). A dedicated, discoverable, reviewable file is the
    thing you commit and diff — the "reviewed like code" half of the project thesis.

- **Option C: `build` inherits `apply`'s overwrite guard (`--yes` required)**
  - What it is: keep one overwrite rule across the whole CLI.
  - Why rejected: a build that needs `--yes` on every re-run is not a build. `make`
    overwrites its targets; so does every asset pipeline. The guard exists to protect
    files the user didn't ask you to touch — but a manifest's `out` tree is *declared*,
    which is exactly the user asking. (See Consequences for the containment argument.)

- **Option D (chosen): a dedicated, versioned `crustyimg.build.toml` + overwrite-owned-outputs**
  - What it is: the schema above; `deny_unknown_fields` on every table; a `version`
    gate; a size guard; a two-phase executor (validate all targets → run all targets)
    reusing `apply_one` verbatim.
  - Why selected: recipes stay portable, builds stay declarative and reviewable, and the
    executor is pure reuse (no new machinery, no new dependency). The cache (STAGE-021)
    keys on a target's resolved (input, recipe, config); the lockfile (STAGE-022) pins
    a target's outputs; `--watch` (STAGE-023) re-runs affected targets. All three need
    exactly this shape.

## Consequences

- **Positive:**
  - "Declare my asset build once, run it with one command," and re-run it freely.
  - The manifest is the natural key surface for the cache and the lockfile.
  - Pure reuse: `run_build` orchestrates the shipped `apply_one` / `source::resolve` /
    `Recipe::build_pipeline` / sink. No new dependency; `just deny` unchanged.
  - **Fail-before-write:** every target's recipe is parsed + pipeline-probed and every
    source set resolved *before* the first output is written, so target #2's typo can't
    strand target #1's half-built outputs. (Stronger than `apply`, which has one target.)

- **Negative:**
  - `Overwrite::Allow` means a build silently replaces files in its `out` dirs. Mitigated
    by containment, not by prompting: outputs are written only *inside* a target's `out`
    dir (the sink rejects name-template path escapes and symlinked destinations), and a
    build never deletes. STAGE-022's lockfile makes replacements reviewable; STAGE-021's
    cache makes most of them no-ops.
  - A second TOML format to version and document alongside recipes.
  - Manifest paths resolve against the **process working directory**, not the manifest's
    directory (the `make`/`apply` convention). Running `crustyimg build sub/x.build.toml`
    from a parent dir therefore resolves `sub`'s sources against the parent. Chosen for
    predictability and consistency with every other path argument in the CLI; revisit if
    users report it (a `--dir`/manifest-relative mode is additive).

- **Neutral:**
  - `version` is an **integer** here (`version = 1`), where a recipe's is a string
    (`version = "1"`). The manifest is new, so it gets the type it should have had.
  - `-o`/`--out-dir`/`--name-template` are ignored by `build` — the manifest declares them.
  - Targets run sequentially; each target's inputs fan out over rayon (`-j` bounds the pool).
  - Output format follows the source format (as `apply`'s batch path does). A per-target
    `format`/`quality` override is deliberate future scope, not an oversight.

## Hardening

The manifest is untrusted config, so it inherits the recipe discipline exactly:
`BUILD_MANIFEST_MAX_BYTES` (64 KiB) checked against on-disk size *before* the read and
against string length *before* `toml::from_str`; `BUILD_MANIFEST_MAX_TARGETS` (1024);
`#[serde(deny_unknown_fields)]` on both `BuildManifest` and `Target` so a typo is an
error, never a silently-ignored key; a `version` gate before any semantic validation; and
a per-target check rejecting empty fields and `-` (stdin) sources. Decode caps, path
traversal, and symlink-destination guards are inherited unchanged from image/source/sink.

## Exit codes (mirror `apply`, `docs/api-contract.md`)

| Failure | Code |
|---|---|
| Manifest parse / unknown field / unsupported version / too large / invalid target | 2 |
| Manifest file missing or unreadable (incl. the discovered default) | 3 |
| A target's recipe file missing or unreadable | 3 |
| A target's recipe invalid (unknown op, bad params) | 1 |
| A target's source missing / empty glob | 3 (invalid glob pattern → 2) |
| Per-output decode/encode failure | 6 (partial batch, DEC-015) |

## Validation

Right if STAGE-021/022/023 extend this manifest and executor without reshaping either —
the cache keys on `(resolved input, recipe, config)` per target, the lockfile pins each
target's outputs, `--watch` re-runs affected targets. Revisit if:

- users trip over cwd-relative paths (→ add manifest-relative resolution or `--dir`);
- targets need per-target `format`/`quality`/inline ops (→ `version = 2`, additive keys);
- `Overwrite::Allow` surprises someone in practice (→ a `--frozen`-style guard, which
  STAGE-022 introduces anyway);
- sequential-targets parallelism becomes the bottleneck on wide, shallow builds (→ flatten
  the fan-out to one rayon pass over (target, input) pairs).

## References

- Related specs: SPEC-063 (this decision's spec), SPEC-006 (recipe TOML), SPEC-031
  (rayon batch + `apply_one`), SPEC-035 (recipe size guard)
- Related decisions: DEC-005 (recipes — the sibling contract), DEC-006 (rayon batch),
  DEC-007 (typed errors), DEC-015 (partial-batch exit 6), DEC-036 (`RECIPE_MAX_BYTES`),
  DEC-044 (`--out-dir` auto-create), DEC-035 (symlink-destination guard)
- Stage: `projects/PROJ-007-reproducible-build/stages/STAGE-020-build-command-and-manifest.md`
- User docs: `docs/cli-reference.md` (`build [FILE]`)
