"""配置与数据目录管理。"""
from __future__ import annotations

from dataclasses import dataclass, replace
from pathlib import Path


@dataclass(frozen=True)
class Settings:
    """运行时配置（不可变；变更用 replace）。

    Attributes:
        port: 监听端口（0 表示未由 Tauri 指派，开发兜底为 45170）
        host: 绑定地址，强制 127.0.0.1（见 architecture.md §6 安全模型）
        data_dir: 数据目录（库、向量、模型缓存、日志）
        log_level: 日志级别
        initial_token: 首次启动时写入的可选初始 token
    """

    port: int
    host: str = "127.0.0.1"
    data_dir: Path = Path.home() / "Library" / "Application Support" / "mood-music-studio"
    log_level: str = "INFO"
    initial_token: str | None = None

    def replace(self, **kwargs) -> "Settings":
        return replace(self, **kwargs)

    @property
    def db_path(self) -> Path:
        return self.data_dir / "library.db"

    @property
    def models_dir(self) -> Path:
        return self.data_dir / "models"

    @property
    def lance_dir(self) -> Path:
        return self.data_dir / "lance"

    @property
    def logs_dir(self) -> Path:
        return self.data_dir / "logs"
