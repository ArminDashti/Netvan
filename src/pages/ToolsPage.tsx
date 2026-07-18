import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { rpc } from "@/lib/api";
import { formatMs } from "@/lib/utils";

export function ToolsPage() {
  const [query, setQuery] = useState("example.com");
  const [pingTarget, setPingTarget] = useState("1.1.1.1");
  const [httpUrl, setHttpUrl] = useState("https://www.google.com/generate_204");
  const [traceTarget, setTraceTarget] = useState("1.1.1.1");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);

  const run = async (fn: () => Promise<string>) => {
    setBusy(true);
    try {
      setOutput(await fn());
    } catch (e) {
      setOutput(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-semibold">Tools</h1>
        <p className="text-sm text-[var(--color-muted-foreground)]">
          nslookup, ping, HTTP probe, traceroute
        </p>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>nslookup</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input value={query} onChange={(e) => setQuery(e.target.value)} />
            <Button
              disabled={busy}
              onClick={() =>
                run(async () => {
                  const r = await rpc<{
                    type: "Nslookup";
                    data: { records: string[]; raw: string };
                  }>({ method: "RunNslookup", params: { query } });
                  return `Records:\n${r.data.records.join("\n")}\n\n${r.data.raw}`;
                })
              }
            >
              Lookup
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Ping</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input value={pingTarget} onChange={(e) => setPingTarget(e.target.value)} />
            <Button
              disabled={busy}
              onClick={() =>
                run(async () => {
                  const r = await rpc<{
                    type: "PingResult";
                    data: { success: boolean; rtt_ms: number | null; error: string | null };
                  }>({
                    method: "RunPing",
                    params: { target: pingTarget, nic_id: null },
                  });
                  return r.data.success
                    ? `OK ${formatMs(r.data.rtt_ms)}`
                    : `FAIL ${r.data.error}`;
                })
              }
            >
              Ping
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>HTTP latency</CardTitle>
          </CardHeader>
          <CardContent className="flex gap-2">
            <Input value={httpUrl} onChange={(e) => setHttpUrl(e.target.value)} />
            <Button
              disabled={busy}
              onClick={() =>
                run(async () => {
                  const r = await rpc<{
                    type: "HttpLatencyResult";
                    data: {
                      dns_ms: number | null;
                      connect_ms: number | null;
                      tls_ms: number | null;
                      ttfb_ms: number | null;
                      total_ms: number | null;
                      status_code: number | null;
                      error: string | null;
                    };
                  }>({
                    method: "RunHttpLatency",
                    params: { url: httpUrl, nic_id: null },
                  });
                  const d = r.data;
                  if (d.error) return d.error;
                  return `HTTP ${d.status_code}\nDNS ${formatMs(d.dns_ms)}\nTCP ${formatMs(d.connect_ms)}\nTLS ${formatMs(d.tls_ms)}\nTTFB ${formatMs(d.ttfb_ms)}\nTotal ${formatMs(d.total_ms)}`;
                })
              }
            >
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
            <Button
              disabled={busy}
              onClick={() =>
                run(async () => {
                  const r = await rpc<{
                    type: "Traceroute";
                    data: {
                      hops: { hop: number; address: string | null; rtt_ms: number | null }[];
                    };
                  }>({
                    method: "RunTraceroute",
                    params: {
                      target: traceTarget,
                      nic_id: null,
                      max_hops: 20,
                    },
                  });
                  return r.data.hops
                    .map(
                      (h) =>
                        `${String(h.hop).padStart(2, " ")}  ${h.address ?? "*"}  ${formatMs(h.rtt_ms)}`,
                    )
                    .join("\n");
                })
              }
            >
              Trace
            </Button>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Output</CardTitle>
        </CardHeader>
        <CardContent>
          <pre className="max-h-96 overflow-auto whitespace-pre-wrap rounded-md bg-[var(--color-muted)] p-3 font-mono text-xs">
            {output || "Results appear here."}
          </pre>
        </CardContent>
      </Card>
    </div>
  );
}
