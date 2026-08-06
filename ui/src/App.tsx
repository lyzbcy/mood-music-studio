import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * MVP-0 验证用主界面。
 *
 * 目标：证明整条链路通了——
 *   Tauri 主进程 → sidecar 端口注入 → 前端调 /health → 显示状态
 *
 * 真正的三栏库管理 UI 在线 A/B 推进时替换（见 docs/modules/desktop-gui.md）。
 */

interface SidecarInfo {
  base_url: string;
  port: number;
}

interface HealthResp {
  ok: boolean;
  status: string;
  version: string;
  uptime_sec: number;
}

type ConnState =
  | { kind: "loading" }
  | { kind: "ok"; info: SidecarInfo; health: HealthResp }
  | { kind: "error"; message: string };

// 纯浏览器开发模式（无 Tauri 时）兜底直连
const DEV_SIDECAR_URL = "http://127.0.0.1:45170";

export default function App() {
  const [state, setState] = useState<ConnState>({ kind: "loading" });

  const probe = useCallback(async () => {
    try {
      let info: SidecarInfo;
      try {
        info = await invoke<SidecarInfo>("get_sidecar_info");
      } catch {
        // 非 Tauri 环境（纯浏览器调试），用兜底地址
        info = { base_url: DEV_SIDECAR_URL, port: 45170 };
      }
      const resp = await fetch(`${info.base_url}/health`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const health = (await resp.json()) as HealthResp;
      if (!health.ok) throw new Error("health.ok = false");
      setState({ kind: "ok", info, health });
    } catch (e) {
      setState({ kind: "error", message: e instanceof Error ? e.message : String(e) });
    }
  }, []);

  useEffect(() => {
    probe();
    // 每 5s 复查一次，sidecar 重启时能自愈
    const t = setInterval(probe, 5000);
    return () => clearInterval(t);
  }, [probe]);

  return (
    <div className="app">
      <header className="header">
        <h1>🎵 Mood Music Studio</h1>
        <span className="version">v0.0.1 · MVP-0</span>
      </header>

      <section className="status-card">
        <StatusView state={state} onRetry={probe} />
      </section>

      <section className="placeholder">
        <h2>🚧 三路 MVP 进行中</h2>
        <ul>
          <li>
            <strong>线 A · 自动打标签</strong>：Essentia 情绪/场景标签
          </li>
          <li>
            <strong>线 B · AI 分镜配乐</strong>：LAION CLAP 语义检索
          </li>
          <li>
            <strong>线 C · 来源 + AI 接管</strong>：MCP API + 剪映对接
          </li>
        </ul>
        <p className="hint">
          见 <code>docs/roadmap.md</code>
        </p>
      </section>
    </div>
  );
}

function StatusView({
  state,
  onRetry,
}: {
  state: ConnState;
  onRetry: () => void;
}) {
  if (state.kind === "loading") {
    return (
      <div className="status loading">
        <Spinner /> 连接 sidecar 中…
      </div>
    );
  }
  if (state.kind === "error") {
    return (
      <div className="status error">
        <span>❌ sidecar 未就绪：{state.message}</span>
        <button onClick={onRetry}>重试</button>
      </div>
    );
  }
  const { info, health } = state;
  return (
    <div className="status ok">
      <span className="dot ok" />
      <div>
        <strong>sidecar 已连接</strong>
        <div className="meta">
          {info.base_url} · v{health.version} · uptime {health.uptime_sec}s
        </div>
      </div>
    </div>
  );
}

function Spinner() {
  return <span className="spinner" />;
}
