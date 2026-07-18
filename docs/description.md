# Netvan

Netvan is a Windows desktop network monitor. It tracks per-NIC bandwidth and link state, ICMP and HTTP latency, traceroute/nslookup tools, Ookla Speedtest.net runs, and per-app / IP / host traffic history. Data lives in local SQLite; a Tauri + React UI presents live charts and history filters (Today → Custom).

## Tech stack

- **UI**: Tauri 2, React, TypeScript, Tailwind CSS, shadcn-style components, Apache ECharts
- **Backend**: Rust crates (`netvan-core`, `netvan-collectors`, `netvan-service`)
- **DB**: SQLite (WAL) via `rusqlite`
- **OS**: Windows IP Helper, ICMP/`ping`, `tracert`, `nslookup`, optional WinDivert + Ookla CLI

## How to run

```bash
npm install
npm run tauri dev
```

## Release .exe

```powershell
.\install.ps1
.\install.ps1 --with-service
.\install.ps1 --help
```

Builds via `npm run tauri build`, copies `Netvan.exe` to `--out` (default `.\Netvan`). Optional `--with-service` also builds `netvan-service.exe`.

Collectors start inside the Tauri process. Optional Windows service:

```bash
cargo build -p netvan-service --release
.\target\release\netvan-service.exe install
.\target\release\netvan-service.exe start
```

DB path: project data dir from `directories` (`Netvan` / `com.Netvan`).
