# PROJ-002 framing kickoff (scope confirmed — stake in the ground)

Hand this to a fresh **Claude Code session running in the crustyimg repo** (it has file access
and will write spec files). This confirms the scope and completes the framing so PROJ-002 is
build-ready. It does NOT write `src/` code — building is a later session.

---

You are continuing work on **crustyimg** in a fresh session. A prior scoping session framed
PROJ-002 and **the maintainer has CONFIRMED the scope — this is a stake in the ground.** Your job
is to **complete the framing so PROJ-002 is build-ready.** Do NOT write `src/` code — that's the
build cycle, a later session.

## Read first (in the repo)
- `projects/PROJ-002-optimization-engine/brief.md` + `stages/STAGE-011-*` + `stages/STAGE-012-*` +
  `specs/SPEC-046-*` (already written in the design cycle)
- `docs/research/proj-002-design-analysis-layer.md`, `-format-engine.md`, `-classification.md`
  (the authoritative designs for the specs you'll write)
- `docs/research/proj-002-findings.md` (thesis, ranking, the reconciliation note in §9)
- `AGENTS.md` (§2 hierarchy, §8 cycle model, §11–12 conventions, the spec-metadata format) and
  the templates in `projects/_templates/` (`spec.md`, `stage.md`)
- Your persistent memory carries the direction ("PROJ-002 is engine-led, not crop-led").

## The confirmed scope (do not re-open unless feedback below overrides it)
**PROJ-002 = the focused engine core; ships as 0.3.0.** Zero new default dependencies.
- **STAGE-011 Analysis foundation** — `src/analysis/`: a computed-once, immutable `Analysis` layer
  (histogram, entropy, edge density, alpha coverage, capped unique-colours, dominant colour) + a
  deterministic no-ML **internal classification** (→ three optimization buckets). Specs: **SPEC-046**
  (the layer — already written) and **SPEC-047** (classification + a labeled fixture corpus).
- **STAGE-012 Auto-decide & explain** — format auto-decision inside `optimize` ("the local
  `f_auto`") composing the existing SSIMULACRA2 search across a ≤3-format shortlist, `--profile
  web|docs|preserve`, and a concise `--explain` trace. Specs: **SPEC-048** (auto-decide) and
  **SPEC-049** (explain).
- **Deferred to their own projects:** the goal-driven planner → PROJ-003 (it *generalizes* STAGE-012's
  decision engine — "one decision engine, two entry points"), `lint` → 004, manifest → 005, crop → 006.

## Confirmed adjustments from the 2026-07-05 review (already reconciled — APPLY these)
- **Auto-decide is the DEFAULT** for `optimize` (the marquee). **Migration is a non-issue** (no
  released users yet) — design for the best forward behaviour. Add a **clear-win guard**: the output
  format switches away from the source only when the byte win clears a `FORMAT_SWITCH_THRESHOLD`
  (else keep the source format — no surprising switch for a marginal gain); ALWAYS report the chosen
  format + savings (one line by default, full detail under `--explain`). Explicit overrides:
  `--format`/`-o <ext>` (pin, bypasses the engine) and `--profile preserve` (force format-preserving).
- **`explain` is a FLAG** on `optimize` (`--explain`) — never a subcommand.
- **`Analysis` is a flat, concrete struct** (`Analysis` + `compute()`) — NO `Analyzer` trait, NO
  registry, NO generic trait system. Scrub any residual "trait shapes" wording. A concrete
  `Decision`-style struct is the auto-decide output.
- **Metadata stays pure-Rust** — the shipped `strip`/`clean --gps`/`set` lane. Do NOT introduce
  EXIFTool or any external/system metadata tool (it would break zero-system-deps).
- **(Roadmap-level, not 002's framing):** execution order is now PROJ-002 → **PROJ-004 lint** →
  PROJ-003 (a slim orchestration/dry-run layer reusing the engine + the EXISTING recipe/`apply`
  system, NOT a second decision engine, NOT the PROJ-005 output manifest). No action for 002; just
  don't design STAGE-012 assuming the planner is next.

## Feedback to incorporate (optional)
```
[PASTE any review / red-team / other feedback here, or write "none".]
```
Reconcile it against the confirmed scope: if it **materially changes scope** (adds/removes a stage
or spec, reorders, or challenges the thesis), summarize the change and **get explicit maintainer
confirmation before proceeding** — do not silently re-scope. If it's within scope or a refinement,
fold it in and note where. Default with no feedback: **proceed with the confirmed scope.**

## Task — complete the framing (design cycle; NO `src/` code)
1. **Confirm the brief + both stage files** reflect the confirmed scope; correct any drift. Move
   `project.status` to `active` and `STAGE-011.status` to `active` (per the AGENTS status model),
   noting the date.
2. **Write the remaining specs** — SPEC-047, SPEC-048, SPEC-049 — using the house template
   (`projects/_templates/spec.md`), each in the **design cycle**, each with: Context · Goal (1–2
   sentences) · Inputs · Outputs · **Acceptance Criteria** (testable) · **Failing Tests** (written
   now, before build) · Implementation Context (applicable DECs, constraints, prior work, explicit
   out-of-scope) · Notes. Ground them in the design briefs (format-engine, classification). Keep each
   **S or M** (split anything L). Use the next free SPEC ids and match SPEC-046's style.
3. **Capture the decisions (DEC-*)** the wave needs (or list them explicitly for the build session):
   the classification thresholds + safe-fallback bias (STAGE-011); the decision-tree thresholds +
   profiles + the `ExplainTrace` schema + the "AVIF competes only in byte-budget mode" rule
   (STAGE-012). Use the DEC template + the next free DEC id.
4. **Verify dependencies + the reconciliation seam:** STAGE-012 depends on STAGE-011; both reuse
   `src/quality/` unchanged. Ensure SPEC-048 factors the decision engine cleanly so **PROJ-003's
   planner can WRAP it, not rebuild it** (findings §9). No new default dependency anywhere.
5. **Update the stage backlogs** — spec lists, cycle markers, and counts — so PROJ-002 reads as a
   complete, build-ready backlog. Leave everything in the design cycle.

## Guardrails
- Permissive-only default deps (this wave adds **none**); pure-Rust; zero system deps.
- `Analysis::compute` is a **new untrusted-input surface** → bounded, no-panic, cap `unique_colors`
  (STAGE-006 discipline).
- `Analysis` is a **peer** of the pixel pipeline — **never** on the `Operation` trait, **never** a
  recipe-serialized step (it's derived; this preserves the byte-stable recipe round-trip).
- **Do NOT write `src/` code.** Stop at a fully-framed, build-ready spec set.

## Deliverable
PROJ-002 fully framed and build-ready: brief + 2 stages + **4 specs** (046 already done; 047/048/049
written) in the design cycle, DECs captured or listed with ids, backlogs + counts updated, statuses
set to active. End with a 5-line summary of what's ready and confirm the **recommended first build
target is SPEC-046** (the standalone Analysis layer, all existing tests green).
