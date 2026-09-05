"""Rev2.1 base + lid (mm). Fixes: full-width screen slot (extrude start
offset bug), lid entry door wide enough for the lid, lid grip tab kept
outside the box, legible fenced bays with embossed labels.
"""
import math

import adsk.core
import adsk.fusion

BY = 140.0
BW, BD, BH = 180.0, 58.0, 46.0
WALL, FLOOR = 2.4, 2.8
IN_Y0, IN_Y1 = 22.0, BD - WALL          # interior y 22..55.6 (local)
IN_X = 87.6
SLOT_Z0 = 34.0                          # slot floor (12 deep)
SLOT_Y0, SLOT_Y1 = 9.0, 17.4            # slot gap 8.4
TAN_T = math.tan(math.radians(12.0))
LIDZ0 = 43.6                            # lip underside; lid rides 41.2..43.4
LID_Y = 240.0
LID_W, LID_L, LID_T = 175.0, 39.0, 2.2


def mm(v):
    return v / 10.0


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    comp = design.rootComponent
    sketches = comp.sketches
    extrudes = comp.features.extrudeFeatures
    JOIN = adsk.fusion.FeatureOperations.JoinFeatureOperation
    CUT = adsk.fusion.FeatureOperations.CutFeatureOperation
    NEW = adsk.fusion.FeatureOperations.NewBodyFeatureOperation

    def sk_rects(rects):
        sk = sketches.add(comp.xYConstructionPlane)
        for (x0, y0, x1, y1) in rects:
            a = sk.modelToSketchSpace(
                adsk.core.Point3D.create(mm(x0), mm(BY + y0), 0))
            b = sk.modelToSketchSpace(
                adsk.core.Point3D.create(mm(x1), mm(BY + y1), 0))
            a.z = b.z = 0.0
            sk.sketchCurves.sketchLines.addTwoPointRectangle(a, b)
        return sk

    def up_sign(sk):
        n = sk.xDirection.crossProduct(sk.yDirection)
        return 1.0 if n.z > 0 else -1.0

    def do_extrude(sk, prof, z0, z1, op, body):
        s = up_sign(sk)
        inp = extrudes.createInput(prof, op)
        inp.startExtent = adsk.fusion.OffsetStartDefinition.create(
            adsk.core.ValueInput.createByReal(s * mm(z0)))
        inp.setDistanceExtent(False,
                              adsk.core.ValueInput.createByReal(s * mm(z1 - z0)))
        if body:
            inp.participantBodies = [body]
        return extrudes.add(inp)

    def box(x0, y0, x1, y1, z0, z1, op, body=None):
        sk = sk_rects([(x0, y0, x1, y1)])
        return do_extrude(sk, sk.profiles.item(0), z0, z1, op, body)

    def ring(rects, z0, z1, op, body):
        sk = sk_rects(rects)
        prof = None
        for i in range(sk.profiles.count):
            if sk.profiles.item(i).profileLoops.count == 2:
                prof = sk.profiles.item(i)
        return do_extrude(sk, prof, z0, z1, op, body)

    def poly_x(pts_yz, x0, x1, op, body=None):
        """Polygon in (y local, z), extruded from x0 to x1 (with start offset)."""
        sk = sketches.add(comp.yZConstructionPlane)
        conv = []
        for (y, z) in pts_yz:
            p = sk.modelToSketchSpace(
                adsk.core.Point3D.create(0.0, mm(BY + y), mm(z)))
            p.z = 0.0
            conv.append(p)
        for i in range(len(conv)):
            sk.sketchCurves.sketchLines.addByTwoPoints(
                conv[i], conv[(i + 1) % len(conv)])
        prof = sk.profiles.item(0)
        n = sk.xDirection.crossProduct(sk.yDirection)
        s = 1.0 if n.x > 0 else -1.0
        inp = extrudes.createInput(prof, op)
        inp.startExtent = adsk.fusion.OffsetStartDefinition.create(
            adsk.core.ValueInput.createByReal(s * mm(x0)))
        inp.setDistanceExtent(False,
                              adsk.core.ValueInput.createByReal(s * mm(x1 - x0)))
        if body:
            inp.participantBodies = [body]
        return extrudes.add(inp)

    # ---------------- BASE ----------------
    b = box(-BW / 2, 0, BW / 2, BD, 0, BH, NEW)
    base = b.bodies.item(0)
    base.name = "base"
    box(-IN_X, IN_Y0, IN_X, IN_Y1, FLOOR, BH + 1, CUT, base)   # interior
    # screen slot: FULL-width tilted channel (start offset now correct)
    rise = (BH + 1) - SLOT_Z0
    pts = [(SLOT_Y0, SLOT_Z0), (SLOT_Y1, SLOT_Z0),
           (SLOT_Y1 + rise * TAN_T, BH + 1), (SLOT_Y0 + rise * TAN_T, BH + 1)]
    poly_x(pts, -BW / 2 - 1, BW / 2 + 1, CUT, base)
    # ribbon pass-through behind the slot
    box(-32.0, SLOT_Y1 - 1.0, 32.0, IN_Y0 + 1.0, 38.0, BH + 1, CUT, base)
    # USB cable notch, right wall, open top
    box(86.0, 40.0, BW / 2 + 1, 48.0, 36.0, BH + 1, CUT, base)
    # lid C-channel: upper lips + lower ledges (lid slides at z 41.2..43.4)
    for sx in (1, -1):
        box(sx * (IN_X - 2.0), IN_Y0 + 2.0, sx * IN_X, IN_Y1, LIDZ0, BH,
            JOIN, base)
        box(sx * (IN_X - 1.0), IN_Y0 + 2.0, sx * IN_X, IN_Y1, 40.2, 41.2,
            JOIN, base)
    # lid entry door through the rear wall - WIDER than the lid (175 + 0.4)
    box(-87.7, IN_Y1 - 0.5, 87.7, BD + 1.0, 41.0, 43.8, CUT, base)
    # lid front stops
    for sx in (1, -1):
        box(sx * 66.0, IN_Y0, sx * 74.0, IN_Y0 + 1.4, 40.2, 43.6, JOIN, base)

    # ---- bays (legible): fences + pedestals + labels ----
    box(19.0, IN_Y0, 21.0, IN_Y1, FLOOR, 10.0, JOIN, base)     # divider
    # HAT bay: perimeter fence (1.2 wall, +0.8 play around 65 x 30.2)
    ring([(-57.0, 21.4, 12.0, 55.8), (-55.8, 22.6, 10.8, 54.4)],
         FLOOR, 9.0, JOIN, base)
    box(-54.0, 23.0, -50.0, 54.0, FLOOR, 12.3, JOIN, base)     # pedestal L
    box(5.0, 23.0, 9.0, 54.0, FLOOR, 12.3, JOIN, base)         # pedestal R
    # Pico bay: three fence strips, open toward the cable notch side
    box(24.8, 24.8, 80.2, 26.0, FLOOR, 9.0, JOIN, base)        # front strip
    box(24.8, 24.8, 26.0, 50.2, FLOOR, 9.0, JOIN, base)        # left strip
    box(24.8, 49.0, 80.2, 50.2, FLOOR, 9.0, JOIN, base)        # rear strip
    # embossed floor labels (cosmetic; skipped if the text API differs)
    try:
        sk = sketches.add(comp.xYConstructionPlane)
        texts = sk.sketchTexts
        coll = adsk.core.ObjectCollection.create()
        for (label, cx, cy) in (("HAT", -30.0, 35.0), ("PICO", 44.0, 34.5)):
            ti = texts.createInput2(label, mm(8.0))
            ti.setAsMultiLine(
                adsk.core.Point3D.create(mm(cx), mm(BY + cy), 0),
                adsk.core.Point3D.create(mm(cx + 26.0), mm(BY + cy + 10.0), 0),
                adsk.core.HorizontalAlignments.CenterHorizontalAlignment,
                adsk.core.VerticalAlignments.MiddleVerticalAlignment, 0.0)
            coll.add(texts.add(ti))
        do_extrude(sk, coll, FLOOR - 0.6, FLOOR + 0.05, CUT, base)
        print("labels engraved")
    except Exception as exc:  # cosmetic only - never fail the build
        print("labels skipped:", str(exc)[:120])

    # ---------------- LID ----------------
    ld = box(-LID_W / 2, LID_Y - BY, LID_W / 2, LID_Y - BY + LID_L, 0, LID_T,
             NEW)
    lid = ld.bodies.item(0)
    lid.name = "lid"
    # grip ridge on the tail - stays OUTSIDE the box when closed
    box(-20.0, LID_Y - BY + LID_L - 3.5, 20.0, LID_Y - BY + LID_L - 0.5,
        LID_T, LID_T + 2.5, JOIN, lid)
    for sx in (1, -1):   # front corner entry notches
        box(sx * (LID_W / 2 - 1.2), LID_Y - BY - 0.2, sx * (LID_W / 2 + 0.2),
            LID_Y - BY + 1.2, -0.2, LID_T + 0.2, CUT, lid)

    print(f"base volume {base.volume * 1000:.0f} mm3, "
          f"lid volume {lid.volume * 1000:.0f} mm3")
    app.activeViewport.fit()
