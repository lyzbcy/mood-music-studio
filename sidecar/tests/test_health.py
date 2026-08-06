"""健康检查与安全中间件测试。

MVP-0 阶段验证：
  - /health 返回 200 且含 status=healthy
  - 非本地 Host 头被 403 拒绝（DNS rebinding 防护）
"""
from __future__ import annotations

import sys
from pathlib import Path

# 让 tests/ 能 import app（无 package 安装时）
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from fastapi.testclient import TestClient  # noqa: E402

from app import create_app  # noqa: E402
from app.config import Settings  # noqa: E402


def _client():
    settings = Settings(port=0, data_dir=Path("/tmp/mood-test"))
    # base_url 用 127.0.0.1，让 Host 头通过 DNS rebinding 中间件校验
    # （TestClient 默认 base_url 是 http://testserver，会被安全中间件正确拦截）
    return TestClient(create_app(settings), base_url="http://127.0.0.1")


def test_health_ok():
    with _client() as c:
        r = c.get("/health")
    assert r.status_code == 200
    body = r.json()
    assert body["status"] == "healthy"
    assert body["ok"] is True
    assert "version" in body
    assert "uptime_sec" in body


def test_root_meta():
    with _client() as c:
        r = c.get("/")
    assert r.status_code == 200
    body = r.json()
    assert "Mood Music Studio" in body["name"]


def test_version_endpoint():
    with _client() as c:
        r = c.get("/api/version")
    assert r.status_code == 200
    assert r.json()["ok"] is True


def test_dns_rebinding_blocked():
    """带外部 Host 头的请求应被 403 拒绝。"""
    with _client() as c:
        r = c.get("/health", headers={"Host": "evil.com"})
    assert r.status_code == 403
    assert r.json()["error"]["code"] == "forbidden_host"


def test_localhost_host_allowed():
    """显式 localhost Host 头应放行。"""
    with _client() as c:
        r = c.get("/health", headers={"Host": "127.0.0.1:45170"})
    assert r.status_code == 200
