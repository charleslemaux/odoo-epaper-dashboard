"""Stage B: seat panel+plate into the frame, show base with hovering boards."""
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

    def body(name):
        for i in range(comp.bRepBodies.count):
            if comp.bRepBodies.item(i).name == name:
                return comp.bRepBodies.item(i)
        raise RuntimeError("no body " + name)

    def translate(name, dx, dy, dz):
        coll = adsk.core.ObjectCollection.create()
        coll.add(body(name))
        m = adsk.core.Matrix3D.create()
        m.translation = adsk.core.Vector3D.create(mm(dx), mm(dy), mm(dz))
        moves.add(moves.createInput(coll, m))

    translate("asm_panel", 0.0, -30.0 * S, 30.0 * C)     # seat into pocket
    translate("asm_plate", 0.0, -45.0 * C, -45.0 * S)    # slide down channel
    for name in ("proxy_hat_pcb", "proxy_hat_parts", "proxy_pico"):
        translate(name, 0.0, 0.0, 40.0)                  # hover over bays

    show = {"asm_frame", "asm_panel", "asm_plate", "asm_lid", "base",
            "proxy_hat_pcb", "proxy_hat_parts", "proxy_pico"}
    for i in range(comp.bRepBodies.count):
        b = comp.bRepBodies.item(i)
        b.isLightBulbOn = b.name in show
    app.activeViewport.fit()
    print("STAGE_B_READY")
