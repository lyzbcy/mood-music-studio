"""
Mood Music Studio — Python sidecar 入口

被 Tauri 主进程拉起，监听 127.0.0.1:{MOOD_PORT}，对外提供：
  - REST API（库管理 / 打标 / 检索）
  - MCP Server（AI 接管）

启动协议（与 Tauri 主进程约定）：
  环境变量 MOOD_PORT   - 监听端口（必填）
  环境变量 MOOD_DATA   - 数据目录（默认 ~/Library/Application Support/mood-music-studio）
  环境变量 MOOD_TOKEN  - 可选初始 token（首次启动时写入）

架构见 docs/architecture.md。
"""
from __future__ import annotations

import logging
import os
from pathlib import Path

from .__version__ import __version__
from .config import Settings
from .server import create_app

__all__ = ["Settings", "create_app", "get_settings", "configure_logging", "__version__"]


def configure_logging(level: str = "INFO") -> None:
    """统一日志格式，方便 Tauri 主进程采集 sidecar 输出。"""
    fmt = "%(asctime)s [%(levelname)s] %(name)s: %(message)s"
    logging.basicConfig(level=level, format=fmt)


def get_settings() -> Settings:
    """从环境变量读配置。"""
    data_dir = Path(
        os.environ.get(
            "MOOD_DATA",
            Path.home() / "Library" / "Application Support" / "mood-music-studio",
        )
    )
    return Settings(
        port=int(os.environ.get("MOOD_PORT", "0") or "0"),
        host=os.environ.get("MOOD_HOST", "127.0.0.1"),
        data_dir=data_dir,
        log_level=os.environ.get("MOOD_LOG", "INFO"),
        initial_token=os.environ.get("MOOD_TOKEN"),
    )


def main() -> None:
    """CLI 启动入口（开发时直接 python -m app；生产由 PyInstaller 打包）。"""
    import uvicorn

    settings = get_settings()
    configure_logging(settings.log_level)
    log = logging.getLogger("mood.sidecar")

    if not settings.port:
        # 开发模式兜底：端口为 0 时固定 45170，避免 uvicorn 随机分配导致前端找不到
        settings = settings.replace(port=45170)
        log.warning("MOOD_PORT 未设置，开发模式兜底为 %d", settings.port)

    log.info("启动 sidecar：host=%s port=%d data=%s", settings.host, settings.port, settings.data_dir)
    settings.data_dir.mkdir(parents=True, exist_ok=True)

    app = create_app(settings)
    uvicorn.run(
        app,
        host=settings.host,
        port=settings.port,
        log_level=settings.log_level.lower(),
    )


if __name__ == "__main__":
    main()
