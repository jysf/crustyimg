# ddphotos parity fixtures

Five inputs for `tests/ddphotos_parity.mjs`, made outside crustyimg (ImageMagick and exiftool),
from `tests/fixtures/classify/color_photo_fuji.png` (maintainer-owned, EXIF-stripped). They carry
the repo's licence (MIT or Apache-2.0). The last three are made from `photo.webp` for the rows that
turn an aspect to follow each image's orientation: a portrait, a square, and a near-square that is
portrait by one pixel.

| file | what it is | bytes | SHA-256 |
|---|---|---|---|
| `photo.webp` | 1600×1071 lossy WebP, the shape ddphotos publishes | 23,424 | `e34df127630d40715eee45754799a08d9b7f0c96ada9f24e1daecb0483511c6e` |
| `photo_rot.jpg` | the same picture as a 1600×1071 JPEG with EXIF Orientation 6 | 128,696 | `c23511dacbb744bac894b194f5c4f6567d8498a8ada2e16e03281b880dc742ed` |
| `photo_portrait.webp` | `photo.webp` turned 90°: a 1071×1600 portrait | 20,006 | `ad873f3840e8c014da78ab12a2c5880f7e2bf6d34755150759039af578ac83b8` |
| `photo_square.webp` | the middle 1071×1071 of `photo.webp` | 13,358 | `f5ef7dcab17171e6dd349fb9bf041db951d4391940e26205d5a4964ffb0a0127` |
| `photo_near_square.webp` | the middle 1071×1072 of the turned picture: portrait by one pixel | 13,166 | `8f64ced1f64393544fbdd20e4a8c0c9550d68fe5029dfe20229cb72662f76ab2` |

Made with ImageMagick 7.1.2-34 (Q16-HDRI, aarch64) and exiftool 13.55, from the repo root:

```sh
cd tests/fixtures/ddphotos
magick ../classify/color_photo_fuji.png -gravity center -crop 448x300+0+0 +repage \
  -resize 1600x -strip -quality 82 -define webp:lossless=false photo.webp
magick ../classify/color_photo_fuji.png -gravity center -crop 448x300+0+0 +repage \
  -resize 1600x -strip -quality 90 photo_rot.jpg
exiftool -q -n -Orientation=6 -overwrite_original photo_rot.jpg
magick photo.webp -rotate 90 -strip -quality 82 -define webp:lossless=false photo_portrait.webp
magick photo.webp -gravity center -crop 1071x1071+0+0 +repage -strip -quality 82 \
  -define webp:lossless=false photo_square.webp
magick photo.webp -rotate 90 -gravity center -crop 1071x1072+0+0 +repage -strip -quality 82 \
  -define webp:lossless=false photo_near_square.webp
```

Do not regenerate them to make a row pass. (The last three are built from the committed `photo.webp`, so run their commands from this directory.) A different ImageMagick shows in the encoder's output,
so a rebuild changes the pixels, and the measured bound on resampling rows (`K` in the harness)
must then be measured again. CI needs no generator: it uses these committed bytes.
