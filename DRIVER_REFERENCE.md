# Waveshare 7.3" e-Paper HAT (E) + Raspberry Pi Pico 2 W — Référence driver

Documentation destinée à un agent de code devant produire des affichages sur
cet écran. Le driver (`main.py`) est validé et fonctionnel. Lis cette page
avant d'écrire du code : elle contient les contraintes non évidentes qui font
échouer la plupart des tentatives naïves.

---

## 1. Résumé matériel

- **Écran** : Waveshare 7.3 pouces e-Paper HAT (E), technologie Spectra 6.
- **Résolution** : 800 × 480 pixels, **6 couleurs** (pas de niveaux de gris).
- **MCU** : Raspberry Pi Pico 2 W (RP2350), MicroPython.
- **Bus** : SPI 4 fils, interrupteur « SPI Select » du HAT sur **0**.

### Câblage (ne pas changer sans mettre à jour les constantes du driver)

| Signal | Fil    | Broche Pico | Constante |
|--------|--------|-------------|-----------|
| VCC    | gris   | 3V3 OUT (36)| —         |
| GND    | marron | GND         | —         |
| DIN    | bleu   | GP11        | `DIN_PIN` |
| CLK    | jaune  | GP10        | `CLK_PIN` |
| CS     | orange | GP9         | `CS_PIN`  |
| DC     | vert   | GP8         | `DC_PIN`  |
| RST    | blanc  | GP12        | `RST_PIN` |
| BUSY   | violet | GP13        | `BUSY_PIN`|

---

## 2. Contrainte critique : transport en BIT-BANG, pas en SPI matériel

**Le driver communique volontairement en bit-bang** (GPIO logiciels), pas via
le périphérique SPI matériel. Ce n'est pas un oubli : sur ce montage (fils
Dupont longs + nappe), le SPI matériel à ≥1 MHz corrompt les commandes et
l'écran ne répond pas — le programme se termine « avec succès » sans que rien
ne s'affiche. Le bit-bang, plus lent, est tolérant et fiable.

**Un agent ne doit PAS « optimiser » en réintroduisant `machine.SPI`** sans
revalider sur le matériel réel. En particulier, initialiser `machine.SPI` sur
GP10/GP11 puis vouloir rebasculer ces broches en GPIO échoue (les broches
restent capturées par le périphérique). Reste en bit-bang.

Conséquence pratique : l'envoi du framebuffer (192 000 octets) prend ~15-20 s,
suivi du rafraîchissement e-paper de ~20-30 s. C'est normal.

---

## 3. API du driver (`EPD_7in3E` dans `main.py`)

```python
from main import EPD_7in3E, big_text, big_text_centered
from main import BLACK, WHITE, YELLOW, RED, BLUE, GREEN

epd = EPD_7in3E()          # reset + init + verifie que l'ecran reagit
                           # (leve RuntimeError si aucun contact)

epd.image                  # framebuf.FrameBuffer (GS4_HMSB), 800x480
                           # -> toutes les primitives framebuf sont dispo

epd.image.fill(WHITE)                       # efface le tampon
epd.image.fill_rect(x, y, w, h, RED)        # rectangle plein
epd.image.rect(x, y, w, h, BLACK)           # contour
epd.image.line(x1, y1, x2, y2, BLUE)
epd.image.ellipse(cx, cy, rx, ry, GREEN, True)  # True = plein
epd.image.pixel(x, y, BLACK)
epd.image.text("ascii", x, y, BLACK)        # police 8 px (petite)

epd.display()              # envoie le tampon + rafraichit l'ecran
epd.clear(WHITE)           # raccourci : remplit + affiche
epd.sleep()                # OBLIGATOIRE apres usage (voir section 5)
```

### Texte lisible

`epd.image.text()` n'a qu'une police 8 px, minuscule sur 800×480. Utilise les
helpers fournis :

```python
big_text(epd.image, "Texte", x, y, BLACK, scale=3)      # x3 = ~24 px de haut
big_text_centered(epd.image, "Titre", y, RED, scale=4)  # centre horizontalement
```

`big_text` agrandit la police bitmap par duplication de pixels (rendu carré,
pas d'anticrénelage — normal pour e-paper).

### Modèle d'usage type

```python
epd = EPD_7in3E()
try:
    epd.image.fill(WHITE)
    # ... dessin ...
    epd.display()
finally:
    epd.sleep()            # meme en cas d'erreur
```

---

## 4. Les 6 couleurs

Seules ces 6 valeurs sont affichables. Toute autre valeur donne un résultat
imprévisible. Ce sont les codes 4 bits du contrôleur :

```python
BLACK  = 0x0
WHITE  = 0x1
YELLOW = 0x2
RED    = 0x3
BLUE   = 0x5
GREEN  = 0x6
```

Pas de dégradés ni de demi-teintes natifs. Pour simuler des nuances, il faut
faire du **dithering** côté préparation d'image (voir `photo_display.py`).

Le framebuffer est en `framebuf.GS4_HMSB` : 4 bits par pixel, 2 pixels par
octet, nibble de poids fort = pixel de gauche. Un octet `0x11` = deux pixels
blancs. Taille du tampon = 800 × 480 ÷ 2 = 192 000 octets (OK dans les 520 Ko
du Pico 2 W).

---

## 5. Règles de sécurité e-paper (impératif)

- **Toujours appeler `epd.sleep()` après affichage.** Laisser l'écran alimenté
  hors veille le maintient sous haute tension interne → dommage irréversible.
  Le mettre dans un `finally`.
- **Espacer les rafraîchissements d'au moins 3 minutes** en usage régulier.
- Si l'écran reste alimenté, le rafraîchir **au moins une fois par 24 h** ;
  l'effacer en blanc avant un stockage prolongé.
- Plage de température idéale **15–35 °C** ; éviter le plein soleil prolongé.
- Ne jamais brancher/débrancher les fils quand le Pico est sous tension.

---

## 6. Diagnostic si l'écran ne réagit plus

Le driver lève `RuntimeError` au démarrage s'il ne détecte aucune réaction
(BUSY ne passe pas à 0 après le power-on `0x04`). Causes, par ordre de
probabilité sur ce montage :

1. **Contact intermittent** d'un fil CLK/DIN/DC/CS dans son logement Dupont →
   réenfoncer fermement chaque fil.
2. Interrupteur SPI Select qui aurait bougé (doit être sur **0**).
3. Nappe FFC mal insérée (droite, à fond, loquets rabattus, des deux côtés).

Le reset (RST) et BUSY n'utilisent pas les lignes SPI : un écran qui « réagit
au reset » mais « ne prend aucune commande » = problème isolé sur CLK/DIN/DC/CS.

---

## 7. Notes pour l'agent

- Ne pas dépasser les 6 couleurs listées.
- Ne pas réintroduire `machine.SPI` sans revalidation matérielle.
- Coordonnées : origine (0,0) en haut à gauche, x → droite, y → bas.
- Un seul `epd.display()` par image finale (chaque refresh coûte ~30 s et use
  l'écran) ; composer entièrement le tampon avant d'appeler `display()`.
- `big_text` ne gère que l'ASCII (la police framebuf est ASCII). Pour les
  accents, soit les retirer, soit fournir une police bitmap étendue.
