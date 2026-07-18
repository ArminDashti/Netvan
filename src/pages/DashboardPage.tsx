import { useEffect, useMemo, useState } from "react";
import ReactECharts from "echarts-for-react";
import { rpc, type NicInfo } from "@/lib/api";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { formatBps } from "@/lib/utils";

export function DashboardPage() {
  const [nics, setNics] = useState<NicInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [history, setHistory] = useState<{ ts: number; rx: number; tx: number }[]>([]);

  useEffect(() => {
    let alive = true;
    const load = async () => {
      try {
        const res = await rpc<{ type: "Nics"; data: NicInfo[] }>({ method: "ListNics" });
        if (!alive) return;
        const list = res.data.filter((n) => n.media_type !== "Loopback");
        setNics(list);
        setError(null);
        const primary = list.find((n) => n.oper_status === "Up") ?? list[0];
        if (primary) {
          const hist = await rpc<{
            type: "BandwidthHistory";
            data: { ts: number; rx_bytes: number; tx_bytes: number }[];
          }>({
            method: "GetBandwidthHistory",
            params: {
              nic_id: primary.id,
              range: "today",
              start_ts: null,
              end_ts: null,
            },
          });
          if (!alive) return;
          setHistory(
            hist.data.slice(-60).map((p) => ({
              ts: p.ts,
              rx: p.rx_bytes,
              tx: p.tx_bytes,
            })),
          );
        }
      } catch (e) {
        if (alive) setError(e instanceof Error ? e.message : String(e));
      }
    };
    load();
    const id = setInterval(load, 2000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  const totalDown = nics.reduce((s, n) => s + n.rx_bps, 0);
  const totalUp = nics.reduce((s, n) => s + n.tx_bps, 0);

  const chartOption = useMemo(
    () => ({
      backgroundColor: "transparent",
      textStyle: { color: "#8b9bb4" },
      grid: { left: 48, right: 16, top: 24, bottom: 28 },
      tooltip: { trigger: "axis" },
      legend: { data: ["RX bytes", "TX bytes"], textStyle: { color: "#8b9bb4" } },
      xAxis: {
        type: "category",
        data: history.map((h) => new Date(h.ts * 1000).toLocaleTimeString()),
        axisLabel: { color: "#8b9bb4" },
      },
      yAxis: { type: "value", axisLabel: { color: "#8b9bb4" }, splitLine: { lineStyle: { color: "#243049" } } },
      series: [
        {
          name: "RX bytes",
          type: "line",
          smooth: true,
          showSymbol: false,
          areaStyle: { opacity: 0.15 },
          data: history.map((h) => h.rx),
          color: "#3d9cf0",
        },
        {
          name: "TX bytes",
          type: "line",
          smooth: true,
          showSymbol: false,
          areaStyle: { opacity: 0.1 },
          data: history.map((h) => h.tx),
          color: "#3ecf8e",
        },
      ],
    }),
    [history],
  );

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Dashboard</h1>
        <p className="text-sm text-[var(--color-muted-foreground)]">
          Live bandwidth and adapter status
        </p>
      </div>
      {error && (
        <div className="rounded-md border border-[var(--color-destructive)]/40 bg-[var(--color-destructive)]/10 px-3 py-2 text-sm">
          {error}
        </div>
      )}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader>
            <CardDescription>Download</CardDescription>
            <CardTitle className="text-xl text-[var(--color-primary)]">
              {formatBps(totalDown)}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader>
            <CardDescription>Upload</CardDescription>
            <CardTitle className="text-xl text-[var(--color-success)]">
              {formatBps(totalUp)}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader>
            <CardDescription>Adapters</CardDescription>
            <CardTitle className="text-xl">{nics.length}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader>
            <CardDescription>Online</CardDescription>
            <CardTitle className="text-xl">
              {nics.filter((n) => n.oper_status === "Up").length}
            </CardTitle>
          </CardHeader>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        {nics.map((n) => (
          <Card key={n.id}>
            <CardHeader className="flex-row items-start justify-between space-y-0">
              <div>
                <CardTitle>{n.name}</CardTitle>
                <CardDescription>{n.media_type} · {n.description}</CardDescription>
              </div>
              <Badge
                className={
                  n.oper_status === "Up"
                    ? "border-[var(--color-success)]/40 text-[var(--color-success)]"
                    : ""
                }
              >
                {n.oper_status}
              </Badge>
            </CardHeader>
            <CardContent className="space-y-1 text-sm">
              <div className="flex justify-between">
                <span className="text-[var(--color-muted-foreground)]">Down</span>
                <span>{formatBps(n.rx_bps)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[var(--color-muted-foreground)]">Up</span>
                <span>{formatBps(n.tx_bps)}</span>
              </div>
              <div className="truncate text-xs text-[var(--color-muted-foreground)]">
                {n.ipv4_addresses[0] ?? "No IPv4"}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Primary NIC usage (today)</CardTitle>
          <CardDescription>Recent bandwidth counter samples</CardDescription>
        </CardHeader>
        <CardContent>
          <ReactECharts option={chartOption} style={{ height: 280 }} />
        </CardContent>
      </Card>
    </div>
  );
}
