import { useEffect, useMemo, useState } from "react";
import ReactECharts from "echarts-for-react";
import { HistoryFilter, customToTs } from "@/components/HistoryFilter";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { rpc, type HistoryRange, type NicInfo } from "@/lib/api";
import { formatMs } from "@/lib/utils";

export function LatencyPage() {
  const [range, setRange] = useState<HistoryRange>("today");
  const [customStart, setCustomStart] = useState("");
  const [customEnd, setCustomEnd] = useState("");
  const [nics, setNics] = useState<NicInfo[]>([]);
  const [nicId, setNicId] = useState<string>("");
  const [pingHist, setPingHist] = useState<
    { ts: number; rtt_ms: number | null; target: string; success: boolean }[]
  >([]);
  const [httpHist, setHttpHist] = useState<
    { ts: number; ttfb_ms: number | null; total_ms: number | null; url: string }[]
  >([]);
  const [target, setTarget] = useState("8.8.8.8");
  const [url, setUrl] = useState("https://www.cloudflare.com/cdn-cgi/trace");
  const [traceTarget, setTraceTarget] = useState("8.8.8.8");
  const [hops, setHops] = useState<
    { hop: number; address: string | null; rtt_ms: number | null }[]
  >([]);
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    rpc<{ type: "Nics"; data: NicInfo[] }>({ method: "ListNics" }).then((r) => {
      const list = r.data.filter((n) => n.media_type !== "Loopback");
      setNics(list);
    });
  }, []);

  useEffect(() => {
    const start_ts = range === "custom" ? customToTs(customStart) : null;
    const end_ts = range === "custom" ? customToTs(customEnd) : null;
    Promise.all([
      rpc<{
        type: "PingHistory";
        data: {
          ts: number;
          rtt_ms: number | null;
          target: string;
          success: boolean;
        }[];
      }>({
        method: "GetPingHistory",
        params: { nic_id: nicId || null, range, start_ts, end_ts },
      }),
      rpc<{
        type: "HttpLatencyHistory";
        data: {
          ts: number;
          ttfb_ms: number | null;
          total_ms: number | null;
          url: string;
        }[];
      }>({
        method: "GetHttpLatencyHistory",
        params: { nic_id: nicId || null, range, start_ts, end_ts },
      }),
    ]).then(([p, h]) => {
      setPingHist(p.data);
      setHttpHist(h.data);
    });
  }, [range, customStart, customEnd, nicId]);

  const pingChart = useMemo(
    () => ({
      backgroundColor: "transparent",
      tooltip: { trigger: "axis" },
      legend: { data: ["ICMP RTT"], textStyle: { color: "#8b9bb4" } },
      grid: { left: 48, right: 12, top: 28, bottom: 28 },
      xAxis: {
        type: "category",
        data: pingHist.map((p) => new Date(p.ts * 1000).toLocaleTimeString()),
        axisLabel: { color: "#8b9bb4", hideOverlap: true },
      },
      yAxis: {
        type: "value",
        name: "ms",
        axisLabel: { color: "#8b9bb4" },
        splitLine: { lineStyle: { color: "#243049" } },
      },
      series: [
        {
          name: "ICMP RTT",
          type: "line",
          showSymbol: false,
          data: pingHist.map((p) => p.rtt_ms),
          color: "#3d9cf0",
        },
      ],
    }),
    [pingHist],
  );

  const httpChart = useMemo(
    () => ({
      backgroundColor: "transparent",
      tooltip: { trigger: "axis" },
      legend: { data: ["TTFB", "Total"], textStyle: { color: "#8b9bb4" } },
      grid: { left: 48, right: 12, top: 28, bottom: 28 },
      xAxis: {
        type: "category",
        data: httpHist.map((p) => new Date(p.ts * 1000).toLocaleTimeString()),
        axisLabel: { color: "#8b9bb4", hideOverlap: true },
      },
      yAxis: {
        type: "value",
        name: "ms",
        axisLabel: { color: "#8b9bb4" },
        splitLine: { lineStyle: { color: "#243049" } },
      },
      series: [
        {
          name: "TTFB",
          type: "line",
          showSymbol: false,
          data: httpHist.map((p) => p.ttfb_ms),
          color: "#e6b84d",
        },
        {
          name: "Total",
          type: "line",
          showSymbol: false,
          data: httpHist.map((p) => p.total_ms),
          color: "#3ecf8e",
        },
      ],
    }),
    [httpHist],
  );

  const runPing = async () => {
    setBusy(true);
    setMsg(null);
    try {
      const r = await rpc<{
        type: "PingResult";
        data: { rtt_ms: number | null; success: boolean; error: string | null };
      }>({
        method: "RunPing",
        params: { target, nic_id: nicId || null },
      });
      setMsg(
        r.data.success
          ? `Ping OK: ${formatMs(r.data.rtt_ms)}`
          : `Ping failed: ${r.data.error ?? "unknown"}`,
      );
    } catch (e) {
      setMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const runHttp = async () => {
    setBusy(true);
    setMsg(null);
    try {
      const r = await rpc<{
        type: "HttpLatencyResult";
        data: {
          dns_ms: number | null;
          connect_ms: number | null;
          tls_ms: number | null;
          ttfb_ms: number | null;
          total_ms: number | null;
          success: boolean;
          error: string | null;
        };
      }>({
        method: "RunHttpLatency",
        params: { url, nic_id: nicId || null },
      });
      if (!r.data.success) {
        setMsg(r.data.error ?? "HTTP probe failed");
      } else {
        setMsg(
          `DNS ${formatMs(r.data.dns_ms)} · TCP ${formatMs(r.data.connect_ms)} · TLS ${formatMs(r.data.tls_ms)} · TTFB ${formatMs(r.data.ttfb_ms)} · Total ${formatMs(r.data.total_ms)}`,
        );
      }
    } catch (e) {
      setMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const runTrace = async () => {
    setBusy(true);
    setMsg(null);
    try {
      const r = await rpc<{
        type: "Traceroute";
        data: {
          hops: { hop: number; address: string | null; rtt_ms: number | null }[];
        };
      }>({
        method: "RunTraceroute",
        params: { target: traceTarget, nic_id: nicId || null, max_hops: 20 },
      });
      setHops(r.data.hops);
      setMsg(`Traceroute completed (${r.data.hops.length} hops)`);
    } catch (e) {
      setMsg(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-semibold">Latency</h1>
        <p className="text-sm text-[var(--color-muted-foreground)]">
          ICMP ping and browser-like HTTP timing
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <select
          className="h-9 rounded-md border border-[var(--color-border)] bg-[var(--color-muted)] px-2 text-sm"
          value={nicId}
          onChange={(e) => setNicId(e.target.value)}
        >
          <option value="">All NICs</option>
          {nics.map((n) => (
            <option key={n.id} value={n.id}>
              {n.name}
            </option>
          ))}
        </select>
        <HistoryFilter
          value={range}
          onChange={setRange}
          customStart={customStart}
          customEnd={customEnd}
          onCustomStart={setCustomStart}
          onCustomEnd={setCustomEnd}
        />
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>ICMP ping history</CardTitle>
          </CardHeader>
          <CardContent>
            <ReactECharts option={pingChart} style={{ height: 260 }} />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>HTTP latency history</CardTitle>
            <CardDescription>TTFB and total request time</CardDescription>
          </CardHeader>
          <CardContent>
            <ReactECharts option={httpChart} style={{ height: 260 }} />
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Run ping</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input value={target} onChange={(e) => setTarget(e.target.value)} />
            <Button disabled={busy} onClick={runPing}>
              Ping
            </Button>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>HTTP probe</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input value={url} onChange={(e) => setUrl(e.target.value)} />
            <Button disabled={busy} onClick={runHttp}>
              Probe
            </Button>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Traceroute</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input value={traceTarget} onChange={(e) => setTraceTarget(e.target.value)} />
            <Button disabled={busy} onClick={runTrace}>
              Trace
            </Button>
          </CardContent>
        </Card>
      </div>

      {msg && (
        <div className="rounded-md border border-[var(--color-border)] bg-[var(--color-muted)] px-3 py-2 text-sm">
          {msg}
        </div>
      )}

      {hops.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Traceroute hops</CardTitle>
          </CardHeader>
          <CardContent>
            <table className="w-full text-sm">
              <thead>
                <tr className="text-left text-[var(--color-muted-foreground)]">
                  <th className="py-1">Hop</th>
                  <th>Address</th>
                  <th>RTT</th>
                </tr>
              </thead>
              <tbody>
                {hops.map((h) => (
                  <tr key={h.hop} className="border-t border-[var(--color-border)]">
                    <td className="py-1.5">{h.hop}</td>
                    <td className="font-mono text-xs">{h.address ?? "*"}</td>
                    <td>{formatMs(h.rtt_ms)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
