# LocalDock — Smoke checklist

Manual smoke pass before release or after changes to spawn, registry, ports, or Tauri IPC.

**v0.1.0 ship:** Windows zip on `Mr-Aurevo-X/LocalDock`. Linux is **source-build** (no official Linux asset on this tag). That is an explicit waiver of the old “both OS binaries” bar.

Prerequisites: Rust stable, WebView2 (Windows). See [`README.md`](../README.md).

## Windows (required for this tag)

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

## Linux (source-only on v0.1.0)

Optional: `cargo test -p localdock-core` and `cargo build -p localdock` on a machine with WebKitGTK. Not a blocker for the Windows GitHub Release.

## Gates

- [ ] `cargo test -p localdock-core` passes.
- [ ] `.\scripts\check-no-network-deps.ps1` passes.

## Pass criteria

All Windows boxes + gates. Linux binary is out of scope for v0.1.0.
