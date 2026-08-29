---
project:
  id: PROJ-012
  status: proposed
  priority: medium
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-08-23
shipped_at: null

value:
  thesis: >
    Every defect crustyimg's lint can name, crustyimg should be able to act on.
    Nine of its eleven rules already have a fix in the tool — `optimize` answers
    `size/oversized-bytes`, `resize` answers `dims/oversized-dimensions`,
    `meta clean` answers `privacy/gps-metadata-leak`, `auto-orient` answers
    `orient/orientation-not-baked`. **Two families do not.**
    `format/animated-gif` and `format/animated-input` flag an animation the tool
    cannot re-encode, because it has no animated output format at all — and
    `lint` then recommends the command that flattens it.
    `color/wrong-colorspace` flags a colour space the tool cannot convert.
    A linter that names a defect it cannot fix is a diagnosis without a
    treatment, and closing those two gaps is a bounded, testable claim rather
    than a wish to add features.
  beneficiaries:
    - "Anyone whose `lint` run reports an animated input, because today the only advice available is a command that discards every frame but the first"
    - "Anyone with a wrong-colourspace image, told about it by a tool that cannot convert it"
    - "Users shipping animated content to the web — measured at 11.2x smaller than GIF and 6.3x smaller than animated WebP at higher quality"
    - "The lint framework itself, which gains the property that its rule roster and its capability set agree"
  success_signals:
    - "`crustyimg lint` on an animated GIF names a fix the tool can actually perform, and performing it preserves every frame — asserted with an independent decoder, not the encoder's packet count"
    - "`color/wrong-colorspace` can be answered by a crustyimg command rather than by external tooling"
    - "A test enumerating all 11 lint rules asserts each one either names an actionable fix or documents why none exists — so a new rule cannot be added without answering the question"
  risks_to_thesis:
    - "⚠ The animated-AVIF muxer is measured at ~1,000 lines on top of a box library, not the 150-250 an in-house RIFF estimate suggests, and needs a `mp4-atom` DEC. It is the largest single piece of work in either open project"
    - "⚠ ICC transforms must use `qcms` only — `lcms2` bindings would breach the zero-system-dependency property the whole identity rests on (DEC-088). If `qcms` turns out not to cover the needed transforms, the stage stalls rather than substitutes"
    - "The third rule family that cannot be fixed — `size/truncated-or-corrupt` — is genuinely unfixable (nothing can un-truncate a file). The thesis must accommodate 'documents why not' or it is false on its own terms"
    - "⚠ This project's members were decided in a document nothing reads and were invisible for weeks. If it is created and then not sequenced, it reproduces the failure it was created to fix"
---

# PROJ-012: Lint Can Fix What It Names

## What This Project Is

`crustyimg lint` names eleven defects. **Nine of them the tool can already act on:**

| rule | the fix crustyimg already has |
|---|---|
| `size/oversized-bytes` | `optimize` |
| `dims/oversized-dimensions` | `resize` |
| `orient/orientation-not-baked` | `auto-orient` (SPEC-110) |
| `privacy/gps-metadata-leak` | `meta clean` |
| `privacy/camera-metadata` | `meta strip` |
| `color/missing-icc`, `color/unexpected-icc` | the metadata lane |

**Two families it cannot.** `format/animated-gif` and `format/animated-input` flag an animation
crustyimg has no way to re-encode, because **it has no animated output format at all** — and until
SPEC-119, `lint` actively recommended the command that discarded every frame but the first.
`color/wrong-colorspace` flags a colour space crustyimg cannot convert.

**A linter that names a defect it cannot fix is a diagnosis without a treatment.** This project
closes both gaps.

## Why Now

**Both features were already decided and both were invisible.** `docs/feature-set-triage-2026-08.md`
marked animated output and ICC transforms as takes — *"Closes `format/animated-gif`, which today
detects a problem it cannot fix"* and *"Closes `color/wrong-colorspace` the same way §11 closes its
rule"* — and that document is read by no command. A 2026-08-23 audit found **8 of its 12 decisions
invisible to `just backlog`**. This project is the home two of them needed.

**The animated half is measured, not aspirational:** a 308,156 B / 36-frame GIF → **27,564 B at
SSIMULACRA2 86.7** (**11.2×**); animated WebP's best point was 172,492 B at 84.1, so **AVIF is 6.3×
smaller at higher quality**. The path is pure Rust and patent-clear — `rav1e` encodes, `re_rav1d`
decodes, both already in the tree. ⚠ **There is no pure-Rust animated WebP encoder at all**
(`image-webp` 0.2.4 writes `VP8X` but emits no `ANIM`/`ANMF`), which is why AVIF and not WebP.

## Success Criteria

- `crustyimg lint` on an animated GIF **names a fix the tool can perform**, and performing it
  **preserves every frame** — asserted with an **independent decoder** (`re_rav1d`), not the
  encoder's own packet count.
- **`color/wrong-colorspace` can be answered by a crustyimg command**, not by external tooling.
- **A test enumerating all 11 rules** asserts each either names an actionable fix or documents why
  none exists — so a **new rule cannot be added without answering the question.** ⚠ That test is
  what makes the thesis hold over time rather than being true once.

## Scope

### In scope
- Animated AVIF output: muxer, per-frame timing, loop count, reusing the existing quality search.
- ICC colour transforms via **`qcms` only**.
- The rule-roster completeness test above.

### Explicitly out of scope
- **Animated WebP output** — no pure-Rust encoder exists.
- **`lcms2`** — system-dependency bindings breach DEC-088. `qcms` or nothing.
- **Alpha in animated output** — refused explicitly, deferred to a follow-on.
- **Video containers, or any codec not already shipped** (DEC-088).
- ⚠ **The other decided-but-homeless items** from the same triage doc — the `.cube` LUT op,
  declared convolution kernels, JPEG XL decode, SVG optimization, the perceptual-dedup lint rule,
  the MCP server, the brand-consistency gate. **They do not share this thesis and are not parked
  here.** Per AGENTS §10's rule, with no home they are **proposals, not decisions**, and the audit
  table in `docs/feature-set-triage-2026-08.md` §0 now says so. ⚠ **Adding them here would make
  this a filing cabinet**, which is the failure mode the rule exists to prevent — not a licence to
  create one.

## Stage Plan

- [ ] (not yet defined) — **Animated output.** The muxer and the frame/timing round-trip, then the
  quality search and speed pinning. ⛔ **Blocked on a `mp4-atom` DEC** (`no-new-top-level-deps-without-decision`,
  licence gate DEC-018) **and on splitting `docs/research/draft-spec-animated-avif-output.md`**,
  which is marked complexity **L** with its own note saying *"L means split it"*.
  ⚠ **Budget the muxer at ~1,000 lines, measured** — `mp4-atom` supplies boxes, not a muxer, and
  the sample-table bookkeeping is yours. ⚠ **Speed 10 is a trap for animation** — do not inherit
  the still-image intuition; the draft measured this. ⚠ **Run a near-lossless colour-range control
  first** — a near-lossless encode scoring 57.2 was traced to `Range: Limited` against full-range
  input and scored 96.5 after the fix.
- [ ] (not yet defined) — **ICC colour transforms (`qcms`).** Closes `color/wrong-colorspace`.
- [ ] (not yet defined) — **The rule-roster completeness test.** ⚠ Cheap, and it should land
  **first** — it is what turns this thesis into something that stays true.
  📌 The roster is **11 rules, not 9**: `privacy/gps-metadata-leak` and `size/truncated-or-corrupt`
  live in `src/lint/mod.rs`, **not** `src/lint/rules.rs`, **and they are the two Error-severity
  rules that gate CI**. Any sweep scoped to `rules.rs` misses the load-bearing half. Authoritative:
  `grep -rn 'fn id(&self)' -A1 src/lint/`.

**Count:** 0 shipped / 0 active / **3 pending** — re-derive with a grep you just ran.

## Design Notes

- ⚠ **`size/truncated-or-corrupt` is genuinely unfixable** — nothing can un-truncate a file. The
  thesis therefore says *"names an actionable fix **or documents why none exists**"*. Without that
  clause it is false on its own terms, and the completeness test would be unpassable.
  📌 Related and already filed on PROJ-010: that rule **does not currently fire** on a truncated
  JPEG, because the file decodes successfully with a warning and the rule keys on `Err`.
- **The independent-decoder rule is load-bearing.** An encoder reporting N packets is not evidence
  of N frames. `re_rav1d` is in the tree and is the oracle.
- **Browser support for animated AVIF is a moving fact.** caniuse's 94.65 % measures **still** AVIF
  and does not break out sequences — an upper bound, not the number. Re-check before any published
  claim.

## Dependencies

### Depends on
- ⛔ **A `mp4-atom` DEC** — blocks the animated stage entirely.
- **PROJ-011 landing first is preferable but not required.** PROJ-011 teaches the registry its
  first parameter-rich op; an animated encoder's configuration is the same shape, so inheriting
  that precedent is cheaper than inventing it.

### Enables
- `lint` advice that is true rather than destructive.
- The animated-image research banked by PR #177 becoming shipped capability.

## Project-Level Reflection

*Filled in when status moves to shipped.*
