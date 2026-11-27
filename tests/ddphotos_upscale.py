"""Make a replay original: SRC enlarged by FACTOR with Pillow's LANCZOS.

Usage: python -I tests/ddphotos_upscale.py SRC DST FACTOR [SHIFT]
Each side is round(side * FACTOR). A JPEG keeps its EXIF block, so a photo tagged
with an orientation stays tagged. SHIFT moves the content left by that many pixels
and keeps the size, repeating the last column: the harness's framing-shift control.
Pillow shares no code with crustyimg, so the original is independent of the engine.
"""
import sys

from PIL import Image

src, dst, factor = sys.argv[1], sys.argv[2], float(sys.argv[3])
shift = int(sys.argv[4]) if len(sys.argv) > 4 else 0

with Image.open(src) as im:
    exif = im.getexif().tobytes() if im.format == "JPEG" else None
    big = im.convert("RGB").resize((round(im.width * factor), round(im.height * factor)), Image.LANCZOS)

if shift:
    w, h = big.size
    moved = Image.new("RGB", (w, h))
    moved.paste(big.crop((shift, 0, w, h)), (0, 0))
    moved.paste(big.crop((w - 1, 0, w, h)).resize((shift, h)), (w - shift, 0))
    big = moved

if dst.endswith(".jpg"):
    big.save(dst, quality=95, exif=exif or b"")
else:
    big.save(dst)
