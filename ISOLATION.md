# Isolated runtime

LocalDock is a **Rust + Tauri 2** desktop app (WebView2 on Windows, WebKitGTK on Linux). It is **not** a Python hub and does not require UAC.

## 1. Pinned Rust (recommended)

Use the stable toolchain that built the release (`rustc --version` at tag time). From the repo root:

```bat
cargo test -p localdock-core
cargo build --release -p localdock
Lancer.cmd
```

`Lancer.cmd` prefers `target\release\localdock.exe`, then `target\debug\localdock.exe`.

## 2. Official zip

GitHub Release asset `LocalDock.zip` contains the Windows exe plus `Lancer.cmd` and licence files. No installer. Data stays under `%APPDATA%\LocalDock\` (this PC / this Windows user).

## 3. No OS-forever promise

The publisher does **not** guarantee future Windows or Linux builds. Pinning the toolchain is how a given tag stays reproducible.

## 4. Windows SmartScreen (« potentially unsafe »)

Windows may flag the app as **potentially unsafe** or show « Windows protected your PC »: official `.exe` builds are **not Authenticode-signed** and may lack Microsoft download reputation. That is a **SmartScreen reputation warning**, not an antivirus malware verdict. You can run from source via `Lancer.cmd` / `cargo` if you prefer. See also `CGU.md` / `TERMS.md`.
