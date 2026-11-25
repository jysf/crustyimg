"""Decode image files to raw RGBA with Pillow, so the parity harness compares pixels
decoded by libraries that are not crustyimg's (libjpeg-turbo, libwebp, libavif, zlib).

Usage: python -I tests/ddphotos_decode.py FILE...
For each FILE this writes FILE.rgba (raw RGBA8) and prints one JSON line
{"path", "width", "height"}. Any failure exits non-zero.
"""
import json
import sys

from PIL import Image

for path in sys.argv[1:]:
    with Image.open(path) as im:
        rgba = im.convert("RGBA")
        with open(path + ".rgba", "wb") as out:
            out.write(rgba.tobytes())
        print(json.dumps({"path": path, "width": rgba.width, "height": rgba.height}))
