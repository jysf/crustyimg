---
task:
  id: SPEC-123
  type: task
  cycle: ship
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-010
  stage: STAGE-042
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-opus-5
  created_at: 2026-08-16

references:
  decisions:
    - DEC-058
    - DEC-077
  constraints:
    - clippy-fmt-clean
    - one-spec-per-pr
  related_specs:
    - SPEC-120
    - SPEC-091

value_link: >
  The repo's reproducible-build story — `build --frozen`, the lockfile, the
  cache key — rests on byte-stable output. Upstream gives no such guarantee for
  AVIF and has a filed nondeterminism bug. Nobody has measured whether the claim
  we already ship is true.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Framed off the
        `docs/backlog.md` entry that says "measure before claiming either way",
        and because two separate roadmap items (encoder threading, the deploy
        pipeline benchmark) are gated on the answer.
    - cycle: build
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 67302832
      duration_minutes: 215
      recorded_at: 2026-08-17
      tokens_breakdown:
        input: 636
        output: 293635
        cache_creation: 924671
        cache_read: 66083890
      estimated_usd: 46.17
      note: >
        MEASURED — transcript sum over 318 assistant messages, priced per
        component at the Opus anchors the transcript's `.message.model` actually
        reports ($5/$25 per MTok; cache_creation ×1.25 input, cache_read ×0.10
        input). Cache reads are 98.2% of volume, so a flat rate would overstate
        this by more than an order of magnitude (DEC-083). Transcript identified
        by content — the only one of 100 in this project containing
        `spec123_avif_thread_determinism` — not by recency. ⚠ A THIRD DATA POINT
        FOR THE MEASURE-AT-THE-END RULE, after SPEC-114 and SPEC-117, and this
        one has a sharper edge: the cycle recorded $32.80 at 242 messages with
        the PR already open and only CI observation left, then $40.37 at 285,
        then $46.17 at 318. **The "almost done" reading was 29% low, and the
        overrun was almost entirely spent WATCHING CI** — on a 98.2%-cache-read
        cycle every poll re-reads the whole accumulated context, so a quiet
        wait costs about as much as real work. Concrete lesson for the next
        cycle prompt: watching a CI matrix settle cost **$5.80** here; measure
        cost after CI, and prefer one long wait to many short polls. Wall clock
        also includes ~35 min blocked on a full `cargo test` that had to finish
        before the timing-sensitive harness re-run.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 17012782
      duration_minutes: 14
      recorded_at: 2026-08-17
      tokens_breakdown:
        input: 270
        output: 140172
        cache_creation: 385276
        cache_read: 16487064
      estimated_usd: 14.16
      note: >
        MEASURED — transcript sum over 135 assistant messages, priced per
        component at the Opus anchors ($5/$25 per MTok; cache_creation ×1.25
        input, cache_read ×0.10 input). Read-only cycle: it made no commits, so
        this block was transcribed by the orchestrator from the verify readout
        at ship, per AGENTS §13. ⚠ Verdict was **PUNCH LIST**, not APPROVED —
        four items, all applied to the branch by the orchestrator before merge.
        Verify re-derived the mechanism independently by a stronger method than
        the build used (`image` declares `[dependencies.ravif] default-features
        = false`; `ravif?/threading` appears at exactly one place in `image`'s
        manifest, inside `rayon = [...]`), and rebuilt all three binaries from
        scratch, reproducing every hash in every leg bit-for-bit. Cost was 30%
        of the build's on 42% of its message count — the read-only, no-CI-watch
        shape is measurably cheaper.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Orchestrator main-loop, not separately metered (AGENTS §4). Merged
        PR #179, transcribed the verify cost block, computed totals, archived.
  totals:
    tokens_total: 84315614
    estimated_usd: 60.33
    session_count: 4
---

# SPEC-123: is AVIF output byte-deterministic across thread counts?

## Context

`docs/backlog.md`:

> **AVIF byte-determinism is unbacked upstream.** `aomenc`/`vpxenc` ship
> `-D, --debug` *to become* deterministic; rav1e has no guarantee and a filed
> nondeterminism bug (#2781). **If crustyimg's AVIF is not deterministic across
> thread counts, existing "reproducible" language is a false claim.**
> **Measure before claiming either way.**

That has never been measured, and three shipped things assume the answer:

- **`build --frozen`** and the lockfile. `src/build/lock.rs` records `hash` as
  *"the observed output bytes"*, promised stable *within a machine* (STAGE-021's
  determinism experiment) but not across arch/OS/codec versions. **Thread count is
  not in that list** — it is neither the machine nor the codec version.
- **The cache key** (DEC-058) — `version + features + recipe + quality + input
  ext + input content`. **Thread count is not a component.** If output varies with
  it, two runs on one machine can disagree while the key says they must not.
- **Any future threading work.** Encoder threading is filed as a probe in
  PROJ-010's brief; `par_iter run_pixel_op` is filed as a SPEC-091 follow-up.
  Both are gated on this answer, and neither should be scoped before it.

**This spec ships no behaviour.** Its deliverable is a measurement and a DEC.

## The design calls — settled here

### Call 1 — vary ONE thing, and prove the variable moved

The measurement is: same input, same version, same features, same machine,
**different thread counts** → compare SHA-256 of the output bytes.

**A control is required in both directions.** Prove the thread count actually
changed the work done — an encoder that silently ignores the setting produces
identical bytes and looks like a clean pass
[[a-control-you-never-verified-applied-is-not-a-control]]. Show a wall-clock or
CPU-time difference between the counts, or instrument the thread pool. **Without
that, a "deterministic" verdict is unearned.**

### Call 2 — test what actually ships, at the surface users touch

The claim under test is about **crustyimg's output**, not rav1e's. Drive the
binary (`convert --format avif`, `web`, `optimize`), not a library harness.
Include the **lean build** — `--no-default-features` drops the AVIF encoder, so
the matrix legs differ in what they can even produce.

### Call 3 — three outcomes, and two of them are findings

- **Deterministic across thread counts** → the reproducible language is safe, and
  threading work is unblocked on this axis. Record it.
- **Non-deterministic** → **existing shipped language is false** and that is the
  finding: `RELEASING.md`, the lockfile docs and any "reproducible" claim need
  correcting, and encoder threading needs a determinism story before it is scoped.
- **The encoder ignores the thread setting** → the question is moot *today* and
  becomes live the moment anyone changes it. Record it as such; do not report
  "deterministic".

### Call 4 — run-to-run at a fixed thread count, and it decides something

Is output byte-identical **run-to-run at a fixed thread count** on one machine?
STAGE-021 measured that once; it is the narrower claim the lockfile actually makes.

⚠ **Amended 2026-08-17: this is not the cheap extra it was framed as.** Since thread
count feeds tile partitioning (see Inputs), a *pin* is a candidate fix for any
variance this spec finds — and whether a pin would be **sufficient** depends entirely
on this answer. Stable run-to-run at a fixed count → a pin is a real fix. Not stable →
there is residual nondeterminism underneath it and a pin only narrows the problem.
Run it with enough repeats to be a claim.

## Inputs

- `docs/backlog.md`'s determinism entry.
- `src/build/lock.rs:20-45` — what is promised, and in what terms.
- `src/cli/build.rs:275-302` — the cache-key components.
- **DEC-077 / SPEC-091** — why AVIF *decode* is pinned to one thread. Different
  code path; read it so the two are not conflated in the write-up.
- `bench/corpus/` for inputs; the harness shape from `scripts/spec120_linear_light.py`.
- ⚡ **`src/sink/mod.rs:679` — the AVIF encode call, and the reason the lever is not obvious.**
  crustyimg constructs `AvifEncoder::new_with_speed_quality(..)` and **never calls
  `with_num_threads`**, so the encoder takes `image` 0.25.10's documented default: *"all
  threads in the default `rayon` thread pool"* (`codecs/avif/encoder.rs:89-91`), which
  `ravif` 0.13.0 resolves as `rayon::current_num_threads()` (`av1encoder.rs:653`).
  So the count is the **ambient pool size**, set by `RAYON_NUM_THREADS` globally or by
  `--jobs`'s scoped pool — which is read in exactly two places, `src/cli/build.rs:661` and
  `src/cli/optimize.rs:177`. ⚠ **`--jobs` is silently ignored by `convert`** and the five
  other serial verbs (STAGE-042's `run_pixel_op` item), so a matrix built on `-j` for
  `convert` measures one thread count three times and reports a false "deterministic".
- ⚡ **`ravif` 0.13.0 `av1encoder.rs:651-655` — thread count is an ENCODER PARAMETER.**
  `tiles = threads.min((w*h) / min_tile_size²)`, so the ambient count sets the **AV1 tile
  count**, and tile boundaries reset entropy-coding contexts. A different tile count is a
  different bitstream **by construction**, independent of rav1e's nondeterminism bug. This
  raises the prior on a "non-deterministic" verdict; it does not replace measuring it.
  ⚠ **The `.min(..)` is a third false-null mechanism** beyond the two Call 1 names. At
  speed 6 `min_tile_size` is 128 or 256 (ravif's `high_quality` gate, `:544`), so the
  size-term is **16 or 4** for `graphic_large.png` and **25 or 6** for
  `photo_forest_cc0.jpg` — a 1/4/8 matrix can clamp two legs to the same tile count,
  producing identical bytes that are not determinism. **Report the computed clamp beside
  the hashes; choose inputs where the thread term binds.**

## Outputs

- **A DEC** recording the verdict either way, with the measurements and the
  control. `affected_scope`: `src/sink/**` if the answer constrains encoding,
  `[]` if it is purely a documentation finding.
- **The harness**, committed, so the number can be re-derived rather than trusted.
- **Corrections to any shipped "reproducible" language** if Call 3's second branch
  fires — in the same PR, since a false claim should not outlive its disproof.
- **No `src/` behaviour change.**

## Acceptance Criteria

- [ ] **AC-1.** Output SHA-256 compared across **at least three thread counts**
      (1, a middle value, all cores) on the same input, version, features and
      machine. Report the hashes, not a verdict.
- [ ] **AC-2.** **The control fires** — evidence that thread count changed the
      work (timing delta or instrumentation). A null result without this is
      unearned.
- [ ] **AC-3.** Driven through the **shipped binary** on `convert --format avif`,
      `web` and `optimize`, not a library harness.
- [ ] **AC-4.** **Run-to-run stability at a fixed thread count** re-confirmed
      (Call 4).
- [ ] **AC-5.** A verdict stated as exactly one of Call 3's three outcomes.
- [ ] **AC-6.** If non-deterministic: **every shipped "reproducible" claim
      located and corrected**, with the grep cited
      [[mechanical-sweeps-need-a-mechanical-check]].
- [ ] **AC-7.** **No functional `src/` change** — `git diff` against `main` shows
      none; the shipped test suite untouched and green.
- [ ] **AC-8.** Reproducible from the committed harness — re-run and confirm the
      numbers land in the same place.

## Failing Tests

**None, and that is correct** — this is a measurement. AC-2's control is the
load-bearing criterion in their place, exactly as in SPEC-120.

## Implementation Context

### Out of scope
- **Making** AVIF deterministic. If it isn't, that is a finding, not this spec's
  fix.
- Encoder threading and `par_iter run_pixel_op` — both gated on this.
- Decode threading (DEC-077 settled it).

## Notes for the Implementer

- **Report hashes, not conclusions.** The verdict follows from the table.
- **A "deterministic" answer is the one most likely to be wrong for a boring
  reason** — the setting was ignored. Call 1's control is what separates them.
- **Budget: S.** Past ~2 hours, report what you have.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-123 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `chore/spec-123-avif-byte-determinism`
- **PR (if applicable):** #179
- **All acceptance criteria met?** yes (AC-6 did not fire — see below)
- **New decisions emitted:** **DEC-094** — *AVIF thread settings never reach the encoder — the
  machine's core count does.* `affected_scope: src/sink/**`, `src/quality/mod.rs`.
- **Deviations from spec:** two, both deliberate.
  1. **STAGE-042's item was corrected on this branch**, not left for main. AGENTS §13 puts stage
     bookkeeping on main at ship, but two of that item's sentences are *claims this build
     falsified* ("the encoder already takes every core"; "`build -j 2` and `build -j 8` can write
     different bytes"). A disproved premise sitting in the scoping note for the very next spec is
     the harm AC-6 exists to prevent, so it was corrected in place, with the correction marked.
  2. **AC-6's sweep was run even though AC-6 did not fire.** The verdict is Call 3's *third* branch,
     not "non-deterministic", so no shipped "reproducible" claim is falsified by the thread axis.
     The sweep is reported anyway (below) because a different claim — the lockfile's caveat list —
     turned out to be incomplete.
- **Follow-up work identified:**
  1. **Correct `src/build/lock.rs:32-37`'s caveat list and the `[env]` block.** The output hash is
     qualified with arch/OS/codec version; **core count belongs in that list** and in `[env]`,
     because it sets the AVIF tile count. Until then `diff` can flag a differently-cored machine as
     a real regression under the same `env.target`. Not done here: AC-7 forbids a `src/` edit.
  2. **Split STAGE-042's encoder-pin item in two.** `image/rayon` is the *performance* lever
     (measured 5.7× / 4.4×, byte-identical on a 14-core host); `with_num_threads(Some(N))` is the
     *determinism* lever (changes every byte). They were one item because the encoder was believed
     to be multi-threaded already. It is not.
  3. **Consider whether core-count tiling is the right default at all.** The shipped build takes
     the worst cell: full multi-tile compression penalty (**+47.9 %** bytes on a 512×512 graphic),
     zero parallelism. That is a quality-per-byte cost on a tool sold on quality-per-byte.
  4. **`par_iter run_pixel_op` (SPEC-091 follow-up) is unblocked, conditionally.** Pool size is
     invisible to the encoder *today*; the clearance lapses the moment `image/rayon` is enabled.
  5. Not measured: **a second host with a different core count.** The core-count dependence is
     established by driving `tiles` directly plus the source path, not by two hosts disagreeing.

### The verdict (AC-5)

> **hashes identical, control did not fire → the encoder ignores the thread setting.**
> Moot today, live the moment anyone changes it. **Not** "deterministic".

Mechanism: `ravif` is compiled without its `threading` feature (reachable only via `image`'s
`rayon` feature, which `avif = ["image/avif"]` does not enable), so it uses its own `rayoff` shim —
sequential `join`, and `current_num_threads()` = `std::thread::available_parallelism()`. Full
table, controls and numbers in **DEC-094**; re-derive with
`python3 scripts/spec123_avif_thread_determinism.py`.

### AC ledger — each criterion and the evidence that meets it

| AC | evidence | met |
|---|---|---|
| **AC-1** ≥3 thread counts | `RAYON_NUM_THREADS` ∈ {1, 4, 14} (1 / middle / all cores), same input, version, features, machine. Hashes reported, verdict derived after. | yes |
| **AC-2** the control fires | ⚠ **Read carefully.** On the *shipped* build the control **does not fire on the encode** — `cpu/wall` 0.99–1.00 across every leg, timings flat to the millisecond — and *that is the verdict*, not a gap. Two controls make the null earned rather than assumed. **(i) Positive control:** the `--features image/rayon` probe, same harness, same corpus, same verbs, moves the bytes (3 distinct hashes/input), the clock (0.530 → 0.093 s) and `cpu/wall` (1.00 → 7.09) — the measurement demonstrably *can* register a thread-count change. **(ii) In-process control:** in leg A2, `web`'s auto path at 14 threads runs `cpu/wall` **1.17** against 1.00 at one thread, reproduced on three independent runs — so `RAYON_NUM_THREADS` verifiably took effect *inside the very process whose output did not move*. It reached the program; it did not reach the encoder. Leg F then pins the shipped tile count to the core count by byte-identity. | yes |
| **AC-3** shipped binary, 3 verbs | `convert --format avif`, `web`, `optimize`, all through `target-full/release/crustyimg`; no library harness. ⚠ Disclosed weakness: with `--format` pinned, all three collapse to one identical encoder call at q80, so **leg A2** additionally drives `web`/`optimize` on their auto path (q85). | yes |
| **AC-4** run-to-run at a fixed count | 10 repeats × 3 verbs at `RAYON_NUM_THREADS=14` → 1 distinct hash each (30 runs). Stable, so a pin would be *sufficient*, not merely narrowing. | yes |
| **AC-5** verdict as one of three | Call 3's **third** branch, stated verbatim above. | yes |
| **AC-6** corrections if non-deterministic | **Did not fire** — the verdict is not "non-deterministic". Sweep run and cited anyway (below); nothing falsified. | n/a |
| **AC-7** no functional `src/` change | `git diff main -- src/` is **empty**. Shipped test suite untouched; `cargo fmt --check` rc=0, `cargo clippy --all-targets -- -D warnings` rc=0, `cargo test` green (exit codes read from `$?`, never through a pipe). | yes |
| **AC-8** reproducible from the harness | Harness committed at `scripts/spec123_avif_thread_determinism.py`; run end to end twice on an idle machine — **43** hash occurrences, **9** distinct values, identical multiset, compared mechanically rather than by eye. | yes |

### AC-6's sweep, cited

`/usr/bin/grep -rniE "reproducib|byte-identical|byte-stable|determinis"` over `*.rs *.md *.toml
*.yaml *.yml`, excluding `./target`, `./projects`, `./decisions`, `docs/backlog` → **316 hits**
across 91 files. Narrowed to the surfaces that could carry the claim under test — `src/build/`,
`src/cli/build.rs`, `src/sink/mod.rs`, `README.md`, `CHANGELOG.md`, `docs/api-contract.md`,
`docs/USAGE.md`, `docs/cli-reference.md` — **30 hits**. Reading them: none claims byte-stability
across thread counts, and none is falsified by this measurement. The 316 is dominated by unrelated
senses ("deterministic classification", "deterministic encoders" for test fixtures, "reproducible
comparison" in benchmark prose). **`RELEASING.md`** — named by Call 3 as the first place a false
claim would live — returns **0** hits for all five stems (positive control on the same file: 127
lines, 20 hits for `release`), and mentions no output hash, lockfile hash or AVIF at all. ⚠ **The
sweep's scope is itself a claim:** it cannot see a reproducibility promise phrased without any of
those five stems.

One hit is **incomplete rather than false**, and is follow-up 1 above: `src/build/lock.rs:32-37`.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Nothing slowed me down, but the spec and
   the prompt both asserted the mechanism one layer too shallow: *"the encoder takes `image`'s
   documented default — all threads in the default rayon thread pool"*, cited from
   `codecs/avif/encoder.rs:89-91`. That doc comment is true of `image` **with its `rayon` feature
   on**, and crustyimg's build does not have it on. The prompt's *"Do not spend an hour discovering
   this"* section was therefore itself wrong on the load-bearing point, and its predicted verdict
   ("non-deterministic, for a structural reason") is the opposite of what the binary does. It cost
   little, because the first smoke test showed three thread counts agreeing to the millisecond —
   which is not what a real lever looks like — but a builder who trusted the prompt over the
   measurement would have shipped a confident wrong answer. **A dependency's documented default is
   a claim about a feature set, not about your build.**
2. **Was there a constraint or decision that should have been listed but wasn't?** The **Cargo
   feature graph** should have been an Input alongside the source lines. Everything that decides
   this spec lives in `Cargo.toml` + `cargo tree -e features`, and the spec pointed only at `src/`
   and the crate sources. Relatedly, the prompt's clamp analysis left `min_tile_size` at "128 or
   256" without resolving which: it is **128** for every verb in the matrix (quality 80 and 85 both
   land under ravif's threshold), so the size terms are 25 and 16 and no main-matrix row is
   clamped. Transcribing `quality_to_quantizer` settled it in minutes and made the clamp column
   cheap — and made room for leg G, which demonstrates the clamp on purpose at `-q 50`.
3. **If you did this task again, what would you do differently?** Build the `--features
   image/rayon` probe **first**, before the shipped matrix. It is the positive control, it takes 36
   seconds, and it answers "can this measurement see anything at all?" — the only question that
   makes a null worth reporting. I built it third, after reading source to explain a null I had
   already collected. Same answer, worse order: I spent a stretch reasoning about *why* the hashes
   were identical when one build would have told me *whether* they could ever differ.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

**1. Did this spec deliver what it set out to?** Yes, and the answer was none of the
two outcomes anyone expected. Call 3's **third** branch fired: the encoder ignores the
thread setting, because `ravif` is compiled without its `threading` feature. Both the
design cycle and the build prompt predicted *non-deterministic by construction* from
`ravif`'s tile formula — the tile mechanism was real, but the lever driving it does
not exist in this build. **Two riders ended up outranking the deliverable**: AVIF
output varies with the machine's **core count** (in neither the cache key nor the
lockfile's `[env]`, so `diff` can report a differently-cored host as a real
regression), and the shipped build takes the worst cell on both axes — **+1.5% /
+47.9%** bytes against a 1-tile encode at **5.7× / 4.4×** the wall clock of the same
tiles in parallel.

**2. What would we do differently?** State design findings as **priors, not
conclusions**. A confident wrong prediction in the spec's `## Inputs` and the build
prompt meant the build had to refute a stated position rather than answer an open
question, then correct five documents that carried the wrong version. The root cause
is banked: *a dep's documented default is a claim about a feature set* — `image`'s
"all threads in the default rayon pool" is true of `image` **with `rayon` on**, and
`avif = ["image/avif"]` never enables it. `cargo tree -e features` cannot see this;
the **build fingerprint** can.

**3. What did it cost, and was it worth it?** **$60.33** across build ($46.17) and
verify ($14.16) on a spec sized `[S]` — against SPEC-120, the same "measure a
premise" shape, at $8.69. The 5.3× gap is not effort: SPEC-120's premise **held**
(confirm and stop), this one's was **wrong** (refute, re-derive, correct everything
downstream). Identifiable waste was ~$6: $5.80 watching CI, now fixed in
`cost-snippet.md` and replaced with a backgrounded `gh pr checks --watch`. The rest
bought a 5.7× performance lever, a compression defect on the flagship codec, and a
live false-positive path in `diff` — none of which the spec was scoped to find. The
durable lesson is about **sizing, not spending**: a measurement spec's cost is set by
whether its premise survives, which is unknowable at framing.
