#!/usr/bin/env bash
# Lance LocalDock (natif). Compile si besoin, copie hors noexec.
set +e
SHARE="$(cd "$(dirname "$0")" && pwd)"
LOCAL="${XDG_DATA_HOME:-$HOME/.local/share}/localdock"
BIN_LOCAL="$LOCAL/localdock"
LOG="$LOCAL/launch.log"

mkdir -p "$LOCAL"
chmod 700 "$LOCAL" 2>/dev/null || true
touch "$LOG"
chmod 600 "$LOG" 2>/dev/null || true

exec > >(tee -a "$LOG") 2>&1

echo "========== $(date) =========="
echo "SHARE=$SHARE"
echo "LOCAL=$LOCAL"
echo "DISPLAY=${DISPLAY-}"
echo "XDG_SESSION_TYPE=${XDG_SESSION_TYPE-}"
echo "WAYLAND_DISPLAY=${WAYLAND_DISPLAY-}"
echo

pause() {
  echo
  echo "Appuie sur Entrée pour fermer…"
  if [[ -r /dev/tty ]]; then
    read -r _ </dev/tty
  else
    sleep 8
  fi
}

need_pkg() {
  echo "ERREUR : dépendance manquante — $1"
  echo
  echo "  Arch/CachyOS  : sudo pacman -Sy --needed rust cargo webkit2gtk-4.1 gtk3 libsoup openssl pkgconf"
  echo "  Debian/Ubuntu : sudo apt install -y cargo rustc libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev pkg-config"
  echo "  Fedora        : sudo dnf install -y cargo rust webkit2gtk4.1-devel gtk3-devel libsoup3-devel pkgconf"
  pause
  exit 1
}

command -v cargo >/dev/null || need_pkg "cargo"
command -v rustc >/dev/null || need_pkg "rustc"
pkg-config --exists webkit2gtk-4.1 || need_pkg "webkit2gtk-4.1"

cd "$SHARE" || { pause; exit 1; }

BIN_SRC="$SHARE/target/release/localdock"
NEED_BUILD=0
if [[ ! -x "$BIN_SRC" ]]; then
  NEED_BUILD=1
else
  if [[ -n "$(find crates src-tauri ui Cargo.lock Cargo.toml -newer "$BIN_SRC" 2>/dev/null | head -1)" ]]; then
    NEED_BUILD=1
  fi
fi

if [[ "$NEED_BUILD" -eq 1 ]]; then
  echo "Compilation LocalDock (release)…"
  if command -v notify-send >/dev/null 2>&1; then
    notify-send "LocalDock" "Compilation en cours…"
  fi
  cargo build --release -p localdock || {
    echo "ERREUR : cargo build a échoué. Détails dans $LOG"
    pause
    exit 1
  }
fi

install -Dm755 "$BIN_SRC" "$BIN_LOCAL"
# WebKitGTK + NVIDIA / KWin Wayland → EPROTO 71 / GBM invalide sans ça.
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
export WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"
# Mint/Cinnamon injecte xapp-gtk3-module (Favorites Nemo). Inutile ici, et
# le .so hôte n’existe pas dans le Flatpak / certains préfixes GTK.
unset GTK_MODULES GTK3_MODULES
if [[ -z "${GDK_BACKEND:-}" && "${XDG_SESSION_TYPE-}" == "wayland" ]]; then
  export GDK_BACKEND=x11
fi
echo "Binaire : $BIN_LOCAL"
echo "GDK_BACKEND=${GDK_BACKEND-} WEBKIT_DISABLE_DMABUF_RENDERER=${WEBKIT_DISABLE_DMABUF_RENDERER}"
echo "Démarrage UI…"
START_TS="$(date +%s)"
"$BIN_LOCAL"
CODE=$?
END_TS="$(date +%s)"
DUR="$((END_TS - START_TS))"
echo "exit=$CODE duration=${DUR}s"
if [[ $CODE -ne 0 ]]; then
  echo "Erreur — détails dans $LOG"
  if command -v notify-send >/dev/null 2>&1; then
    notify-send -u critical "LocalDock" "Échec du lancement. Voir ${LOG}"
  fi
  pause
  exit "$CODE"
fi
exit 0
