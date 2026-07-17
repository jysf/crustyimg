---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-093
  type: bug
  cycle: design
  blocked: false
  priority: high
  complexity: M
project:
  id: PROJ-008
  stage: STAGE-030
repo:
  id: crustyimg
agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-17
references:
  decisions: [DEC-003, DEC-017, DEC-030]
  constraints: [ergonomic-defaults, every-public-fn-tested, test-before-implementation]
  related_specs: [SPEC-027, SPEC-026, SPEC-087, SPEC-089]
value_link: >
  The metadata container lane silently CORRUPTS numeric EXIF tags. `meta clean --gps` — the privacy
  verb, documented to preserve "orientation, copyright, ICC" — rewrites Orientation 6 → 1536, so the
  user's photo displays rotated wrong in every viewer afterward. `meta set` does the same and also
  degrades GPS coordinates to a plausible-but-wrong location. This is silent corruption of user files
  by the verbs whose entire promise is "we only touch what you asked."

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-093: the metadata write path corrupts numeric EXIF tags

## Context

Found by SPEC-089's verify (as an explicitly out-of-scope observation) and **independently reproduced by
the orchestrator at framing**. It is **pre-existing** — the pre-move binary emits identical bytes — and
predates the `meta` group entirely (SPEC-026/027 era).

**Reproduced (2026-07-17, release binary, `bench/corpus/photo_forest_cc0.jpg` + exiftool):**

```
baseline:                 Orientation 6      GPS Latitude 50.4957
after `meta clean --gps`: Orientation 1536   ← corrupted (GPS correctly removed)
after `meta set --artist`:Orientation 1536   GPS Latitude 50.4843223958333   ← corrupted + precision lost
```

`1536 == 6 << 8`. Both `set_tags` and `clean_gps` are affected → the bug is in the **shared container
lane**, `src/metadata/tiff.rs` (the hand-rolled TIFF writer that "replaces `little_exif` for the two
tag-level edits the container lane needs").

**Why this is worse than a cosmetic tag bug:**
- **`meta clean --gps` is the privacy verb.** `docs/api-contract.md` ~326 documents it as *"preserving
  everything else (**orientation**, copyright, ICC)"*. **That documented promise is false.** A user
  strips their location and their photo silently comes back rotated wrong in every viewer.
- **GPS degrades to a plausible-but-wrong coordinate** (50.4957 → 50.4843…), not an obvious error — the
  worst kind of wrong.
- **Orientation is load-bearing in this codebase** (DEC-017; `auto-orient` exists to bake it into pixels).

**Why it survived this long — read this, it is the useful part:**
1. **ASCII tags are immune** (byte order doesn't apply to ASCII). `set` writes Artist/Copyright/
   ImageDescription — all ASCII — so **every existing test checks exactly the tags that can't exhibit the
   bug**. The suite is green and always was.
2. **Byte-identity proofs are structurally blind to it.** SPEC-087/089 proved `meta strip/clean/copy/set`
   byte-identical *to the pre-move binary* — which is equally broken. As SPEC-089's verify put it:
   **identical to the old bytes ≠ correct bytes.** Every proof this stage ran compared against an oracle
   that shares the defect.

**A hypothesis the orchestrator formed and REFUTED at framing — do not re-derive it.** `serialize`
(`src/metadata/tiff.rs` ~331) emits a *normalized little-endian* block (`b'I', b'I'`) *"regardless of the
input's original byte order"*, which suggests inline value bytes are passed through unswapped from a
big-endian input. It explains `6 → 1536` perfectly. **It is wrong:** driving both orders gives the same
corruption (`-ExifByteOrder=MM` → 1536; `-ExifByteOrder=II` → 1536). The bug is **unconditional**, not
input-byte-order dependent. **The mechanism is undiagnosed — diagnose it, don't assume it.**

## Goal

Find and fix the actual mechanism by which the container lane corrupts **non-ASCII** EXIF tags
(SHORT/LONG/RATIONAL), so `meta clean --gps`, `meta set`, and `meta copy` preserve every tag they do not
intend to change — byte-exactly where the value is unchanged. Close the test gap that let ASCII-only
coverage hide it.

## Inputs — files to read

- `src/metadata/tiff.rs` — **the prime suspect**: `serialize` (~331), `put_ifd` (~344), `type_size` (~77),
  the type constants (`TY_LONG`/`TY_ASCII` ~68; note there is no `TY_SHORT` constant though the comment at
  ~72 lists `3=SHORT`), and the parse side. **Read the whole parse→serialize round-trip before theorizing.**
- `src/metadata/mod.rs` — `set_tags`, `clean_gps`, `strip_all`, `copy_metadata` and the container lane.
- `docs/api-contract.md` ~318–345 — the `meta strip`/`clean`/`set`/`copy` contracts, incl. the false
  "preserving … orientation" claim to reconcile.
- `DEC-003` (container-lane model), `DEC-017` (orientation is an image op — why the tag matters),
  `DEC-030` (JPEG-only `meta copy`).

## Outputs

- **`src/metadata/tiff.rs`** (or wherever the diagnosis lands) — the fix. **Round-trip fidelity is the
  invariant:** a tag the operation does not target must serialize back **byte-identically**, for every
  TIFF type, for both input byte orders.
- **Tests that would have caught this** — the real deliverable alongside the fix. Coverage must include
  **non-ASCII types**: SHORT (Orientation), RATIONAL (GPS coordinates), LONG, and both input byte orders.
- **`docs/api-contract.md`** — reconcile the `meta clean` contract with reality once the fix lands (the
  promise becomes true rather than deleted).
- **A DEC** (next free at build — 090/091 also reserve numbers; take the next actually-free) recording the
  mechanism, the fix, and the **testing-gap lesson**: ASCII-only fixtures cannot exercise a byte-order/
  numeric-encoding path, and byte-identity vs a shared-defect oracle proves nothing about correctness.

## Acceptance Criteria

- [ ] **`meta clean --gps` preserves Orientation exactly** (6 stays 6) — driven end-to-end with an
      independent tool (exiftool), not just our own reader.
- [ ] **`meta set --artist/--copyright/--description` preserves Orientation AND GPS exactly** (no
      precision loss: 50.4957 stays 50.4957 to the source's precision).
- [ ] Round-trip fidelity for **every** TIFF type present in a real fixture — SHORT, LONG, RATIONAL,
      ASCII, UNDEFINED — for **both** input byte orders (`MM` and `II`). Untargeted tags byte-identical.
- [ ] `meta copy` and `meta strip` re-checked for the same defect (copy grafts EXIF; strip removes all —
      confirm each is correct, don't assume).
- [ ] A test exists that **fails on the pre-fix code** — demonstrate it (the whole point is that the
      current suite is green while the bug ships).
- [ ] Verified against an **independent decoder** (exiftool and/or `sips`), because our own reader may
      share the writer's misunderstanding.
- [ ] `docs/api-contract.md`'s `meta clean` "preserving … orientation" claim is **true as written**.
- [ ] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`,
      `cargo build --no-default-features`, `just validate` pass.

## Failing Tests (written at design)

- **`src/metadata/tiff.rs` / `tests/metadata.rs`**
  - `clean_gps_preserves_orientation` — Orientation 6 in, Orientation 6 out. **This fails today.**
  - `set_tags_preserves_orientation_and_gps` — Orientation and GPS coordinates survive an ASCII-tag write
    with no value change and no precision loss. **Fails today.**
  - `tiff_roundtrip_is_byte_identical_for_untargeted_tags` — parse→serialize with no edit reproduces every
    untargeted tag byte-exactly, across SHORT/LONG/RATIONAL/ASCII/UNDEFINED.
  - `tiff_roundtrip_handles_both_byte_orders` — `MM` and `II` inputs both round-trip correctly.

## Implementation Context

### Decisions that apply
- `DEC-003` — the container lane writes metadata without a pixel re-encode; this bug is inside it.
- `DEC-017` — `auto-orient` bakes orientation into pixels *and clears the tag*; a corrupted Orientation
  tag is therefore a real, user-visible rotation bug, not a cosmetic detail.
- `DEC-030` — `meta copy` is JPEG-only (PNG EXIF chunk incompatibility) — scope the copy re-check to that.

### Constraints
- `test-before-implementation` / `every-public-fn-tested` — the test gap **is** the story here.

### Out of scope (this spec)
- The `meta` group's **surface** (SPEC-087/089 — shipped, correct; this is the layer beneath).
- Adding new metadata capabilities or tags.
- The audit report / bench (SPEC-088); `web` never-bigger (SPEC-090); AVIF threads (SPEC-091).

## Notes for the Implementer
- **Diagnose before you theorize.** The orchestrator's byte-order hypothesis was clean, explained the
  observed `6 → 1536` exactly, and was **refuted by one test**. Read the whole parse→serialize round-trip
  and find the real mechanism ([[read-whole-function-before-asserting-a-gap]],
  [[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]]).
- **Do not trust our own reader to grade the fix** — it may share the writer's assumption. Use exiftool
  ([[verify-wasm-output-with-an-independent-decoder]] generalizes: verify with a decoder you didn't write).
- **The test gap is the deliverable as much as the fix.** ASCII-only fixtures made a green suite that
  shipped corruption for a month ([[a-harness-that-exercises-nothing-reports-green]] — the tests ran, but
  never touched a type that could fail).
- Expect the fix to be small and the *coverage* to be the work.

---

## Build Completion
- **Branch:** · **PR:** · **All acceptance criteria met?** · **New decisions:** · **Deviations:** · **Follow-ups:**
### Build-phase reflection
1. <answer> 2. <answer> 3. <answer>

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
