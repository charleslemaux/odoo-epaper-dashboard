import adsk.core
import adsk.fusion


def run(_context: str):
    app = adsk.core.Application.get()
    print("fusion version:", app.version)
    doc = app.activeDocument
    print("active document:", doc.name, "| saved:", doc.isSaved,
          "| modified:", doc.isModified)
    product = app.activeProduct
    print("active product:", product.productType)
    design = adsk.fusion.Design.cast(product)
    if design is None:
        print("NOT A DESIGN - aborting")
        return
    print("design type:",
          "parametric" if design.designType == adsk.fusion.DesignTypes.ParametricDesignType
          else "direct")
    units = design.unitsManager
    print("default units:", units.defaultLengthUnits)
    root = design.rootComponent
    print("root component:", root.name)
    print("existing bodies:", root.bRepBodies.count,
          "| occurrences:", root.occurrences.count,
          "| sketches:", root.sketches.count)
