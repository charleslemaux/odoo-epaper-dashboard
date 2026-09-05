import adsk.core
import adsk.fusion


def run(_context: str):
    app = adsk.core.Application.get()
    design = adsk.fusion.Design.cast(app.activeProduct)
    tl = design.timeline
    n = tl.count
    for i in range(n - 1, -1, -1):
        item = tl.item(i)
        ent = item.entity
        if ent is not None and ent.isValid:
            ent.deleteMe()
    print("timeline items removed:", n, "-> now", tl.count)
    print("bodies left:", design.rootComponent.bRepBodies.count)
