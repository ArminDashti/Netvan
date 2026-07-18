import { useEffect, useState } from "react";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { rpc, type AppSettings, type CaptureMode } from "@/lib/api";
import { invoke } from "@tauri-apps/api/core";

export function SettingsPage() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [status, setStatus] = useState("");
  const [autostart, setAutostart] = useState(false);
  const [dataDir, setDataDir] = useState("");
  const [msg, setMsg] = useState<string | null>(null);

  const reload = async () => {
    const [s, st, dir] = await Promise.all([
      rpc<{ type: "Settings"; data: AppSettings }>({ method: "GetSettings" }),
      rpc<{ type: "Status"; data: { message: string; capture_mode: string } }>({
        method: "GetStatus",
      }),
      invoke<string>("get_data_dir"),
    ]);
    setSettings(s.data);
    setStatus(st.data.message);
    setDataDir(dir);
    try {
      setAutostart(await isEnabled());
    } catch {
      setAutostart(false);
    }
  };

  useEffect(() => {
    reload().catch((e) => setMsg(String(e)));
  }, []);

  const save = async () => {
    if (!settings) return;
    setMsg(null);
    try {
      await rpc({ method: "SetSettings", params: { settings } });
      setMsg("Settings saved");
      await reload();
    } catch (e) {
      setMsg(e instanceof Error ? e.message : String(e));
    }
  };

  const setMode = async (mode: CaptureMode) => {
    if (!settings) return;
    await rpc({ method: "SetCaptureMode", params: { mode } });
    setSettings({ ...settings, capture_mode: mode });
    await reload();
  };

  const toggleAutostart = async () => {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
      } else {
        await enable();
        setAutostart(true);
      }
      if (settings) {
        const next = { ...settings, start_ui_with_windows: !autostart };
        setSettings(next);
        await rpc({ method: "SetSettings", params: { settings: next } });
      }
    } catch (e) {
      setMsg(e instanceof Error ? e.message : String(e));
    }
  };

  if (!settings) {
    return <div className="text-sm text-[var(--color-muted-foreground)]">Loading…</div>;
  }

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-semibold">Settings</h1>
        <p className="text-sm text-[var(--color-muted-foreground)]">
          Capture mode, targets, retention, startup
        </p>
      </div>

      {msg && (
        <div className="rounded-md border border-[var(--color-border)] bg-[var(--color-muted)] px-3 py-2 text-sm">
          {msg}
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Traffic capture mode</CardTitle>
          <CardDescription>{status}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          <Button
            variant={settings.capture_mode === "process" ? "default" : "outline"}
            onClick={() => setMode("process")}
          >
            Process (app + IP)
          </Button>
          <Button
            variant={settings.capture_mode === "full" ? "default" : "outline"}
            onClick={() => setMode("full")}
          >
            Full (DNS/SNI/Host + WinDivert)
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Targets & intervals</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <label className="block text-sm">
            <span className="text-[var(--color-muted-foreground)]">Ping targets (comma-separated)</span>
            <Input
              className="mt-1"
              value={settings.ping_targets.join(", ")}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  ping_targets: e.target.value
                    .split(",")
                    .map((s) => s.trim())
                    .filter(Boolean),
                })
              }
            />
          </label>
          <label className="block text-sm">
            <span className="text-[var(--color-muted-foreground)]">HTTP targets (comma-separated)</span>
            <Input
              className="mt-1"
              value={settings.http_targets.join(", ")}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  http_targets: e.target.value
                    .split(",")
                    .map((s) => s.trim())
                    .filter(Boolean),
                })
              }
            />
          </label>
          <div className="grid gap-3 sm:grid-cols-3">
            <label className="text-sm">
              <span className="text-[var(--color-muted-foreground)]">Ping interval (s)</span>
              <Input
                type="number"
                className="mt-1"
                value={settings.ping_interval_secs}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    ping_interval_secs: Number(e.target.value) || 5,
                  })
                }
              />
            </label>
            <label className="text-sm">
              <span className="text-[var(--color-muted-foreground)]">HTTP interval (s)</span>
              <Input
                type="number"
                className="mt-1"
                value={settings.http_interval_secs}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    http_interval_secs: Number(e.target.value) || 30,
                  })
                }
              />
            </label>
            <label className="text-sm">
              <span className="text-[var(--color-muted-foreground)]">Retention (days)</span>
              <Input
                type="number"
                className="mt-1"
                value={settings.retention_raw_days}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    retention_raw_days: Number(e.target.value) || 14,
                  })
                }
              />
            </label>
          </div>
          <label className="block text-sm">
            <span className="text-[var(--color-muted-foreground)]">Speedtest CLI path (optional)</span>
            <Input
              className="mt-1"
              value={settings.speedtest_cli_path ?? ""}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  speedtest_cli_path: e.target.value || null,
                })
              }
            />
          </label>
          <Button onClick={save}>Save settings</Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Startup & service</CardTitle>
          <CardDescription>
            UI autostart uses the OS login item. Background collection also runs inside the app;
            install <code className="text-xs">netvan-service</code> for a dedicated Windows service.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Button variant="outline" onClick={toggleAutostart}>
            {autostart ? "Disable start with Windows" : "Start UI with Windows"}
          </Button>
          <div className="text-xs text-[var(--color-muted-foreground)]">
            Data directory: {dataDir}
          </div>
          <pre className="overflow-auto rounded-md bg-[var(--color-muted)] p-3 font-mono text-xs">
{`# Elevated PowerShell — install Windows service
cargo build -p netvan-service --release
.\\target\\release\\netvan-service.exe install
.\\target\\release\\netvan-service.exe start`}
          </pre>
        </CardContent>
      </Card>
    </div>
  );
}
