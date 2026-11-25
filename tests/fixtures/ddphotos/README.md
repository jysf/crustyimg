# ddphotos parity fixtures

Two inputs for `tests/ddphotos_parity.mjs`, made outside crustyimg (ImageMagick and exiftool),
from `tests/fixtures/classify/color_photo_fuji.png` (maintainer-owned, EXIF-stripped). They carry
the repo's licence (MIT or Apache-2.0).

| file | what it is | bytes | SHA-256 |
|---|---|---|---|
| `photo.webp` | 1600×1071 lossy WebP, the shape ddphotos publishes | 23,424 | `e34df127630d40715eee45754799a08d9b7f0c96ada9f24e1daecb0483511c6e` |
| `photo_rot.jpg` | the same picture as a 1600×1071 JPEG with EXIF Orientation 6 | 128,696 | `c23511dacbb744bac894b194f5c4f6567d8498a8ada2e16e03281b880dc742ed` |

Made with ImageMagick 7.1.2-34 (Q16-HDRI, aarch64) and exiftool 13.55, from the repo root:

```sh
cd tests/fixtures/ddphotos
magick ../classify/color_photo_fuji.png -gravity center -crop 448x300+0+0 +repage \
  -resize 1600x -strip -quality 82 -define webp:lossless=false photo.webp
magick ../classify/color_photo_fuji.png -gravity center -crop 448x300+0+0 +repage \
  -resize 1600x -strip -quality 90 photo_rot.jpg
exiftool -q -n -Orientation=6 -overwrite_original photo_rot.jpg
```

Do not regenerate them to make a row pass. A different ImageMagick shows in the encoder's output,
so a rebuild changes the pixels, and the measured bound on resampling rows (`K` in the harness)
must then be measured again. CI needs no generator: it uses these committed bytes.
