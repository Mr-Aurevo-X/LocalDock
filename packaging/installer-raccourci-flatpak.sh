#!/usr/bin/env bash
# Menu applications + raccourci Bureau pour le Flatpak déjà installé.
set -euo pipefail

APP_ID=org.mraurevox.LocalDock
SRC="${XDG_DATA_HOME:-$HOME/.local/share}/flatpak/exports/share/applications/${APP_ID}.desktop"

if ! command -v flatpak >/dev/null 2>&1; then
  echo "ERREUR : flatpak n’est pas installé."
  exit 1
fi

if ! flatpak info --user "$APP_ID" >/dev/null 2>&1 \
  && ! flatpak info "$APP_ID" >/dev/null 2>&1; then
  echo "ERREUR : $APP_ID n’est pas installé."
  echo "  flatpak install --user -y org.mraurevox.LocalDock.flatpak"
  echo "  bash packaging/installer-raccourci-flatpak.sh"
  exit 1
fi

if [[ ! -f "$SRC" ]]; then
  echo "ERREUR : pas de .desktop exporté ($SRC)."
  echo "  Réinstalle le Flatpak, puis relance ce script."
  exit 1
fi

MENU="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$MENU"
MENU_OUT="$MENU/${APP_ID}.desktop"
cp -f "$SRC" "$MENU_OUT"
chmod +x "$MENU_OUT"

DESKTOP="${XDG_DESKTOP_DIR:-}"
if [[ -z "$DESKTOP" && -f "$HOME/.config/user-dirs.dirs" ]]; then
  # shellcheck disable=SC1091
  . "$HOME/.config/user-dirs.dirs"
  DESKTOP="${XDG_DESKTOP_DIR:-}"
fi
[[ -n "$DESKTOP" && -d "$DESKTOP" ]] || DESKTOP="$HOME/Bureau"
[[ -d "$DESKTOP" ]] || DESKTOP="$HOME/Desktop"
[[ -d "$DESKTOP" ]] || DESKTOP="$HOME"

DESK_OUT="$DESKTOP/LocalDock.desktop"
cp -f "$SRC" "$DESK_OUT"
chmod +x "$DESK_OUT"
gio set "$DESK_OUT" metadata::trusted true 2>/dev/null || true
gio set "$MENU_OUT" metadata::trusted true 2>/dev/null || true

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$MENU" 2>/dev/null || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
  kbuildsycoca6 --noincremental 2>/dev/null || true
fi

echo "Menu : $MENU_OUT"
echo "Bureau : $DESK_OUT"
echo "Cherche « LocalDock » dans le menu (Développement) ou double-clique le fichier Bureau."
