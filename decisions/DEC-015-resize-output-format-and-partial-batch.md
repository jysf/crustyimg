---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-015
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-15
supersedes: null
superseded_by: null

affected_scope: ["src/cli/**", "docs/api-contract.md"]

tags:
  - cli
  - ergonomics
  - batch
  - exit-codes
---

# DEC-015: Pixel-command output format defaults to source-format preservation; multi-input batches use a uniform partial-failure exit code 6

## Decision

For pixel-lane CLI commands (starting with `resize`, SPEC-011), the output
format defaults to **preserving the input's source format** when neither an
explicit `--format` nor an `-o <path>` extension dictates otherwise; and a
**multi-input** batch that has ANY per-input failure (load/run/write) exits with
code **6** (partial batch failure) after writing the successes and printing a
per-failure summary to stderr — including the all-fail case. Single-input
failures keep their natural per-error exit code (3/1/4/5), not 6.

## Context

SPEC-011 wires the `resize` command and its sequential multi-input `--out-dir`
fan-out on top of SPEC-010's `Resize` operation. Two behavioral questions had no
prior decision record and are durable (they govern every later STAGE-003 fan-out
command — `thumbnail`/`shrink`/`convert`/`auto-orient`):

1. **Output format default.** The existing `Sink::Dir` (`src/sink/mod.rs`)
   defaults to **PNG** when no format is supplied. Applied naively, `resize
   a.jpg --max 800 --out-dir d` would write `d/a.png` — silently changing the
   container format. That violates the `ergonomic-defaults` constraint (the
   simple case should "just work"); a user resizing JPEGs expects JPEGs out.
2. **Partial batch semantics.** `docs/api-contract.md` reserves exit code **6**
   for "partial batch failure (some inputs failed; summary on stderr)", but no
   exit-6 path existed in the binary, and the all-fail and single-input cases
   were unspecified.

Constraints in play: `ergonomic-defaults` (warning), `untrusted-input-hardening`
(the Sink's traversal/overwrite guards must still apply per input),
`no-unwrap-on-recoverable-paths` (failures are typed, never panics),
`no-async-runtime` (the fan-out is sequential — DEC-006; rayon is STAGE-005).

## Alternatives Considered

- **Option A: Keep the `Sink::Dir` PNG default.**
  - What it is: let the fan-out write PNG for every input unless `--format` is
    given.
  - Why rejected: silently converts `a.jpg → a.png`; surprising and lossy;
    violates `ergonomic-defaults`. Forces a `--format` flag for the common case.

- **Option B (chosen for format): Per-input source-format preservation.**
  - What it is: resolve the output format PER INPUT — `--format` wins, else the
    `-o <path>` extension, else `img.source_format()` (preserve). Implemented by
    passing `format: Some(resolved)` to the existing `Sink` variants inside the
    fan-out loop; no Sink change.
  - Why selected: the simple case "just works" (`resize *.jpg --out-dir web/`
    keeps JPEGs); mixed batches keep each input's own format; `--format` remains
    the explicit override; reuses the Sink as-is.

- **Option C: All-or-nothing batch (abort + exit non-zero on first failure).**
  - What it is: stop the batch at the first failing input, leaving partial
    output, exit 1.
  - Why rejected: a single bad file in a large directory shouldn't discard the
    work already done; the api-contract explicitly reserves a distinct
    partial-failure code (6) precisely to support "continue and report".

- **Option D (chosen for batch): Continue-and-report, uniform exit 6.**
  - What it is: process every resolved input sequentially; on a per-input
    load/run/write error, print `error: <input>: <reason>` to stderr and keep
    going; after the loop, if any failed, exit 6 (all-fail included). Resolution
    errors (missing path / empty glob) on the initial resolve pass remain hard
    exit-3 errors. Single-input failures keep their natural code (3/1/4/5).
  - Why selected: matches the api-contract's documented exit 6; a uniform code
    keeps scripting predictable (`$? == 6` ⇒ "some/all batch items failed, check
    stderr"); distinguishing all-fail with a different code adds complexity for
    no scripting benefit.

## Consequences

- **Positive:** Ergonomic default (format preserved); predictable, documented
  batch failure code reused by every later STAGE-003 fan-out command; no Sink or
  library change required (format threaded via the existing `format: Some(_)`
  field; the `{ext}` template token already derives from the chosen format).
- **Negative:** "Preserve" means the pixel lane drops container metadata on
  re-encode (the Sink re-encodes via the `image` crate) — preserving *format* is
  not preserving *metadata*; genuine metadata preservation needs the STAGE-004
  container lane. Exit 6 conflates "1 of 100 failed" and "100 of 100 failed";
  the stderr summary disambiguates for humans.
- **Neutral:** `-q/--quality` is not honored by `resize` yet (encoder default);
  quality-aware encode is the `shrink`/`convert` story. `-o -` (stdout) on an
  undeterminable source still needs `--format`.

## Validation

Right if: users resizing JPEGs/PNGs to a directory get the same format back
without a flag (no surprise conversions), and batch scripts can rely on exit 6
to mean "inspect stderr for per-file failures". Revisit if: the metadata lane
(STAGE-004) makes true default-preserve achievable (the format-preservation
policy then composes with metadata carry-over), or if users want a distinct
all-fail code.

## References

- Related specs: SPEC-011 (this decision's first implementation), SPEC-010
  (the `Resize` op it builds on), SPEC-005 (the `Sink`).
- Related decisions: DEC-003 (metadata dual-lane — why format-preserve ≠
  metadata-preserve), DEC-007 (typed errors → exit codes), DEC-006 (no async /
  rayon-for-batch — fan-out is sequential here), DEC-014 (op-params construction
  path the CLI shares).
- External docs: `docs/api-contract.md` (Exit Codes table; the `resize` entry).
