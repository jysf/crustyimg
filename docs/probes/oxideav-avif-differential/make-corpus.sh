#!/bin/sh
# Build the differential corpus from crustyimg's own fixtures (no personal images).
# Needs: avifenc (libavif + libaom), magick, and a crustyimg binary on PATH for the ravif files.
set -eu
R="$(cd "$(dirname "$0")/../../.." && pwd)"
C="${1:?usage: make-corpus.sh OUT_DIR}"; mkdir -p "$C/src"; S="$C/src"
magick "$R/tests/fixtures/classify/color_photo_fuji.png" -alpha off "$S/fuji.png"
magick "$R/tests/fixtures/classify/grayscale_photo_leica.png" -colorspace sRGB -type TrueColor "$S/leica.png"
magick "$R/tests/fixtures/c2pa/signed_gps.jpg" -strip "$S/gps.png"
magick "$S/fuji.png" \( -size 448x448 radial-gradient:white-black \) -alpha off -compose CopyOpacity -composite "$S/fuji_alpha.png"
magick "$S/gps.png" -resize 4000x3000! "$S/big12mp.png"
magick "$R/tests/fixtures/classify/photo_entropy_floor.png" -crop 333x217+0+0 +repage "$S/odd.png"
magick "$S/fuji.png" -resize 4032x3024! -attenuate 0.4 +noise Gaussian "$S/color12.png"
e() { out=$1; shift; avifenc -j 1 "$@" "$C/$out" >/dev/null; }
for d in 8 10 12; do for y in 420 422 444 400; do e "fuji_d${d}_${y}_q60.avif" -d $d -y $y -q 60 "$S/fuji.png"; done; done
e fuji_q20.avif -q 20 "$S/fuji.png"; e fuji_q95.avif -q 95 "$S/fuji.png"; e fuji_lossless.avif -l "$S/fuji.png"
e fuji_s0_q50.avif -s 0 -q 50 "$S/fuji.png"
e fuji_alpha_q60.avif -q 60 --qalpha 50 "$S/fuji_alpha.png"; e fuji_alpha_10b.avif -d 10 -q 60 "$S/fuji_alpha.png"
e leica_q55.avif -q 55 "$S/leica.png"
e gps_tiles.avif -q 60 --tilerowslog2 1 --tilecolslog2 1 "$S/gps.png"
e gps_grid2x2.avif -q 60 --grid 2x2 "$S/gps.png"
e gps_q70.avif -q 70 "$S/gps.png"; e gps_film.avif -q 50 -a denoise-noise-level=25 "$S/gps.png"
e odd_q60.avif -q 60 "$S/odd.png"; e odd_444.avif -y 444 -q 60 "$S/odd.png"
e big12mp_q60.avif -q 60 -s 8 "$S/big12mp.png"
e fuji_limited.avif -q 60 -r limited "$S/fuji.png"; e fuji_cicp2020.avif -q 60 --cicp 9/16/9 -d 10 "$S/fuji.png"
avifenc -j 8 -y 420 -q 60 -s 6 "$S/color12.png" "$C/color12mp_420.avif" >/dev/null
avifenc -j 8 -y 420 -d 10 -q 60 -s 6 "$S/color12.png" "$C/color12mp_420_10b.avif" >/dev/null
for q in 40 75; do crustyimg convert --format avif -q $q "$S/fuji.png" -o "$C/ravif_fuji_q$q.avif"; done
crustyimg convert --format avif "$S/fuji_alpha.png" -o "$C/ravif_fuji_alpha.avif"
crustyimg convert --format avif "$S/gps.png" -o "$C/ravif_gps.avif"
cp "$R"/tests/fixtures/avif/*.avif "$C/"
ls "$C"/*.avif | wc -l
