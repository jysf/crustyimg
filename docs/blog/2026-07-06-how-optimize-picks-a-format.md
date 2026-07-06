# How `optimize` picks a format: shortlist, search, winner

*2026-07-06 · feature deep-dive · the optimization engine*

crustyimg 0.3.0's headline is short: run `optimize`, and it chooses the
output format for you. This post is the long version — exactly *how* it
decides, what it guarantees, and how to steer it — because an auto-decider
is only useful if you can trust it, and you can only trust it if you know
what it does.

The whole thing is four steps: **look, shortlist, measure, pick.**

## 1. Look

First `optimize` reads the image once and forms a verdict: what *kind* of
image is this (photograph, graphic, icon, document, screenshot), and does
it have transparency? That's the analysis layer doing its job — a cheap,
deterministic, no-ML read of the pixels. (There's a
[separate post](2026-07-06-deciding-photo-vs-graphic-without-ml.md) on how
that verdict is made.) Everything downstream keys off two things from it:
the codec family the image wants (lossy vs. lossless), and whether alpha
has to be preserved.

## 2. Shortlist

Based on that verdict, `optimize` builds an **ordered shortlist of at most
three candidate formats** — the formats worth actually trying for *this*
image:

- A photo with no transparency → `[WebP (lossy), JPEG]`.
- A photo *with* transparency → `[WebP (lossy), PNG]` — JPEG is out, it has
  no alpha.
- A few-color graphic → `[WebP (lossless), PNG]`.
- A screenshot → try both families and let the bytes decide.

Two rules keep the shortlist honest:

- **Only formats that are actually built in appear.** If your binary
  wasn't compiled with lossy-WebP, WebP-lossy is never proposed — no
  runtime surprises.
- **AVIF is special.** It can *encode* but crustyimg ships no AVIF
  *decoder*, so it can't be perceptually scored. It therefore appears only
  in `--max-size` (byte-budget) mode, where the target is bytes, not a
  quality score — and only when built with `--features avif`.

Three candidates is the cap. It's the one knob that bounds how much work a
decision can cost, and it's plenty: the right answer is almost always in
the top two or three for a given image type.

## 3. Measure

Here's the part that keeps `optimize` cheap and honest: it invents **no new
compression logic**. Each candidate is solved with the *same* quality
search crustyimg already used for single-format optimization — the one that
binary-searches encoder quality to hit a perceptual (SSIMULACRA2) target,
or fits a byte budget. Lossless candidates are a single encode; lossy ones
run the capped search.

And crucially, the encoder that *measures* a candidate is the exact same
one that would *write* it. So the byte count `optimize` compares isn't an
estimate — it's the size of the file you'd actually get. The winner's
reported size equals the shipped file's size, by construction.

## 4. Pick

Now `optimize` has, for each candidate, a real byte size and whether it met
the quality target. The winner is:

> the **smallest** candidate that **met the target** *and* **beats your
> original**.

That second clause is the guarantee that makes it safe to leave on: if no
candidate actually comes out smaller than the file you started with,
`optimize` ships **nothing new** — it leaves your original untouched. It
will never hand you a *bigger* file in the name of optimization.

### The clear-win guard

One more rule, and it's the one people appreciate once they hit it:
`optimize` won't change your file's *format* for a trivial gain. If the
best different-format candidate is only, say, 2% smaller than the best
*same-format* option, it keeps your format. A format switch has to clear a
real threshold to be worth the churn of a new extension. So you don't get
`logo.png` silently becoming `logo.webp` to save 40 bytes — but you *do*
get the switch when it saves 40%.

Every tie is broken deterministically (smaller bytes, then shortlist
order), so the same image and flags always produce the same output — on
your laptop and in CI, today and next month.

## Steering it

The defaults are meant to be right, but you're never locked out:

- **`--profile web | docs | preserve`** — `web` (default) auto-picks;
  `docs` biases toward crisp lossless for text and screenshots;
  **`preserve` turns the engine off** and keeps your input format (the
  pre-0.3.0 behavior).
- **`--explain`** shows the whole decision — every candidate tried, its
  quality and size, which met the target, and why the winner won:

  ```
  optimize: png → jpeg (57803 → 33231 B, 43% smaller)
    class=photograph ... profile=web
    * jpeg lossy q=92 33231 B met=true
    reason: smallest candidate that met the target and beat the source (jpeg)
  ```

  `--explain=json` gives the same trace as machine-readable output for CI.
- **`--max-size 200KB`** switches the target from "a quality score" to "a
  byte budget" — and that's the mode where AVIF joins the shortlist.
- Pinning `--format` or an output extension (`-o out.webp`) bypasses the
  engine entirely — you asked for a format, you get it.

## The shape of it

Notice what `optimize` *isn't*: it isn't a new compressor. It's a thin
orchestrator that looks at an image, picks a few formats worth trying,
drives machinery crustyimg already had, and keeps the smallest honest
winner. That's why the whole feature added **zero new dependencies** and
still fits in a single pure-Rust binary — and why the same decision engine
is positioned to power a future goal-driven planner ("hit this size, keep
this quality") without being rewritten.

Give it a file. It looks, shortlists, measures, and picks — and with
`--explain`, it shows its work.
