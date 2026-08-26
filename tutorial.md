# Tutoriel : écran Waveshare 7.3" e-Paper HAT (E) + Raspberry Pi Pico 2 W (headers)

Ton matériel confirmé : **Pico 2 W** (puce RP2350, 520 Ko de RAM — parfait pour cet écran), HAT (E) avec câble **8 fils**, et interrupteur **SPI Select** sur le HAT.

---

## Étape 1 — Le matériel et comment il s'assemble

- **Le panneau e-paper 7.3"** : relié au HAT par la grande nappe blanche (FFC). Sur ta photo elle est déjà connectée des deux côtés — vérifie juste qu'elle est insérée bien droite et à fond, loquets fermés.
- **Le HAT bleu** : c'est l'interface entre l'écran et le microcontrôleur.
- **Le câble blanc 8 fils (PH2.0 → Dupont femelle)** : c'est le lien HAT ↔ Pico.
- **Ton Pico 2 W avec headers** : les fils Dupont femelles s'enfichent directement sur ses broches.

> **Pourquoi on ne branche pas le HAT directement sur le Pico :** le connecteur 40 broches du HAT est prévu pour le brochage d'un Raspberry Pi "classique" (Pi 4/5/Zero). Le Pico a aussi 40 broches, mais dans un ordre totalement différent — le brancher dessus enverrait du courant au mauvais endroit. Avec un Pico, on passe **toujours** par le câble 8 fils. C'est le montage normal, tu ne rates rien.

---

## Étape 2 — L'interrupteur "SPI Select" : position 0

Sur le HAT, à côté du connecteur de nappe, il y a un petit interrupteur **SPI Select** avec deux positions sérigraphiées :
- `1` = 3-line SPI
- `0` = **4-line SPI ← c'est celle qu'il te faut**

Le mode 4 fils utilise la broche DC (Data/Command), comme le fait le code standard Waveshare et le script fourni. Vérifie qu'il est sur **0** avant de commencer.

---

## Étape 3 — Câblage des 8 fils vers le Pico

Branche le connecteur blanc PH2.0 sur le HAT (un seul sens possible). Les noms des signaux sont **sérigraphiés sur le HAT** à côté du connecteur : fie-toi à eux en priorité. Les couleurs Waveshare habituelles sont indiquées ci-dessous à titre de repère (elles correspondent à ton câble) :

| Signal HAT | Couleur (habituelle) | Broche Pico 2 W | N° physique |
|---|---|---|---|
| VCC | Gris | 3V3 (OUT) | 36 |
| GND | Marron | GND | 38 (ou n'importe quel GND) |
| DIN | Bleu | GP11 | 15 |
| CLK | Jaune | GP10 | 14 |
| CS | Orange | GP9 | 12 |
| DC | Vert | GP8 | 11 |
| RST | Blanc | GP12 | 16 |
| BUSY | Violet | GP13 | 17 |

**Comment retrouver GP8–GP13 sur ton Pico :** les noms sont sérigraphiés **au dos** de la carte (visibles sur ta 2e photo : GP8, GP9, GND, GP10, GP11, GP12, GP13 sont alignés sur le même côté, dans la moitié opposée à l'USB). C'est pratique : tous les fils de signaux vont sur des broches voisines. 3V3(OUT) et le GND de la broche 38 sont sur l'autre rangée, côté USB.

**Règle de sécurité :** jamais de fil du HAT sur VBUS ou VSYS (5 V). L'écran fonctionne en 3,3 V uniquement.

Double-vérifie DIN/CLK et CS/DC (les inversions classiques) avant de brancher l'USB.

---

## Étape 4 — Flasher MicroPython sur le Pico 2 W

Le Pico n'a pas de système d'exploitation : on y installe un firmware une fois, puis on y dépose des scripts. "Flasher" est très simple :

1. Va sur **micropython.org/download/RPI_PICO2_W** et télécharge le fichier `.uf2` le plus récent (bien la version **Pico 2 W**, pas Pico W ni Pico 2).
2. **Maintiens le bouton blanc BOOTSEL** du Pico enfoncé.
3. Tout en le maintenant, branche le câble micro-USB à ton ordi, puis relâche le bouton.
4. Un disque amovible apparaît sur ton ordi (nommé `RP2350`).
5. Glisse-dépose le fichier `.uf2` dessus. Le disque disparaît tout seul : MicroPython est installé. ✅

C'est réversible à volonté et impossible à rater définitivement : au pire, on recommence l'opération BOOTSEL.

> Attention : utilise un câble micro-USB **de données** (certains câbles de charge n'ont pas les fils data et le Pico ne sera jamais détecté).

---

## Étape 5 — Installer Thonny et vérifier la connexion

1. Télécharge **Thonny** (thonny.org) et installe-le.
2. Branche le Pico normalement (sans BOOTSEL cette fois).
3. Dans Thonny, en bas à droite, clique sur la zone de l'interpréteur → choisis **"MicroPython (Raspberry Pi Pico)"** et le port proposé.
4. La console du bas doit afficher un prompt `>>>`. Tape `print("hello")` : si ça répond, la chaîne complète ordi ↔ Pico fonctionne.

---

## Étape 6 — Premier affichage 🎉

1. Ouvre le fichier **`main_epd7in3e.py`** (fourni avec ce tutoriel) dans Thonny.
2. Clique sur le **triangle vert** (Exécuter le script courant).
3. L'écran va rester gris puis **clignoter/flasher pendant ~20-30 s** : c'est le cycle de rafraîchissement normal d'un e-paper couleur, pas un bug.
4. Résultat attendu : bandes de couleurs (noir, rouge, jaune, bleu, vert), quelques formes, et un texte de bienvenue.

Pour que ton programme se lance automatiquement à chaque mise sous tension (sans ordinateur) : dans Thonny, **Fichier → Enregistrer sous… → Raspberry Pi Pico**, nomme le fichier **`main.py`**. Le Pico exécute toujours `main.py` au démarrage.

### Ce que fait le script
- Il embarque un **driver minimal** pour le contrôleur de l'écran (adapté du code officiel Waveshare `epd7in3e`)
- Il crée un framebuffer 800×480 en 6 couleurs (~192 Ko, à l'aise dans les 520 Ko du Pico 2 W)
- Il dessine, envoie l'image, puis met l'écran **en veille** (indispensable, voir ci-dessous)

---

## ⚠️ Règles d'or pour ne pas abîmer l'écran

- **Toujours terminer par `epd.sleep()`** (le script le fait). Laisser l'écran alimenté hors veille le maintient sous haute tension interne → dommages irréversibles.
- En usage régulier, espace les rafraîchissements d'**au moins 3 minutes**.
- S'il reste alimenté, rafraîchis-le **au moins une fois par 24 h** ; efface-le en blanc avant un stockage prolongé.
- Écran couleur = plage idéale **15–35 °C**, évite le plein soleil prolongé.
- Ne jamais brancher/débrancher les fils quand le Pico est sous tension.

---

## Dépannage rapide

| Symptôme | Piste |
|---|---|
| Aucun clignotement au lancement | VCC sur 3V3 (broche 36) ? GND commun ? Nappe FFC bien insérée ? Interrupteur SPI Select sur 0 ? |
| Le script bloque sur l'attente "busy" | BUSY bien sur GP13, RST sur GP12 ; vérifie CS/DC non inversés |
| Thonny ne détecte pas le Pico | Câble USB de charge seulement → prends un câble data ; refais BOOTSEL avec le `.uf2` RPI_PICO2_W |
| Image décalée / couleurs fausses | Nappe FFC de travers, ou fils DIN/CLK inversés |
| `MemoryError` | Firmware Pico W flashé par erreur au lieu de Pico 2 W |

Si le driver fourni pose souci, le code officiel est dans les ressources de la page wiki Waveshare « 7.3inch e-Paper HAT (E) ».

---

## Et ensuite ?

Ton Pico 2 W a le Wi-Fi : idées de suite naturelles → tableau de bord météo, agenda du jour, citations, cadre photo (avec conversion des images en 6 couleurs + dithering côté ordinateur). Le framebuffer te donne déjà `text()`, `line()`, `rect()`, `fill_rect()`, `ellipse()`, `pixel()`.

Bon montage ! 🚀