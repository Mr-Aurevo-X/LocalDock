# LocalDock — Security Policy

LocalDock is a **local-only** launcher for loopback dev servers. The controller process must not expose a network control plane and must not make outbound connections during normal operation.

See also: [design spec](superpowers/specs/2026-08-21-localdock-design.md).

## Threat model

| Threat | Mitigation |
|--------|------------|
| Remote attacker triggers Start/Stop | No listen socket; no HTTP API; UI uses Tauri IPC only |
| Malicious webpage drives the app | No loopback HTTP API; CSP `default-src 'self'`; no remote assets |
| Supply-chain / telemetry in launcher | Minimal direct deps; no analytics crates; Rust core owns spawn |
| Command injection via UI | Structured argv from registry; never `shell=true` with free text |
| Path escape / launching outside tree | Paths canonicalized; must resolve under `allowed_roots`; reject `..` |
| App binds `0.0.0.0` and exposes LAN | Inject `HOST`/`PORT` + framework flags; warn on non-loopback bind |
| Accidental outbound from LocalDock | No HTTP client crates in **direct** deps; no updater; grep gate on URLs; `open_support` allowlist only |
| Registry tampering | Config under user data dir with restrictive permissions |

### Trust boundary

Anything running **as the same OS user** can already run commands. LocalDock does not widen that boundary; it must not create a **network-reachable** control plane.

Child processes may use the network for dev tooling (package managers, APIs). That is expected and outside LocalDock’s control.

## Architecture (security-relevant)

```
UI (Tauri WebView, static assets, CSP locked)
        │ IPC only (no TCP control plane)
Rust core (localdock-core: registry, spawn, port scan)
        │ spawn (argv, env, cwd — no shell)
Child dev server (prefer 127.0.0.1 bind)
```

## Tauri surface

- **Capabilities:** `src-tauri/capabilities/default.json` grants `core:default` only (window + core IPC). Verified minimal in Task 8 (`core:default` only; no extra permissions).
- **CSP:** `default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'` in `tauri.conf.json`.
- **No updater plugin**, no HTTP/MCP portal, no remote URL configuration.
- **`open_loopback`:** accepts only `http://127.0.0.1:<port>` or `http://localhost:<port>` before delegating to the OS opener.
- **`open_support`:** allowlisted Discord / PayPal / Revolut / GitHub only (voluntary; not telemetry).

## Dependency policy

### Direct dependencies (must pass gate)

LocalDock-owned packages:

- `crates/localdock-core`
- `src-tauri` (`localdock`)

These **must not** list HTTP/network client crates in `[dependencies]`, `[dev-dependencies]`, or `[build-dependencies]`.

Blocked crate names (direct): `reqwest`, `hyper`, `ureq`, `curl`, `surf`, `isahc`, `awc`, `attohttpc`, `minreq`, `oauth2`, `hubcaps`, `octocrab`.

Allowed direct deps today: serialization (`serde`, `serde_json`), errors (`thiserror`), IDs (`uuid`), platform APIs (`windows-sys` on Windows), and Tauri shell crates (`tauri`, `tauri-build`).

### Transitive exceptions (Tauri / WebView)

`Cargo.lock` may contain `reqwest`, `hyper`, `tokio` networking stacks **transitively** via `tauri` and its WebView/runtime graph. That is expected for Tauri 2 on desktop; LocalDock does not call these crates and does not enable Tauri’s updater or remote plugins.

Verify with:

```bash
cargo tree --depth 1 -p localdock-core
cargo tree --depth 1 -p localdock
```

Neither tree should show forbidden crates at depth 1.

## URL grep gate

Source scan (excluding `docs/`):

```bash
rg -n "https?://" crates src-tauri/src ui --glob '!docs/**'
```

Permitted matches:

- Loopback open URL builders: `http://127.0.0.1:…` and `http://localhost:…`
- Allowlisted support / legal contact URLs only (Discord, PayPal, Revolut, GitHub org) used by `open_support` and `ui/legal/*.md`
- No other remote `http://` or `https://` literals in application source

`open_support` opens the system browser for voluntary donations/contact. A donation is **not** a license fee.

`tauri.conf.json` schema URLs and `Cargo.lock` registry indices are out of scope for this gate.

## Release checklist

Before tagging a release:

1. Run `scripts/check-no-network-deps.ps1` (Windows) or `scripts/check-no-network-deps.sh` (Linux/macOS).
2. `cargo test -p localdock-core`
3. `cargo check -p localdock`
4. Semgrep / gitleaks: the check scripts run them when installed and **fail on findings**; missing tools emit a warning only (OK for local dev).
5. Manual smoke: idle LocalDock PID shows no unexpected listen sockets or outbound connections (Wireshark / TCPView / `ss -tlnp`).

## Reporting issues

Report security concerns privately to the maintainer. Do not open public issues for unfixed exploit details.
