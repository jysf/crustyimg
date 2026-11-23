# Metadata fixtures

Made with tools independent of crustyimg (libjpeg-turbo's `cjpeg`), like `tests/fixtures/lint/README.md`.

## `rgb_adobe_t0_ids123.jpg` (452 bytes)

A 16x16, 3-component JPEG with an Adobe APP14 segment (colour transform 0, RGB) and component ids 1, 2, 3
instead of the letters `R`, `G`, `B`. A decoder that cannot read the APP14 falls back on the ids and takes
the scan for YCbCr, so the colours change. Segments: `APP14(t=0) DQT SOF0 DHT DHT SOS`.

Recipe (libjpeg-turbo 3.2.0; another version may give other bytes, so check the hashes):

1. A 16x16 binary PPM, pixel `(x, y)` = `(16x, 16y, 255 - 8x)`, rows top to bottom:

   ```
   python3 -I -c "import sys; px=b''.join(bytes([x*16, y*16, 255-x*8]) for y in range(16) for x in range(16)); sys.stdout.buffer.write(b'P6\n16 16\n255\n'+px)" > grad16.ppm
   ```

   sha256 `9ab37bf50d6b2a140914253443f05e3551c9bfebe7cb48a296b2ec60eba45caa`.
2. `cjpeg -rgb -quality 90 grad16.ppm > rgbRGB.jpg`: 452 bytes, sha256
   `7d106aa3cb5aeb08534dd8193d550bb9174bd034d4be99d1878ccf40136173c9`. APP14 is at offset 2, SOF0 at 87,
   SOS at 322; the component ids are `R G B` (`0x52 0x47 0x42`).
3. Byte edit: set offsets 97, 100, 103 (SOF0 component ids) and 327, 329, 331 (SOS component selectors)
   from `0x52 0x47 0x42` to `0x01 0x02 0x03`. Result: 452 bytes, sha256
   `dd3ec8ba9942afbeaab6e5e024afd0e32ae7418c3173cd72c91fb2490fdbf414`.
4. Check: `djpeg -pnm` of the result gives `srgb(128,128,191)` at pixel (8,8).
   With the APP14 removed it gives `srgb(216,83,128)`.
