import os

import adsk.core
import adsk.fusion

OUT = r"C:\Users\charl\Documents\GitHub\e-paper_pico-2W\hardware"


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    comp = design.rootComponent
    os.makedirs(OUT, exist_ok=True)
    mgr = design.exportManager

    opts = mgr.createSTEPExportOptions(os.path.join(OUT, "epaper73_case.step"))
    mgr.execute(opts)
    print("exported epaper73_case.step")

    for name in ("frame", "plate", "base", "lid"):
        body = None
        for i in range(comp.bRepBodies.count):
            if comp.bRepBodies.item(i).name == name:
                body = comp.bRepBodies.item(i)
        path = os.path.join(OUT, f"epaper73_{name}.stl")
        sopts = mgr.createSTLExportOptions(body, path)
        sopts.meshRefinement = \
            adsk.fusion.MeshRefinementSettings.MeshRefinementHigh
        mgr.execute(sopts)
        print(f"exported epaper73_{name}.stl ({body.volume * 1000:.0f} mm3)")
