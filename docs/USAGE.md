# How to use this template

A practical, end-to-end walkthrough. The README covers setup and the two
variants; this goes deeper into the daily loop. If you're brand new,
`just init` then open `GETTING_STARTED.md` first — this doc is the
reference you come back to.

## The mental model

```
Repo (the app — persists forever)
 └─ Project (a wave of work: "MVP", "v2", "redesign")
     └─ Stage (a coherent chunk: 2–5 per project)
         └─ Spec (one implementable task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

The repo *is* the app and outlives every project. Architecture,
conventions, constraints, and decisions accumulate at repo level
(`docs/`, `guidance/`, `decisions/`) so they survive across waves of
work. Projects are bounded; specs are individual tasks; the cycle is the
five phases each spec moves through.

## Day 0 — initialize

```bash
just init          # pick claude-only or claude-plus-agents; scaffolds the root
```

Then describe the app once in `.repo-context.yaml`, and skim
`AGENTS.md` — it's the single source of truth every agent reads.

## Start a project, a stage, a spec

A **project** is a brief. Copy `projects/_templates/project-brief.md`
into `projects/PROJ-NNN-<slug>/brief.md` and fill in the value thesis
(see `GETTING_STARTED.md` and Prompt 1a in `FIRST_SESSION_PROMPTS.md`).

A **stage** and a **spec** are scaffolded for you, with IDs and
front-matter filled in:

```bash
just new-stage "Foundational infra"            # → STAGE-NNN in the active project
just new-spec  "Logger module" STAGE-001       # → SPEC-NNN under that stage
```

## The five-phase cycle

Each spec moves through the cycle; `task.cycle` in its front-matter is
the source of truth, advanced with `just advance-cycle`:

```bash
just advance-cycle SPEC-001 design
just advance-cycle SPEC-001 build     # (allowlisted: frame|design|build|verify|ship)
```

| Phase | What happens | Commands / artifacts |
|---|---|---|
| **Frame** | Why this spec exists, acceptance criteria. | Fill the spec; add open questions to `guidance/questions.yaml`. |
| **Design** | The approach, interfaces, trade-offs. | Record non-trivial choices as `DEC-*` in `decisions/`. |
| **Build** | Implement against the spec. | Create `DEC-*` for build decisions; fill `affected_scope`. |
| **Verify** | Acceptance met? tests pass? no decision drift? | `just decisions-audit --changed` (see below). |
| **Ship** | Reflection + cost totals, then archive. | `just archive-spec SPEC-001`. |

Honest confidence matters: decisions carry an `insight.confidence`
(0.0–1.0) that drives questions at design and flags at verify. See
`AGENTS.md` → Confidence Discipline.

## Decisions and guardrails

Architectural decisions live in `decisions/` as `DEC-*` records. Audit
them anytime:

```bash
just decisions-audit             # lint structure + warn on scope conflicts
just decisions-audit --changed   # which decisions govern your pending edits
```

Fill a decision's optional `affected_scope:` (path globs it governs) so
`--changed` can flag it when those paths change — this is the "decision
drift" check in the Verify phase. Repo-wide rules live in
`guidance/constraints.yaml` (each rule has a severity and paths).

## Staying oriented

Four read-only views, each answering a different question:

```bash
just status            # current state: active project/stage, specs by cycle, stale items
just backlog           # spec-grained "what's next" (active stage; --all to widen)
just roadmap           # stage-grained "where is this project going" (counts per stage)
just specs-by-stage    # flat ledger: every spec by stage, ship date + complexity
                       #   defaults to ALL projects; --active or PROJ-NNN to scope
```

And periodic reflection / reporting:

```bash
just weekly-review     # prints the weekly-review prompt with recent activity loaded
just report-daily      # writes reports/daily/YYYY-MM-DD.md
just report-weekly     # writes reports/weekly/YYYY-WNN.md
```

## When you outgrow the defaults

Everything above is zero-dependency. For diagrams, use Mermaid fenced
blocks in markdown (the default — `docs/architecture.md` and
`docs/data-model.md` ship with examples). When a project genuinely needs
more — C4 modeling, protocol-level integration tests — see the optional,
project-level escalations in `guidance/recommended-tools.md`.

## Dogfood the built CLI on your own photos

Everything above is about *driving the template*. This section is about *using
the tool you are building* — trying the compiled `crustyimg` binary on real
photos before publishing them to a website.

> **Always work on a copy of your originals.** Commands only touch the explicit
> output, but a copy is cheap insurance.

```bash
cargo build --release                        # WebP built-in; binary at target/release/crustyimg
alias ci="$PWD/target/release/crustyimg"     # AVIF output: add --features avif
cp -R ~/site/images ~/cimg-test && mkdir -p ~/cimg-out
```

1. **Smoke-test one photo** — inspect, optimize, verify the quality you gave up:
   ```bash
   ci info ~/cimg-test/hero.jpg --exif
   ci optimize ~/cimg-test/hero.jpg -o ~/cimg-out/hero.jpg   # auto-orient + strip + visually-lossless
   ci diff ~/cimg-test/hero.jpg ~/cimg-out/hero.jpg          # SSIMULACRA2 score (higher = closer)
   ```

2. **Tune once → replay the folder** — the recipe round-trip (`edit` captures the
   chain; `apply` replays it byte-identically across the directory, in parallel):
   ```bash
   ci edit ~/cimg-test/hero.jpg --auto-orient --resize-max 1600 \
      --save-recipe ~/web.toml -o ~/cimg-out/hero.jpg
   ci apply --recipe ~/web.toml ~/cimg-test/*.jpg --out-dir ~/cimg-out -j 8 -y
   ```

3. **Batch web-prep to a target or budget** — `optimize`/`shrink` hit a *look* or a
   *size*, not a guessed number; `convert` re-encodes a whole folder:
   ```bash
   ci optimize ~/cimg-test/*.jpg --out-dir ~/cimg-out -j 8 -y
   ci shrink   ~/cimg-test/*.jpg --max 1600 --target high --out-dir ~/cimg-out -j 8 -y
   ci shrink   ~/cimg-test/*.jpg --max-size 200KB         --out-dir ~/cimg-out -j 8 -y
   ci convert  ~/cimg-test/*.jpg --format webp            --out-dir ~/cimg-out -j 8 -y
   ```

4. **Responsive `<picture>`/srcset for the page** — variant set + a paste-ready
   snippet on stdout:
   ```bash
   ci responsive ~/cimg-test/hero.jpg --widths 480,960,1440 --formats webp,jpeg --out-dir ~/cimg-out
   ```

5. **Verify before publishing** — quality gate + confirm GPS is gone:
   ```bash
   ci diff ~/cimg-test/hero.jpg ~/cimg-out/hero.jpg --fail-under 70
   ci info ~/cimg-out/hero.jpg --exif | grep -i gps || echo "no GPS — good"
   ```

Current safety behavior: GPS is dropped by default (`--keep-gps` to retain);
existing files are not overwritten without `-y` and symlinked output paths are
refused; multi-input commands need `--out-dir`; absurd-dimension/decode-bomb
inputs are rejected. Full CLI contract: `docs/api-contract.md`.

## Pointers

- First-time walkthrough: `GETTING_STARTED.md`
- Copy-paste prompts per phase: `FIRST_SESSION_PROMPTS.md`
- Conventions every agent follows: `AGENTS.md`
- All commands: `just --list`
