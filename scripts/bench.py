#!/usr/bin/env python3
"""crustyimg committed benchmark harness (SPEC-088).

Runs `web` and `optimize` over a corpus of images and reports **savings / time /
score**, either as a human table or as `--json`. It drives the real compiled
binary and parses the CLI's own `--json --timing` audit report (the
`crustyimg.optimize.explain/v1` schema), so the harness and the report can't
drift.

Design guarantees (the whole point of a *committed* bench a skeptic can re-run):

  * OFFLINE     — only ever spawns the local `crustyimg` binary; no sockets,
                  no downloads, no package installs.
  * NO TELEMETRY — nothing is phoned home; results go to stdout only.
  * DETERMINISTIC — savings and scores are a pure function of the inputs; only
                  the wall-clock timings vary run to run (they are measurements).
  * STDLIB ONLY  — no pip installs, no third-party modules (Python 3.8+).

Usage:
    python3 scripts/bench.py [--corpus DIR] [--bin PATH] [--json]
                             [--verbs web,optimize]

    --corpus DIR  Directory of source images (default: bench/corpus).
                  Point this at a REAL corpus for the SPEC-083 numbers — those
                  photos never need to enter git.
    --bin PATH    The crustyimg binary (default: target/release then debug).
    --json        Emit machine-readable JSON instead of the table.
    --verbs LIST  Comma-separated subset of {web,optimize} (default: both).
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
IMAGE_EXTS = {".jpg", ".jpeg", ".png", ".webp", ".avif", ".gif", ".bmp", ".tiff"}


def find_binary(explicit):
    """Resolve the crustyimg binary: an explicit --bin, else release, else debug."""
    if explicit:
        p = Path(explicit)
        if not p.is_file():
            sys.exit(f"bench: --bin not found: {p}")
        return str(p)
    for build in ("release", "debug"):
        cand = REPO_ROOT / "target" / build / "crustyimg"
        if cand.is_file():
            return str(cand)
    sys.exit("bench: no crustyimg binary; run `cargo build --release` or pass --bin")


def corpus_images(corpus_dir):
    """Sorted list of image files in the corpus (deterministic order)."""
    d = Path(corpus_dir)
    if not d.is_dir():
        sys.exit(f"bench: corpus dir not found: {d}")
    imgs = sorted(p for p in d.iterdir() if p.suffix.lower() in IMAGE_EXTS)
    if not imgs:
        sys.exit(f"bench: no images in corpus dir: {d}")
    return imgs


def run_one(binary, verb, image, out_dir):
    """Run one verb over one image, returning the parsed audit report dict.

    Uses `--json --timing` (and `--verify` for optimize, so its score is present
    like web's). Returns a normalized result row, or None on failure.
    """
    args = [binary, verb, str(image), "--out-dir", str(out_dir), "--json", "--timing"]
    if verb == "optimize":
        args.append("--verify")
    # Explicitly deny network access to the child by clearing proxy env; the binary
    # never opens a socket, this just makes the offline guarantee belt-and-braces.
    env = dict(os.environ)
    for k in ("http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"):
        env.pop(k, None)
    proc = subprocess.run(
        args, capture_output=True, text=True, env=env, cwd=str(REPO_ROOT)
    )
    if proc.returncode != 0:
        sys.stderr.write(
            f"bench: {verb} {image.name} failed (exit {proc.returncode}): "
            f"{proc.stderr.strip()}\n"
        )
        return None
    line = proc.stdout.strip().splitlines()[-1] if proc.stdout.strip() else ""
    try:
        report = json.loads(line)
    except json.JSONDecodeError as e:
        sys.stderr.write(f"bench: could not parse {verb} report for {image.name}: {e}\n")
        return None
    timing = report.get("timing", {}) or {}
    return {
        "image": image.name,
        "verb": verb,
        "source_format": report.get("source_format"),
        "source_bytes": report.get("source_bytes"),
        "out_bytes": report.get("out_bytes"),
        "savings_percent": report.get("savings_percent"),
        "ssim": report.get("ssim"),
        "total_ms": timing.get("total_ms"),
        "decode_ms": timing.get("decode_ms"),
        "encode_ms": timing.get("encode_ms"),
    }


def print_table(rows):
    header = f"{'image':<20} {'verb':<9} {'src B':>8} {'out B':>8} {'savings':>8} {'ssim':>6} {'total ms':>9}"
    print(header)
    print("-" * len(header))
    for r in rows:
        ssim = f"{r['ssim']:.1f}" if r.get("ssim") is not None else "-"
        total = f"{r['total_ms']:.1f}" if r.get("total_ms") is not None else "-"
        print(
            f"{r['image']:<20} {r['verb']:<9} {r['source_bytes']:>8} {r['out_bytes']:>8} "
            f"{r['savings_percent']:>7}% {ssim:>6} {total:>9}"
        )


def main():
    ap = argparse.ArgumentParser(description="crustyimg committed benchmark harness")
    ap.add_argument("--corpus", default=str(REPO_ROOT / "bench" / "corpus"))
    ap.add_argument("--bin", default=None)
    ap.add_argument("--json", action="store_true", dest="as_json")
    ap.add_argument("--verbs", default="web,optimize")
    args = ap.parse_args()

    verbs = [v.strip() for v in args.verbs.split(",") if v.strip()]
    for v in verbs:
        if v not in ("web", "optimize"):
            sys.exit(f"bench: unknown verb {v!r} (expected web/optimize)")

    binary = find_binary(args.bin)
    images = corpus_images(args.corpus)

    rows = []
    with tempfile.TemporaryDirectory(prefix="crustyimg-bench-") as tmp:
        for image in images:
            for verb in verbs:
                out_dir = Path(tmp) / f"{image.stem}-{verb}"
                out_dir.mkdir(parents=True, exist_ok=True)
                row = run_one(binary, verb, image, out_dir)
                if row is not None:
                    rows.append(row)

    if not rows:
        sys.exit("bench: no successful runs")

    if args.as_json:
        print(json.dumps({"corpus": str(args.corpus), "results": rows}, indent=2))
    else:
        print_table(rows)


if __name__ == "__main__":
    main()
