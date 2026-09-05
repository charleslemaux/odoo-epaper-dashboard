"""Rev2 proxies + boolean interference report (all mm, axis-aligned)."""
import adsk.core
import adsk.fusion

BY = 140.0


def mm(v):
    return v / 10.0


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    comp = design.rootComponent
    sketches = comp.sketches
    extrudes = comp.features.extrudeFeatures
    NEW = adsk.fusion.FeatureOperations.NewBodyFeatureOperation

    def zbox(name, x0, y0, x1, y1, z0, z1):
        sk = sketches.add(comp.xYConstructionPlane)
        a = sk.modelToSketchSpace(adsk.core.Point3D.create(mm(x0), mm(y0), 0))
        b = sk.modelToSketchSpace(adsk.core.Point3D.create(mm(x1), mm(y1), 0))
        a.z = b.z = 0.0
        sk.sketchCurves.sketchLines.addTwoPointRectangle(a, b)
        n = sk.xDirection.crossProduct(sk.yDirection)
        s = 1.0 if n.z > 0 else -1.0
        inp = extrudes.createInput(sk.profiles.item(0), NEW)
        inp.startExtent = adsk.fusion.OffsetStartDefinition.create(
            adsk.core.ValueInput.createByReal(s * mm(z0)))
        inp.setDistanceExtent(False,
                              adsk.core.ValueInput.createByReal(s * mm(z1 - z0)))
        f = extrudes.add(inp)
        f.bodies.item(0).name = name

    # panel seated on the bottom ledge (thickness worst-case 1.2)
    zbox("proxy_panel", -85.1, 4.2, 85.1, 115.4, 2.0, 3.2)
    # HAT in the base bay (x -55..10, y local 23.4..53.6)
    zbox("proxy_hat_pcb", -55.0, BY + 23.4, 10.0, BY + 53.6, 12.3, 13.9)
    zbox("proxy_hat_socketF", -48.0, BY + 24.2, 3.0, BY + 29.3, 3.8, 12.3)
    zbox("proxy_hat_socketR", -48.0, BY + 47.7, 3.0, BY + 52.8, 3.8, 12.3)
    zbox("proxy_hat_parts", -52.0, BY + 25.0, 7.0, BY + 52.0, 13.9, 19.9)
    # Pico anywhere in the loose right bay (board+pins+jack envelope)
    zbox("proxy_pico", 28.0, BY + 26.6, 79.0, BY + 47.6, 2.8, 14.8)

    tmp = adsk.fusion.TemporaryBRepManager.get()
    bodies = {}
    for i in range(comp.bRepBodies.count):
        bd = comp.bRepBodies.item(i)
        bodies[bd.name] = bd
    inter = adsk.fusion.BooleanTypes.IntersectionBooleanType

    def check(a, b):
        ca, cb = tmp.copy(bodies[a]), tmp.copy(bodies[b])
        tmp.booleanOperation(ca, cb, inter)
        vol = ca.volume * 1000.0
        flag = "  <-- !!" if vol > 0.001 else ""
        print(f"{a:18s} x {b:6s}: {vol:8.2f} mm3{flag}")

    check("proxy_panel", "frame")
    for name in ("proxy_hat_pcb", "proxy_hat_socketF", "proxy_hat_socketR",
                 "proxy_hat_parts", "proxy_pico"):
        check(name, "base")
    check("frame", "plate")
    check("base", "lid")

    # arithmetic fit report (parts modeled apart, so booleans don't apply)
    print("fit: frame 7.6 in slot gap 8.4 -> 0.8 play")
    print("fit: plate 171.0 in channel 171.5 -> 0.5 play")
    print("fit: plate 2.0 under lips gap 2.0 (3.9..5.9), springs push up")
    print("fit: lid 175.0 wide / 2.2 thick in channel 175.2 / 2.4")
    app.activeViewport.fit()
