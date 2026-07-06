# crustyimg 0.3.0: `optimize` picks the format for you (the local `f_auto`)

*2026-07-06 · feature release · the optimization engine*

## The guessing game

Shipping images for the web means answering a question you probably don't
want to think about: **what format?** JPEG is small for photos but mangles
crisp text and has no transparency. PNG is lossless and perfect for logos
but enormous for photographs. WebP is usually smaller than both — but not
*always*, and not for every image. The honest answer is "it depends on the
picture," which is exactly the kind of per-file judgement humans are bad at
and get bored doing.

Cloudinary solved this years ago with `f_auto`: you stop naming a format,
and their service looks at the image and picks the best one for you. It's
genuinely great — and it's also a hosted service in your delivery path. If
you just want to optimize an `assets/` folder in CI, on your own machine,
with no account and no Node, `f_auto` isn't available to you.

**crustyimg 0.3.0 is the local, deterministic version of that idea.**

## Give it the file; it decides

Before 0.3.0, `crustyimg optimize` kept your file in whatever format you
gave it and just tuned the quality. Now it chooses the format too:

```
$ crustyimg optimize photo.png --out-dir web/
photo.png: png → jpeg · 57803 → 33231 B (43% smaller)
```

It looked at `photo.png`, recognized it as a photograph, tried the formats
that make sense for a photo, measured them, and shipped the smallest one
that still hit the quality bar — a JPEG, 43% smaller than the source. A
logo or a screenshot with flat colors would have gone the other way, to a
lossless WebP or PNG.

Two promises make it safe to leave on by default:

- **It never hands you a bigger file.** If nothing actually beats your
  original, it leaves the file untouched and says so.
- **It's deterministic.** The same image and flags always produce the same
  bytes — no surprises between runs or machines.

## It tells you *why*

An auto-decider you can't inspect is just a black box with better PR. So
every decision is auditable:

```
$ crustyimg optimize photo.png --out-dir web/ --explain
photo.png:
optimize: png → jpeg (57803 → 33231 B, 43% smaller)
  class=photograph entropy=7.62 edges=0.42 flat=0.00 colors=4096+ profile=web
  * jpeg lossy q=92 33231 B met=true
  reason: smallest candidate that met the target and beat the source (jpeg)
```

There's a `--explain=json` too, for wiring the decision into CI or a build
report. The moat here isn't "we have another format" — it's *"the smallest
file, in the format the tool picked, with the reasons shown."*

## The knobs (and the one thing to know)

- **`--profile web | docs | preserve`** — `web` (default) auto-picks;
  `docs` leans lossless for crisp text and screenshots; **`preserve`
  reproduces the old, format-preserving behavior exactly.**
- **`--explain` / `--explain=json`** — the trace above, human or machine.
- By default `optimize` prints a one-line summary of what it did;
  `--quiet` silences it.

The **one breaking change**: because `optimize`'s default now *may change
the file's format* (a photographic PNG can come out as a JPEG), scripts
that relied on it keeping the input format should add `--profile preserve`
(or pin the format with `--format` / `-o name.png`). That's it.

Still a **single pure-Rust binary with zero system dependencies** — the
whole engine is analysis plus logic over the pixels you already decoded, so
0.3.0 adds no new default dependency and nothing new to install.

## What's under it — and what it unlocks

The visible feature sits on an invisible one: a new shared **image-analysis
layer**. Once per image, crustyimg computes a compact set of facts — color
histogram, entropy, edge density, transparency, a capped unique-color
count, a dominant color — and runs a small, deterministic (no-ML)
classifier that decides *photograph vs. graphic vs. icon vs. document vs.
screenshot*. That classification is what biases the format choice; the
`optimize` engine just shortlists a few candidate formats, drives the
existing perceptual-quality search across them, and keeps the smallest
winner.

Building that layer as a shared foundation — rather than burying the logic
inside one command — is the point. The same analysis is what the next waves
read: a general **goal-driven planner** ("hit this size, keep this
quality"), a source-tree **linter** ("this asset would be 60% smaller as
WebP"), and a **web-asset manifest**. `optimize` is the first, most
useful thing built on it — not the last.

Give it a file. It decides, and it tells you why.

---

*crustyimg is a fast, scriptable, single-binary image CLI written in Rust —
view, resize, convert, watermark, strip metadata, and now auto-optimize,
with zero system dependencies by default. 0.3.0 is the optimization-engine
release.*
