# Recipe corpus

Every recipe here is pinned twice, by two files that answer different questions.

| file | pins | checked by |
|---|---|---|
| `<stem>.toml` | the recipe as written | `tests/recipe_round_trip.rs` parses it |
| `<stem>.golden` | the recipe's canonical TOML (`to_toml()`), which is an input to the cache key | `tests/recipe_round_trip.rs` |
| `<stem>.output` | what the recipe produces on three fixed fixtures | `tests/recipe_corpus_output.rs` |

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

Each `.output` file is exact on every CI target. If the targets ever disagree, the pin becomes one
file per target (`<stem>.<os>-<arch>.output`), still exact. It is never loosened to a tolerance.

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
- The row counts in `recipe_corpus_output.rs` are literals, not derived from the recipe list, so
  a recipe dropped from the list fails the test. Adding a recipe means adding its stem, its
  expected dimensions and its `.output` file, and raising the literals in the same diff: 3 rows
  per recipe, plus one per new byte row, and the golden-line total.
