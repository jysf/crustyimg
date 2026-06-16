# AGENTS.md — Claude-Only Variant

Instructions for Claude working across all phases of this repository. Read this file first, every session.

> This variant assumes Claude plays every role: architect, implementer, reviewer. The context normally in a handoff document lives inside each spec's `## Implementation Context` section.

> This file contains conventions only. For rules/constraints, see `/guidance/constraints.yaml`. For architectural rationale, see `/decisions/`. For waves of work against this app, see `/projects/`.

---

## 1. Repo Overview

- **Repo (the app):** `crustyimg` — a fast Rust CLI to view and transform images.
- **Purpose:** View images in the terminal and run everyday transforms
  (resize, shrink-for-web, thumbnail, convert, auto-orient, watermark, EXIF
  strip/clean/set) on a single image or a batch, via a load-once
  `Operation` pipeline and reusable TOML recipes.
- **Primary stakeholders:** Web/content developers prepping images; terminal
  power users; the maintainer (clean trait-based base to extend).
- **Active project:** PROJ-001 — crustyimg MVP (clean rebuild). Active stage:
  STAGE-001 (foundation and pipeline core).

See `.repo-context.yaml` for structured metadata.

---

## 2. Work Hierarchy

```
REPO (the app — persists across all projects)
 └─ PROJECT (a wave of work: "MVP", "improvements", "v2 redesign")
     └─ STAGE (a coherent chunk within a project)
         └─ SPEC (an individual task)
```

- The **repo** is the app. `AGENTS.md`, `/docs/`, `/guidance/`,
  `/decisions/` live at repo level because they accumulate across all
  projects.
- A **project** (`/projects/PROJ-*/`) is a bounded wave of work.
- A **stage** is an epic-sized chunk within a project (2–5 per project).
- A **spec** is a single implementable task. Belongs to one stage in
  one project.

In this variant, Claude plays architect and implementer in **separate
sessions**. The spec file itself carries all the context — see its
`## Implementation Context` section.

**Decisions persist at repo level.** A decision made during PROJ-001
binds PROJ-002 as well.

**Specs do not cross project boundaries.**

---

## 3. Business Value

Value structure exists at project and stage levels; specs link lightly.

**Project `value:` block** states the thesis — a testable claim about
what this wave of work delivers. Beneficiaries, success signals, and
risks to the thesis make it falsifiable, not marketing copy.

**Stage `value_contribution:` block** states what this coherent chunk
of work advances, what capabilities it delivers, and what it
explicitly doesn't try to do. Helps avoid stages that seem valuable
but don't contribute to the project thesis.

**Spec `value_link:`** is a one-sentence reference back to the
stage's value. Infrastructure specs may have
`value_link: "infrastructure enabling X"`. Optional but encouraged —
it surfaces specs that don't trace back to the thesis.

Reports (`just report-daily`, `just report-weekly`) aggregate these
signals: which stages advanced the thesis, which specs most directly
delivered it, and where value traceability broke down.

---

## 4. Cost Tracking Discipline

Every cycle on a spec appends a session entry to the spec's
`cost.sessions` list. Agents self-report so reports can aggregate AI
spend over time.

- **Claude Code:** run `/cost` at the end of your session.
- **API calls:** use the `usage` object in the API response.
- **Claude.ai web:** estimate based on session length. Set
  `interface: claude-ai` so reports can distinguish estimates.
- **Third-party agents** (Ollama, Kilo, Factory, etc.): use whatever
  cost mechanism the agent provides. If none, enter null numeric
  values with a note.

Verify cycle flags specs missing cost entries for prior cycles (does
not block the PR — visibility only). Ship cycle computes `cost.totals`
from the session entries.

Reports aggregate cost by cycle, by interface, by spec, and by stage.

---

## 5. Tech Stack

- **Language:** Rust, edition 2021, stable toolchain (DEC-009).
- **Runtime:** Native single binary. **No async runtime** — CPU-bound work,
  instant startup; batch parallelism via `rayon` (DEC-006).
- **CLI framework:** `clap` 4 (derive, subcommands).
- **Pixel core:** `image` 0.25 (the single pixel library, DEC-002);
  `fast_image_resize` 5 for the SIMD resize backend (DEC-008).
- **Terminal display:** `viuer` 0.9.
- **Serialization:** `serde` 1 + `toml` 0.8 (recipes, DEC-005).
- **Errors:** `thiserror` 2 (library) + `anyhow` 1 (binary boundary) (DEC-007).
- **Sources:** `glob` 0.3, `walkdir` 2. **Batch:** `rayon` 1, `indicatif` 0.17.
- **Metadata (container lane):** `kamadak-exif` 0.6 (read-only), `img-parts`
  0.3, `little_exif` 0.6 (DEC-003).
- **Native codecs (off by default, cargo features):** `mozjpeg`, `ravif`
  (avif), `rexiv2` (DEC-004).
- **Database:** none (no persistent store; recipes are user TOML files).
- **Testing:** `cargo test` — unit tests in `#[cfg(test)]` modules,
  integration tests under `tests/`, native-generated image fixtures
  (no shell-out to ImageMagick).
- **Linter / Formatter:** `cargo clippy --all-targets -- -D warnings`, `cargo fmt`.
- **Hosting:** release artifacts (GitHub Releases); target brew / crates.io.
- **CI:** GitHub Actions, three-OS matrix (Linux/macOS/Windows) (DEC-009).

> Pin exact patch versions in `Cargo.toml` at SPEC-001 build. Adding any new
> top-level crate requires a `DEC-*` (constraint
> `no-new-top-level-deps-without-decision`).

---

## 6. Commands (exact)

These are the APP's commands. For template/workflow commands, see `justfile`.

```bash
# install / build
cargo build                         # debug build
cargo build --release               # optimized binary (target/release/crustyimg)

# dev (run the CLI)
cargo run -- --help                 # see subcommands
cargo run -- view path/to/image.jpg # run a subcommand

# test
cargo test                          # all tests (unit + integration)
cargo test <name>                   # single test or module, e.g. `cargo test recipe_round_trip`

# lint / format ("typecheck" = clippy; Rust type-checks as part of build)
cargo clippy --all-targets -- -D warnings         # lint, warnings are errors
cargo fmt --check                   # formatting gate (CI); `cargo fmt` to fix

# native-codec feature build (off by default)
cargo build --features mozjpeg
```

---

## 7. Directory Structure

```
/
├── AGENTS.md                          # This file
├── CLAUDE.md                          # Pointer to AGENTS.md
├── README.md                          # Human-facing readme
├── GETTING_STARTED.md                 # First-project walkthrough
├── FIRST_SESSION_PROMPTS.md           # Phase prompts
├── .repo-context.yaml                 # Repo (app) metadata
├── .variant                           # "claude-only"
├── justfile                           # Commands: just status, just new-spec, etc.
├── scripts/                           # Shell scripts powering justfile
├── docs/                              # Architecture, data model, API contract
├── guidance/                          # Repo-level rules (across all projects)
│   ├── constraints.yaml
│   └── questions.yaml
├── decisions/                         # Repo-level DEC-* (across all projects)
├── feedback/                          # Downstream user feedback captures
├── reports/                           # Daily + weekly report outputs
├── projects/                          # Waves of work
│   ├── _templates/                    # Shared templates
│   │   ├── spec.md
│   │   ├── stage.md
│   │   └── project-brief.md
│   ├── PROJ-001-<slug>/
│   │   ├── brief.md
│   │   ├── stages/
│   │   └── specs/
│   │       └── done/
│   └── PROJ-002-<slug>/
├── Cargo.toml                         # crate manifest + deps (pinned at SPEC-001)
├── tests/                             # integration tests + image fixtures
└── src/                               # the crustyimg crate (lib + binary)
    ├── main.rs                        # parse args, dispatch, map errors → exit codes
    ├── error.rs                       # crate-wide error/result aliases (thiserror)
    ├── image/                         # canonical Image (wraps image::DynamicImage), load, ImageInfo
    ├── operation/                     # Operation trait, params, registry (extension point)
    ├── pipeline/                      # Pipeline executor: decode once → ops → result
    ├── recipe/                        # Recipe struct + TOML (de)serialization
    ├── source/                        # Source: file | glob | dir | stdin
    ├── sink/                          # Sink: file | dir+template | stdout | display (viuer)
    ├── metadata/                      # container lane: read/edit EXIF/ICC (separate from pixel lane)
    └── cli/                           # clap types, subcommand surface, arg → pipeline wiring
```

> The `src/` layout above is the **planned** structure; no application code
> is scaffolded yet (it lands in STAGE-001 build cycles). See
> `docs/architecture.md` for the module/layer rationale.

---

## 8. Cycle Model

Every spec moves through five cycles. **Cycles are tags, not gates**.

| Cycle | Purpose |
|---|---|
| **frame** | Go/no-go on the spec |
| **design** | Write the spec + failing tests + implementation context |
| **build** | Make failing tests pass |
| **verify** | Review + validation in one pass |
| **ship** | Merge, deploy, reflect, archive |

Valid transitions:
```
frame → design → build → verify → ship
                   ↑       │
                   └───────┘ (verify sends back on punch list)
```

**In this variant**, use **separate Claude sessions** for each cycle.
A fresh session prevents design-phase context from contaminating build
decisions, and a fresh verify session catches drift a continuation
session wouldn't.

Project and stage lifecycles are lighter:
- **Project status:** `proposed | active | shipped | cancelled`
- **Stage status:** `proposed | active | shipped | cancelled | on_hold`

---

## 9. Instruction Timeline

Every spec has a timeline file at
`projects/*/specs/SPEC-NNN-<slug>-timeline.md` listing cycle
instructions in order with status markers.

Status markers:

- `[ ]` not started — no one has picked this up yet
- `[~]` in progress — an executor is currently running this
- `[x]` complete — cycle finished; see the prompt file for what was run
- `[?]` blocked — needs a human decision or external unblock before
  proceeding. Include a one-line reason after the marker.

Cycle prompts live at `projects/*/specs/prompts/SPEC-NNN-<cycle>.md`.
The architect writes them; executors read and run them.

**Discipline for executors:**

- When you start a cycle, mark it `[~]`.
- When you finish, mark it `[x]` with a one-line result (PR number,
  cost, completion date).
- If you hit a real blocker — constraint ambiguous, dependency
  missing, verify surfaced something needing architect judgment —
  mark `[?]` with a one-line reason. Do NOT use `[?]` as a "I don't
  know what to do" dumping ground. Blocked means the next move
  requires someone else; everything else is in-progress or a
  question to resolve in the current session.

This is a convention, not a mechanism. No tooling enforces it; the
discipline lives in the prompt set. Skip it and nothing breaks, but
you lose the history artifact and the next executor has to hunt for
the right prompt.

---

## 10. Cross-Reference Rules

Every spec has these relationships, encoded in front-matter:
- `project.id` → the project it belongs to
- `project.stage` → the stage within that project
- `references.decisions` → DEC-* it was designed against
- `references.constraints` → constraints that apply

DECs are stable; specs come and go. DECs don't reciprocally list specs.

---

## 11. Coding Conventions

- **Naming:** Rust standard — `snake_case` for fns/modules/files,
  `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for consts. One
  module per concern under `src/<area>/` (see Section 7). The pixel
  extension point is the `Operation` trait; new transforms are new impls.
- **File organization:** Library-first. The library (`image`, `operation`,
  `pipeline`, `recipe`, `source`, `sink`, `metadata`, `cli`) holds the
  logic; `main.rs` is a thin shell that parses args and maps errors to exit
  codes. The pixel core (`image/`, `operation/`) must not depend on `clap`,
  files, or terminals.
- **Imports:** Group std / external crates / local (`crate::`); prefer
  explicit `use` paths over glob imports (`use foo::*`) except in test
  modules and preludes.
- **Error handling (DEC-007):** Library code returns typed `thiserror`
  enums; **no `unwrap()`/`expect()`/`panic!()` on recoverable paths**
  (constraint `no-unwrap-on-recoverable-paths`). Only `main.rs`/`cli` use
  `anyhow` to add context and map to the exit codes in
  `docs/api-contract.md`.
- **Logging:** Diagnostics go to **stderr** (so `-o -` / stdout pipes stay
  clean), gated by `-v/--verbose` / `--quiet`. Do not `println!` diagnostic
  output to stdout. No `tokio`/async (constraint `no-async-runtime`).
- **Comments:** Explain *why*, not *what*.
- **No dead code.** Delete, don't comment out. Code must pass
  `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` (constraint
  `clippy-fmt-clean`).
- **Diagrams:** author them as Mermaid fenced blocks in markdown
  (`/docs/`, `/decisions/`, specs) so they render on GitHub and you can
  keep them current as part of the work. Update the relevant diagram in
  the same change, not afterward. See `/guidance/recommended-tools.md`.

---

## 12. Testing Conventions

- Every new public function gets at least one test (constraint
  `every-public-fn-tested`). The prototype had zero tests; we don't.
- **Unit tests** live in a `#[cfg(test)] mod tests { ... }` block at the
  bottom of the module they test.
- **Integration tests** live under `tests/` (one file per area, e.g.
  `tests/recipe_round_trip.rs`, `tests/pipeline.rs`); they exercise the
  public crate API and the binary.
- **Fixtures:** generate test images natively (solid/gradient/noise) — do
  **not** shell out to ImageMagick (the prototype's mistake). Prefer
  deterministic encoders so golden/byte-size assertions are stable.
- **What to assert:** behavior, not just exit-0 — dimensions, byte sizes,
  recipe round-trip equality, typed error variants, SSIM/tolerance where
  pixel-exactness isn't guaranteed (e.g. resize backend parity, DEC-008).
- **Coverage expectations:** every acceptance criterion in a spec maps to a
  test; new public functions are covered. No hard percentage gate.
- **TDD:** Tests live in the spec's `## Failing Tests` section, written
  during **design**, made to pass during **build** (constraint
  `test-before-implementation`).

---

## 13. Git and PR Conventions

- **Branch:** `feat/spec-NNN-<slug>` for spec builds; `chore/<slug>`,
  `fix/<slug>`, `docs/<slug>`, `ci/<slug>` for non-spec work. Base `main`.
- **One spec per branch, one PR per branch** (constraint `one-spec-per-pr`).
- **Design/specs commit to `main` directly** (no PR — they are the contract
  the build branches off). **Build work goes through a PR.**
- **Verify + ship bookkeeping lands on `main`, not the feature branch.** The
  feature branch carries only the build's code + the spec's `## Build
  Completion`. Verify/ship edits (timeline marks, ship prompt, reflections,
  cost totals, stage backlog, archiving to `done/`) are made on `main` after
  the PR merges — so ship is a clean fast-forward with no spec-file
  divergence between the branch and `main`. (Lesson from SPEC-001 ship.)
- **Commits:** Conventional Commits — `<type>(<scope>): <summary>`. Types:
  `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`, `ci`. Scope is
  the module/area (e.g. `feat(pipeline): fold ordered ops over Image`,
  `test(recipe): assert TOML round-trip`, `ci(matrix): add windows`).
  Imperative mood, lowercase summary, no trailing period. End commit
  messages with the Co-Authored-By trailer for the model that wrote them.
- **PR title:** conventional-commit form **carrying the spec id**, e.g.
  `feat(pipeline): decode-once executor (SPEC-003)` or
  `feat(SPEC-001): cargo project + multi-OS CI`.
- **PR body** (template — match `bragfile000`'s shape):

  ```
  ## Summary
  - <bullet per user-visible / structural change>

  ## Spec metadata
  - **Project:** PROJ-001
  - **Stage:** STAGE-NNN
  - **Spec:** SPEC-NNN

  ## Decisions referenced
  DEC-NNN (why it applied here), …

  ## Constraints checked
  - `constraint-id` ✅ — <one-line evidence>

  ## New decisions
  - DEC-NNN — <title> (or "No new DEC")
  ```
  End the PR body with the Claude Code generated-with footer.

---

## 14. Domain Glossary

- **Operation** — A named, parameterized, pure pixel transform implementing
  the `Operation` trait (`name`, `params`, `apply(Image) -> Result<Image>`).
  The single extension point for new image transforms (DEC-002).
- **Recipe** — An ordered, versioned list of operations serialized as TOML.
  The same recipe runs on one image or a whole batch; it round-trips via the
  operation registry (DEC-005).
- **Registry** — The map `operation name + params -> Operation`. Both the
  CLI and the recipe loader construct operations through it, which is what
  makes recipes round-trip.
- **Pipeline** — The executor that decodes an image **once**, folds an
  ordered `Vec<Operation>` over it in memory, and hands the result to a
  sink. No per-operation disk round-trips (DEC-002).
- **Source** — The input abstraction: resolves a CLI argument into inputs —
  a single file, a glob, a directory, or `-` (stdin).
- **Sink** — The output abstraction: a file, a directory plus a
  name-template (`{stem}_web.{ext}`), stdout (`-`), or terminal display
  (viuer).
- **Image** — The one canonical in-memory model, wrapping
  `image::DynamicImage` (plus source format and optional metadata bundle).
- **Pixel lane** — decode → operations → encode. Drops metadata by nature
  (the `image` crate discards it on encode).
- **Container lane** — Container-level metadata read/edit/preserve
  (`kamadak-exif`, `img-parts`, `little_exif`) that never re-decodes pixels.
  Separate from the pixel lane (DEC-003).
- **Default-preserve policy** — On pixel-lane encodes, keep orientation +
  ICC + copyright/artist; drop GPS unless `--keep-gps` (DEC-003).
- **Gravity** — A compass anchor (north/south/east/west/center and
  combinations) for placing a watermark or crop region within an image.

---

## 15. Cycle-Specific Rules

### During **build**

Start a **new Claude session**. Do not continue from the design session.

Before writing code:
1. Read the spec's `## Implementation Context` section.
2. Read every `DEC-*` it references.
3. Read the parent `STAGE-*.md` and project `brief.md`.
4. Read `/guidance/constraints.yaml`.
5. If anything is ambiguous, add to `/guidance/questions.yaml` and stop.

When done:
1. Fill in spec's `## Build Completion` (including reflection).
2. Append a build cost session entry to `cost.sessions`.
3. `just advance-cycle SPEC-NNN verify`.
4. Create `DEC-*` files for non-trivial build decisions. When a
   decision is tied to specific code, fill in its `affected_scope`
   with the path globs it governs (e.g. `src/lib/log.ts`,
   `src/api/**`). This is required for file-bound decisions — it's
   what lets `just decisions-audit --changed` surface the decision
   when those paths change later. Leave `affected_scope: []` only for
   decisions not tied to particular files (e.g. a process choice).
5. Open PR.

### During **verify**

Start **another new Claude session**. Do not reuse build session.

Check: acceptance criteria met? tests pass? no decision drift? no
constraint violations? non-trivial choices have DEC-*? build reflection
answered honestly? `cost.sessions` has entries for prior cycles
(flag if missing, don't block)?

For the "decision drift" check, run `just decisions-audit --changed` —
it flags which `DEC-*` records govern the files this spec touched, so you
can confirm the build stayed consistent with them. `just decisions-audit`
(no flag) lints the records themselves. See
`/guidance/recommended-tools.md` for optional, heavier verify tooling
(e.g. LineSpec for protocol-level integration tests).

Append a verify cost session entry to `cost.sessions`.

Output: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

### During **ship**

Append `## Reflection` to spec. Three answers. Append a ship cost
session entry, then compute `cost.totals`. Then
`just archive-spec SPEC-NNN`. If stage backlog is complete, run the
Stage Ship prompt.

---

## 16. Session Hygiene (claude-only specific)

Because one agent plays multiple roles, context contamination is a real
risk. Four habits keep it at bay:

1. **New session per cycle where possible.** Especially design → build
   and build → verify.
2. **Never reference "as I said earlier"** in later cycles. The spec
   is the source of truth.
3. **Weekly review is non-optional.** Without a second agent pushing
   back, drift compounds silently. Run `just weekly-review`.
4. **Honest confidence values on decisions.** See Section 17.

---

## 17. Confidence Discipline

Decisions have an `insight.confidence` field (0.0–1.0). Honest values drive:

- **Design:** decisions at confidence < 0.7 also create a question in
  `/guidance/questions.yaml`.
- **Verify:** specs referencing decisions at confidence < 0.6 get a
  yellow flag.
- **Weekly review:** all decisions < 0.8 are listed with strength/weakness trend.

Most decisions should land between 0.7 and 0.95. 1.0 only for truly locked choices.

---

## 18. Pointers

- Constraints: `/guidance/constraints.yaml`
- Open questions: `/guidance/questions.yaml`
- Decisions: `/decisions/` (audit with `just decisions-audit`)
- Recommended (optional) tools: `/guidance/recommended-tools.md`
- Projects: `/projects/`
- Templates: `/projects/_templates/`
- Architecture: `/docs/architecture.md`
- Feedback: `/feedback/`
- Reports: `/reports/` (daily, weekly)
- Timelines: `/projects/*/specs/SPEC-NNN-*-timeline.md` (per-spec)
- Cycle prompts: `/projects/*/specs/prompts/`
- Phase prompts: `/FIRST_SESSION_PROMPTS.md`
- First walkthrough: `/GETTING_STARTED.md`
- Daily commands: run `just --list`
