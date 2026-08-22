[Français](README.md) · [English](README.en.md)

# LocalDock

Lanceur **local-only** pour tes serveurs de dev en loopback.  
**Gratuit** · **100 % local-first** · **Mr-Aurevo-X** · mises à jour **non garanties**

**v0.1.0** — deux packs officiels, même app, même release :

| | Windows | Linux |
|---|---|---|
| **Fichier** | [`LocalDock.zip`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/LocalDock.zip) | [`org.mraurevox.LocalDock.flatpak`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/org.mraurevox.LocalDock.flatpak) |
| **Contenu** | `localdock.exe` + `Lancer.cmd` | runtime GNOME 49, id `org.mraurevox.LocalDock` |
| **Install** | Extraire → `Lancer.cmd` | `flatpak install --user` (voir plus bas) |

[Toutes les releases](https://github.com/Mr-Aurevo-X/LocalDock/releases)

## Aperçu

- **Accueil** — nombre de ports ouverts, apps enregistrées à la main, racines de confiance + **Parcourir** / scan
- **Ports ouverts** — nom du process, chemin, ligne de commande, ouvert depuis…
- **Historique** — journal local sur **ce PC**
- FR | EN dans l’app · chrome Void Glow (`min` / `max` / `close`)

## Pourquoi LocalDock

- Gratuit (usage non commercial, `LICENSE`) — pas de compte, pas d’abonnement
- Local-first — **pas de télémétrie éditeur**, **pas de portail HTTP**
- LocalDock n’écoute **aucun** port ; tes apps enfants sont orientées vers `127.0.0.1`
- Seule connexion hors machine : vérif. version GitHub **si tu la laisses activée** dans À propos (lecture seule, pas de téléchargement)
- Le registre (`apps.json`) est **propre à ce PC et à ce compte** — les chemins ne suivent pas une install ailleurs

## Sur ton PC

### Windows

| Quoi | Où |
|------|-----|
| **App** | Extrais `LocalDock.zip`, lance `Lancer.cmd` (ou `localdock.exe`) |
| Registre | `%APPDATA%\LocalDock\apps.json` |
| Historique | `%APPDATA%\LocalDock\history.json` |
| Préférences | `%LOCALAPPDATA%\Mr-Aurevo-X\user-settings.json` (partagé entre apps) |

### Linux

| Quoi | Où |
|------|-----|
| **App (Flatpak)** | `flatpak run org.mraurevox.LocalDock` |
| Données Flatpak | `~/.var/app/org.mraurevox.LocalDock/config/LocalDock/` (`apps.json`, `history.json`) |
| Préférences Flatpak | `~/.var/app/org.mraurevox.LocalDock/config/Mr-Aurevo-X/user-settings.json` |
| Données natives (sources) | `~/.config/LocalDock/` |
| Préférences natives | `~/.config/Mr-Aurevo-X/user-settings.json` |

Le Flatpak liste et lance les localhost **de l’hôte** (`ss` + `flatpak-spawn --host`), pas seulement le sandbox. Un Windows monté (`/run/media/…`) se scanne ; **Importer Windows** remap `C:\…`.

## Lancer

### Windows

1. Télécharge [`LocalDock.zip`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/LocalDock.zip)  
2. Extrais où tu veux  
3. Lance `Lancer.cmd` (ou `localdock.exe`)

Windows peut afficher un avertissement : les binaires ne sont **pas signés**. C’est **SmartScreen**, pas un antivirus qui dit « virus ».

### Linux

1. [Flatpak](https://flatpak.org/setup/) + runtime **GNOME 49** :

```bash
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y flathub org.gnome.Platform//49
```

2. Pack officiel :

```bash
curl -fL -o org.mraurevox.LocalDock.flatpak \
  https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/org.mraurevox.LocalDock.flatpak
flatpak install --user -y ./org.mraurevox.LocalDock.flatpak
```

3. Menu applications + raccourci Bureau (**sans** cloner le repo) :

```bash
mkdir -p ~/.local/share/applications
SRC=~/.local/share/flatpak/exports/share/applications/org.mraurevox.LocalDock.desktop
cp -f "$SRC" ~/.local/share/applications/
. ~/.config/user-dirs.dirs 2>/dev/null || true
DESK="${XDG_DESKTOP_DIR:-$HOME/Bureau}"
[ -d "$DESK" ] || DESK="$HOME/Desktop"
cp -f "$SRC" "$DESK/LocalDock.desktop"
chmod +x ~/.local/share/applications/org.mraurevox.LocalDock.desktop "$DESK/LocalDock.desktop"
```

Depuis un clone : `bash packaging/installer-raccourci-flatpak.sh` (même logique Bureau / Desktop).

4. `flatpak run org.mraurevox.LocalDock`

Sources (optionnel) : `bash LANCER.sh` · raccourci natif : `bash INSTALLER-RACCOURCI.sh` · rebuild Flatpak : `bash packaging/build-flatpak.sh`.

## Version officielle uniquement

La seule version que je cautionne :

**https://github.com/Mr-Aurevo-X/LocalDock**

Un fork ou une copie modifiée ailleurs **n’est pas** ma version — je n’en suis pas responsable.  
Logiciel **tel quel**, sans garantie — détails dans `LICENSE` et `PRIVACY.md`.

## Légal

- **100 % gratuit** (usage non commercial, PolyForm Noncommercial 1.0.0)
- **Local-first** — pas de télémétrie éditeur
- **Mise à jour non garantie** — vérif. GitHub optionnelle, pas d’install auto
- **Copyright © 2026 Mr-Aurevo-X**

À propos dans l’app : CGU, confidentialité, mentions, licences, chemins locaux.

## Soutien (optionnel)

Si le boulot te plaît, un café — sinon profite.

[![Discord](https://img.shields.io/badge/Discord-Mr--Aurevo--X-5865F2?style=for-the-badge&logo=discord&logoColor=white&labelColor=050807)](https://discord.com/users/406891052516114442)
[![PayPal](https://img.shields.io/badge/PayPal-Donate-39ff14?style=for-the-badge&logo=paypal&logoColor=00f0ff&labelColor=050807)](https://www.paypal.com/paypalme/aurevo1)
[![Revolut](https://img.shields.io/badge/Revolut-mr__aurevo__x-00f0ff?style=for-the-badge&logo=revolut&logoColor=39ff14&labelColor=050807)](https://revolut.me/mr_aurevo_x)

---

Rêvée par **Mr-Aurevo-X**. Cursor a réalisé le rêve.
