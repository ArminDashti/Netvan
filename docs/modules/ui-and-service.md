# netvan-service / Tauri host

`netvan-service` runs collectors and serves RPC on `\\.\pipe\netvan-service`. CLI: `run|install|uninstall|start|stop`.

The Tauri host (`src-tauri`) embeds the same `CollectorEngine` so the UI works without the service during development. UI pages call `invoke("rpc")`. Autostart uses `tauri-plugin-autostart`.

## Release export

Root script `install.ps1` builds the UI `.exe` (`npm run tauri build`) and copies it to `--out` (default `.\Netvan`). Flags: `--with-service`, `--skip-npm`, `--help`.
