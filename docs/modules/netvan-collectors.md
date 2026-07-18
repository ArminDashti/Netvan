# netvan-collectors

Windows collectors orchestrated by `CollectorEngine`:

- NIC enumeration + bandwidth + link events
- ICMP ping (per-NIC bind via `ping -S`)
- HTTP latency phases (DNS/TCP/TLS/TTFB/total)
- Traceroute / nslookup
- Ookla Speedtest CLI wrapper
- Process traffic (Mode A) via TCP table + PID
- Full mode (Mode B) DNS enrichment + WinDivert probe

Background loops write samples into `netvan-core::Database`.
