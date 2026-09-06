# SPEC-128 — VERIFY prompt

Cycle: **verify**. New session, **read-only**. You did not build this.

**What it claims:** the registry seam now constructs operations whose params name a **file**,
resolved at the IO boundary; `watermark` is registered in both modes; `src/operation/**` stays
filesystem-free; and nothing else changes bytes.

## ⚠ The PR is OPEN and must NOT merge

**PR #189, branch `feat/spec-128-recipe-watermark`, head `bdbb74a`. Review the BRANCH.**
Base is `main`.

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec128-verify feat/spec-128-recipe-watermark
```

⛔ **Do not merge, do not bump the version, do not cut a release.** Batches with the rest of
STAGE-050 as PROJ-011's single lockfile migration. **Make no commits** — verdict and
`## Cost readout` go in the return message (AGENTS §13).

## Already settled — do NOT re-derive

Checked by the orchestrator against the branch:

1. **PR #189 OPEN, MERGEABLE, head `bdbb74a`, CI 16 SUCCESS + 6 release-only skips**, nothing
   failing or pending. Snapshot taken at the true head. **Never poll CI.**
2. **19 files changed**, including `docs/api-contract.md` **and** `docs/data-model.md`.
3. **DEC-100 uses the block-list `affected_scope`** — 11 paths, including the wasm size gate. The
   audit can read it.
4. **AC-8's premise holds:** the only `std::fs` match under `src/operation/` is a doc comment
   *forbidding* it. No real filesystem dependency was added.

---

## Six specific things

### 1 — ⚡ A SIZE GATE WAS MOVED. This is the item.

`scripts/lib/wasm-artifact.mjs`: `WASM_BROTLI_BASELINE` **1_144_921 → 1_266_535**, **+10.6 %**.
The reason given is that registering `watermark` in `with_builtins()` — the one constructor set
both native and wasm share — links `crate::text`, `skrifa`, `zeno` and the bundled Go-Regular.ttf
into the `.wasm` for the first time, so a text watermark with no `font` key can run on wasm with
no asset resolution.

⚠ **This repo's own lesson is "a guard that gets relaxed whenever it fires stops being a guard."**
Moving a baseline because your change exceeded it is the canonical bad pattern. **The build may
well be right — but it is the reviewer's job to rule, not to accept.** Three questions, in order:

- **Was the capability required at all?** The spec's Call 3 says `wasm::transform` must **refuse**
  a recipe whose steps need assets. A text watermark using the *bundled default* font needs no
  asset, so it falls outside the refusal and works — but **no AC ever said text watermarks must
  run on wasm.** That capability, and its 121_614 B, is an inference from Call 3's wording rather
  than something the spec asked for. Was there a cheaper option — e.g. `#[cfg]`-gating the
  registration so wasm keeps the lean bundle — and if so, is the trade recorded as a decision or
  taken silently?
- **Does the guard still guard?** The build says yes and shows its work: a lean build measures
  865_980 B against the new floor of 1_087_675 B, 20.4 % below, so a missing AVIF encoder still
  trips it. ⚠ **Re-derive that number yourself.** A floor that no longer catches the thing it
  exists to catch is the actual failure mode here.
- **Is +10.6 % on a web-delivered bundle a user-visible cost that needed a decision?** The demo
  ships this artifact. DEC-100 should either own that trade or say why it is not one.

### 2 — The cost figure cannot be re-derived, and the key name drifted

`estimated_usd: 42.29` / `tokens_total: 123802069`, and the note describes the **correct** method
(dedup by `.message.id`, static fields from the group, **MAX** output) — which is the method this
project only established last cycle, so that is a genuine improvement.

**But the entry carries no `tokens_breakdown`**, so nobody can check the per-component pricing —
and per-component is exactly where SPEC-127's 11.7× error lived. It also writes **`notes:`** where
every sibling entry uses **`note:`**. **Recompute the figure from the transcript yourself** and say
whether $42.29 stands; flag the key drift.

### 3 — AC-9's negative controls: one revert per independent condition

Calls 1, 2 and 3 were declared independent. **Drive all three reverts, one at a time**, and confirm
each flips **only** its own tests. The evidence is the **behavioural flip**, never a hash — a debug
rebuild from identical source already differs.

### 4 — ⚡ Call 3b: text mode's keys, and a test that must assert PIXELS

Before this spec, `params()` wrote the *text* under the key **`image`** — the field meaning file
path — so a text watermark would have round-tripped as an attempt to load a file named after the
text. Call 3b requires distinct, non-overlapping keys: `text`/`font`/`size`/`color`, and **never
`image`**.

⚠ **AC-2 must assert the round-tripped step renders the same PIXELS**, not merely that the TOML
parses — a parseability-only test passes on the *old, broken* behaviour. Check the test actually
does that, and that AC-2b's both-or-neither case is a typed error at build-pipeline time.

### 5 — AC-7 is the seam's real test, and the easiest to fake

A *fixture* asset-bearing op must register through the same resolve path **without the seam
changing**. Check the fixture genuinely exercises the resolve step (not a shortcut that bypasses
it), and that the assertion about the seam being untouched is real rather than decorative. This is
the AC that determines whether the `.cube` LUT op can land later without reopening this work.

### 6 — AC-10's sweep, and whether its boundary is stated

39/39 byte-identical is claimed. ⚠ **Confirm the sweep followed the CALL GRAPH, not the file** —
`run_convert`, `run_optimize` and `run_web` all reach `run_pixel_op`, which is what the
orchestrator got wrong on SPEC-127. Confirm the positive control is real (a case known to differ,
shown differing), and that DEC-100 **states the corpus boundary** rather than implying a whole-
surface sweep.

---

## Also check

- **Two CI-caught misses the local matrix did not catch:** DCO sign-off, and the bundle-size
  regression. ⚠ **That is a gap in the local gate, not just two fixes** — `just check` never runs
  the wasm size gate. Worth a filed item if it is not already one.
- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref**, or a
  clean checkout exits 0 on a green that cannot go red. ⚠ It **cannot see DEC-015** (inline
  `affected_scope`, filed PROJ-013 STAGE-047).
- **DEC-031 is the premise this spec removes.** Confirm DEC-100 either supersedes it explicitly or
  states precisely which part still holds — a decision silently contradicted is worse than one
  reopened.
- **Every file in Build Completion**, from `git diff --name-only`, not recall — 19.
- **AC-11's matrix**: default, `--no-default-features`, `--features webp-lossy`, fresh
  `CARGO_TARGET_DIR` each, sequential; clippy + `fmt --check`; plus `just wasm-check`.
- **`docs/api-contract.md` / `docs/data-model.md`**: every added sentence is a testable claim about
  a shipped binary. Drive the ones that are. Documentation has no green.

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not merge or bump the version.
- **Budget ~200 exchanges.**
- `cargo test` fails `display_sink_refuses_non_tty` interactively — redirect stdout, do not "fix"
  it. A piped command reports the **pipe's** exit code — redirect and read `$?`. zsh does not
  word-split unquoted parameters. Use `/usr/bin/grep`. macOS has no `timeout(1)`.
- ⚠ **Do not run a backgrounded feature build while editing source for a revert** — SPEC-127's
  build contaminated a leg that way.

## When you finish

**Measure your own cost from your transcript** at `~/.claude/projects/<cwd-slug>/<session-id>.jsonl`
(session id = last path component of your scratchpad dir). ⚠ **Dedup by `.message.id`**: one JSONL
line per *content block*; lines sharing an id repeat identical `input`/`cache_creation`/
`cache_read` while `output_tokens` grows. Take those three from the group and **MAX** output.
Summing every line over-counts ~2×; taking the first line's output under-counts ~11×. **Emit a
`tokens_breakdown`** — the entry you are reviewing lacks one, and that is why its figure cannot be
checked. Price per component at Opus anchors ($5/$25 per MTok, cache_creation ×1.25, cache_read
×0.10), never flat.

End with **✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED**, leading with **item 1** — whether moving the
wasm size gate was justified, cheaper alternatives were considered, and the floor still guards.
