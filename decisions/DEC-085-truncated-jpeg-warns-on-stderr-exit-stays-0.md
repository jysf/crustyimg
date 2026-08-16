---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-085
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-07-29
supersedes: null
superseded_by: null

affected_scope:
  - "src/image/mod.rs"
  - "src/cli/report.rs"
  - "src/cli/ops.rs"
  - "src/cli/optimize.rs"
  # SPEC-116 made `build` the second place the unconditional-gating rule is
  # ENFORCED, not merely referenced. Without these globs `decisions-audit
  # --changed` stays silent on a PR that touches only `build.rs` — which is
  # exactly the "let's make these two warnings consistent" tidy-up DEC-085
  # exists to stop. Found by SPEC-116's verify via the audit tool's own output.
  - "src/cli/build.rs"
  - "tests/build.rs"
  - "tests/hostile_inputs.rs"
  - "tests/fixtures/hostile/"
  - "docs/api-contract.md"

tags:
  - jpeg
  - truncation
  - hostile-input
  - exit-codes
  - untrusted-input-hardening
---

# DEC-085: a truncated JPEG warns on stderr; exit stays 0

## Decision

A truncated JPEG (missing the trailing end-of-image marker, `FF D9`) prints a one-line
`warning: <input>: truncated JPEG: …` to stderr on every verb that decodes it
(`info`/`web`/`convert`/`resize`/`optimize`), but **still exits 0** and still writes
whatever output the pipeline produces from the partial decode.

Detection is a **container-level** check on the already-in-memory input bytes — `!bytes
.ends_with(&[0xFF, 0xD9])`, run once at decode time in [`Image::from_bytes`]
(`src/image/mod.rs`) and cached on the `Image` as `truncated_jpeg: bool`, exposed via
`pub(crate) fn is_truncated_jpeg(&self)`. It is **not** a decoder change: `image` 0.25's
JPEG decoder is untouched, and the check runs beside the existing decode, not inside it
(DEC-003's container lane). The warning text is centralized as
`pub(crate) const TRUNCATED_JPEG_WARNING` so all call sites print identical wording.

The warning is printed **unconditionally** — not gated behind `--quiet`, unlike this
crate's other CLI advisories (e.g. the auto-quality "could not reach target" warnings in
`src/cli/optimize.rs`). See "Why" below.

## Why

**The finding (F1, SPEC-107 design cycle).** Driving a truncated JPEG through the CLI at
`f4c9d22` showed exit 0 with **empty stderr** on `info`/`web`/`resize` — the user is
handed a partially-grey image and never told. Truncated PNG and truncated AVIF both error
correctly (`could not decode image: …`, exit 1); JPEG is the outlier because the `image`
crate's JPEG decoder tolerates truncation by design (it decodes whatever entropy data is
present and returns a frame, rather than erroring on a missing EOI). This is the one
finding that fails STAGE-035's own launch-gating bar ("a clear message on every input"),
and it is on the flagship `web` path.

**Two alternatives were considered and rejected:**

- **Exit 1 (treat it like PNG/AVIF).** Rejected: this is a **frozen exit-code surface**
  (STAGE-030/DEC-0xx CLI freeze) and JPEG's tolerance-of-truncation is not a crustyimg
  bug — it is the same leniency every image viewer, browser, and OS thumbnailer has
  shipped for decades. Turning a JPEG that opens fine everywhere else into a hard failure
  here would break a workflow that works everywhere, for no correctness gain (the decoder
  is not producing wrong pixels, it is producing INCOMPLETE ones, which the user can now
  see in the warning and decide about themselves).
- **Document-only (no code change).** Rejected: this would close a launch-gating item
  (the browser's "hostile input" confirmation, which the native pass mirrors) while
  knowingly leaving a **silent** corruption path on the flagship `web` verb. A stated
  gap is honest; a known-and-unstated one is not.

**Why unconditional (not `--quiet`-gated). Corrected on the SPEC-107 punch-list pass — this
crate's other `--quiet`-gated advisories are not merely cosmetic, which makes the
departure below more notable, not less.** `DEC-023`'s downscale warning
("scaled to WxH to fit the N budget") is explicitly recorded as "surprising if
unnoticed," and `DEC-075`'s larger-than-source `note:` exists precisely to explain an
unexpected output property (the shipped bytes are bigger than the source) — both are
`--quiet`-gated anyway, because a script that opted into `--quiet` is presumed to also
have opted out of caring about a shape/size deviation it can still discover by
inspecting its own output. F1 is different in kind, not merely in degree: the shipped
bytes are **wrong** (an incomplete decode passed off as a complete one), not merely a
different size or shape than expected, and there is no downstream signal (file size,
dimensions) a script could check instead to notice it. Gating it behind `--quiet` would
let a scripted/batch pipeline (`-Q`, the exact context where a human is least likely to
notice) reintroduce the very silent-corruption failure mode this decision closes. The
top-level `error:` line (`src/cli/mod.rs::run`) is likewise never `--quiet`-suppressed,
for the same reason — this warning is closer to that than to the other advisories.

## Consequences

- **Positive:** the launch-readiness "hostile/edge inputs hold natively" claim is now
  true without caveat — no input, decodable or not, silently loses data on the CLI's
  flagship path.
- **Negative:** a truncated-but-decodable JPEG piped through a script that greps stderr
  for `error:` (not `warning:`) will not notice unless it also checks stderr is
  non-empty. This is the intended, minimal-surprise behavior (exit code is the load-
  bearing signal on this frozen surface); `docs/api-contract.md` documents the warning
  on the affected verbs so a caller can grep for it if it matters to them.
- **Neutral:** `Image` gained one `bool` field and one `pub(crate)` accessor; no public
  API changed. `optimize_decide_one` (`src/cli/optimize.rs`) gained a 4th return-tuple
  element (the flag) rather than printing internally, preserving its existing "does NOT
  print" contract — the caller (`run_optimize_autodecide`) prints against the label it
  already has.
- Committed a reusable hostile-input regression harness (`tests/hostile_inputs.rs`,
  `tests/fixtures/hostile/`) as a side effect of proving this fix — see SPEC-107.
