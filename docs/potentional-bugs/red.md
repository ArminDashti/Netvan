# Potential bugs — red

1. **Named pipe ACL**: pipe is created with default security; harden to interactive user + SYSTEM before shipping enterprise builds.
2. **Service + in-process collectors**: if both the Windows service and the Tauri app run collectors against the same SQLite file, expect write contention — prefer one writer (service) in production.
3. **Mode B without WinDivert**: Full mode currently enriches via DNS cache only; UI must not imply deep packet URL capture until WinDivert path is complete.
