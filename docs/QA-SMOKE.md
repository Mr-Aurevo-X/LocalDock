# LocalDock — Cross-platform smoke checklist

Manual smoke pass before release or after changes to spawn, registry, ports, or Tauri IPC. Run on **Windows** and **Linux** (native Ubuntu or WSL2 with GUI/WebView).

Prerequisites: Rust stable, Tauri CLI v2, platform WebView deps (WebView2 / WebKitGTK). See [`README.md`](../README.md).

## Windows

- [ ] **Build** — From repo root: `cargo tauri build` completes without errors.
- [ ] **Add root & scan** — Add `Dev Central Tree` as an allowed root; run scan; register one Vite/npm app discovered under that tree.
- [ ] **Start & browse** — Start the registered app; open `http://127.0.0.1:<port>` in the browser; page loads.
- [ ] **Ports tab** — Ports tab shows the child as a loopback listener; **Stop** terminates the process and frees the port.
- [ ] **Path guard** — Attempt to register a path outside allowed roots → UI shows an error; entry is not saved.
- [ ] **Controller not listening** — With LocalDock running and no child started, confirm LocalDock itself is not listening:

  ```powershell
  netstat -ano | findstr LISTENING | findstr /V "127.0.0.1"
  ```

  Expect no LocalDock process bound to `0.0.0.0` or a non-loopback interface. (Child dev servers may listen on loopback only.)

## Linux

Repeat the same steps on Ubuntu (native or WSL2 with display):

- [ ] **Build** — `cargo tauri build` from repo root.
- [ ] **Add root & scan** — Add `Dev Central Tree` (or equivalent mount path); scan; register one Vite/npm app.
- [ ] **Start & browse** — Start → `http://127.0.0.1:<port>` works in browser.
- [ ] **Ports tab** — Loopback listener visible; **Stop** frees port.
- [ ] **Path guard** — Register path outside roots → error.
- [ ] **Controller not listening** — With LocalDock idle:

  ```bash
  ss -tlnp | grep -v '127.0.0.1\|::1' || true
  ```

  Expect no LocalDock listen socket on LAN interfaces.

## Optional gates (same on both platforms)

- [ ] `cargo test -p localdock-core` passes.
- [ ] `.\scripts\check-no-network-deps.ps1` (Windows) or `./scripts/check-no-network-deps.sh` (Linux) passes.

## Pass criteria

All boxed items checked on **both** Windows and Linux. Any failure blocks release until fixed or documented with an explicit waiver.
