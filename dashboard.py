# -----------------------------------------------------------------------------
# Image complexe de demonstration - Waveshare 7.3" e-Paper (E) + Pico 2 W
# Reutilise le driver bit-bang de main.py (import).
#
# Montre : degrades par dithering, formes, graphe de donnees, palette,
# roue chromatique, cadre. Un seul refresh.
# -----------------------------------------------------------------------------

from main import (EPD_7in3E, big_text, big_text_centered,
                  BLACK, WHITE, YELLOW, RED, BLUE, GREEN,
                  EPD_WIDTH, EPD_HEIGHT)
import time
import math

# Palette ordonnee clair -> fonce, utile pour le dithering
PALETTE = [WHITE, YELLOW, GREEN, BLUE, RED, BLACK]


def dither_rect(fb, x, y, w, h, c_light, c_dark, ratio):
    """Remplit un rectangle en melangeant 2 couleurs facon 'ordered dither'
    (matrice de Bayer 4x4). ratio 0.0 = tout clair, 1.0 = tout fonce."""
    bayer = (
        (0, 8, 2, 10),
        (12, 4, 14, 6),
        (3, 11, 1, 9),
        (15, 7, 13, 5),
    )
    thr = ratio * 16
    for j in range(h):
        for i in range(w):
            c = c_dark if bayer[j & 3][i & 3] < thr else c_light
            fb.pixel(x + i, y + j, c)


def h_gradient(fb, x, y, w, h, c_light, c_dark):
    """Degrade horizontal clair -> fonce par bandes ditherees."""
    steps = 32
    sw = w // steps
    for s in range(steps):
        r = s / (steps - 1)
        dither_rect(fb, x + s * sw, y, sw, h, c_light, c_dark, r)


def main():
    epd = None
    try:
        print("Init ecran...")
        epd = EPD_7in3E()

        print("Composition (image complexe)...")
        img = epd.image
        img.fill(WHITE)

        # --- Cadre general ---
        img.rect(0, 0, EPD_WIDTH, EPD_HEIGHT, BLACK)
        img.rect(1, 1, EPD_WIDTH - 2, EPD_HEIGHT - 2, BLACK)

        # --- Bandeau titre ---
        img.fill_rect(2, 2, EPD_WIDTH - 4, 60, BLUE)
        big_text_centered(img, "PICO 2 W  -  e-Paper 7.3 (E)", 22, WHITE, scale=3)

        # --- Colonne gauche : degrades ditheres ---
        gx, gy = 20, 80
        big_text(img, "Degrades (dithering)", gx, gy, BLACK, scale=2)
        h_gradient(img, gx, gy + 25, 340, 40, WHITE, BLACK)
        big_text(img, "blanc -> noir", gx, gy + 70, BLACK, scale=1)

        h_gradient(img, gx, gy + 95, 340, 40, YELLOW, RED)
        big_text(img, "jaune -> rouge", gx, gy + 140, BLACK, scale=1)

        h_gradient(img, gx, gy + 165, 340, 40, WHITE, BLUE)
        big_text(img, "blanc -> bleu", gx, gy + 210, BLACK, scale=1)

        # --- Colonne droite : mini graphe facon 'donnees' ---
        cx, cy, cw, ch = 400, 105, 360, 170
        img.rect(cx, cy, cw, ch, BLACK)
        big_text(img, "Signal", cx + 8, cy - 22, BLACK, scale=2)
        # axes
        img.line(cx, cy + ch, cx + cw, cy + ch, BLACK)
        # deux courbes sinusoidales echantillonnees
        prev1 = prev2 = None
        for px in range(cw):
            t = px / cw * 4 * math.pi
            v1 = cy + ch // 2 - int((ch / 2 - 8) * math.sin(t))
            v2 = cy + ch // 2 - int((ch / 2 - 8) * math.sin(t * 0.5 + 1) * 0.6)
            if prev1 is not None:
                img.line(cx + px - 1, prev1, cx + px, v1, RED)
                img.line(cx + px - 1, prev2, cx + px, v2, GREEN)
            prev1, prev2 = v1, v2

        # --- Roue chromatique (6 secteurs des 6 couleurs) ---
        wheel_cx, wheel_cy, wheel_r = 640, 370, 75
        for a in range(360):
            idx = (a * 6) // 360
            color = PALETTE[idx]
            rad = math.radians(a)
            for rr in range(wheel_r):
                x = wheel_cx + int(rr * math.cos(rad))
                y = wheel_cy + int(rr * math.sin(rad))
                img.pixel(x, y, color)
        img.ellipse(wheel_cx, wheel_cy, wheel_r, wheel_r, BLACK)
        img.ellipse(wheel_cx, wheel_cy, 22, 22, WHITE, True)
        img.ellipse(wheel_cx, wheel_cy, 22, 22, BLACK)

        # --- Bandeau bas : palette + formes ---
        by = 300
        big_text(img, "Palette :", 20, by, BLACK, scale=2)
        swatches = [(BLACK, "K"), (WHITE, "W"), (YELLOW, "Y"),
                    (RED, "R"), (BLUE, "B"), (GREEN, "G")]
        sx = 20
        for color, lbl in swatches:
            img.fill_rect(sx, by + 25, 44, 44, color)
            img.rect(sx, by + 25, 44, 44, BLACK)
            sx += 52

        # quelques formes decoratives
        img.ellipse(120, 420, 30, 30, RED, True)
        img.ellipse(120, 420, 30, 30, BLACK)
        img.fill_rect(170, 392, 56, 56, GREEN)
        img.rect(170, 392, 56, 56, BLACK)
        for k in range(6):
            img.line(250, 448, 250 + k * 12, 392, BLUE)

        # --- Pied de page ---
        big_text_centered(img, "800 x 480  -  6 couleurs  -  bit-bang",
                          EPD_HEIGHT - 24, BLACK, scale=1)

        print("Envoi (~15-20 s) + rafraichissement (~20-30 s)...")
        epd.display()
        print("Termine !")

    finally:
        if epd:
            print("Mise en veille (obligatoire).")
            epd.sleep()


if __name__ == "__main__":
    main()
