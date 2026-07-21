#!/usr/bin/env python3
"""crustyimg cross-tool benchmark harness (SPEC-083, DEC-080).

The companion to the crustyimg-only `scripts/bench.py` (SPEC-088, DEC-074): an
honest, EQUAL-QUALITY, reproducible comparison of `crustyimg` against the tools
people actually reach for — **sharp** (libvips), **ImageMagick**, **@squoosh/cli**,
and **cwebp** (WebP-only) — on **size, speed, and quality**, over a real photo
`--corpus`.

The method (fixed in DEC-080, before any number was read):

  * ONE PIPELINE for every tool — downscale the long edge to <= 2048 px (never
    upscale), then encode AVIF (cwebp does WebP, labelled). The harness ASSERTS
    this: every reference and every encoded output is measured, and a tool whose
    output long edge or aspect ratio departs from the source is flagged and the
    run exits non-zero. "Same pipeline for every tool" is a checkable claim, not
    a promise — the quality column cannot catch a distorted output, because a
    squashed image scored against its own squashed reference still scores fine.
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
    --q-from PATH     Reuse the quality each tool matched in a previous run's JSON
                      instead of sweeping the grid. Use it to re-time the same
                      encodes under different conditions (e.g. VIPS_CONCURRENCY=1)
                      so only the condition changes, not the encoder setting.
    --self-test       Check the dimension guard against known-good and known-bad
                      shapes and exit. No corpus, no tools needed.
    --json            Emit machine-readable JSON instead of the Markdown tables.
    --json-out PATH   Also write the JSON to PATH.

Exit codes: 0 ok · 2 usage/setup · 3 a tool's output failed the dimension check.

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
    extra process start vs a one-shot tool — noted in BENCHMARKS.md).
    """
    total = 0.0
    for cmd in cmds:
        rc, out, err, dt = sh(cmd, timed=True)
        total += dt
        if rc not in (0,):
            return rc, err, (total if timed else None)
    return 0, "", (total if timed else None)


class Plan:
    """The one downscale every tool must perform for a given source.

    Long edge to `cap`, aspect preserved, never upscaling — so the effective long
    edge is min(cap, source long edge). Tools disagree on how you say that: some
    take a long-edge number, some a bounding box, some a single axis. `Plan` holds
    the source shape so each tool can be told the same thing in its own dialect,
    and so the result can be checked against what actually came out.
    """

    def __init__(self, width, height, cap):
        self.src_w = width
        self.src_h = height
        self.cap = cap
        self.edge = min(cap, max(width, height))
        self.portrait = height > width

    @property
    def expected(self):
        """(long, short) edges the output should have, ignoring orientation."""
        scale = self.edge / max(self.src_w, self.src_h)
        return self.edge, round(min(self.src_w, self.src_h) * scale)


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
    def ref_pipeline(self, src, work, plan):
        raise NotImplementedError

    def enc_pipeline(self, src, work, plan, q):
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

    def ref_pipeline(self, src, work, plan):
        # `--max` is a long-edge bound, so one number covers both orientations.
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, "resize", src, "--max", str(plan.edge), "-o", ref, "-y"]], ref)

    def enc_pipeline(self, src, work, plan, q):
        png = os.path.join(work, "ds.png")
        out = os.path.join(work, f"q{q}.avif")
        return ([
            [self.bin, "resize", src, "--max", str(plan.edge), "-o", png, "-y"],
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

    def ref_pipeline(self, src, work, plan):
        # `web` bakes EXIF orientation, so the reference must be an auto-oriented
        # downscale to match web's AVIF dimensions (a plain `resize` would leave a
        # portrait photo transposed -> a dimension mismatch that scores as None).
        # `web -o ref.png` is the same downscale+orient pipeline, written lossless.
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, "web", src, "--max", str(plan.edge), "-o", ref, "-y"]], ref)

    def enc_pipeline(self, src, work, plan, q):
        # q is ignored: web picks its own fixed fast-AVIF quality.
        out = os.path.join(work, "web.avif")
        return ([[self.bin, "web", src, "--max", str(plan.edge),
                  "-o", out, "-y"]], out)


class Sharp(Tool):
    def __init__(self, sharp_bin, version):
        super().__init__("sharp", "avif", "avif",
                         grid=[50, 60, 70, 78, 85], higher_is_better=True,
                         available=True, version=version)
        self.bin = sharp_bin

    def _resize(self, plan):
        # sharp-cli's `resize <width> [height]` constrains only the axes it is
        # given: `resize 2048` alone caps the WIDTH, which leaves a portrait
        # source with a 3068 px long edge. Pass the full box and let `--fit
        # inside` bound the long edge in either orientation.
        return ["resize", str(plan.edge), str(plan.edge),
                "--fit", "inside", "--withoutEnlargement"]

    def ref_pipeline(self, src, work, plan):
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, "-i", src, "-o", ref, *self._resize(plan),
                  "-f", "png"]], ref)

    def enc_pipeline(self, src, work, plan, q):
        out = os.path.join(work, f"q{q}.avif")
        return ([[self.bin, "-i", src, "-o", out, *self._resize(plan),
                  "-f", "avif", "-q", str(q)]], out)


class ImageMagick(Tool):
    def __init__(self, magick_bin, version):
        super().__init__("imagemagick", "avif", "avif",
                         grid=[45, 55, 65, 72, 80], higher_is_better=True,
                         available=True, version=version)
        self.bin = magick_bin

    def _geometry(self, plan):
        # `WxH>` is a bounding box that only ever shrinks — correct in either
        # orientation.
        return f"{plan.edge}x{plan.edge}>"

    def ref_pipeline(self, src, work, plan):
        ref = os.path.join(work, "ref.png")
        return ([[self.bin, src, "-resize", self._geometry(plan), ref]], ref)

    def enc_pipeline(self, src, work, plan, q):
        out = os.path.join(work, f"q{q}.avif")
        return ([[self.bin, src, "-resize", self._geometry(plan),
                  "-quality", str(q), out]], out)


class Squoosh(Tool):
    """@squoosh/cli 0.7.2 — archived; runs only on Node < 18 (a finding)."""

    def __init__(self, node_bin, cli_js, version):
        super().__init__("squoosh", "avif", "avif",
                         grid=[23, 18, 14, 10, 6], higher_is_better=False,
                         available=True, version=version)
        self.node = node_bin
        self.cli = cli_js

    def _resize_json(self, plan):
        # squoosh has no "long edge" or "fit" mode. Given BOTH width and height it
        # stretches the image to exactly that box — a 6016x4016 photo comes back
        # 2048x2048, squashed. Given ONE axis it derives the other from the source
        # aspect, so constrain the long axis only. (No upscale guard either, which
        # is why the caller's plan caps the edge at the source long edge.)
        axis = "height" if plan.portrait else "width"
        return json.dumps({"enabled": True, axis: plan.edge, "method": "lanczos3"})

    def ref_pipeline(self, src, work, plan):
        d = os.path.join(work, "sqref")
        os.makedirs(d, exist_ok=True)
        stem = Path(src).stem
        return ([[self.node, self.cli, "--oxipng", '{"level":1}',
                  "--resize", self._resize_json(plan), "-d", d, src]],
                os.path.join(d, stem + ".png"))

    def enc_pipeline(self, src, work, plan, q):
        d = os.path.join(work, f"sq{q}")
        os.makedirs(d, exist_ok=True)
        stem = Path(src).stem
        return ([[self.node, self.cli, "--avif", json.dumps({"cqLevel": q}),
                  "--resize", self._resize_json(plan), "-d", d, src]],
                os.path.join(d, stem + ".avif"))


class Cwebp(Tool):
    """cwebp — WebP only, NO AVIF. Included as a labelled format-context point."""

    def __init__(self, cwebp_bin, version):
        super().__init__("cwebp", "webp", "webp",
                         grid=[78, 85, 90, 93, 96], higher_is_better=True,
                         available=True, version=version)
        self.bin = cwebp_bin

    def _resize(self, plan):
        # `-resize W H` treats a 0 as "derive from the other axis". Constraining
        # the width unconditionally leaves a portrait source oversized, so pin
        # whichever axis is actually the long one.
        return ["-resize", "0", str(plan.edge)] if plan.portrait else \
               ["-resize", str(plan.edge), "0"]

    def ref_pipeline(self, src, work, plan):
        # cwebp is an encoder, not a resizer with lossless PNG out; use lossless
        # WebP at the target size as the reference (same downscaler as its lossy
        # output, so the score isolates lossy encode fidelity).
        ref = os.path.join(work, "ref.webp")
        return ([[self.bin, "-lossless", *self._resize(plan), src, "-o", ref]], ref)

    def enc_pipeline(self, src, work, plan, q):
        out = os.path.join(work, f"q{q}.webp")
        return ([[self.bin, "-q", str(q), *self._resize(plan), src, "-o", out]], out)


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


def image_dims(crustyimg_bin, path):
    rc, out, err = sh([crustyimg_bin, "info", path, "--json"])
    try:
        d = json.loads(out)
        return d.get("width"), d.get("height")
    except Exception:
        return None, None


# --------------------------------------------------------------------------- #
# The dimension guard
# --------------------------------------------------------------------------- #
#
# The quality column cannot police the downscale. Every tool is scored against
# ITS OWN reference, so a tool that stretches the image to a square scores its
# squashed encode against its squashed reference and lands in the band like
# everyone else — which is exactly how @squoosh/cli ran distorted for a whole
# benchmark. Nothing else in the harness looks at output shape, so "same pipeline
# for every tool" was an unchecked claim. This is the check.
#
# Orientation-insensitive on purpose: `crustyimg web` bakes EXIF orientation, so
# its output can be the transpose of the source. That is a correct downscale, not
# a distortion, so compare long-vs-short rather than width-vs-width.

ASPECT_TOL = 0.01   # 1% — absorbs each tool's rounding (cwebp rounds a 1367 up to 1368)
EDGE_TOL = 1        # px


def check_dims(plan, out_w, out_h):
    """Did this output actually get the plan's downscale? -> (ok, reason)."""
    if not out_w or not out_h:
        return False, "output dimensions unreadable"
    exp_long, exp_short = plan.expected
    got_long, got_short = max(out_w, out_h), min(out_w, out_h)
    if abs(got_long - exp_long) > EDGE_TOL:
        return False, (f"long edge {got_long} px, expected {exp_long} "
                       f"(source {plan.src_w}x{plan.src_h}, got {out_w}x{out_h})")
    src_aspect = max(plan.src_w, plan.src_h) / min(plan.src_w, plan.src_h)
    got_aspect = got_long / got_short
    if abs(got_aspect - src_aspect) / src_aspect > ASPECT_TOL:
        return False, (f"aspect {got_aspect:.4f} vs source {src_aspect:.4f} "
                       f"({plan.src_w}x{plan.src_h} -> {out_w}x{out_h})")
    return True, None


def self_test():
    """Prove the guard can fail — including on the two shapes that shipped.

    A guard nobody has seen reject anything is not a guard.
    """
    cases = [
        # (label, src_w, src_h, cap, out_w, out_h, expect_ok)
        ("landscape, correct", 6016, 4016, 2048, 2048, 1367, True),
        ("landscape, squooshed to square", 6016, 4016, 2048, 2048, 2048, False),
        ("portrait, correct", 4016, 6016, 2048, 1367, 2048, True),
        ("portrait, long edge unconstrained", 4016, 6016, 2048, 2048, 3068, False),
        ("EXIF-rotated output (transposed)", 6016, 4016, 2048, 1367, 2048, True),
        ("square source", 2832, 2832, 2048, 2048, 2048, True),
        ("small source, not upscaled", 979, 734, 2048, 979, 734, True),
        ("cwebp rounding the short edge up", 6016, 4016, 2048, 2048, 1368, True),
    ]
    failures = 0
    for label, sw, sh_, cap, ow, oh, want_ok in cases:
        ok, why = check_dims(Plan(sw, sh_, cap), ow, oh)
        good = (ok == want_ok)
        failures += 0 if good else 1
        mark = "ok  " if good else "FAIL"
        print(f"{mark} {label}: {'accepted' if ok else 'rejected'}"
              f"{'' if ok else ' — ' + why}")
    if failures:
        print(f"\nself-test FAILED ({failures} case(s))")
        return 1
    print(f"\nself-test passed ({len(cases)} cases)")
    return 0


def bench_tool(tool, crustyimg_bin, src, mp, plan, args, forced_q=None):
    """Sweep the grid, pick nearest-target, time the winner. Returns a result dict."""
    if not tool.available:
        return {"tool": tool.name, "available": False, "why": tool.why}

    grid = tool.grid if forced_q is None else [forced_q]
    violations = []

    def dim_check(stage, path):
        ok, why = check_dims(plan, *image_dims(crustyimg_bin, path))
        if not ok:
            violations.append(f"{stage}: {why}")
        return ok

    with tempfile.TemporaryDirectory(prefix=f"bcmp-{tool.name}-") as work:
        # reference (own lossless downscale)
        ref_cmds, ref_path = tool.ref_pipeline(src, work, plan)
        rc, err, _ = run_pipeline(ref_cmds)
        if rc != 0 or not os.path.exists(ref_path):
            return {"tool": tool.name, "available": True, "error": f"ref failed: {err[:160]}"}
        dim_check("reference", ref_path)

        # grid sweep -> (q, bytes, score)
        grid_pts = []
        for q in grid:
            gwork = os.path.join(work, f"g{q}")
            os.makedirs(gwork, exist_ok=True)
            enc_cmds, out_path = tool.enc_pipeline(src, gwork, plan, q)
            rc, err, _ = run_pipeline(enc_cmds)
            if rc != 0 or not os.path.exists(out_path):
                continue
            dim_check(f"q={q}", out_path)
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
            enc_cmds, out_path = tool.enc_pipeline(src, twork, plan, best["q"])
            rc, err, dt = run_pipeline(enc_cmds, timed=True)
            if rc != 0:
                break
            if i >= args.warmup:
                times.append(dt)
        median_ms = round(statistics.median(times) * 1000, 1) if times else None

        res = {
            "tool": tool.name,
            "available": True,
            "kind": tool.kind,
            "version": tool.version,
            "q": best["q"],
            "q_source": "forced" if forced_q is not None else "grid",
            "matched_score": best["score"],
            "out_bytes": best["bytes"],
            "median_ms": median_ms,
            "grid": grid_pts,
        }
        if violations:
            res["dim_violations"] = violations
        return res


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
    L.append("| Photo | MP | Tool | Format | q | Matched score | Output | Savings | Median time |")
    L.append("|---|---:|---|---|---:|---:|---:|---:|---:|")
    for row in report["images"]:
        for r in row["results"]:
            if not r.get("available"):
                L.append(f"| {row['image']} | {row['mp']:.1f} | {r['tool']} | — | — | — | — | — | NOT RUN ({r.get('why','')}) |")
                continue
            if r.get("error"):
                L.append(f"| {row['image']} | {row['mp']:.1f} | {r['tool']} | — | — | — | — | — | ERROR |")
                continue
            sav = 1 - r["out_bytes"] / row["source_bytes"]
            flag = " ⚠ BAD DIMENSIONS" if r.get("dim_violations") else ""
            L.append(f"| {row['image']} | {row['mp']:.1f} | {r['tool']} | {r['kind'].upper()} "
                     f"| {r['q']} | {r['matched_score']:.1f} | {fmt_bytes(r['out_bytes'])} "
                     f"| {sav*100:.1f}% | {r['median_ms']:.0f} ms{flag} |")
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

    bad = report.get("dimension_violations") or []
    L.append(f"_Dimension check: {'PASSED' if not bad else 'FAILED'} — "
             f"every reference and every encoded output measured against the "
             f"source long edge and aspect ratio._")
    for v in bad:
        L.append(f"- ⚠ {v['image']} / {v['tool']}: {'; '.join(v['violations'])}")
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
    ap.add_argument("--corpus", default=None)
    ap.add_argument("--bin", default=None)
    ap.add_argument("--tools-dir", default=None)
    ap.add_argument("--squoosh-node", default=None)
    ap.add_argument("--target", type=float, default=82.0)
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--warmup", type=int, default=1)
    ap.add_argument("--max-edge", type=int, default=2048)
    ap.add_argument("--tools", default=None)
    ap.add_argument("--q-from", default=None, dest="q_from")
    ap.add_argument("--self-test", action="store_true", dest="self_test")
    ap.add_argument("--json", action="store_true", dest="as_json")
    ap.add_argument("--json-out", default=None)
    args = ap.parse_args()

    if args.self_test:
        sys.exit(self_test())

    if not args.corpus:
        sys.exit("bench-compare: --corpus is required (or --self-test)")
    corpus = Path(args.corpus)
    if not corpus.is_dir():
        sys.exit(f"bench-compare: corpus dir not found: {corpus}")
    images = sorted(p for p in corpus.iterdir() if p.suffix.lower() in IMAGE_EXTS)
    if not images:
        sys.exit(f"bench-compare: no images (.jpg/.jpeg/.png) in {corpus}")

    # --q-from: reuse a previous run's matched quality per (image, tool), so a
    # re-run under different conditions changes only the conditions.
    forced_q = {}
    if args.q_from:
        try:
            prev = json.loads(Path(args.q_from).read_text())
        except Exception as e:
            sys.exit(f"bench-compare: --q-from unreadable: {e}")
        for row in prev.get("images", []):
            for r in row.get("results", []):
                if r.get("available") and not r.get("error"):
                    forced_q[(row["image"], r["tool"])] = r["q"]
        sys.stderr.write(f"bench-compare: quality held fixed from {args.q_from} "
                         f"({len(forced_q)} points)\n")

    crustyimg_bin = find_crustyimg(args.bin)
    rc, ver_out, _ = sh([crustyimg_bin, "--version"])
    ci_version = ver_out.strip() or "crustyimg ?"

    tools = discover_tools(args, crustyimg_bin, ci_version)
    tool_order = list(tools.keys())

    sys.stderr.write(f"bench-compare: {len(images)} photos, tools: {', '.join(tool_order)}\n")

    image_rows = []
    violations = []
    for src in images:
        w, h = image_dims(crustyimg_bin, str(src))
        if not w or not h:
            sys.stderr.write(f"  skip {src.name}: could not read dimensions\n")
            continue
        mp = (w * h) / 1e6
        plan = Plan(w, h, args.max_edge)
        sys.stderr.write(f"  {src.name}  {mp:.1f} MP  ({w}x{h}) -> long edge {plan.edge}\n")
        results = []
        for tname in tool_order:
            res = bench_tool(tools[tname], crustyimg_bin, str(src), mp, plan, args,
                             forced_q=forced_q.get((src.name, tname)))
            results.append(res)
            if res.get("available") and not res.get("error"):
                sys.stderr.write(f"    {tname:<13} score={res.get('matched_score')} "
                                 f"bytes={res.get('out_bytes')} q={res.get('q')} "
                                 f"t={res.get('median_ms')}ms\n")
            else:
                sys.stderr.write(f"    {tname:<13} {res.get('why') or res.get('error')}\n")
            if res.get("dim_violations"):
                violations.append({"image": src.name, "tool": tname,
                                   "violations": res["dim_violations"]})
                for v in res["dim_violations"]:
                    sys.stderr.write(f"    {tname:<13} ⚠ DIMENSION CHECK FAILED — {v}\n")
        image_rows.append({
            "image": src.name, "mp": round(mp, 2), "width": w, "height": h,
            "source_bytes": src.stat().st_size, "long_edge": plan.edge, "results": results,
        })

    report = {
        "corpus": str(corpus),
        "target": args.target,
        "runs": args.runs,
        "warmup": args.warmup,
        "max_edge": args.max_edge,
        "q_from": args.q_from,
        "machine": machine_context(),
        "tool_order": tool_order,
        "tool_versions": [
            {"name": t.name, "available": t.available, "why": t.why, "version": t.version}
            for t in tools.values()
        ],
        "images": image_rows,
        "dimension_violations": violations,
    }

    if args.json_out:
        Path(args.json_out).write_text(json.dumps(report, indent=2))
    if args.as_json:
        print(json.dumps(report, indent=2))
    else:
        print(render_markdown(report))

    if violations:
        sys.stderr.write(
            f"\nbench-compare: DIMENSION CHECK FAILED — {len(violations)} tool/photo "
            f"pair(s) did not get the downscale every other tool got. These numbers "
            f"are not comparable; fix the tool's resize arguments before publishing "
            f"anything from this run.\n")
        sys.exit(3)


if __name__ == "__main__":
    main()
