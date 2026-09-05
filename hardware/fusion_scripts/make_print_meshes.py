"""Produce print-oriented STLs in hardware/print/ from the CAD exports.

frame: rotated 102 deg about X so the bezel face lies flat on the plate.
shell: already base-down; both get z-min -> 0 and XY centered.
"""
import math
import os
import struct

SRC = r"C:\Users\charl\Documents\GitHub\e-paper_pico-2W\hardware"
DST = os.path.join(SRC, "print")
os.makedirs(DST, exist_ok=True)


def read_stl(path):
    with open(path, "rb") as f:
        data = f.read()
    ntri = struct.unpack("<I", data[80:84])[0]
    tris = []
    off = 84
    for _ in range(ntri):
        vals = struct.unpack("<12f", data[off:off + 48])
        tris.append(list(vals))
        off += 50
    return tris


def write_stl(path, tris):
    with open(path, "wb") as f:
        f.write(b"epaper73 print-oriented".ljust(80, b"\0"))
        f.write(struct.pack("<I", len(tris)))
        for t in tris:
            f.write(struct.pack("<12f", *t))
            f.write(b"\0\0")


def transform(tris, rot_deg):
    c = math.cos(math.radians(rot_deg))
    s = math.sin(math.radians(rot_deg))
    out = []
    for t in tris:
        nt = []
        for i in range(4):  # normal + 3 vertices, each (x,y,z)
            x, y, z = t[i * 3], t[i * 3 + 1], t[i * 3 + 2]
            ny = y * c - z * s
            nz = y * s + z * c
            nt += [x, ny, nz]
        out.append(nt)
    # recentre XY, drop to z=0 (skip normal at index 0)
    xs = [v for t in out for v in (t[3], t[6], t[9])]
    ys = [v for t in out for v in (t[4], t[7], t[10])]
    zs = [v for t in out for v in (t[5], t[8], t[11])]
    cx = (min(xs) + max(xs)) / 2.0
    cy = (min(ys) + max(ys)) / 2.0
    mz = min(zs)
    for t in out:
        for vi in (1, 2, 3):
            t[vi * 3] -= cx
            t[vi * 3 + 1] -= cy
            t[vi * 3 + 2] -= mz
    return out, (max(xs) - min(xs), max(ys) - min(ys), max(zs) - min(zs))


for name, rot in (("frame", 0.0), ("plate", 0.0), ("base", 0.0),
                  ("lid", 0.0)):
    tris = read_stl(os.path.join(SRC, f"epaper73_{name}.stl"))
    out, dims = transform(tris, rot)
    dest = os.path.join(DST, f"epaper73_{name}_print.stl")
    write_stl(dest, out)
    print(f"{name}: {len(out)} tris, print bbox "
          f"{dims[0]:.1f} x {dims[1]:.1f} x {dims[2]:.1f} mm -> {dest}")
