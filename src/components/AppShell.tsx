import { NavLink, Outlet } from "react-router-dom";
import {
  Activity,
  AppWindow,
  Gauge,
  Network,
  Settings,
  Timer,
  Wrench,
  Zap,
} from "lucide-react";
import { cn } from "@/lib/utils";

const NAV = [
  { to: "/", label: "Dashboard", icon: Gauge },
  { to: "/nics", label: "NICs", icon: Network },
  { to: "/latency", label: "Latency", icon: Timer },
  { to: "/speedtest", label: "Speed Test", icon: Zap },
  { to: "/apps", label: "Apps", icon: AppWindow },
  { to: "/tools", label: "Tools", icon: Wrench },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function AppShell() {
  return (
    <div className="flex h-full min-h-0">
      <aside className="flex w-56 shrink-0 flex-col border-r border-[var(--color-border)] bg-[var(--color-card)]/70 backdrop-blur">
        <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-4 py-4">
          <Activity className="h-5 w-5 text-[var(--color-primary)]" />
          <div>
            <div className="text-sm font-semibold tracking-wide">Netvan</div>
            <div className="text-[11px] text-[var(--color-muted-foreground)]">
              Network Monitor
            </div>
          </div>
        </div>
        <nav className="flex flex-1 flex-col gap-1 p-2">
          {NAV.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2 rounded-md px-3 py-2 text-sm text-[var(--color-muted-foreground)] transition-colors hover:bg-[var(--color-muted)] hover:text-[var(--color-foreground)]",
                  isActive &&
                    "bg-[var(--color-muted)] text-[var(--color-foreground)] font-medium",
                )
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="border-t border-[var(--color-border)] p-3 text-[11px] text-[var(--color-muted-foreground)]">
          Local SQLite · Windows
        </div>
      </aside>
      <main className="min-w-0 flex-1 overflow-auto p-5">
        <Outlet />
      </main>
    </div>
  );
}
