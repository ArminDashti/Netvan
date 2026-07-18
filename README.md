# Netvan

Windows network monitor built with **Tauri 2**, **Rust**, and **React** (shadcn-style UI + ECharts).

## Features

- Live bandwidth per NIC with full adapter details
- ICMP ping and browser-like HTTP latency (DNS / TCP / TLS / TTFB / total)
- Traceroute, nslookup, and related tools
- Usage history: Today / Yesterday / Week / Months / All / Custom
- Ping & HTTP latency history with the same ranges
- Connect / disconnect history per NIC
- Per-app / IP / host usage (Process mode or Full mode with DNS/SNI enrichment)
- Built-in Ookla Speedtest.net via official CLI
- SQLite storage, optional Windows service + UI autostart

## Prerequisites

- Windows 10/11
- [Rust](https://rustup.rs/) (MSVC toolchain)
- Node.js 20+
- WebView2 (usually preinstalled)
- Optional: [Ookla Speedtest CLI](https://www.speedtest.net/apps/cli) for speed tests
- Optional: [WinDivert](https://reqrypt.org/windivert.html) for Full capture mode

## Develop

```bash
npm install
npm run tauri dev
```

Collectors run inside the Tauri process during development. SQLite DB path:

`%APPDATA%\Netvan\Netvan\netvan.db` (via `directories` crate project dirs)

## Release build

```powershell
.\install.ps1
.\install.ps1 --with-service
```

Or manually:

```bash
npm run tauri build
cargo build -p netvan-service --release
```

`install.ps1` runs the Tauri release build and copies `Netvan.exe` to `.\Netvan` (override with `--out=`). Use `--help` for options.

## Windows service

Elevated PowerShell:

```powershell
.\target\release\netvan-service.exe install
.\target\release\netvan-service.exe start
```

Commands: `install` | `uninstall` | `start` | `stop` | `run` (foreground)

See [installer/README.md](installer/README.md).

## Capture modes

| Mode | What you get | Requirements |
|------|----------------|--------------|
| **Process** | App + remote IP usage | Normal privileges |
| **Full** | + DNS cache / hostname enrichment; WinDivert when installed | Admin + WinDivert for packet divert |

HTTPS full URL paths are **not** captured (no TLS MITM). “URL” means hostname from DNS / SNI / HTTP Host.

## Speedtest

Place `speedtest.exe` on PATH or in the Netvan data directory, accept Ookla EULA in the Speed Test page, then run.
