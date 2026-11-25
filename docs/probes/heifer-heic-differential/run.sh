#!/bin/sh
# Differential probe for the `heifer` HEIC decoder against ffmpeg and libheif.
#
# Usage: run.sh WORK_DIR FILE.heic [FILE.heic ...]
#
# Needs: cargo, ffmpeg + ffprobe (with libx265), heif-dec (libheif), magick, perl, python3.
# Builds heifer from crates.io into WORK_DIR; nothing is installed globally.
# The first FILE is also used for the negative control and the corruption sweep,
# so make it a real, multi-tile phone photo.
#
# Exits non-zero if any decode differs from ffmpeg, the negative control fails to
# differ, or a corrupt file crashes or hangs the decoder.
set -eu

V="${HEIFER_VERSION:-0.3.20}"
W="${1:?usage: run.sh WORK_DIR FILE.heic...}"; shift
[ "$#" -ge 1 ] || { echo "usage: run.sh WORK_DIR FILE.heic..." >&2; exit 2; }
HERE="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$W/yuv" "$W/rgb" "$W/src"
W="$(cd "$W" && pwd)"
FAIL=0

# macOS has no `timeout`; perl's alarm kills the child after N seconds.
with_timeout() { perl -e 'alarm shift; exec @ARGV' "$@"; }
now() { perl -MTime::HiRes=time -e 'printf "%.2f", time'; }

echo "== build heifer $V"
CARGO_TARGET_DIR="$W/target" cargo install heifer-cli --version "$V" --locked --root "$W/bin" --quiet
H="$W/bin/bin/heifer"
# The low-level examples (item extraction, raw YUV decode) ship in the crate sources
# that `cargo install` just downloaded. Copy them out and build them there.
REG="$(find "${CARGO_HOME:-$HOME/.cargo}/registry/src" -maxdepth 2 -type d -name "heifer-isobmff-$V" | head -1)"
REG="$(dirname "$REG")"
for c in heifer-isobmff heifer-hevc-dec; do
  rm -rf "$W/src/$c-$V"; cp -R "$REG/$c-$V" "$W/src/"
done
(cd "$W/src/heifer-isobmff-$V" && CARGO_TARGET_DIR="$W/target" cargo build --release --quiet --example info)
(cd "$W/src/heifer-hevc-dec-$V" && CARGO_TARGET_DIR="$W/target" cargo build --release --quiet --example decode)
INFO="$W/target/release/examples/info"
DEC="$W/target/release/examples/decode"

# Decode one Annex-B stream with heifer and ffmpeg; print "same" or "DIFF".
yuv_cmp() {
  "$DEC" "$1" "$W/yuv/h.yuv" ${2:+"$2"} >/dev/null 2>&1 || { echo "heifer-error"; return; }
  pf="$(ffprobe -v error -show_entries stream=pix_fmt -of csv=p=0 "$1")"
  ffmpeg -v error -y -i "$1" -f rawvideo -pix_fmt "$pf" "$W/yuv/f.yuv" 2>/dev/null || { echo "ffmpeg-error"; return; }
  if cmp -s "$W/yuv/h.yuv" "$W/yuv/f.yuv"; then echo same; else echo DIFF; fi
}

echo "== 1. every HEVC item, raw YUV vs ffmpeg"
for f in "$@"; do
  total=0; same=0; bad=""
  for id in $("$INFO" "$f" | awk '$1=="item" && $3=="hvc1" {print $2}'); do
    total=$((total + 1))
    "$INFO" "$f" --extract "$id" "$W/yuv/t.h265" >/dev/null
    r="$(yuv_cmp "$W/yuv/t.h265" "")"
    if [ "$r" = same ]; then same=$((same + 1)); else bad="$bad $id:$r"; FAIL=1; fi
  done
  echo "$(basename "$f"): $same/$total identical${bad:+  MISMATCH:$bad}"
done

echo "== 2. negative control: loop filters off must differ"
first="$1"
# Read all of `info`'s output: exiting awk early would close the pipe on it.
id="$("$INFO" "$first" | awk '$1=="item" && $3=="hvc1" && !found {print $2; found=1}')"
"$INFO" "$first" --extract "$id" "$W/yuv/t.h265" >/dev/null
r="$(yuv_cmp "$W/yuv/t.h265" --no-filters)"
if [ "$r" = DIFF ]; then echo "ok: --no-filters output differs from ffmpeg"; else echo "CONTROL FAILED: $r"; FAIL=1; fi

echo "== 3. chroma formats and bit depths (x265 streams)"
magick -seed 7 -size 960x540 plasma:fractal -depth 16 "$W/yuv/src16.png"
for pf in yuv420p yuv422p yuv444p yuv420p10le yuv422p10le yuv444p10le yuv420p12le yuv444p12le; do
  if ffmpeg -v error -y -i "$W/yuv/src16.png" -frames:v 1 -c:v libx265 -pix_fmt "$pf" \
       -x265-params log-level=error -f hevc "$W/yuv/s.h265" 2>/dev/null; then
    r="$(yuv_cmp "$W/yuv/s.h265" "")"; echo "$pf: $r"; [ "$r" = same ] || FAIL=1
  else
    echo "$pf: x265 cannot encode it here (skipped)"
  fi
done

echo "== 4. RGB output vs libheif (bilinear chroma upsampling) and speed"
for f in "$@"; do
  b="$(basename "$f" .heic)"
  t0="$(now)"; "$H" "$f" "$W/rgb/$b.heifer.png" -f >/dev/null 2>&1; t1="$(now)"
  heif-dec -C bilinear "$f" "$W/rgb/$b.libheif.png" >/dev/null 2>&1; t2="$(now)"
  lh="$W/rgb/$b.libheif.png"
  # A multi-image file (e.g. a dynamic wallpaper) makes heif-dec write <name>-1.png, -2.png...
  [ -f "$lh" ] || lh="$W/rgb/$b.libheif-1.png"
  magick "$W/rgb/$b.heifer.png" -depth 8 "rgba:$W/rgb/a.raw"
  magick "$lh" -depth 8 "rgba:$W/rgb/b.raw"
  echo "$b: heifer $(echo "$t1 - $t0" | bc)s, libheif $(echo "$t2 - $t1" | bc)s; $(python3 -I "$HERE/stats.py" "$W/rgb/a.raw" "$W/rgb/b.raw")"
done

echo "== 5. corrupt inputs must error, not crash or hang"
size="$(wc -c < "$first")"
for cut in 100 2000 $((size / 8)) $((size / 2)) $((size - 1000)); do
  head -c "$cut" "$first" > "$W/yuv/bad.heic"
  set +e; with_timeout 30 "$H" "$W/yuv/bad.heic" "$W/yuv/bad.png" -f >/dev/null 2>&1; rc=$?; set -e
  echo "truncated to $cut bytes: exit $rc"; [ "$rc" -le 1 ] || FAIL=1
done
for seed in 1 2 3 4 5 6 7 8; do
  python3 -I "$HERE/stats.py" --flip "$seed" "$first" "$W/yuv/bad.heic"
  set +e; with_timeout 30 "$H" "$W/yuv/bad.heic" "$W/yuv/bad.png" -f >/dev/null 2>&1; rc=$?; set -e
  echo "300 random byte flips, seed $seed: exit $rc"; [ "$rc" -le 1 ] || FAIL=1
done

[ "$FAIL" -eq 0 ] && echo "== PASS" || echo "== FAIL"
exit "$FAIL"
