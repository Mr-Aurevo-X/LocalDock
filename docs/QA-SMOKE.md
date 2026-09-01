# LocalDock — Smoke checklist

Manual smoke pass before release or after changes to spawn, registry, ports, or Tauri IPC.

**v0.1.0 ship:** same tag, two official assets on `Mr-Aurevo-X/LocalDock`:

- `LocalDock.zip` (Windows)
- `org.mraurevox.LocalDock.flatpak` (Linux, GNOME 49)

Both download URLs must return **200** (follow redirects).

Prerequisites: Rust stable + WebView2 (Windows). Linux consumer: Flatpak + `org.gnome.Platform//49`. See [`README.md`](../README.md).

## Windows (zip)

- [ ] **Build** — `.\scripts\package-localdock-zip.ps1` (or `cargo build --release -p localdock`) completes.
- [ ] **Accueil** — Parcourir + Ajouter a trusted root; scan; register one app; KPIs show port / app / root counts.
- [ ] **Start** — Start a registered app; open `http://127.0.0.1:<port>` in the browser; page loads.
- [ ] **Ports ouverts** — Tile shows process name (not only IP), path when known, and “ouvert depuis …”; Stop / kill frees the port.
- [ ] **Historique** — Scan / start / stop appear on this PC.
- [ ] **Path guard** — Register a path outside allowed roots → error; entry is not saved.
- [ ] **Controller not listening** — With LocalDock idle, LocalDock itself is not bound to `0.0.0.0`:

  ```powershell
  netstat -ano | findstr LISTENING | findstr /V "127.0.0.1"
  ```

## Linux (Flatpak)

- [ ] **Runtime** — `flatpak install --user -y flathub org.gnome.Platform//49`
- [ ] **Install** — `flatpak install --user -y ./org.mraurevox.LocalDock.flatpak` from the GitHub asset (no clone).
- [ ] **Shortcut** — copy the exported `.desktop` into `~/.local/share/applications/` and onto `$XDG_DESKTOP_DIR` / `~/Bureau` / `~/Desktop` (or `bash packaging/installer-raccourci-flatpak.sh` from a clone).
- [ ] **Run** — `flatpak run org.mraurevox.LocalDock` opens Accueil.
- [ ] **Host ports** — Ports list shows host listeners (`ss` via `flatpak-spawn --host`), not an empty sandbox.
- [ ] **Registry** — data under `~/.var/app/org.mraurevox.LocalDock/config/LocalDock/`.

Optional native: `bash LANCER.sh` (WebKitGTK). Not required if the Flatpak asset is published.

## Gates

- [ ] `cargo test -p localdock-core` passes.
- [ ] `.\scripts\check-no-network-deps.ps1` passes.

## Pass criteria

Windows zip boxes **or** Linux Flatpak boxes (the OS you ship this pass for) + gates. Both official assets must exist on the tag.
