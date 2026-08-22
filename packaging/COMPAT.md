# Compatibilité Linux

| Distro | Natif (`LANCER.sh`) | Flatpak |
|--------|---------------------|---------|
| CachyOS / Arch | Rust + webkit2gtk-4.1 | GNOME 49 |
| Fedora | cargo + webkit2gtk4.1-devel | GNOME 49 |
| Debian / Ubuntu 24.04+ | cargo + libwebkit2gtk-4.1-dev | GNOME 49 |
| Linux Mint 21.3+ (Cinnamon) | préférer Flatpak | GNOME 49 (`GTK*_MODULES` xapp coupé) |

Le Flatpak est le canal multi-distro. Le natif sert au test local et aux partages noexec.
