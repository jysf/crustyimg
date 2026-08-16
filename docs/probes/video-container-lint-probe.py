#!/usr/bin/env python3
"""Throwaway ISO-BMFF container linter. Parses BOX STRUCTURE ONLY - never decodes a frame."""
import struct, subprocess, sys, json

UA = "crustyimg-video-lint-probe (jyashinsky@gmail.com)"

def fetch(url, rng):
    r = subprocess.run(["curl","-sL","--max-time","30","-H",f"User-Agent: {UA}",
                        "-H",f"Range: bytes={rng}",url], capture_output=True)
    return r.stdout

def head_len(url):
    r = subprocess.run(["curl","-sLI","--max-time","20","-H",f"User-Agent: {UA}",url],
                       capture_output=True, text=True)
    n=None; ct=None
    for line in r.stdout.splitlines():
        l=line.lower()
        if l.startswith("content-length:"): n=int(line.split(":",1)[1].strip())
        if l.startswith("content-type:"):  ct=line.split(":",1)[1].strip()
    return n, ct

def walk(buf, off=0, end=None, depth=0, want=None, out=None):
    """Yield (type, start, size, payload_off) for boxes at this level."""
    if out is None: out=[]
    if end is None: end=len(buf)
    while off + 8 <= end:
        size = struct.unpack(">I", buf[off:off+4])[0]
        typ  = buf[off+4:off+8]
        hdr  = 8
        if size == 1:
            if off+16 > end: break
            size = struct.unpack(">Q", buf[off+8:off+16])[0]; hdr = 16
        elif size == 0:
            size = end - off
        if size < hdr: break
        out.append((typ.decode("latin1"), off, size, off+hdr))
        if want and typ in want and off+size <= end:
            walk(buf, off+hdr, min(off+size, end), depth+1, want, out)
        off += size
    return out

CONTAINERS = {b"moov", b"trak", b"mdia", b"minf", b"stbl", b"stsd", b"edts", b"udta"}

def analyse(url):
    n, ct = head_len(url)
    head = fetch(url, "0-262143")          # first 256 KB only
    if len(head) < 16 or head[4:8] not in (b"ftyp", b"styp"):
        return {"url": url, "error": f"not ISO-BMFF (first bytes {head[:12]!r}, ct={ct})"}
    top = walk(head)
    order = [t for t,_,_,_ in top]
    brand = head[8:12].decode("latin1", "replace")

    # faststart: does moov precede mdat?
    i_moov = order.index("moov") if "moov" in order else None
    i_mdat = order.index("mdat") if "mdat" in order else None
    if i_moov is not None and i_mdat is not None:
        faststart = i_moov < i_mdat
    elif i_mdat is not None and i_moov is None:
        faststart = False                   # mdat first, moov beyond our window
    else:
        faststart = None

    # If moov is at the tail, we must make a SECOND request - which is the whole point.
    buf, base, extra = head, 0, False
    if i_moov is None and n:
        tail = fetch(url, f"-{min(n, 4_000_000)}")
        if tail:
            buf, base, extra = tail, n - len(tail), True

    boxes = walk(buf, want=CONTAINERS)
    codecs, dims, tracks = [], [], []
    for t, off, size, poff in boxes:
        if t == "stsd" and poff + 16 <= len(buf):
            for t2, o2, s2, p2 in walk(buf, poff + 8, min(off + size, len(buf))):
                codecs.append(t2)
        if t == "hdlr" and poff + 16 <= len(buf):
            tracks.append(buf[poff + 8:poff + 12].decode("latin1", "replace"))
        if t == "tkhd" and poff + 84 <= len(buf):
            ver = buf[poff]
            o = poff + 4 + (32 if ver == 1 else 20) + 16 + 36
            if o + 8 <= len(buf):
                w, h = struct.unpack(">II", buf[o:o+8])
                if w >> 16 and h >> 16: dims.append((w >> 16, h >> 16))
    return {"url": url, "bytes": n, "brand": brand, "top": order[:8],
            "faststart": faststart, "needed_tail_request": extra,
            "codecs": sorted(set(codecs)), "handlers": tracks,
            "dims": sorted(set(dims), reverse=True)}

if __name__ == "__main__":
    for u in sys.argv[1:]:
        print(json.dumps(analyse(u)))
