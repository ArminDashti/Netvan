import { useEffect, useMemo, useState } from "react";
import ReactECharts from "echarts-for-react";
import { HistoryFilter, customToTs } from "@/components/HistoryFilter";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { rpc, type HistoryRange } from "@/lib/api";
import { formatTs } from "@/lib/utils";

type SpeedRow = {
  id: number;
  ts: number;
  server_name: string | null;
  download_mbps: number;
  upload_mbps: number;
  ping_ms: number;
  jitter_ms: number | null;
};

export function SpeedtestPage() {
  const [range, setRange] = useState<HistoryRange>("all");
  const [customStart, setCustomStart] = useState("");
  const [customEnd, setCustomEnd] = useState("");
  const [history, setHistory] = useState<SpeedRow[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [last, setLast] = useState<SpeedRow | null>(null);
  const [eula, setEula] = useState(false);

  const load = async () => {
    const start_ts = range === "custom" ? customToTs(customStart) : null;
    const end_ts = range === "custom" ? customToTs(customEnd) : null;
    const r = await rpc<{ type: "SpeedtestHistory"; data: SpeedRow[] }>({
      method: "GetSpeedtestHistory",
      params: { range, start_ts, end_ts },
    });
    setHistory(r.data);
    const s = await rpc<{
      type: "Settings";
      data: { speedtest_eula_accepted: boolean };
    }>({ method: "GetSettings" });
    setEula(s.data.speedtest_eula_accepted);
  };

  useEffect(() => {
    load().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [range, customStart, customEnd]);

  const run = async () => {
    setBusy(true);
    setError(null);
    try {
      if (!eula) {
        await rpc({ method: "AcceptSpeedtestEula" });
        setEula(true);
      }
      const r = await rpc<{ type: "Speedtest"; data: SpeedRow }>({
        method: "RunSpeedtest",
        params: { nic_id: null, server_id: null, accept_eula: true },
      });
      setLast(r.data);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const gauge = useMemo(() => {
    const dl = last?.download_mbps ?? history[0]?.download_mbps ?? 0;
    const ul = last?.upload_mbps ?? history[0]?.upload_mbps ?? 0;
    return {
      backgroundColor: "transparent",
      series: [
        {
          type: "gauge",
          min: 0,
          max: Math.max(100, Math.ceil(Math.max(dl, ul) * 1.2)),
          splitNumber: 5,
          axisLine: {
            lineStyle: {
              width: 14,
              color: [
                [0.3, "#3ecf8e"],
                [0.7, "#3d9cf0"],
                [1, "#e35d6a"],
              ],
            },
          },
          pointer: { width: 4 },
          detail: {
            formatter: `{value} Mbps↓\n${ul.toFixed(1)} Mbps↑`,
            color: "#e8eef8",
            fontSize: 14,
          },
          data: [{ value: Number(dl.toFixed(1)), name: "Download" }],
          title: { color: "#8b9bb4" },
        },
      ],
    };
  }, [last, history]);

  const histChart = useMemo(
    () => ({
      backgroundColor: "transparent",
      tooltip: { trigger: "axis" },
      legend: { data: ["Download", "Upload"], textStyle: { color: "#8b9bb4" } },
      grid: { left: 48, right: 12, top: 28, bottom: 28 },
      xAxis: {
        type: "category",
        data: [...history].reverse().map((h) => new Date(h.ts * 1000).toLocaleString()),
        axisLabel: { color: "#8b9bb4", hideOverlap: true },
      },
      yAxis: {
        type: "value",
        name: "Mbps",
        axisLabel: { color: "#8b9bb4" },
        splitLine: { lineStyle: { color: "#243049" } },
      },
      series: [
        {
          name: "Download",
          type: "bar",
          data: [...history].reverse().map((h) => h.download_mbps),
          color: "#3d9cf0",
        },
        {
          name: "Upload",
          type: "bar",
          data: [...history].reverse().map((h) => h.upload_mbps),
          color: "#3ecf8e",
        },
      ],
    }),
    [history],
  );

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold">Speed Test</h1>
          <p className="text-sm text-[var(--color-muted-foreground)]">
            Built-in Ookla Speedtest.net (official CLI)
          </p>
        </div>
        <Button disabled={busy} onClick={run}>
          {busy ? "Running…" : eula ? "Run speed test" : "Accept EULA & run"}
        </Button>
      </div>

      <p className="text-xs text-[var(--color-muted-foreground)]">
        Requires Ookla Speedtest CLI (`speedtest.exe`) on PATH or in the Netvan data folder.
        Results use Ookla servers; license/GDPR terms apply.
      </p>

      {error && (
        <div className="rounded-md border border-[var(--color-destructive)]/40 bg-[var(--color-destructive)]/10 px-3 py-2 text-sm">
          {error}
        </div>
      )}

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Latest result</CardTitle>
            <CardDescription>
              {(last ?? history[0])?.server_name ?? "No runs yet"} ·{" "}
              {(last ?? history[0]) ? formatTs((last ?? history[0])!.ts) : "—"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <ReactECharts option={gauge} style={{ height: 280 }} />
            {(last ?? history[0]) && (
              <div className="mt-2 grid grid-cols-3 gap-2 text-center text-sm">
                <div>
                  <div className="text-[var(--color-muted-foreground)]">Ping</div>
                  <div className="font-semibold">
                    {(last ?? history[0])!.ping_ms.toFixed(1)} ms
                  </div>
                </div>
                <div>
                  <div className="text-[var(--color-muted-foreground)]">Download</div>
                  <div className="font-semibold">
                    {(last ?? history[0])!.download_mbps.toFixed(1)} Mbps
                  </div>
                </div>
                <div>
                  <div className="text-[var(--color-muted-foreground)]">Upload</div>
                  <div className="font-semibold">
                    {(last ?? history[0])!.upload_mbps.toFixed(1)} Mbps
                  </div>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="gap-3">
            <CardTitle>History</CardTitle>
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
            <ReactECharts option={histChart} style={{ height: 280 }} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
