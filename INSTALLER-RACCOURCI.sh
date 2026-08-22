#!/usr/bin/env bash
# Raccourci Bureau → LANCER.sh (binaire natif).
# Flatpak (menu + Bureau) : bash packaging/installer-raccourci-flatpak.sh
set -e
SHARE="$(cd "$(dirname "$0")" && pwd)"
if [[ ! -f "$SHARE/LANCER.sh" || ! -f "$SHARE/Cargo.toml" ]]; then
  echo "ERREUR : ce script doit être lancé depuis le dossier LocalDock."
  echo "  Trouvé : $SHARE"
  echo "  Depuis le dossier :"
  echo "    bash INSTALLER-RACCOURCI.sh"
  exit 1
fi
DESKTOP="${XDG_DESKTOP_DIR:-$HOME/Desktop}"
[[ -d "$DESKTOP" ]] || DESKTOP="$HOME/Bureau"
[[ -d "$DESKTOP" ]] || DESKTOP="$HOME"

OUT="$DESKTOP/LocalDock.desktop"
ICON="$SHARE/packaging/flatpak/org.mraurevox.LocalDock.svg"
[[ -f "$ICON" ]] || ICON=applications-development

cat > "$OUT" << EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=LocalDock
Comment=Lanceur local-only pour tes serveurs de dev en loopback
Exec=bash "$SHARE/LANCER.sh"
Path=$SHARE
Icon=$ICON
Terminal=false
StartupWMClass=LocalDock
Categories=Development;
StartupNotify=true
EOF

chmod +x "$OUT"
gio set "$OUT" metadata::trusted true 2>/dev/null || true

echo "Raccourci créé : $OUT"
echo "Double-clique CE fichier sur le Bureau."
