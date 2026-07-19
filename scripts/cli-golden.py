#!/usr/bin/env python3
"""crustyimg CLI golden-output differential gate (SPEC-097).

`src/cli/mod.rs` is being split into a submodule tree — a pure mechanical
refactor with no behavior change. The only way to *prove* that is to run a
representative command set through a binary, byte-for-byte, before and after
each extraction. This script IS that proof.

Design (mirrors `scripts/bench.py`'s guarantees):

  * OFFLINE      — only ever spawns the local `crustyimg` binary; no sockets.
  * STDLIB ONLY  — no pip installs, no third-party modules (Python 3.8+).
  * DETERMINISTIC — fixtures are hand-built (a minimal pure-Python PNG
                  writer), not sourced from the binary under test, so a
                  behavior change in the binary can never corrupt the inputs
                  used to detect it. The only known source of run-to-run
                  noise is wall-clock `--timing` output, which is masked
                  before comparison (see `normalize`).

Usage:
    python3 scripts/cli-golden.py capture --bin PATH [--golden-dir DIR]
    python3 scripts/cli-golden.py check   --bin PATH [--golden-dir DIR]

`capture` runs the full command set against --bin and stores stdout, stderr,
exit code, and a hash of every file the command wrote, as the oracle.
`check` re-runs the same command set against a (possibly different) --bin and
diffs each case against the stored oracle, byte-for-byte. Nonzero exit if any
case mismatches or errors.

Fixtures are generated once (independent of --bin) into `<golden-dir>/fixtures/`
and reused verbatim by every subsequent capture/check invocation.
"""

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import zlib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent


# ─── Minimal pure-Python PNG writer (no `image` crate, no PIL, no ImageMagick) ─


def _chunk(kind: bytes, data: bytes) -> bytes:
    return (
        len(data).to_bytes(4, "big")
        + kind
        + data
        + (zlib.crc32(kind + data) & 0xFFFFFFFF).to_bytes(4, "big")
    )


def make_png(w: int, h: int, seed: int, alpha: bool = False) -> bytes:
    """A small deterministic RGB(A) PNG: a seeded per-pixel pattern (not flat,
    so pixel-dependent code paths are actually exercised), 8-bit, no filter."""
    channels = 4 if alpha else 3
    rows = bytearray()
    for y in range(h):
        rows.append(0)  # filter type 0 (None) for this scanline
        for x in range(w):
            r = (x * 7 + y * 3 + seed * 11) % 256
            g = (x * 13 + y * 17 + seed * 5) % 256
            b = (x * 3 + y * 29 + seed * 41) % 256
            rows.extend((r, g, b))
            if alpha:
                rows.append(200)
    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = (
        w.to_bytes(4, "big")
        + h.to_bytes(4, "big")
        + bytes([8, 6 if alpha else 2, 0, 0, 0])
    )
    idat = zlib.compress(bytes(rows), 9)
    return sig + _chunk(b"IHDR", ihdr) + _chunk(b"IDAT", idat) + _chunk(b"IEND", b"")


RESIZE_RECIPE = 'version = "1"\n\n[[step]]\nop = "resize"\nmode = "max"\nwidth = 16\n'


def ensure_fixtures(fixtures_dir: Path, binary: str) -> None:
    """Build the fixture set once; every later capture/check call reuses these
    exact bytes verbatim regardless of which --bin generated them.

    PNGs are hand-built (fully independent of the binary under test, see
    `make_png`). `meta copy` only supports JPEG in v1 (`src/metadata/mod.rs`),
    so the two JPEG fixtures are derived from the PNGs via a one-time `convert`
    through `binary` — `convert` itself is not part of what this harness
    verifies, it is only ever used here, once, as fixture prep (the same role
    `tests/common/mod.rs`'s in-process encoders play for the Rust integration
    tests) — never regenerated afterward, so a later behavior change in
    `convert` cannot corrupt an already-captured oracle's inputs.
    """
    if fixtures_dir.exists():
        return
    tmp = fixtures_dir.with_name(fixtures_dir.name + ".tmp")
    if tmp.exists():
        shutil.rmtree(tmp)
    tmp.mkdir(parents=True)
    (tmp / "a.png").write_bytes(make_png(64, 48, seed=1))
    (tmp / "b.png").write_bytes(make_png(64, 48, seed=2))
    (tmp / "s.png").write_bytes(make_png(16, 16, seed=3))
    lint_dir = tmp / "lint_dir"
    lint_dir.mkdir()
    (lint_dir / "a.png").write_bytes((tmp / "a.png").read_bytes())
    (lint_dir / "b.png").write_bytes((tmp / "b.png").read_bytes())
    (tmp / "r.toml").write_text(RESIZE_RECIPE)
    for stem in ("a", "b"):
        proc = subprocess.run(
            [binary, "convert", f"{stem}.png", "--format", "jpeg", "-o", f"{stem}.jpg"],
            cwd=tmp,
            capture_output=True,
        )
        if proc.returncode != 0:
            raise RuntimeError(
                f"fixture prep: convert {stem}.png -> jpeg failed (exit {proc.returncode}): "
                f"{proc.stderr.decode('utf-8', errors='replace')}"
            )
    tmp.rename(fixtures_dir)


# ─── Case definitions ───────────────────────────────────────────────────────
# Each case is (name, setup(workdir, fixtures_dir), argv). `setup` copies in
# whatever fixture files the case needs (relative paths only — cwd is the
# case's own workdir, so output-format auto-decisions and lint's cwd-relative
# path/URI rendering are stable across runs regardless of the golden-dir's
# absolute location).


def _copy(fixtures_dir: Path, workdir: Path, *names: str) -> None:
    for name in names:
        src = fixtures_dir / name
        dst = workdir / name
        if src.is_dir():
            shutil.copytree(src, dst)
        else:
            dst.write_bytes(src.read_bytes())


def _setup_build(fixtures_dir: Path, workdir: Path) -> None:
    _copy(fixtures_dir, workdir, "a.png", "r.toml")
    (workdir / "crustyimg.build.toml").write_text(
        'version = 1\n\n[[target]]\nsource = "a.png"\nrecipe = "r.toml"\nout = "dist"\n'
    )


CASES = [
    ("info_json", lambda f, w: _copy(f, w, "a.png"), ["info", "a.png", "--json"]),
    ("info_human_exif", lambda f, w: _copy(f, w, "a.png"), ["info", "a.png", "--exif"]),
    ("diff_json", lambda f, w: _copy(f, w, "a.png", "b.png"), ["diff", "a.png", "b.png", "--json"]),
    ("diff_human", lambda f, w: _copy(f, w, "a.png", "b.png"), ["diff", "a.png", "b.png"]),
    (
        "diff_dimension_mismatch",
        lambda f, w: _copy(f, w, "a.png", "s.png"),
        ["diff", "a.png", "s.png"],
    ),
    (
        "lint_format_json",
        lambda f, w: _copy(f, w, "lint_dir"),
        ["lint", "lint_dir", "--format", "json", "--no-config"],
    ),
    (
        "lint_format_sarif",
        lambda f, w: _copy(f, w, "lint_dir"),
        ["lint", "lint_dir", "--format", "sarif", "--no-config"],
    ),
    (
        "optimize_explain_human",
        lambda f, w: _copy(f, w, "a.png"),
        ["optimize", "a.png", "--explain", "--out-dir", "out"],
    ),
    (
        "optimize_explain_json",
        lambda f, w: _copy(f, w, "a.png"),
        ["optimize", "a.png", "--explain=json", "--out-dir", "out"],
    ),
    (
        "optimize_json_flag",
        lambda f, w: _copy(f, w, "a.png"),
        ["optimize", "a.png", "--json", "--out-dir", "out"],
    ),
    (
        "optimize_timing",
        lambda f, w: _copy(f, w, "a.png"),
        ["optimize", "a.png", "--timing", "--out-dir", "out"],
    ),
    (
        "web_json",
        lambda f, w: _copy(f, w, "a.png"),
        ["web", "a.png", "--json", "--out-dir", "out"],
    ),
    (
        "web_timing",
        lambda f, w: _copy(f, w, "a.png"),
        ["web", "a.png", "--timing", "--out-dir", "out"],
    ),
    ("build", _setup_build, ["build"]),
    (
        "resize",
        lambda f, w: _copy(f, w, "a.png"),
        ["resize", "a.png", "--max", "16", "-o", "out.png"],
    ),
    (
        "resize_usage_error",
        lambda f, w: _copy(f, w, "a.png"),
        ["resize", "a.png"],
    ),
    (
        "watermark",
        lambda f, w: _copy(f, w, "a.png"),
        ["watermark", "a.png", "--text", "golden", "-o", "out.png"],
    ),
    (
        "meta_strip",
        lambda f, w: _copy(f, w, "a.png"),
        ["meta", "strip", "a.png", "-o", "out.png"],
    ),
    (
        "meta_clean",
        lambda f, w: _copy(f, w, "a.png"),
        ["meta", "clean", "a.png", "--gps", "-o", "out.png"],
    ),
    (
        "meta_set",
        lambda f, w: _copy(f, w, "a.png"),
        [
            "meta", "set", "a.png",
            "--artist", "A", "--copyright", "C", "--description", "D",
            "-o", "out.png",
        ],
    ),
    (
        "meta_copy",
        lambda f, w: _copy(f, w, "a.jpg", "b.jpg"),
        ["meta", "copy", "--from", "a.jpg", "--to", "b.jpg", "-o", "out.jpg"],
    ),
]


# ─── Normalization (mask real-but-non-deterministic wall-clock timing) ─────

_MS_HUMAN = re.compile(r"\d+\.\d+(?= ms)")
_MS_JSON = re.compile(r'("(?:decode|encode|total)_ms":)[0-9]+\.[0-9]+')


def normalize(text: str) -> str:
    text = _MS_HUMAN.sub("T", text)
    text = _MS_JSON.sub(r"\1T", text)
    return text


# ─── Run one case ────────────────────────────────────────────────────────────


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def snapshot_files(root: Path) -> dict:
    out = {}
    for p in sorted(root.rglob("*")):
        if p.is_file():
            out[str(p.relative_to(root))] = sha256(p.read_bytes())
    return out


def run_case(binary: str, name: str, setup, argv: list, case_root: Path) -> dict:
    workdir = case_root / name
    if workdir.exists():
        shutil.rmtree(workdir)
    workdir.mkdir(parents=True)
    fixtures_dir = case_root.parent / "fixtures"
    setup(fixtures_dir, workdir)
    before = snapshot_files(workdir)

    proc = subprocess.run(
        [binary] + argv,
        cwd=workdir,
        capture_output=True,
        timeout=120,
    )

    after = snapshot_files(workdir)
    produced = {
        rel: h
        for rel, h in after.items()
        if before.get(rel) != h
    }

    return {
        "argv": argv,
        "returncode": proc.returncode,
        "stdout": normalize(proc.stdout.decode("utf-8", errors="replace")),
        "stderr": normalize(proc.stderr.decode("utf-8", errors="replace")),
        "produced_files": produced,
    }


# ─── capture / check ─────────────────────────────────────────────────────────


def do_capture(binary: str, golden_dir: Path) -> int:
    fixtures_dir = golden_dir / "fixtures"
    ensure_fixtures(fixtures_dir, binary)
    golden_cases_dir = golden_dir / "golden"
    if golden_cases_dir.exists():
        shutil.rmtree(golden_cases_dir)
    golden_cases_dir.mkdir(parents=True)
    case_root = golden_dir / "work-capture"

    for name, setup, argv in CASES:
        result = run_case(binary, name, setup, argv, case_root)
        (golden_cases_dir / f"{name}.json").write_text(json.dumps(result, indent=2, sort_keys=True))
        print(f"captured {name}: exit={result['returncode']} "
              f"stdout={len(result['stdout'])}B stderr={len(result['stderr'])}B "
              f"files={len(result['produced_files'])}")

    print(f"\ncapture complete: {len(CASES)} cases -> {golden_cases_dir}")
    return 0


def do_check(binary: str, golden_dir: Path) -> int:
    fixtures_dir = golden_dir / "fixtures"
    golden_cases_dir = golden_dir / "golden"
    if not fixtures_dir.exists() or not golden_cases_dir.exists():
        print("check: no oracle captured yet — run `capture` first", file=sys.stderr)
        return 2
    case_root = golden_dir / "work-check"

    failures = []
    for name, setup, argv in CASES:
        oracle_path = golden_cases_dir / f"{name}.json"
        if not oracle_path.exists():
            failures.append((name, f"no oracle recorded for case {name!r}"))
            continue
        oracle = json.loads(oracle_path.read_text())
        actual = run_case(binary, name, setup, argv, case_root)

        diffs = []
        if actual["argv"] != oracle["argv"]:
            diffs.append(f"argv changed: {oracle['argv']} -> {actual['argv']}")
        if actual["returncode"] != oracle["returncode"]:
            diffs.append(f"exit code {oracle['returncode']} -> {actual['returncode']}")
        if actual["stdout"] != oracle["stdout"]:
            diffs.append(f"stdout differs (oracle {len(oracle['stdout'])}B, actual {len(actual['stdout'])}B)")
        if actual["stderr"] != oracle["stderr"]:
            diffs.append(f"stderr differs (oracle {len(oracle['stderr'])}B, actual {len(actual['stderr'])}B)")
        if actual["produced_files"] != oracle["produced_files"]:
            missing = set(oracle["produced_files"]) - set(actual["produced_files"])
            extra = set(actual["produced_files"]) - set(oracle["produced_files"])
            changed = {
                rel for rel in set(oracle["produced_files"]) & set(actual["produced_files"])
                if oracle["produced_files"][rel] != actual["produced_files"][rel]
            }
            if missing:
                diffs.append(f"files missing: {sorted(missing)}")
            if extra:
                diffs.append(f"files extra: {sorted(extra)}")
            if changed:
                diffs.append(f"files hash-changed: {sorted(changed)}")

        if diffs:
            failures.append((name, "; ".join(diffs)))
            print(f"FAIL {name}: {diffs}")
        else:
            print(f"ok   {name}")

    print(f"\n{len(CASES) - len(failures)}/{len(CASES)} cases byte-identical")
    if failures:
        print("\nFAILURES:", file=sys.stderr)
        for name, reason in failures:
            print(f"  {name}: {reason}", file=sys.stderr)
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("mode", choices=["capture", "check"])
    parser.add_argument("--bin", required=True, help="path to the crustyimg binary to exercise")
    parser.add_argument(
        "--golden-dir",
        default=str(REPO_ROOT / "target" / "cli-golden"),
        help="directory for fixtures + oracle (default: target/cli-golden, gitignored)",
    )
    args = parser.parse_args()

    binary = str(Path(args.bin).resolve())
    if not Path(binary).is_file():
        parser.error(f"--bin not found: {binary}")
    golden_dir = Path(args.golden_dir).resolve()
    golden_dir.mkdir(parents=True, exist_ok=True)

    if args.mode == "capture":
        return do_capture(binary, golden_dir)
    return do_check(binary, golden_dir)


if __name__ == "__main__":
    sys.exit(main())
