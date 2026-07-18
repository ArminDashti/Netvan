# Suggestions

1. Add true WinDivert packet parsing (SNI/Host) behind a feature flag once the driver is bundled with the installer.
2. Use TCP ESTATS or ETW for accurate per-connection byte counters instead of nominal connection weights.
3. Code-split ECharts in the frontend to reduce the ~1.3 MB JS bundle.
4. Add a first-run wizard for Ookla CLI download and WinDivert install.
