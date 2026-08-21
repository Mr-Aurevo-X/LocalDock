# LocalDock Final Whole-Branch Important Fix Report

## Status

Implemented the Important review findings I1 through I6 on `feat/localdock-v1`.

## Fixes

- I6: `apply_loopback` now sets both `HOST=127.0.0.1` and `HOSTNAME=127.0.0.1`.
- I2: framework detection normalizes command/argument basenames and strips Windows/script shim suffixes `.cmd`, `.bat`, `.exe`, `.js`, and `.mjs` case-insensitively before matching `vite`, `next`, or `uvicorn`.
- I4: bare `--host` / `-H` no longer blocks loopback injection; only inline or explicit separate host values count as existing host configuration.
- I3: `Registry::load` revalidates every app command and cwd against `allowed_roots`, rejecting the entire load with an app-named `LocalDockError::InvalidRegistryApp`.
- I1: Windows TCP listener parsing now bounds-checks rows before unsafe reads and retries `GetExtendedTcpTable` on `ERROR_INSUFFICIENT_BUFFER`.
- I5: Tauri `list_ports` keeps loopback listeners visible and additionally surfaces non-loopback listeners for running LocalDock child PIDs; the UI shows `LAN EXPOSED` badges on affected apps and port rows.

## Verification

- `cargo test -p localdock-core`: passed.
- `cargo check -p localdock`: passed.
- IDE diagnostics: one stale Tauri macro/icon diagnostic was reported, but `cargo check -p localdock` passed.
- Secret fallback scan with ripgrep patterns: no matches.
- Semgrep/gitleaks: local CLIs were unavailable; Semgrep MCP was unavailable; Docker was installed but Docker Desktop Linux engine was not running, so containerized scans could not execute.

