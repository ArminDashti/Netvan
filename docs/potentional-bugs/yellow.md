# Potential bugs — yellow

1. ICMP/HTTP samples for every up NIC × every target can grow the DB quickly on multi-homed machines — tune intervals/retention.
2. `tracert`/`ping` locale-dependent output parsing may miss RTT on non-English Windows.
3. HTTP TLS phase timing is estimated, not a true handshake probe.
4. Process traffic byte accounting uses a placeholder weight per connection without ESTATS.
5. `app_usage_hourly` upsert may under-count if remote_ip formatting differs (`ip` vs `ip:port` stripping edge cases).
