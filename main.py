# -----------------------------------------------------------------------------
# Waveshare 7.3" e-Paper HAT (E) - Spectra 6 couleurs - 800x480
# Driver MicroPython pour Raspberry Pi Pico 2 W (RP2350)
#
# Transport : BIT-BANG PUR (aucun SPI materiel).
# C'est exactement la methode qui a fait clignoter ton ecran au test
# d'autodetection. Plus lent (~15-20 s d'envoi) mais fiable sur ton cablage.
#
# Cablage (SPI Select sur 0) :
#   VCC(gris)->3V3(36)  GND(marron)->GND  DIN(bleu)->GP11  CLK(jaune)->GP10
#   CS(orange)->GP9  DC(vert)->GP8  RST(blanc)->GP12  BUSY(violet)->GP13
# -----------------------------------------------------------------------------

from machine import Pin
import framebuf
import time

RST_PIN = 12
DC_PIN = 8
CS_PIN = 9
BUSY_PIN = 13
CLK_PIN = 10
DIN_PIN = 11

EPD_WIDTH = 800
EPD_HEIGHT = 480

# Couleurs (4 bits/pixel) - codes du controleur Spectra 6
BLACK = 0x0
WHITE = 0x1
YELLOW = 0x2
RED = 0x3
BLUE = 0x5
GREEN = 0x6


class EPD_7in3E:
    def __init__(self):
        self.reset_pin = Pin(RST_PIN, Pin.OUT)
        self.busy_pin = Pin(BUSY_PIN, Pin.IN, Pin.PULL_UP)
        self.cs_pin = Pin(CS_PIN, Pin.OUT, value=1)
        self.dc_pin = Pin(DC_PIN, Pin.OUT, value=0)
        self.clk_pin = Pin(CLK_PIN, Pin.OUT, value=0)
        self.din_pin = Pin(DIN_PIN, Pin.OUT, value=0)

        self.width = EPD_WIDTH
        self.height = EPD_HEIGHT
        self.buffer = bytearray(self.width * self.height // 2)
        self.image = framebuf.FrameBuffer(
            self.buffer, self.width, self.height, framebuf.GS4_HMSB
        )

        # Init avec verification que l'ecran reagit vraiment (jusqu'a 3 essais)
        ok = False
        for attempt in range(3):
            self.reset()
            self._wait_idle_short()
            self._send_init()
            if self._power_on_reacts():
                ok = True
                break
            print("  (essai %d : pas de reaction, nouvelle tentative...)" % (attempt + 1))
            time.sleep_ms(200)
        if not ok:
            raise RuntimeError(
                "L'ecran ne reagit pas. Reenfonce fermement chaque fil "
                "CLK/DIN/DC/CS (contact intermittent le plus probable), "
                "puis relance."
            )

    # --- Bas niveau bit-bang (SPI mode 0, MSB first) --------------------------
    def reset(self):
        self.reset_pin(1); time.sleep_ms(20)
        self.reset_pin(0); time.sleep_ms(2)
        self.reset_pin(1); time.sleep_ms(20)

    def _tx(self, data):
        clk = self.clk_pin
        din = self.din_pin
        for b in data:
            din((b >> 7) & 1); clk(1); clk(0)
            din((b >> 6) & 1); clk(1); clk(0)
            din((b >> 5) & 1); clk(1); clk(0)
            din((b >> 4) & 1); clk(1); clk(0)
            din((b >> 3) & 1); clk(1); clk(0)
            din((b >> 2) & 1); clk(1); clk(0)
            din((b >> 1) & 1); clk(1); clk(0)
            din(b & 1);        clk(1); clk(0)

    def send_command(self, cmd):
        self.dc_pin(0); self.cs_pin(0)
        self._tx(bytes([cmd]))
        self.cs_pin(1)

    def send_data(self, data):
        self.dc_pin(1); self.cs_pin(0)
        if isinstance(data, int):
            self._tx(bytes([data]))
        else:
            self._tx(data)
        self.cs_pin(1)

    def _wait_idle_short(self, timeout_ms=5000):
        time.sleep_ms(30)
        t = time.ticks_ms()
        while self.busy_pin.value() == 0:
            if time.ticks_diff(time.ticks_ms(), t) > timeout_ms:
                return
            time.sleep_ms(10)

    def wait_until_idle(self, timeout_ms=60_000):
        time.sleep_ms(50)
        t = time.ticks_ms()
        while self.busy_pin.value() == 0:
            if time.ticks_diff(time.ticks_ms(), t) > timeout_ms:
                raise RuntimeError("Timeout BUSY (>%d s)" % (timeout_ms // 1000))
            time.sleep_ms(20)

    def _power_on_reacts(self, watch_ms=3000):
        # power-on, et verifie que BUSY passe a 0 (l'ecran se met au travail)
        self.send_command(0x04)
        t = time.ticks_ms()
        while time.ticks_diff(time.ticks_ms(), t) < watch_ms:
            if self.busy_pin.value() == 0:
                self.wait_until_idle()
                return True
            time.sleep_ms(2)
        return False

    # --- Init officielle epd7in3e ---------------------------------------------
    def _send_init(self):
        self.send_command(0xAA)
        self.send_data(bytearray([0x49, 0x55, 0x20, 0x08, 0x09, 0x18]))
        self.send_command(0x01); self.send_data(0x3F)
        self.send_command(0x00); self.send_data(bytearray([0x5F, 0x69]))
        self.send_command(0x03); self.send_data(bytearray([0x00, 0x54, 0x00, 0x44]))
        self.send_command(0x05); self.send_data(bytearray([0x40, 0x1F, 0x1F, 0x2C]))
        self.send_command(0x06); self.send_data(bytearray([0x6F, 0x1F, 0x17, 0x49]))
        self.send_command(0x08); self.send_data(bytearray([0x6F, 0x1F, 0x1F, 0x22]))
        self.send_command(0x30); self.send_data(0x03)
        self.send_command(0x50); self.send_data(0x3F)
        self.send_command(0x60); self.send_data(bytearray([0x02, 0x00]))
        self.send_command(0x61); self.send_data(bytearray([0x03, 0x20, 0x01, 0xE0]))
        self.send_command(0x84); self.send_data(0x01)
        self.send_command(0xE3); self.send_data(0x2F)

    # --- Affichage ------------------------------------------------------------
    def turn_on_display(self):
        self.send_command(0x04); self.wait_until_idle()   # power on
        self.send_command(0x12); self.send_data(0x00)      # refresh
        self.wait_until_idle()
        self.send_command(0x02); self.send_data(0x00)      # power off
        self.wait_until_idle()

    def display(self):
        self.send_command(0x10)
        self.send_data(self.buffer)
        self.turn_on_display()

    def clear(self, color=WHITE):
        self.image.fill(color)
        self.display()

    def sleep(self):
        self.send_command(0x07)
        self.send_data(0xA5)
        time.sleep_ms(20)


# --- Texte agrandi (police framebuf native = 8 px, trop petite en 800x480) ----
def big_text(fb, s, x, y, color, scale=2):
    w = len(s) * 8
    tmp_buf = bytearray(w * 8 // 2 + 8)
    tmp = framebuf.FrameBuffer(tmp_buf, w, 8, framebuf.GS4_HMSB)
    tmp.fill(WHITE)
    tmp.text(s, 0, 0, BLACK)
    for dy in range(8):
        for dx in range(w):
            if tmp.pixel(dx, dy) != WHITE:
                fb.fill_rect(x + dx * scale, y + dy * scale, scale, scale, color)


def big_text_centered(fb, s, y, color, scale=2, x0=0, width=EPD_WIDTH):
    x = x0 + (width - len(s) * 8 * scale) // 2
    big_text(fb, s, x, y, color, scale)


# ==============================================================================
# Bienvenue + test des 6 couleurs (une bande legendee par couleur)
# ==============================================================================
if __name__ == "__main__":
    epd = None
    try:
        print("Initialisation de l'ecran (bit-bang)...")
        epd = EPD_7in3E()

        print("Composition de l'image...")
        epd.image.fill(WHITE)

        big_text_centered(epd.image, "Bienvenue !", 25, BLACK, scale=4)
        big_text_centered(epd.image, "Test des 6 couleurs - Waveshare 7.3 (E)", 95, RED, scale=2)

        bands = [
            (BLACK, "NOIR", WHITE),
            (WHITE, "BLANC", BLACK),
            (YELLOW, "JAUNE", BLACK),
            (RED, "ROUGE", WHITE),
            (BLUE, "BLEU", WHITE),
            (GREEN, "VERT", WHITE),
        ]
        top = 130
        bh = EPD_HEIGHT - top
        bw = EPD_WIDTH // len(bands)
        for i, (color, label, txt) in enumerate(bands):
            x = i * bw
            epd.image.fill_rect(x, top, bw, bh, color)
            epd.image.rect(x, top, bw, bh, BLACK)
            lx = x + (bw - len(label) * 8 * 2) // 2
            big_text(epd.image, label, lx, top + 30, txt, scale=2)

        print("Envoi (bit-bang, ~15-20 s) puis rafraichissement (~20-30 s)...")
        epd.display()
        print("Affichage termine !")

    finally:
        if epd:
            print("Mise en veille de l'ecran (obligatoire).")
            epd.sleep()
