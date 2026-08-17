# SPEC-121 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design calls are settled; implement them.

**One-line summary:** three `Operation::apply` bodies call `to_rgba8()` and never narrow back, so
`resize`, `thumbnail`, `edit` and **the flagship `web`** add an all-opaque alpha channel (+12.4%
bytes, measured) and silently halve 16-bit input. Fix the three bodies; preserve what you were
given.

## Where you sit

You are the **second of three serial specs**. SPEC-123 (no `src/` change) runs before you.
**SPEC-122 follows you into the same function**, `Resize::apply` — it makes that resample happen in
linear light. You go first because a colour-type fix under a linear-light rewrite is harder than
the reverse. Leave `Resize::apply` in a state someone can build on.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-121-ops-preserve-colour-type-and-bit-depth.md`,
   in full. **10 ACs, 5 settled design calls, 8 failing tests already written.**
2. **`docs/backlog.md`** — `## ⚠ Live defect — ops widen to RGBA and never narrow back`.
   **Read it; do not re-derive it.** The measurements are done: the per-verb IHDR table, the
   377,132 B vs 423,756 B comparison, the 16-bit truncation. Re-measuring costs money and changes
   nothing.
3. **The code** — the three sites, confirmed present today:
   - `src/operation/mod.rs:197` — `Invert::apply`, `let mut buf = img.pixels().to_rgba8();`
   - `src/operation/mod.rs:395` — `Resize::apply`, `let rgba = img.pixels().to_rgba8();`
   - `src/operation/mod.rs:816` — `Watermark::apply`, `let mut canvas = img.pixels().to_rgba8();`

   Also `src/sink/mod.rs` — the default `write_to` path that **already preserves** the
   `DynamicImage` variant. That is why only these three bodies collapse it.
4. **DEC-058** (cache-key composition) and **DEC-090** (the diagnostic channel Call 3 uses).
5. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR, **§15**.

## The design calls are SETTLED — do not reopen them

1. **Widen to work, narrow to write.** Widening internally is fine; *returning* a widened image is
   the defect. The narrowing is **lossless-only**: RGBA→RGB only when every alpha sample is
   opaque, and **16→8 never** — that is a downgrade, not a narrowing.
2. **`Watermark` decides in code, not by exemption.** It may return RGBA from RGB input *when the
   overlay actually contributed non-opaque samples*. Fully opaque composite → it narrows like the
   others. AC-4 tests **both directions**.
3. **A lossy target's 8-bit downgrade is reported.** JPEG and lossy WebP are 8-bit; feeding them
   16-bit pixels is a real loss and the user is told. **One line at the sink**, in the spirit of
   SPEC-090's honest size reporting. Not a new policy, not a new flag.
5. **No new flag.** "Preserve what you were given when the target can hold it" is not something a
   user should have to ask for.

⚠ **`DynamicImage` already has `ImageRgb16` / `ImageRgba16`. No type change is needed anywhere.**
If you are adding a variant or a field, you have left the spec.

## ⚡ Call 4 — the migration ALREADY EXISTS. Do not build one.

Both backlog entries flag *"this invalidates every PROJ-007 build lockfile"* as what makes this
non-trivial. **Design read the code and settled it: it does not.**

- **`cache_key_for` already includes `crate::version()`** — `src/cli/build.rs:294`, via
  `cache::compute_key(crate::version(), …)`. A release changes every key, so old and new renders
  **cannot collide in the cache.** No new key component. No colour-pipeline-version field.
- **The lockfile never promised output-hash stability across versions.** `src/build/lock.rs:32-36`:
  `hash` is *"recorded as observed under `[env]`, never promised"*, explicitly not stable *"across
  arch/OS/codec versions"*. `key` is a function of inputs including the version, so a bump is
  *"an unambiguous, cross-machine drift signal"* — the designed behaviour.

So a user upgrading sees keys change, `--frozen` fails, and they regenerate. **That is the normal
upgrade path, already specified.**

**Your job is AC-8: drive it and confirm it.** Build a target with a committed lockfile on `main`'s
binary, upgrade to your branch binary, and show all four: the key changes, `--frozen` fails,
regeneration succeeds, no stale cache entry is served. **If the contract does not hold, STOP and
report it as a finding.** Do not design around it, and do not invent machinery to paper over it.

## The two controls that stop this from being a test that cannot fail

**AC-3 — the lossless-only control.** "Always narrow" passes AC-1 and AC-2 and *destroys real
transparency*. An RGBA input with a genuinely translucent pixel must stay RGBA. This is the test
that separates the fix from a plausible-looking regression.

**AC-9 — one revert per op body, three reverts.** Revert `Invert`, `Resize` and `Watermark`
**independently**; each turns **only its own** tests red and leaves the other two green. That is
what proves the three tests are independent rather than co-dependent. AGENTS §15: **the evidence
is the behavioural flip, not a binary hash** — measured on this repo 2026-08-16, a debug rebuild
from byte-identical source produced a different binary, so a changed hash proves only that cargo
relinked. Cite the test going RED.

## The sweep — mechanical, and its scope is a claim

⚠ **"crustyimg is 8-bit internally" is written down somewhere and it is not accurate as stated.**
Decode preserves the variant, `Identity` and `AutoOrient` preserve it, and the default encode path
preserves it — **only these three op bodies collapse it.** Correcting that claim is **part of this
spec**, not a follow-up; it outlives the stage.

Do it mechanically and **cite the grep you ran, including what it covered** — `docs/`, `README.md`,
`decisions/`, doc comments in `src/`, the wasm demo copy. A sweep whose scope you did not state is
a sweep nobody can check.

## The matrix — AC-10, and it is easy to get wrong

Clean full matrix, **fresh per-leg `CARGO_TARGET_DIR`**, **sequential**, through `rtk proxy`:

- default features
- `--no-default-features`
- `--features webp-lossy`

Clippy (`--all-targets -- -D warnings`) and `cargo fmt --check` on each leg. **Own `main`
baseline** — AC-7 needs `convert`, `optimize` and `auto-orient` byte-identical to `main`, which
means you build `main` yourself rather than trusting a remembered number.

⚠ **Never both shared-and-parallel.** Concurrent differently-featured builds sharing one target
dir corrupt it. Isolate `CARGO_TARGET_DIR` per leg **or** run legs sequentially — this repo has
measured the corruption. Then **read the CI legs individually**; a green summary is not a matrix.

## Guardrails

- **Own git worktree**, branch `fix/spec-121-ops-preserve-colour-type-and-bit-depth`. Do not work
  in the primary checkout.
- **Your DEC is DEC-095, and it is SHARED with SPEC-122.** The ID is reserved — **do not run
  `next_id`**; it scans only the working tree, so a record on an unmerged branch is invisible
  (SPEC-119 and SPEC-120 both minted DEC-092 that way). Write DEC-095 to cover **the wave**: the
  byte change across every recipe and its migration posture, scoped so SPEC-122 can amend it for
  linear light rather than mint a second decision. `affected_scope`: `src/operation/**`,
  `src/sink/**`.
- **⚡ Checkpoint early.** Push a WIP commit **as soon as the branch compiles**, before you start
  the matrix. SPEC-113's build ran three hours and $40 with **zero commits** and had nothing to
  show when it stalled.
- **Budget in exchanges, not minutes.** This is an **M** — past **~250 exchanges** without having
  started the matrix, checkpoint and report. Cost scales with the *square* of message count and
  anti-correlates with wall clock: SPEC-116 ran 104 minutes for $11.91; SPEC-119 ran 61 for
  $51.24.
- **A piped command reports the pipe's exit code** — `cargo test | tail` turns a red leg green.
  Redirect and read `$?`.
- macOS has no `timeout(1)`. `git commit -s` (DCO). Never `git reset --hard`.
- **Multi-line markdown bullets: rewrite the file, do not `sed`.** Line-oriented `sed` has left
  orphaned continuation lines twice in this project.
- **Do not merge the PR. Do not bump the version.**

## When you finish, in this order

1. Fill in the spec's `## Build Completion`, including its three reflection questions.
2. Append a build cost session entry to `cost.sessions` (see below).
3. Write **DEC-095**, with `affected_scope` set to the globs above — that is what lets
   `decisions-audit --changed` surface it later.
4. Run `just advance-cycle SPEC-121 verify`, and **CONFIRM it moved** — `git diff` on the spec
   should show the `cycle:` line change. It reports success even when it changes nothing.
5. Open the PR. **Do not merge it.**

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by something only
your session emitted — **never by "the newest `.jsonl`."** Price per component at the anchors of
the model `.message.model` actually reports (you are expected to be **Sonnet**: $3/$15 per MTok;
cache_creation ×1.25 input, cache_read ×0.10 input). **Do not price at Opus anchors because a
prompt named them** — SPEC-108 did exactly that and overstated its total by ~67%.

**Measure at session end, not mid-session.** Mid-session readings have run 40–49% low, measured
twice. Re-measure as the last thing you do.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
