---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-093
  type: bug
  cycle: verify
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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 300000
      estimated_usd: 3.00
      recorded_at: 2026-07-16
      note: >
        main-loop build session (not a metered subagent) — ORDER-OF-MAGNITUDE ESTIMATE per
        docs/cost-tracking.md's autonomous-run guidance (no subagent_tokens available).
        Diagnosed the mechanism empirically rather than from the spec's stated one: the spec's
        REFUTED byte-order hypothesis was in fact CORRECT — its refutation was an artifact of
        exiftool's `-ExifByteOrder` being silently ignored on files that already carry EXIF, so
        the "II" arm was still MM. Also found the spec's cited repro file
        (bench/corpus/photo_forest_cc0.jpg) carries no EXIF at all; seeded MM/II fixtures with
        exiftool instead. Fix: `Tiff::byte_order`, preserved through serialize (DEC-076, amends
        DEC-046). Wrote 8 failing tests first behind a serialize-independent fixture builder
        (src/metadata/tiff/fixture.rs), watched them fail, then fixed; MUTATION-TESTED all 8 by
        reverting the fix in place. Graded end-to-end with exiftool 13.55 (independent decoder)
        across clean/set/copy/strip × MM/II. Found a third unreported symptom (IFD1 thumbnail
        length 6430 → 504954880, dangling pointer). Gates green (test default 745 / avif 758,
        clippy, fmt, no-default-features, `just validate`, decisions-audit 0 structural errors).
        Rate: Opus blended (~$9/MTok, 80/20 in/out, no cache discount, per AGENTS.md §4).
  totals:
    tokens_total: 300000
    estimated_usd: 3.00
    session_count: 1
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

- [x] **`meta clean --gps` preserves Orientation exactly** (6 stays 6) — driven end-to-end with an
      independent tool (exiftool), not just our own reader. *Release binary + exiftool: MM input →
      Orientation **6**, GPS removed, Artist preserved. II likewise.*
- [x] **`meta set --artist/--copyright/--description` preserves Orientation AND GPS exactly** (no
      precision loss: 50.4957 stays 50.4957 to the source's precision). *exiftool: MM → Orientation **6**,
      GPSLatitude **50.4957**, GPSLongitude **4.4699**, Artist updated.*
- [x] Round-trip fidelity for **every** TIFF type present in a real fixture — SHORT, LONG, RATIONAL,
      ASCII, UNDEFINED — for **both** input byte orders (`MM` and `II`). Untargeted tags byte-identical.
      *`every_type()` fixture covers all five + sub-IFDs (ExifIFD/GPS) + IFD1 thumbnail, both orders.*
- [x] `meta copy` and `meta strip` re-checked for the same defect (copy grafts EXIF; strip removes all —
      confirm each is correct, don't assume). *Both **confirmed correct** — they operate at the
      segment level and never reach the TIFF writer. Pinned by
      `copy_metadata_preserves_big_endian_numeric_tags` + `strip_all_on_big_endian_removes_everything`
      and by exiftool on real MM files. The copy test correctly does **not** fail under the mutation —
      that is the evidence it was unaffected, rather than an assumption.*
- [x] A test exists that **fails on the pre-fix code** — demonstrate it. *All **8** new tests fail with
      the fix reverted in place (mutation-tested; list in Build Completion). `clean_gps_preserves_orientation`
      pre-fix: `left: Some(1536), right: Some(6)`.*
- [x] Verified against an **independent decoder** (exiftool and/or `sips`). *Graded end-to-end with
      **exiftool 13.55**; unit/integration tests grade with **kamadak-exif**, not our own reader.*
- [x] `docs/api-contract.md`'s `meta clean` "preserving … orientation" claim is **true as written**.
      *Reconciled — and its stale `little_exif` attribution corrected (DEC-046 removed that crate).*
- [x] `cargo test` (default **and** `--features avif`), `cargo clippy`, `cargo fmt --check`,
      `cargo build --no-default-features`, `just validate` pass. *745 / 758 / clean / clean / clean.*

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

- **Branch:** `spec-093-metadata-corruption` · **PR:** #94 · **All acceptance criteria met?** **Yes** (8/8, each with evidence above) · **New decisions:** **DEC-076** (amends DEC-046) · **Deviations:** two, both recorded below · **Follow-ups:** two, filed below

### The mechanism (diagnosed, not assumed)

`parse` stores each entry's value bytes **verbatim in the input's byte order** (the opaque-value model
that lets unknown tags round-trip). `serialize` emitted a hardcoded `II` header — DEC-046's
"normalize to little-endian" — while copying those big-endian bytes straight through. **Nothing
byte-swapped them.** The header ends up lying about its own contents:

- **Orientation** (SHORT): `0x00 0x06` read LE = `0x0600` = **1536** = `6 << 8`.
- **GPS** (RATIONAL): stays *plausible*, which is why it's the worst symptom. `50° 29' 44.52"` is
  `50/1, 29/1, 1113/25`. Byte-reversing a `u32` whose value fits in the low byte multiplies it by
  `2^24` — and a RATIONAL is a **ratio**, so the factor cancels: `50/1` → `838860800/16777216` = exactly
  50. Degrees and minutes survive by arithmetic luck; `1113` needs two bytes, so seconds drift
  `44.52"` → `3.56"`. Output: a well-formed coordinate ~1.3 km off.
- **Thumbnail length** (LONG): 6,430 → **504,954,880**, dangling the IFD1 pointer. A **third symptom
  nobody had reported**, found while fixing this and fixed by the same change.

### ⚠️ The spec's "REFUTED" hypothesis was correct — the refutation was the error

SPEC-093 instructed: *"`-ExifByteOrder=MM` → 1536 and `-ExifByteOrder=II` → 1536. The bug is
unconditional… do not re-derive it."* **The bug is byte-order dependent.** Measured: MM → 1536,
II → **6, correct**.

The framing's experiment was broken, not its hypothesis: **exiftool's `-ExifByteOrder` only applies
when EXIF is created from scratch — on a file that already carries EXIF it is silently ignored.** The
"II" arm was still an MM file. Verified directly: `-ExifByteOrder=II` on an MM-EXIF JPEG leaves it MM.
The refutation had the *shape* of evidence (two arms, same result) while testing one condition twice.

Also corrected at build: the spec's repro cites `bench/corpus/photo_forest_cc0.jpg`, but the committed
corpus copy **carries no EXIF at all** (exiftool: no Orientation, no GPS) — it cannot host this repro.
Fixtures were seeded from it with exiftool instead.

### The fix

`Tiff` gains a `byte_order`; `serialize` emits the header — and every tag/type/count/offset it
writes — in the order the block was parsed in. Values keep passing through verbatim. `minimal()`
(no existing EXIF) stays little-endian. **Rejected** the alternative of byte-swapping values into a
canonical LE form: that forces the writer to *understand* every value it touches, which is exactly
what the opaque model avoids — any type modelled wrong, or not modelled (`type_size` treats unknown
codes as byte-sized), would be silently mangled. Preserving the order makes round-trip fidelity hold
for **every** type, including ones this module has never heard of. See DEC-076.

### The coverage (the real deliverable)

`src/metadata/tiff/fixture.rs` — a TIFF builder **deliberately independent of `serialize`**. It takes
*typed* values (`V::Short(6)`, not "these two bytes") and encodes them itself in the requested order.
The old fixtures were all seeded by calling `serialize`, which could only emit LE — so the suite had
**no big-endian fixture at all**, and its one Orientation assertion checked `is_some()`, never `== 6`.
It could not fail.

**All 8 new tests mutation-tested** — reverting the fix in place (`let le = true;`) fails every one:

```
metadata::tests::clean_gps_preserves_orientation
metadata::tests::set_tags_preserves_orientation_and_gps
metadata::tiff::tests::tiff_roundtrip_handles_both_byte_orders
metadata::tiff::tests::tiff_roundtrip_is_byte_identical_for_untargeted_tags
metadata::tiff::tests::serialize_preserves_declared_byte_order
metadata::tiff::tests::roundtrip_preserves_ifd1_thumbnail_for_both_byte_orders
metadata::tiff::tests::set_ascii_tag_on_big_endian_preserves_numeric_tags
metadata::tiff::tests::remove_gps_on_big_endian_preserves_numeric_tags
```

Coverage spans SHORT · LONG · RATIONAL (single + GPS triplet) · ASCII · UNDEFINED, sub-IFDs
(ExifIFD/GPS), the IFD1 thumbnail, and **both** byte orders — graded by kamadak-exif, then by exiftool
end-to-end.

Note `tiff_roundtrip_is_byte_identical_for_untargeted_tags` fails **only** on its byte-order
assertion: the value bytes round-trip identically even pre-fix, because it was the *header* that moved
underneath them. A byte-identity test without an order assertion would have passed on the broken code —
the same shape of blindness as SPEC-087/089's proofs against a shared-defect oracle.

### Deviations

1. **Took DEC-076, not DEC-075.** The spec said "next actually-free". DEC-075 is on disk nowhere, but
   **SPEC-090 names it in its acceptance criteria** and SPEC-091 reserves "DEC-075+" — taking it would
   collide with an open spec. 074 is the highest on disk; 072/073 are gaps/claimed. Cost of the gap: nil.
2. **Also corrected `docs/api-contract.md`'s `little_exif` attribution** for `meta clean`/`meta set`.
   Not in the spec's scope, but those are the exact lines the spec sent me to reconcile and the crate
   was removed by DEC-046 — leaving it would have made the reconciled sentence false in a new way.

### Follow-ups (not this PR — `one-spec-per-pr`)

1. **Repo-wide `little_exif` doc drift.** The crate is gone from `Cargo.toml`, but `AGENTS.md` (§5 tech
   stack, §14 glossary), `docs/architecture.md` (×4, incl. the dep table and a Mermaid diagram), and
   `docs/data-model.md` (×2) still list it as a live dependency. Found by grep; out of scope here.
   (`docs/sessions/**` and `docs/backlog.md` are legitimately historical — leave them.)
2. **`meta copy`'s PNG limitation cites a crate that no longer exists.** `docs/api-contract.md` ~345
   and DEC-030 justify JPEG-only by "`little_exif`/`img-parts` use incompatible PNG EXIF chunks". With
   `little_exif` gone the rationale may no longer hold — PNG `meta copy` might now be free. Worth a
   frame; needs its own spec, not a doc edit.

### Build-phase reflection

1. **What surprised me?** That the spec's *most emphatic* instruction — "REFUTED… do not re-derive it"
   — was the wrong one, and that the error was one layer below where anyone was looking. The hypothesis
   was right; the *tool* silently no-op'd the control. The generalizable lesson isn't about EXIF: **a
   control you never verified applied is not a control.** The refutation looked like strong evidence
   (two arms, same result) while testing one condition twice. It cost one command to check — and the
   spec's own instruction to grade with an independent decoder is what made checking natural.

2. **What was hardest?** Deciding the failing test's *shape*. The obvious one — "round-trip reproduces
   every value byte" — **passes on the broken code**, because the bytes were never what changed. I only
   caught it by asking what pre-fix output would actually look like before writing the assertion. That's
   the same trap that made SPEC-087/089's byte-identity proofs certify the defect, reappearing one level
   down. The invariant had to be *semantic* (decode per the block's own declared order), not byte-level.

3. **What would I do differently?** I'd have reached for mutation-testing sooner. I wrote 8 tests and
   *believed* they covered the bug; reverting the fix in place is what turned that into evidence — and
   it paid twice, once by proving the 7 real tests bite, and once by proving `meta copy` was genuinely
   unaffected (its test correctly *doesn't* fail) rather than my assuming it. "Confirm each is correct,
   don't assume" has a mechanical answer, and it's cheap.

---

## Reflection (Ship)
1. <answer> 2. <answer> 3. <answer>
