---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-024
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium
  target_complete: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-10
shipped_at: null

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
- [~] SPEC-070 (design) — **bound peak decode memory (Medium, user-prioritized).** Framed + build-ready 2026-07-10 → DEC-063 at build. `image::Limits.max_alloc` bounds a single allocation, not the cumulative/peak working set → a crafted near-max-dimension image (JPEG SOF / RAW embedded preview) peaks ~1.9 GB while passing every DEC-034 cap (F-RAW-1; product-facing — `info` on a 782 B `.nef` → ~1.93 GB). Fix = a total-pixel / peak-bytes cap enforced via a **pre-decode header dimension peek** at the two seams that lack one (generic `ImageReader` + RAW `decode_jpeg_with_limits`) + align AVIF/SVG `check_caps`; DEC-063 for the budget/amplification-factor tradeoff (rec. ~1 GiB ⇒ ~64 Mpix — reject the 160 MP bomb, keep ~24–50 MP photos). **Closes F-RAW-1 + general decode peak; NOT F-AVIF-3 (upstream parse-stage, needs vendoring — stays filed).** F-RAW-1's reproducer graduates into the corpus smoke. No new dep.
- [ ] (not yet framed) — non-UTF-8 / unusual-filename hardening (typed error, no silent empty stem) *(SPEC-068: resized to UX/correctness, Low)*
- [ ] (not yet framed) — cache-key / determinism-envelope completeness (build profile; the envelope's true bound)
- [ ] (not yet framed) — `CACHE_ENTRY_MAX_BYTES` read-bound off-by-53 fix + regression test *(SPEC-068 confirmed the asymmetry; correctness not safety; boundary test already pinned)*
- [ ] (not yet framed) — pre-decode format sniff (closes SPEC-065 `{ext}` false positives + SPEC-066 residual)
- [ ] (not yet framed) — exit-code mapping totality audit + `is_total` value-assertion completeness *(SPEC-068: the `code()` match is already compiler-exhaustive — the gap is missing value assertions, not missing arms)*
- [ ] (not yet framed, from SPEC-067 verify) — reject or document `--watch` as build-only (it's a global clap flag → silent no-op on non-build subcommands)
- [ ] (not yet framed, from SPEC-067 verify) — orphaned-output prune on source removal under build/watch (a future `--clean`)
- [ ] (not yet framed, from SPEC-068 re-verify) — canonicalize-contain the `out` dir: require the *canonicalized* out dir to stay within the canonicalized build root, closing the symlinked-out-dir write-escape. Accepted residual for now (needs a committed in-tree symlink + manifest control); closing it rejects intentionally symlinked output dirs (`dist → ramdisk`), so it needs its own spec + that tradeoff call.

**Count:** 2 shipped / 1 active / 8 pending — SPEC-068 + SPEC-069 shipped; **SPEC-070 (bound peak decode memory, Medium, user-prioritized) framed + build-ready.** 4 items carried from SPEC-067 (×2) + SPEC-068 (×1) + SPEC-069 (×1) verify.

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

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
