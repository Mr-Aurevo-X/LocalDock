[Français](README.md) · [English](README.en.md)

# LocalDock

**Local-only** launcher for loopback dev servers.  
**Free** · **100% local-first** · **Mr-Aurevo-X** · updates **not guaranteed**

[Download LocalDock.zip](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/LocalDock.zip) — Windows, `localdock.exe` + `Lancer.cmd` · **v0.1.0**  
[All releases](https://github.com/Mr-Aurevo-X/LocalDock/releases)

**Linux:** use **[Hub Dev](https://github.com/Mr-Aurevo-X/Hub-Dev-Linux)** — not LocalDock.

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
- The registry (`apps.json`) is **this PC / this Windows account** — absolute paths do not follow you to another machine

## On your PC

| What | Where |
|------|-------|
| **App** | Extract `LocalDock.zip`, run `Lancer.cmd` (or `localdock.exe`) |
| Registry | `%APPDATA%\LocalDock\apps.json` |
| History | `%APPDATA%\LocalDock\history.json` |
| Prefs | `%LOCALAPPDATA%\Mr-Aurevo-X\user-settings.json` (shared across apps) |

## Launch

1. Download [`LocalDock.zip`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/LocalDock.zip)  
2. Extract anywhere  
3. Run `Lancer.cmd` (or `localdock.exe`)

Windows may show a warning: binaries are **not signed**. That is **SmartScreen**, not an antivirus “virus” verdict.

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

Stable Rust + WebView2 (Windows).

```powershell
cargo test -p localdock-core
cargo build --release -p localdock
.\Lancer.cmd
```

```powershell
.\scripts\package-localdock-zip.ps1
.\scripts\check-no-network-deps.ps1
```

Docs: `docs/SECURITY.md` · `RELEASES.md` · `ISOLATION.md` · `docs/QA-SMOKE.md`

## Support (optional)

If you like the work, a coffee — otherwise just enjoy.

[![Discord](https://img.shields.io/badge/Discord-Mr--Aurevo--X-5865F2?style=for-the-badge&logo=discord&logoColor=white&labelColor=050807)](https://discord.com/users/406891052516114442)

---

Dreamed by **Mr-Aurevo-X**. Cursor made the dream real.
