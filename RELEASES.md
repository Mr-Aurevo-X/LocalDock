# Release channels

Official LocalDock binaries ship through GitHub Releases on **this repo**:

**https://github.com/Mr-Aurevo-X/LocalDock/releases**

Packs:

- **`LocalDock.zip`** — Windows portable (`localdock.exe` + `Lancer.cmd` + licence)
- **`org.mraurevox.LocalDock.flatpak`** — Linux (GNOME 49), built with `bash packaging/build-flatpak.sh`

There is no hub pack, no `MrAurevoX-Launcher` channel, and no `Launch-Hub-*.zip`.

## Stable

Production tags on the default branch (`v0.1.0`, …). GitHub “Latest” non-prerelease.

## Beta

Prerelease tags named `*-beta.*`. Expect breakage.

## Isolation

See `ISOLATION.md`. LocalDock is a Rust / Tauri desktop app; pin the Rust toolchain you used to build, and keep WebView2 available on Windows.
