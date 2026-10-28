# 0.8.0 — RESVG 0.49 VERIFY prompt: draft PR #211

Cycle: **verify** (dependency bump going into 0.8.0). Fresh CLI session, own worktree, **Opus**,
**read-only**. You did not write this change.

```
just wt verify/resvg-0-49 -- --model opus
```

Then put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin && git reset --hard origin/chore/resvg-0.49
```

Draft PR #211's head was `5dea0c2` when this prompt was written. Record the SHA you verify. The
maintainer pasted this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own.

## Why this verify exists
- The maintainer opened #211 to go into **0.8.0**, which is not tagged yet. If it merges, the tag
  moves from the release commit `363f22f` to #211's merge commit. It is the last change before the
  tag.
- It bumps `resvg` and `usvg` from 0.47 to 0.49 (native and wasm dependency lists), and drops
  `ttf-parser` and `rustybuzz`. It also removes their two advisory ignores (RUSTSEC-2026-0192,
  RUSTSEC-2026-0206) from `deny.toml`, and adds a CHANGELOG line under `## [0.8.0]` → `### Changed`.
- CI is green on every leg. **CI cannot see three things**, and they are your job:
  1. **Rendering.** No test compares SVG output pixel for pixel. resvg 0.48 moved text layout to
     `skrifa` + `harfrust`, and its own changelog warns that text may shift.
  2. **The CHANGELOG entry's claims.** It lists resvg fixes (filters in rotated or skewed groups,
     `text-anchor` and nested `<tspan>` baselines, gradients inherited through `href`, a panic when
     a filter region is smaller than its input) and size numbers (+~440 KB native, +~21 KB wasm
     brotli). These were copied from upstream and from the author's measurements. Nobody has run
     them against our binary. A citation looks like prose, but it is a claim.
  3. **The hostile-input guards in `src/image/svg.rs`.** `hardened_options()` sets
     `resources_dir: None`, an `image_href_resolver` that refuses every file and URL href, and an
     explicit `fontdb` holding only the bundled Go font. `check_caps` caps dimensions before the
     raster is allocated. The PR does not touch `src/`, so if 0.49 changed a default or added a new
     way to load an external resource, the code would still compile and the guard would quietly
     stop covering it.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit anything outside your scratch directory. Do not touch any tag.

## Check, in order
Build two release binaries in **separate, fresh** `CARGO_TARGET_DIR`s:
- **BASE**: `origin/main`'s code. Use `git worktree add` on a scratch path, or
  `git archive origin/main | tar -x -C <scratch>`. Do not use your own worktree.
- **HEAD**: the PR head.

Also use the released **0.7.1** from Homebrew (`crustyimg --version` must say 0.7.1) as a third
reference. Check each binary's `--version` and path before you record anything from it. Make every
input by hand (written SVG text, ImageMagick or Python), never with crustyimg.

1. **Scope.**
   - The PR touches 4 files: `CHANGELOG.md`, `Cargo.toml`, `Cargo.lock`, `deny.toml`. Nothing under
     `src/` or `tests/`.
   - In `Cargo.toml`, only the two `resvg` lines and a comment change.
   - **The lockfile.** List every package whose version was added, removed or changed. The
     orchestrator expects exactly this, and anything else is a finding:
     - **up:** `resvg` and `usvg` 0.47 → 0.49, `fontdb` 0.23 → 0.24, one 0.14 → 0.15 (name it);
     - **added:** `harfrust 0.12.0`, `base64 0.23.1`;
     - **removed:** `ttf-parser`, `rustybuzz`, `core_maths`, `unicode-bidi-mirroring`,
       `unicode-ccc`, `unicode-properties`.
   - Every commit is signed off.
2. **Advisories.**
   - `cargo deny check` passes on HEAD. Read `$?`.
   - `ttf-parser` and `rustybuzz` are gone from every build, not only the default:
     `cargo tree -i <crate> --all-features`, and again with `--target wasm32-unknown-unknown`. Both
     must report the package as not found.
   - **Negative control:** on BASE, delete the two ignore entries from a scratch copy of
     `deny.toml` and run `cargo deny check advisories`. It must fail and name both advisories. That
     proves the ignores were doing work on BASE, so "gone on HEAD" means something.
   - `base64` now appears twice, and `deny.toml` has `multiple-versions = "warn"`. Confirm it warns
     and does not fail, and say which feature sets pull in each copy.
3. **Hostile-input guards still hold on HEAD.**
   - Read the usvg/resvg 0.48 and 0.49 changelogs (in `~/.cargo/registry/src/*/usvg-0.49*/` and
     `resvg-0.49*/`, or the crates' repositories). List every change to `Options`,
     `ImageHrefResolver`, font loading, or anything that can fetch a resource. For each, say whether
     `hardened_options()` still covers it.
   - Drive these on BASE and HEAD and record the exit code and output for each. The answers must
     match:
     - `<image href="file:///etc/passwd">`;
     - an `http://` image href;
     - a CSS `@import` or `url(...)` in a `<style>` that points at a file;
     - `<text font-family="Helvetica">`, which must render with the Go font and must not read a
       system font. Compare the pixels with `font-family` removed;
     - a huge `width`/`height`, which must be refused before allocation.
   - Run `cargo test --test input_svg --test fuzz_regressions` on HEAD.
4. **Rendering: what changed, measured.**
   - Write a corpus of at least 10 SVGs into scratch. Include:
     - plain shapes, gradients and paths with no text;
     - text with `text-anchor` start, middle and end;
     - nested `<tspan>` with `dy` and `baseline-shift`;
     - rotated and multi-line text;
     - a filter inside a rotated group and inside a skewed group;
     - a gradient inherited through `href`;
     - a filter region smaller than its input;
     - `tests/fixtures/svg/rect_text_40x30.svg`.
   - Convert each to PNG with all three binaries (`crustyimg convert in.svg -o out.png`, or the verb
     the docs name). Compare with `magick compare -metric AE`, and also with RMSE.
   - Report a table: file, BASE vs HEAD differing pixels, 0.7.1 vs HEAD differing pixels, exit
     codes.
   - **No-text SVGs should be byte- or pixel-identical between BASE and HEAD.** If one is not, that
     is the main finding. Explain it with a diff image.
   - For text SVGs, a shift is expected. Say how big it is (pixels, and the largest glyph offset you
     can see), and keep the BASE and HEAD PNGs plus a diff image for each in scratch so the
     maintainer can look. Give their absolute paths.
5. **The CHANGELOG entry, claim by claim.**
   - For each listed resvg fix, build an input where BASE does the wrong thing and HEAD does the
     right thing. Name the input, and give both outputs or exit codes.
   - **The panic claim:** BASE must panic or exit non-zero (record which; a panic exits 101 or is
     turned into an error), and HEAD must exit 0. If you cannot make BASE fail, the claim is
     unverified for our binary. Say so, and propose wording that keeps only what you drove.
   - **The sizes:** measure the native release binary on BASE and HEAD from the clean target dirs.
     For wasm, run `just wasm-build` (it chains `wasm-size`) on both, if the toolchain is present.
     Compare with "about 440 KB" and "about 21 KB compressed". If wasm cannot be built, say so.
   - "Text may render with small differences from 0.7.1": does your step-4 table support "small"?
   - Check where the line sits: it must be under `## [0.8.0]` → `### Changed`, with no spec or DEC
     ids. Note the section date (`2026-10-27`) for the maintainer. Do not fix it.
6. **Gates that CI ran but did not report in detail.**
   - Run `gh pr checks 211` **once**. List every leg and its result at the head SHA, including
     windows-latest, msrv, lean, avif, heic, webp-lossy, cargo-deny and the wasm "build + browser
     smoke".
   - Locally, run only `just wasm-test` on HEAD (it includes `svg_text_renders_glyphs_in_wasm`).
     Redirect output to a file and read `$?`.
   - Do not re-run the full matrix.

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no `timeout(1)`.
- If a binary you drive no longer exists, the shell exits **127**. That is not a result.
- Never share a target dir between BASE and HEAD, or between native and wasm builds, and never run
  two builds at once.
- Keep scratch files in a scratch directory, never in `~`. Delete your scratch target dirs at the
  end, but **keep the PNG and diff images** for the maintainer.
- **Budget ~60 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content:
  `grep -l "0.8.0 — RESVG 0.49 VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Opus $5/$25 per
  MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #211 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #211 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then, **with the evidence inline**. Do not refer to an earlier message; the orchestrator only sees
this one.
- items 1–6, one to three lines each;
- the step-4 rendering table, and the absolute paths of the image folder;
- every CHANGELOG claim: driven ✅, false ❌, or unverified, each with your proposed wording;
- any guard that 0.49 weakened;
- your cost.
