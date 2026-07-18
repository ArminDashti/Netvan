# RPC endpoints

All UI calls go through Tauri command `rpc` with a tagged `RpcRequest`. Responses are tagged `RpcResponse` (`type` + `data`).

| Method | Params | Response |
|--------|--------|----------|
| `Ping` | — | `Pong` |
| `GetStatus` | — | `Status` |
| `GetSettings` | — | `Settings` |
| `SetSettings` | `{ settings }` | `Ok` |
| `SetCaptureMode` | `{ mode: process\|full }` | `Ok` |
| `ListNics` | — | `Nics` |
| `GetNic` | `{ nic_id }` | `Nic` |
| `GetBandwidthHistory` | `{ nic_id?, range, start_ts?, end_ts? }` | `BandwidthHistory` |
| `GetPingHistory` | same | `PingHistory` |
| `GetHttpLatencyHistory` | same | `HttpLatencyHistory` |
| `GetLinkEvents` | same | `LinkEvents` |
| `GetAppUsage` | `{ range, start_ts?, end_ts?, group_by }` | `AppUsage` |
| `RunPing` | `{ target, nic_id? }` | `PingResult` |
| `RunHttpLatency` | `{ url, nic_id? }` | `HttpLatencyResult` |
| `RunTraceroute` | `{ target, nic_id?, max_hops? }` | `Traceroute` |
| `RunNslookup` | `{ query }` | `Nslookup` |
| `RunSpeedtest` | `{ nic_id?, server_id?, accept_eula }` | `Speedtest` |
| `GetSpeedtestHistory` | `{ range, start_ts?, end_ts? }` | `SpeedtestHistory` |
| `AcceptSpeedtestEula` | — | `Ok` |

Also: Tauri `get_data_dir` → string path.

Service IPC: named pipe `\\.\pipe\netvan-service`, length-prefixed JSON (`RpcEnvelope` / `RpcReply`).

History `range`: `today` | `yesterday` | `week` | `months` | `all` | `custom`.

## Build script (`install.ps1`)

| Flag | Purpose |
|------|---------|
| `--help` | Show usage |
| `--out=<path>` | Copy built binaries here (default `.\Netvan`) |
| `--with-service` | Also build `netvan-service.exe` |
| `--skip-npm` | Skip `npm install` |
