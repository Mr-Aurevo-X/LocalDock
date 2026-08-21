# LocalDock — Design Spec (security-first)

**Date:** 2026-08-21  
**Status:** Ready for implementation plan  
**Platforms:** Windows 10/11, Linux (x86_64)

## Problem

Developers juggle many local projects (`npm run dev`, Vite, Next, Flask, etc.). Existing “localhost managers” often ship HTTP portals, MCP servers, Electron telemetry surfaces, or bind beyond loopback. We want a **dedicated** launcher that is **local-only** and **network-hostile by default**.

## Product name

**LocalDock** — dock to start/stop loopback-only dev apps.

## Goals

1. Register project folders under a whitelist and start/stop them in one click.
2. Discover projects by scanning configured roots (`package.json`, common Python/Rust markers).
3. Show listening **loopback** ports and which registered app (if any) owns them.
4. Force child processes toward `127.0.0.1` (never advertise LAN by default).
5. **Controller has zero inbound and zero outbound network** (hard requirement).
6. Cross-platform: Windows + Linux from one codebase.

## Non-goals (v1)

- Cloud sync, accounts, updates phone-home, crash analytics
- MCP / HTTP control API / “web portal”
- Managing Docker Desktop lifecycle (detect-only optional later)
- Killing arbitrary unrelated system processes without explicit user action on a known port
- Forcing child apps offline (would break package managers / APIs); children may open outbound sockets unless user enables optional “offline child” later

## Threat model

| Threat | Mitigation |
|--------|------------|
| Remote attacker triggers Start/Stop | No listen socket; no HTTP; UI is local IPC only (Tauri) |
| Malicious webpage drives the app | No loopback HTTP API; no CORS surface |
| Supply-chain / telemetry in launcher | Minimal deps; no analytics crates; Rust core owns spawn |
| Command injection via UI | Commands are structured argv from registry, never `shell=true` with free text |
| Path escape / launching outside tree | Absolute paths must be under an allowed root; canonicalize + reject `..` |
| App binds `0.0.0.0` and exposes LAN | Inject `HOST`/`PORT` + known framework flags; warn if scan sees non-loopback bind |
| Accidental outbound from LocalDock itself | No `reqwest`/`ureq` in app; Tauri CSP `default-src 'self'`; deny remote asset loads; no updater |
| Registry tampering | Config in user data dir with restrictive ACLs (Windows ACL / `0600` on Linux); integrity optional later |

### Trust boundary (honest)

Anything running **as the same OS user** can already run commands. LocalDock does not widen that boundary; it must not create a **network-reachable** control plane.

## Architecture

```
┌─────────────────────────────────────────────┐
│  UI (Tauri WebView)                         │
│  — static assets only, CSP locked           │
│  — invoke Rust commands only                │
└──────────────────┬──────────────────────────┘
                   │ IPC (no TCP)
┌──────────────────▼──────────────────────────┐
│  Rust core                                  │
│  registry · scanner · spawner · portscan    │
│  NO network crates · NO listen sockets      │
└──────────────────┬──────────────────────────┘
                   │ spawn (argv, env, cwd)
┌──────────────────▼──────────────────────────┐
│  Child process (user project)                │
│  Prefer bind 127.0.0.1:<port>               │
│  Outbound allowed (dev tooling)             │
└─────────────────────────────────────────────┘
```

## Tech stack

| Layer | Choice | Why |
|-------|--------|-----|
| Shell | **Tauri 2** | Smaller attack surface than Electron; no Node in production runtime for core |
| Core | **Rust** | Spawn/process/port logic without accidental HTTP servers |
| UI | Vanilla HTML/CSS/JS or lightweight React **bundled locally** | No CDN, no remote fonts |
| Config | `apps.json` in OS app data dir | Simple, auditable |
| Tests | `cargo test` for core; manual smoke on Win+Linux | Security logic unit-tested |

## Data model

```json
{
  "version": 1,
  "allowed_roots": ["D:/Partage_VM/Dev Tree", "C:/Users/.../Documents/Dev Central Tree"],
  "apps": [
    {
      "id": "uuid",
      "name": "factory-x",
      "cwd": "C:/.../Game-Lounge",
      "command": "npm",
      "args": ["run", "dev"],
      "preferred_port": 5179,
      "force_loopback": true,
      "enabled": true
    }
  ]
}
```

Rules:

- `cwd` must resolve inside some `allowed_roots` entry (after `canonicalize`).
- `command` is a single executable name or absolute path; no shell metacharacters path.
- `args` is a string array only.
- Start uses `Command::new(command).args(args).current_dir(cwd)` — **never** `cmd.exe /C` or `sh -c` for user commands in v1.

## Loopback enforcement

When `force_loopback` is true (default):

1. Set env: `HOST=127.0.0.1`, `HOSTNAME=127.0.0.1`, `PORT=<preferred_port>` if set.
2. Apply known framework arg injections when safe (documented table), e.g. Vite `--host 127.0.0.1`, Next `-H 127.0.0.1`.
3. After start, port scanner verifies listener address family is loopback; UI shows **LAN EXPOSED** warning otherwise.

## Port scanner

- Windows: enumerate listeners via OS APIs / `GetExtendedTcpTable` (or equivalent safe crate) filtered to loopback by default.
- Linux: parse `/proc/net/tcp` + `/proc/net/tcp6` or use a local-only crate.
- Display: port, PID, process name, matched LocalDock app id (if any).
- Kill: only for ports the user explicitly selects; confirm dialog; kill process tree of that PID.

## UI (v1)

Single window:

1. **Apps** — list, Start / Stop / Open (`http://127.0.0.1:<port>` via OS open URL — local only).
2. **Ports** — refresh loopback listeners, Kill (confirmed).
3. **Roots** — add/remove allowed scan roots; Scan → propose apps (user confirms before register).

No settings that enable remote access. No “enable portal” toggle.

## Security checklist (release gate)

- [ ] `cargo deny` / audit: no unexpected network deps in LocalDock binary
- [ ] Grep binary strings / source: no `http://` phone-home URLs in app code
- [ ] Tauri `dangerousRemoteDomainIpcAccess` unset; CSP default-src 'self'
- [ ] Spawn path never uses shell
- [ ] Path traversal tests pass
- [ ] Manual: Wireshark/tcpview while idle LocalDock — no unexpected connections from LocalDock PID
- [ ] Children started with force_loopback do not listen on `0.0.0.0` for known frameworks

## Success criteria

- User can register Dev Central Tree projects and start/stop them without a terminal.
- LocalDock process makes **no** inbound listen and **no** outbound connections at rest or during normal use of the UI.
- Works on Windows and Linux with the same config shape.
- MIT license, no account, source-buildable.
