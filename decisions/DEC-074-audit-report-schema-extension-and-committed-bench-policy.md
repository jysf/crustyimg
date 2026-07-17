---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-074
  type: decision
  confidence: 0.88
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-16
supersedes: null
superseded_by: null

affected_scope:
  - "src/cli/mod.rs"
  - "src/analysis/decide.rs"
  - "scripts/bench.py"
  - "examples/gen_bench_corpus.rs"
  - "bench/**"
  - "justfile"
  - "docs/cli-reference.md"

tags:
  - audit
  - timing
  - explain-json
  - benchmark
  - corpus
  - privacy
  - no-telemetry
  - lint
  - cc0
  - pipe-clean
---

# DEC-074: audit-report schema extension (`--timing`/`--json`) + committed-bench policy

## Decision

Two coupled choices for SPEC-088:

1. **`--timing` extends the existing `crustyimg.optimize.explain/v1` schema
   additively and gated** — a trailing `"timing":{"decode_ms","encode_ms",
   "total_ms"}` object emitted **only** under `--timing`, exactly as `"ssim"` is
   emitted only under scoring (DEC-071). No version bump, no per-command fork. The
   consistent audit `--json` is made available on `web` and `apply` (terminal
   `optimize` recipes) by routing them through the same `ExplainTrace.write_json`
   the `optimize` auto-decision already used; `optimize`'s `--json` is a synonym for
   `--explain=json`. The audit surface is **auto-decision-only**: on a format-pinned
   (`-o`/`--format`), `--profile preserve`, or plain-pixel-recipe run there is no
   decision to report, so `--json`/`--timing` is a **usage error (exit 2)**, not a
   silently ignored flag. For the same reason — **stdout stays pipe-clean** — the
   JSON report is also a **usage error (exit 2)** whenever **the image sink resolves
   to stdout**: both target stdout, and emitting both yields an unparseable report
   glued to an undecodable image. The rule keys on the resolved *sink*, not on a
   flag spelling, so it closes **both doors**: an explicit `-o -`, and the **bare
   default** (no `-o`, no `--out-dir`), which is the sink `optimize photo.jpg
   --json` silently used. It is enforced at the one writer all three verbs share, so
   it also **corrects the pre-existing `optimize --explain=json`** by either door
   (see *Corrections*). `--timing` alone and the human `--explain` render to stderr
   and stay compatible with a stdout image.

2. **The committed benchmark is a checked-in Python-3 stdlib script
   (`scripts/bench.py`) over a small, license-clean corpus (`bench/corpus/`)** that
   the CLI's own `--json --timing` report feeds — offline, deterministic (for
   savings/scores), and with **no telemetry**. The harness accepts `--corpus <dir>`
   so the maintainer runs the real SPEC-083 numbers **without those photos entering
   git**. `just bench` is the corpus harness; the prior criterion recipe is renamed
   `just bench-micro` (DEC-028 amended).

   **Corpus policy — synthetic *and* one real CC0 photo.** Four synthetic images
   (pure math, regenerable via `examples/gen_bench_corpus.rs`) **plus**
   `photo_forest_cc0.jpg`, a real CC0-1.0 public-domain photograph (Wikimedia
   Commons, author DimiTalen, own work; downscaled to 800×532 and stripped of all
   metadata — provenance in `bench/corpus/README.md`). The real photo is not
   decoration: **synthetic math does not produce a photograph**. Smooth gradients
   measure a flat-region ratio of 1.00 and the classifier routes every synthetic
   image to `graphic-logo`, so before this addition `just bench` built
   `--features avif` and **never encoded a single AVIF** — the flagship path was
   unbenched. The CC0 photo classifies `photograph` and its AVIF candidate wins
   (~30%, SSIMULACRA2 ≈ 81), which is the only reason the harness covers all three
   branches (AVIF-lossy, lossless-WebP, never-bigger passthrough).

3. **The two audit schemas stay separate.** `optimize`/`web`/`apply` share
   `crustyimg.optimize.explain/v1` (one file's encode *decision*); `lint` keeps
   `crustyimg.lint/v1` and its `--format json` spelling (*findings* across many
   files, plus `human`/`sarif` reports a boolean `--json` could not select).
   Reconciled by **documentation, not by a merge**: `docs/cli-reference.md` §"Audit
   surface" states both schemas, both spellings, and why they differ. Forcing
   findings into the decision object would serve neither consumer.

## Context

STAGE-030's honesty pitch is "we measure, you can check." Two gaps blocked it: the
machine-readable surface was inconsistent (no `--timing`; `--json` only reachable on
`optimize` via `--explain=json`, absent on `web`/`apply`), and the benchmark that
drove the whole taxonomy freeze lived in a since-deleted session scratchpad —
nothing reproducible was committed, yet BENCHMARKS.md (SPEC-083, a launch blocker)
needs exactly that. The audit report **is** what the bench consumes, so the two must
not drift.

## Alternatives Considered

- **Fork a new per-command JSON shape for web/apply**
  - What it is: a bespoke `web.report/v1` distinct from `optimize.explain/v1`.
  - Why rejected: three schemas to keep in sync; the spec explicitly wants ONE
    versioned shape. The auto-decision path is identical across the three verbs, so
    the trace already fits.

- **Bump the schema to `/v2` for the timing field**
  - What it is: version the schema on every additive field.
  - Why rejected: `ssim` (DEC-071) set the precedent that a *gated, additive,
    optional* field keeps `/v1` — existing consumers of a non-`--timing` run see a
    byte-identical object. A version bump would wrongly signal a breaking change.

- **Silently ignore `--json`/`--timing` on the pinned/preserve/plain path**
  - What it is: accept the flag, emit nothing.
  - Why rejected: a flag that silently does nothing is a known footgun in this repo
    (pinned `optimize --verify` already silently ignores its flag). A usage error is
    honest and enforceable.

- **Commit the maintainer's real photos / a large synthetic corpus for the bench**
  - Why rejected: privacy (`no-secrets-in-code` — real EXIF/GPS) and repo bloat
    (detailed photos are incompressible → large files). A small committed corpus +
    an external `--corpus` for the real numbers gives both a committed smoke and
    honest launch figures. The one committed photograph is **CC0 and
    metadata-stripped**, which carries none of that risk.

- **Ship a synthetic-only corpus** *(what SPEC-088 originally built)*
  - Why rejected: the classifier calls every synthetic image `graphic-logo`, so no
    row ever reached AVIF and the `--features avif` build tested nothing. Attempts
    to force it fail on their own terms — adding high-frequency noise bloats the
    file *and* still passes through, and a 3000px synthetic still classifies
    `graphic-logo` (verify measured both). A real photo is the only honest fix, and
    the spec permitted "synthetic generators **and/or** CC0" all along.

- **Send the JSON report to stderr when `-o -` owns stdout**
  - What it is: keep the invocation working by relocating the report.
  - Why rejected: the report's stream would depend on an unrelated flag, so a
    consumer parsing stdout gets silence instead of an error — the same
    silently-wrong class of footgun as an ignored flag. A usage error names the
    problem and the fix.

- **Leave `optimize --explain=json -o -` alone as pre-existing behaviour**
  - Why rejected: it produces a corrupt stream today, and SPEC-088 was about to
    propagate the same collision to `web`/`apply`. Fixing two of three spellings
    would leave the surface inconsistent for no benefit (see *Corrections*).

- **A new Rust bench binary / a new dev-dependency (criterion-style, or a JSON crate)**
  - Why rejected: `no-new-top-level-deps-without-decision`. The spec allows a
    checked-in script; Python 3 stdlib (`json`, `subprocess`, `argparse`) parses our
    valid single-line JSON with zero installs and stays offline. (Chosen: option C.)

- **Option C (chosen):** additive gated `timing` on `/v1` + shared `write_json`
  across the three verbs, and a stdlib Python harness over a small CC0 synthetic
  corpus with `--corpus` for real numbers.

## Corrections to pre-existing behaviour

**`optimize --explain=json` changes from exit 0 to exit 2 whenever the image sink
is stdout** — by an explicit `-o -` **or** by the bare default (no `-o`, no
`--out-dir`). It previously wrote the JSON report and the image to the same stream,
producing output that is neither valid JSON nor a valid image; the pre-spec oracle
binary reproduces this exactly, so it is not a SPEC-088 regression. It is corrected
here anyway, because SPEC-088 adds the same `--json` to `web`/`apply` and the spec's
"stdout stays pipe-clean" criterion is explicit: fixing some spellings would freeze
a known-broken one for symmetry's sake.

**Both doors, one rule.** The guard's first cut keyed on the `-o -` *spelling* and
so missed the default sink — which is the door a user is far more likely to walk
through, since `optimize photo.jpg --json` names no output at all and quietly emits
~126 KB of JSON-then-AVIF at exit 0. Driven on the corpus photo before the fix:
`web … --json` → 126,068 bytes, `optimize … --json` → 126,052 bytes, both exit 0 and
unparseable. The condition now keys on the resolved state ("the image sink is
stdout"), which is the only formulation that closes a door nobody enumerated.

The blast radius is `--explain=json`/`--json` **with a stdout image sink** only — a
combination whose output was already unusable, so no working pipeline can depend on
it. Every other `--explain=json` invocation stays byte-identical (verified against
the oracle), and the working path (`--json -o FILE`, clean parseable JSON on stdout)
is unaffected and named in the error message.

## Consequences

- **Positive:** one consistent, versioned audit report across `optimize`/`web`/
  `apply`; stdout is pipe-clean *by construction* on every spelling, not by
  convention; a skeptic can re-run `just bench` offline and it exercises the AVIF
  flagship; the real-corpus numbers are reproducible without committing private
  photos; non-audit output stays byte-identical (regression-anchored).
- **Negative:** `just bench` depends on a `python3` interpreter (ambient on dev
  machines/CI, but not a Rust toolchain guarantee) and on `--features avif` for the
  release build it benches. The committed corpus grows from ~40 KB to ~200 KB (the
  CC0 photo is 174 KB — the price of a row that actually reaches AVIF). One usage
  error (`--explain=json` into a stdout image sink, by either door) is newly
  returned where a corrupt stream was previously emitted.
- **Neutral:** `just bench` was repurposed from the criterion micro-benches, which
  move to `just bench-micro` (DEC-028's recipe name shifts, not its intent;
  DEC-028's References carry the amendment).
- **Known limit, stated in the output itself:** the committed corpus cannot show the
  `web`-vs-`optimize` story — every image is under `web`'s 2048px downscale bound,
  so the two verbs converge on every row. A >2048px real photo cannot be both
  committed and lean. `scripts/bench.py` prints this as a footer under the table
  (suppressed for `--corpus <real>`), so a skeptic reading only the output is not
  misled into "this tool does nothing". **Every headline number for SPEC-083 must
  come from `--corpus <real>`.**

## Validation

Right if: verify re-runs `just bench` offline and green, a plain run's output
diffs byte-identical to pre-spec, and the three verbs' `--json` share one key set.
Revisit if: a consumer needs timing on the pinned path (promote from usage-error to
a decode/encode-only report), if the Python dependency proves a portability problem
(port the harness to a Rust example that shells out), or if anyone reports a real
pipeline broken by the `-o -` correction (no such pipeline should exist — its
output was already corrupt).

**Corpus claims are verified by driving the tool, never asserted.** Every `class` in
`bench/corpus/README.md` comes from `crustyimg optimize <file> --json`. The corpus
previously shipped files named `photo_*.jpg` that the engine called `graphic-logo`;
they are renamed `gradient_*.jpg`. Revisit the corpus if a future classifier change
moves any row to a different branch — the README table and the bench footer then
state something false, and `just bench` is where that must be caught.

## References

- Related specs: SPEC-088 (this), SPEC-083 (BENCHMARKS, consumes the harness),
  SPEC-084/085/086 (the taxonomy freeze the audit proves).
- Related decisions: DEC-049 (the `optimize.explain/v1` schema), DEC-071 (the `ssim`
  additive-gated precedent), DEC-069/DEC-070 (the fast decision + terminal-optimize
  recipe the verbs share), DEC-028 (the criterion bench recipe, **amended**: renamed
  to `just bench-micro`), DEC-052/DEC-018 (the licensing posture the CC0 corpus
  respects).
- Corpus source: [Petite Hesse spruce forest undergrowth, Waimes, 2023](https://commons.wikimedia.org/wiki/File:Petite_Hesse_spruce_forest_undergrowth,_Waimes,_2023.jpg)
  — Wikimedia Commons, DimiTalen, own work, **CC0-1.0**; license read from the
  Commons API's `extmetadata` (`AttributionRequired = false`), not assumed.
