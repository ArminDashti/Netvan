import { invoke } from "@tauri-apps/api/core";

export type HistoryRange =
  | "today"
  | "yesterday"
  | "week"
  | "months"
  | "all"
  | "custom";

export type CaptureMode = "process" | "full";

export interface NicInfo {
  id: string;
  guid: string;
  name: string;
  description: string;
  interface_index: number;
  mac: string | null;
  mtu: number | null;
  media_type: string;
  oper_status: string;
  admin_status: string;
  link_speed_bps: number | null;
  ipv4_addresses: string[];
  ipv6_addresses: string[];
  gateways: string[];
  dns_servers: string[];
  dhcp_enabled: boolean;
  wifi_ssid: string | null;
  wifi_bssid: string | null;
  wifi_signal: number | null;
  driver: string | null;
  rx_bytes: number;
  tx_bytes: number;
  rx_bps: number;
  tx_bps: number;
}

export interface AppSettings {
  capture_mode: CaptureMode;
  ping_targets: string[];
  http_targets: string[];
  ping_interval_secs: number;
  http_interval_secs: number;
  bandwidth_interval_ms: number;
  retention_raw_days: number;
  start_ui_with_windows: boolean;
  speedtest_cli_path: string | null;
  speedtest_eula_accepted: boolean;
  default_nic_id: string | null;
}

export interface ServiceStatus {
  running: boolean;
  pipe_connected: boolean;
  capture_mode: string;
  message: string;
}

export type RpcRequest =
  | { method: "Ping" }
  | { method: "GetStatus" }
  | { method: "GetSettings" }
  | { method: "SetSettings"; params: { settings: AppSettings } }
  | { method: "SetCaptureMode"; params: { mode: CaptureMode } }
  | { method: "ListNics" }
  | { method: "GetNic"; params: { nic_id: string } }
  | {
      method: "GetBandwidthHistory";
      params: {
        nic_id: string | null;
        range: HistoryRange;
        start_ts: number | null;
        end_ts: number | null;
      };
    }
  | {
      method: "GetPingHistory";
      params: {
        nic_id: string | null;
        range: HistoryRange;
        start_ts: number | null;
        end_ts: number | null;
      };
    }
  | {
      method: "GetHttpLatencyHistory";
      params: {
        nic_id: string | null;
        range: HistoryRange;
        start_ts: number | null;
        end_ts: number | null;
      };
    }
  | {
      method: "GetLinkEvents";
      params: {
        nic_id: string | null;
        range: HistoryRange;
        start_ts: number | null;
        end_ts: number | null;
      };
    }
  | {
      method: "GetAppUsage";
      params: {
        range: HistoryRange;
        start_ts: number | null;
        end_ts: number | null;
        group_by: string;
      };
    }
  | { method: "RunPing"; params: { target: string; nic_id: string | null } }
  | {
      method: "RunHttpLatency";
      params: { url: string; nic_id: string | null };
    }
  | {
      method: "RunTraceroute";
      params: { target: string; nic_id: string | null; max_hops: number | null };
    }
  | { method: "RunNslookup"; params: { query: string } }
  | {
      method: "RunSpeedtest";
      params: {
        nic_id: string | null;
        server_id: string | null;
        accept_eula: boolean;
      };
    }
  | {
      method: "GetSpeedtestHistory";
      params: {
        range: HistoryRange;
        start_ts: number | null;
        end_ts: number | null;
      };
    }
  | { method: "AcceptSpeedtestEula" };

export type RpcResponse =
  | { type: "Ok" }
  | { type: "Pong" }
  | { type: "Status"; data: ServiceStatus }
  | { type: "Settings"; data: AppSettings }
  | { type: "Nics"; data: NicInfo[] }
  | { type: "Nic"; data: NicInfo }
  | { type: "BandwidthHistory"; data: { ts: number; rx_bytes: number; tx_bytes: number }[] }
  | {
      type: "PingHistory";
      data: {
        nic_id: string | null;
        target: string;
        ts: number;
        rtt_ms: number | null;
        success: boolean;
        error: string | null;
      }[];
    }
  | {
      type: "HttpLatencyHistory";
      data: {
        nic_id: string | null;
        url: string;
        ts: number;
        dns_ms: number | null;
        connect_ms: number | null;
        tls_ms: number | null;
        ttfb_ms: number | null;
        total_ms: number | null;
        status_code: number | null;
        success: boolean;
        error: string | null;
      }[];
    }
  | {
      type: "LinkEvents";
      data: {
        nic_id: string;
        ts: number;
        event: string;
        detail: string | null;
      }[];
    }
  | {
      type: "AppUsage";
      data: {
        process_name: string;
        process_path: string | null;
        remote_ip: string | null;
        host: string | null;
        bytes_in: number;
        bytes_out: number;
        nic_id: string | null;
      }[];
    }
  | {
      type: "PingResult";
      data: {
        nic_id: string | null;
        target: string;
        ts: number;
        rtt_ms: number | null;
        success: boolean;
        error: string | null;
      };
    }
  | {
      type: "HttpLatencyResult";
      data: {
        nic_id: string | null;
        url: string;
        ts: number;
        dns_ms: number | null;
        connect_ms: number | null;
        tls_ms: number | null;
        ttfb_ms: number | null;
        total_ms: number | null;
        status_code: number | null;
        success: boolean;
        error: string | null;
      };
    }
  | {
      type: "Traceroute";
      data: {
        id: number;
        nic_id: string | null;
        target: string;
        ts: number;
        hops: {
          hop: number;
          address: string | null;
          hostname: string | null;
          rtt_ms: number | null;
        }[];
      };
    }
  | {
      type: "Nslookup";
      data: { query: string; records: string[]; raw: string };
    }
  | {
      type: "Speedtest";
      data: {
        id: number;
        nic_id: string | null;
        ts: number;
        server_id: string | null;
        server_name: string | null;
        download_mbps: number;
        upload_mbps: number;
        ping_ms: number;
        jitter_ms: number | null;
        packet_loss: number | null;
        raw_json: string | null;
      };
    }
  | {
      type: "SpeedtestHistory";
      data: {
        id: number;
        nic_id: string | null;
        ts: number;
        server_id: string | null;
        server_name: string | null;
        download_mbps: number;
        upload_mbps: number;
        ping_ms: number;
        jitter_ms: number | null;
        packet_loss: number | null;
        raw_json: string | null;
      }[];
    }
  | { type: "Error"; data: { message: string } };

export async function rpc<T = RpcResponse>(req: RpcRequest): Promise<T> {
  const raw = await invoke<unknown>("rpc", { request: req });
  const normalized = normalizeResponse(raw);
  if (normalized.type === "Error") {
    throw new Error(normalized.data.message);
  }
  return normalized as T;
}

function normalizeResponse(raw: unknown): RpcResponse {
  if (!raw || typeof raw !== "object") {
    return { type: "Error", data: { message: "empty response" } };
  }
  const obj = raw as Record<string, unknown>;
  if ("type" in obj) {
    return raw as RpcResponse;
  }
  // Externally tagged: { "Nics": [...] }
  const key = Object.keys(obj)[0];
  if (!key) {
    return { type: "Error", data: { message: "empty response" } };
  }
  if (key === "Ok" || key === "Pong") {
    return { type: key } as RpcResponse;
  }
  if (key === "Error") {
    const data = obj[key] as { message: string };
    return { type: "Error", data };
  }
  return { type: key, data: obj[key] } as RpcResponse;
}
