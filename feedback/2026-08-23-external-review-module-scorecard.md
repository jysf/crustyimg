---
source: external LLM code review (batch 3) — module scorecard + "top 5 zero-trade-off fixes"
received: 2026-08-23
scope: crustyimg 0.7.1, whole-repo module grades and refactor recommendations
triaged_by: orchestrator session f20dabb9, 2026-08-23
---

# External review batch 3 — module scorecard

**2 of 5 recommendations hold. The one it says to do first does not exist.**

The review hedges throughout — *"likely has"*, *"current state (inferred)"*, *"current state
(suspected)"*, *"assumed to exist based on release notes"* — and each hedge marks a place it
reasoned from a module name instead of reading the module. Every claim below was checked against
the code.

| # | recommendation | verdict |
|---|---|---|
| 1 | Unify `sink` duplication behind `encode_to_writer<W>` | ❌ **refuted — the delegation already exists** |
| 2 | Typed per-operation params via serde | ✅ **actionable** |
| 3 | `#[must_use]` on fallible constructors | ❌ rationale is false |
| 4 | Split `Image` into data + meta | ⚠ observation true, proposal conflicts with DEC-002 |
| 5 | Feature-gate `quality/` behind `bench` | ❌ **would delete shipped features** |
| — | `#[non_exhaustive]` on public error enums (in the scorecard, not the top 5) | ✅ **best item in the review** |

---

## Actionable

### `#[non_exhaustive]` on the public error enums — the most valuable item, and it is buried

**Confirmed:** `src/error.rs:20` `pub enum ImageError`, `src/quality/mod.rs:61` `pub enum
QualityError`, `src/sink/mod.rs:202` `pub enum SinkError` — **none carries `#[non_exhaustive]`**.
The single occurrence in `src/` is a doc comment about an unrelated struct.

crustyimg is a **published crates.io library**. Adding one error variant is a breaking change for
every downstream `match`. This is a real semver hazard, it is cheap, and the review put it in a
scorecard footnote rather than its top 5.

⚠ Adding it is **itself** a breaking change for exhaustive matchers, so it wants a major/minor
boundary — cheapest to do at the next version bump, not mid-cycle.

### Typed per-operation parameter structs (#2)

**Confirmed:** `OperationParams` exposes `get_str` (`src/operation/mod.rs:58`) and `get_u32` (`:63`),
with a hand-written `Deserialize` impl at `:98`. Each operation re-implements its own validation.

The payoff the review names is the real one and it is not stylistic: **schema errors surface at
recipe-parse time rather than partway through a batch.** That is precisely the failure a batch
recipe user hits, and it lands squarely inside **PROJ-011 STAGE-049**, which already has to give
`Recipe` a format and quality field. **Scope it with that work, not separately.**

---

## Refuted, with the check

### #1 — the flagship recommendation describes duplication that is not there

The review asserts `encode_to_bytes` and `write_to_file` "duplicate ~90% of the encoding logic",
supplies an invented before/after, and says it would tackle this first.

**There is no `write_to_file`.** `Sink::write` (`src/sink/mod.rs:457`) **calls
`encode_to_bytes(img, fmt, quality)?`** — it delegates. `Sink::write_bytes` (`:609`) writes
already-encoded bytes and never encodes. **The single-source-of-truth architecture the review
proposes is the one the code already has.** The five `write_with_encoder` sites are the per-format
arms of one match inside `encode_to_bytes_with`, not a second copy of anything.

### #3 — the stated rationale is false

*"If a user calls them and ignores the `Result`, the compiler won't warn them."* **`Result` is
already `#[must_use]` in std**, so ignoring `Pipeline::new()` warns today. `#[must_use]` would only
add value on non-`Result` returns, which the review does not identify. Zero `#[must_use]` in `src/`
is accurate; the consequence claimed for it is not.

### #5 — would remove shipped functionality

**Half-true and dangerous.** `ssimulacra2 = "0.5.1"` **is** an unconditional dependency
(`Cargo.toml:75`) — that observation is correct. But the review's scorecard describes `quality/` as
*"SSIMULACRA2 harness / benchmarking helpers"*, and that is **not what the module is**. It powers
`optimize --verify`, the `ssim` figure in `web`'s report, and the `--target`/`--ssim`/`--max-size`
quality search. **Feature-gating it behind `bench` deletes those from the default binary** — the
opposite of a "zero-quality-trade-off" refactor.
`imagequant`, the other dependency it names, **is not in `Cargo.toml` at all**. `criterion` is
already a native-only **dev**-dependency (`:313`), so it is not in the release binary either. The
claimed "15–20% binary size, 30% compile time" saving rests on two crates that are not there and
one that cannot be removed.

### The testing grade rests on an admitted non-reading

*"tests/ (integration) — **Assumed to exist based on release notes**"* and *"Missing: CLI
integration tests using `assert_cmd`."* There are **37 files under `tests/`**, and at least five —
`edit.rs`, `apply_batch.rs`, `colour_type_preservation.rs`, `hostile_inputs.rs`, `audit_bench.rs` —
**drive the real binary** via `CARGO_BIN_EXE`. The 0.7.1 gate run was **936 tests**. Not using the
`assert_cmd` crate specifically is a style choice, not a missing harness.

---

## Partially true

### #4 — `Image` does carry metadata; the proposed split still conflicts with DEC-002

**Confirmed:** `src/image/mod.rs:240` — `Image { pixels, source_format, metadata:
Option<MetadataBundle>, truncated_jpeg, .. }`. So *"holds both the pixel buffer and metadata"* is
accurate, and a `data` / `meta` split is more defensible than **batch 1's** `ImageCore` /
`ImageMetadata` / `ImageOperations` proposal — this one does not try to move operations onto the
struct, which is what DEC-002 forbids.

⚠ Still not scheduled, for the reason given to batch 1: `image/mod.rs` is **797 production lines**
while the repo's own measured decomposition target, `src/cli/optimize.rs`, is **1,876**. Ranking by
God-Object smell instead of by measurement picks the wrong file.

---

## The pattern across three review batches

**Every one has led with a recommendation it inferred rather than read** — batch 1 asked for three
fuzz targets that already exist; batch 2's headline optimizer-monotonicity concern did not
reproduce; batch 3's flagship `sink` refactor is already the architecture. In all three the
*genuinely* valuable items were the quiet ones: a wasm CI leg, a resource-cap arithmetic gap, and
`#[non_exhaustive]` in a scorecard footnote.

**Worth carrying into how the next batch is requested:** ask for findings that cite a file and line
they actually read, and treat any recommendation containing *"likely"*, *"inferred"* or *"assumed"*
as a hypothesis to drive, not a finding to schedule.
