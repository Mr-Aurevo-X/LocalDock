# LocalDock Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Windows + Linux Tauri/Rust app that starts/stops whitelisted localhost projects with zero inbound/outbound networking in the controller.

**Architecture:** Tauri 2 UI talks to a Rust core over IPC only. Core owns registry (JSON), project scan, process spawn (no shell), and loopback port scan. No HTTP server, no MCP, no updater, no telemetry.

**Tech Stack:** Rust 1.78+, Tauri 2, serde/serde_json, uuid, thiserror; UI = local static HTML/CSS/JS (no CDN). Tests via `cargo test`.

## Global Constraints

- Controller must never open a listen socket or make outbound network connections.
- Child processes spawned with `Command::new` + argv array only — never `shell(true)` / `cmd /C` / `sh -c`.
- App `cwd` must canonicalize inside an `allowed_roots` entry.
- Default `force_loopback: true`; inject `HOST=127.0.0.1` and known framework host flags.
- Platforms: Windows 10/11 and Linux x86_64.
- License: MIT. No accounts. Config in OS app data dir.
- Do not add analytics, auto-update, MCP, or HTTP portal features.
- Prefer zero unnecessary crates; reject crates whose purpose is HTTP clients/servers unless unavoidable and unused.

---

## File structure (target)

```
LocalDock/
├── README.md
├── LICENSE
├── Cargo.toml                 # workspace
├── crates/
│   └── localdock-core/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── error.rs
│           ├── path_guard.rs
│           ├── registry.rs
│           ├── scanner.rs
│           ├── spawn.rs
│           ├── loopback_env.rs
│           └── ports.rs
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── src/main.rs
├── ui/
│   ├── index.html
│   ├── styles.css
│   └── app.js
└── docs/superpowers/
    ├── specs/2026-08-21-localdock-design.md
    └── plans/2026-08-21-localdock.md
```

---

### Task 1: Workspace + core crate skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `crates/localdock-core/Cargo.toml`
- Create: `crates/localdock-core/src/lib.rs`
- Create: `crates/localdock-core/src/error.rs`
- Create: `LICENSE`
- Create: `README.md`

**Interfaces:**
- Produces: `localdock_core` library crate; `LocalDockError` enum; `pub use` surface for later modules.

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/localdock-core"]

[workspace.package]
edition = "2021"
license = "MIT"
version = "0.1.0"
```

- [ ] **Step 2: Create core crate with error type**

`crates/localdock-core/Cargo.toml`:

```toml
[package]
name = "localdock-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
```

`crates/localdock-core/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LocalDockError {
    #[error("path escapes allowed roots: {0}")]
    PathNotAllowed(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("app not found: {0}")]
    AppNotFound(String),
    #[error("invalid command")]
    InvalidCommand,
    #[error("already running")]
    AlreadyRunning,
    #[error("not running")]
    NotRunning,
}
```

`lib.rs` exports `error` and a placeholder `pub fn crate_name() -> &'static str { "localdock-core" }`.

- [ ] **Step 3: Write MIT LICENSE + short README stating network policy**

README must include:

```markdown
# LocalDock

Secure local-only launcher for loopback dev servers (Windows + Linux).

## Network policy

LocalDock itself opens **no** listen ports and makes **no** outbound connections.
Child processes you start may use the network for their own needs; by default they are steered to bind `127.0.0.1` only.
```

- [ ] **Step 4: Verify compile**

Run: `cargo test -p localdock-core`
Expected: PASS (empty/default tests ok)

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates LICENSE README.md docs
git commit -m "chore: bootstrap LocalDock workspace and core crate"
```

---

### Task 2: Path guard (canonicalize + allowed roots)

**Files:**
- Create: `crates/localdock-core/src/path_guard.rs`
- Create: `crates/localdock-core/tests/path_guard_tests.rs`
- Modify: `crates/localdock-core/src/lib.rs`

**Interfaces:**
- Produces: `pub fn assert_under_roots(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, LocalDockError>`

- [ ] **Step 1: Write failing tests**

```rust
use localdock_core::path_guard::assert_under_roots;
use std::path::{Path, PathBuf};

#[test]
fn accepts_path_inside_root() {
    let root = std::env::temp_dir().join("ld-root-ok");
    let child = root.join("proj");
    std::fs::create_dir_all(&child).unwrap();
    let got = assert_under_roots(&child, &[root.clone()]).unwrap();
    assert!(got.ends_with("proj"));
}

#[test]
fn rejects_path_outside_root() {
    let root = std::env::temp_dir().join("ld-root-a");
    let other = std::env::temp_dir().join("ld-root-b").join("proj");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    let err = assert_under_roots(&other, &[root]).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("escapes") || msg.contains("not allowed"));
}
```

- [ ] **Step 2: Run tests — expect FAIL (module missing)**

Run: `cargo test -p localdock-core --test path_guard_tests`
Expected: compile fail or FAIL

- [ ] **Step 3: Implement path_guard**

```rust
use crate::LocalDockError;
use std::path::{Path, PathBuf};

pub fn assert_under_roots(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, LocalDockError> {
    let canon = std::fs::canonicalize(path).map_err(LocalDockError::Io)?;
    for root in roots {
        let root_canon = std::fs::canonicalize(root).map_err(LocalDockError::Io)?;
        if canon.starts_with(&root_canon) {
            return Ok(canon);
        }
    }
    Err(LocalDockError::PathNotAllowed(canon.display().to_string()))
}
```

Note (Windows): `canonicalize` may yield `\\?\` prefix; compare consistently (strip verbatim prefix helper if needed so `starts_with` works).

- [ ] **Step 4: Run tests — expect PASS**

Run: `cargo test -p localdock-core --test path_guard_tests`

- [ ] **Step 5: Commit**

```bash
git add crates/localdock-core
git commit -m "feat: enforce cwd under allowed roots"
```

---

### Task 3: Registry load/save

**Files:**
- Create: `crates/localdock-core/src/registry.rs`
- Create: `crates/localdock-core/tests/registry_tests.rs`
- Modify: `lib.rs`

**Interfaces:**
- Produces:
  - `struct AppEntry { id, name, cwd, command, args, preferred_port, force_loopback, enabled }`
  - `struct Registry { version: u32, allowed_roots: Vec<PathBuf>, apps: Vec<AppEntry> }`
  - `Registry::load(path) / save(path) / add_app / remove_app / get`

- [ ] **Step 1: Write failing round-trip test**

```rust
#[test]
fn registry_round_trip() {
    let dir = tempfile::tempdir().unwrap(); // or std::env::temp_dir unique folder
    let path = dir.path().join("apps.json");
    let mut reg = localdock_core::registry::Registry::default_empty();
    reg.allowed_roots.push(dir.path().to_path_buf());
    reg.save(&path).unwrap();
    let loaded = localdock_core::registry::Registry::load(&path).unwrap();
    assert_eq!(loaded.version, 1);
    assert_eq!(loaded.allowed_roots.len(), 1);
}
```

If avoiding `tempfile` crate, use `std::env::temp_dir()` + uuid folder and clean up.

- [ ] **Step 2: Implement registry with serde**

- `version` must be `1` on save.
- `add_app` calls `assert_under_roots` before insert.
- Reject empty `command` or `command` containing spaces/`&`/`|`/`;` → `InvalidCommand`.

- [ ] **Step 3: Tests PASS**

Run: `cargo test -p localdock-core --test registry_tests`

- [ ] **Step 4: Commit**

```bash
git commit -am "feat: JSON registry with path-validated apps"
```

---

### Task 4: Loopback env + framework host flags

**Files:**
- Create: `crates/localdock-core/src/loopback_env.rs`
- Create: `crates/localdock-core/tests/loopback_env_tests.rs`

**Interfaces:**
- Produces: `pub fn apply_loopback(command: &str, args: &[String], preferred_port: Option<u16>) -> (Vec<(String,String)>, Vec<String>)`
  - returns `(env_pairs, maybe_extended_args)`

- [ ] **Step 1: Tests**

```rust
#[test]
fn injects_host_env() {
    let (env, args) = localdock_core::loopback_env::apply_loopback("npm", &["run".into(), "dev".into()], Some(5173));
    assert!(env.iter().any(|(k,v)| k == "HOST" && v == "127.0.0.1"));
    assert!(env.iter().any(|(k,v)| k == "PORT" && v == "5173"));
    assert_eq!(args, vec!["run", "dev"]);
}

#[test]
fn vite_gets_host_flag_when_command_is_vite() {
    let (_env, args) = localdock_core::loopback_env::apply_loopback("vite", &[], Some(5173));
    assert!(args.windows(2).any(|w| w == ["--host", "127.0.0.1"]));
}
```

- [ ] **Step 2: Implement table**

| Detection | Extra args |
|-----------|------------|
| command/args contain `vite` | `--host`, `127.0.0.1` |
| contain `next` | `-H`, `127.0.0.1` |
| contain `uvicorn` | `--host`, `127.0.0.1` |
| else | env only |

Do not duplicate flags if already present.

- [ ] **Step 3: Tests PASS + commit**

```bash
git commit -am "feat: force loopback env and known host flags"
```

---

### Task 5: Process spawner + running map

**Files:**
- Create: `crates/localdock-core/src/spawn.rs`
- Create: `crates/localdock-core/tests/spawn_tests.rs`

**Interfaces:**
- Produces:
  - `struct ProcessManager` with interior mutability (`Mutex<HashMap<String, Child>>`)
  - `start(app: &AppEntry) -> Result<(), LocalDockError>`
  - `stop(app_id: &str) -> Result<(), LocalDockError>`
  - `is_running(app_id: &str) -> bool`
  - `running_ids() -> Vec<String>`

- [ ] **Step 1: Write test that starts a harmless process**

Windows: `command = "cmd"`, args = `["/C", "ping", "-n", "3", "127.0.0.1"]` — **EXCEPTION:** this test may use cmd only inside test binary, not production `start`.

Better cross-platform test:

```rust
// Use a tiny helper binary or `rustc` — simplest portable:
// start `localdock-core` test helper via env::current_exe() with arg `--sleep-child`
```

Pragmatic approach for v1 tests:

```rust
#[test]
fn start_and_stop_echo_like() {
    // Linux: /bin/sleep 5
    // Windows: powershell -Command Start-Sleep -Seconds 5
    // BUT production spawn.rs must still never use shell for AppEntry.
}
```

Split: `spawn::start_command(program, args, cwd, env)` is the primitive; `start(app)` builds env via `apply_loopback` then calls primitive. Tests call `start_command` with `/bin/sleep` or `timeout`.

- [ ] **Step 2: Implement without shell**

```rust
let mut cmd = std::process::Command::new(&app.command);
cmd.args(&final_args)
   .current_dir(&app.cwd)
   .envs(env_pairs)
   .stdin(Stdio::null())
   .stdout(Stdio::piped())
   .stderr(Stdio::piped());
// On Windows: CREATE_NO_WINDOW optional later; do not set shell
```

Kill on stop: `child.kill()` then `wait()`; on Unix prefer killing process group if you `pre_exec` setsid — document follow-up; v1 kill direct child PID.

- [ ] **Step 3: Tests PASS + commit**

```bash
git commit -am "feat: spawn and stop registered apps without shell"
```

---

### Task 6: Loopback port scanner

**Files:**
- Create: `crates/localdock-core/src/ports.rs`
- Create: `crates/localdock-core/tests/ports_tests.rs`

**Interfaces:**
- Produces: `struct PortRow { port: u16, pid: u32, process_name: String, addr: String, is_loopback: bool }`
- `pub fn list_listeners(loopback_only: bool) -> Result<Vec<PortRow>, LocalDockError>`

- [ ] **Step 1: Implement platform modules**

`#[cfg(target_os = "windows")]` and `#[cfg(target_os = "linux")]`.

Linux v1: read `/proc/net/tcp` + `/proc/net/tcp6`, parse local address; map inode → PID via `/proc/*/fd`.

Windows v1: use `windows-sys` GetExtendedTcpTable **or** invoke nothing network-client related — prefer `windows` crate TCP table APIs only.

Default UI calls `list_listeners(true)`.

- [ ] **Step 2: Smoke test**

Bind a `TcpListener::bind("127.0.0.1:0")` in test, call `list_listeners(true)`, assert port appears.

- [ ] **Step 3: Commit**

```bash
git commit -am "feat: list loopback TCP listeners on Win and Linux"
```

---

### Task 7: Project scanner (discovery proposals)

**Files:**
- Create: `crates/localdock-core/src/scanner.rs`
- Create: `crates/localdock-core/tests/scanner_tests.rs`

**Interfaces:**
- Produces: `struct ProposedApp { name, cwd, command, args, preferred_port: Option<u16> }`
- `pub fn scan_root(root: &Path, max_depth: u32) -> Result<Vec<ProposedApp>, LocalDockError>`

- [ ] **Step 1: Detection rules (v1)**

| Marker | command/args | port guess |
|--------|--------------|------------|
| `package.json` with script `dev` | `npm`, `["run","dev"]` | parse `vite.config.*` or `.env` PORT if easy; else `None` |
| `manage.py` | `python`, `["manage.py","runserver","127.0.0.1:8000"]` | 8000 |
| `pyproject.toml` / `requirements.txt` + `main.py` | skip auto unless clear | — |

Skip `node_modules`, `.git`, `dist`, `target`, `venv`.

- [ ] **Step 2: Fixture dir in tests + commit**

```bash
git commit -am "feat: scan allowed roots for proposable apps"
```

---

### Task 8: Tauri shell (no network capabilities)

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `ui/index.html`, `ui/styles.css`, `ui/app.js`
- Modify: workspace `Cargo.toml` members += `src-tauri`

**Interfaces:**
- Produces Tauri commands: `list_apps`, `add_root`, `scan`, `register_app`, `start_app`, `stop_app`, `list_ports`, `kill_port`, `open_loopback`

- [ ] **Step 1: Scaffold Tauri 2 app with `uiDir` = `../ui`**

`tauri.conf.json` critical settings:

- No updater plugin
- CSP: `default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self'`
- Do **not** enable remote URLs

`capabilities/default.json`: only core IPC + `shell.open` **restricted** later; prefer custom `open_loopback` that allows only `http://127.0.0.1:*` / `http://localhost:*`.

- [ ] **Step 2: Wire commands to localdock-core**

Keep a `AppState { registry_path, registry: Mutex<Registry>, processes: Mutex<ProcessManager> }`.

Config path:

- Windows: `%APPDATA%/LocalDock/apps.json`
- Linux: `~/.config/LocalDock/apps.json`

- [ ] **Step 3: Minimal UI**

Three sections: Roots+Scan, Apps Start/Stop, Ports Refresh/Kill. Dark neutral CSS (not purple SaaS). No external fonts/CDNs.

- [ ] **Step 4: Manual idle network check**

Run LocalDock, leave idle 30s, confirm LocalDock PID has no remote TCP in Resource Monitor / `ss -tp` filtered by pid.

- [ ] **Step 5: Commit**

```bash
git commit -am "feat: Tauri UI over IPC-only core"
```

---

### Task 9: Security hardening pass

**Files:**
- Modify: `src-tauri/capabilities/default.json`
- Create: `docs/SECURITY.md`
- Create: `scripts/check-no-network-deps.sh` (and `.ps1`)
- Modify: `README.md` with build/run

- [ ] **Step 1: Document threat model** (copy from design spec, short)

- [ ] **Step 2: Dependency check script**

Fail CI/local if `Cargo.lock` contains obvious HTTP client packages used by LocalDock packages (`reqwest`, `hyper` as direct deps of localdock-core/src-tauri — allow transitive only if Tauri forces them for WebView; document exceptions).

- [ ] **Step 3: Grep gate**

```bash
rg -n "https?://" crates src-tauri/src ui --glob '!docs/**'
```

Allow only comments in SECURITY/README or `127.0.0.1` open URL builder.

- [ ] **Step 4: Semgrep / gitleaks on diff before release commit**

- [ ] **Step 5: Commit**

```bash
git commit -am "docs: security policy and dependency network gates"
```

---

### Task 10: Cross-platform smoke checklist

**Files:**
- Create: `docs/QA-SMOKE.md`

- [ ] **Step 1: Write checklist**

Windows:

1. Build `cargo tauri build` (or `npm`/`cargo` tauri equivalent used in repo).
2. Add `Dev Central Tree` as root, scan, register one Vite/npm app.
3. Start → browser `http://127.0.0.1:<port>` works.
4. Ports tab shows loopback listener; Stop frees port.
5. Attempt register path outside roots → error.
6. Confirm LocalDock itself not listening (`netstat`).

Linux: same on Ubuntu/WSL2 or native.

- [ ] **Step 2: Commit QA doc**

```bash
git commit -am "docs: Win/Linux smoke checklist"
```

---

## Self-review

| Spec requirement | Task |
|------------------|------|
| Zero inbound/outbound controller | 8, 9 |
| Whitelist roots + path guard | 2, 3 |
| No shell spawn | 5 |
| Loopback force | 4, 6 warning |
| Scan + one-click start | 7, 8 |
| Win + Linux | 6, 8, 10 |
| No MCP/portal/telemetry | Global + 8/9 |

No TBD placeholders remaining in tasks. Types align: `Registry` / `AppEntry` / `ProcessManager` / `PortRow` / `ProposedApp`.

---

## Out of scope follow-ups (do not implement in this plan)

- Offline-forced children (block outbound)
- Docker compose orchestration
- MCP bridge
- Auto-update
- macOS (can come later with same core)
