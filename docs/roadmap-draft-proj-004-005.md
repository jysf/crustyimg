# Tentative breakdown — PROJ-004 (lint) & PROJ-005 (web-asset delivery)

> **DRAFT / provisional — everything here is movable.** Per `AGENTS.md §2`, a project is framed
> for real only once the prior one ships, so the IDs, stage splits, spec lists, and ordering below
> are *candidates to react to*, not commitments. Both projects **depend on PROJ-002's `Analysis`
> layer**; beyond that they're largely independent of each other and freely reorderable. Backed by
> `docs/research/proj-002-design-lint.md`, `docs/research/proj-002-findings.md §3–5`, and the
> SSG + GitHub-Actions adoption research. Snapshot: 2026-07-05. New deps are flagged with license
> (all permissive, pure-Rust, verified).

---

## PROJ-004 — Image lint ("clippy for image assets")

**Thesis:** a source-file, no-URL, deterministic, pass/fail linter for an asset tree — the CI
white space Lighthouse (needs a URL) and `--maxkb` gates (format-blind) can't fill. Folds in the
privacy moat.
**Depends on:** PROJ-002 (`Analysis` + the format-decision engine, for the "could-be-smaller"
rules). **Note:** the *shipped-capability* rules (privacy/orientation/size/corrupt) need only
today's features, so STAGE-A could even start before PROJ-002 fully lands.
**Enables:** the GitHub Action adoption vector (Track B); the "image budget in CI" story.

### Candidate stages

| Stage | Theme | Delivers |
|---|---|---|
| **A — Lint core & shipped-capability rules** | The command + a useful linter on day one | `lint` scaffold (glob/dir input, config discovery, human + JSON output, exit-7) + rules that need only shipped features |
| **B — Engine-backed rules** | The "could be smaller / wrong format" rules | rules that call PROJ-002's format-decision + SSIMULACRA2 probe + the savings-threshold gate |
| **C — CI integration & adoption** | Make it a one-line CI win | SARIF output, the GitHub Action(s), pre-commit hook, docs |

### Candidate specs

**STAGE-A — lint core & shipped-capability rules**
- `lint` command scaffold — clap subcommand, `source::resolve` glob/dir input, non-image skip,
  **exit-7 `CheckFailed` reuse** (DEC-025), human grouped-by-file output. **S–M**
- lint config — `.crustyimg-lint.toml` auto-discovery, ruff-style `select`/`ignore` +
  `per-path-ignores`, eslint-style per-rule severity, per-glob `[[budget]]`, savings-threshold
  defaults (4 KiB / 10%). **M**
- `--json` lint report — hand-rolled (matches `write_json`, no new dep): findings + summary. **S**
- shipped-capability rules — `privacy/gps-metadata-leak`, `privacy/camera-metadata`,
  `orient/orientation-not-baked`, `size/oversized-bytes`, `color/wrong-colorspace`,
  `size/truncated-or-corrupt`, `format/animated-gif`. Each maps to `info`/exif-read/metadata-lane.
  **M** (may split rules into two specs). *No new dep.*

**STAGE-B — engine-backed rules** *(depends on PROJ-002)*
- `format/legacy-format` — equal-SSIMULACRA2 probe → "could be AVIF/WebP, saves N%"; reuses the
  format-decision engine. **M**
- `quality/excessive-jpeg-quality` + `format/indexed-png-opportunity` + the savings-threshold
  gate wiring. **M** *(indexed-png stays advisory until PROJ-007's permissive quantizer ships.)*

**STAGE-C — CI integration & adoption** *(some work lands in separate Action repos)*
- SARIF output — `lint --format=sarif` for GitHub code-scanning (opt-in second tier). **S–M**
- **`setup-crustyimg`** (composite Action: download + checksum + PATH) — likely a **separate
  repo**. **S–M**
- **`crustyimg-action`** lint mode — native `::error file=,line=::` PR annotations + job summary
  + exit-code pass/fail; opt-in optimize/fix mode (commit-back, fork-safe / autofix.ci). Separate
  repo. **M**
- pre-commit hook + CI docs/examples (the format-aware upgrade from `check-added-large-files`). **S**

**Rough size:** ~9 specs across 3 stages → likely a **1.5–2 week** project. New default deps: **none**
(SARIF/JSON hand-rolled; Actions are packaging).

---

## PROJ-005 — Web-asset delivery layer (manifest + placeholders + color + favicon)

**Thesis:** the interface that turns crustyimg's optimized output into *a described, verifiable
asset set* — a machine-readable manifest (+ placeholders, dominant color, favicon) that any SSG /
build / the separate web-content tool consumes without re-probing the images.
**Depends on:** PROJ-002 (`Analysis` gives dominant color for free; `responsive` already computes
the variant data the manifest serializes).
**Enables:** the SSG integrations (Track B: the `eleventy-crustyimg` plugin, the Astro image
service) — all consume the manifest.

### Candidate stages

| Stage | Theme | Delivers |
|---|---|---|
| **A — Manifest core** | The interface | the path-keyed JSON manifest + `--manifest` on `responsive`/`apply`/`optimize` |
| **B — Placeholder & color** | The high-value manifest fields | `placeholder` (blurDataURL + thumbhash/blurhash) + dominant color / palette |
| **C — Favicon set** | The standalone differentiator | `favicon` multi-output (multi-size ICO + Apple/Android PNGs + snippet) |

### Candidate specs

**STAGE-A — manifest core**
- manifest data model + `serde_json` decision — **key by source path** (a map), self-contained per
  entry (variants/srcset/dims/mime), versioned schema (findings §5, refined by the SSG research).
  Promote `serde_json` dev→runtime (permissive) *or* extend hand-rolled JSON — decide here. **S–M**
- `responsive --manifest out.json` — the cheapest entry: `responsive` already computes widths/
  formats/paths/dims; this serializes them. **M**
- `apply --manifest` / `optimize --manifest` — batch manifest emission + merge semantics across
  inputs. **M**

**STAGE-B — placeholder & color**
- dominant color / palette — expose `Analysis.dominant_color` (already computed in PROJ-002) + a
  `palette` command/field via **`kmeans_colors`** (MIT/Apache) for N-colour palettes. **S–M**
- `placeholder` op — **`blurDataURL`** (tiny base64 webp) as the primary interoperable field +
  **thumbhash** (`fast-thumbhash`, MIT) + **blurhash** (`blurhash`, Apache/MIT — never enable
  `gdk-pixbuf`) as extras; a command + a manifest field. **M** *(new deps: fast-thumbhash, blurhash)*

**STAGE-C — favicon set**
- `favicon logo.png` — multi-size **`.ico`** via the **`ico`** crate (MIT) + Apple-touch +
  Android/manifest PNGs + a paste-ready `<link>` snippet; a new multi-output Sink. **M**
  *(new dep: ico; SVG-input favicons wait for PROJ-007's `resvg`.)*

**Rough size:** ~6 specs across 3 stages → likely a **1.5–2 week** project. New default deps:
**serde_json** (promote), **kmeans_colors**, **fast-thumbhash**, **blurhash**, **ico** — all
permissive, pure-Rust, `just deny`-green; each gets its own DEC (the STAGE-008 native-codec DEC
pattern, minus the C-dep concern).

---

## Movability notes (since "they can all be moved around")

- **Both depend on PROJ-002's `Analysis` layer** — that's the one hard ordering constraint.
- **PROJ-004 and PROJ-005 are independent** of each other → either can go first (004 = the CI/
  differentiator story; 005 = the SSG/web-workflow story). Pick by which audience you want to
  activate first.
- **Within PROJ-004:** STAGE-A (shipped-capability rules) has no PROJ-002 dependency and could be
  pulled early or even overlap PROJ-002; STAGE-B needs the format engine.
- **PROJ-005 STAGE-A (manifest) is the linchpin** for all Track-B SSG integration — if the SSG
  story is the priority, front-load it.
- **The GitHub Action (PROJ-004 STAGE-C) is the single highest-leverage adoption move** and could
  be split out and shipped as soon as *any* lint core exists — it doesn't need the full rule set.
- Everything here reuses the shipped `Operation`/`Sink`/recipe + exit-code plumbing; no new
  architecture beyond the manifest Sink and the favicon multi-output Sink.

*Refine or resequence freely; this feeds the real framing of each project when its predecessor ships.*
