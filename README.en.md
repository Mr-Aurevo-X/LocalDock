[Français](README.md) · [English](README.en.md)

# LocalDock

**Local-only** launcher for loopback dev servers.  
**Free** · **100% local-first** · **Mr-Aurevo-X** · updates **not guaranteed**

**v0.1.0** — two official packs, same app, same release:

| | Windows | Linux |
|---|---|---|
| **File** | [`LocalDock.zip`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/LocalDock.zip) | [`org.mraurevox.LocalDock.flatpak`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/org.mraurevox.LocalDock.flatpak) |
| **Payload** | `localdock.exe` + `Lancer.cmd` | GNOME 49 runtime, id `org.mraurevox.LocalDock` |
| **Install** | Extract → `Lancer.cmd` | `flatpak install --user` (below) |

[All releases](https://github.com/Mr-Aurevo-X/LocalDock/releases)

## Overview

- **Home** — open-port count, manually registered apps, trusted roots + **Browse** / scan
- **Open ports** — process name, path, command line, how long it has been open
- **History** — local log on **this PC**
- In-app FR | EN · Void Glow chrome (`min` / `max` / `close`)

## Why LocalDock

- Free (non-commercial use, `LICENSE`) — no account, no subscription
- Local-first — **no publisher telemetry**, **no HTTP portal**
- LocalDock itself listens on **no** port; child apps are steered to `127.0.0.1`
- Only off-machine call: GitHub version check **if you leave it on** in About (read-only, no download)
- The registry (`apps.json`) is **this PC / this account** — absolute paths do not follow you to another machine

## On your PC

### Windows

| What | Where |
|------|-------|
| **App** | Extract `LocalDock.zip`, run `Lancer.cmd` (or `localdock.exe`) |
| Registry | `%APPDATA%\LocalDock\apps.json` |
| History | `%APPDATA%\LocalDock\history.json` |
| Prefs | `%LOCALAPPDATA%\Mr-Aurevo-X\user-settings.json` (shared across apps) |

### Linux

| What | Where |
|------|-------|
| **App (Flatpak)** | `flatpak run org.mraurevox.LocalDock` |
| Flatpak data | `~/.var/app/org.mraurevox.LocalDock/config/LocalDock/` (`apps.json`, `history.json`) |
| Flatpak prefs | `~/.var/app/org.mraurevox.LocalDock/config/Mr-Aurevo-X/user-settings.json` |
| Native data (from source) | `~/.config/LocalDock/` |
| Native prefs | `~/.config/Mr-Aurevo-X/user-settings.json` |

The Flatpak lists and launches **host** localhost apps (`ss` + `flatpak-spawn --host`). A mounted Windows volume (`/run/media/…`) can be scanned; **Import Windows** remaps `C:\…`.

## Launch

### Windows

1. Download [`LocalDock.zip`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/LocalDock.zip)  
2. Extract anywhere  
3. Run `Lancer.cmd` (or `localdock.exe`)

Windows may show a warning: binaries are **not signed**. That is **SmartScreen**, not an antivirus “virus” verdict.

### Linux

1. [Flatpak](https://flatpak.org/setup/) + **GNOME 49** runtime:

```bash
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y flathub org.gnome.Platform//49
```

2. Official pack:

```bash
curl -fL -o org.mraurevox.LocalDock.flatpak \
  https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/org.mraurevox.LocalDock.flatpak
flatpak install --user -y ./org.mraurevox.LocalDock.flatpak
```

3. App menu + desktop shortcut (**no** git clone):

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

From a clone: `bash packaging/installer-raccourci-flatpak.sh` (same Bureau / Desktop fallback).

4. `flatpak run org.mraurevox.LocalDock`

Mint / Cinnamon: `Failed to load module "xapp-gtk3-module"` is the host Favorites hook. The pack unsets it; it is not a crash. Already installed: `flatpak override --user --unset-env=GTK_MODULES --unset-env=GTK3_MODULES org.mraurevox.LocalDock`

From source (optional): `bash LANCER.sh` · native shortcut: `bash INSTALLER-RACCOURCI.sh` · rebuild Flatpak: `bash packaging/build-flatpak.sh`.

## Official version only

The only build I stand behind:

**https://github.com/Mr-Aurevo-X/LocalDock**

A fork or modified copy elsewhere is **not** my version — I am not responsible for it.  
Software **as is**, without warranty — see `LICENSE` and `PRIVACY.md`.

## Legal

- **100% free** (non-commercial use, PolyForm Noncommercial 1.0.0)
- **Local-first** — no publisher telemetry
- **Updates not guaranteed** — optional GitHub check, no auto-install
- **Copyright © 2026 Mr-Aurevo-X**

In-app About: terms, privacy, notices, licenses, local paths.

## Build from source

Stable Rust + WebView2 (Windows) or WebKitGTK (Linux). Linux: `bash LANCER.sh` or Flatpak (`org.mraurevox.LocalDock`).

```powershell
cargo test -p localdock-core
cargo build --release -p localdock
.\Lancer.cmd
```

```bash
bash LANCER.sh
bash packaging/build-flatpak.sh
```

```powershell
.\scripts\package-localdock-zip.ps1
.\scripts\check-no-network-deps.ps1
```

Docs: `docs/SECURITY.md` · `RELEASES.md` · `ISOLATION.md` · `docs/QA-SMOKE.md`

## Support (optional)

If you like the work, a coffee — otherwise just enjoy.

[![Discord](https://img.shields.io/badge/Discord-Mr--Aurevo--X-5865F2?style=for-the-badge&logo=discord&logoColor=white&labelColor=050807)](https://discord.com/users/406891052516114442)
[![PayPal](https://img.shields.io/badge/PayPal-Donate-39ff14?style=for-the-badge&logo=paypal&logoColor=00f0ff&labelColor=050807)](https://www.paypal.com/paypalme/aurevo1)
[![Revolut](https://img.shields.io/badge/Revolut-mr__aurevo__x-00f0ff?style=for-the-badge&logo=revolut&logoColor=39ff14&labelColor=050807)](https://revolut.me/mr_aurevo_x)

---

Dreamed by **Mr-Aurevo-X**. Cursor made the dream real.
