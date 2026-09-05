"""Probe: is there frame material where the detent bumps should be?"""
import adsk.core
import adsk.fusion


def mm(v):
    return v / 10.0


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    comp = design.rootComponent
    tmp = adsk.fusion.TemporaryBRepManager.get()

    frame = None
    for i in range(comp.bRepBodies.count):
        if comp.bRepBodies.item(i).name == "frame":
            frame = comp.bRepBodies.item(i)

    inter = adsk.fusion.BooleanTypes.IntersectionBooleanType

    def probe(tag, x0, y0, z0, x1, y1, z1):
        obb = adsk.core.OrientedBoundingBox3D.create(
            adsk.core.Point3D.create(mm((x0 + x1) / 2), mm((y0 + y1) / 2),
                                     mm((z0 + z1) / 2)),
            adsk.core.Vector3D.create(1, 0, 0),
            adsk.core.Vector3D.create(0, 1, 0),
            mm(abs(x1 - x0)), mm(abs(y1 - y0)), mm(abs(z1 - z0)))
        cube = tmp.createBox(obb)
        fcopy = tmp.copy(frame)
        tmp.booleanOperation(cube, fcopy, inter)
        print(f"{tag:28s}: {cube.volume * 1000.0:7.3f} mm3 of frame material")

    probe("bump R (85.4..85.7,117,4.6)", 85.40, 116.5, 4.6, 85.70, 118.5, 5.2)
    probe("bump L mirror", -85.70, 116.5, 4.6, -85.40, 118.5, 5.2)
    probe("wall R behind (85.8..86.5)", 85.80, 116.5, 4.6, 86.50, 118.5, 5.2)
    probe("lip R (84..85, z6.5)", 84.0, 60.0, 6.2, 85.0, 66.0, 7.2)
    probe("stop R (81..83,11,4.5)", 81.0, 10.7, 4.0, 83.0, 11.8, 5.5)
    probe("rib R (85.0..85.3,32,2.5)", 85.0, 31.0, 2.2, 85.3, 35.0, 3.2)
