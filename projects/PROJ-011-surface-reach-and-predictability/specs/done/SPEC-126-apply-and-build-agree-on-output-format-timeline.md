# SPEC-126 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-126-<cycle>.md`.

> ⚠ **Cost correction, 2026-09-05.** Every dollar figure below was summed over every
> transcript line carrying `usage`; Claude Code writes one line per **content block**, so
> `input`/`cache_creation`/`cache_read` were double-counted once per extra block. Corrected:
> build **$18.02** (was $38.16), verify **$9.18** (was $15.64), re-approve **$5.96** (was
> $15.02), total **$33.16** (was $68.82). The spec's `cost.sessions` carries the recomputed
> numbers; the lines below are left as written so the record of what was believed at the
> time survives. Project-wide item on STAGE-053.

## Instructions

- [x] **design** — 2026-08-23. 7 ACs, 4 settled design calls, 4 failing tests. **PROJ-011's entry
      point**, framed from a defect driven on `main` rather than reported.
      ⚡ **Call 1 was settled by measurement, not argument.** Every other pixel verb —
      `resize`, `thumbnail`, `watermark` — plus `build` plus `apply`-at-2-inputs all preserve the
      source format; **`apply` at one input is the sole outlier on the entire surface**, so it is
      the one that moves. The opposite case is arguable (PNG avoids JPEG→JPEG generation loss) and
      loses because consistency across six paths beats a local optimum on one — and because
      changing `build` would invalidate every existing lockfile.
      ⚠ **Byte-changing, and explicitly must not ship alone** — it batches into PROJ-011's single
      migration with STAGE-050.
      📌 **Call 4 is the one most likely to be got wrong:** the test asserts that `apply` and
      `build` AGREE, not that `apply` writes `.jpg`. Pinning the format string pins the answer
      instead of the property, and would go green again the day someone changes the default for a
      good reason.
- [x] **build** — prompt: `prompts/SPEC-126-build.md` (2026-08-23). **Sonnet**, own worktree,
      branch `fix/spec-126-apply-and-build-agree`. **DEC-098 reserved in the prompt** — `next_id`
      scans only the working tree and has collided here before.
      ⛔ **Byte-changing: the prompt says do not bump the version, do not cut a release.** It
      batches into PROJ-011's single migration with STAGE-050.
      Carries the three traps this repo has paid for most recently: never poll CI (and the
      `--watch` summary line is unreliable — read the direct snapshot at the true head SHA);
      `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
      stdout and do not try to fix it; and list every file from `git diff --name-only`, not recall.
      ✅ **2026-08-23 — PR [#187](https://github.com/jysf/crustyimg/pull/187), CI green (16/16), NOT
      merged.** $38.16 / 60.8 min / 105.3M tokens (Sonnet), re-derived from the transcript after CI
      settled — an earlier mid-build reading under-reported by 26 %. DEC-098 emitted.
      ⚠ **The `cycle: verify` advance and `## Build Completion` live only on the branch**, so
      `just status` / `just backlog` will keep reporting `cycle: design` until this merges with
      STAGE-050. This line is the trace on `main`.
- [x] **verify** — prompt: `prompts/SPEC-126-verify.md` (2026-08-23). **Opus**, new session,
      read-only, own worktree, reviewing the **branch** (base ref `9b4fb80`). Ready to dispatch.
      ⚡ Leads with an **unnamed exit-code change** the orchestrator found reading the diff:
      `build_sink` now always passes `Some(fmt)`, so `apply` one input `-o -` with no `--format`
      went from `UnknownFormat` **exit 4** to **exit 0**. Probably correct, but no AC covers it,
      Build Completion claims zero deviations, and `docs/api-contract.md` is not in the diff.
      Three things are pre-settled in the prompt so verify does not re-derive them: the cost, the
      call-graph containment (`build_sink` has one caller; `build` reaches `encode_one` directly),
      and that `apply --recipe web` returns early and never touches the changed code.
      ⚠ **2026-08-23 — PUNCH LIST, 7 items.** $15.64 / 21.6 min / 22.7M tokens (Opus), 41 % of
      the build. **The code is right; the record and the docs were not.** Only one item touched
      code, and only as a test split.
      ⚡ Item 1 confirmed and **widened**: not one exit-code change but **three** — `-o -` with no
      `--format`, `-o` at an extensionless path, and `-o` at an unrecognised extension, all
      `4` → `0`. Ruled a **conformance fix, not a contract break**, on evidence: `resize` and
      `thumbnail` already did all three on `main`, and the old `4` sat outside
      `api-contract.md`'s own enumeration of that code. Now documented.
      ⚡ A **fourth** unnamed consequence found: at single input a literal-extension
      `--name-template` used to be ignored, so `{stem}_w.jpg` wrote **PNG bytes into a `.jpg`
      file**. Fixed outright; the converse now matches `resize`/`build`. Nothing regressed.
      ⚡ Verify extended AC-6 from 16 files / 4 verbs to **39 files / 9 paths**, including the
      flagship `apply --recipe web` — all identical, positive control still diverging.
      **Items 1–6 applied by the orchestrator on the branch** (`77f1050`); **item 7 filed on
      `main`**. `cycle:` **HELD at verify** for re-approval, not advanced.

- [x] **re-approve** — prompt: `prompts/SPEC-126-re-approve.md` (2026-08-23). **Opus**, new
      session, read-only, own worktree. Scope is **`git diff f8deb55..77f1050`** and nothing else —
      4 files, +157/−42. Ready to dispatch.
      ⚠ **This cycle exists because the punch list was self-graded**: the orchestrator received
      verify's 7 items, applied all 7, and wrote the record describing what it had done. The
      posture is adversarial toward those records, not toward the fix — verify already approved
      the fix at `f8deb55`.
      Three things carry the most risk: prose added to `docs/api-contract.md`, which is the only
      part that reaches users and the only part no test covers; a test split, which is the shape
      that silently drops an assertion (a 940→941 count proves nothing about assertions); and the
      control behind that split, re-driven rather than read.
      📌 Known and stated in the prompt, not hidden: `just backlog --all` returns **zero** for the
      STAGE-047 filing, because PROJ-013 is `proposed` — the item is in the right topical home but
      invisible to tooling, which is the same class of problem it describes.

      ⚠ **2026-09-03 — PUNCH LIST, 3 items, all text, no code.** $15.02 / 19.1M tokens (Opus).
      The fix and the test split came through **clean** — the reviewer did a normalised multiset
      diff of the split rather than trusting the 940→941 count, found **zero assertions dropped**
      (8 before, 4+4 after), and re-drove the control in **both** directions, not just the one
      claimed. The Call-1 revert passed the first `--format png` block by coincidence and failed
      on the jpeg block — which is exactly what AC-1's two-format requirement exists for, now
      demonstrated rather than asserted.
      ⚡ **The gate caught an error in the orchestrator's own filing.** STAGE-047 claimed **two**
      live decisions lose globs to the inline-array parser. **Only DEC-015 does.** DEC-043's
      scope is `affected_scope: []` — an EMPTY array, which yields zero because it has no globs,
      and AGENTS §15 explicitly sanctions that form. The tell was visible in the filing itself:
      it quoted DEC-015's scope contents but quoted `superseded_by:` for DEC-043, because only
      DEC-043's *liveness* had been checked and its scope never had. **This is exactly what the
      cycle was created to catch**, and it is the argument for keeping it.
      ⚡ Also caught: a **false universal** in `docs/api-contract.md` ("every other pixel-lane
      verb" — `web`/`optimize` measurably do not preserve, `ftypavif` on a photo source, and the
      winning format is **content-dependent**), and a file list stale **by its own stated
      derivation** (six shown, seven returned; the missing one was the user-facing file).
      **All 3 applied by the orchestrator** — items 1–2 on the branch (`08b37ac`), item 3 on
      `main`. `cycle:` still **HELD at verify**.

- [x] **ship** — merge #187, then ship. ⛔ **NO TAG.** Batches with STAGE-050 as PROJ-011's one
      lockfile migration. Running total across build + verify + re-approve: **$68.82**.
      ✅ **2026-09-03 — MERGED and SHIPPED.** PR #187 squashed to `main` as **`dd60ef5`**.
      The branch needed `gh pr update-branch` first (it had fallen behind `main`) — **not
      `--admin`**, which would have bypassed the protection this repo relies on; CI re-ran and
      was re-confirmed green at the true head before the merge. `cost.totals` = 147,088,502
      tokens / **$68.82** / 3 metered sessions. ⛔ **No tag, version still 0.7.1** — batches
      with STAGE-050 as PROJ-011's single lockfile migration.
