# SPEC-134 — FIX prompt: the verify punch list, on the same draft PR #202

Cycle: **build** (punch-list follow-up). Fresh CLI session, **Sonnet**, on the PR's own branch:

```
just wt feat/spec-134-name-template-pins-format -- --model sonnet
```

(That reuses the build's existing worktree if it is still there; close any old session in it
first.) You did not build or verify SPEC-134. Its verify at `635adfb` found the code sound on
every check and returned a punch list: three doc fixes, plus one code fix the maintainer ruled
into this spec. Fix exactly these on branch `feat/spec-134-name-template-pins-format`, pushing to
the **same draft PR #202**. The maintainer pasted this prompt into a fresh session and will paste
your final message back to the orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git status` (must be clean), then `git pull --ff-only`: the old build
worktree may be behind its own PR branch, which already has a merge of `main` (`33912eb`).
`git log -1` (expect `33912eb` or later). Then `git merge origin/main` (not rebase: the PR is shared) and resolve any
`CHANGELOG.md` / `docs/api-contract.md` conflict by keeping every other spec's lines. Read the spec
(`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-134-name-template-extension-pins-the-output-format.md`,
especially `## Build Completion`) and DEC-105.

## 1 — CHANGELOG understates which outputs change bytes
It says bytes change "for any such template whose extension differs". Verify measured otherwise:
`web in.jpg --out-dir d --name-template '{stem}.avif'` gives 2069 B with a summary line on `main`
and 1576 B with no line on the PR, because **any** literal template on `web` / `optimize` /
`apply --recipe web` now takes the pinned encode, even when the extension matches the automatic
choice. DEC-105's Consequences already says this. Fix the CHANGELOG entry to say so.

## 2 — The new exit 2's trigger is missing from user docs
`docs/cli-reference.md` (~:134 and ~:442, "With a pinned -o/--format … exit 2") and the `web` help
text (`src/cli/mod.rs` ~:315) name only `-o`/`--format`. Add "or a `--name-template` extension" in
all three places.

## 3 — One `docs/api-contract.md` sentence is false at N inputs
It says "With `-o` given … the template is not the output's name and does not pin." At N inputs
`-o` is ignored and the template **is** the name (verify: `resize --max 64 in.jpg in2.jpg -o x.jpg
--out-dir d --name-template '{stem}.png'` → PNG; `meta strip` with the same arguments → exit 4).
Scope the sentence to one input and add the N-input rule. Do this **after** item 4, so the text
describes the fixed behaviour.

## 4 — Code (maintainer ruling 2026-10-26): at N inputs the format check must read the template
With several inputs `-o` is ignored and the write uses the template, but the pin check on `web` /
`apply --recipe web` still reads `-o`, so the file is mislabelled. Measured by verify (same on
`main` today):
- `apply --recipe web in.jpg in2.jpg -o x.jpg --out-dir d --name-template '{stem}.png'` → **JPEG
  bytes in `in.png`**
- `web` and `apply --recipe web` with `-o x.bin` (same other args) → **AVIF in `in.png`**, with a
  `jpeg → avif` line.
**Fix:** make `name_pinned_format` (`src/cli/common.rs`) read the template whenever there is more
than one input, as `meta`'s pre-flight already does; one input keeps today's `-o` > template order.
**Tests first:** add these rows (and a `web` N-input `-o x.jpg` row) to the table in
`tests/name_template_pins_format.rs` or a named sibling test, asserting the sniffed format of each
output matches its file name. Run on the current branch before the fix and confirm each is **RED**
(wrong format inside `in.png`); then fix; then GREEN. Update the table's row-count assertion if you
add rows there, and the spec's roster note in `## Build Completion`.
**Negative control:** revert only the N-input branch in `name_pinned_format`, rebuild in a fresh
target dir, confirm exactly the new rows go RED, restore.
Add a one-line **Fixed** CHANGELOG entry for it (it is a user-visible mislabel on 0.7.x too), and a
sentence in DEC-105 (block-list `affected_scope` unchanged unless you touch a new file).

## Boundaries
Only `CHANGELOG.md`, `docs/cli-reference.md`, `docs/api-contract.md`, `src/cli/mod.rs` (help text),
`src/cli/common.rs` (`name_pinned_format` only), `tests/name_template_pins_format.rs`, DEC-105, and
the spec's `## Build Completion` / `cost.sessions` on this branch. Do not edit the spec's timeline
(it lives on `main`). `watch_impl`, `register_roots`, `target_recipe_hash`, `build_one`,
`encode_one` stay untouched (SPEC-132 / SPEC-133 own them).

## Finish
- Drive the four item-4 commands on a release binary and identify each output with `file`.
- Clean matrix: default, `--no-default-features`, `--features webp-lossy`, each in a **fresh**
  `CARGO_TARGET_DIR`, **sequential** (feature flags as separate literal arguments; zsh does not
  word-split); tests + `clippy --all-targets -D warnings`; `fmt --check`.
- Every commit `git commit -s` (DCO). Push the branch. ⛔ The PR stays a **draft**.
- **Never poll CI.** Background `gh pr checks 202 --watch`; when it exits, read a direct snapshot
  at the true head SHA, every leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the pipe's exit code. macOS has no `timeout(1)`.
- **Budget ~70 exchanges.**

## Measure your own cost
Transcript: a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`; find it by content:
`grep -l "SPEC-134 — FIX prompt" ~/.claude/projects/<slug>/*.jsonl`. Dedup by `.message.id`
(input / cache_creation / cache_read once per id, MAX output_tokens), price per component (Sonnet
$3/$15 per MTok, Opus $5/$25; cache_creation ×1.25, cache_read ×0.10), never flat.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-134 fix: draft PR #202 at <head SHA> — CI <green | red: legs>`. Then: items 1–4
(done / not done); item 4's rows RED before / GREEN after and the negative-control flip; the four
commands' `file` output; files changed in this fix; your cost.
