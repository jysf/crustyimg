#!/usr/bin/env python3
"""SPEC-123 — is crustyimg's AVIF output byte-deterministic across thread counts?

⚠ **SUPERSEDED AS A LIVE HARNESS BY SPEC-124 (DEC-096), 2026-08-21. Read this
file as a record of what was true before the pin, not as a check you can run.**

Two things below are now false, and one of them fails silently:

  * The present-tense claim under "What sets the thread count" — *"crustyimg
    never calls `with_num_threads`"* — is **no longer true**. Both AVIF encode
    arms now pass `with_num_threads(Some(AVIF_TILE_THREADS))` with
    `AVIF_TILE_THREADS = 1`. (The `src/sink/mod.rs:679` line reference is also
    stale; the site moved.)
  * ⚡ **Leg E, this harness's positive control, is DEAD.** It asserts *"if the
    shipped null is real rather than a broken harness, this leg must show the
    bytes moving"* — but leg E's probe is built from the same source, so it is
    pinned too, and the bytes can no longer move. **Re-running this harness now
    reports a green whose control cannot fail**, which is exactly the failure
    mode the null was designed to rule out. Legs F and G's stated
    interpretations rest on leg E and are void for the same reason.

The live check for the pinned behaviour is `tests/avif_tile_pin.rs`, which
builds its own `--features image/rayon` probe. To make THIS harness discriminate
again, its probe would have to be built from a source tree with the pin removed.

Three shipped things assume it is: `build --frozen`, the lockfile's `hash`
(`src/build/lock.rs:32-37`), and the DEC-058 cache key (`src/cli/build.rs:294`).
Thread count is in none of their qualifying lists. This harness measures it and
reports hashes, not a verdict.

## What sets the thread count

crustyimg never calls `with_num_threads` — `src/sink/mod.rs:679` constructs
`AvifEncoder::new_with_speed_quality(..)` and nothing else — so `ravif` resolves
the count itself (`av1encoder.rs:653`):

    let threads = p.threads.unwrap_or_else(rayon::current_num_threads);
    threads.min((p.width * p.height) / (p.speed.min_tile_size as usize).pow(2))

That value is the AV1 **tile count**. Tile boundaries reset entropy-coding
contexts, so a different tile count is a different bitstream by construction.

⚠ Which `rayon` that is depends on a Cargo feature. `ravif`'s `threading`
feature is reached only through `image`'s `rayon` feature (`image` Cargo.toml:
`rayon = ["dep:rayon", "ravif?/threading", ...]`), and crustyimg enables
`avif = ["image/avif"]` only (`Cargo.toml:139`, `[features]`). With `threading`
OFF, `ravif` substitutes its own shim (`lib.rs:33`):

    mod rayoff {
        pub fn current_num_threads() -> usize {
            std::thread::available_parallelism().map(|v| v.get()).unwrap_or(1)
        }
        pub fn join<A, B>(a: .., b: ..) -> (A, B) { (a(), b()) }   // sequential
    }

So on the shipped build the count is **the machine's core count**, read from the
OS — not `RAYON_NUM_THREADS`, not `--jobs` — and the encode itself is serial.

## The legs

  A  shipped binary × {convert, web, optimize} × {photo, graphic}
     × RAYON_NUM_THREADS ∈ threads          → sha256, bytes, wall, cpu
  A2 shipped binary, the AUTO decision path (no `--format` pin): `web` and
     `optimize` pick AVIF themselves and encode at 85, not 80. Without this the
     matrix is three verbs making one identical encoder call.
  B  shipped `optimize --jobs N`, batch size 1 (the scoped-pool lever;
     `src/cli/optimize.rs:177`). ⚠ `--jobs` is inert on `convert` — six serial
     verbs ignore it (STAGE-042) — so it is a confirming leg on `optimize` only.
  C  run-to-run stability at one fixed thread count, `--repeats` runs (AC-4).
  D  the lean build (`--no-default-features`): AVIF encode is not compiled in.
     What it does instead is part of the answer.
  E  **positive control** — a probe binary built with `--features image/rayon`,
     i.e. `ravif/threading` ON, run over the same matrix. If the shipped null is
     real rather than a broken harness, this leg must show the bytes moving.
  F  cross-check: probe at RAYON_NUM_THREADS = core count vs the shipped bytes.
     Byte equality pins the shipped tile count to `available_parallelism()`.
  G  **the clamp trap, demonstrated** — `min_tile_size` doubles to 256 below
     quality 80 (`ravif` `high_quality` gate, `av1encoder.rs:544/584`), which
     drops `graphic_large.png`'s size term from 16 to 4. Above 4 threads the
     probe's bytes must stop moving: identical hashes that are the clamp, not
     determinism.

Every row carries its computed `tiles` so the table is interpretable: a hash
table without its clamp column is not.

Requirements: a built `crustyimg` (default features), optionally the lean and
probe binaries, `bench/corpus/`. Stdlib only; no network.

Usage:
    cargo build --release
    CARGO_TARGET_DIR=target-lean  cargo build --release --no-default-features
    CARGO_TARGET_DIR=target-probe cargo build --release --features image/rayon
    python3 scripts/spec123_avif_thread_determinism.py [--json] [--repeats 10]
"""

import argparse
import hashlib
import json
import os
import platform
import resource
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# (label, corpus file, width, height) — dimensions are asserted at run time.
INPUTS = [
    ("photo", "photo_forest_cc0.jpg", 800, 532),
    ("graphic", "graphic_large.png", 512, 512),
]

# (label, argv-after-input, quality the verb encodes at)
#
# ⚠ Pinning `--format avif` drops ALL THREE verbs to the sink default, 80
# (`sink::AVIF_DEFAULT_QUALITY`, `src/sink/mod.rs:59`) — measured, not assumed:
# `web --format avif` and `optimize --format avif` are byte-identical to
# `convert -q 80` (100344 B, `db798cf…`), and NOT to `convert -q 85`
# (125548 B, `1c5ed3f…`). So the pinned matrix is three verbs driving one
# encoder call, which is a weaker triple than it looks.
#
# Their AUTO path is the one users actually touch, and it encodes at 85
# (`sink::FAST_LOSSY_QUALITY`, `src/sink/mod.rs:80`) — `web`/`optimize --json`
# both report `quality: 85`, and the output is byte-identical to
# `convert -q 85`. Leg A2 drives that path so the matrix is not one encode
# wearing three hats.
VERBS = [
    ("convert", ["convert"], 80),
    ("web", ["web"], 80),
    ("optimize", ["optimize"], 80),
]

# Verbs whose auto-decision picks AVIF for the photo input (leg A2).
AUTO_VERBS = [("web", ["web"], 85), ("optimize", ["optimize"], 85)]

AVIF_SPEED = 6  # src/sink/mod.rs:48


def quality_to_quantizer(quality: float) -> int:
    """ravif 0.13.0 `av1encoder.rs:513`, transcribed."""
    q = quality / 100.0
    if q >= 0.82:
        x = (1.0 - q) * 2.6
    elif q > 0.25:
        x = q * -0.5 + (1.0 - 0.125)
    else:
        x = 1.0 - q
    # Rust's f32::round is half-away-from-zero; Python's round() is half-to-even.
    v = x * 255.0
    return int(v + 0.5) if v >= 0 else -int(-v + 0.5)


def min_tile_size(speed: int, quality: float) -> int:
    """ravif 0.13.0 `av1encoder.rs:584`, transcribed.

    Note the upstream names read backwards: `high_quality` is true when the
    QUANTIZER is high, i.e. when the *quality* is below 80. Follow the maths,
    not the identifier.
    """
    base = {0: 4096, 1: 2048, 2: 1024, 3: 512, 4: 256}.get(speed, 128)
    high_quality = quality_to_quantizer(quality) > quality_to_quantizer(80.0)
    return base * (2 if high_quality else 1)


def predict_tiles(threads: int, width: int, height: int, quality: float,
                  cores: int) -> dict:
    """The `tiles` rav1e is configured with, under BOTH live hypotheses.

    We do not get to assume which `current_num_threads` ravif compiled against,
    so the table carries both and lets the measurement choose:

      H_lever  tiles = min(RAYON_NUM_THREADS/--jobs, size_term)
               — true iff `ravif/threading` is ON (real rayon).
      H_cores  tiles = min(available_parallelism(), size_term)
               — true iff it is OFF (the `rayoff` shim), and then the thread
                 settings are invisible to the encoder.

    Leg A falsifies H_lever if the hashes do not move; leg F confirms H_cores by
    byte-identity with a threading-ON probe pinned to the core count.
    """
    mts = min_tile_size(AVIF_SPEED, quality)
    size_term = (width * height) // (mts * mts)
    return {
        "min_tile_size": mts,
        "size_term": size_term,
        "tiles_h_lever": min(threads, size_term),
        "tiles_h_cores": min(cores, size_term),
        "thread_term_binds": threads <= size_term,
    }


def run(cmd, env=None, cwd=None):
    """Run a command; return (rc, stdout, stderr, wall_s, cpu_s).

    Never piped — a piped command reports the pipe's exit code, not the tool's.
    CPU time is the child's user+sys from getrusage, which is what separates a
    serial encode (cpu/wall ~ 1) from a parallel one (cpu/wall ~ cores).
    """
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    t0 = time.perf_counter()
    p = subprocess.run(
        [str(c) for c in cmd],
        env=full_env,
        cwd=str(cwd or REPO_ROOT),
        capture_output=True,
        text=True,
    )
    wall = time.perf_counter() - t0
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    cpu = (after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime)
    return p.returncode, p.stdout, p.stderr, wall, cpu


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def encode(binary, verb_args, src, dst, threads=None, jobs=None, quality=None,
           pin_format=True):
    """One AVIF encode through the shipped CLI surface. Returns a result row."""
    cmd = [binary, *verb_args, src]
    if pin_format:
        cmd += ["--format", "avif"]
    if quality is not None:
        cmd += ["-q", str(quality)]
    if jobs is not None:
        cmd += ["-j", str(jobs)]
    cmd += ["-o", dst]
    env = {"RAYON_NUM_THREADS": str(threads)} if threads is not None else {}
    rc, out, err, wall, cpu = run(cmd, env=env)
    dst = Path(dst)
    row = {
        "rc": rc,
        "wall_s": round(wall, 4),
        "cpu_s": round(cpu, 4),
        "cpu_per_wall": round(cpu / wall, 2) if wall > 0 else None,
    }
    if rc == 0 and dst.exists():
        row["sha256"] = sha256(dst)
        row["bytes"] = dst.stat().st_size
    else:
        row["sha256"] = None
        row["bytes"] = None
        row["stderr"] = err.strip().splitlines()[:2]
    return row


def find_binary(explicit, target_dir):
    if explicit:
        p = Path(explicit)
        return p if p.exists() else None
    for profile in ("release", "debug"):
        p = REPO_ROOT / target_dir / profile / "crustyimg"
        if p.exists():
            return p
    return None


def assert_dimensions(binary, work):
    """Dimensions feed the clamp; read them from the tool rather than assume."""
    seen = {}
    for label, fname, w, h in INPUTS:
        src = REPO_ROOT / "bench" / "corpus" / fname
        rc, out, err, _, _ = run([binary, "info", src])
        if rc != 0:
            raise SystemExit(f"info failed on {fname}: {err}")
        dims = [ln for ln in out.splitlines() if ln.startswith("dimensions:")]
        got = dims[0].split()[1] if dims else "?"
        if got != f"{w}x{h}":
            raise SystemExit(f"{fname}: expected {w}x{h}, tool reports {got}")
        seen[label] = got
    return seen


def leg_a(binary, work, threads_list, cores, results):
    """The main matrix: 3 verbs x 2 inputs x N thread counts, shipped binary."""
    rows = []
    for verb, argv, q in VERBS:
        for label, fname, w, h in INPUTS:
            src = REPO_ROOT / "bench" / "corpus" / fname
            for t in threads_list:
                dst = work / f"A_{verb}_{label}_t{t}.avif"
                r = encode(binary, argv, src, dst, threads=t)
                r.update({"verb": verb, "input": label, "threads": t,
                          "quality": q, **predict_tiles(t, w, h, q, cores)})
                rows.append(r)
    results["leg_a_rayon_matrix"] = rows
    return rows


def leg_a2(binary, work, threads_list, cores, results):
    """The AUTO-decision path — no `--format` pin, so the verb picks AVIF itself
    and encodes at FAST_LOSSY_QUALITY. This is the surface users touch."""
    rows = []
    label, fname, w, h = INPUTS[0]
    src = REPO_ROOT / "bench" / "corpus" / fname
    for verb, argv, q in AUTO_VERBS:
        for t in threads_list:
            out_dir = work / f"A2_{verb}_t{t}"
            out_dir.mkdir(exist_ok=True)
            cmd = [binary, *argv, src, "--out-dir", out_dir]
            rc, _, err, wall, cpu = run(cmd, env={"RAYON_NUM_THREADS": str(t)})
            produced = sorted(out_dir.glob("*"))
            dst = produced[0] if produced else None
            r = {
                "verb": f"{verb} (auto)", "input": label, "threads": t,
                "quality": q, "rc": rc,
                "produced": dst.name if dst else None,
                "sha256": sha256(dst) if dst else None,
                "bytes": dst.stat().st_size if dst else None,
                "wall_s": round(wall, 4),
                "cpu_per_wall": round(cpu / wall, 2) if wall > 0 else None,
                **predict_tiles(t, w, h, q, cores),
            }
            rows.append(r)
    results["leg_a2_auto_path"] = rows
    return rows


def leg_b(binary, work, threads_list, cores, results):
    """`--jobs` on `optimize`, batch size 1 — the scoped-pool lever."""
    rows = []
    label, fname, w, h = INPUTS[0]
    src = REPO_ROOT / "bench" / "corpus" / fname
    for j in threads_list:
        dst = work / f"B_optimize_{label}_j{j}.avif"
        r = encode(binary, ["optimize"], src, dst, jobs=j)
        r.update({"verb": "optimize", "input": label, "jobs": j, "quality": 85,
                  **predict_tiles(j, w, h, 85, cores)})
        rows.append(r)
    results["leg_b_jobs"] = rows
    return rows


def leg_c(binary, work, fixed_threads, repeats, results):
    """AC-4: run-to-run at one fixed thread count."""
    rows = []
    for verb, argv, q in VERBS:
        label, fname, w, h = INPUTS[0]
        src = REPO_ROOT / "bench" / "corpus" / fname
        hashes = []
        for i in range(repeats):
            dst = work / f"C_{verb}_{i}.avif"
            r = encode(binary, argv, src, dst, threads=fixed_threads)
            hashes.append(r["sha256"])
            dst.unlink(missing_ok=True)
        rows.append({
            "verb": verb, "input": label, "threads": fixed_threads,
            "repeats": repeats, "distinct_hashes": len(set(hashes)),
            "sha256": hashes[0], "stable": len(set(hashes)) == 1,
        })
    results["leg_c_run_to_run"] = rows
    return rows


def leg_d(lean_binary, work, results):
    """The lean build has no AVIF encoder at all. Record what it does instead."""
    if lean_binary is None:
        results["leg_d_lean"] = {"skipped": "no --no-default-features binary"}
        return results["leg_d_lean"]
    label, fname, _, _ = INPUTS[0]
    src = REPO_ROOT / "bench" / "corpus" / fname
    dst = work / "D_lean.avif"
    rc, out, err, wall, _ = run([lean_binary, "convert", src, "--format", "avif",
                                 "-o", dst])
    results["leg_d_lean"] = {
        "rc": rc,
        "produced_output": dst.exists(),
        "stderr": err.strip().splitlines()[:3],
    }
    return results["leg_d_lean"]


def leg_e(probe_binary, work, threads_list, cores, results):
    """Positive control: ravif/threading ON, so the ambient pool IS the lever."""
    if probe_binary is None:
        results["leg_e_probe"] = {"skipped": "no --features image/rayon binary"}
        return results["leg_e_probe"]
    rows = []
    for label, fname, w, h in INPUTS:
        src = REPO_ROOT / "bench" / "corpus" / fname
        for t in threads_list:
            dst = work / f"E_convert_{label}_t{t}.avif"
            r = encode(probe_binary, ["convert"], src, dst, threads=t)
            r.update({"verb": "convert", "input": label, "threads": t,
                      "quality": 80, **predict_tiles(t, w, h, 80, cores)})
            rows.append(r)
    results["leg_e_probe"] = rows
    return rows


def leg_g(probe_binary, work, threads_list, cores, results):
    """The clamp, demonstrated: q50 drops graphic_large's size term to 4."""
    if probe_binary is None:
        results["leg_g_clamp"] = {"skipped": "no probe binary"}
        return results["leg_g_clamp"]
    rows = []
    label, fname, w, h = INPUTS[1]  # graphic_large.png, 512x512
    src = REPO_ROOT / "bench" / "corpus" / fname
    for t in threads_list:
        dst = work / f"G_clamp_t{t}.avif"
        r = encode(probe_binary, ["convert"], src, dst, threads=t, quality=50)
        r.update({"verb": "convert -q 50", "input": label, "threads": t,
                  "quality": 50, **predict_tiles(t, w, h, 50, cores)})
        rows.append(r)
    results["leg_g_clamp"] = rows
    return rows


def print_table(res):
    def hdr(t):
        print(f"\n{t}\n{'-' * len(t)}")

    env = res["env"]
    print(f"crustyimg {env['version']}  host {env['machine']} {env['system']}  "
          f"cores(available_parallelism-equivalent) {env['cores']}")
    print(f"inputs: {env['dimensions']}")

    def rows(title, data, keycols):
        hdr(title)
        cols = keycols + ["tiles|lever", "tiles|cores", "sha256", "bytes",
                          "wall_s", "cpu/wall"]
        print(" | ".join(f"{c:<12}" for c in cols))
        for r in data:
            vals = [str(r.get(k, "")) for k in keycols]
            clamp = "" if r.get("thread_term_binds") else "*"
            vals += [
                f"{r.get('tiles_h_lever')}{clamp}",
                str(r.get("tiles_h_cores")),
                (r.get("sha256") or "-")[:16],
                str(r.get("bytes")),
                f"{r.get('wall_s'):.3f}" if r.get("wall_s") is not None else "-",
                str(r.get("cpu_per_wall")),
            ]
            print(" | ".join(f"{v:<12}" for v in vals))
        print("  * = the size term binds, not the thread term (the clamp)")

    rows("LEG A — shipped binary, RAYON_NUM_THREADS", res["leg_a_rayon_matrix"],
         ["verb", "input", "threads"])
    rows("LEG A2 — shipped binary, AUTO decision path (no --format pin, q85)",
         res["leg_a2_auto_path"], ["verb", "input", "threads"])
    rows("LEG B — shipped binary, optimize --jobs (batch size 1)",
         res["leg_b_jobs"], ["verb", "input", "jobs"])

    hdr("LEG C — run-to-run at a fixed thread count (AC-4)")
    for r in res["leg_c_run_to_run"]:
        print(f"{r['verb']:<10} threads={r['threads']} repeats={r['repeats']} "
              f"distinct_hashes={r['distinct_hashes']} "
              f"stable={'YES' if r['stable'] else 'NO'} {r['sha256'][:16]}")

    hdr("LEG D — lean build (--no-default-features)")
    d = res["leg_d_lean"]
    if "skipped" in d:
        print(f"skipped: {d['skipped']}")
    else:
        print(f"rc={d['rc']} produced_output={d['produced_output']}")
        for ln in d["stderr"]:
            print(f"  {ln}")

    if isinstance(res["leg_e_probe"], list):
        rows("LEG E — POSITIVE CONTROL: probe build (image/rayon → "
             "ravif/threading ON)", res["leg_e_probe"], ["verb", "input", "threads"])
    else:
        hdr("LEG E — positive control")
        print(f"skipped: {res['leg_e_probe']['skipped']}")

    hdr("LEG F — cross-check: probe at N=cores vs shipped")
    for line in res["leg_f_crosscheck"]:
        print(f"  {line}")

    if isinstance(res["leg_g_clamp"], list):
        rows("LEG G — the clamp demonstrated (probe, q50 → min_tile_size 256, "
             "size term 4)", res["leg_g_clamp"], ["verb", "input", "threads"])


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--bin", help="shipped binary (default target/release)")
    ap.add_argument("--lean-bin", help="--no-default-features binary "
                                       "(default target-lean/release)")
    ap.add_argument("--probe-bin", help="--features image/rayon binary "
                                        "(default target-probe/release)")
    ap.add_argument("--threads", default=None,
                    help="comma-separated thread counts (default 1,4,<cores>)")
    ap.add_argument("--repeats", type=int, default=10,
                    help="run-to-run repeats for leg C (default 10)")
    ap.add_argument("--work", help="working directory (default: a temp dir)")
    ap.add_argument("--keep", action="store_true", help="keep the working dir")
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    args = ap.parse_args()

    binary = find_binary(args.bin, "target")
    if binary is None:
        binary = find_binary(args.bin, "target-full")
    if binary is None:
        raise SystemExit("no crustyimg binary found; run `cargo build --release`")
    lean = find_binary(args.lean_bin, "target-lean")
    probe = find_binary(args.probe_bin, "target-probe")

    cores = os.cpu_count() or 1
    threads_list = ([int(t) for t in args.threads.split(",")] if args.threads
                    else sorted({1, 4, cores}))

    work = Path(args.work) if args.work else Path(tempfile.mkdtemp(prefix="spec123_"))
    work.mkdir(parents=True, exist_ok=True)

    rc, out, _, _, _ = run([binary, "--version"])
    res = {"env": {
        "version": out.strip(),
        "system": platform.system(),
        "machine": platform.machine(),
        "cores": cores,
        "threads_tested": threads_list,
        "binary": str(binary),
        "lean_binary": str(lean) if lean else None,
        "probe_binary": str(probe) if probe else None,
        "dimensions": assert_dimensions(binary, work),
    }}

    a = leg_a(binary, work, threads_list, cores, res)
    a2 = leg_a2(binary, work, threads_list, cores, res)
    leg_b(binary, work, threads_list, cores, res)
    leg_c(binary, work, cores, args.repeats, res)
    leg_d(lean, work, res)
    e = leg_e(probe, work, threads_list, cores, res)

    # Leg F — does the probe at N=cores land on the shipped bytes? If it does,
    # the shipped tile count IS the machine's core count.
    cross = []
    if isinstance(e, list):
        for label, _, _, _ in INPUTS:
            ship = next((r for r in a if r["verb"] == "convert"
                         and r["input"] == label and r["threads"] == cores), None)
            prb = next((r for r in e if r["input"] == label
                        and r["threads"] == cores), None)
            if ship and prb:
                same = ship["sha256"] == prb["sha256"]
                cross.append(
                    f"{label}: shipped {ship['sha256'][:16]} vs probe@{cores} "
                    f"{prb['sha256'][:16]} → {'IDENTICAL' if same else 'DIFFER'}")
    else:
        cross.append("skipped: no probe binary")
    res["leg_f_crosscheck"] = cross

    leg_g(probe, work, threads_list, cores, res)

    # Derived summary — hashes first; the verdict follows from them.
    shipped_hashes = {}
    for r in a:
        shipped_hashes.setdefault((r["verb"], r["input"]), set()).add(r["sha256"])
    res["summary"] = {
        "shipped_distinct_hashes_per_cell":
            {f"{k[0]}/{k[1]}": len(v) for k, v in shipped_hashes.items()},
        "shipped_invariant_across_threads":
            all(len(v) == 1 for v in shipped_hashes.values()),
        "auto_path_distinct_hashes_per_cell": {
            f'{v}/{INPUTS[0][0]}': len({r["sha256"] for r in a2 if r["verb"] == v})
            for v in {r["verb"] for r in a2}},
        "jobs_invariant": len({r["sha256"] for r in res["leg_b_jobs"]}) == 1,
        "run_to_run_stable": all(r["stable"] for r in res["leg_c_run_to_run"]),
    }
    if isinstance(e, list):
        probe_hashes = {}
        for r in e:
            probe_hashes.setdefault(r["input"], set()).add(r["sha256"])
        res["summary"]["probe_control_fired"] = \
            any(len(v) > 1 for v in probe_hashes.values())

    if args.json:
        print(json.dumps(res, indent=2))
    else:
        print_table(res)
        print(f"\nsummary: {json.dumps(res['summary'])}")
        print(f"work dir: {work}" if args.keep else "")

    if not args.keep and not args.work:
        shutil.rmtree(work, ignore_errors=True)   # leg A2 writes subdirectories
    return 0


if __name__ == "__main__":
    sys.exit(main())
