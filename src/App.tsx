import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "@/components/AppShell";
import { DashboardPage } from "@/pages/DashboardPage";
import { NicsPage } from "@/pages/NicsPage";
import { LatencyPage } from "@/pages/LatencyPage";
import { SpeedtestPage } from "@/pages/SpeedtestPage";
import { AppsPage } from "@/pages/AppsPage";
import { ToolsPage } from "@/pages/ToolsPage";
import { SettingsPage } from "@/pages/SettingsPage";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<DashboardPage />} />
          <Route path="nics" element={<NicsPage />} />
          <Route path="latency" element={<LatencyPage />} />
          <Route path="speedtest" element={<SpeedtestPage />} />
          <Route path="apps" element={<AppsPage />} />
          <Route path="tools" element={<ToolsPage />} />
          <Route path="settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
