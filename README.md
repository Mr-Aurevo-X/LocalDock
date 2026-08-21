# LocalDock

**[Releases](https://github.com/Mr-Aurevo-X/LocalDock/releases)** · **[Latest](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest)**

**© 2026 Mr-Aurevo-X — LocalDock — 100% local-first — free — updates not guaranteed**

Lanceur local-only pour serveurs de dev en loopback (Windows ; Linux en sources).  
Local-only launcher for loopback dev servers (Windows; Linux from source).

**License:** PolyForm Noncommercial 1.0.0 (`LICENSE`) — © 2026 Mr-Aurevo-X.  
**Legal / About:** in-app **À propos** (CGU, confidentialité, mentions, licences, chemins locaux).  
**Support (voluntary):** Discord · PayPal · Revolut — a donation is **not** a license fee.

## Network policy

LocalDock itself opens **no** listen ports and has **no** HTTP portal.  
Child processes you start may use the network; by default they are steered to bind `127.0.0.1`.  
Optional GitHub Latest check (toggle in About) is the only off-machine call when enabled — read-only, **no** in-app download/install.  
Support buttons open allowlisted URLs in the system browser when you click them.

## Legal / Légal

| FR | EN |
|:--|:--|
| **100 % gratuit** (usage non commercial, `LICENSE`) | **100% free** (non-commercial use, `LICENSE`) |
| **Local-first** — pas de télémétrie éditeur | **Local-first** — no publisher telemetry |
| **Mise à jour non garantie** — vérif. GitHub optionnelle, pas d’install auto | **Updates not guaranteed** — optional GitHub check, no auto-install |
| **Copyright © 2026 Mr-Aurevo-X** | **Copyright © 2026 Mr-Aurevo-X** |

## What it does

- **Accueil** — count of open loopback ports, manually registered apps, trusted roots + **Parcourir** / scan
- **Ports ouverts** — listeners with process name, path, command line, and how long they have been open
- **Historique** — local log on this PC (`%APPDATA%\LocalDock\history.json`)
- Registry is **per Windows user / this PC** (`%APPDATA%\LocalDock\apps.json`) — absolute paths do not migrate to another machine

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, 2021 edition)
- Platform WebView (WebView2 on Windows; WebKitGTK on Linux)

Day-to-day: `Lancer.cmd` (uses `target\release\localdock.exe` if present).

## Build and run

From the repo root:

```powershell
cargo test -p localdock-core
cargo build --release -p localdock
.\Lancer.cmd
```

Windows zip for GitHub Releases:

```powershell
.\scripts\package-localdock-zip.ps1
```

Registry file:

- Windows: `%APPDATA%\LocalDock\apps.json`
- Linux: `$XDG_CONFIG_HOME/LocalDock/apps.json` or `~/.config/LocalDock/apps.json`

Shared preferences (language, GitHub check): `%LOCALAPPDATA%\Mr-Aurevo-X\user-settings.json`

## Security checks

Policy and threat model: [`docs/SECURITY.md`](docs/SECURITY.md).

```powershell
.\scripts\check-no-network-deps.ps1
```

```bash
./scripts/check-no-network-deps.sh
```

## Docs

- Security: `docs/SECURITY.md`
- Releases: `RELEASES.md`
- Isolation: `ISOLATION.md`
- Smoke: `docs/QA-SMOKE.md`
- Design: `docs/superpowers/specs/2026-08-21-localdock-design.md`

## Status

v0.1.0 — Windows portable zip + sources. UI Void Glow (chrome `min / max / close`), tabs Accueil / Ports / Historique, About / legal / optional GitHub Latest.
