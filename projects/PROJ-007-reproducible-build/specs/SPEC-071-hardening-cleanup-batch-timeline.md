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
- [ ] **build** — implement the four fixes independently, each with its regression test; drive the real
  binary for fix 1 (`lint` on an over-cap image → no truncated/corrupt finding) and fix 4 (`info
  --watch` → exit 2). Then fix 5: sync `api-contract.md` (add the 64 Mpix cap; de-stale the
  `--max-pixels` line) + `cli-reference.md` (document `build --watch` + build-only). Gates: default +
  lean + clippy + fmt + `just deny` (unchanged) + `just validate`; repo-root Cargo/lock/deny diff empty.
- [ ] **verify** — fresh session. Re-drive on the real binary: lint no longer false-diagnoses a
  too-large image (but still flags a genuinely corrupt one); a near-cap cache payload is consistent
  (round-trips or refused at store, never a silent miss); `exit_code_mapping_is_total` covers every
  concrete arm; `--watch` on non-build → exit 2 with the build-only message, `build --watch` intact.
  Confirm the docs now match behavior (64 Mpix cap documented; `--watch` documented + build-only; no
  stale "planned"/"fixed in v1" claims). Gate matrix green, no new dep.
- [ ] **ship** — merge PR; build/verify/ship cost sessions + totals + reflection; archive to done/.
  STAGE-024 backlog: mark the 4 batched items shipped; then re-assess "where we are" — which of the
  remaining larger items to build vs defer before closing PROJ-007.
