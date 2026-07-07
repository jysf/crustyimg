---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-053
  type: story
  cycle: ship
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-004
  stage: STAGE-013
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-06

references:
  decisions: [DEC-050, DEC-003, DEC-017]
  constraints: [untrusted-input-hardening, no-new-top-level-deps-without-decision, ergonomic-defaults, test-before-implementation]
  related_specs: [SPEC-050, SPEC-051, SPEC-026]

value_link: >
  Fills out the day-one rule catalog with everything that needs only shipped
  capabilities — the privacy/correctness/size/colorspace rules that make `lint`
  genuinely useful before any engine-backed cleverness.

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-06
      notes: >
        Main-loop orchestrator (PROJ-004 framing session), not separately metered.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 155000
      estimated_usd: 1.40
      duration_minutes: 42
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — autonomous merge-on-green run in the orchestrator main loop, NOT a metered
        subagent. Order-of-magnitude (~155k at Opus 4.8 ~80/20 ≈ $1.40). Built src/lint/rules.rs
        (7 rules + JPEG-SOF/GIF-frame byte sniffs), refactored LintTarget EXIF to a single
        ExifFacts parse + added camera/orientation/icc accessors, added Rule::default_enabled +
        the opt-in enable logic in config, added png_16bit/animated_gif fixtures. 11 unit + 4
        integration tests. PR #62.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 13000
      estimated_usd: 0.12
      duration_minutes: 3
      recorded_at: 2026-07-06
      notes: >
        ESTIMATE — same autonomous run; CI-driven verify, all matrix/feature/lean/msrv/deny jobs
        green on #62 (incl. Windows). Order-of-magnitude (~13k).
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-06
      notes: >
        Main-loop ship bookkeeping (reflection, cost totals, stage backlog, archive) + STAGE-013
        stage-ship, not separately metered.
  totals:
    tokens_total: 168000
    estimated_usd: 1.52
    session_count: 4
---

# SPEC-053: the shipped-capability rules

## Context

SPEC-050 proved the framework with two rules. This spec adds the rest of the rules that need only
capabilities crustyimg already ships — `info`/EXIF read + the metadata lane — so `lint` is a
genuinely useful CI check before STAGE-014's engine-backed rules. Each rule maps to a shipped
capability and names a runnable fix; each respects the SPEC-051 config (severities, per-glob
budgets, intended width). Rule ids are the DEC-050 stability surface.

## Goal

Add the shipped-capability rules — `privacy/camera-metadata`, `orient/orientation-not-baked`,
`size/oversized-bytes`, `dims/oversized-dimensions`, `color/wrong-colorspace`,
`color/missing-icc`|`unexpected-icc`, `format/animated-gif` — each reading only `info`/EXIF, each
with a runnable fix, respecting the config, adding no dependency.

## Inputs

- **Files to read:**
  - `docs/research/proj-002-design-lint.md` §"Rule catalog" — the detect/powered-by/severity/fix
    for each rule (the authoritative table).
  - `src/lint/mod.rs` + `config.rs` (SPEC-050/051) — the `Rule` registry + `LintConfig` (budgets,
    intended width, per-rule severity).
  - `src/cli/mod.rs` — `InfoReport`/`run_info` (`:1055`): dims, format, color_type, bit_depth,
    has_alpha, has_icc, has_exif — the fields these rules read.
  - `src/metadata/` — the EXIF read (Orientation, Make/Model/DateTime, GPS already in SPEC-050);
    the *fixes* (`strip`, `clean --gps`, `auto-orient`, `convert`).

## Outputs

- **Files modified:** `src/lint/` — add the rule impls + register them; each is a small
  `fn(&LintTarget) -> Option<Finding>`.
- **New rules (ids = stable surface, DEC-050):**
  - `privacy/camera-metadata` (info) → fix `strip` — non-GPS identifying EXIF (Make/Model/Serial/
    DateTime).
  - `orient/orientation-not-baked` (warn) → fix `auto-orient` — EXIF Orientation ≠ 1.
  - `size/oversized-bytes` (error) → fix `optimize`/`shrink --max-size` — exceeds the per-glob/
    per-format byte budget from config (format-aware `--maxkb`).
  - `dims/oversized-dimensions` (warn, opt-in) → fix `resize --max <W>` — natural width exceeds a
    **declared** intended width + slack (no page; honest source-file analogue of "properly size").
  - `color/wrong-colorspace` (warn) → fix `convert --format` — CMYK/non-sRGB JPEG, or a needless
    16-bit PNG for web.
  - `color/missing-icc` | `color/unexpected-icc` (info, opt-in) → tag sRGB / `strip`.
  - `format/animated-gif` (warn) → fix `convert --format webp` — an animated GIF that should be
    WebP/video.
- **Database changes:** none.

## Acceptance Criteria

- [ ] Each rule fires on a constructed positive fixture and is **clean** on a negative; each finding
  carries its stable id, the config-resolved severity, and a **runnable `crustyimg` fix**.
- [ ] `size/oversized-bytes` reads the per-glob `[[budget]]` from config (SPEC-051); with no budget
  configured it does not fire. `dims/oversized-dimensions` fires only when an intended width is
  declared (opt-in), never by default.
- [ ] `orient/orientation-not-baked` fires on EXIF Orientation ≠ 1 (fix `auto-orient`);
  `format/animated-gif` fires on a multi-frame GIF; `privacy/camera-metadata` fires on Make/Model
  EXIF (and is distinct from the SPEC-050 GPS rule).
- [ ] Every rule reads only `info`/EXIF (no re-encode, no engine) — the engine-backed "could be
  smaller" rules are STAGE-014; assert no `src/quality`/`src/analysis::decide` use here.
- [ ] No panic on any input (a truncated/mislabeled file is handled by SPEC-050's corrupt rule, not
  a panic in these); determinism upheld; `just deny` green (**no new dependency**); existing tests
  green.

## Failing Tests

- **`src/lint/mod.rs` (unit tests — one positive + one negative per rule)**
  - `"orientation-not-baked fires on Orientation=6, clean on 1"`.
  - `"camera-metadata fires on Make/Model EXIF, clean on none; distinct from gps rule"`.
  - `"oversized-bytes fires over a per-glob budget, clean under; no budget ⇒ no finding"`.
  - `"oversized-dimensions fires over a declared intended width, silent when undeclared"`.
  - `"wrong-colorspace fires on a CMYK/16-bit fixture, clean on sRGB 8-bit"`.
  - `"animated-gif fires on a 2-frame gif, clean on a static image"`.
  - `"each finding carries a runnable crustyimg fix string"`.
- **`tests/lint.rs` (integration)**
  - `"a mixed tree yields grouped findings across several rules with the right exit code"`.
  - `"per-rule severity override (config) flips animated-gif warn→error and changes exit"`.

## Implementation Context

### Decisions that apply
- `DEC-050` — rule ids (stability surface), severities, the config these read.
- `DEC-003` — the metadata lane the privacy/ICC fixes route to; `DEC-017` — `auto-orient` bakes
  orientation (the orientation fix).

### Constraints that apply
- `untrusted-input-hardening` — rules read already-bounded `info`/EXIF; no panic.
- `no-new-top-level-deps-without-decision` — `info`/EXIF only; no new crate (animated-GIF detection
  reuses the shipped decode/frame sniff, not a new lib).
- `ergonomic-defaults` — sensible default severities; opt-in rules (`dims`, ICC) off unless
  configured.
- `test-before-implementation` — the positive/negative tests are the contract.

### Prior related work
- `SPEC-050`/`SPEC-051` (this stage) — the framework + config. `SPEC-026` (shipped) — the metadata
  lane (`strip`/`clean --gps`) the fixes reference.

### Out of scope (for this spec specifically)
- Engine-backed rules (`legacy-format`, `excessive-jpeg-quality`, `indexed-png`) — STAGE-014.
- `format/non-progressive-jpeg`, `format/alpha-in-jpeg-source`, `dupe/near-duplicate` — opt-in/v2,
  deferred (near-dup needs a new dep).

## Notes for the Implementer

- Keep each rule a tiny pure function over `LintTarget`; register in the SPEC-050 registry. The
  hard part is fixtures, not logic — extend `tests/common` with a CMYK JPEG, a multi-frame GIF, and
  a Make/Model-EXIF helper (mirroring `jpeg_with_orientation`).
- `wrong-colorspace`/ICC rules read `has_icc` + color type from `InfoReport`; don't parse the ICC
  profile — presence/absence + a size heuristic is enough for v1.

---

## Build Completion

- **Branch:** `feat/spec-053-shipped-capability-rules`
- **PR (if applicable):** (opened after green local gates)
- **All acceptance criteria met?** yes
- **New decisions emitted:** None — the rule ids/severities/fixes are DEC-050's catalog; the build
  followed it. One small trait addition (`Rule::default_enabled`) to realize the catalog's opt-in
  column — noted below, not weighty enough for a DEC.
- **Deviations from spec:**
  - **Opt-in mechanism:** added `Rule::default_enabled()` (default `true`). The three `info` opt-in
    rules (`privacy/camera-metadata`, `color/missing-icc`, `color/unexpected-icc`) return `false` —
    off by default, enabled by a `--select` prefix or a per-rule severity entry. `dims/oversized-
    dimensions` and `size/oversized-bytes` stay default-*on* but are inert without their config input
    (intended width / budget), which realizes their "opt-in"/"only-with-config" behavior without a
    flag — matching the acceptance ("fires only when an intended width is declared").
  - **CMYK detection:** `color/wrong-colorspace` detects CMYK JPEGs via a raw SOF component-count
    scan (`jpeg_component_count`, `Nf == 4`), unit-tested at the helper level. A native *CMYK JPEG
    fixture* isn't producible — the `image` crate can't encode CMYK and converts CMYK→RGB on decode,
    erasing the signal — so the rule's **16-bit PNG** path is the end-to-end fixture-tested trigger;
    the CMYK branch rides the tested helper. Honest scope note, not a gap.
  - **EXIF single-parse:** replaced the SPEC-050 `has_gps` cache with one lazily-parsed `ExifFacts`
    (`has_gps` + `has_camera` + `orientation`), so the GPS/camera/orientation rules share one
    `kamadak-exif` pass per file.
  - Inherited SPEC-051's deferred integration test — a per-glob `[[budget]]` now drives a real
    `size/oversized-bytes` finding end-to-end.
- **Follow-up work identified:**
  - STAGE-014 (engine-backed rules) is the next stage: `format/legacy-format`,
    `quality/excessive-jpeg-quality`, `format/indexed-png-opportunity`. They read the already-resolved
    `LintTarget::savings_threshold()` and populate `Finding::bytes_saved`. Needs a framing pass first.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — The catalog's "opt-in" column needed a
   concrete mechanism (nothing in SPEC-050/051 defined default-off rules). Resolved with
   `Rule::default_enabled()` + the config enable logic; the intended-width/budget rules were made
   config-gated instead, which is cleaner.
2. **Was there a constraint or decision that should have been listed but wasn't?** — Not really. The
   only friction was fixture generation (CMYK unavailable natively); worth capturing in the testing
   conventions that CMYK/ICC/animation fixtures are hand-built or helper-tested, since the pure-Rust
   `image` encoders don't cover them.
3. **If you did this task again, what would you do differently?** — Nothing major. Putting the seven
   rules in `src/lint/rules.rs` (with the byte-sniff helpers) kept `mod.rs` the framework and made
   each rule a tiny pure `check`. The `ExifFacts` single-parse refactor is the piece I'd have done in
   SPEC-050 with hindsight.

---

## Reflection (Ship)

1. **What would I do differently next time?** — Nothing structural. The `Rule` trait made each of the
   seven rules a tiny pure `check` over `LintTarget`; the only real work was fixtures. The one thing I'd
   pull earlier is the single-parse `ExifFacts` (it belonged in SPEC-050) — three EXIF rules now share
   one `kamadak-exif` pass.
2. **Does any template, constraint, or decision need updating?** — Worth adding to the testing
   conventions (AGENTS §12) that some formats have **no native pure-Rust encoder** (CMYK JPEG, embedded
   ICC), so their fixtures are hand-built byte splices or the detector is helper-tested — a recurring
   pattern across the metadata/lint work. No constraint/DEC change needed; DEC-050's catalog held.
3. **Is there a follow-up spec I should write now before I forget?** — No spec to write now. STAGE-013
   is complete (4/4). The next stage, STAGE-014 (engine-backed rules: `legacy-format`,
   `excessive-jpeg-quality`, `indexed-png-opportunity`), is backlog-only and needs a framing pass —
   they read `LintTarget::savings_threshold()` (already resolved) and populate `Finding::bytes_saved`
   (already wired). That framing is the checkpoint this run stops at.
