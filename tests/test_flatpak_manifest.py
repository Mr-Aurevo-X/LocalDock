#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
YML = ROOT / "packaging" / "flatpak" / "org.mraurevox.LocalDock.yml"
DESKTOP = ROOT / "packaging" / "flatpak" / "org.mraurevox.LocalDock.desktop"
META = ROOT / "packaging" / "flatpak" / "org.mraurevox.LocalDock.metainfo.xml"


def test_flatpak_manifest_uses_host_bridge() -> None:
    text = YML.read_text(encoding="utf-8")
    assert "org.mraurevox.LocalDock" in text
    assert "org.gnome.Platform" in text
    assert "org.freedesktop.Sdk.Extension.rust-stable" in text
    assert "--talk-name=org.freedesktop.Flatpak" in text
    assert "--socket=session-bus" in text
    assert "--share=network" in text
    assert "--filesystem=/run/media" in text
    assert "--filesystem=/mnt" in text
    assert "--socket=x11" in text
    assert "--socket=fallback-x11" not in text
    assert "WEBKIT_DISABLE_DMABUF_RENDERER=1" in text
    assert "GDK_BACKEND=x11" in text
    # Mint/Cinnamon injects xapp-gtk3-module; GNOME runtime has no host .so.
    assert "--unset-env=GTK_MODULES" in text
    assert "--unset-env=GTK3_MODULES" in text
    assert "cargo build --release -p localdock" in text


def test_lancer_strips_mint_gtk_modules() -> None:
    lancer = (ROOT / "LANCER.sh").read_text(encoding="utf-8")
    assert "unset GTK_MODULES GTK3_MODULES" in lancer


def test_desktop_and_metainfo_match_app_id() -> None:
    desktop = DESKTOP.read_text(encoding="utf-8")
    meta = META.read_text(encoding="utf-8")
    assert "Name=LocalDock" in desktop
    assert "Exec=localdock" in desktop
    assert "<id>org.mraurevox.LocalDock</id>" in meta
    assert "flatpak-spawn --host" in meta
