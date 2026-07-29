# SPEC-107 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; your job is to implement it.

**One-line summary of the job:** the launch board claims hostile inputs "hold natively" and
nobody has ever driven them. The design cycle drove them. They mostly do hold — **one thing
does not**, and this spec fixes it and locks the rest down with a harness.

This is mostly a **test-and-fixture** spec with **one small engine-adjacent fix** (the truncated
-JPEG warning). Treat the fix as the risky part and the harness as the bulk of the work.

## Read in order — deliberately short

Everything load-bearing is inlined below. **Do not go hunting the wider evidence trail unless
something here contradicts what you find in the code** — if it does, stop and report the
contradiction rather than resolving it yourself.

1. **The spec** —
   `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-107-hostile-and-edge-input-confirmation-pass.md`,
   in full. It is your contract: **11 acceptance criteria, 13 pre-written failing tests, two
   negative controls.** Do not skip `### Coverage matrix` — it is what keeps you from
   re-writing tests that already exist.
2. **The code** — `src/image/mod.rs` (the load path, `:417`), `src/image/avif.rs` (`decode_obus`
   and its test module's `build_avif_with_empty_alpha`), `src/cli/mod.rs:711-770` (the exit-code
   mapping), `src/wasm.rs:530-600`.
3. **The existing tests you are extending, not replacing** — `tests/cli.rs:3745-3785`
   (`info_on_oversized_image_exits_1_not_panic`, the CLI shape to copy),
   `tests/wasm_roundtrip.rs:625-680` and `:810-870` (the wasm hostile tests that already exist),
   `tests/fuzz_regressions.rs` (the library-level corpus sweep this mirrors at the binary level).
4. **`/AGENTS.md`, these sections only** — §4 cost tracking, §6 commands, §12 testing
   conventions, §13 git/PR conventions. Skip the rest.

## The measured facts, inlined

All from `target/release/crustyimg` at `f4c9d22` unless a row says otherwise, 60 s timeout per
case. **No hang, no panic, no OOM on any input; nothing on release exceeded 0.25 s.**

| input | verb | exit | stderr |
|---|---|---|---|
| zero-byte `.png` / text as `.jpg` / text as `.png` / no extension | `info` | **4** | `unsupported or undetectable image format` |
| truncated AVIF (½) | `info` | **1** | `could not decode image: avif: container box size exceeds input` |
| **truncated JPEG (⅓)** | `info` / `web` / `resize` | **0** | **(empty)** ← the defect |
| truncated PNG (⅓) | `info` | **1** | `could not decode image: unexpected end of file` |
| forged 20000×20000 PNG (69 B) | `info` | **1** | `image exceeds decode limits: image 20000x20000 declares 400000000` |
| forged 70000×1 PNG | `info` | **1** | `image exceeds decode limits: Image size exceeds limit` |
| missing path / empty directory | `info` | **3** | `input not found or unreadable: <path>` |
| the three committed `fuzz/avif_decode/*.avif` | `info` | **1** | typed decode errors, clean on release |
| `pixel_bomb.nef` | `info` | **1** | `image exceeds decode limits: raw: embedded preview exceeds …` |
| `no_preview.cr2` | `info` | **1** | `could not decode image: raw: no decodable embedded JPEG preview` |
| `oversize_preview.dng` | `info` | **0** | (empty — correct; see F4/F5 below) |

**The defect (F1).** A truncated JPEG is handed back as a partially-grey image with exit 0 and
**no message at all**, on the flagship `web` path. Truncated PNG and AVIF both error correctly;
JPEG is the outlier because the decoder tolerates truncation by design.

**The fix, already decided by the maintainer — do not re-litigate it.** Warn on stderr, **still
exit 0**, still write the output. Detect it at the **container level** via the missing
end-of-image marker `FF D9`; do **not** touch the codec. Verified discriminating on the probe
pair: `real.jpg` (5694 B) ends `ffd9`, `truncated.jpg` (1898 B) ends `2f69`. Rejected
alternatives, recorded so they are not re-proposed: exit 1 (breaks a workflow every image viewer
supports, and changes a frozen exit-code surface) and document-only (would close a launch gate
over a known silent-corruption path).

**F2 — fix the doc, not the code.** `docs/api-contract.md:59` says exit 4 means "codec not
built … the message names the feature to rebuild with", but a zero-byte or text file also lands
on 4 (`src/image/mod.rs:417` → `src/cli/mod.rs:720`) and names no feature, because there is no
feature to name. The mapping is deliberate and pinned by `exit_code_mapping_is_total`, and the
CLI surface is frozen (STAGE-030). Widen the doc row; change no exit code.

**F3 — record it, do not fix it.** On the **debug** profile only, `meta_parser_state.avif` leaks
upstream `avif-parse` panic text to stderr before `decode_avif`'s `catch_unwind`
(`src/image/avif.rs:127`) converts it:

```
$ target/debug/crustyimg info …/meta_parser_state.avif
thread 'main' panicked at …/avif-parse-2.1.0/src/lib.rs:921:9:
assertion `left == right` failed: bad parser state bytes left
error: could not decode image: avif: decoder panicked on malformed input      RC=1

$ target/release/crustyimg info …/meta_parser_state.avif
error: could not decode image: avif container: unread box content or bad parser sync   RC=1
```

Both exit 1 with a correct final message. **This matters to you** because `cargo test` *is* the
debug profile, so a naive "stderr contains no `panicked`" assertion asserts the wrong thing.
AC-6: carve the upstream text out **by name** under `cfg!(debug_assertions)`, with a comment.
A blanket skip is not acceptable — a *new* panic must still fail the suite.

**F4 — the 60 MP gate is wasm-only.** `MAX_RAW_PREVIEW_MEGAPIXELS = 60` is in `src/wasm.rs:536`
(DEC-082, the demo ceiling). Native RAW is bounded by DEC-063's **64 Mpix**. So the stage's "RAW
either side of the 60 MP gate" is a **wasm** case — and it is **already covered** by
`raw_preview_rejects_over_threshold_before_decode_and_extracts_under_it`
(`tests/wasm_roundtrip.rs:815`), both sides. Do not add a native "60 MP" row; it would be a
number no code path claims.

**F5 — `oversize_preview.dng` exiting 0 natively is correct**, given F4. Recorded so the
native/wasm split does not read as an inconsistency.

## What is already covered — do NOT rebuild it

`just wasm-test` is **green at 26/26 in 30.17 s** (confirmed at design). Already driven on wasm:
the 60 MP gate both sides, the forged-header decompression bomb
(`optimize_detailed_rejects_oversize_without_panic`), module-survival-after-rejection, and the
approved-message hygiene assertions. Already driven at the library level: the fuzz crash corpus
sweep, `bogus_bytes_return_typed_error_not_panic`, `truncated_png_returns_decode_error`, and
both SPEC-094 empty-OBU unit tests.

Your wasm work is **four named gaps** (AC-7), not a harness. Your CLI work is the bulk.

## The one precondition that is real work

**The empty-OBU AVIF has no committed file.** It is synthesised in-process by
`build_avif_with_empty_alpha` inside `src/image/avif.rs`'s test module, so neither the CLI nor
wasm can drive it today. AC-4: commit those exact bytes as a fixture **and re-point the existing
unit test at the file**, so the artifact the CLI and wasm drive is provably the same one the
SPEC-094 guard is pinned against. Two copies of these bytes that can drift apart is the failure
mode to avoid.

## Notes that will save you time

- **Keep the new corpus out of `tests/fixtures/fuzz/`.** That directory has a provenance rule —
  one file per fuzzer finding (DEC-062). Hand-built edge inputs go in `tests/fixtures/hostile/`
  with a `README.md` line each. A corpus whose provenance is not written down decays into "some
  bytes".
- **Generate the forged PNGs in Rust, not by committing opaque blobs.** `wasm_roundtrip.rs`'s
  `png_header_declaring` (`:652`) is the exact shape — real CRC32, so the decoder reads the
  header rather than bailing on a malformed chunk. Reuse that approach.
- **Do not assert on upstream decoder wording.** `Format error decoding Png: IDAT or fDAT …` and
  `Image size exceeds limit` come from the `image` crate and will change under it. Assert on the
  **exit code** and on **our** prefixes (`image exceeds decode limits:`,
  `could not decode image:`).
- **`no-unwrap-on-recoverable-paths` is blocking and this is exactly where it bites.** The EOI
  check reads the last two bytes of untrusted, possibly-empty input. Use slice methods that
  cannot panic on a short buffer.
- **`CARGO_BIN_EXE_crustyimg` is the debug binary under `cargo test`.** That is what makes AC-4's
  `debug_assertions` leg meaningful (a `debug_abort()` is not an unwind, so a thread boundary
  will not save us) — and it is also why AC-6's carve-out is needed. Same property, both
  consequences; do not "fix" one and break the other.
- **Run the negative controls and record them (AC-10).** Revert the truncation warning → at
  least one test must go RED. Delete a fixture file → the enumeration test must go RED. A harness
  nobody has seen fail is not evidence.
- **Report what you could not cover.** If a roster item turns out unreachable or already covered,
  say so in Build Completion with the evidence. A silent drop reads as coverage.
- **`docs/launch-readiness.md` line 34 is stale** — it calls Mobile "STILL OPEN, the remaining
  cross-browser blocker", but SPEC-101 closed that gate (iOS Safari + DuckDuckGo PASS on real
  devices; Android Chrome untested, accepted on maintainer judgment). Correct the stale line
  while you are in the file. This is a correction, **not** a re-grading — the board is
  maintainer-owned.
- **AC-9 must not silently drop the browser half.** Name what remains browser-specific (does the
  demo *surface* these errors legibly; how a phone behaves on the big ones) and leave it on the
  board.

## Verify before handing back

**Clean full matrix from a fresh per-leg `CARGO_TARGET_DIR`.** Run the legs **sequentially**, or
with a **separate** target dir each — never both shared and parallel; a concurrent
differently-featured build corrupted SPEC-108's first lean leg.

```bash
cargo test --no-default-features && cargo test && cargo test --features webp-lossy
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features webp-lossy -- -D warnings
cargo fmt --check
just wasm-test
```

Confirm each log says `Compiling crustyimg` — an incremental build false-greens here and cost
this repo about a day on SPEC-105. Reference totals on `main`: **lean 784 / default 803 /
webp-lossy 810**. Your numbers should exceed these; report them, and reconcile the delta against
the tests you added.

## Repo guardrails

- **Every commit signed off (`git commit -s`).** DCO is enforced and has gone red three times.
- **Never `git reset --hard`.**
- **`rtk` silently corrupts output** — it has dropped the newest commit from `git log`, returned
  "0 matches" against files that plainly match, and mangled `ls`/`cargo`. It is *intermittent*,
  so one clean comparison proves nothing about the next call. Cross-check anything load-bearing
  with `python3` or plain `/usr/bin/git`, plus a positive control that must return nonzero.
- **macOS has no `timeout(1)`.** The design cycle's harness used a Python `subprocess.run(...,
  timeout=)` wrapper. If your hang-detection needs a timeout, do the same — do not assume GNU
  coreutils.
- **`just advance-cycle` / `just archive-spec` mis-target `specs/prompts/*.md`** — `git mv` by hand.
- **Work in a git worktree if any other session is open on this repo**, and check
  `git branch --show-current` before any commit.
- **Do not open or merge the PR.** Maintainer's call.

## When you finish

Fill in `## Build Completion` in the spec and the three reflection questions. Update the
timeline's `build` line. Create the DEC for the F1 decision (`affected_scope` filled in with the
paths it governs).

### Cost

Follow `projects/_templates/prompts/cost-snippet.md` on `main` — it is **current** as of
`f4c9d22` (the corrected DEC-083 version). Two things it stresses that have already gone wrong
on this project: **price at the anchors of the model that actually ran** (read `.message.model`
from your own transcript, not the anchors a prompt names — SPEC-108 overstated by ~67% this
way), and **price per component, not a flat rate** (cache reads dominate a long agentic cycle;
the flat shortcut overstated SPEC-109's build by ~14×).

Close your return message with the `## Cost readout` block, verbatim, as the last thing you
emit.

**Report what you could not do as clearly as what you did.** A stated gap is worth more than a
green tick that quietly skipped something.
