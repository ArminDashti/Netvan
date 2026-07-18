# Netvan installer notes

## UI bundle

```powershell
.\install.ps1
# or: npm run tauri build
```

Produces `Netvan.exe` under `src-tauri/target/release/` (copied to `.\Netvan` by `install.ps1`) and an MSI/NSIS installer under `src-tauri/target/release/bundle/` (exact path depends on Tauri targets).

Recommended post-install steps (custom NSIS/WiX script or README for users):

1. Copy `netvan-service.exe` next to the app or into `Program Files\Netvan\`.
2. Run elevated: `netvan-service.exe install` then `netvan-service.exe start`.
3. Optionally enable **Start UI with Windows** from Settings (autostart plugin).

## Service recovery

`netvan-service install` configures Automatic start and restart-on-failure via `sc.exe`.

## Elevation

- Service install/start requires Administrator.
- Full capture mode (WinDivert) requires the service/process to load `WinDivert.dll` with sufficient privileges.
- Process capture mode works without WinDivert.

## Autostart (UI only)

Settings → “Start UI with Windows” uses `tauri-plugin-autostart` (HKCU Run / equivalent). The service is separate and should be set to Automatic.
