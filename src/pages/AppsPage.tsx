import { useEffect, useMemo, useState } from "react";
import ReactECharts from "echarts-for-react";
import { HistoryFilter, customToTs } from "@/components/HistoryFilter";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { rpc, type HistoryRange } from "@/lib/api";
import { formatBytes } from "@/lib/utils";

type Row = {
  process_name: string;
  remote_ip: string | null;
  host: string | null;
  bytes_in: number;
  bytes_out: number;
};

export function AppsPage() {
  const [range, setRange] = useState<HistoryRange>("today");
  const [customStart, setCustomStart] = useState("");
  const [customEnd, setCustomEnd] = useState("");
  const [groupBy, setGroupBy] = useState<"app" | "ip" | "host">("app");
  const [rows, setRows] = useState<Row[]>([]);
  const [mode, setMode] = useState("process");
  const [statusMsg, setStatusMsg] = useState("");

  const load = async () => {
    const start_ts = range === "custom" ? customToTs(customStart) : null;
    const end_ts = range === "custom" ? customToTs(customEnd) : null;
    const [usage, status] = await Promise.all([
      rpc<{ type: "AppUsage"; data: Row[] }>({
        method: "GetAppUsage",
        params: { range, start_ts, end_ts, group_by: groupBy },
      }),
      rpc<{ type: "Status"; data: { capture_mode: string; message: string } }>({
        method: "GetStatus",
      }),
    ]);
    setRows(usage.data);
    setMode(status.data.capture_mode);
    setStatusMsg(status.data.message);
  };

  useEffect(() => {
    load().catch(console.error);
    const id = setInterval(() => load().catch(console.error), 10000);
    return () => clearInterval(id);
  }, [range, customStart, customEnd, groupBy]);

  const chart = useMemo(() => {
    const top = rows.slice(0, 12);
    return {
      backgroundColor: "transparent",
      tooltip: { trigger: "axis" },
      legend: { data: ["In", "Out"], textStyle: { color: "#8b9bb4" } },
      grid: { left: 120, right: 16, top: 28, bottom: 28 },
      xAxis: {
        type: "value",
        axisLabel: { color: "#8b9bb4" },
        splitLine: { lineStyle: { color: "#243049" } },
      },
      yAxis: {
        type: "category",
        data: top.map((r) => r.process_name).reverse(),
        axisLabel: { color: "#8b9bb4", width: 110, overflow: "truncate" },
      },
      series: [
        {
          name: "In",
          type: "bar",
          stack: "t",
          data: top.map((r) => r.bytes_in).reverse(),
          color: "#3d9cf0",
        },
        {
          name: "Out",
          type: "bar",
          stack: "t",
          data: top.map((r) => r.bytes_out).reverse(),
          color: "#3ecf8e",
        },
      ],
    };
  }, [rows]);

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold">Apps</h1>
          <p className="text-sm text-[var(--color-muted-foreground)]">
            Usage by application, remote IP, or hostname
          </p>
        </div>
        <Badge className="capitalize">Mode: {mode}</Badge>
      </div>

      {statusMsg && (
        <p className="text-xs text-[var(--color-muted-foreground)]">{statusMsg}</p>
      )}

      <div className="flex flex-wrap items-center gap-2">
        <HistoryFilter
          value={range}
          onChange={setRange}
          customStart={customStart}
          customEnd={customEnd}
          onCustomStart={setCustomStart}
          onCustomEnd={setCustomEnd}
        />
        <div className="flex gap-1">
          {(["app", "ip", "host"] as const).map((g) => (
            <Button
              key={g}
              size="sm"
              variant={groupBy === g ? "default" : "outline"}
              onClick={() => setGroupBy(g)}
            >
              {g === "app" ? "App" : g === "ip" ? "IP" : "URL/Host"}
            </Button>
          ))}
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Top usage</CardTitle>
        </CardHeader>
        <CardContent>
          <ReactECharts option={chart} style={{ height: 360 }} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Details</CardTitle>
        </CardHeader>
        <CardContent className="overflow-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-left text-[var(--color-muted-foreground)]">
                <th className="py-1">Name</th>
                <th>IP</th>
                <th>Host</th>
                <th>In</th>
                <th>Out</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r, i) => (
                <tr key={i} className="border-t border-[var(--color-border)]">
                  <td className="py-1.5">{r.process_name}</td>
                  <td className="font-mono text-xs">{r.remote_ip ?? "—"}</td>
                  <td className="text-xs">{r.host ?? "—"}</td>
                  <td>{formatBytes(r.bytes_in)}</td>
                  <td>{formatBytes(r.bytes_out)}</td>
                </tr>
              ))}
            </tbody>
          </table>
          {rows.length === 0 && (
            <p className="py-4 text-sm text-[var(--color-muted-foreground)]">
              No usage samples yet — collectors populate this over time.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
