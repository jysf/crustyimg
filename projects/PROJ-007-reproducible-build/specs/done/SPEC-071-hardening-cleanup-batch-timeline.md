# SPEC-071 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — a deliberate BATCH of the STAGE-024 small-fix tail (the high-value items shipped as
  their own adversarial specs; the tail is disproportionate to that cadence). Four independent,
  localized point-fixes in ONE cycle, each grounded firsthand + each with a test: (1) **lint
  `LimitsExceeded` false-diagnosis** — `TruncatedOrCorrupt` (`src/lint/mod.rs:576`) calls a
  valid-but-too-large image "truncated or corrupt / re-export"; add a `LimitsExceeded => None` arm
  mirroring the `CodecNotBuilt` special-case; (2) **cache off-by-53** — `store_bounded` bounds the
  payload while `read_entry` bounds the frame (payload+53+ext), so a near-cap payload is
  stored-but-unreadable; bound the frame consistently + rework the `:855` documenting test; (3)
  **exit-code `is_total` value-assertions** — `code()` is already compiler-exhaustive, so add the
  missing `Metadata::Container`/`Exif` (+ confirm `Watch::Watch`) value assertions (test-completeness);
  (4) **`--watch` build-only** — a global clap flag only `Build` honors; reject on non-build
  subcommands with a usage error (exit 2). **(5) docs sync** (added pre-ship, per the "consider docs
  before we ship" check) — `docs/api-contract.md` decode-limits section is missing SPEC-070's 64 Mpix
  cap (DEC-063) + still calls `--max-pixels` "planned"; `docs/cli-reference.md` has no `--watch` doc at
  all (SPEC-067 debt) → document it + its build-only restriction (fix 4). No new dep, no new DEC (item 4
  decided: reject). Larger tail items (format-sniff, canonicalize-out, unusual-filename sweep, cache-key
  profile, `--max-pixels`) explicitly left as their own specs. Framing, 2026-07-11.
- [x] **build** — all 5 landed in one cycle. **Fix 1 was NOT the specced bug:** the `LimitsExceeded`
  arm went green in unit tests but driving the real binary showed `lint <pixel_bomb.nef>` still said
  "truncated or corrupt" — because `lint` decoded by BYTES while holding a path, so RAW (extension-
  routed, SPEC-061/DEC-055) never reached its decoder and **every valid `.nef`/`.cr2` linted as corrupt
  at exit 7**. Root-caused + fixed via `Image::decode_path` (one line, existing documented contract) —
  a scope expansion forced by fix 1's own acceptance criterion, no new DEC. Fixes 2–4 mechanical; fix 5
  docs synced. Six commits (one per fix + bookkeeping). → **PR #79**, 26/26 3-OS CI green, dep diff
  empty. Est. ~190k tok. 2026-07-11.
- [x] **verify** — fresh adversarial session. **CLEAN, ships.** Bounded fix 1's blast radius
  structurally (decode_path only diverts RAW extensions; everything else byte-identical) + a
  before/after table + DROVE a decode-dependent lint rule to FIRE (proving lint now genuinely inspects
  RAW/AVIF/SVG/HEIC, not just stops complaining). Mutation-checked fix 2's boundary sweep (revert →
  fails). Confirmed `--watch` exit-2 + `build --watch` intact + every doc claim empirically. One
  non-blocking observation (PNG mislabeled `.nef` now flags corrupt — consistent with `info`; lands
  under the filed meta/not-inspected follow-up). Gates green, dep diff empty. Est. ~150k tok. 2026-07-11.
- [x] **ship** — squash-merged **PR #79** → main (**06c9927**); DEC-hygiene from the builder's
  spec-quality note (DEC-055 `affected_scope` += `src/lint/mod.rs`; DEC-055/063 added to references);
  filled verify/ship cost sessions + `cost.totals` (410k tok / ~$5.08, 4 sessions, labelled estimates
  §4) + ship reflection; timeline; **STAGE-024 marks SPEC-071's 4 items shipped** + files the
  build/verify follow-ups (lint decode-seam audit + per-format smoke; meta/not-inspected honest rule
  incl. extension-mismatch; `-v` cache-refused note); archived to `done/`; cost-audit + validate green;
  brag + memory. **SPEC-071 SHIPPED.** Next: the "where are we" triage on the remaining tail before
  closing PROJ-007. 2026-07-11.
