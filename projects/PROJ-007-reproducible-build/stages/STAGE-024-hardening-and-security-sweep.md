---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-024
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium
  target_complete: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-10
shipped_at: 2026-07-12

value_contribution:
  advances: >
    Hardens and security-reviews what the wave shipped rather than adding features: a systematic
    threat-model pass over PROJ-007's NEW untrusted-input surface (build manifest, recipe files,
    the `.crustyimg/` cache store, the committed lockfile, and `--watch`), plus the specific
    correctness defects a prior-session review flagged. It protects the "verifiable" thesis — a
    build tool people trust in CI can't panic on a hostile committed file, serve a stale cache hit,
    or corrupt on an unusual filename. Sequenced LAST in PROJ-007, after STAGE-023 (`--watch`).
  delivers:
    - "A threat-model / attack-surface review of PROJ-007's new untrusted-input paths (manifest, recipe, cache store, lockfile, watch) against the shipped `untrusted-input-hardening` posture — the SPEC-037 precedent, for this wave's surface"
    - "The decoder fuzz gate actually run (AVIF/SVG/RAW/HEIC), with whatever it surfaces fixed — the roadmap's pre-1.0 gate, never yet executed"
    - "Graceful, typed handling of non-UTF-8 / unusual filenames instead of silent empty-stem collisions"
    - "Cache-key / determinism-envelope completeness (build profile is an unkeyed output-affecting input today)"
    - "The two filed defects closed: the `CACHE_ENTRY_MAX_BYTES` read-bound off-by-53, and the pre-decode format sniff that closes SPEC-065's `{ext}` false positives + SPEC-066's literal-`{ext}` residual"
    - "An exit-code-mapping totality audit (the `is_total` test shipped wrong twice)"
  explicitly_does_not:
    - "Add new build/cache/lockfile/watch features (those are STAGE-020..023)"
    - "Re-audit PROJ-001's surface (SPEC-037 already did) or run a full repo-wide external audit / ultrareview — the review here is scoped to PROJ-007's NEW surface"
    - "Re-open shipped decisions (DEC-057/058/059/060) unless the review or a defect forces it"
---

# STAGE-024: hardening & security sweep

## What This Stage Is

The wave's closing correctness + security pass. STAGE-020..023 built the declared, cached,
verifiable, watchable build; this stage hardens it and reviews its **new attack surface**. Two
threads. First — the security thread — a systematic threat-model pass over the untrusted-input
paths PROJ-007 added (the build manifest and recipe files as config, the `.crustyimg/` cache
store it reads, the committed and hand-editable lockfile, and the tree `--watch` walks),
checked against the shipped `untrusted-input-hardening` posture — the SPEC-037 threat-model
precedent, applied to this wave's surface. Second — the hardening thread — the specific
latent-bug suspects a self-review surfaced (2026-07-10, grounded in a code sweep): the never-run
decoder fuzz gate, silent unusual-filename handling, cache-key/determinism gaps, and two filed
defects. It is deliberately the **last** stage of PROJ-007: it depends on nothing downstream,
and it's the difference between "the machinery works on the happy path" and "you can trust it in
CI on inputs — and committed files — you didn't write." (The lockfile-panic that blocked
SPEC-066 ship is the proof: that surface has issues we stumble on rather than enumerate.)

## Why Now

- **The recurring lesson of this wave was that green exit-code tests miss whole classes of
  defect** — user-facing strings, hostile serialized/committed input, cross-platform path
  handling. Three shipped-green defects proved it (SPEC-065 `{output:?}`, SPEC-066 `--strict`
  message + the non-hex-digest panic). A dedicated sweep is how you find the rest before 1.0.
- **The fuzz gate is a pre-1.0 roadmap item that was never executed** — the four targets exist
  but there's no nightly/cargo-fuzz in the build/verify envs, so untrusted-binary decode paths
  (AVIF/SVG/RAW/HEIC) have never been fuzzed. That's the highest-severity open surface.
- **It's cheap relative to its value** — most items are small, and finding a memory-safety bug
  or a stale-hit bug now is far cheaper than a post-1.0 CVE or a "the cache lied" report.

## Success Criteria

- A written threat-model note exists for PROJ-007's new surface (manifest / recipe / cache /
  lockfile / watch), each path checked against `untrusted-input-hardening`; every finding is
  fixed or explicitly accepted with a rationale (the SPEC-037 shape, for this wave).
- The four fuzz targets run (locally, `cargo +nightly fuzz run …`) for a documented budget with
  no crash left unaddressed; the run is recorded so it's repeatable.
- A non-UTF-8 (or otherwise `.to_str()`-unrepresentable) input filename produces a **clear typed
  error**, not a silent empty stem/extension that collides or writes a `""`-stem output.
- The cache cannot serve a stale hit across an output-affecting input it doesn't key today
  (build profile at minimum) — either the input joins the key or the schema version gates it.
- The `CACHE_ENTRY_MAX_BYTES` store-vs-read bound is consistent (a near-cap payload round-trips),
  and the pre-decode format sniff closes both `{ext}` collision gaps.
- Every `CliError` variant provably maps to its documented exit code; the totality test can't
  silently omit one again.
- No regressions: full gate matrix green, `just deny` unchanged, no new default dependency
  (the fuzz tooling is dev/nightly-only).

## Scope

### In scope (candidate specs — frame when the stage is picked up)
- **(security — LEAD) Threat-model / attack-surface review of PROJ-007's new untrusted-input
  surface.** Systematically walk each new surface the wave added — the **build manifest** and
  **recipe** files (parsed config), the **`.crustyimg/` cache store** (reads committed, hand-editable
  entries; verify-on-read exists but the whole path wants an adversarial look), the **committed
  lockfile** (the non-hex-digest panic came from here), and the tree **`--watch`** walks (symlink
  escapes, watching outside declared roots, resource use) — against `guidance/constraints.yaml`'s
  `untrusted-input-hardening` + DEC-034/DEC-035. Mirror SPEC-037 (PROJ-001's threat-model pass) for
  this wave. Output: a short threat-model note + a punch list. **This runs FIRST — its findings feed,
  add to, or reprioritize the items below; the fuzz gate is one instrument of it.**
- **(fuzz) Run the decoder fuzz gate + fix findings.** Execute `fuzz/avif_decode`,
  `fuzz/svg_decode`, and the RAW/HEIC targets; triage + fix any crash; document the run and a
  repeat recipe. *Lineage: the untrusted-decode surface is PROJ-009's, but the targets live
  in-repo and the gate is a shared pre-1.0 item — running it here is fine.*
- **(paths) Unusual-filename hardening.** Audit every `stem`/`ext`/path `.to_str()` seam
  (`src/source`, `src/sink`, `src/cli`, `src/build`); a non-UTF-8 or empty-stem input becomes a
  typed error or a documented, collision-safe fallback — never a silent `""` that the injectivity
  check has to catch as a confusing "output collision."
- **(cache) Cache-key / determinism-envelope completeness.** Build profile (debug vs release) is
  an output-affecting input absent from the key (DEC-058's seven) — add it or bump
  `CACHE_SCHEMA_VERSION`'s contract; re-examine the "byte-identical within a machine" envelope
  (the rav1e thread-lever caveat) and document its true boundary.
- **(cache) The `CACHE_ENTRY_MAX_BYTES` off-by-53 read bound** — `store` bounds the payload,
  `read_entry` bounds payload+53-byte header, so a near-cap payload is stored-but-unreadable
  (permanent silent miss). One-line fix + a regression test.
- **(collision) The pre-decode format sniff** — closes SPEC-065's conservative-`{ext}` false
  positives AND SPEC-066's literal-`{ext}` residual (the mixed `{stem}.png` / `{stem}.{ext}`
  collision the lockfile catches only after the race writes both files). DEC-059 threat-model item.
- **(cli) Exit-code totality audit** — verify every `CliError` variant maps to the exit code in
  `docs/api-contract.md`; strengthen `exit_code_mapping_is_total` so an omission can't ship green
  again (it missed `Cache` and `Metadata` in prior specs).

### Explicitly out of scope
- New features; a full repo-wide security audit beyond these surfaces (a separate ultrareview);
  reworking shipped decisions unless a defect forces it; message-text test infrastructure as a
  framework (add grep-stderr assertions per spec instead).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-068 (shipped 2026-07-10) — **LEAD: threat-model / attack-surface review of PROJ-007's new untrusted-input surface (manifest, recipe, cache store, lockfile, watch); SPEC-037 for this wave.** PR #75 → main b8283bb, DEC-061 emitted. Threat-model note (`docs/research/proj-007-threat-model.md`) + a **reprioritized backlog** (the items below) + inline tightenings: recipe **top-level** `deny_unknown_fields` (closed a zero-step silent-passthrough footgun) + an **out-directory write-escape clamp** (verify found a hostile `out = "../.."` wrote outside the tree via `build --check`; clamped at `Target::validate`, exit 2, prepare-phase). Symlinked-out-dir residual accepted + documented (→ #10). Suspects resolved: recipe unknown-key CONFIRMED+fixed; cache off-by-53 CONFIRMED (correctness, filed); watch roots + `.to_str()→""` ACCEPTED; exit-code map RESIZED (already compiler-exhaustive). 6 sessions (design→build→verify→punch-fix→re-verify→ship), 690k tok. No new dep.
- [x] SPEC-069 (shipped 2026-07-10) — ran the decoder fuzz gate (AVIF/SVG/RAW/HEIC). **#1 High** — the untrusted-binary surface SPEC-068 couldn't close by reading; the roadmap's pre-1.0 gate. PR #76 → main 7bd18fc, DEC-062 emitted, roadmap ticked. **SVG + RAW clean of crashes; all AVIF findings upstream `avif-parse` 2.1.0** (not re_rav1d/our glue): 2 fixed at the boundary (`box_sizes_fit`+`catch_unwind`+`frame_size_limit`) w/ mutation-checked regressions, 1 documented residual (F-AVIF-3). Durability = minimized fixtures + `tests/fuzz_regressions.rs` + always-on `fuzz_corpus_never_panics` smoke (3-OS per-PR) + `just fuzz` + run record. Verify found + corrected an overclaim (raw "CLEAN" → F-RAW-1 memory residual). No new default dep. 4 sessions, 570k tok.
- [x] SPEC-070 (shipped 2026-07-11) — **bound peak decode memory (Medium, user-prioritized).** PR #78 → main 5ecc717, DEC-063 emitted. Added `MAX_IMAGE_PIXELS` (64 Mpix) + a saturating `check_pixel_budget(w,h)` enforced BEFORE decode via a header/SOF peek at the two unchecked seams (generic `ImageReader` + RAW) + aligned AVIF/SVG/HEIC `check_caps` + dav1d `frame_size_limit`. **Closes F-RAW-1** (782 B `.nef` bomb 1.93 GB → ~8 MB). Fresh verify CLEAN: bounded on every route + command, progressive-JPEG held at 1.6×, no false rejection, F-RAW-1 in the smoke. **F-AVIF-3 stays open** (upstream parse-stage). DEC-063 sets the 64 Mpix tradeoff (rejects >64 MP medium-format/panoramas) + the `--max-pixels` future dial. 4 sessions, 380k tok. No new dep.
- [x] SPEC-071 (shipped 2026-07-11) — **STAGE-024 hardening cleanup (batch).** PR #79 → main 06c9927, no new DEC. Four point-fixes + a docs sync in one cycle: (1) **lint false-diagnosis** — the specced `LimitsExceeded` arm PLUS the strictly-worse bug it exposed on the real binary: lint decoded by BYTES while holding a path, so every valid `.nef`/`.cr2` linted "truncated or corrupt" at exit 7; fixed by routing through `Image::decode_path` (SPEC-061/DEC-055) so lint genuinely inspects RAW/AVIF/SVG/HEIC; (2) **cache off-by-53** — `store_bounded` bounds the frame now (unified `ENTRY_HEADER_BYTES`), boundary-sweep test; (3) **exit-code `is_total`** value-asserts (Metadata::Container/Exif); (4) **`--watch` build-only** (exit 2 on non-build); (5) **docs** — api-contract 64 Mpix cap + honest `--max-pixels`; cli-reference `build --watch`. Verify CLEAN across every format, mutation-checked, dep diff empty. 4 sessions, 410k tok. **The "drive the real binary" discipline caught a green-unit-test / wrong-shipped-behavior bug — 5th [IMAGE_EXTENSIONS-exposes-every-decode-caller] instance.**
- [ ] (not yet framed, from SPEC-071 build/verify) — **lint decode-seam audit + honest not-inspected rule (2 items, Low):** (a) grep every `Image::from_bytes` caller holding a path (other commands may share the lint-on-RAW latent bug) + a per-format lint smoke test, or it recurs a 6th time; (b) a `meta/not-inspected`-style lint rule (non-Error) so files lint couldn't inspect (LimitsExceeded; a mislabeled PNG-as-`.nef`) get an honest signal, not silence-or-false-corrupt. Plus a `-v` note when an output is refused by the cache size cap (fix 2 refuses silently).
- [ ] (not yet framed, from SPEC-070 verify) — **full-pipeline peak envelope:** DEC-063's 1 GiB budget governs DECODE only; encode/rule buffers stack on top (a legit 49 Mpix `convert` hit 934 MB, at-cap GIF `lint` 776 MB). Document/bound the true decode+pipeline peak. Med.
- [ ] (not yet framed, from SPEC-070 cap decision) — **`--max-pixels` opt-in / raise the 64 Mpix cap:** DEC-063 keeps 64 Mpix now; the maintainer flagged likely future medium-format interest (revisit trigger is live). A documented opt-in flag/config, not a silently higher default — its own spec.
- [ ] (not yet framed) — **pre-decode format sniff** (closes SPEC-065 `{ext}` false positives + SPEC-066 literal-`{ext}` residual). Med; its own spec.
- [ ] (not yet framed, from SPEC-068 re-verify) — **canonicalize-contain the `out` dir:** require the *canonicalized* out dir to stay within the canonicalized build root, closing the symlinked-out-dir write-escape (security residual). Tradeoff: rejects intentionally symlinked output dirs (`dist → ramdisk`) → its own spec.
- [ ] (not yet framed) — non-UTF-8 / unusual-filename `.to_str()` sweep (typed error, no silent empty stem) *(SPEC-068: resized to UX/correctness, Low; a sweep, not a point-fix)*
- [ ] (not yet framed) — cache-key / determinism-envelope completeness (build profile → cross-profile stale hit; touches the DEC-058 key/schema)
- [ ] (not yet framed, from SPEC-067 verify) — orphaned-output prune on source removal under build/watch (a future `--clean`), Low

**Count:** 4 shipped / 0 active / 8 **DEFERRED** (post-1.0 maintenance) — SPEC-068 + SPEC-069 + SPEC-070 + SPEC-071 shipped. The small-fix tail is cleared; the remaining 8 are the **larger/decision-bearing** items (lint decode-seam audit + not-inspected rule, full-pipeline envelope, `--max-pixels`, format-sniff, canonicalize-out, unusual-filename sweep, cache-key profile, orphan prune).

**TRIAGE + CLOSE DECISION (2026-07-12).** Ran the "where are we" triage against the bar *"does a 1.0 claiming 'trust it in CI on files you didn't write' actually need this?"* Result: **stage + PROJ-007 close now; all 8 deferred to a post-1.0 maintenance pass.** Rationale per item:
- **canonicalize-contain the `out` dir** (the one genuine out-of-tree WRITE escape) — SPEC-068 already made a deliberate **accept+document** call on it: the exploit bar is high (needs a committed in-tree symlink + attacker-controlled manifest, both reviewable under "reviewed like code"), and the fix rejects legitimate symlinked out dirs (`dist → ramdisk`), so it warrants its own spec with an opt-out hatch — not a rushed close-gate. **Accepted as a documented residual.**
- **pre-decode format sniff** — the SPEC-066 literal-`{ext}` residual **fails closed** (writes land inside the tree; the post-encode re-check / lockfile catches the collision at exit 2, no lock written). Correctness/UX polish, not a trust break.
- **full-pipeline peak envelope** — decode is capped at 64 Mpix (DEC-063); encode/rule buffers stacking to ~934 MB on a *legit* large image is not attacker-amplifiable past the decode cap. Document, don't gate.
- lint decode-seam audit + not-inspected rule (Low), `--max-pixels` (a capability *dial*, revisit-trigger live), unusual-filename `.to_str()` sweep (SPEC-068 accepted `→""` as safe-but-silent; injectivity catches collisions), cache-key build-profile (local stale-hit only across debug+release of the same crustyimg version — tiny; trivial fold-in whenever), orphan-output prune (a future `--clean` feature) — all polish/feature, none touch the trust surface.

These 8 remain listed above as the **post-1.0 maintenance backlog** for PROJ-007; they are pulled individually if adoption/usage surfaces the need. See `docs/roadmap.md` (post-1.0 section references this backlog).

## Design Notes

- **These are grounded, not speculative.** A 2026-07-10 self-review + code sweep produced this
  list: the byte-slicing discipline is mostly good (find/hex-based), the `.to_str()` seams
  correctly use `and_then(...).unwrap_or("")` (safe but *silent* — the item above), and the
  cache key omits build profile. Fuzzing is the one high-severity unknown. Don't gold-plate:
  fix what's real, record what isn't.
- **Framing bias:** the same architect framed all of PROJ-007, so this stage benefits most from a
  fresh adversarial eye — prefer driving the binary with hostile input and a fuzzer over reading
  the code the author already trusts.
- **No new default dependency:** fuzz tooling is nightly/dev-only; the fixes are pure-Rust.

## Dependencies

### Depends on
- All of PROJ-007's shipped machinery (STAGE-020..022) + STAGE-023 (`--watch`); the four fuzz
  targets already in `fuzz/`; DEC-034 (decode caps, the mitigation these harden past).

### Enables
- A PROJ-007 that's trustworthy on untrusted input — a precondition for the pre-1.0 "reviewed
  like code" claim to be safe, not just aspirational.

## Stage-Level Reflection

*Shipped 2026-07-12.*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes.** The security thread landed
  fully: a written threat-model note (`docs/research/proj-007-threat-model.md`) covering all five
  new untrusted-input surfaces (manifest / recipe / cache store / lockfile / watch), each checked
  against `untrusted-input-hardening`, with every finding fixed or explicitly accepted (DEC-061).
  The never-run decoder fuzz gate was **actually run** (SPEC-069, DEC-062) — the roadmap's stated
  pre-1.0 gate — with crashes triaged, boundary-fixed, and converted to always-on per-PR
  regressions (`tests/fuzz_regressions.rs` + `fuzz_corpus_never_panics`). The memory-amplification
  class it surfaced (F-RAW-1) was closed by a pre-decode pixel budget (SPEC-070, DEC-063), and the
  small hardening tail (lint false-diagnosis, cache off-by-53, exit-code totality, `--watch`
  build-only, docs sync) shipped as one proportionate batch (SPEC-071). What we deliberately did
  **not** gate on: the larger decision-bearing tail (8 items), triaged 2026-07-12 and deferred to a
  post-1.0 maintenance pass — the one security-flavored residual (canonicalize-contain-out) was
  already an accept+document decision with a real usability tradeoff, and the rest are
  polish/feature/fails-closed.
- **How many specs did it actually take?** **4** (SPEC-068 threat model, SPEC-069 fuzz gate,
  SPEC-070 peak-memory cap, SPEC-071 cleanup batch) — matching the "lead with the threat model,
  let its findings reprioritize the rest" plan. The threat model (SPEC-068) did its job: it spawned
  the reprioritized backlog that became the queue, promoted the fuzz gate to #1, and confirmed /
  resized / dismissed the self-review suspects instead of us building all of them.
- **What changed between starting and shipping?** The stage started as a flat list of ~7 candidate
  fixes; the SPEC-068 threat-model pass turned it into a *ranked* queue where three items proved
  high-value (fuzz, peak-memory, batch) and eight proved deferrable — the review's real output was
  the triage, not just the tightenings.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **A "clean / contained / safe" verdict must be EARNED under the exact command + config it
    claims.** Four overclaims this wave (SPEC-068 out-escape, SPEC-068 symlink-out-dir, SPEC-069 raw
    "clean" that OOMed under `-O`) were all "the note/verdict claims a safety the binary lacks."
    Threat-model / verify specs should cite a *driven* attack (or a fuzz run under the shipped
    config) for every safety claim — an unearned "safe" is a defect, not prose.
  - **Batch the small tail; don't run a 4-session spec lifecycle per 3-line fix.** SPEC-071 (4
    point-fixes + docs in one cycle) is the proportionate cadence for a cleared small-fix tail.
  - **`IMAGE_EXTENSIONS` exposes every decode caller** landed a 5th time (SPEC-071: lint decoded RAW
    by bytes → every valid `.nef` linted "corrupt"). A new extension / `ImageError` variant needs an
    audit of *every* decode caller and `Err(_)` catch-all, not just the exit-code map.
  - **Audit user-facing docs (api-contract / cli-reference / README) for drift before closing a
    wave** — shipped behavior (`--watch`, the 64 Mpix cap) left doc-debt; SPEC-071 folded a docs sync
    in as pre-ship discipline.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - Yes — the "drive the real binary with adversarial / hostile-serialized input; green exit-code
    tests miss string / cross-platform / hostile-file / config-dependent defects" lesson recurred
    across every spec of this stage and the prior two, and is the wave's load-bearing process lesson
    (promote to the project reflection + AGENTS verify guidance).
  - The **fuzz-under-the-shipped-config** point (SPEC-069 ran RAW in debug, hiding F-RAW-1) is worth
    a line in the fuzz/`just fuzz` docs: fuzz `-O`/release to match what ships.
