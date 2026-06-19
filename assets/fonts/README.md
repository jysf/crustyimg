# Bundled fonts

## Go-Regular.ttf

The **default font for `watermark --text`** (SPEC-030, DEC-032). Embedded into the
binary via `include_bytes!`; users can override it with `watermark --text … --font
PATH`.

- **Family:** Go (Go Regular), by Bigelow & Holmes for the Go project.
- **License:** **BSD-3-Clause** — see [`LICENSE-Go`](LICENSE-Go) (the `golang/image`
  repository license; "Copyright 2009 The Go Authors"). Permissive, SPDX-clean, and
  compatible with crustyimg's `MIT OR Apache-2.0` (DEC-018).
- **Source:** `github.com/golang/image` → `font/gofont/ttfs/Go-Regular.ttf`.
- **Why this one:** small (~145 KB), clean permissive license, broad Latin coverage.
  `ab_glyph` (Apache-2.0) rasterizes it; we deliberately do **not** use `imageproc`
  (it drags in `sdl2`/`nalgebra` — DEC-032).

The license file is shipped alongside the font and the attribution is retained in the
binary's `--help`/docs as required by BSD-3.
