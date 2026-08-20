#!/usr/bin/env python3
"""SPEC-120 — measure the linear-light premise (STAGE-046's falsification gate).

`Resize::apply` hands non-linear sRGB values to `fast_image_resize` as if they
were linear (`src/operation/mod.rs`). This harness answers two questions and
keeps them apart:

  1. **Is the physical error real?**  Mean *linear* luminance error of each
     candidate against an independent linear-light reference.
  2. **Can SSIMULACRA2 (DEC-019) see it?**  The repo's perceptual oracle scored
     against the same reference. A null on the realistic cases only means
     "premise false" if the metric provably registers the extreme case — so the
     synthetic worst case is a **positive control**, not just another row.

Three arms, scored at equal dimensions because SSIMULACRA2 requires them
(`src/cli/report.rs:329`) — you cannot score a downscale against its source:

    source ─┬─► crustyimg today   (sRGB U8x4 Lanczos3, the shipped binary)
            ├─► prototype         (linear f32 Lanczos3, examples/spec120_linear_probe.rs)
            └─► REFERENCE         (ImageMagick, explicit linear colorspace)

The reference is generated **outside this codebase** on purpose: a reference
produced by the code under test cannot fail the code under test.

Four controls run every time and are reported, not assumed:

  C1  the prototype's sRGB arm reproduces the shipped binary **pixel-exactly**
      — so the linear arm's delta is attributable to the transfer function and
      nothing else.
  C2  ImageMagick's own sRGB-space resize differs hugely from its linear-space
      resize — proof that `-colorspace RGB` actually moved the variable.
  C3  crustyimg today ≈ ImageMagick's sRGB-space resize — proof the two Lanczos3
      implementations agree, so arm-vs-reference gaps are gamma, not filter drift.
  C4  (alpha) a deliberately non-premultiplied resize shows a large edge error —
      proof the alpha oracle can fire before any null from it is believed.

Requirements: `magick` (ImageMagick 7, HDRI build) on PATH, a built
`crustyimg`, and a built `spec120_linear_probe` example. Stdlib only; no network.

Usage:
    cargo build --release
    cargo build --release --example spec120_linear_probe
    python3 scripts/spec120_linear_light.py [--json] [--work DIR] [--keep]
"""

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# (name, source, target_w, target_h) — the luminance/SSIMULACRA2 cases.
CASES = [
    ("synthetic_worst_case", "synth", 256, 256),   # 2048x2048 -> 256x256, 8x
    ("graphic_large.png", "corpus", 128, 128),     # 512x512   -> 128x128, 4x
    ("photo_forest_cc0.jpg", "corpus", 200, 133),  # 800x532   -> 200x133, 4x
]
ALPHA_TARGET = (128, 128)                          # 1024x1024 -> 128x128, 8x


def run(cmd, **kw):
    """Run a command, capture both streams, raise with context on failure.

    Never piped: a piped command reports the pipe's exit code, not the tool's.
    """
    cmd = [str(c) for c in cmd]
    p = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if p.returncode != 0:
        raise SystemExit(
            f"command failed ({p.returncode}): {' '.join(cmd)}\n"
            f"stdout: {p.stdout}\nstderr: {p.stderr}"
        )
    return p.stdout


def probe_json(probe, *args):
    return json.loads(run([probe, *args]).strip())


def find_binary(explicit, name, sub=""):
    if explicit:
        p = Path(explicit)
        if p.is_file():
            return p
        raise SystemExit(f"not found: {p}")
    for profile in ("release", "debug"):
        p = REPO_ROOT / "target" / profile / sub / name
        if p.is_file():
            return p
    raise SystemExit(
        f"{name} not built. Run: cargo build --release"
        + (f" --example {name}" if sub else "")
    )


def magick_resize(magick, src, dst, w, h, linear):
    """Independent reference resize. `-colorspace RGB` is ImageMagick's *linear*
    RGB; the Q16-HDRI build carries it in float, so nothing is quantized between
    the two colorspace conversions. `-filter Lanczos` is 3-lobe Lanczos, the
    same kernel `Resize::apply` asks `fast_image_resize` for."""
    cmd = [magick, str(src)]
    if linear:
        cmd += ["-colorspace", "RGB"]
    cmd += ["-filter", "Lanczos", "-resize", f"{w}x{h}!"]
    if linear:
        cmd += ["-colorspace", "sRGB"]
    cmd += ["-depth", "8", "-strip", str(dst)]
    run(cmd)


def ss2(binary, ref, cand):
    out = run([str(binary), "diff", str(ref), str(cand), "--json"])
    return json.loads(out)["score"]


def versions(magick, binary):
    im = run([magick, "-version"]).splitlines()[0].strip()
    cimg = run([str(binary), "--version"]).strip()
    fir = "unknown"
    lock = (REPO_ROOT / "Cargo.lock").read_text().splitlines()
    for i, line in enumerate(lock):
        if line.strip() == 'name = "fast_image_resize"':
            fir = lock[i + 1].split('"')[1]
            break
    return {
        "imagemagick": im,
        "crustyimg": cimg,
        "fast_image_resize": fir,
        "python": sys.version.split()[0],
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--bin", help="crustyimg binary (default: target/{release,debug})")
    ap.add_argument("--probe", help="spec120_linear_probe example binary")
    ap.add_argument("--magick", default=shutil.which("magick") or "magick")
    ap.add_argument("--work", help="working directory (default: a temp dir)")
    ap.add_argument("--keep", action="store_true", help="keep the working directory")
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    args = ap.parse_args()

    binary = find_binary(args.bin, "crustyimg")
    probe = find_binary(args.probe, "spec120_linear_probe", sub="examples")
    if not shutil.which(args.magick) and not Path(args.magick).is_file():
        raise SystemExit(
            "ImageMagick (`magick`) not found. It generates the INDEPENDENT "
            "reference; the measurement is not valid without an outside tool."
        )

    work = Path(args.work) if args.work else Path(tempfile.mkdtemp(prefix="spec120-"))
    work.mkdir(parents=True, exist_ok=True)

    report = {
        "spec": "SPEC-120",
        "versions": versions(args.magick, binary),
        "cases": [],
        "controls": {},
        "alpha": {},
    }

    # ── sources ──────────────────────────────────────────────────────────────
    run([str(probe), "synth", str(work / "synthetic_worst_case.png")])
    run([str(probe), "synth-alpha", str(work / "alpha_case.png")])
    # The photo is a JPEG; decode it ONCE to PNG so all three arms consume
    # identical pixels and no JPEG decoder difference leaks into the numbers.
    run([str(probe), "topng", str(REPO_ROOT / "bench/corpus/photo_forest_cc0.jpg"),
         str(work / "photo_forest_cc0.jpg.png")])
    shutil.copy(REPO_ROOT / "bench/corpus/graphic_large.png",
                work / "graphic_large.png.png")

    c1_all_identical = True
    c2_rows, c3_rows = [], []

    for name, origin, w, h in CASES:
        src = work / (f"{name}.png" if origin == "corpus" else f"{name}.png")
        ref_lin = work / f"{name}.ref_linear.png"
        ref_srgb = work / f"{name}.ref_srgb.png"
        today = work / f"{name}.crustyimg_today.png"
        proto = work / f"{name}.proto_linear.png"
        replica = work / f"{name}.proto_srgb.png"

        magick_resize(args.magick, src, ref_lin, w, h, linear=True)
        magick_resize(args.magick, src, ref_srgb, w, h, linear=False)
        run([str(binary), "resize", str(src), "--exact", f"{w}x{h}", "-o", str(today)])
        run([str(probe), "resize", str(src), str(proto), w, h, "linear"])
        run([str(probe), "resize", str(src), str(replica), w, h, "srgb"])

        c1 = probe_json(probe, "pixdiff", today, replica)
        c1_all_identical &= c1["identical"]
        c2 = probe_json(probe, "luma", ref_lin, ref_srgb)
        c3 = probe_json(probe, "luma", ref_srgb, today)
        c2_rows.append({"case": name, **c2})
        c3_rows.append({"case": name, **c3})

        report["cases"].append({
            "case": name,
            "source_dims": None,
            "target": f"{w}x{h}",
            "control_c1_prototype_matches_shipped_binary": c1,
            "today": {
                "luma": probe_json(probe, "luma", ref_lin, today),
                "ssimulacra2": ss2(binary, ref_lin, today),
            },
            "prototype_linear": {
                "luma": probe_json(probe, "luma", ref_lin, proto),
                "ssimulacra2": ss2(binary, ref_lin, proto),
            },
        })

    for row in report["cases"]:
        row["ssimulacra2_delta"] = round(
            row["prototype_linear"]["ssimulacra2"] - row["today"]["ssimulacra2"], 4
        )

    report["controls"] = {
        "c1_prototype_reproduces_shipped_binary": c1_all_identical,
        "c2_imagemagick_colorspace_moved_the_variable": c2_rows,
        "c3_today_agrees_with_imagemagick_srgb_resize": c3_rows,
    }

    # ── Call 5: the alpha half, its own oracle ───────────────────────────────
    aw, ah = ALPHA_TARGET
    asrc = work / "alpha_case.png"
    aref = work / "alpha_case.ref_premul.png"
    atoday = work / "alpha_case.crustyimg_today.png"
    anopremul = work / "alpha_case.proto_nopremul.png"
    # sRGB space on BOTH sides: this isolates premultiplication from gamma.
    # ImageMagick associates alpha for resize by default, so this reference is
    # a premultiplied one.
    magick_resize(args.magick, asrc, aref, aw, ah, linear=False)
    run([str(binary), "resize", str(asrc), "--exact", f"{aw}x{ah}", "-o", str(atoday)])
    run([str(probe), "resize", str(asrc), str(anopremul), aw, ah, "srgb-nopremul"])
    report["alpha"] = {
        "target": f"{aw}x{ah}",
        "method": (
            "max per-channel difference in PREMULTIPLIED 8-bit RGB over pixels "
            "where either image has 0 < alpha < 255 (the anti-aliased edge band). "
            "Premultiplied difference is the visible composite error and is "
            "background-independent when the alphas agree, which max_alpha_err checks."
        ),
        "today_vs_premultiplied_reference": probe_json(probe, "alpha-edge", aref, atoday),
        "control_c4_nonpremultiplied_arm": probe_json(probe, "alpha-edge", aref, anopremul),
        # C5 removes the cross-implementation variable entirely: same resampler,
        # same filter, same build — only `mul_div_alpha` differs. If the shipped
        # binary premultiplies, its output cannot equal the non-premultiplied arm.
        "control_c5_today_differs_from_nonpremultiplied_arm": probe_json(
            probe, "pixdiff", atoday, anopremul
        ),
    }

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print_table(report)

    if not args.keep and not args.work:
        shutil.rmtree(work, ignore_errors=True)
    else:
        print(f"working directory kept: {work}", file=sys.stderr)
    return 0


def print_table(r):
    v = r["versions"]
    print("SPEC-120 — linear-light premise, measured")
    print(f"  reference tool : {v['imagemagick']}")
    print(f"  binary         : {v['crustyimg']}")
    print(f"  resampler      : fast_image_resize {v['fast_image_resize']} (both arms)")
    print()
    hdr = f"{'case':<24} {'target':>9} {'arm':<18} {'mean luma err':>14} {'vs ref %':>9} {'SSIMULACRA2':>12}"
    print(hdr)
    print("-" * len(hdr))
    for c in r["cases"]:
        for arm, key in (("crustyimg today", "today"), ("prototype linear", "prototype_linear")):
            m = c[key]
            print(f"{c['case']:<24} {c['target']:>9} {arm:<18} "
                  f"{m['luma']['mean_signed_luma_err']:>14.6f} "
                  f"{m['luma']['mean_signed_luma_err_pct_of_ref']:>8.2f}% "
                  f"{m['ssimulacra2']:>12.4f}")
        print(f"{'':<24} {'':>9} {'Δ SSIMULACRA2':<18} {'':>14} {'':>9} {c['ssimulacra2_delta']:>12.4f}")
    print()
    ctl = r["controls"]
    print("controls")
    print(f"  C1 prototype sRGB arm == shipped binary, pixel-exact : "
          f"{'PASS' if ctl['c1_prototype_reproduces_shipped_binary'] else 'FAIL'}")
    for row in ctl["c2_imagemagick_colorspace_moved_the_variable"]:
        print(f"  C2 IM sRGB vs IM linear   [{row['case']:<22}] mean luma err "
              f"{row['mean_signed_luma_err']:+.6f} ({row['mean_signed_luma_err_pct_of_ref']:+.2f}%)")
    for row in ctl["c3_today_agrees_with_imagemagick_srgb_resize"]:
        print(f"  C3 today vs IM sRGB       [{row['case']:<22}] mean |luma err| "
              f"{row['mean_abs_luma_err']:.6f}, max {row['max_abs_luma_err']:.6f}")
    print()
    a = r["alpha"]
    print(f"alpha (Call 5), {a['target']}, premultiplied-edge oracle")
    t = a["today_vs_premultiplied_reference"]
    n = a["control_c4_nonpremultiplied_arm"]
    print(f"  crustyimg today            max premul RGB err {t['max_premul_rgb_err']:>4}"
          f"   (mean {t['mean_premul_rgb_err']:.3f}, edge px {t['edge_pixels']}, "
          f"max alpha err {t['max_alpha_err']})")
    print(f"  C4 non-premultiplied arm   max premul RGB err {n['max_premul_rgb_err']:>4}"
          f"   (mean {n['mean_premul_rgb_err']:.3f}, edge px {n['edge_pixels']}, "
          f"max alpha err {n['max_alpha_err']})")
    c5 = a["control_c5_today_differs_from_nonpremultiplied_arm"]
    print(f"  C5 today vs that same arm  differing pixels {c5['differing_pixels']}, "
          f"max RGBA err {c5['max_abs_rgba_err']} "
          f"-> premultiplication is {'ON' if not c5['identical'] else 'OFF'} in the shipped binary")
    print(f"  (mean alpha err today {t['mean_alpha_err']:.4f}/255 — the two implementations' "
          f"alpha channels agree on average. Read any residual max against the C4 line "
          f"above, which has premultiplication OFF: where the two carry the same "
          f"residual it is not a halo and not the premultiply/divide round-trip, but "
          f"8-bit quantization in the integer resampling path — alpha's own "
          f"convolution included, since alpha is never premultiplied or divided)")


if __name__ == "__main__":
    raise SystemExit(main())
