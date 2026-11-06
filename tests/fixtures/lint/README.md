# lint fixtures

Binary fixtures for `tests/lint.rs` and `src/lint/rules.rs`. Each is made by a tool
independent of crustyimg, so the test cannot pass because the code under test wrote it.

- `cmyk_16x16.jpg` (a 4-component CMYK JPEG, fires `color/wrong-colorspace`):
  `magick -size 16x16 xc:'#c04020' -colorspace CMYK -quality 90 cmyk_16x16.jpg`
