"""Re-pose asm_frame/panel/plate with full-precision pose matrices."""
import math

import adsk.core
import adsk.fusion

S = math.sin(math.radians(78.0))
C = math.cos(math.radians(78.0))


def mm(v):
    return v / 10.0


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    comp = design.rootComponent
    moves = comp.features.moveFeatures
    copies = comp.features.copyPasteBodies

    def body(name, required=True):
        for i in range(comp.bRepBodies.count):
            if comp.bRepBodies.item(i).name == name:
                return comp.bRepBodies.item(i)
        if required:
            raise RuntimeError("no body " + name)
        return None

    for name in ("asm_frame", "asm_panel", "asm_plate"):
        b = body(name, required=False)
        if b:
            b.deleteMe()

    def copy_body(src, new_name):
        coll = adsk.core.ObjectCollection.create()
        coll.add(body(src))
        copies.add(coll)
        nb = comp.bRepBodies.item(comp.bRepBodies.count - 1)
        nb.name = new_name
        return nb

    def pose(b, origin_mm, xv, yv, zv):
        m = adsk.core.Matrix3D.create()
        ok = m.setWithCoordinateSystem(
            adsk.core.Point3D.create(mm(origin_mm[0]), mm(origin_mm[1]),
                                     mm(origin_mm[2])),
            adsk.core.Vector3D.create(*xv),
            adsk.core.Vector3D.create(*yv),
            adsk.core.Vector3D.create(*zv))
        print(f"  setWithCoordinateSystem ok={ok}")
        coll = adsk.core.ObjectCollection.create()
        coll.add(b)
        try:
            inp = moves.createInput(coll, m)
            moves.add(inp)
            print("  posed via createInput")
        except RuntimeError as exc:
            print("  createInput rejected:", exc)
            inp = moves.createInput2(coll)
            inp.defineAsFreeMove(m)
            moves.add(inp)
            print("  posed via createInput2/defineAsFreeMove")

    X_M = (-1.0, 0.0, 0.0)
    Y_UP = (0.0, C, S)
    Z_REAR = (0.0, S, -C)

    fr = copy_body("frame", "asm_frame")
    pose(fr, (0.0, 149.8, 35.8), X_M, Y_UP, Z_REAR)

    pn = copy_body("proxy_panel", "asm_panel")
    pose(pn, (0.0, 149.8 + 30.0 * S, 35.8 - 30.0 * C), X_M, Y_UP, Z_REAR)

    pl = copy_body("plate", "asm_plate")
    # world = M*(flip(p) + (0,-20.6,5.9)) + T + 45*Y_UP hover
    oy = 149.8 + (-20.6 * C + 5.9 * S) + 45.0 * C
    oz = 35.8 + (-20.6 * S - 5.9 * C) + 45.0 * S
    pose(pl, (0.0, oy, oz), X_M, (0.0, -C, -S), (0.0, -S, C))

    for i in range(comp.bRepBodies.count):
        b = comp.bRepBodies.item(i)
        b.isLightBulbOn = b.name in ("asm_frame", "asm_panel", "asm_plate")

    for name in ("asm_frame", "asm_panel", "asm_plate"):
        bb = body(name).boundingBox
        print(f"{name}: x[{bb.minPoint.x * 10:.0f},{bb.maxPoint.x * 10:.0f}] "
              f"y[{bb.minPoint.y * 10:.0f},{bb.maxPoint.y * 10:.0f}] "
              f"z[{bb.minPoint.z * 10:.0f},{bb.maxPoint.z * 10:.0f}]")
    app.activeViewport.fit()
