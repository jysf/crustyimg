---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-090
  type: story
  cycle: build
  blocked: false
  priority: high
  complexity: S
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-16
references:
  decisions: [DEC-048, DEC-069, DEC-070]
  constraints: [ergonomic-defaults, every-public-fn-tested, test-before-implementation]
  related_specs: [SPEC-084, SPEC-085, SPEC-088]
value_link: >
  The flagship verb's headline promise must be true as a user reads it. `web` claims "never bigger",
  but enforces that against the DOWNSCALED intermediate, not the file the user handed in — so an
  already-small >2048px source can come back larger while the docs promise it can't. Make the claim and
  the behavior agree, in whichever direction the evidence supports.

cost:
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 850000
      estimated_usd: 7.65
      note: >
        Main-loop build (not a separately-metered subagent), so tokens_total is an
        order-of-magnitude ESTIMATE ([[autonomous-run-cost-estimates]]); estimated_usd
        at Opus 4.8 list ($5/$25, ~80/20 in/out, no cache discount). Heavy on large
        source reads (decide.rs, cli.rs) + repeated end-to-end reproduction/oracle runs
        (incl. two slow AVIF debug encodes).
  totals:
    tokens_total: 850000
    estimated_usd: 7.65
    session_count: 1
---

# SPEC-090: reconcile `web`'s never-bigger claim with its actual baseline

## Context

SPEC-088's verify measured `web` shipping a file **14% larger than the source** on a 3000px input. The
pre-spec binary reproduces it identically, so this is **pre-existing SPEC-085/`web` behavior, not a
SPEC-088 regression** — and it is *honestly reported* ("14% larger", per SPEC-084's negative-savings
work). The defect is that **the documented promise is measured against a different baseline than the
code enforces.**

**Mechanism (grounded at framing, read the code — do not re-derive from this paragraph alone):**
`analysis::decide::pick_winner` (`src/analysis/decide.rs` ~212) admits candidates with
`cands[i].bytes < source_bytes` and returns `None` (passthrough) when none qualify. That is a correct,
mode-independent never-bigger guard **relative to whatever `source_bytes` is at that step**. In the
`web` recipe (`recipes/web.toml`: `auto-orient` → `resize max 2048` → terminal `optimize`), the terminal
`optimize` step's source is the **downscaled intermediate**. The original file's bytes are never in the
comparison. So:

- **Enforced:** output ≤ the 2048px intermediate.
- **Claimed:** `recipes/web.toml` — "pick the smallest modern format that beats the source — **never
  shipping a larger file**"; `description = "… never bigger, score."`; `docs/cli-reference.md` ~122 —
  "(never larger)". A user reads all of these as *never larger than the file I gave you*.
- **Divergence:** exactly when downscale + re-encode exceeds an already-small/heavily-compressed source
  above 2048px. Rare, but real, and the promise is unconditional.

**The genuine design tension (why this isn't a one-line fix):** `web`'s *other* contract is a dimension
contract — downscale the long edge to 2048. When the downscaled output exceeds the original, there is
**no fallback that satisfies both**: passing through the original returns a >2048px file the user
explicitly asked to shrink. `optimize` (the keep-dims byte-primitive) has no such tension — its
never-bigger guarantee is unconditional and correct, and must stay that way.

## Goal

Make `web`'s promise and `web`'s behavior agree. **Decide, with evidence, which gives** — the claim or
the behavior — and land it: either enforce never-bigger against the **original file** (and define what
happens to the dimension request), or **correct the claim** to state the real guarantee and surface the
larger-output case to the user. Emit a DEC recording the choice and its rationale.

## Inputs — files to read

- `src/analysis/decide.rs` — `pick_winner` (~202–240) and the `source_bytes` it is handed; the
  `savings_percent` / `size_delta_phrase` honest-reporting path (~325–340, SPEC-084).
- `src/cli/mod.rs` — `run_web` / the terminal-optimize recipe step (~1097–1150, SPEC-085/DEC-070); what
  `source_bytes` is at that step, and whether the ORIGINAL file's byte count is even in scope there.
- `recipes/web.toml` — the claim, in both the header comment and `description`.
- `docs/cli-reference.md` (~122, ~141), `README.md` (~136), `docs/recipes.md` (~21) — every rendered
  never-bigger claim; note which are about `optimize` (keep-dims, correct) vs `web` (the problem).
- `projects/.../specs/done/SPEC-084-*.md` — the never-bigger + honest-negative-savings design this must
  not regress; `SPEC-085-*` — how `web` was specified.

## Outputs

- **A decision + its implementation.** The two coherent candidates (evaluate both; a third is fine if
  better):
  - **(A) Correct the claim + surface it.** Keep behavior (the dimension contract wins). Restate the
    guarantee precisely wherever it is rendered: `optimize` = unconditionally never bigger; `web` =
    downscales to a dimension bound and picks the smallest format that beats the *downscaled* image —
    which on an already-tiny large-dimension source can exceed the original, reported honestly as
    "N% larger". **Add a user-visible signal** when `web`'s output exceeds the ORIGINAL file (a stderr
    note, and a field in the `--json` audit report — SPEC-088's schema, additively).
  - **(B) Enforce against the original.** Compare the final `web` output to the original file's bytes and
    passthrough the original when it wins — **explicitly accepting that the output can then exceed 2048px**,
    which contradicts the dimension request and must be loudly reported, not silent.
  - Recommendation to test, not assume: **(A)** — the dimension request is the user's explicit
    instruction and silently ignoring it is worse than an honest "larger" report; SPEC-084 already built
    the honest-negative-savings machinery this leans on. Prove or refute it with the evidence below.
- **Whatever docs change** to make every rendered claim true (`recipes/web.toml` header + `description`,
  `docs/cli-reference.md`, `README.md`, `docs/recipes.md`).
- **`DEC-075`** (next free) — the `web` never-bigger baseline decision: which baseline, why, what the user
  sees when output > original, and the explicit note that `optimize`'s unconditional guarantee is unchanged.

## Acceptance Criteria

- [ ] Every **rendered** never-bigger claim for `web` is **true as written** (recipe header +
      `description`, cli-reference, README, recipes doc). `optimize`'s unconditional keep-dims guarantee
      is unchanged and still stated.
- [ ] The `web`-output-exceeds-**original** case is **reproducible in a test** (a committed fixture or a
      generated one: an already-small, heavily-compressed >2048px image) and the behavior is asserted —
      whichever of (A)/(B) is chosen.
- [ ] Under (A): a `web` run whose output exceeds the original emits a **user-visible signal** (stderr +
      an additive field in the `--json` audit report), and the reported savings still reads honestly as
      "N% larger" (SPEC-084 not regressed). Under (B): the passthrough-the-original path is asserted and
      the dimension-contract violation is loudly reported.
- [ ] **`optimize` is byte-identical** to before this spec on the same inputs (the keep-dims primitive is
      not touched) — proven against the pre-spec binary.
- [ ] `DEC-075` records the choice, the rejected alternative, and the rationale.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`,
      `cargo build --no-default-features`, `just validate` pass.

## Failing Tests (written at design)

- **`src/analysis/decide.rs` / integration**
  - `web_output_larger_than_original_is_surfaced` — an already-small >2048px source through `web`
    produces the chosen behavior (A: shipped + signalled + "N% larger"; B: original passed through +
    dimension violation reported). This is the spec's central case.
  - `optimize_never_bigger_still_unconditional` — the keep-dims primitive still never ships a larger file
    (SPEC-084's guarantee, unregressed).
  - `optimize_byte_identical_to_pre_spec` — regression anchor against the parent-commit oracle.
  - `web_normal_case_unchanged` — a typical photo through `web` is byte-identical to pre-spec (this spec
    must not move the common path).

## Implementation Context

### Decisions that apply
- `DEC-048` — the winner-picking/decision model `pick_winner` implements.
- `DEC-069` (SPEC-084) — `Mode::Fast`, the never-bigger guarantee, and **honest negative savings**
  (`savings_percent` goes negative; never a clamped "0% smaller"). Do not regress this — it is the
  machinery option (A) leans on.
- `DEC-070` (SPEC-085) — `web == apply --recipe web` via the terminal-optimize recipe step. **Any change
  must hold that identity**: `web <x>` and `apply --recipe web <x>` must stay byte-identical.

### Constraints
- `ergonomic-defaults` — the flagship verb's promise must match what a user reasonably reads.
- `test-before-implementation` / `every-public-fn-tested`.

### Out of scope (this spec)
- Changing `optimize`'s keep-dims never-bigger guarantee (correct as-is).
- The `web` quality/format decision itself (SPEC-084's `Mode::Fast`, q85) — this is about the *baseline*
  a size comparison is made against, not about what gets encoded.
- SPEC-088's audit report/bench (shipping separately); the demo/wasm surface (SPEC-080).

## Notes for the Implementer
- **The bug is a baseline mismatch, not a broken guard.** `pick_winner` is correct for what it is handed;
  the question is what it *should* be handed for `web`, and what the docs may promise. Read the whole
  `web` path before changing anything ([[read-whole-function-before-asserting-a-gap]]).
- **`web == apply --recipe web` is a shipped identity (DEC-070)** — whatever you change must preserve it.
- **Prove the failure case first.** Build the reproducer (already-small, heavily-compressed, >2048px)
  and drive the *pre-spec* binary to confirm the 14%-larger behavior before touching code — the SPEC-087
  oracle discipline. Don't take this spec's numbers on faith; verify measured them on one image.
- If option (A) wins, the `--json` signal should extend SPEC-088's audit schema **additively/gated**
  (the DEC-074 precedent) — coordinate with whatever shape 088 lands.

---

## Build Completion
- **Branch:** `spec-090-web-never-bigger` · **PR:** (pending) · **All acceptance criteria met?** Yes ·
  **New decisions:** DEC-075 · **Deviations:** see below · **Follow-ups:** the resize-adds-alpha quirk
  (below) is a separate pre-existing bug worth its own frame.

**Option chosen: (A)** — keep behavior (the dimension contract wins), correct every rendered claim, and
surface the larger-than-original case (stderr `note:` + an additive/gated `"larger_than_source":true` field
in the `--json` audit report). Proven, not assumed:

- **Pre-spec reproduction (the parent-commit oracle, `0eac2cf`, built `--features avif,webp-lossy`):** a
  heavily-compressed 3000×3000 JPEG (420816 B) through `web` ships AVIF at **572218 B — 36% larger**
  (`savings_percent:-36`, `winner:0`). Confirms the case is pre-existing SPEC-085 behavior, honestly
  reported by SPEC-084's negative-savings path, and promised away by the docs. **The spec's framing
  mechanism was imprecise** — `source_bytes` is the **original** file (`read_raw_bytes`), *not* the
  downscaled intermediate; the larger output comes from the `pipeline_altered` override (src/cli/mod.rs
  ~4556) shipping the smallest correct re-encode when nothing beats the original. Reading the code before
  trusting the paragraph ([[read-whole-function-before-asserting-a-gap]]) changed the fix's shape.
- **The signal surfaces** (verified on the real binary, not just asserted): `--json` stdout carries
  `...,"savings_percent":-36,"larger_than_source":true,"ssim":84.5}`; stderr carries an explicit
  `note: shipped 572218 B, larger than the 420816 B source (36% larger) — …` on every channel (respecting
  `--quiet`); the default summary still reads "36% larger".
- **`web == apply --recipe web` (DEC-070) holds byte-for-byte** — image bytes AND the full `--json` report
  (including the new flag) are `cmp`-identical (post-change binary).
- **`optimize` byte-identical to pre-spec** — `cmp`-identical output vs the parent-commit binary on the
  same input; the keep-dims primitive (`pick_winner`/`solve_candidate`/encode) is untouched (only an
  additive gated field + a gated stderr note were added).
- **Every rendered never-bigger claim made true** — grep of the live surface
  (`never bigger|never larger|never ship|never-bigger`, 20 hits across README/docs/recipes/src): `web`'s
  claims fixed at 8 rendered sites (`web` clap `--help`, `recipes/web.toml` header+description,
  `recipes/gallery.toml`, `recipes/product.toml`, `docs/cli-reference.md`, `docs/api-contract.md`,
  `README.md`, `docs/recipes.md`) + the `src/recipe/bundled.rs` module doc; `optimize`'s unconditional
  keep-dims claims left intact and still stated (4 sites), per scope.

**Deviations / judgment calls:**
- **`json_shape_consistent_across_verbs` (SPEC-088) fixture changed** `jpeg_with_exif(256,256)` →
  `detailed_png(800,600)`. Cause: on the tiny gradient, `optimize`'s metadata-forced re-encode ships
  larger → emits the (data-gated) `larger_than_source` key, which `web`/`apply` (which the resize op lets
  shrink it) do not — a legitimate divergence for a *data-driven* additive field. The new fixture is one
  all three verbs shrink, so the golden key set documents the **base** schema; the flag's
  presence-when-larger is covered by SPEC-090's own tests.
- **The two heavy end-to-end `web`-larger tests are gated `#[cfg(not(feature = "avif"))]`.** Under
  `--features avif`, this content selects AVIF whose *debug* encode is ~180 s — untenable for CI. The
  DEFAULT `cargo test` codec set re-encodes as fast lossless-WebP/PNG (~2 s) and reproduces the case; the
  signal is codec-independent and unit-tested in `analysis::decide` for **every** feature build.
- **Fixture reuse, not a new fixture.** The reproducer uses the existing `detailed_jpeg_with_icc` (a
  lossy source in a lossless-only bucket, >2048px, with `--max 512` to keep the debug encode quick) rather
  than a bespoke generator (which I built, measured, then deleted as dead code).

### Build-phase reflection
1. **The spec's stated mechanism was wrong in a load-bearing way, and only reading the code caught it.**
   The framing said `source_bytes` is the downscaled intermediate; it is the original file. The real
   mechanism is the `pipeline_altered` override, and SPEC-084 had *already* built the honest-larger
   reporting — so the fix was "make it explicit + fix the docs", not "change the comparison". Taking the
   paragraph on faith would have produced a wrong, invasive change.
2. **A data-driven additive JSON field is not the same as a flag-driven one.** `ssim`/`timing` are gated by
   *flags*, so a cross-verb parity test can force them present; `larger_than_source` is gated by *bytes*,
   which `web` (downscales) and `optimize` (keeps dims) legitimately disagree on. That broke a parity test
   in a way that was a real signal about the field's nature, not a bug — the fix was a fixture, not a hack.
3. **AVIF debug-encode cost is a real test-design constraint.** A "just run web on a big image" end-to-end
   test costs 180 s under `--features avif`. Measuring that (twice — once by timeout) forced the right
   design: gate the heavy path to the fast codec set and carry the codec-independent proof in unit tests.

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
