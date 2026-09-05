# Desk stand — Waveshare 7.3" e-Paper (E) + Pico 2 W (Rev 2.1)

Four flat 3D-printed parts, zero supports, zero glue, zero screws.
Every insertion path (not just the final positions) is verified by
boolean sweeps in `fusion_scripts/r8_motion.py` — if a part couldn't
physically slide or drop into place, the build fails loudly.

**How it goes together (all joints are slides or drop-ins):**

1. **panel → frame**: slides down from the frame's open top edge in
   its own plane, seats on the bottom ledge (exact vertical position),
   four side ribs center it with a light scratch-fit.
2. **plate → frame**: slides down the channel just behind the panel,
   dome-side toward the glass, under the side lips, clicks past two
   detents onto the stops. The six domes press the panel gently.
3. **screen module → base**: drops into the full-width 12° slot;
   gravity holds it; the ribbon falls through the notch behind the
   slot into the box.
4. **HAT / Pico → base**: drop into their fenced, labeled bays with
   all cables attached (HAT rests on two pedestals so its 40-pin
   socket hangs free; Pico lies loose; micro-USB cable exits the notch
   in the right wall).
5. **lid → base**: slides in through the rear-wall door until it hits
   the front stops; its ridge tail stays outside as the grip.

| File | What |
|---|---|
| `print/epaper73_H2C_PLA.3mf` | **Open in Bambu Studio → Slice → Print.** 4 parts arranged, H2C 0.4, 0.20mm Standard, Bambu PLA Basic, Textured PEI, supports OFF |
| `print/epaper73_*_print.stl` | Same four parts as pre-oriented STLs |
| `epaper73_case.step` | Full CAD + named `proxy_*` hardware reference bodies |
| `fusion_scripts/` | Regenerate/verify: `reset.py` → `r1_screen.py` → `r2_base.py` → `r3_check.py` (seated interference) → `r8_motion.py` (insertion sweeps) → `r4_export.py` → `make_print_meshes.py`. `r9_debug.py` probes material at specific spots; `r5/r6/r7/vis` pose explanatory views |

Window geometry uses the panel's real active-area offsets (borders
5.1 L/R, 4.7 top, 10.5 bottom per the Waveshare drawing) with the
panel *seated on the ledge*, so the visible area is deterministic.

## Printing (H2C, PLA)

All parts print as placed, flat side down, **no supports** (the only
overhangs are two 2.2 mm lip bridges in the frame and short lip strips
in the base — trivial bridging for the H2C). Keep filament shrinkage
at 100% and don't rescale; FDM clearances are modeled in: panel
+0.8/side (ribs re-center it), plate 0.5, screen slot 0.8, lid 0.2
thickness play, bays ≥0.8.

## Tuning (one constant each)

- Panel slides too hard/loose → rib reach in `r1_screen.py`
  (`85.0` in the rib boxes; ribs crush 0.1 nominal).
- Plate click too weak/strong → detent bump reach (`CH_X - 0.4`).
- Screen wobbles in slot → `SLOT_Y0/SLOT_Y1` in `r2_base.py` (gap 8.4
  vs frame 7.6).
- Lid slides hard → `LID_W` (175.0 in 175.2) or the door cut (±87.7).
- White border visible → `WIN_V0/WIN_V1/WIN_X` (active +0.8 reveal).

To reflash the Pico: slide the lid out, press BOOTSEL, replug.
