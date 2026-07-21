#!/usr/bin/env python3
"""crustyimg cross-tool benchmark harness (SPEC-083, DEC-080).

The companion to the crustyimg-only `scripts/bench.py` (SPEC-088, DEC-074): an
honest, EQUAL-QUALITY, reproducible comparison of `crustyimg` against the tools
people actually reach for — **sharp** (libvips), **ImageMagick**, **@squoosh/cli**,
and **cwebp** (WebP-only) — on **size, speed, and quality**, over a real photo
`--corpus`.

The method (fixed in DEC-080, before any number was read):

  * ONE PIPELINE for every tool — downscale the long edge to <= 2048 px (never
    upscale), then encode AVIF (cwebp does WebP, labelled).
  * ONE SCORER — `crustyimg diff A B` (SSIMULACRA2). Every tool's output is
    scored against *that tool's own* lossless 2048 px downscale, so the quality
    column is encode fidelity, resampler-neutral, and identical in kind to the
    number `crustyimg web` reports. (`crustyimg diff` requires equal dimensions,
    which own-reference scoring guarantees.)
  * ISO-QUALITY — each tool is swept over a fixed quality grid and the harness
    picks the grid point whose score is NEAREST the target band (default 82,
    "high"). "Smallest file" is meaningless without equal quality; this makes the
    comparison a fair one. The matched score is printed, so a reader sees where
    each tool actually landed.
  * NO HAND-EDITED NUMBERS — the table is emitted from measured runs. Scores and
    bytes are deterministic; only wall-times vary (they are measurements). Two
    runs match within timing noise.

The crustyimg side is the shipped 0.5.0 engine built `--features avif` (the
flagship AVIF path is a pure-Rust opt-in; `cargo install crustyimg --features
avif`). Competitor versions are pinned by whatever is installed and recorded in
the output header, so the doc's commands are reproducible.

Usage:
    python3 scripts/bench-compare.py --corpus DIR [options]

    --corpus DIR      Directory of real source photos (REQUIRED for real numbers;
                      those photos never enter git — see DEC-074).
    --bin PATH        crustyimg binary built --features avif (default: target/release).
    --tools-dir DIR   node_modules dir holding sharp-cli + @squoosh/cli
                      (default: probes a few known locations).
    --squoosh-node P  Node binary for @squoosh/cli (it is archived and needs
                      Node < 18). Default: $SQUOOSH_NODE, else skip squoosh.
    --target N        SSIMULACRA2 band centre to match (default 82).
    --runs N          Timed repeats per selected point (median reported, default 3).
    --warmup N        Untimed warmup runs (default 1).
    --tools LIST      Comma-separated subset of {crustyimg,crustyimg-web,sharp,
                      imagemagick,squoosh,cwebp} (default: all available).
    --json            Emit machine-readable JSON instead of the Markdown tables.
    --json-out PATH   Also write the JSON to PATH.

Stdlib only (json, subprocess, argparse, time, ...); no pip installs; offline.
"""

import argparse
import json
import os
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
IMAGE_EXTS = {".jpg", ".jpeg", ".png"}

# Size buckets by source megapixels (DEC-080).
BUCKETS = [("small", 0.0, 2.0), ("medium", 2.0, 12.0), ("large", 12.0, 1e9)]


def sh(cmd, timed=False):
    """Run a command list. Returns (returncode, stdout, stderr[, elapsed_s])."""
    t0 = time.perf_counter()
    p = subprocess.run(cmd, capture_output=True, text=True)
    dt = time.perf_counter() - t0
    if timed:
        return p.returncode, p.stdout, p.stderr, dt
    return p.returncode, p.stdout, p.stderr


def run_pipeline(cmds, timed=False):
    """Run a sequence of commands; on timed runs return the SUMMED wall time.

    A tool whose downscale+encode is one process has a single-command pipeline;
    crustyimg's grid path is resize + convert, so its time sums both steps (an
    extra process start vs a one-shot tool — disclosed in DEC-080/BENCHMARKS.md).
    """
    total = 0.0
    for cmd in cmds:
        rc, out, err, dt = sh(cmd, timed=True)
        total += dt
        if rc not in (0,):
            return rc, err, (total if timed else None)
    return 0, "", (total if timed else None)


class Tool:
    """A benchmarked tool: how it downscales (for scoring) and encodes (timed)."""

    def __init__(self, name, kind, fmt, grid, higher_is_better, available, why=None,
                 version=None):
        self.name = name
        self.kind = kind          # 'avif' | 'webp'
        self.fmt = fmt            # output extension
        self.grid = grid
        self.higher_is_better = higher_is_better
        self.available = available
        self.why = why            # reason if unavailable
        self.version = version or "?"

    # Each returns (list_of_cmds, output_path). Subclasses override.
    def ref_pipeline(self, src, work, max_edge):
        raise NotImplementedError

    def enc_pipeline(self, src, work, max_edge, q):
        raise NotImplementedError


# --------------------------------------------------------------------------- #
# Tool definitions
# --------------------------------------------------------------------------- #

class Crustyimg(Tool):
    def __init__(self, binary, version):
        super().__init__("crustyimg", "avif", "avif",
                         grid=[80, 85, 88, 90, 92, 94], higher_is_better=True,
                         available=True, version=version)
        self.bin = binary

    def ref_pipeline(self, src, work, max_edge):
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, "resize", src, "--max", str(max_edge), "-o", ref, "-y"]], ref)

    def enc_pipeline(self, src, work, max_edge, q):
        png = os.path.join(work, "ds.png")
        out = os.path.join(work, f"q{q}.avif")
        return ([
            [self.bin, "resize", src, "--max", str(max_edge), "-o", png, "-y"],
            [self.bin, "convert", png, "--format", "avif", "-q", str(q), "-o", out, "-y"],
        ], out)


class CrustyimgWeb(Tool):
    """The real one-command flagship: `crustyimg web` (downscale + fast-AVIF,
    fixed quality). Not grid-swept — its fixed operating point is the point."""

    def __init__(self, binary, version):
        super().__init__("crustyimg-web", "avif", "avif",
                         grid=[None], higher_is_better=True,
                         available=True, version=version)
        self.bin = binary

    def ref_pipeline(self, src, work, max_edge):
        # `web` bakes EXIF orientation, so the reference must be an auto-oriented
        # downscale to match web's AVIF dimensions (a plain `resize` would leave a
        # portrait photo transposed -> a dimension mismatch that scores as None).
        # `web -o ref.png` is the same downscale+orient pipeline, written lossless.
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, "web", src, "--max", str(max_edge), "-o", ref, "-y"]], ref)

    def enc_pipeline(self, src, work, max_edge, q):
        # q is ignored: web picks its own fixed fast-AVIF quality.
        out = os.path.join(work, "web.avif")
        return ([[self.bin, "web", src, "--max", str(max_edge),
                  "-o", out, "-y"]], out)


class Sharp(Tool):
    def __init__(self, sharp_bin, version):
        super().__init__("sharp", "avif", "avif",
                         grid=[50, 60, 70, 78, 85], higher_is_better=True,
                         available=True, version=version)
        self.bin = sharp_bin

    def _resize(self, max_edge):
        return ["resize", str(max_edge), "--fit", "inside", "--withoutEnlargement"]

    def ref_pipeline(self, src, work, max_edge):
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, "-i", src, "-o", ref, *self._resize(max_edge),
                  "-f", "png"]], ref)

    def enc_pipeline(self, src, work, max_edge, q):
        out = os.path.join(work, f"q{q}.avif")
        return ([[self.bin, "-i", src, "-o", out, *self._resize(max_edge),
                  "-f", "avif", "-q", str(q)]], out)


class ImageMagick(Tool):
    def __init__(self, magick_bin, version):
        super().__init__("imagemagick", "avif", "avif",
                         grid=[45, 55, 65, 72, 80], higher_is_better=True,
                         available=True, version=version)
        self.bin = magick_bin

    def ref_pipeline(self, src, work, max_edge):
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, src, "-resize", f"{max_edge}x{max_edge}>", ref]], ref)

    def enc_pipeline(self, src, work, max_edge, q):
        out = os.path.join(work, f"q{q}.avif")
        return ([[self.bin, src, "-resize", f"{max_edge}x{max_edge}>",
                  "-quality", str(q), out]], out)


class Squoosh(Tool):
    """@squoosh/cli 0.7.2 — archived; runs only on Node < 18 (a finding)."""

    def __init__(self, node_bin, cli_js, version):
        super().__init__("squoosh", "avif", "avif",
                         grid=[23, 18, 14, 10, 6], higher_is_better=False,
                         available=True, version=version)
        self.node = node_bin
        self.cli = cli_js

    def _resize_json(self, max_edge):
        # squoosh has no "long edge" mode; pass both W and H caps + no upscale is
        # NOT built in, so we skip resize when the source already fits (handled by
        # the caller passing max_edge = source long edge when it is <= 2048).
        return json.dumps({"enabled": True, "width": max_edge, "height": max_edge,
                           "method": "lanczos3"})

    def ref_pipeline(self, src, work, max_edge):
        d = os.path.join(work, "sqref")
        os.makedirs(d, exist_ok=True)
        stem = Path(src).stem
        return ([[self.node, self.cli, "--oxipng", '{"level":1}',
                  "--resize", self._resize_json(max_edge), "-d", d, src]],
                os.path.join(d, stem + ".png"))

    def enc_pipeline(self, src, work, max_edge, q):
        d = os.path.join(work, f"sq{q}")
        os.makedirs(d, exist_ok=True)
        stem = Path(src).stem
        return ([[self.node, self.cli, "--avif", json.dumps({"cqLevel": q}),
                  "--resize", self._resize_json(max_edge), "-d", d, src]],
                os.path.join(d, stem + ".avif"))


class Cwebp(Tool):
    """cwebp — WebP only, NO AVIF. Included as a labelled format-context point."""

    def __init__(self, cwebp_bin, version):
        super().__init__("cwebp", "webp", "webp",
                         grid=[78, 85, 90, 93, 96], higher_is_better=True,
                         available=True, version=version)
        self.bin = cwebp_bin

    def ref_pipeline(self, src, work, max_edge):
        # cwebp is an encoder, not a resizer with lossless PNG out; use lossless
        # WebP at the target size as the reference (same downscaler as its lossy
        # output, so the score isolates lossy encode fidelity).
        ref = os.path.join(work, "ref.webp")
        return ([[self.bin, "-lossless", "-resize", str(max_edge), "0", src,
                  "-o", ref]], ref)

    def enc_pipeline(self, src, work, max_edge, q):
        out = os.path.join(work, f"q{q}.webp")
        return ([[self.bin, "-q", str(q), "-resize", str(max_edge), "0", src,
                  "-o", out]], out)


# --------------------------------------------------------------------------- #
# Discovery
# --------------------------------------------------------------------------- #

def find_crustyimg(explicit):
    if explicit:
        p = Path(explicit)
        if not p.is_file():
            sys.exit(f"bench-compare: --bin not found: {p}")
        return str(p)
    for build in ("release", "debug"):
        cand = REPO_ROOT / "target" / build / "crustyimg"
        if cand.is_file():
            return str(cand)
    sys.exit("bench-compare: no crustyimg binary; `cargo build --release --features avif`")


def tool_version(cmd):
    try:
        rc, out, err = sh(cmd)
        return (out + err).strip().splitlines()[0] if (out or err) else "?"
    except Exception:
        return "?"


def resolve_node_bin(node_bin):
    """Find a node<18 for squoosh: explicit path, $SQUOOSH_NODE, else nvm 16/14."""
    if node_bin:
        return node_bin if Path(node_bin).is_file() else None
    env = os.environ.get("SQUOOSH_NODE")
    if env and Path(env).is_file():
        return env
    for ver in ("16", "14", "18"):
        base = Path.home() / ".nvm" / "versions" / "node"
        if base.is_dir():
            hits = sorted(base.glob(f"v{ver}.*/bin/node"))
            if hits:
                return str(hits[-1])
    return None


def discover_tools(args, crustyimg_bin, ci_version):
    tools = {}
    # crustyimg (grid) + crustyimg-web (flagship one-command)
    tools["crustyimg"] = Crustyimg(crustyimg_bin, ci_version)
    tools["crustyimg-web"] = CrustyimgWeb(crustyimg_bin, ci_version)

    # node tools dir
    tools_dir = args.tools_dir
    if not tools_dir:
        for cand in (
            os.environ.get("BENCH_TOOLS_DIR"),
            str(REPO_ROOT / "bench" / "tools"),
        ):
            if cand and Path(cand, "node_modules").is_dir():
                tools_dir = cand
                break
    bin_dir = Path(tools_dir, "node_modules", ".bin") if tools_dir else None

    # sharp-cli
    sharp = shutil.which("sharp") or (str(bin_dir / "sharp") if bin_dir and (bin_dir / "sharp").exists() else None)
    if sharp:
        tools["sharp"] = Sharp(sharp, tool_version([sharp, "--version"]))
    else:
        tools["sharp"] = Sharp("sharp", "not found")
        tools["sharp"].available, tools["sharp"].why = False, "sharp-cli not installed"

    # imagemagick
    magick = shutil.which("magick") or shutil.which("convert")
    if magick:
        v = tool_version([magick, "--version"])
        tools["imagemagick"] = ImageMagick(magick, v.replace("Version: ", ""))
    else:
        t = ImageMagick("magick", "not found"); t.available, t.why = False, "ImageMagick not installed"
        tools["imagemagick"] = t

    # squoosh (needs node<18)
    cli_js = None
    if tools_dir:
        cand = Path(tools_dir, "node_modules", "@squoosh", "cli", "src", "index.js")
        if cand.is_file():
            cli_js = str(cand)
    node_bin = resolve_node_bin(args.squoosh_node)
    if cli_js and node_bin:
        # read version from package.json
        ver = "?"
        try:
            pj = Path(tools_dir, "node_modules", "@squoosh", "cli", "package.json")
            ver = json.loads(pj.read_text()).get("version", "?")
        except Exception:
            pass
        tools["squoosh"] = Squoosh(node_bin, cli_js, f"@squoosh/cli {ver} (node {tool_version([node_bin, '--version'])})")
    else:
        t = Squoosh("node", "", "not found")
        t.available = False
        t.why = ("@squoosh/cli not installed" if not cli_js
                 else "no Node < 18 for @squoosh/cli (set --squoosh-node / $SQUOOSH_NODE)")
        tools["squoosh"] = t

    # cwebp
    cwebp = shutil.which("cwebp")
    if cwebp:
        tools["cwebp"] = Cwebp(cwebp, tool_version([cwebp, "-version"]))
    else:
        t = Cwebp("cwebp", "not found"); t.available, t.why = False, "cwebp not installed"
        tools["cwebp"] = t

    if args.tools:
        want = {t.strip() for t in args.tools.split(",") if t.strip()}
        tools = {k: v for k, v in tools.items() if k in want}
    return tools


# --------------------------------------------------------------------------- #
# Measurement
# --------------------------------------------------------------------------- #

def score_of(crustyimg_bin, ref, out):
    if not (os.path.exists(ref) and os.path.exists(out)):
        return None
    rc, so, se = sh([crustyimg_bin, "diff", ref, out, "--json"])
    if rc not in (0, 7):
        return None
    try:
        return round(json.loads(so.strip().splitlines()[-1])["score"], 2)
    except Exception:
        return None


def image_megapixels(crustyimg_bin, src):
    rc, out, err = sh([crustyimg_bin, "info", src, "--json"])
    try:
        d = json.loads(out)
        return (d.get("width", 0) * d.get("height", 0)) / 1e6, d.get("width"), d.get("height")
    except Exception:
        return None, None, None


def target_edge(width, height, cap=2048):
    """The pipeline downscales the long edge to <= cap and never upscales, so the
    effective long edge is min(cap, source long edge). Passing that to a tool with
    no 'no-upscale' mode (squoosh) keeps it from enlarging small sources."""
    return min(cap, max(width, height))


def bench_tool(tool, crustyimg_bin, src, mp, edge, args):
    """Sweep the grid, pick nearest-target, time the winner. Returns a result dict."""
    if not tool.available:
        return {"tool": tool.name, "available": False, "why": tool.why}

    with tempfile.TemporaryDirectory(prefix=f"bcmp-{tool.name}-") as work:
        # reference (own lossless downscale)
        ref_cmds, ref_path = tool.ref_pipeline(src, work, edge)
        rc, err, _ = run_pipeline(ref_cmds)
        if rc != 0 or not os.path.exists(ref_path):
            return {"tool": tool.name, "available": True, "error": f"ref failed: {err[:160]}"}

        # grid sweep -> (q, bytes, score)
        grid_pts = []
        for q in tool.grid:
            gwork = os.path.join(work, f"g{q}")
            os.makedirs(gwork, exist_ok=True)
            enc_cmds, out_path = tool.enc_pipeline(src, gwork, edge, q)
            rc, err, _ = run_pipeline(enc_cmds)
            if rc != 0 or not os.path.exists(out_path):
                continue
            sc = score_of(crustyimg_bin, ref_path, out_path)
            if sc is None:
                continue
            grid_pts.append({"q": q, "bytes": os.path.getsize(out_path), "score": sc})

        if not grid_pts:
            return {"tool": tool.name, "available": True, "error": "no grid point produced a scorable output"}

        # pick the point whose score is nearest the target band centre (tie -> smaller)
        target = args.target
        best = min(grid_pts, key=lambda p: (abs(p["score"] - target), p["bytes"]))

        # time the winning pipeline: warmup then N timed, report median
        times = []
        for i in range(args.warmup + args.runs):
            twork = os.path.join(work, f"t{i}")
            os.makedirs(twork, exist_ok=True)
            enc_cmds, out_path = tool.enc_pipeline(src, twork, edge, best["q"])
            rc, err, dt = run_pipeline(enc_cmds, timed=True)
            if rc != 0:
                break
            if i >= args.warmup:
                times.append(dt)
        median_ms = round(statistics.median(times) * 1000, 1) if times else None

        return {
            "tool": tool.name,
            "available": True,
            "kind": tool.kind,
            "version": tool.version,
            "q": best["q"],
            "matched_score": best["score"],
            "out_bytes": best["bytes"],
            "median_ms": median_ms,
            "grid": grid_pts,
        }


# --------------------------------------------------------------------------- #
# Reporting
# --------------------------------------------------------------------------- #

def bucket_of(mp):
    for name, lo, hi in BUCKETS:
        if lo <= mp < hi:
            return name
    return "large"


def fmt_bytes(n):
    # Decimal KB/MB (÷1000), matching macOS Finder and BENCHMARKS.md.
    if n is None:
        return "-"
    if n >= 1_000_000:
        return f"{n / 1_000_000:.2f} MB"
    return f"{round(n / 1000)} KB"


def render_markdown(report):
    L = []
    m = report["machine"]
    L.append(f"_Machine: {m['cpu']}, {m['cores']} cores, {m['os']}. "
             f"Scorer: crustyimg diff (SSIMULACRA2), target band {report['target']}. "
             f"Times = median of {report['runs']} runs (warmup {report['warmup']})._")
    L.append("")
    L.append("**Tools**")
    for t in report["tool_versions"]:
        status = t["version"] if t["available"] else f"NOT RUN — {t['why']}"
        L.append(f"- `{t['name']}` — {status}")
    L.append("")

    # per-image table
    L.append("### Per-image (web-ready: downscale ≤2048px long edge + AVIF, matched quality)")
    L.append("")
    L.append("| Photo | MP | Tool | Format | Matched score | Output | Savings | Median time |")
    L.append("|---|---:|---|---|---:|---:|---:|---:|")
    for row in report["images"]:
        for r in row["results"]:
            if not r.get("available"):
                L.append(f"| {row['image']} | {row['mp']:.1f} | {r['tool']} | — | — | — | — | NOT RUN ({r.get('why','')}) |")
                continue
            if r.get("error"):
                L.append(f"| {row['image']} | {row['mp']:.1f} | {r['tool']} | — | — | — | — | ERROR |")
                continue
            sav = 1 - r["out_bytes"] / row["source_bytes"]
            L.append(f"| {row['image']} | {row['mp']:.1f} | {r['tool']} | {r['kind'].upper()} "
                     f"| {r['matched_score']:.1f} | {fmt_bytes(r['out_bytes'])} "
                     f"| {sav*100:.1f}% | {r['median_ms']:.0f} ms |")
    L.append("")

    # per-bucket aggregate
    L.append("### By size bucket (median across photos in the bucket)")
    L.append("")
    L.append("| Bucket | Tool | Format | Median matched score | Median output | Median savings | Median time |")
    L.append("|---|---|---|---:|---:|---:|---:|")
    for bname, _, _ in BUCKETS:
        for tname in report["tool_order"]:
            pts = []
            for row in report["images"]:
                if bucket_of(row["mp"]) != bname:
                    continue
                for r in row["results"]:
                    if r["tool"] == tname and r.get("available") and not r.get("error"):
                        pts.append((r["matched_score"], r["out_bytes"],
                                    1 - r["out_bytes"] / row["source_bytes"], r["median_ms"], r["kind"]))
            if not pts:
                continue
            msc = statistics.median([p[0] for p in pts])
            mby = statistics.median([p[1] for p in pts])
            msa = statistics.median([p[2] for p in pts])
            mms = statistics.median([p[3] for p in pts])
            kind = pts[0][4]
            L.append(f"| {bname} | {tname} | {kind.upper()} | {msc:.1f} | {fmt_bytes(int(mby))} "
                     f"| {msa*100:.1f}% | {mms:.0f} ms |")
    L.append("")
    return "\n".join(L)


def machine_context():
    def s(cmd):
        try:
            return subprocess.run(cmd, capture_output=True, text=True).stdout.strip()
        except Exception:
            return "?"
    cpu = s(["sysctl", "-n", "machdep.cpu.brand_string"]) or "?"
    cores = s(["sysctl", "-n", "hw.logicalcpu"]) or "?"
    osv = "?"
    sw = s(["sw_vers", "-productVersion"])
    if sw:
        osv = f"macOS {sw}"
    return {"cpu": cpu, "cores": cores, "os": osv}


def main():
    ap = argparse.ArgumentParser(description="crustyimg cross-tool benchmark harness")
    ap.add_argument("--corpus", required=True)
    ap.add_argument("--bin", default=None)
    ap.add_argument("--tools-dir", default=None)
    ap.add_argument("--squoosh-node", default=None)
    ap.add_argument("--target", type=float, default=82.0)
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--warmup", type=int, default=1)
    ap.add_argument("--max-edge", type=int, default=2048)
    ap.add_argument("--tools", default=None)
    ap.add_argument("--json", action="store_true", dest="as_json")
    ap.add_argument("--json-out", default=None)
    args = ap.parse_args()

    corpus = Path(args.corpus)
    if not corpus.is_dir():
        sys.exit(f"bench-compare: corpus dir not found: {corpus}")
    images = sorted(p for p in corpus.iterdir() if p.suffix.lower() in IMAGE_EXTS)
    if not images:
        sys.exit(f"bench-compare: no images (.jpg/.jpeg/.png) in {corpus}")

    crustyimg_bin = find_crustyimg(args.bin)
    rc, ver_out, _ = sh([crustyimg_bin, "--version"])
    ci_version = ver_out.strip() or "crustyimg ?"

    tools = discover_tools(args, crustyimg_bin, ci_version)
    tool_order = list(tools.keys())

    sys.stderr.write(f"bench-compare: {len(images)} photos, tools: {', '.join(tool_order)}\n")

    image_rows = []
    for src in images:
        mp, w, h = image_megapixels(crustyimg_bin, str(src))
        if mp is None:
            sys.stderr.write(f"  skip {src.name}: could not read dimensions\n")
            continue
        edge = target_edge(w, h, args.max_edge)
        sys.stderr.write(f"  {src.name}  {mp:.1f} MP  ({w}x{h}) -> long edge {edge}\n")
        results = []
        for tname in tool_order:
            res = bench_tool(tools[tname], crustyimg_bin, str(src), mp, edge, args)
            results.append(res)
            if res.get("available") and not res.get("error"):
                sys.stderr.write(f"    {tname:<13} score={res.get('matched_score')} "
                                 f"bytes={res.get('out_bytes')} q={res.get('q')} "
                                 f"t={res.get('median_ms')}ms\n")
            else:
                sys.stderr.write(f"    {tname:<13} {res.get('why') or res.get('error')}\n")
        image_rows.append({
            "image": src.name, "mp": round(mp, 2), "width": w, "height": h,
            "source_bytes": src.stat().st_size, "long_edge": edge, "results": results,
        })

    report = {
        "corpus": str(corpus),
        "target": args.target,
        "runs": args.runs,
        "warmup": args.warmup,
        "max_edge": args.max_edge,
        "machine": machine_context(),
        "tool_order": tool_order,
        "tool_versions": [
            {"name": t.name, "available": t.available, "why": t.why, "version": t.version}
            for t in tools.values()
        ],
        "images": image_rows,
    }

    if args.json_out:
        Path(args.json_out).write_text(json.dumps(report, indent=2))
    if args.as_json:
        print(json.dumps(report, indent=2))
    else:
        print(render_markdown(report))


if __name__ == "__main__":
    main()
