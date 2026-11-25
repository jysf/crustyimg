"""Helpers for run.sh.

stats.py A.raw B.raw          compare two same-size raw 8-bit buffers and print how far apart they are
stats.py --flip SEED IN OUT   write IN to OUT with 300 random bytes changed
"""
import random
import sys


def compare(path_a, path_b):
    a = open(path_a, "rb").read()
    b = open(path_b, "rb").read()
    if len(a) != len(b):
        print(f"SIZE MISMATCH {len(a)} vs {len(b)}")
        return
    counts = {}
    for x, y in zip(a, b):
        d = abs(x - y)
        counts[d] = counts.get(d, 0) + 1
    n = len(a)
    within1 = (counts.get(0, 0) + counts.get(1, 0)) * 100 / n
    print(f"identical {counts.get(0, 0) * 100 / n:.1f}%, within 1 {within1:.2f}%, max diff {max(counts)}")


def flip(seed, src, dst):
    data = bytearray(open(src, "rb").read())
    rng = random.Random(seed)
    for _ in range(300):
        data[rng.randrange(len(data))] ^= rng.randrange(1, 256)
    open(dst, "wb").write(data)


if __name__ == "__main__":
    if sys.argv[1] == "--flip":
        flip(int(sys.argv[2]), sys.argv[3], sys.argv[4])
    else:
        compare(sys.argv[1], sys.argv[2])
