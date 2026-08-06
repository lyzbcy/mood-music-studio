# Python Sidecar — Mood Music Studio

> 业务核心进程。被 Tauri 主进程拉起，监听 127.0.0.1，提供 REST + MCP API。
> 架构见 `docs/architecture.md`。

## 开发

```bash
cd sidecar
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# 跑测试
pytest -v

# 启动（开发模式，端口 45170）
MOOD_PORT=45170 MOOD_LOG=DEBUG python -m app
# 或
MOOD_PORT=45170 uvicorn app:asgi_app --port 45170
```

启动后访问 http://127.0.0.1:45170/health 应返回 `{"ok": true, "status": "healthy", ...}`。

## 与 Tauri 的启动协议

| 环境变量 | 必填 | 说明 |
|---|---|---|
| `MOOD_PORT` | 是 | Tauri 选定的空闲端口 |
| `MOOD_DATA` | 否 | 数据目录，默认 `~/Library/Application Support/mood-music-studio` |
| `MOOD_TOKEN` | 否 | 首次启动写入的初始 MCP token |
| `MOOD_LOG` | 否 | 日志级别，默认 INFO |

## 模块演进路线

- ✅ MVP-0：健康检查 + 安全中间件 + 应用工厂
- ⬜ 线 A：`tagging` 路由（Essentia 打标）
- ⬜ 线 B：`score` 路由（CLAP 检索）
- ⬜ 线 C：`library` 路由 + `mcp` server

## 打包（PyInstaller）

待 Rust 主进程骨架稳定后配置 `mood-worker-{target-triple}` 二进制产出。
