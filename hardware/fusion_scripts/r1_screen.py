"""Rev2.1 screen frame + sliding back plate (mm).

BOTH the panel and the plate slide in from the frame's open top edge:
 - panel slides down the pocket plane (z 2.0..3.6) and seats on the
   bottom ledge -> its vertical position is exact, no float;
 - plate slides down the channel plane (z 3.9..5.9) behind it, under
   the side lips, until it clicks past two detents onto the stops.
Window placed from the real Waveshare drawing: borders L/R 5.1,
top 4.7, bottom 10.5; panel 170.2 x 111.2 x 0.85..1.2.
Frame modeled in print pose: bezel face at z=0.
"""
import adsk.core
import adsk.fusion

W, H = 180.0, 121.2
FACE = 2.0
PKZ0, PKZ1 = 2.0, 5.9        # open cavity depth band behind the face
SKZ1 = 7.6                   # total frame thickness
PK_W = 171.8                 # pocket width (panel +0.8/side)
PK_V0 = 4.2                  # bottom ledge line - panel SEATS here
PAN_W, PAN_H = 170.2, 111.2
WIN_X = 80.8                 # active 160/2 + 0.8 reveal
WIN_V0 = PK_V0 + 10.5 - 0.8      # 13.9
WIN_V1 = PK_V0 + 10.5 + 96.0 + 0.8   # 111.5
CH_X = 85.75                 # plate channel half-width (plate 171)
LIP_X = 83.55                # lip inner edge (covers plate edge 2.2)
LIPZ0 = 5.9
STOP_V = 12.0
PLATE_W, PLATE_L, PLATE_T = 171.0, 107.4, 2.0
PY = -140.0


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
            a = sk.modelToSketchSpace(adsk.core.Point3D.create(mm(x0), mm(y0), 0))
            b = sk.modelToSketchSpace(adsk.core.Point3D.create(mm(x1), mm(y1), 0))
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

    def ring(rects, z0, z1, op, body=None):
        sk = sk_rects(rects)
        prof = None
        for i in range(sk.profiles.count):
            if sk.profiles.item(i).profileLoops.count == 2:
                prof = sk.profiles.item(i)
        return do_extrude(sk, prof, z0, z1, op, body)

    def cyl(cx, cy, r, z0, z1, op, body):
        sk = sketches.add(comp.xYConstructionPlane)
        ctr = sk.modelToSketchSpace(adsk.core.Point3D.create(mm(cx), mm(cy), 0))
        ctr.z = 0.0
        sk.sketchCurves.sketchCircles.addByCenterRadius(ctr, mm(r))
        return do_extrude(sk, sk.profiles.item(0), z0, z1, op, body)

    # ---------------- FRAME ----------------
    f = ring([(-W / 2, 0, W / 2, H), (-WIN_X, WIN_V0, WIN_X, WIN_V1)],
             0, FACE, NEW)
    frame = f.bodies.item(0)
    frame.name = "frame"
    # cavity band z 2.0..5.9: two side walls + bottom ledge band, top OPEN
    box(-W / 2, 0, -PK_W / 2, H, PKZ0, PKZ1, JOIN, frame)
    box(PK_W / 2, 0, W / 2, H, PKZ0, PKZ1, JOIN, frame)
    box(-W / 2, 0, W / 2, PK_V0, PKZ0, PKZ1, JOIN, frame)
    # channel band z 5.9..7.6: side walls + bottom band, top OPEN
    box(-W / 2, 0, -CH_X, H, PKZ1, SKZ1, JOIN, frame)
    box(CH_X, 0, W / 2, H, PKZ1, SKZ1, JOIN, frame)
    box(-W / 2, 0, W / 2, PK_V0, PKZ1, SKZ1, JOIN, frame)
    # lips over the plate (2.2mm bridges, printable face-down on the H2C)
    for sx in (1, -1):
        box(sx * LIP_X, 8.0, sx * CH_X, 114.0, LIPZ0, SKZ1, JOIN, frame)
    # plate bottom stops (rise into the lip band so they fuse to the frame)
    for sx in (1, -1):
        box(sx * 80.0, 10.5, sx * 84.0, STOP_V, 3.6, SKZ1, JOIN, frame)
    # detent bumps: embedded 0.1 into the mid-band wall so they truly fuse
    for sx in (1, -1):
        box(sx * (CH_X - 0.4), 116.0, sx * 86.0, 119.0, 4.4, 5.4, JOIN, frame)
    # panel side centering ribs (0.9 wide, 0.1 nominal crush)
    for sx in (1, -1):
        for v0 in (30.0, 84.0):
            box(sx * 85.0, v0, sx * (PK_W / 2), v0 + 6, PKZ0, 3.6, JOIN, frame)
    # FPC ribbon doorway through the bottom band (face stays closed)
    box(-30.0, -1.0, 30.0, PK_V0 + 0.3, FACE, SKZ1 + 0.2, CUT, frame)

    # ---------------- PLATE ----------------
    p = box(-PLATE_W / 2, PY, PLATE_W / 2, PY + PLATE_L, 0, PLATE_T, NEW)
    plate = p.bodies.item(0)
    plate.name = "plate"
    # six low press domes (face the panel once flipped in; the thin plate
    # flexes so they act as gentle springs against the glass back)
    for (cx, cy) in ((-45.0, PY + 16.0), (45.0, PY + 16.0),
                     (-70.0, PY + 52.0), (70.0, PY + 52.0),
                     (-45.0, PY + 95.0), (45.0, PY + 95.0)):
        cyl(cx, cy, 3.0, PLATE_T, PLATE_T + 1.1, JOIN, plate)
    # detent notches on the edges; the plate flips on insertion, so these
    # sit at the LOW end of the model to land on the frame bumps (v 115..118)
    for sx in (1, -1):
        box(sx * (PLATE_W / 2 - 0.5), PY + 1.4, sx * (PLATE_W / 2 + 0.3),
            PY + 4.4, -0.2, PLATE_T + 0.2, CUT, plate)

    print(f"frame volume {frame.volume * 1000:.0f} mm3, "
          f"plate volume {plate.volume * 1000:.0f} mm3")
    app.activeViewport.fit()
