"""Motion-sweep verification: every part is boolean-tested against its
mating part at several positions ALONG ITS REAL INSERTION PATH.
Uses temporary BRep copies - the document is not modified.
"""
import math

import adsk.core
import adsk.fusion

S78 = math.sin(math.radians(78.0))
C78 = math.cos(math.radians(78.0))
LIMIT = 2.5   # mm3 - anything above this that isn't a designed ride fails


def mm(v):
    return v / 10.0


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    comp = design.rootComponent
    tmp = adsk.fusion.TemporaryBRepManager.get()

    def body(name):
        for i in range(comp.bRepBodies.count):
            if comp.bRepBodies.item(i).name == name:
                return comp.bRepBodies.item(i)
        raise RuntimeError("no body " + name)

    inter = adsk.fusion.BooleanTypes.IntersectionBooleanType
    failures = []

    def probe(tag, moving, static, matrix, limit=LIMIT, expect_note=""):
        a = tmp.copy(body(moving))
        tmp.transform(a, matrix)
        b = tmp.copy(body(static))
        tmp.booleanOperation(a, b, inter)
        vol = a.volume * 1000.0
        status = "ok"
        if vol > limit:
            status = "FAIL"
            failures.append(tag)
        elif vol > 0.01:
            status = "ride"
        print(f"{tag:34s} {vol:8.2f} mm3  {status} {expect_note}")

    def m_translate(dx, dy, dz):
        m = adsk.core.Matrix3D.create()
        m.translation = adsk.core.Vector3D.create(mm(dx), mm(dy), mm(dz))
        return m

    def m_cs(origin_mm, xv, yv, zv):
        m = adsk.core.Matrix3D.create()
        m.setWithCoordinateSystem(
            adsk.core.Point3D.create(mm(origin_mm[0]), mm(origin_mm[1]),
                                     mm(origin_mm[2])),
            adsk.core.Vector3D.create(*xv),
            adsk.core.Vector3D.create(*yv),
            adsk.core.Vector3D.create(*zv))
        return m

    print("--- panel slides down the frame pocket (top entry) ---")
    for t in (112.0, 60.0, 25.0, 8.0, 0.0):
        probe(f"panel   v+{t:5.1f}", "proxy_panel", "frame",
              m_translate(0, t, 0), limit=3.5,
              expect_note="(designed rib crush <= ~2.9)")

    print("--- plate slides down the frame channel (flipped, top entry) ---")
    for t in (110.0, 70.0, 40.0, 20.0, 8.0, 0.0):
        probe(f"plate   v+{t:5.1f}", "plate", "frame",
              m_cs((0.0, -20.6 + t, 5.9), (1, 0, 0), (0, -1, 0), (0, 0, -1)),
              limit=1.5, expect_note="(detent ride ~0.9 mid-travel)")

    print("--- screen module lowers into the base slot ---")
    for h in (45.0, 25.0, 10.0, 0.0):
        origin = (0.0, 149.8 + h * C78, 35.8 + h * S78)
        probe(f"frame   drop-{h:5.1f}", "frame", "base",
              m_cs(origin, (-1, 0, 0), (0, C78, S78), (0, S78, -C78)))

    print("--- lid slides into the base from the rear ---")
    for d in (40.0, 22.0, 10.0, 0.0):
        probe(f"lid     y{-76.6 + d:6.1f}", "lid", "base",
              m_translate(0, -76.6 + d, 41.2))

    print("--- boards drop into their bays ---")
    for t in (25.0, 8.0, 0.0):
        for prox in ("proxy_hat_pcb", "proxy_hat_socketF", "proxy_hat_socketR",
                     "proxy_hat_parts"):
            probe(f"{prox[6:]:8s} z+{t:4.1f}", prox, "base",
                  m_translate(0, 0, t))
        probe(f"pico     z+{t:4.1f}", "proxy_pico", "base",
              m_translate(0, 0, t))

    if failures:
        print("RESULT: FAIL ->", ", ".join(failures))
    else:
        print("RESULT: ALL INSERTION PATHS CLEAR")
