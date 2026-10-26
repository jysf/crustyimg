# SPEC-134 — VERIFY prompt: a `--name-template` extension pins the output format

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build this.

```
just wt verify/spec-134 -- --model opus
```

Then put the worktree on the PR's code (the fresh branch has nothing on it to lose):
```
git fetch origin && git reset --hard origin/feat/spec-134-name-template-pins-format
```
PR #202 is a **draft**; its head was `635adfb` when this prompt was written. Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

**What the build claims:** a recognised extension in `--name-template` (and, for `meta`, in `-o`)
pins the output format on every encoding verb, at one input and at many, and on plain-recipe
`build` targets, as DEC-105 extends DEC-087; on `web` / `optimize` / `apply --recipe web` it skips
the automatic choice; `meta` refuses a format change with exit 4 before writing anything; `{ext}`,
extensionless and unrecognised templates are byte-identical to `main`. 22/22 ACs; a 41-row table
whose RED baseline was 28 rows; NC-1…NC-10 flipped; CI green on all 16 required legs.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no version bump. Do not fix what you find:
report it with evidence and a suggested fix. Do not edit the spec or timeline.

## Read first
The spec (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-134-name-template-extension-pins-the-output-format.md`
on the PR branch, all of it including `## Build Completion`), **DEC-105** (edited again at build
for AC-22) and **DEC-087** — read both, do not trust a summary — DEC-058/DEC-101 (cache key),
DEC-015 (exit codes), constraint `metadata-not-via-pixel-encode`, HANDOFF.md's "Traps".

## Eight specific things — in this order

### 1 — ⚡ What must NOT change, driven on `main` vs the PR (release binaries, fresh target dir each)
- `{ext}` templates (`{stem}_w.{ext}`), extensionless templates and an unrecognised extension
  (`{stem}.bin`): output bytes identical on both binaries, across at least `resize`, `apply` (1 and
  N), `web` and `meta strip`.
- Plain `build` with `name = "{stem}"` and `name = "{stem}.txt"`: exit 0 on both, same bytes.
- **Existing build caches survive the upgrade** for every target the change does not pin: build a
  `{stem}_x.{ext}` plain-recipe project with the `main` binary, then with the PR binary: all cached,
  0 rebuilt. A pinned target (`{stem}.png` on a JPEG source) must rebuild (its key changed by design).

### 2 — The pin is real, checked with a sniffer you did not write
Pick at least eight table rows across different verbs and arities and drive them yourself on the
PR binary; identify every output's format with `file` (or the magic bytes), never the name or the
code under test. Include `web --out-dir d/ --name-template '{stem}.webp'` (WebP, not AVIF) and a
`thumbnail` N-input row.

### 3 — The behaviour changes users will notice
The build lists: `--json` with a template pin on `web` / `optimize` / `apply --recipe web` → exit
2 (as `-o` already is); `optimize --profile preserve` now pins on a literal template; a
template-pinned `web` prints no summary line. Drive each, and confirm each is named in the
`CHANGELOG.md` entry. Report any other changed behaviour you find that is **not** listed.

### 4 — `meta` refusals (rulings 3 and 4, AC-12/13/22)
Drive: template naming another format (1 input) → exit 4, nothing written, message says convert
first; the **mixed batch** (`a.png` listed first, then `in.jpg`, T = `{stem}.png`) → exit 4 and
`a.png` **not** written; `meta strip in.jpg -o E.png` → exit 4, nothing written; same-format `-o`
and template → exit 0, unchanged bytes. The build notes that multi-input `meta` reads the
template even when `-o` is given: judge whether that is right, and whether it is documented.

### 5 — RED baseline and the two readings of NC-7
Copy the PR's test files onto `main` (unmodified `src/`) and run them: 28 of the 41 rows must fail
for the reasons the spec gives and the 13 guards pass; the AC-22, `build_plain_*` and cache tests
fail as stated. Then NC-7: the build found it has two readings (check moved inside the per-input
error capture vs in the loop outside it). Run both shapes and say whether a test catches each.

### 6 — Your own mutations, beyond NC-1…NC-10
At least three, e.g.: let `template_literal_format` return a pin even when `{ext}` is present;
make the plain-target `build` plan strict again (exit 4 on `{stem}.txt`); swap the precedence so
the template beats `--format`; make `meta` check only the first input. Name any mutation no test
catches. ⚠ Reverting source does not rebuild the binary; use a fresh target dir per mutation (a
reused one silently compiled stale code in SPEC-131's verify).

### 7 — Boundaries and records
- `git diff --name-only origin/main...HEAD` = the 15 files listed in Build Completion.
  `src/build/cache.rs` untouched, `CACHE_SCHEMA_VERSION` unchanged; in `src/cli/build.rs` the
  functions `watch_impl`, `register_roots`, `target_recipe_hash`, `build_one` and `encode_one`
  (common.rs) are untouched (SPEC-132 and SPEC-133 own them). List every function the diff touches.
- DEC-105: block-list `affected_scope`; `supersedes: null`; it extends DEC-087 and says the
  contrary sentence in `docs/api-contract.md` was undocumented by any DEC; the AC-22 `-o` refusal
  is described. DEC-087 carries the "Extended by DEC-105" line.
- `grep -n 'does \*\*not\*\* pin' docs/api-contract.md` → 0 hits; no SPEC/DEC id in any comment or
  user-facing string the diff adds.
- `./scripts/decisions-audit.sh --changed origin/main` (pass the base ref).

### 8 — Docs have no green
Drive the claims the diff adds to `docs/api-contract.md`, `docs/cli-reference.md`,
`docs/data-model.md` and `CHANGELOG.md` (precedence order, recognised extensions, the exit codes).
List any that is false. The build moved stale line references; check it changed the passages the
spec meant.

## Also
- **CI:** `gh pr checks 202` at the head you verified; every leg, including windows-latest, lean,
  MSRV, the feature legs and browser smoke. Do not poll; if pending, background
  `gh pr checks 202 --watch` and read a direct snapshot when it exits.
- **Matrix once, yourself:** default, `--no-default-features`, `--features webp-lossy`, each in a
  fresh `CARGO_TARGET_DIR`, **sequential**; tests + `clippy --all-targets -D warnings`;
  `fmt --check`.

## Guardrails
- **Read-only. No commits. Do not fix what you find.** Scratch directories for every driven
  project; remove any scratch worktree you add.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the pipe's exit code. zsh does not word-split unquoted
  parameters: pass feature flags as separate literal arguments. macOS has no `timeout(1)`.
- **Budget ~120 exchanges.** If past it, stop and report what you covered.

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it **by content,
not recency**: `grep -l "SPEC-134 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup
by `.message.id`**: one JSONL line per *content block*; lines sharing an id repeat identical
`input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens` while `output_tokens`
grows. Take those three once per id and the **MAX** `output_tokens`. Summing every line
over-counts ~2×; taking the first line's output under-counts ~11×. Price each component at the
model that actually ran (Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 the input
rate, cache_read ×0.10), never flat. Report `tokens_total`, a `tokens_breakdown`, `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #202 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #202 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`
Then lead with **item 1** (what must not change: identical yes/no per case; caches survive the
upgrade yes/no), then items 2–8, short. A punch list is numbered, each with evidence and a
suggested fix. End with your cost readout.
