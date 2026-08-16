# SPEC-120 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; run the measurement.

**One-line summary:** `resize` resamples non-linear sRGB values as if they were linear. Before
anyone fixes that, find out whether it actually measures worse — and whether the instrument we
would judge it with can see the difference at all.

**This spec ships no behaviour.** Its deliverable is a number, a verdict, and a decision record.
Any resampling code you write is a throwaway prototype that must NOT be merged.

## This spec is fully self-contained — that is deliberate

You are forked to run in isolation. Nothing else in flight touches what you touch:

- **No `src/` changes.** AC-7 requires `git diff` against `main` to show no functional `src/`
  change. If you find yourself keeping a source edit, you have left the spec.
- Your outputs are a **harness** (under `scripts/` or `bench/`), a **new DEC file**, and a
  **result appended to `docs/backlog.md`**'s linear-light entry.
- SPEC-119, the only other framed STAGE-046 spec, touches `src/image`, `src/lint` and `tests/` —
  **zero overlap**.
- **Nothing is blocked on you.** You *gate* the linear-light fix, so finishing early only removes
  a dependency. Take the time to get the instrument question right.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-120-measure-the-linear-light-premise.md`,
   in full. **8 ACs, 5 settled design calls, no failing tests.**
2. **`docs/backlog.md`**, the entry `## Open — resize resamples in sRGB, not linear light` — the
   measured claim you are testing, and its own falsification gate.
3. **The code** — `src/operation/mod.rs:395-527` (`Resize::apply`: `to_rgba8` at `:396`, the
   `fast_image_resize` handoff at `:519`); `src/quality/mod.rs:25-100` (the scorer, and that it
   consumes **8-bit sRGB**); `src/cli/report.rs:329` (the equal-dimensions rule).
4. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR, **§15**.

## The thing that makes the naive experiment impossible

**SSIMULACRA2 requires equal dimensions.** `src/cli/report.rs:329`: *"The two images MUST have
equal dimensions."* So "score the downscale against its source" **errors rather than answering**.
Do not spend time discovering this.

The shape that works — produce a reference at the target size, score both candidates against it:

```
source ──┬─► crustyimg today (sRGB U8x4 Lanczos3)  ─┐
         ├─► prototype (linear-light f32)          ─┼─► SSIMULACRA2 vs reference
         └─► REFERENCE (independent, f32 linear)   ─┘
```

## The design calls are SETTLED — do not reopen them

1. **The reference comes from OUTSIDE this codebase** — numpy/Pillow with explicit linearization,
   or ImageMagick with an explicit colorspace. **Not** `fast_image_resize` with different flags,
   and not crustyimg. State the tool and version; commit the generator.
   > AGENTS §12's "no ImageMagick" rule governs **test fixtures**, which must be hermetic. This is
   > a one-off measurement harness whose validity *depends* on independence. Using an outside tool
   > here is the right call, not an exception you are smuggling.
2. **Prove the instrument can see the effect before trusting a null.** ← *the load-bearing call*
3. **Measure the physical quantity too** — mean luminance error vs the reference, alongside the
   SSIMULACRA2 delta.
4. **The alpha half gets its own, simpler oracle** — max colour error at transparent edges. Not
   SSIMULACRA2.

## Call 2 in detail, because everything rests on it

SSIMULACRA2 is tuned for **compression artifacts** and eats **8-bit sRGB**. Whether it registers a
systematic luminance shift from gamma-incorrect resampling is **itself unknown**.

That matters because a null result has two readings — *the premise is false* and *the instrument
is wrong* — and **they lead to opposite decisions.**

So: build an extreme case (thin bright lines on black, downscaled hard), confirm the physical
error is large, then **check whether SSIMULACRA2 registers it**.

- If it does → the instrument works, and the realistic-case numbers mean what they say.
- **If it does not → the verdict is "SSIMULACRA2 is the wrong gate for this question", and you
  propose the instrument that would settle it.** That is a legitimate, complete outcome — not a
  failure to finish.

**A gate you never proved could fire is not a gate.**

## Corpus

`bench/corpus/` (SPEC-088, DEC-074) — license-clean, committed. Use at minimum:

- a **synthetic worst case** you build for Call 2's positive control (thin bright features on dark);
- `graphic_large.png` — the closest existing case to the premise's worst case;
- `photo_forest_cc0.jpg` — the representative photo.

## What "done" looks like

- A **verdict**, stated plainly, as exactly one of: *premise holds, spec the fix* / *premise does
  not hold, close it* / *SSIMULACRA2 cannot settle this; here is the instrument that can*.
- Both metrics per case, plus the alpha number.
- The harness committed so the number can be **re-derived rather than trusted**, and re-run once
  to confirm it lands in the same place.
- A **DEC either way.** A closed premise deserves the same record as an accepted one.
  `affected_scope`: `src/operation/**` if the verdict is *proceed*, `[]` if *close*.
- The result appended to `docs/backlog.md`'s linear-light entry.

## Guardrails

- **Own git worktree**, branch `chore/spec-120-measure-linear-light-premise`. Do not work in the
  primary checkout — several sessions have been live in this repo.
- **Report the disagreement if the two metrics disagree.** Do not reconcile them into a tidier
  story than the data supports.
- **Budget: this is an S.** Past ~2 hours, stop and report what you have and what remains
  unmeasured. Note that cost tracks **rebuilds**, not minutes — SPEC-117 cost $23 in 62 minutes
  because of its control rebuilds. You should need far fewer.
- `git commit -s` (DCO). Never `git reset --hard`. macOS has no `timeout(1)`.
- **A piped command reports the pipe's exit code** — redirect and read `$?`.
- **Do not merge the PR. Do not bump the version.**

## When you finish, in this order

1. Fill in the spec's `## Build Completion`, including its three reflection questions.
2. Append a build cost session entry to `cost.sessions` (see below).
3. Write the DEC, with `affected_scope` set per the verdict.
4. Run `just advance-cycle SPEC-120 verify`, and **CONFIRM it moved** — `git diff` on the spec
   should show the `cycle:` line change. It reports success even when it changes nothing.
5. Open the PR. **Do not merge it.**

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by something only
your session emitted — **never by "the newest `.jsonl`."** Price per component at the anchors of
the model `.message.model` actually reports (you are expected to be **Opus**: $5/$25 per MTok;
cache_creation ×1.25 input, cache_read ×0.10 input).

**Measure at session end, not mid-session.** Measured twice in this project: SPEC-114 reported
$25.75 mid-run and finished at $34.31; SPEC-117's build reported $11.76 mid-run and finished at
**$23.06** — 49% low. Re-measure as the last thing you do.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
