# SPEC-125 — VERIFY prompt

Cycle: **verify**. New session, **read-only**. You did not build this.

**What it claims:** the 8-bit downgrade warning now fires for **every target measured to be
8-bit-only** — BMP, lossless WebP and AVIF added to SPEC-121's JPEG + lossy WebP — with PNG and
TIFF confirmed 16-bit-capable and staying silent, and GIF/ICO excluded for two reasons that are
**not** "the format holds the depth." And `web`/`optimize` stop reporting a false-perfect
`ssim 100.0` across a depth-reducing winner. **Reporting-only: no output bytes change.**

## ⚠ Read this first — the PR is already MERGED

**PR #185 merged to `main` at `2735f60` before this cycle ran.** Review `main`, not a branch.
The pre-merge base is **`f35e28a`**; `git diff f35e28a..2735f60` is the change under review.

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec125-verify main
```

**Make no commits.** Emit your `## Cost readout` and verdict in the return message — that is the
deliverable (AGENTS §13). A ❌ or a punch-list item here means a follow-up commit on `main`, not a
blocked merge, so **say plainly what would have to change** rather than softening it.

## Read in order

1. **The spec** — 7 ACs, 3 design calls, 4 failing tests, `## Build Completion` with **three
   deviations**. ⚠ Its `## Context` carries a **correction applied after the merge** — read it.
2. **DEC-097** (new — the measured capability table and the Call 2 reasoning), **DEC-095**
   (SPEC-121/122's rule, not to be reopened), **DEC-019** (the scorer boundary), **DEC-090**.
3. `src/sink/mod.rs`, `src/cli/optimize.rs`, `src/analysis/decide.rs`, `tests/sink.rs`.

---

## Already settled — do NOT re-derive

1. **Cost.** Re-derived: `$81.78` is exact at 654 of 670 messages; full session **$84.58**
   (246,724,470 tokens, 124 min, Sonnet throughout). Applied. **Measure your own; don't audit it.**
2. **`tests/colour_type_preservation.rs` (+14/-11) is COMMENT-ONLY.** I diffed it: SPEC-121's
   regression guard has **no assertion weakened, added or removed**. This is the "a guard that gets
   relaxed whenever it fires stops being a guard" check, and it passes. Don't re-open it.
3. **The spec's Context repro was wrong and is now corrected** (`convert` never scores; the
   `ssim 100.0` line is `web`'s). The build found it by driving; the orchestrator applied it to the
   spec at `5062ac6`. Nothing left to do.
4. **No double-warn on lossy WebP.** I traced it: the `webp-lossy` arm `return`s inside
   `if let Some(q) = quality`, so it never falls through to the `Bmp | WebP` fallback, and the
   `quality == None` case correctly falls through and says "lossless WebP". Confirmed, not assumed.
5. **CI is green on `main` at `2735f60`** — 15 checks, all success, `pages` included.

---

## Six specific things

### 1 — ⚡ AC-6 is the item this whole spec lives or dies on. Drive it.

The spec promised a **reporting fix**. The diff touches `src/cli/optimize.rs` (+37/-14) and
`src/analysis/decide.rs` (+17/-0) — the candidate-search surface. **AC-6 pins `optimize`'s
candidate selection byte-identical to `main` on the corpus**, and Call 2 said in terms: *if your fix
would touch DEC-019's scorer path, STOP AND REPORT.*

The build says `scored_source_depth` reads the already-decoded images' own colour depth and never
touches the shared scorer. **That is a claim about a code path, and it is the one claim that, if
wrong, turns a reporting fix into a silent byte-changer in a release that is about to be tagged.**

Drive it: run `optimize` over the full corpus at `f35e28a` and at `2735f60` and diff the **output
bytes and the chosen format per file**, not just a summary line. Then satisfy yourself by reading
that no value derived from `scored_source_depth` can flow back into candidate ranking.

### 2 — AC-2's table is a mechanical sweep, so it needs a mechanical check

The set — BMP, lossless WebP, AVIF **warn**; PNG, TIFF **don't**; GIF, ICO **excluded** — was
derived by encode→decode-back rather than copied from the spec's list, which is exactly what Call 1
asked for. **Re-derive it yourself** [[mechanical-sweeps-need-a-mechanical-check]], and push on the
part the build did not have to answer:

- **Does the answer change across feature sets?** AVIF's warn sits inside `#[cfg(feature = "avif")]`
  and lossy WebP's inside `#[cfg(feature = "webp-lossy")]`. Under `--no-default-features`, AVIF is
  not built — what happens to a `>8-bit → avif` request, and is the resulting behaviour honest?
  Sweep **default**, `--no-default-features`, and `--features webp-lossy`.
- **Is the enumeration exhaustive?** The fallback arm keys on `matches!(format, Bmp | WebP)`. Every
  format reaching that path that is *not* named is silently in the "holds the depth" bucket by
  omission. Is that true for all of them, or just the ones someone thought to test?
  [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]

### 3 — The JSON contract gained a key. That is a public surface.

`ssim_source_depth` is now emitted on `--explain json` / `--json`, and `docs/api-contract.md`
changed. The build's defence is that it is **additive and gated** — present only when the score is
blind to a real depth reduction, so a run scoring a depth-preserving winner keeps today's schema.

**Verify that gating behaviourally**, both directions: a depth-preserving run must emit **byte-identical
JSON to `f35e28a`**, and a depth-reducing one must carry the key. Then rule on the contract question
the build did not raise: is a new conditional key in a documented JSON output something this repo
considers additive, or does it want a note? crustyimg is a **published library** and this ships in a
tag imminently.

### 4 — Two strong claims about a dependency, both load-bearing for an exclusion

GIF and ICO are excluded on claims about `image`'s behaviour, not about bit depth:

- **GIF "REJECTS a >8-bit source outright"** — a typed `SinkError::Encode`, exit 5. Drive it. If GIF
  in fact narrows silently for some colour types, it belongs in the warning set and the exclusion is
  a hole.
- **ICO "cannot be read back by `image`'s own ICO decoder for ANY source colour type, 8-bit RGB
  included."** That is a **severe** claim about a shipped dependency
  [[a-grep-of-src-cannot-see-a-dependencys-default]]. Reproduce it. If it holds, confirm the
  STAGE-042 item is filed where `just backlog` reads and states the maintainer ruling it needs
  (warn / fix / accept). If it does **not** hold, ICO's exclusion loses its reason.

### 5 — AC-5's negative control, one revert per independent condition

The build reports "two independent-condition reverts" for Call 1 and Call 2. **Drive both**, and
check they are genuinely independent — that reverting Call 1 does not also disable the test guarding
Call 2, which is how SPEC-113 shipped a vacuous test (AGENTS §15). The evidence is the **behavioural
flip**, not a hash.

Then the other half of AC-5: an 8-bit source through the same verbs warns **nowhere** and its
output is **byte-identical to `f35e28a`** — stderr included.

### 6 — AC-4 asserts the rendered line. Check what it renders across all three channels.

The qualifier must land on the **default summary**, `--explain human`, **and** `--explain
json`/`--json`. A test that pins one channel and lets the others drift is
[[a-guards-advertised-reach-is-a-claim]]. And confirm `-o -` keeps stdout pure — the warning and the
qualifier are both stderr-only (AGENTS §11).

---

## Also check

- **Decision drift:** `./scripts/decisions-audit.sh --changed f35e28a` — **pass the base ref**, or a
  clean checkout reports "No changed files in scope" and exits 0 on a green that cannot go red.
- **DEC-097's `affected_scope`** covers all four source files it governs (`sink`, `cli/optimize`,
  `analysis/decide`, and whatever else), and its confidence is honest (AGENTS §17).
- **DEC-095 not reopened** — no `Operation` body changed (Call 3), and SPEC-121's and SPEC-124's
  tests stay green.
- **Every file the diff touches is listed** in Build Completion — 12 files.
- **`just backlog` reads the ICO item back.** The build had to fix its own nesting once already.
- **AC-7's matrix** — three legs, fresh `CARGO_TARGET_DIR` each, clippy and `fmt --check` on each.

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not bump the version.
- **⚡ NEVER POLL CI.** `main` is already green at `2735f60`; one snapshot if you need it. Do not
  re-read a backgrounded watcher's output while it runs. Take your cost reading **once, at the end**
  — and note that this wave's cycles all under-report by 3–7 % because a cycle cannot count the
  messages that write its own cost block. Say so if you snapshot early.
- **Budget ~200 exchanges.** The build ran **670** against ~150 — the fourth consecutive cycle to
  blow its budget without the checkpoint firing.
- macOS has no `timeout(1)`. A piped command reports the **pipe's** exit code — redirect and read
  `$?`. zsh does **not** word-split unquoted parameters — use `while IFS= read -r`, and write
  `"${B}:path"`, never `$B:path`. Use `/usr/bin/grep`, not the shell's aliased one: a sweep in this
  repo has already silently dropped a file [[rtk-can-silently-corrupt-grep-counts]].
- **`pages / build + browser smoke` is a known intermittent** — a 10 s Chrome-startup cap against
  the file's own 90 s convention, filed on STAGE-042. If you see it red, that is it. Don't chase it.

## When you finish

1. **No commits.** 2. Emit `## Cost readout` (`cost-snippet.md`; price at the anchors
`.message.model` actually reports). 3. Verdict — ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED, with
**item 1's AC-6 result stated explicitly and first** — it is the difference between a reporting fix
and an unannounced byte change going into a tag.
