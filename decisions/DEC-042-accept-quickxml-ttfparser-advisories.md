---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-042
  type: decision
  confidence: 0.8
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

created_at: 2026-07-03
supersedes: null
superseded_by: null

affected_scope:
  - deny.toml

tags:
  - supply-chain
  - security
  - advisories
  - cargo-deny
  - risk-acceptance
---

# DEC-042: accept (with revisit) three RustSec advisories that have no available fix

## Decision

Add three documented `ignore` entries to `deny.toml` `[advisories]`, each with a reason
and a revisit trigger (mirroring the existing `RUSTSEC-2024-0436` `paste` entry and the
`no unexplained ignores` discipline from DEC-037), so `cargo deny check advisories` is
green again:

- **RUSTSEC-2026-0194** and **RUSTSEC-2026-0195** — `quick-xml 0.37.5` (quadratic
  start-tag duplicate-attribute check; unbounded namespace-declaration allocation /
  memory-DoS in `NsReader`). Transitive via **`little_exif`**.
- **RUSTSEC-2026-0192** — `ttf-parser 0.25.1` is **unmaintained** (author-declared EOL).
  Transitive via **`ab_glyph` → `owned_ttf_parser`**.

No code, no dependency change — only the three tracked exceptions.

## Context

These advisories appeared in the RustSec DB after SPEC-041 shipped (the advisory DB is
time-varying — a green `cargo deny` run can go red days later with zero code change).
They broke the `supply-chain (cargo-deny)` CI gate on `main`, blocking SPEC-042 and the
`v0.1.0` release. **Neither has an available fix**, so the only responses are a
documented ignore or dropping the dependency; dropping `little_exif` (the EXIF-write
lane, DEC-029) or `ab_glyph` (the watermark rasterizer, DEC-032) is disproportionate.

### Reachability / risk assessment

**`quick-xml` (0194 / 0195) — the two actual vulnerabilities:**
- They live in `quick-xml`'s **XML reader** (namespace resolution + start-tag attribute
  parsing). `little_exif` uses `quick-xml` for **XMP** (the XML-based metadata packet).
- crustyimg drives `little_exif` only through its **binary-EXIF** APIs (`Metadata`,
  `ExifTag`, `ExifTagGroup`, `Endian`) for `set` (artist/copyright/description) and
  `clean --gps` — EXIF is a binary TIFF-IFD structure, not XML. **crustyimg has zero
  XMP/XML handling anywhere in `src/`** (verified by grep). So crustyimg does not drive
  the vulnerable XML-reader path with untrusted input.
- Defence-in-depth: all image/recipe inputs are already bounded by the STAGE-006
  hardening (decode `image::Limits`, recipe size/step caps, resize output cap —
  DEC-034/036/038), so even a hypothetically-reached allocation is capped, not unbounded.
- **No upstream fix path:** `little_exif 0.6.23` (the latest) pins `quick-xml ^0.37`; the
  fixed `quick-xml ≥ 0.41` is semver-incompatible and cannot be forced (`cargo update
  --precise 0.41.0` is rejected). A `[patch]` to a backported 0.37 fork is heavier than
  the residual risk warrants.
- Confidence is **high but not certified** against `little_exif`'s internals (hence
  DEC confidence 0.8) — the residual is that some `little_exif` read path might touch XMP
  incidentally; the bounded-input mitigation covers that.

**`ttf-parser` (0192) — unmaintained, informational (no vulnerability):**
- Reached via `ab_glyph`/`owned_ttf_parser` for watermark-text glyph rasterization
  (SPEC-030/DEC-032). Font input is the **bundled BSD-3 Go font** or an **explicit
  `--font PATH`** the user chooses — not untrusted/network input. No fix exists (EOL).

## Alternatives Considered

- **Upgrade `quick-xml` to ≥ 0.41** — blocked: `little_exif 0.6.23` pins `^0.37`, no
  newer `little_exif` exists. Rejected (not possible without upstream action).
- **Replace `little_exif`** (EXIF-write lane) — disproportionate to the residual risk;
  it is the only pure-Rust read+write EXIF crate (DEC-029). Revisit only if the risk
  changes.
- **Replace `ab_glyph`/`ttf-parser`** — the watermark rasterizer choice (DEC-032);
  swapping it for an unmaintained-informational advisory is not warranted now.
- **Switch the `cargo deny` advisories check from `deny` to `warn`** — rejected: that
  would silently swallow *future* real advisories too. The per-advisory documented
  ignore keeps the gate strict while accepting these three explicitly.
- **`cargo update`/patch fork of quick-xml 0.37** — heavier than the residual risk; the
  vulnerable path is not on crustyimg's API surface.

## Consequences

- **Positive:** `cargo deny check advisories` is green again; `main` and SPEC-042/`v0.1.0`
  are unblocked. Each acceptance is explicit, dated, and revisit-tracked — the gate stays
  strict for everything else (yanked/other advisories still `deny`).
- **Negative:** crustyimg ships with three known-open advisories. Two are *vulnerabilities*
  we assess as not-reachable-with-untrusted-input + bounded; one is informational
  (unmaintained). This is accepted risk, documented here, not a silent suppression.
- **Neutral:** No runtime/code change. Revisit triggers: `little_exif` bumps `quick-xml`
  (or we replace it) → drop 0194/0195; `ab_glyph` moves off `ttf-parser` (or it becomes
  a real vuln) → drop/act on 0192. Also worth a periodic recheck since the advisory DB
  drifts — consider surfacing `cargo deny` status on doc-only pushes too.
