---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.

stage:
  id: STAGE-013                     # stable, zero-padded within the project
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null

project:
  id: PROJ-004                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-06
shipped_at: 2026-07-06

value_contribution:
  advances: >
    Delivers a useful linter on day one and the framework everything else plugs into: the
    `lint` command, the `Rule`/`Finding`/`Severity` model, config discovery, human + JSON
    output, exit-7, and every rule that needs only ALREADY-SHIPPED capabilities — the privacy
    moat included. No PROJ-002 dependency for this stage's rules.
  delivers:
    - "`crustyimg lint [PATHS]…`: source-resolution fan-out (reuse `source::resolve`), non-image
      skip, a `Rule` trait + registry, `Finding { file, rule, severity, message, fix }`, human
      grouped-by-file output, and exit-code mapping (0/7/2/3) reusing `CheckFailed` (DEC-025)"
    - "`.crustyimg-lint.toml` auto-discovery + ruff/eslint-style `select`/`ignore`/per-rule
      severity / per-glob byte budgets / `per-file-ignores`; zero-config works"
    - "`lint --format json` — a stable, hand-rolled report (no new dependency)"
    - "the shipped-capability rules: `privacy/gps-metadata-leak` (error) + `privacy/camera-metadata`,
      `orient/orientation-not-baked`, `size/oversized-bytes`, `dims/oversized-dimensions`,
      `color/wrong-colorspace` + ICC rules, `format/animated-gif`, `size/truncated-or-corrupt`"
  explicitly_does_not:
    - Implement the engine-backed "could-be-smaller" rules (STAGE-014) — those need the format
      engine + SSIMULACRA2 probe
    - Write any image (read-only + advisory; every finding names a fix command, never runs it)
    - Add a new default dependency, SARIF output, or the GitHub Action (STAGE-015 / separate repos)
---

# STAGE-013: lint core & shipped-capability rules

## What This Stage Is

The foundational stage of PROJ-004: the `crustyimg lint` command itself, the rule framework, the
config surface, the report formats, and the rule catalog that needs only capabilities the MVP
already ships. When this stage lands, `crustyimg lint content/` is a genuinely useful CI check —
it catches GPS leaks, non-baked orientation, over-budget and over-sized assets, corrupt files, and
wrong colorspaces, with a runnable fix per finding and a pass/fail exit code — **before** any
engine-backed cleverness. It is the scaffold STAGE-014's engine rules plug into.

## Why Now

- **It's immediately useful and fully unblocked.** Every rule here maps to a shipped capability
  (`info`/EXIF read + the metadata lane), so the stage has no PROJ-002 dependency and delivers CI
  value on its own.
- **It's the privacy moat.** `privacy/gps-metadata-leak` is the browser-invisible check that
  justifies a source-file linter; the fix (`clean --gps`) already ships.
- **It sets the contract.** The rule-id catalog, the 3-severity model, exit-7 reuse, config
  discovery, and the report schema are a stability surface — pinned in DEC-050 so later rules just
  register.

## Success Criteria

- `crustyimg lint content/` resolves inputs via `source::resolve`, skips non-images, runs the
  default rule set, prints grouped-by-file findings with a runnable fix each, and maps its exit
  code: `0` clean · `7` ≥1 error (or warnings > `--max-warnings`) · `2` usage/bad config · `3` no
  inputs. `info`-severity findings never fail the gate. A decode failure is a *finding*
  (`size/truncated-or-corrupt` → 7), not an abort.
- Zero-config works; `.crustyimg-lint.toml` (auto-discovered walking up to the repo root) tunes
  `select`/`ignore`, per-rule severity, per-glob byte budgets, and `per-file-ignores`.
- `lint --format json` emits a stable hand-rolled report; `just deny` green with **no new
  dependency**; determinism (no network/mtime/wall-clock in a finding).

## Scope

### In scope
- The `lint` command scaffold: source resolution, the `Rule` trait + registry, `Finding`/
  `Severity`, human output, exit-7 (`CheckFailed`, DEC-025). Includes 2 foundational rules to
  prove the framework end-to-end. **(SPEC-050)** — the first build target; DEC-050 lands here.
- `.crustyimg-lint.toml` config: auto-discovery, `select`/`ignore`, per-rule severity, per-glob
  `[[budget]]`, `per-file-ignores`, savings-threshold defaults, `--no-config`/`--select`/
  `--ignore`/`--max-warnings` CLI. **(SPEC-051)**
- `lint --format human|json`: the hand-rolled JSON report + the human render (fix line = runnable
  command; potential-savings summary). **(SPEC-052)**
- The remaining shipped-capability rules: `privacy/camera-metadata`, `orient/orientation-not-baked`,
  `size/oversized-bytes`, `dims/oversized-dimensions`, `color/wrong-colorspace` + `missing-icc`/
  `unexpected-icc`, `format/animated-gif`. **(SPEC-053)**

### Explicitly out of scope
- Engine-backed rules (STAGE-014). SARIF / Actions / pre-commit (STAGE-015).
- Any image write (read-only). A `classify`-style surface (not a lint concern).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-050 (shipped on 2026-07-06) — `lint` command core: `source::resolve` fan-out + non-image
  skip + `Rule`/`Finding`/`Severity` framework + human grouped-by-file output + exit-7
  (`CheckFailed`, DEC-025) + 2 foundational rules (`privacy/gps-metadata-leak`,
  `size/truncated-or-corrupt`). DEC-050 landed here. PR #59 → `main` (14e425b).
- [x] SPEC-051 (shipped on 2026-07-06) — `.crustyimg-lint.toml` config: auto-discovery + `select`/`ignore` +
  per-rule severity + per-glob byte budgets + `per-file-ignores` + savings-threshold defaults +
  the config/severity CLI flags. PR #60 → `main` (236581e). Budget plumbing landed; SPEC-053
  inherits the end-to-end budget→size-finding test.
- [x] SPEC-052 (shipped on 2026-07-06) — `lint --format json` (hand-rolled, no new dep) + the human report
  refinements (runnable-fix line, savings summary). PR #61 → `main` (d903b2e).
- [x] SPEC-053 (shipped on 2026-07-06) — the remaining shipped-capability rules (camera-metadata, orientation,
  oversized-bytes, oversized-dimensions, colorspace + ICC, animated-gif). PR #62 → `main` (ec374d6).
  The per-glob budget → `size/oversized-bytes` end-to-end test landed here.

**Count:** 4 shipped / 0 active / 0 pending

## Design Notes

- **Layering:** `src/lint/` is a new peer module. Rules read `InfoReport`/EXIF + raw bytes; they
  NEVER write images. The command lives in `src/cli/` (like other subcommands); the rule engine +
  finding types live in `src/lint/`. No `Operation`/recipe involvement — lint is orthogonal to the
  pixel pipeline.
- **Exit codes (DEC-025):** reuse `CliError::CheckFailed` (exit 7). Its source comment already
  says "reusable by the future EXIF audit-linter" — this is that. Decode failure is a *finding*,
  not exit 1 (the one deliberate divergence: a linter reports a broken asset, it doesn't abort).
- **Determinism guard:** no network, no mtime, no wall-clock in a finding — same tree ⇒ same report.
- Weighty decision → DEC-050: the command shape, the rule-id catalog as a stability surface, the
  3-severity model, exit-7 reuse, the config-discovery order, and the savings-threshold defaults.

## Dependencies

### Depends on
- STAGE-001–006 (PROJ-001) — `source::resolve`, `run_info`/`InfoReport`, EXIF read (Orientation +
  GPS), the metadata lane (the fixes), the hand-rolled JSON pattern, DEC-025 (exit-7).

### Enables
- STAGE-014 (engine-backed rules) — plug into this framework.
- STAGE-015 (SARIF / pre-commit / Actions) — wrap this command's exit code + report.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** Yes. `crustyimg lint content/` is a working
  CI linter: it resolves an asset tree via `source::resolve`, skips non-images, runs the rule catalog,
  prints grouped-by-file findings each with a runnable fix, and maps `0`/`7`/`2`/`3` exit codes
  reusing `CheckFailed` (DEC-025). Zero-config works; `.crustyimg-lint.toml` tunes select/ignore,
  per-rule severity, per-glob budgets, and per-file-ignores; `--format json` emits a stable hand-rolled
  report. The privacy moat (`privacy/gps-metadata-leak`) and a decode-failure-is-a-finding rule are in.
  No new default dependency; `just deny` green throughout.
- **How many specs did it actually take?** 4, exactly as planned (SPEC-050 core → 051 config → 052 JSON
  → 053 rules). No specs added or dropped; one integration test moved 051→053 (its consuming rule).
- **What changed between starting and shipping?** Two contracts got concretely pinned mid-build that the
  specs left open: lint reuses the **global `--format`** flag (avoiding a clap duplicate-arg conflict),
  and **opt-in rules** are realized via a new `Rule::default_enabled()` + a config enable rule.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Testing conventions (§12): note that some formats have no native pure-Rust encoder (CMYK JPEG,
    embedded ICC), so their fixtures are hand-built byte splices or the detector is helper-tested.
  - A newly-published RUSTSEC advisory can turn `just deny` red mid-build independent of your diff;
    sync `main` (which may already carry the hygiene bump) before assuming the failure is yours.
  - A single-parse cache for a shared external read (here `ExifFacts` over `kamadak-exif`) belongs in
    the framework spec, not retrofitted in the rules spec.
