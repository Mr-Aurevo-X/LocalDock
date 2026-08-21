# LocalDock

Secure local-only launcher for loopback dev servers (Windows + Linux).

**License:** PolyForm Noncommercial 1.0.0 (`LICENSE`) — © 2026 Mr-Aurevo-X.  
**Legal / About:** in-app « À propos · Legal · Dons » (CGU, confidentialité, mentions, licences).  
**Support (voluntary):** Discord · PayPal · Revolut — a donation is **not** a license fee.

## Network policy

LocalDock itself opens **no** listen ports and makes **no** outbound connections for telemetry.
Child processes you start may use the network for their own needs; by default they are steered to bind `127.0.0.1` only.
Optional support buttons open allowlisted URLs in the system browser when you click them.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable, 2021 edition)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) v2 (`cargo install tauri-cli --version "^2"`)
- Platform WebView dependencies (WebView2 on Windows; WebKitGTK on Linux)

## Build and run

From the repo root:

```powershell
# Core library tests
cargo test -p localdock-core

# Compile the desktop shell
cargo check -p localdock

# Dev window (hot reload of Rust; static UI from ui/)
cargo tauri dev

# Release binary (bundle disabled in tauri.conf.json v0.1)
cargo tauri build
```

Registry file:

- Windows: `%APPDATA%\LocalDock\apps.json`
- Linux: `$XDG_CONFIG_HOME/LocalDock/apps.json` or `~/.config/LocalDock/apps.json`

## Security checks

Policy and threat model: [`docs/SECURITY.md`](docs/SECURITY.md).

Before release or after dependency changes, run the network gate:

```powershell
# Windows
.\scripts\check-no-network-deps.ps1
```

```bash
# Linux / macOS / WSL
./scripts/check-no-network-deps.sh
```

The script verifies:

1. No HTTP client crates in **direct** dependencies of `localdock-core` / `localdock`.
2. No disallowed `http://` / `https://` literals in `crates/`, `src-tauri/src/`, or `ui/` (loopback builders + allowlisted support/legal URLs only).
3. Optional warnings if `semgrep` / `gitleaks` are not installed; when installed they run and fail on findings.

Transitive `reqwest` / `hyper` from Tauri’s WebView stack are documented exceptions; see SECURITY.md.

## Docs

- Security: `docs/SECURITY.md`
- Design: `docs/superpowers/specs/2026-08-21-localdock-design.md`
- Plan: `docs/superpowers/plans/2026-08-21-localdock.md`

## Status

Workspace, `localdock-core`, and Tauri shell bootstrapped (v0.1.0).
