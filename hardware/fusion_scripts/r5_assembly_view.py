"""Pose Rev2 parts in assembled/exploded positions for tutorial renders.

Leaves asm_* copies in the document (originals stay put) so the user can
orbit a real assembled view. Units: Fusion API cm; all mm are /10.
"""
import math

import adsk.core
import adsk.fusion

A = math.radians(-102.0)     # stand-up rotation about X
T_F = (0.0, 149.8, 35.8)     # frame anchor translation (mm)
V_UP = (0.0, 0.2079, 0.9781)     # screen "up" direction, world
BACKD = (0.0, 0.9781, -0.2079)   # screen "rear" direction, world


def mm(v):
    return v / 10.0


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    comp = design.rootComponent
    moves = comp.features.moveFeatures
    copies = comp.features.copyPasteBodies

    def body(name):
        for i in range(comp.bRepBodies.count):
            if comp.bRepBodies.item(i).name == name:
                return comp.bRepBodies.item(i)
        raise RuntimeError("no body " + name)

    def copy_body(src, new_name):
        coll = adsk.core.ObjectCollection.create()
        coll.add(body(src))
        copies.add(coll)
        nb = comp.bRepBodies.item(comp.bRepBodies.count - 1)
        nb.name = new_name
        return nb

    def move(b, matrix):
        coll = adsk.core.ObjectCollection.create()
        coll.add(b)
        inp = moves.createInput(coll, matrix)
        moves.add(inp)

    def rot_z180(b):
        m = adsk.core.Matrix3D.create()
        m.setToRotation(math.pi, adsk.core.Vector3D.create(0, 0, 1),
                        adsk.core.Point3D.create(0, 0, 0))
        move(b, m)

    def rot_x(b, ang):
        m = adsk.core.Matrix3D.create()
        m.setToRotation(ang, adsk.core.Vector3D.create(1, 0, 0),
                        adsk.core.Point3D.create(0, 0, 0))
        move(b, m)

    def translate(b, dx, dy, dz):
        m = adsk.core.Matrix3D.create()
        m.translation = adsk.core.Vector3D.create(mm(dx), mm(dy), mm(dz))
        move(b, m)

    # ---- assembled screen module copies ----
    asm_frame = copy_body("frame", "asm_frame")
    rot_z180(asm_frame)
    rot_x(asm_frame, A)
    translate(asm_frame, *T_F)

    asm_panel = copy_body("proxy_panel", "asm_panel")
    rot_z180(asm_panel)
    rot_x(asm_panel, A)
    hover_p = tuple(T_F[i] + BACKD[i] * 30.0 for i in range(3))
    translate(asm_panel, *hover_p)

    asm_plate = copy_body("plate", "asm_plate")
    m = adsk.core.Matrix3D.create()
    m.setToRotation(math.pi, adsk.core.Vector3D.create(1, 0, 0),
                    adsk.core.Point3D.create(0, 0, 0))
    move(asm_plate, m)                      # flip: spring bumps downward
    translate(asm_plate, 0.0, -20.6, 5.9)   # into frame-model coords
    rot_z180(asm_plate)
    rot_x(asm_plate, A)
    hover_v = tuple(T_F[i] + V_UP[i] * 45.0 for i in range(3))
    translate(asm_plate, *hover_v)

    asm_lid = copy_body("lid", "asm_lid")
    translate(asm_lid, 0.0, -76.4 + 18.0, 41.2)   # half-inserted

    # visibility for shot A: only the screen module
    vis_all = {}
    for i in range(comp.bRepBodies.count):
        b = comp.bRepBodies.item(i)
        vis_all[b.name] = b.isLightBulbOn
        b.isLightBulbOn = b.name in ("asm_frame", "asm_panel", "asm_plate")
    app.activeViewport.fit()
    print("STAGE_A_READY")
