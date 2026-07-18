---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-075
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-17
supersedes: null
superseded_by: null

affected_scope:
  - "src/analysis/decide.rs"
  - "src/cli/mod.rs"
  - "recipes/web.toml"
  - "recipes/gallery.toml"
  - "recipes/product.toml"
  - "docs/cli-reference.md"
  - "docs/api-contract.md"
  - "README.md"

tags:
  - web
  - optimize
  - never-bigger
  - audit-report
  - honest-reporting
  - docs
---

# DEC-075: `web`'s size baseline is the dimension contract, and a larger-than-source output is surfaced, not promised away

## Decision

**`web`'s dimension contract wins over an unconditional never-bigger promise (option A), and the
larger-than-original case is made visible — it is not enforced away.**

`web` (and its siblings `gallery`/`product`, the same terminal-`optimize` downscale flow) downscales the
long edge to a dimension bound, then picks the smallest modern format that beats the **downscaled** image
(`Mode::Fast`, DEC-069). When the source is already small for its dimensions and lies **above** that bound,
the downscaled re-encode can exceed the original file. There is **no fallback that satisfies both
contracts**: passing the original through would return a file **above** the bound the user explicitly asked
to shrink. The downscale is the user's instruction, so it is kept.

Concretely:

1. **The docs no longer promise "never bigger" for `web`.** Every rendered `web`/`gallery`/`product` claim
   now states the real guarantee: downscale to a dimension bound, pick the smallest format that beats the
   **downscaled** image, and report the size honestly. Each also points at `optimize` for an *unconditional*
   never-bigger guarantee that keeps dimensions. Fixed surfaces (grep of the live surface,
   `never bigger|never larger|never ship`): the `web` clap `--help` (`src/cli/mod.rs`), `recipes/web.toml`
   header + `description`, `recipes/gallery.toml`, `recipes/product.toml`, `docs/cli-reference.md`,
   `docs/api-contract.md`, `README.md`, `docs/recipes.md`, and the `src/recipe/bundled.rs` module doc.

2. **A larger-than-source output is signalled two ways** (SPEC-084's honest-negative-savings machinery,
   extended):
   - **`--json`:** a new additive, **gated** boolean `"larger_than_source":true` rides the
     `optimize.explain/v1` report **only when the shipped output exceeds the source** — placed after
     `savings_percent`, before `ssim`/`timing`. A normal (smaller / break-even) run's JSON is
     byte-identical (the same discipline as `ssim` — SPEC-086 — and `timing` — SPEC-088/DEC-074). The flag
     is derived from `ExplainTrace::exceeds_source()` (`out_bytes > source_bytes`).
   - **stderr:** an explicit `note:` line on **every** channel (default, `--json`, `--explain`), respecting
     `--quiet`, telling the user the shipped output is larger than the source and why (the source could not
     ship unchanged: metadata stripped / orientation baked / resized to the requested bound). Under
     `--json` — whose report owns stdout — the stderr note is the human heads-up.
   - `savings_percent` continues to read honestly negative ("N% larger"), never clamped to a break-even 0
     (SPEC-084 not regressed).

3. **The signal is size-derived and mode-independent.** It fires for any auto-decision flow whose output
   exceeds the source: `web`'s downscale case, and `optimize`'s rare metadata-forced re-encode (SPEC-084).
   This is consistent honesty — `optimize` already reported "N% larger" for that case; the flag/note just
   make it explicit and queryable.

## Rationale

- **The dimension request is the user's explicit instruction.** Silently returning a >bound file (option B)
  contradicts what `web` is for; an honest "larger" report is the lesser evil. Option B would additionally
  need to make the dimension-contract violation loud — trading one silent surprise for another.
- **The machinery already existed.** SPEC-084 built honest negative savings (`savings_percent` goes
  negative, `size_delta_phrase` says "larger"); this spec adds the *explicit* flag + note on top, rather
  than a new enforcement path. No change to `pick_winner`, `solve_candidate`, or the encode path.
- **Additive/gated keeps the schema stable.** Following the `ssim`/`timing` precedent, the flag is absent
  on the common (smaller) path, so a normal run's JSON is byte-identical and existing consumers are
  unaffected.

## Alternatives considered

- **(B) Enforce never-bigger against the original + passthrough the original when it wins.** Rejected: for
  `web` this returns a file **above** the requested dimension bound — a silent violation of the downscale
  contract, which the spec required to be "loud, not silent". Option A honors the explicit instruction and
  is honest about the byte cost.
- **Infer "larger" from `savings_percent < 0` (no new field).** Rejected in favour of an explicit boolean:
  the spec asked for a dedicated additive field, and an explicit flag reads clearly to a machine consumer
  without re-deriving intent from a signed percentage.
- **Rewrite `optimize`'s never-bigger wording too.** Out of scope: the spec fixed `optimize`'s guarantee as
  correct-as-is and unchanged. `optimize` keeps dimensions and re-encodes the same pixels; its only
  larger-than-source case is the rare metadata-forced re-encode (SPEC-084), and its unconditional
  keep-dims never-bigger *primitive* (passthrough when nothing beats the source and nothing forces a
  re-encode) is untouched and still stated. Proven byte-identical to the pre-spec binary.

## Consequences

- `web`/`gallery`/`product` may ship a larger-than-original file for an already-small, heavily-compressed
  `>bound` source — now documented and signalled, no longer contradicted by the docs.
- `web <x>` == `apply --recipe web <x>` (DEC-070) holds byte-for-byte, image **and** `--json` report
  (including the new flag) — verified against the built binary.
- `optimize` is byte-identical to the pre-spec binary on the same inputs (keep-dims primitive untouched) —
  verified against the parent-commit (`0eac2cf`) release binary.
- JSON consumers gain an optional `larger_than_source` field; the base schema is unchanged when the output
  is not larger. The `json_shape_consistent_across_verbs` test (SPEC-088) moved to a fixture all verbs
  shrink, so the golden key set documents the **base** schema (the flag being data-driven, not
  flag-driven, legitimately appears for one verb and not another on a mixed fixture).
