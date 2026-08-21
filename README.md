[Français](README.md) · [English](README.en.md)

# LocalDock

Lanceur **local-only** pour tes serveurs de dev en loopback.  
**Gratuit** · **100 % local-first** · **Mr-Aurevo-X** · mises à jour **non garanties**

[Télécharger LocalDock.zip](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/LocalDock.zip) · **[Releases](https://github.com/Mr-Aurevo-X/LocalDock/releases)** · **v0.1.0**

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
- Le registre (`apps.json`) est **propre à ce PC et à ce compte Windows** — les chemins ne suivent pas une install ailleurs

## Sur ton PC

| Quoi | Où |
|------|-----|
| **App** | Extrais `LocalDock.zip`, lance `Lancer.cmd` (ou `localdock.exe`) |
| Registre | `%APPDATA%\LocalDock\apps.json` |
| Historique | `%APPDATA%\LocalDock\history.json` |
| Préférences | `%LOCALAPPDATA%\Mr-Aurevo-X\user-settings.json` (partagé entre apps) |

## Lancer

1. Télécharge le zip sur la page Releases  
2. Extrais où tu veux  
3. Lance `Lancer.cmd`

Windows peut afficher un avertissement : les binaires ne sont **pas signés**. C’est **SmartScreen**, pas un antivirus qui dit « virus ».

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

## Compiler depuis les sources

Rust stable + WebView2 (Windows) ou WebKitGTK (Linux). Linux : sources seulement sur v0.1.0, pas de zip officiel.

```powershell
cargo test -p localdock-core
cargo build --release -p localdock
.\Lancer.cmd
```

```powershell
.\scripts\package-localdock-zip.ps1
.\scripts\check-no-network-deps.ps1
```

Docs : `docs/SECURITY.md` · `RELEASES.md` · `ISOLATION.md` · `docs/QA-SMOKE.md`

## Soutien (optionnel)

Si le boulot te plaît, un café — sinon profite.

[![Discord](https://img.shields.io/badge/Discord-Mr--Aurevo--X-5865F2?style=for-the-badge&logo=discord&logoColor=white&labelColor=050807)](https://discord.com/users/406891052516114442)
[![PayPal](https://img.shields.io/badge/PayPal-Donate-39ff14?style=for-the-badge&logo=paypal&logoColor=00f0ff&labelColor=050807)](https://www.paypal.com/paypalme/aurevo1)
[![Revolut](https://img.shields.io/badge/Revolut-mr__aurevo__x-00f0ff?style=for-the-badge&logo=revolut&logoColor=39ff14&labelColor=050807)](https://revolut.me/mr_aurevo_x)

---

Rêvée par **Mr-Aurevo-X**. Cursor a réalisé le rêve.
