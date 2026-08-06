"""FastAPI 应用工厂与生命周期。

MVP-0 阶段只提供：
  - GET  /health            健康检查（Tauri 主进程用）
  - GET  /                  版本/能力元信息
  - GET  /api/version       版本
其余业务路由在后续模块迭代中按 modules/ 文档挂载。

安全：只绑定 127.0.0.1；Host 头校验防 DNS rebinding（见 architecture.md §6）。
"""
from __future__ import annotations

import logging
import os
import time
from contextlib import asynccontextmanager

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from .__version__ import __version__
from .config import Settings

log = logging.getLogger("mood.sidecar.server")

ALLOWED_HOSTS = {"127.0.0.1", "localhost", "[::1]"}


def create_app(settings: Settings) -> FastAPI:
    """构造 FastAPI 实例。

    Args:
        settings: 运行时配置
    """

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        log.info("sidecar 启动完成：%s", settings)
        yield
        log.info("sidecar 关停中")

    app = FastAPI(
        title="Mood Music Studio Sidecar",
        version=__version__,
        docs_url="/docs" if settings.log_level == "DEBUG" else None,
        redoc_url=None,
        lifespan=lifespan,
    )

    # CORS：Tauri WebView 的 origin 可能是 tauri://localhost 或 http://tauri.localhost
    # 这里放宽到 localhost 全家族，生产可收紧
    app.add_middleware(
        CORSMiddleware,
        allow_origins=[
            "tauri://localhost",
            "http://tauri.localhost",
            "http://localhost",
            "http://127.0.0.1",
        ],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    @app.middleware("http")
    async def dns_rebinding_guard(request: Request, call_next):
        """防 DNS rebinding：只允许可信 Host 头。

        攻击场景：恶意网页用 evil.com 解析到 127.0.0.1，再带 evil.com 的 Host 头
        访问本地 API。校验 Host 可阻断。
        """
        host = (request.headers.get("host") or "").split(":")[0].lower()
        if host and host not in ALLOWED_HOSTS:
            log.warning("拒绝非本地 Host 头：%s", request.headers.get("host"))
            return JSONResponse({"ok": False, "error": {"code": "forbidden_host"}}, status_code=403)
        return await call_next(request)

    _started_at = time.time()

    @app.get("/health")
    async def health():
        """健康检查端点。Tauri 主进程启动时轮询此端点判断 sidecar 是否就绪。"""
        return {
            "ok": True,
            "status": "healthy",
            "version": __version__,
            "uptime_sec": round(time.time() - _started_at, 1),
        }

    @app.get("/")
    async def root():
        return {
            "name": "Mood Music Studio Sidecar",
            "version": __version__,
            "docs": "/docs" if settings.log_level == "DEBUG" else "(docs disabled)",
        }

    @app.get("/api/version")
    async def version():
        return {"ok": True, "data": {"version": __version__, "python": "3.9"}}

    return app
