---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-059
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

created_at: 2026-07-09
supersedes: null
superseded_by: null

affected_scope:
  - src/build/lock.rs
  - src/build/mod.rs
  - src/cli/mod.rs

tags:
  - build
  - lockfile
  - reproducibility
  - hardening
  - ci
---

# DEC-059: the `crustyimg.build.lock` format; pin the robust, record the fragile; the env-aware `--check`

## Decision

`crustyimg build` writes a committed **`crustyimg.build.lock`**: a versioned TOML file
with one `[[output]]` per written file — `{path, source, recipe, key, hash, bytes}` —
and one `[env]` block `{crustyimg_version, target, features}`.

The `key` is the **pinned** DEC-058 cache key. The `hash` is the **recorded** SHA-256 of
the bytes actually written, together with the `[env]` they were observed under. That
asymmetry is the whole decision: **pin the robust, record the fragile.**

`build --check` (aliases `--frozen`, `--locked`) re-runs the build and diffs it against
the committed lockfile instead of refreshing it, exiting **7** (`CliError::CheckFailed`,
DEC-025) on drift and **never** writing the lockfile. Drift is env-aware:

| committed vs current | verdict |
|---|---|
| a `path` in only one lock | **drift** (added / removed) |
| same `path`, different `key` | **drift**, always — env-independent |
| same `path` + `key`, different `hash`, **same** `env.target` | **drift** (a real regression) |
| same `path` + `key`, different `hash`, **different** `env.target` | **informational** — unless `--strict` |

No new dependency: `serde`, `toml`, and `sha2` all ship already.

## Context

STAGE-020 gave crustyimg a declared build, STAGE-021 made it incremental, SPEC-065 made
its source→output mapping injective. STAGE-022's thesis — "a Makefile for images,
**verifiable**" — needs a committed artifact CI can assert against.

The hard question is what a lockfile is allowed to *promise*. STAGE-021's
encoder-determinism experiment settled the shape of the answer: every encoder in the
tree (including AVIF/rav1e and lossy WebP) is byte-identical run-to-run **and across
thread counts on a fixed machine** — but nothing establishes that across arch, OS, or
codec version, and the upstream encoders do not promise it. A lockfile that asserted
cross-arch byte-identity would be a lockfile that fails CI on a Linux runner for a
macOS-recorded file, for no defect. A lockfile that asserted nothing would be
decoration.

## Alternatives Considered

### What the lockfile pins

- **Option A: pin the output hash.** Record the encoded bytes' digest and require it to
  match.
  - Why rejected: it promises what the encoders don't. The first time CI moves from
    arm64 to x86_64 — or `image` bumps a patch — every output "drifts" and the gate gets
    disabled, which is worse than not having it. A gate that cries wolf is deleted.

- **Option B: pin nothing but the source content hashes.** Record the inputs; ignore what
  came out.
  - Why rejected: it misses the recipe, quality, tool version, and feature set — all of
    which change the output. It would pass a build whose recipe was rewritten.

- **Option C (chosen): pin the cache key; record the output hash + env.**
  - The DEC-058 key already digests exactly the seven output-affecting inputs (schema
    version, tool version, feature signature, canonical recipe, quality, input extension,
    input content). It is a pure function of the **inputs**, so it is reproducible on any
    machine, and key equality *is* input equality. Key drift is therefore unambiguous,
    cross-machine, and always a failure.
  - The output hash is still worth recording: within one machine the encoders *are*
    deterministic, so a same-env hash change is a real regression (a codec bug, a
    dependency swap the key didn't see). Scoping it by `[env]` is what lets the same
    lockfile be strict on the machine that wrote it and tolerant of the one that didn't.
  - `--strict` is the opt-in for shops on a pinned toolchain and arch who genuinely want
    byte-identity enforced. Off by default.

This is the split the STAGE-022 design review insisted on: **don't fail CI on cross-arch
encoder bytes.**

### Perceptual verification stays out of `--check`

Rejected: a `--check --perceptual` mode that decodes both sides and scores SSIMULACRA2.

"Did the image actually change?" is a real question, and crustyimg already answers it —
`crustyimg diff a.png b.png --fail-under 90` (SPEC-023, DEC-025, exit 7). But `--check`
compares a build against a *lockfile*, and a lockfile stores digests, not pixels: there
is no prior image to score against unless the lockfile grows to hold one (it won't) or
the old outputs survive on disk (they don't — the build just overwrote them). Perceptual
verification is a comparison of two *images*, so it stays the composable `diff` command.
The two exit-7 gates compose in a CI script; they do not need to be one flag.

### `--frozen` / `--locked` are aliases, not a second mode

Cargo distinguishes them because one forbids updating `Cargo.lock` and the other also
forbids touching the network. **crustyimg has no network** (the no-service guardrail,
`docs/territory.md`), so both collapse into "assert, never write" — which is exactly
`--check`. They are clap `visible_alias`es of one flag rather than three code paths that
could drift apart.

### Where the lockfile lives, and what it keys on

- `crustyimg.build.lock`, resolved against the **working directory**, like the manifest
  (DEC-057) and the cache root (DEC-058). Written atomically (temp → `rename`), so a
  concurrent reader never sees a half-file.
- Entries key on the **output path** — which is legal precisely because SPEC-065
  guarantees the build maps sources to outputs injectively. (DEC-058's cache key
  deliberately excludes the destination, so it cannot serve as the lockfile's key: one
  cache entry can materialize to many destinations.)
- Paths are stored `/`-separated so a lockfile committed on macOS reads on Windows.
- Outputs are **sorted by `path`** before serialization. Two clean builds on one machine
  therefore write byte-identical lockfiles, and a review diff shows only what moved.

## Consequences

- **Positive:**
  - The project's headline lands: commit the lockfile, add `crustyimg build --check` to
    CI, and an image output that changes without a source change fails the build with a
    named, readable diff — image outputs reviewed like code.
  - Drift messages say *what* drifted and *why*: `drift: "dist/a.png": inputs changed
    (key 673b73d15c55 → 2b4738ced442)`.
  - **A second line of defense on injectivity.** SPEC-065's collision check runs before
    any decode, so it cannot expand `{ext}`; a target naming a *literal* extension
    (`{stem}.png`) alongside one naming `{stem}.{ext}` slips through it (a disclosed
    residual). The lockfile is built *after* the encodes, where the real extensions are
    known, so the executor re-runs `find_output_collision` on the resolved paths and
    refuses to pin an ambiguity — `CliError::OutputCollision`, exit 2. The colliding
    outputs are already written by then (the race already happened), so this closes the
    *silent* failure, not the race. A pre-decode format sniff would close both this and
    SPEC-065's `{ext}` false positives; it remains the better long-term fix.
  - A malformed / oversized / unknown-version lockfile fails at exit 2 **before any
    output is written** — the committed lock is parsed in the prepare phase.

- **Negative:**
  - **`--no-cache` now reads and hashes every source.** The key is what the lockfile
    pins, so it must be computed whether or not the store is open. A `--no-cache` build
    pays one extra read + SHA-256 per input — dwarfed by the decode it precedes, but no
    longer free.
  - The lockfile is only as trustworthy as `[env].target` (`"{ARCH}-{OS}"`). Two machines
    with the same arch/OS but different codec builds (a distro-patched libwebp, say)
    look like one environment. The tool version and feature set are in the key, so this
    only affects the *hash* half; the failure mode is a same-env hash "regression" that
    is really cross-env. Acceptable: it errs toward reporting, not toward silence.
  - Cross-env output-hash variance is reported on **every** `--check` run from a
    different machine. Informational, but noisy for a mixed-arch CI matrix.

- **Neutral:**
  - `--check` still *performs* the build (it writes outputs); it just refuses to write
    the lockfile. That is what makes the comparison possible at all.
  - A partial-batch failure (exit 6) neither writes nor checks the lockfile: a half-built
    tree has nothing coherent to pin.
  - `--strict` without `--check` is silently ignored (there is nothing to be strict about
    when the lockfile is being rewritten).
  - A plain `build` never *reads* the lockfile — it owns it, and regenerates even a
    corrupt one.
  - `--check`/`--strict` are clap `global = true` like `--no-cache`, so they parse on
    every subcommand but only `build` reads them.

## Validation

Right if a team can commit the lockfile, run `build --check` in a mixed-arch CI matrix,
and see it fail on real source/recipe drift while never failing on encoder bytes alone.
Wrong if `--check` earns a reputation for false positives and gets removed from CI.
Revisit if:

- a same-env hash "regression" is traced to two machines sharing an `arch-os` but not a
  codec build → `[env]` grows a codec-version fingerprint (a schema bump);
- users ask for a machine-readable drift report → `--check --json` (a natural follow-up,
  deliberately not built here);
- the pre-decode format sniff lands → the literal-`{ext}` residual closes at the prepare
  phase and the post-encode collision check becomes belt-and-suspenders;
- `--watch` (STAGE-023) needs a lockfile that updates incrementally rather than wholesale.

## References

- Related specs: SPEC-066 (this decision's spec), SPEC-065 (injectivity — what makes the
  output path a valid key), SPEC-064 (the cache key this pins), SPEC-063 (the executor),
  SPEC-023 (the SSIMULACRA2 `diff` — the perceptual escape hatch)
- Related decisions: DEC-058 (the cache key + the within-machine determinism experiment),
  DEC-057 (build manifest + executor; the injective source→output constraint), DEC-025
  (exit 7 "check not satisfied"), DEC-005 (versioned TOML + size guard), DEC-007 (typed
  errors, exit-code mapping only at the CLI boundary), DEC-036 (size-guard pattern),
  DEC-015 (partial-batch exit 6)
- Stage: `projects/PROJ-007-reproducible-build/stages/STAGE-022-reproducibility-lockfile.md`
- User docs: `docs/cli-reference.md` (`build [FILE]` → "The lockfile")
