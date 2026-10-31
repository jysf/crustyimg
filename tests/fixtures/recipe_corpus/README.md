# Recipe corpus

Every recipe here is pinned twice, by two files that answer different questions.

| file | pins | checked by |
|---|---|---|
| `<stem>.toml` | the recipe as written | `tests/recipe_round_trip.rs` parses it |
| `<stem>.golden` | the recipe's canonical TOML (`to_toml()`), which is an input to the cache key | `tests/recipe_round_trip.rs` |
| `<stem>.<target>.output` | what the recipe produces on three fixed fixtures | `tests/recipe_corpus_output.rs` |

The three bundled recipes in `recipes/` are pinned the same way, under the names
`bundled_gallery`, `bundled_product` and `bundled_web`.

## What an `.output` file holds

One line per fixture and kind: `<fixture> <kind> <fields>`. Lines starting with `#` are comments.

- `pixels`: the image after the last pixel step, before encoding. Width and height, colour type,
  bit depth, and the SHA-256 of the raw samples. Every recipe has three of these.
- `png` and `webp`: the SHA-256 of the encoded file, for the two recipes that end in PNG or
  lossless WebP. These encoders are integer-only and pure Rust. AVIF and lossy encodings are not
  pinned, because an encoder promises the same bytes on one machine, not on every machine.
- `optimize`: the format the `optimize` step chose, for the bundled recipes only.

The fixtures are built inside the test from seeded raw samples, so no decoder or encoder produces
an input. Each is larger than every size bound in the corpus, so every resize actually resizes.

Resize and compositing use float maths whose last bits differ by OS and CPU, so the pin is one
file per target (`<stem>.<os>-<arch>.output`: linux-x86_64, macos-aarch64, windows-x86_64), each exact. It is never loosened to a tolerance. A target with no file fails the test; bless on that machine and add it to `TARGETS`. Goldens for a CI target you cannot run locally come from the failing log: each `----- BEGIN … -----` block is a file.

## Running and blessing

    cargo test --test recipe_corpus_output

A failure names the recipe, the fixture and the first field that moved, for example
`v1_resize_max × rgb8_2600x1730: pixels changed (dims 800x532, colour Rgb8, depth 8 unchanged)`,
then prints the full new content of every file between `----- BEGIN … -----` and `----- END -----`
markers.

To rewrite the goldens when an output is meant to change:

    CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless

The bless refuses to run when `CI` is set, and when the build lacks the default codec set (it
needs to run the bundled `optimize` choice). Without `CRUSTYIMG_BLESS` it writes nothing.

## Rules

- **A changed line in an existing `.output` file is a regression until proven otherwise.** A PR
  that changes one needs its acceptance criteria to name the recipe and say why its output
  changes. Review with
  `git diff --name-only origin/main -- 'tests/fixtures/recipe_corpus/*.output'`.
- A new `.output` file, or new lines for a new recipe, is not a re-bless.
- The counts in `recipe_corpus_output.rs` are literals, not derived from the recipe list, so
  a recipe dropped from the list fails the test.

## Adding a recipe

Add the recipe's `<stem>.toml` and `<stem>.golden` here (or a bundled recipe under `recipes/`),
then raise every literal in the same diff. For a new recipe with no byte row, that is:

| where | literal | change |
|---|---|---|
| `recipe_corpus_output.rs` | `RECIPES` | add the stem |
| `recipe_corpus_output.rs` | `EXPECTED_DIMS` | one entry per recipe: its output size on each of the three fixtures |
| `recipe_corpus_output.rs` | `[(&str, [&str; 3]); 14]`, the array length of `EXPECTED_DIMS` | 14 → 15 |
| `recipe_corpus_output.rs` | `ROWS` (48) | +3 per recipe, +3 per new `png` or `webp` byte row |
| `recipe_corpus_output.rs` | `GOLDEN_LINES` (51) | +3 per recipe, +3 per new byte row, +1 per new bundled recipe |
| `recipe_corpus_output.rs` | the `42` in `EXPECTED_DIMS.len() * FIXTURES.len()` | +3 per recipe |
| `recipe_corpus_output.rs` | the module and `ROWS` doc comments ("all 14", "14 recipes x 3 fixtures …") and the `GOLDEN_LINES` doc comment ("42 pixel rows, 3 PNG, 3 WebP and 3 chosen-format lines") | to match |
| `recipe_round_trip.rs` | `CORPUS`, and the `14` in `pairs.len()` | add the stem; +1 for a corpus recipe |

A recipe that adds a byte row or a bundled recipe also needs the code for it: `PNG_RECIPE` and
`WEBP_RECIPE` name the only two byte rows, `BUNDLED` and the `rows_checked == 3` in the
`optimize` test name the three bundled recipes, and the `png`/`webp`/`optimize` line counts in
`check_target_goldens` follow them. Find any other literal with
`grep -nE '[0-9]+' tests/recipe_corpus_output.rs`: a number that counts recipes, fixtures or
rows is one of these.

Then bless the three targets. Bless your own target locally:

    CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless

Push. The CI run fails on the targets you do not have, because their `<stem>.<os>-<arch>.output`
files are missing, and prints each one's file between `----- BEGIN … -----` and `----- END -----`
markers. A missing file is reported by name and does not stop the dumps. Copy each block from
`gh run view <run> --log-failed` into the file named on its `BEGIN` line, and push again. The
second run must be green on every leg. The new files, and the new lines for a new recipe, are
not a re-bless.

## One target goes red with no code change

The goldens pin the maths of each runner image (its `powf`, from its libm), not only its OS and
architecture. If one target goes red and nothing in the code changed, for example after a
runner-image update, that is libm drift. Re-bless that target only, from its CI log, in a commit
of its own whose message says why. Never fold it into a feature PR.
