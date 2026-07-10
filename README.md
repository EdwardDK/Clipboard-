# Clipboard+

Clipboard+ is a private, local-first Windows clipboard manager built with Tauri 2, React, TypeScript, Tailwind CSS, Rust, SQLite, and SQLite FTS5.

## Milestone 1

- Captures Unicode text clipboard changes locally.
- Coalesces duplicate text by SHA-256 hash, updating the copy count and timestamp.
- Stores clipboard history in SQLite with FTS5-backed instant text search.
- Provides a dark, keyboard-first command palette with copy and delete actions.
- Runs from a Windows tray icon and hides instead of quitting when its window is closed.
- Registers `Ctrl + Shift + V` to reveal and focus the palette.

Clipboard content is not logged or sent to a service. This milestone intentionally does not implement sensitive-content detection, pins, groups, application metadata, image/file capture, or cloud backup; those belong to later milestones.

## Run locally

1. Install the [Rust stable toolchain](https://www.rust-lang.org/tools/install) and the Windows C++ Build Tools. Tauri on Windows also requires Microsoft Edge WebView2.
2. Install packages: `npm.cmd install`
3. Start development: `npm.cmd run tauri dev`

Validation commands:

```powershell
npm.cmd run lint
npm.cmd run build
npm.cmd test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd run tauri build
```

The local database is created at `%LOCALAPPDATA%\ClipboardPlus\clipboard-plus.sqlite3`.
