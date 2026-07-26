# Session prompt: frame the project that follows PROJ-008

Working dir: `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg`, on `main`.
Git user `jysf`; **every commit signed off (`-s`)** — DCO is enforced and has gone red three
times for a missing `-s`. **Never `git reset --hard`** — a previous session used it and
silently destroyed a sub-agent's uncommitted work.

Orient with `just status` and the auto-loaded `MEMORY.md`.

**PROJ-008 is closed.** Its thesis — compile the engine to wasm, ship an npm library, ship a
client-side demo — shipped: 0.6.0 is live, the demo is live, `crustyimg-wasm` is published.
Per AGENTS §2 a new project may be framed only once the prior one ships, so this session is
unblocked.

Your job is to **frame the next project**. Three things are already decided and two are not;
this prompt separates them.

---

## 1. The measured state of the classifier regression

A max-effort multi-agent review of the merged SPEC-105 commit (`54ba05e`) produced 15
findings. Its *measured* numbers were re-derived on 2026-07-25 against a from-scratch release
build of `main`. **The re-derivation supersedes the review's numbers** — design against these,
not against the originals.

Full write-up, committed: `docs/research/pr113-classifier-review-findings.md`, section
**"Re-derivation (2026-07-25)"**. The verdicts:

| # | Claim | Measured | Verdict |
|---|---|---|---|
| 1 | 3840×2160 code-editor screenshot: 0.79 native → **4.24** at `--max 2048` → `photograph` → 358,227 B lossy for a 111,095 B source | Four substituted screenshots (dark dense 4K code, light theme, sparse code, image-editor canvas). Entropy rises monotonically with downscale in **every** case, but tops out at **1.14 / 0.80 / 2.04 / 2.37**. All stay `graphic-logo`/`document`, all lossless. | **NOT REPRODUCED** (magnitude). Mechanism **CONFIRMED**. |
| 2 | 3000×2250 1-bit halftone: 0.56 → 4.79 at `--max 2048` → lossy | **0.62 → `document` → passthrough** at native; **5.29 → `photograph` → lossy AVIF** at `--max 2048`. **844,492 B out for a 45,527 B source — 18.5× larger**, `larger_than_source: true`, **SSIMULACRA2 69.2**. At `--max 2560`: 5.20, 1,590,638 B (35×). | **CONFIRMED**, worse than claimed. |
| 3 | committed `tests/fixtures/classify/dithered_graphic.png`: 3.03 native → **7.08 at `--max 256`** → lossy | **3.03 native** (`graphic-logo`, unchanged through `--max 512`); **7.08 at `--max 256` → `photograph` → lossy AVIF**, SSIMULACRA2 81.8. At `--max 128` it flips to `icon` (7.15) → lossless. | **CONFIRMED exactly** — repo and review harness agree. |
| 4 | Mixed UI+photo 1600×1000: `document`→lossless at 25%/33% (3.35/3.92), `photograph`→lossy q85 at 50% (4.93) | Composite of `color_photo_fuji.png` into flat chrome + text bar: **25% → 4.56, 33% → 4.89, 50% → 5.30 — all three `photograph` → lossy AVIF q85.** Outputs smaller, not larger (96% savings at 50%); SSIMULACRA2 81.8. | **DIRECTIONALLY CONFIRMED**, worse: the "safe" 25%/33% band does not exist. Harm is quality on the text half, not size. |
| 5 | `PHOTO_ENTROPY_STRONG = 5.5` leaves the classify suite green | Mutation applied at `src/analysis/mod.rs:97`; `cargo test --release --lib analysis` → **52 passed, 0 failed**, including `calibration_gap_holds_for_committed_fixtures` and `real_grayscale_photo_is_photograph_not_graphic`. Reverted; tree clean. | **CONFIRMED.** The suite cannot detect a threshold move that reinstates the original bug. |
| 6 | 128×128 EXIF-stripped B&W photo thumbnail classifies `Icon` → `LosslessFlat` | 128×128 centre crop of `grayscale_photo_leica.png`, `-strip`: **entropy 6.02 → `icon` → lossless**. | **CONFIRMED.** DEC-047's "**any** image ≥ `PHOTO_ENTROPY_STRONG` is a `Photograph`" is false as written. |
| 7 | Dirty alpha: 6.25 native vs 1.04 at `--max 500` | Not attempted. | **COULD NOT TEST.** |

Two findings the review did not name, also measured:

- **`web` returns larger-than-source output on the *lossless* path**, no misclassification
  required. A 3840×2160 spreadsheet screenshot: 420,717 B → **567,140 B** at the default
  `--max 2048` (`larger_than_source: true`, `savings_percent: -35`). A 256-colour 4K code
  screenshot: 154,259 B → **376,554 B** (−144%). The flag is set and `web --help` discloses
  the trade, so it is disclosed rather than hidden — but it is a poor default result to lead
  a launch post with, and it is independent of the classifier.
- **`--max 128` re-routes a promoted image back to `icon` → lossless** (dithered fixture:
  7.08/`photograph` at 256, 7.15/`icon` at 128) — finding 6's ordering bug seen from the
  other side. It masks the entropy rule at exactly the thumbnail sizes a gallery pipeline
  emits.

The structural half was separately confirmed by reading source and needs no re-checking; it
is listed at the top of the findings doc (classification runs after the resize pipeline at
`src/cli/optimize.rs:989` → `:1013`; rule 6 at `src/analysis/mod.rs:625` is unreachable dead
code; `DOC_ENTROPY_MAX` 4.5 > `PHOTO_ENTROPY_STRONG` 4.0).

## 2. Is it launch-gating? Yes.

**`crustyimg web <file>` demonstrably hands a user a file that is both 18.5× larger than its
input and visually degraded (SSIMULACRA2 69.2), through the default path, with no flags.**
The input is an ordinary 1-bit halftone — a print/scan artifact, not a contrived adversarial
case — and at native size the same file passes through untouched. The trigger is the
downscale ratio, not the input size: any dithered or halftoned source whose long edge exceeds
2048 by more than ~20% is exposed by default.

The blast radius is **dithered and halftone graphics, not screenshots**. The review's
screenshot framing was wrong on magnitude, and a screenshot-only test set would go green
against this defect. Scope the fix's fixtures accordingly.

**Recommendation: fix before the Show HN.** The post leads with `web`. Shipping a feature
wave while the flagship verb can hand a user a worse, larger file is the wrong order.

## 3. The framing decision — yours, not the previous session's

Next free project number is **PROJ-010** (001, 002, 004, 007, 008, 009 taken).

**First, before you frame anything: there is already an untracked draft on disk.**
`projects/PROJ-010-post-launch-correctness-and-consolidation/` exists as **untracked** files —
a `brief.md` plus five stages (STAGE-034 classifier regression fix, STAGE-035 hostile input pass,
STAGE-036 engineering quality and code health, STAGE-037 post-launch CLI surface, STAGE-038
post-launch polish and repo housekeeping). It predates the session that wrote this prompt and was
deliberately left untouched and uncommitted there, because framing the next project was not that
session's job. **Read it before you write anything** — it may already be most of what you need,
and it will otherwise be duplicated. Note it appears to *renumber* the carried stages
(031→036, 032→037, 033→038) and to give the classifier work and SPEC-107 their own stages; decide
deliberately whether you want that renumbering, and reconcile it against the three stage files
still sitting in `projects/PROJ-008-wasm-core-and-demo/stages/`.

Two candidate theses. **This is the maintainer's call.**

**(a) Post-launch correctness and consolidation.** Houses the carried-forward stages
STAGE-031 (engineering quality / code health), STAGE-032 (post-launch CLI surface), and
STAGE-033 (post-launch polish and repo housekeeping) — all currently `proposed` and
deliberately left in place — plus the classifier work above. *Recommended*, for the reason in
§2.

**(b) Roadmap Wave 4 — the manifest feature.** See `docs/roadmap.md`.

Do not frame both. Do not re-home stages 031/032/033 until the thesis is chosen; they are
recorded in `docs/backlog.md` as carried forward and awaiting a home.

## 4. The two-spec shape already chosen for the review findings

The maintainer has already settled the shape of the classifier work. Do not re-litigate it;
frame to it.

**Spec 1 — design-first: classification placement and scale-aware entropy.** Where
classification happens relative to the resize, and whether the entropy threshold must be
scale-aware. Evaluate seriously the **narrower alternative the review surfaced**: gate rule
4's two mis-firing clauses on `entropy < PHOTO_ENTROPY_STRONG` instead of using the
unconditional early return. That fixes the same bug at the same depth, keeps rules 5 and 6
reachable, keeps `PHOTO_ENTROPY` live, and localizes the mask so it can be deleted verbatim
once the detector is fixed. This spec likely **subsumes** the queued follow-up
"scale-normalize the flat/edge detector" rather than layering a second correction on top of
it — check that before carrying the follow-up forward separately.

**Spec 2 — evidence integrity.** Commit the boundary specimens DEC-047 cites but the repo
does not contain (the 4.58-floor photo, the 3.43 16-colour dither). Re-establish each diluted
guard with a **negative control** proving the harness can go red — the calibration guard
(`src/analysis/mod.rs:945`), the never-bigger ICC assertion (`tests/cli.rs:4392`), the
`web` no-EXIF path (`tests/cli.rs:5023`), the SPEC-084 lossy-fallback coverage
(`tests/cli.rs:4381`), and the `#[cfg(feature = "avif")]`-silenced schema test
(`tests/audit_bench.rs:171`). Correct DEC-047's two false claims (the "any image" reach
claim, and the safety claim that no hard-edged graphic reaches 4.0).

## 5. The other launch-gating item

**SPEC-107** — the hostile/edge input confirmation pass — is framed in STAGE-033 and ready to
build. It is launch-gating alongside the classifier work. Sequence both before the post.

## 6. Open question, not a decision

`docs/roadmap.md`'s "Sequencing rationale" is written from inside the AI-agent-experiment
framing in a public file. The maintainer wants it split into an internal and a public
roadmap, but **has not framed the shape yet**. Treat this as an open question awaiting his
framing — do not file it as a decision and do not act on it unprompted.

## 7. Pointers

- `docs/research/pr113-classifier-review-findings.md` — the 15 findings, the structural
  confirmations, the suggested clustering, and the re-derivation.
- `docs/backlog.md` — carried-forward stages and open items.
- `projects/PROJ-008-wasm-core-and-demo/brief.md` — the closing reflection, including what it
  says should update `AGENTS.md` / templates / constraints.
- The launch board — every item on it is maintainer-blocked and none is repo work. Leave it
  alone.

## Standing guardrails

- **`rtk` silently corrupts output.** It has returned "0 matches" for greps against files
  that plainly match, and mangled `ls` and `cargo` output, repeatedly. In the session that
  produced this prompt it reported a 91.5 GiB `cargo clean` followed by a 0.36 s build that
  had compiled nothing. Cross-check every count with raw `grep`/`find` **plus a positive
  control that must return nonzero**; use `rtk proxy <cmd>` when you need real stdout.
- An engine or shared-classifier change requires a **clean full-matrix** verify — `cargo test`
  on default, `--no-default-features`, and `--features webp-lossy`, clippy `-D warnings` on
  each, plus `fmt --check`. Incremental builds false-green; that cost this repo about a day
  on SPEC-105. The orchestrator re-runs the matrix rather than relaying a sub-agent's "CLEAN".
  When in doubt, build into a fresh `CARGO_TARGET_DIR` and confirm the log actually says
  `Compiling crustyimg`.
- `just advance-cycle` / `just archive-spec` mis-target `specs/prompts/*.md` (known
  `find_spec` glob bug) — archive by hand with `git mv`.
- Dispatch build/verify as foreground sub-agents; persist the prompt
  (`prompts/SPEC-NNN-<cycle>.md`) and the readout (`prompts/SPEC-NNN-readouts.md`).
