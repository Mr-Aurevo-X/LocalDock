# Packaging Linux

Pack officiel (sans cloner) : [`org.mraurevox.LocalDock.flatpak`](https://github.com/Mr-Aurevo-X/LocalDock/releases/latest/download/org.mraurevox.LocalDock.flatpak) — install + raccourci : voir `README.md` § Linux.

- `bash LANCER.sh` : build natif (Rust + WebKitGTK) et lancement. Binaire copié dans `~/.local/share/localdock/` (partages noexec).
- `bash INSTALLER-RACCOURCI.sh` : raccourci Bureau vers `LANCER.sh` (natif).
- `bash packaging/installer-raccourci-flatpak.sh` : menu + Bureau pour le Flatpak déjà installé (clone uniquement).
- `bash packaging/build-flatpak.sh` : bundle `.flatpak` (runtime GNOME 49 + Rust). Produit `dist/org.mraurevox.LocalDock.flatpak`.
- `packaging/publish-to-linux-flatpak-releases.sh` : publication optionnelle sur `linux-flatpak-releases`.

## Flatpak et localhost

L’UI tourne dans le sandbox. Le scan des ports (`ss`), le lancement des apps et le stop passent par l’hôte (`flatpak-spawn --host`), comme Gest Linux Pro.

Données Flatpak : `~/.var/app/org.mraurevox.LocalDock/config/LocalDock/`  
Données natives : `~/.config/LocalDock/`

Sous KDE Wayland / NVIDIA : X11 réel + `WEBKIT_DISABLE_DMABUF_RENDERER` (sinon GDK EPROTO 71).

Mint / Cinnamon : `GTK_MODULES` / `GTK3_MODULES` (`xapp-gtk3-module`) sont unset dans le manifest + `LANCER.sh` (le `.so` n’est pas dans le runtime).
