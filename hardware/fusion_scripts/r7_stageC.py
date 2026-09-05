"""Stage C: seat the boards; leave doc with flat parts + assembled view."""
import adsk.core
import adsk.fusion


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

    for name in ("proxy_hat_pcb", "proxy_hat_parts", "proxy_pico"):
        coll = adsk.core.ObjectCollection.create()
        coll.add(body(name))
        m = adsk.core.Matrix3D.create()
        m.translation = adsk.core.Vector3D.create(0, 0, mm(-40.0))
        moves.add(moves.createInput(coll, m))

    show = {"asm_frame", "asm_panel", "asm_plate", "asm_lid", "base",
            "proxy_hat_pcb", "proxy_hat_parts", "proxy_pico",
            "frame", "plate", "lid"}
    for i in range(comp.bRepBodies.count):
        b = comp.bRepBodies.item(i)
        b.isLightBulbOn = b.name in show
    app.activeViewport.fit()
    print("STAGE_C_READY - doc keeps flat parts + assembled view")
