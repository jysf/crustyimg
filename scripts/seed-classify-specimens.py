#!/usr/bin/env python3
"""Seed the two classifier boundary specimens in `tests/fixtures/classify/`.

Provenance tool, not part of the test run: the fixtures are committed, and the
Rust tests decode and measure them. This script exists so the specimens can be
regenerated and so their recipes are executable rather than prose.

It is deliberately dependency-free (stdlib zlib/struct only) and it never calls
crustyimg. The entropy it prints is computed by an independent implementation of
the same documented definition — Shannon entropy of the 256-bin histogram of the
integer BT.601-ish luma `(77R + 150G + 29B) >> 8`. Cross-checking that number
against the value the Rust tests assert is the point: a fixture measured only by
the code under test cannot fail.

Positive control (both implementations, committed fixtures):

    grayscale_photo_leica.png   6.0743      grayscale_photo_canon.png  6.8300
    color_photo_fuji.png        6.3737      dithered_graphic.png       3.0259

Usage:  python3 scripts/seed-classify-specimens.py [--check]

`--check` regenerates into memory and compares against the committed bytes
instead of overwriting them.
"""

import argparse
import math
import os
import struct
import sys
import zlib

FIXTURES = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "tests",
    "fixtures",
    "classify",
)


# ── minimal PNG I/O ──────────────────────────────────────────────────────────


def read_png_rgb(path):
    """Decode an 8-bit non-interlaced PNG (gray / RGB / RGBA) to (w, h, rows),
    each row a `bytes` of 3*w RGB samples. Alpha is dropped: the luma the
    classifier measures ignores it."""
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", f"{path}: not a PNG"
    pos, idat, w, h, ctype = 8, bytearray(), None, None, None
    while pos < len(data):
        (ln,) = struct.unpack(">I", data[pos : pos + 4])
        tag, body = data[pos + 4 : pos + 8], data[pos + 8 : pos + 8 + ln]
        pos += 12 + ln
        if tag == b"IHDR":
            w, h, depth, ctype, _comp, _filt, inter = struct.unpack(">IIBBBBB", body)
            assert depth == 8 and inter == 0, f"{path}: unsupported depth/interlace"
            assert ctype in (0, 2, 6), f"{path}: unsupported colour type {ctype}"
        elif tag == b"IDAT":
            idat.extend(body)
        elif tag == b"IEND":
            break
    raw = zlib.decompress(bytes(idat))
    bpp = {0: 1, 2: 3, 6: 4}[ctype]
    stride, rows, prev, p = w * bpp, [], bytearray(w * bpp), 0
    for _ in range(h):
        ft, p = raw[p], p + 1
        line, p = bytearray(raw[p : p + stride]), p + stride
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b = prev[i]
            c = prev[i - bpp] if i >= bpp else 0
            x = line[i]
            if ft == 0:
                v = x
            elif ft == 1:
                v = x + a
            elif ft == 2:
                v = x + b
            elif ft == 3:
                v = x + ((a + b) >> 1)
            elif ft == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                v = x + pr
            else:
                raise ValueError(f"{path}: bad filter {ft}")
            line[i] = v & 0xFF
        prev = line
        if ctype == 2:
            rows.append(bytes(line))
        elif ctype == 0:
            rows.append(bytes(s for g in line for s in (g, g, g)))
        else:
            rows.append(bytes(line[i] for i in range(stride) if i % 4 != 3))
    return w, h, rows


def encode_png_rgb(w, h, rows):
    raw = bytearray()
    for row in rows:
        raw.append(0)  # filter type 0 (None) — deterministic, no heuristics
        raw.extend(row)

    def chunk(tag, body):
        return (
            struct.pack(">I", len(body))
            + tag
            + body
            + struct.pack(">I", zlib.crc32(tag + body) & 0xFFFFFFFF)
        )

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


# ── the independent measurement ──────────────────────────────────────────────


def luma(r, g, b):
    return (77 * r + 150 * g + 29 * b) >> 8


def measure(rows):
    hist = [0] * 256
    colours = set()
    for row in rows:
        for i in range(0, len(row), 3):
            hist[luma(row[i], row[i + 1], row[i + 2])] += 1
            colours.add(row[i : i + 3])
    total = sum(hist)
    entropy = -sum((c / total) * math.log2(c / total) for c in hist if c)
    return entropy, sum(1 for c in hist if c), len(colours)


# ── the two recipes ──────────────────────────────────────────────────────────


def photo_entropy_floor():
    """A real photograph under flat light.

    Source: the committed `grayscale_photo_leica.png` (a real Leica B&W frame).
    Curve: compress the tonal range to a THIRD of full range about mid-gray,
           `v' = 128 + (v - 128) // 3`, per channel.

    That is what heavy haze or flat overcast light does to a capture: real
    photographic structure living in a third of the tonal range. `k = 1/3` comes
    from that story, not from any threshold. Entropy is predictable in advance
    without running the classifier — merging three input levels into one costs
    log2(3) bits, so expect about 6.07 - 1.58 = 4.49.
    """
    w, h, rows = read_png_rgb(os.path.join(FIXTURES, "grayscale_photo_leica.png"))
    return w, h, [bytes(128 + (v - 128) // 3 for v in row) for row in rows]


def dither_32color():
    """The adversarial high-entropy graphic: an error-diffusion dither.

    Source:  the committed `color_photo_fuji.png` (a real Fuji colour frame).
    Palette: 32 evenly spaced grey levels (5-bit grey), a fixed ramp — NOT an
             adaptive palette, whose equal-population boxes would drive the
             histogram toward uniform and push entropy at log2(levels).
    Render:  Floyd-Steinberg error diffusion, the textbook
             7/16 - 3/16 - 5/16 - 1/16 kernel, left-to-right, no serpentine.

    Entropy is bounded by construction and predictable in advance: quantising
    256 luma levels down to 32 discards at most log2(256/32) = 3 bits, so expect
    about 6.37 - 3 = 3.37 — under the 4-bit photo floor with real margin, and
    well above the flat 8-colour dither already committed at 3.03.

    Note the level count is what bounds this. A 32-*colour* dither (an adaptive
    RGB palette) is a different animal: its 32 colours carry far more than 32
    distinct lumas, which is why DEC-047 records one reaching 5.1 and crossing
    the floor. This specimen is the grayscale-level variant precisely because
    its ceiling is structural.
    """
    w, h, rows = read_png_rgb(os.path.join(FIXTURES, "color_photo_fuji.png"))
    buf = [[float(luma(r[i], r[i + 1], r[i + 2])) for i in range(0, len(r), 3)] for r in rows]
    levels = 32
    step = 255 / (levels - 1)
    for y in range(h):
        for x in range(w):
            old = buf[y][x]
            new = min(255.0, max(0.0, round(old / step) * step))
            err = old - new
            buf[y][x] = new
            for dx, dy, f in ((1, 0, 7 / 16), (-1, 1, 3 / 16), (0, 1, 5 / 16), (1, 1, 1 / 16)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h:
                    buf[ny][nx] += err * f
    rows_out = [bytes(s for v in row for s in (int(round(v)),) * 3) for row in buf]
    return w, h, rows_out


RECIPES = {
    "photo_entropy_floor.png": photo_entropy_floor,
    "dither_32color.png": dither_32color,
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="compare, do not write")
    args = ap.parse_args()

    failed = False
    for name, recipe in RECIPES.items():
        w, h, rows = recipe()
        png = encode_png_rgb(w, h, rows)
        entropy, bins, colours = measure(rows)
        path = os.path.join(FIXTURES, name)
        print(
            f"{name:26} {w}x{h}  entropy={entropy:.4f}  luma_bins={bins}  colours={colours}"
        )
        if args.check:
            if not os.path.exists(path) or open(path, "rb").read() != png:
                print(f"  MISMATCH: {path} differs from the recipe output")
                failed = True
        else:
            with open(path, "wb") as f:
                f.write(png)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
