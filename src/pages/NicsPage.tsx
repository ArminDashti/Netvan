import { useEffect, useMemo, useState } from "react";
import ReactECharts from "echarts-for-react";
import { HistoryFilter, customToTs } from "@/components/HistoryFilter";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { rpc, type HistoryRange, type NicInfo } from "@/lib/api";
import { formatBps, formatBytes, formatTs } from "@/lib/utils";

export function NicsPage() {
  const [nics, setNics] = useState<NicInfo[]>([]);
  const [selected, setSelected] = useState<string>("");
  const [range, setRange] = useState<HistoryRange>("today");
  const [customStart, setCustomStart] = useState("");
  const [customEnd, setCustomEnd] = useState("");
  const [usage, setUsage] = useState<{ ts: number; rx_bytes: number; tx_bytes: number }[]>([]);
  const [events, setEvents] = useState<
    { nic_id: string; ts: number; event: string; detail: string | null }[]
  >([]);

  useEffect(() => {
    rpc<{ type: "Nics"; data: NicInfo[] }>({ method: "ListNics" }).then((r) => {
      const list = r.data.filter((n) => n.media_type !== "Loopback");
      setNics(list);
      if (!selected && list[0]) setSelected(list[0].id);
    });
  }, []);

  useEffect(() => {
    let alive = true;
    const tick = async () => {
      const r = await rpc<{ type: "Nics"; data: NicInfo[] }>({ method: "ListNics" });
      if (!alive) return;
      setNics(r.data.filter((n) => n.media_type !== "Loopback"));
    };
    const id = setInterval(tick, 2000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  useEffect(() => {
    if (!selected) return;
    const start_ts = range === "custom" ? customToTs(customStart) : null;
    const end_ts = range === "custom" ? customToTs(customEnd) : null;
    Promise.all([
      rpc<{
        type: "BandwidthHistory";
        data: { ts: number; rx_bytes: number; tx_bytes: number }[];
      }>({
        method: "GetBandwidthHistory",
        params: { nic_id: selected, range, start_ts, end_ts },
      }),
      rpc<{
        type: "LinkEvents";
        data: { nic_id: string; ts: number; event: string; detail: string | null }[];
      }>({
        method: "GetLinkEvents",
        params: { nic_id: selected, range, start_ts, end_ts },
      }),
    ]).then(([u, e]) => {
      setUsage(u.data);
      setEvents(e.data);
    });
  }, [selected, range, customStart, customEnd]);

  const nic = nics.find((n) => n.id === selected);

  const chartOption = useMemo(
    () => ({
      backgroundColor: "transparent",
      tooltip: { trigger: "axis" },
      legend: { data: ["RX", "TX"], textStyle: { color: "#8b9bb4" } },
      grid: { left: 50, right: 16, top: 30, bottom: 30 },
      xAxis: {
        type: "category",
        data: usage.map((p) => new Date(p.ts * 1000).toLocaleString()),
        axisLabel: { color: "#8b9bb4", hideOverlap: true },
      },
      yAxis: {
        type: "value",
        axisLabel: { color: "#8b9bb4" },
        splitLine: { lineStyle: { color: "#243049" } },
      },
      series: [
        {
          name: "RX",
          type: "line",
          smooth: true,
          showSymbol: false,
          data: usage.map((p) => p.rx_bytes),
          color: "#3d9cf0",
        },
        {
          name: "TX",
          type: "line",
          smooth: true,
          showSymbol: false,
          data: usage.map((p) => p.tx_bytes),
          color: "#3ecf8e",
        },
      ],
    }),
    [usage],
  );

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-semibold">NICs</h1>
        <p className="text-sm text-[var(--color-muted-foreground)]">
          Adapter details, usage history, connect/disconnect timeline
        </p>
      </div>

      <div className="flex flex-wrap gap-2">
        {nics.map((n) => (
          <button
            key={n.id}
            onClick={() => setSelected(n.id)}
            className={`rounded-md border px-3 py-1.5 text-sm ${
              selected === n.id
                ? "border-[var(--color-primary)] bg-[var(--color-muted)]"
                : "border-[var(--color-border)]"
            }`}
          >
            {n.name}
          </button>
        ))}
      </div>

      {nic && (
        <div className="grid gap-4 lg:grid-cols-2">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>{nic.name}</CardTitle>
                <Badge>{nic.oper_status}</Badge>
              </div>
              <CardDescription>{nic.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <dl className="grid grid-cols-[140px_1fr] gap-x-3 gap-y-2 text-sm">
                <dt className="text-[var(--color-muted-foreground)]">GUID</dt>
                <dd className="truncate font-mono text-xs">{nic.guid}</dd>
                <dt className="text-[var(--color-muted-foreground)]">Index</dt>
                <dd>{nic.interface_index}</dd>
                <dt className="text-[var(--color-muted-foreground)]">MAC</dt>
                <dd className="font-mono text-xs">{nic.mac ?? "—"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">Type</dt>
                <dd>{nic.media_type}</dd>
                <dt className="text-[var(--color-muted-foreground)]">MTU</dt>
                <dd>{nic.mtu ?? "—"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">Link speed</dt>
                <dd>{nic.link_speed_bps ? formatBps(nic.link_speed_bps) : "—"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">IPv4</dt>
                <dd>{nic.ipv4_addresses.join(", ") || "—"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">IPv6</dt>
                <dd className="truncate text-xs">{nic.ipv6_addresses.join(", ") || "—"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">Gateway</dt>
                <dd>{nic.gateways.join(", ") || "—"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">DNS</dt>
                <dd>{nic.dns_servers.join(", ") || "—"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">DHCP</dt>
                <dd>{nic.dhcp_enabled ? "Yes" : "No"}</dd>
                <dt className="text-[var(--color-muted-foreground)]">Live down/up</dt>
                <dd>
                  {formatBps(nic.rx_bps)} / {formatBps(nic.tx_bps)}
                </dd>
                <dt className="text-[var(--color-muted-foreground)]">Counters</dt>
                <dd>
                  {formatBytes(nic.rx_bytes)} / {formatBytes(nic.tx_bytes)}
                </dd>
              </dl>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Connect / disconnect</CardTitle>
              <CardDescription>Link state history for this adapter</CardDescription>
            </CardHeader>
            <CardContent className="max-h-80 space-y-2 overflow-auto">
              {events.length === 0 && (
                <p className="text-sm text-[var(--color-muted-foreground)]">No events yet</p>
              )}
              {events.map((e, i) => (
                <div
                  key={`${e.ts}-${i}`}
                  className="flex items-start justify-between gap-2 rounded-md border border-[var(--color-border)] px-3 py-2 text-sm"
                >
                  <div>
                    <div className="font-medium capitalize">{e.event}</div>
                    <div className="text-xs text-[var(--color-muted-foreground)]">
                      {e.detail ?? ""}
                    </div>
                  </div>
                  <div className="shrink-0 text-xs text-[var(--color-muted-foreground)]">
                    {formatTs(e.ts)}
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>
      )}

      <Card>
        <CardHeader className="gap-3">
          <CardTitle>Usage history</CardTitle>
          <HistoryFilter
            value={range}
            onChange={setRange}
            customStart={customStart}
            customEnd={customEnd}
            onCustomStart={setCustomStart}
            onCustomEnd={setCustomEnd}
          />
        </CardHeader>
        <CardContent>
          <ReactECharts option={chartOption} style={{ height: 320 }} />
        </CardContent>
      </Card>
    </div>
  );
}
