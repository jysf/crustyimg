---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-004                       # stable, zero-padded, never reused
  status: active                     # proposed | active | shipped | cancelled
  priority: high                     # critical | high | medium | low
  target_ship: null                  # optional: YYYY-MM-DD

repo:
  id: crustyimg                      # must match .repo-context.yaml

created_at: 2026-07-06
shipped_at: null

# Business value. Testable claim, not marketing copy.
value:
  thesis: >
    `crustyimg lint` — "clippy for image assets": a source-file, no-URL, deterministic,
    pass/fail linter you run on an `assets/`/`content/` tree in CI, BEFORE anything is
    deployed. It occupies the white space Lighthouse (needs a live URL + a rendered page)
    and generic `--maxkb` git gates (format-blind) can't fill: pre-deploy, format-aware,
    per-file, with an exit code. It reuses everything the MVP + optimization engine already
    ship — `info`/EXIF read, the format-decision engine, the SSIMULACRA2 probe, the metadata
    lane — and folds in the privacy moat (a GPS-leak rule no browser tool can run on a source
    file). It is the project's defining CI-adoption differentiator.
  beneficiaries:
    - Web/content teams gating an asset tree in CI/pre-commit before deploy (the "image budget
      in CI" story) — no service, no URL, no Node
    - Privacy-conscious publishers who must not ship location/camera EXIF in public assets
    - The maintainer — `lint` + a GitHub Action is the single highest-leverage adoption vector
  success_signals:
    - "`crustyimg lint content/` on a clean tree exits 0; a tree with a GPS-leaking photo or an
      over-budget asset exits 7 (the existing `CheckFailed` code) with grouped, per-file findings"
    - "each finding names a **runnable `crustyimg` fix command** (e.g. `optimize --format webp`,
      `clean --gps`) — the linter tells you exactly how to fix it"
    - "zero-config works (default rules at default severities); `.crustyimg-lint.toml` tunes
      select/ignore, per-rule severity, and per-glob byte budgets (ruff/eslint conventions)"
    - "`lint --format json` emits a machine-readable report (hand-rolled, no new dependency) for
      CI tooling; a `format/legacy-format` finding proves ≥ savings-threshold via a real
      SSIMULACRA2-equal probe, so it is quiet enough to leave on"
    - "the default build stays pure-Rust / zero-system-deps; `just deny` green; **no new default
      dependency** (lint is composition over shipped capabilities + hand-rolled output)"
  risks_to_thesis:
    - Noise: "could be smaller" rules that fire on trivial gains make teams disable the linter →
      mitigated by the savings-threshold gate (default 4 KiB / 10%, Lighthouse's own floor) and the
      3-severity model (only `error`, and `warn` under `--max-warnings`, affects exit code)
    - The engine-backed rules (`legacy-format`, `indexed-png`) reuse `optimize`'s per-candidate
      solve, which lives in the CLI today → may need a small shared seam exposed (noted in STAGE-014)
    - Suggesting `--format avif` in a build without the AVIF feature would be wrong → a license/
      capability guard degrades the suggestion to a built codec (WebP) or drops it
    - Adoption ultimately needs the GitHub Action, which lives in a **separate repo** (Track B) —
      in-repo PROJ-004 ships the binary + exit code + SARIF + a pre-commit hook; the Action wraps it
---

# PROJ-004: image lint — "clippy for image assets"

## What This Project Is

The wave that turns crustyimg from a set of *fixing* commands into a *checking* tool. `crustyimg
lint [PATHS]…` walks an image asset tree, runs a catalog of deterministic, source-file rules
(no URL, no rendered page), and reports findings with a severity and a **runnable fix command** —
then exits `0` (clean) or `7` (findings that fail the gate). It is `clippy`/`eslint`/`ruff` for
images: zero-config by default, tunable via `.crustyimg-lint.toml`, human- or JSON-formatted, and
built to leave on in CI.

Crucially it is **composition over shipped capabilities** — no new engine. The rule catalog maps
1:1 onto things crustyimg already does: `info`/EXIF read (dims, format, has_alpha, has_icc,
has_exif, Orientation, GPS), the metadata lane (the *fix* for privacy rules), and — for the
"this could be smaller / wrong format" rules — the PROJ-002 format-decision engine + the
SSIMULACRA2 probe. It reuses the shipped `source::resolve` fan-out and the exit-7 `CheckFailed`
code (DEC-025, whose comment already anticipated "reusable by the future EXIF audit-linter").

## Why Now

- **It's the defining CI differentiator, and it's unblocked.** Lighthouse's four image audits and
  Lighthouse-CI budgets all run *in-browser against a deployed page* — they need a URL and a
  layout. Generic `check-added-large-files` git gates are format-blind. crustyimg owns the gap:
  **pre-deploy, no-URL, format-aware, per-file, pass/fail**. PROJ-002 shipped the last missing
  piece (the format engine), so the whole rule catalog is now buildable.
- **The privacy moat is real and browser-invisible.** GPS/camera EXIF is stripped by CDNs before a
  browser ever sees it, so no in-browser tool can flag "this public asset leaks a location." A
  source-file linter can — and crustyimg already has the EXIF-read + `clean --gps` fix.
- **It's the adoption vector.** One binary + an exit code is a one-line CI win (the deliberate
  contrast with Lighthouse-CI's server/URL). It's the on-ramp for the GitHub Action + the "image
  budget in CI" story (`docs/research/proj-002-design-lint.md`, roadmap Track B).

## Success Criteria

- `crustyimg lint content/` runs zero-config, groups findings by file, gives each a severity and a
  runnable `crustyimg` fix, and maps its exit code to the gate: `0` clean · `7` ≥1 error (or
  warnings over `--max-warnings`) · `2` bad usage/config · `3` no inputs. `info` findings never
  fail.
- The privacy rule `privacy/gps-metadata-leak` (error) catches GPS EXIF in a public asset and its
  fix is `clean --gps`; the correctness rule `orient/orientation-not-baked` catches a non-baked
  Orientation with fix `auto-orient`.
- `format/legacy-format` (warn) fires only when a real equal-SSIMULACRA2 probe proves a modern
  format saves ≥ the savings threshold (default 4 KiB / 10%) — quiet enough to leave on; and it
  never suggests a codec the running binary can't produce.
- `.crustyimg-lint.toml` (auto-discovered) tunes `select`/`ignore`, per-rule severity, per-glob
  byte budgets, and `per-file-ignores`; `lint --format json` emits a stable hand-rolled report.
- Default build stays pure-Rust / zero-system-deps; `cargo deny check` green; **no new default
  dependency**; rule ids are a documented stability surface (DEC-050).

## Scope

### In scope
- **STAGE-013 — Lint core & shipped-capability rules:** the `lint` command scaffold (source
  resolution, the `Rule`/`Finding`/`Severity` framework, human grouped-by-file output, exit-7),
  the `.crustyimg-lint.toml` config, the hand-rolled JSON report, and every rule that needs only
  *shipped* capabilities (privacy/orientation/size/dimensions/colorspace/corrupt/animated-gif).
  A useful linter on day one; **no PROJ-002 dependency for this stage's rules**.
- **STAGE-014 — Engine-backed rules:** the "could be smaller / wrong format" rules that call the
  PROJ-002 format-decision engine + the SSIMULACRA2 probe behind the savings-threshold gate
  (`format/legacy-format`, `quality/excessive-jpeg-quality`, `format/indexed-png-opportunity`).
- **STAGE-015 — CI integration & adoption (in-repo parts):** SARIF output (`--format sarif`),
  a `pre-commit` hook, a `just lint-images` recipe, and CI docs/examples. Ships **0.4.0**.

### Explicitly out of scope
- **The GitHub Actions themselves** (`setup-crustyimg`, `crustyimg-action` with PR annotations /
  commit-back autofix) — they live in **separate repos** (Track B). In-repo PROJ-004 ships the
  binary, exit code, SARIF, and the pre-commit hook the Actions wrap.
- **Auto-fixing pixels.** `lint` is read-only and advisory — it never writes an image. Every
  finding *names* a fix command; running it is the user's (or a separate Action's) choice.
- **A URL/page/runtime check** (Lighthouse's `offscreen-images`, DPR-aware responsive) — no source
  meaning; explicitly not implemented.
- **Indexed/lossy-PNG as a *fix*** — `format/indexed-png-opportunity` stays advisory until a
  permissive quantizer lands (PROJ-007); interim the suggestion is lossless WebP.
- **Perceptual near-duplicate detection** (`image_hasher`) — a v2/opt-in rule with a new dep;
  deferred.

## Stage Plan

> Framed 2026-07-06 after PROJ-002 shipped. Execution order per the revised roadmap:
> PROJ-002 → **PROJ-004 (this)** → PROJ-003 (planner) → PROJ-005. IDs are stable; drive by status.

Format: `- [status] STAGE-ID — one-line summary`

- [~] STAGE-013 (active, NEXT) — Lint core & shipped-capability rules: the `lint` scaffold +
  config + JSON report + every rule that needs only shipped features. A useful linter on day one.
- [ ] STAGE-014 (proposed) — Engine-backed rules: `legacy-format` / `excessive-jpeg-quality` /
  `indexed-png-opportunity` via the PROJ-002 engine + SSIMULACRA2 probe + savings-threshold gate.
- [ ] STAGE-015 (proposed) — CI integration & adoption: SARIF output, pre-commit hook,
  `just lint-images`, CI docs. Ships 0.4.0. (The GitHub Actions are separate repos.)

**Count:** 0 shipped / 1 active / 2 pending

## Dependencies

### Depends on
- **PROJ-001 (shipped MVP)** — `source::resolve` (glob/dir/file fan-out), `run_info`/`InfoReport`
  (dims/format/color_type/bit_depth/has_alpha/has_icc/has_exif), the EXIF read (`kamadak-exif`:
  Orientation + GPS), the metadata lane (`strip`/`clean --gps` — the *fixes*), the hand-rolled
  `write_json`/`escape_json` pattern, and **DEC-025's exit-7 `CheckFailed`** (its comment already
  anticipated the audit-linter).
- **PROJ-002 (shipped engine)** — the `Analysis` layer + `src/analysis/decide.rs` format-decision
  engine + the SSIMULACRA2 quality search, for STAGE-014's "could-be-smaller" rules only.
- DEC-004 (codec gating — the license/capability guard for format suggestions), DEC-019 (perceptual
  probe), DEC-020/021/022 (which codecs a suggestion may name).
- External: **no new default dependency** (JSON + SARIF hand-rolled; the near-dup `image_hasher`
  rule is deferred v2).

### Enables
- **The GitHub Action adoption vector** (Track B, separate repos) — wraps the `lint` binary + exit
  code + SARIF for PR annotations and an optional commit-back autofix mode.
- **PROJ-005 (web-asset manifest)** — independent, but shares the report/output conventions.

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
