# 架构设计 — Mood Music Studio

> 本文档描述系统整体架构、数据流、接口约定。改动架构前必读。
> 配套阅读：`agent.md` §4（架构概要）、`decisions/0001-initial-tech-stack.md`（选型理由）。

---

## 1. 架构总览

**三层架构**：WebView 前端 → Tauri Rust 主进程 → Python sidecar（业务核心）。

```
┌──────────────────────────────────────────────────────────┐
│                    外部 AI（Claude/GPT/Cursor）            │
│                          ↕ MCP 协议                        │
├──────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐    │
│  │  WebView 前端（React + TS）                        │    │
│  │  - 音乐库浏览/筛选/播放                            │    │
│  │  - 分镜提示词输入面板                              │    │
│  │  - 标签编辑/导入导出                               │    │
│  └───────────────────┬──────────────────────────────┘    │
│                      │ fetch → 127.0.0.1:PORT              │
│  ┌───────────────────▼──────────────────────────────┐    │
│  │  Tauri 2 Rust 主进程                              │    │
│  │  - 拉起/守护 Python sidecar（核心职责）            │    │
│  │  - 窗口/托盘/系统对话框                            │    │
│  │  - 受限文件系统访问（Tauri capability）            │    │
│  │  - 自动更新（tbuffer-plugin-updater）              │    │
│  └───────────────────┬──────────────────────────────┘    │
│                      │ 持久 HTTP（localhost only）         │
│  ┌───────────────────▼──────────────────────────────┐    │
│  │  Python sidecar（FastAPI + MCP Server）           │    │
│  │  ┌────────────┐ ┌──────────┐ ┌────────────────┐  │    │
│  │  │ 打标签引擎  │ │ 检索引擎 │ │ 库管理器       │  │    │
│  │  │ Essentia   │ │ LAION    │ │ SQLite +       │  │    │
│  │  │ librosa    │ │ CLAP     │ │ LanceDB(向量) │  │    │
│  │  └────────────┘ └──────────┘ └────────────────┘  │    │
│  └───────────────────┬──────────────────────────────┘    │
│                      │ 可插拔                              │
│  ┌───────────────────▼──────────────────────────────┐    │
│  │  存储层（抽象接口）                                │    │
│  │  - LocalStorage（MVP，文件系统 + SQLite）          │    │
│  │  - R2Storage（后做，Cloudflare R2）                │    │
│  │  - GithubReleaseStorage（hack 备用）               │    │
│  └──────────────────────────────────────────────────────┘
│  应用边界（应用进程内部，不暴露公网）                        │
└──────────────────────────────────────────────────────────┘
```

### 设计原则
1. **进程隔离**：Tauri 主进程不直接调用 Essentia，避免 AGPL 传染。Python sidecar 是独立 OS 进程。
2. **单一 HTTP 接口**：前端、外部 AI、内部组件全部走 `127.0.0.1:PORT` 的 HTTP/MCP。不搞 Tauri IPC + HTTP 双协议。
3. **本地优先**：所有数据本地，云同步是可选插件。
4. **127.0.0.1 only**：API 只绑定 localhost，不暴露公网。外部 AI 接管需经用户授权（见 §6 安全）。

---

## 2. 进程模型

### 2.1 启动时序
```
用户双击应用
  → Tauri 主进程启动
    → 查找空闲端口（默认 45170，被占则递增）
    → 拉起 Python sidecar，传入 PORT 与数据目录
    → sidecar 启动 FastAPI + MCP，监听 PORT
    → Tauri 主进程健康检查 sidecar（GET /health，重试 30 次）
    → WebView 加载前端，前端 → http://127.0.0.1:PORT
    → 应用就绪
```

### 2.2 关停时序
```
用户关闭窗口
  → Tauri 主进程收到 close 事件
    → 调用 sidecar 的 POST /shutdown（优雅关停进行中的分析任务）
    → 等待 sidecar 进程退出（超时 5s 后 SIGTERM）
    → 主进程退出
```

### 2.3 sidecar 崩溃恢复
- Tauri 主进程监听 sidecar 子进程 exit 事件
- 非预期退出 → 自动重启（最多 3 次，超过则弹错误提示）
- 进行中的长任务（如批量打标）需支持断点续传（记录到 SQLite）

---

## 3. 数据模型

### 3.1 核心实体（SQLite）

```sql
-- 音乐曲目
CREATE TABLE tracks (
  id            TEXT PRIMARY KEY,          -- uuid
  file_path     TEXT NOT NULL,             -- 本地绝对路径
  file_hash     TEXT UNIQUE,               -- sha256 前 16 位，去重用
  title         TEXT,
  artist        TEXT,
  album         TEXT,
  duration_sec  REAL,
  sample_rate   INTEGER,
  -- 用户元数据
  user_tags     TEXT,                      -- JSON array，用户手动标签
  user_notes    TEXT,
  -- 来源
  source        TEXT DEFAULT 'local',      -- local | cloud | jianying
  source_ref    TEXT,                      -- 剪映音乐 ID 等
  -- 时间戳
  created_at    TEXT DEFAULT (datetime('now')),
  updated_at    TEXT DEFAULT (datetime('now')),
  analysis_status TEXT DEFAULT 'pending'   -- pending|analyzing|done|failed
);

-- 自动标签（多对一关系，便于查询）
CREATE TABLE auto_tags (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  track_id    TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  tag_type    TEXT NOT NULL,               -- mood|genre|scene|instrument|energy|valence_arousal
  tag_value   TEXT NOT NULL,               -- happy, dark, electronic, ...
  confidence  REAL,                        -- 0.0-1.0
  source      TEXT,                        -- essentia|jamendo|lastfm|user|clap
  metadata    TEXT                         -- JSON，如 valence/arousal 数值
);
CREATE INDEX idx_auto_tags_track ON auto_tags(track_id);
CREATE INDEX idx_auto_tags_type_value ON auto_tags(tag_type, tag_value);

-- 向量索引（CLAP embedding，同步写入 LanceDB）
-- LanceDB 表：clip_embeddings
--   track_id TEXT, start_sec REAL, end_sec REAL, vector FLOAT[512]

-- 分镜配乐项目（AI 配乐工作流的产出）
CREATE TABLE scoring_projects (
  id          TEXT PRIMARY KEY,
  name        TEXT,
  prompt      TEXT,                        -- 用户输入的分镜提示词
  result      TEXT,                        -- JSON，配乐方案
  created_at  TEXT DEFAULT (datetime('now'))
);
```

### 3.2 数据目录布局
```
~/Library/Application Support/mood-music-studio/   (macOS)
%APPDATA%/mood-music-studio/                       (Windows)
├── config.json          # 用户配置（API key、端口、库路径）
├── library.db           # SQLite 主库
├── lance/               # LanceDB 向量数据
├── models/              # 下载的 Essentia/CLAP 预训练模型（缓存）
├── cache/               # 音频分析中间产物
└── logs/
```

---

## 4. 接口设计（FastAPI）

> 所有接口绑定 `127.0.0.1`。统一返回 `{ "ok": bool, "data"?: ..., "error"?: {...} }`。

### 4.1 库管理
| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/library/scan` | 扫描指定目录，导入新音乐 |
| GET | `/api/library/tracks` | 列出曲目，支持 `?tag=mood:happy&limit=50` 筛选 |
| GET | `/api/library/tracks/{id}` | 曲目详情（含所有标签） |
| PATCH | `/api/library/tracks/{id}` | 更新用户标签/备注 |
| GET | `/api/library/tracks/{id}/audio` | 流式返回音频文件（支持 Range） |
| DELETE | `/api/library/tracks/{id}` | 移除曲目（默认不删原文件） |

### 4.2 打标签
| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/tag/analyze` | 对单首/多首音乐跑 Essentia 打标（异步任务） |
| GET | `/api/tag/task/{id}` | 查询打标任务进度 |
| POST | `/api/tag/retag/{track_id}` | 重新分析某首 |

### 4.3 AI 配乐检索
| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/score/search` | 核心接口：提示词 → 匹配音乐 |
```json
// POST /api/score/search
{
  "prompt": "雨夜都市，主角独行，忧伤悬疑",
  "scope": ["user_library", "network"],     // 检索范围，用户库优先
  "top_k": 5,
  "duration_hint_sec": 30,                  // 期望片段长度
  "include_sfx": true                       // 是否同时检索音效
}
// 返回
{
  "ok": true,
  "data": {
    "normalized_query": "rainy night city, lonely protagonist, melancholic suspense",
    "matches": [
      {
        "track_id": "...", "title": "...", "source": "user_library",
        "snippet": { "start_sec": 45.0, "end_sec": 75.0, "score": 0.87 },
        "matched_tags": ["dark", "cinematic", "atmospheric"]
      }
    ],
    "sfx": [ /* Freesound 匹配的音效 */ ]
  }
}
```

### 4.4 来源/导入
| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/source/jianying/search` | 查询剪映音乐库对应 ID/名称 |
| POST | `/api/source/import` | 从 URL/文件路径导入音乐 |

---

## 5. MCP Server（对外 AI 接管）

sidecar 同时暴露 MCP（Model Context Protocol）server，让 Claude Desktop / Cursor / GPT 等能直接接管软件。

### 暴露的 MCP Tools
| Tool | 说明 |
|---|---|
| `list_tracks` | 列出音乐库，支持标签筛选 |
| `get_track` | 获取某首音乐详情 |
| `analyze_track` | 触发自动打标 |
| `search_music` | 文本/标签检索音乐 |
| `score_for_prompt` | 分镜提示词 → 配乐方案 |
| `import_music` | 导入新音乐到库 |
| `update_tags` | 修改标签 |
| `export_playlist` | 导出配乐方案为剪映/JSON 格式 |

### 接入流程（用户视角）
1. 用户在 App 里「设置 → AI 接入」生成一个 token
2. App 显示 MCP server URL：`http://127.0.0.1:PORT/mcp`
3. 用户把 URL + token 配置进 Claude Desktop 的 `mcp.json`
4. Claude 即可调用上述 tools 自动配乐

> ⚠️ **云 AI 限制**：ChatGPT 等云端 AI 无法直接访问用户 localhost。需 ngrok/cloudflare tunnel 穿透，或用户运行本地 MCP host（Claude Desktop）。详见 `research/03` §问题二。

---

## 6. 安全模型

| 威胁 | 防护 |
|---|---|
| 局域网内其他设备访问 API | 只绑定 `127.0.0.1`，不监听 `0.0.0.0` |
| 恶意网页 fetch 本地 API（DNS rebinding） | 校验 `Host` 头必须为 `127.0.0.1`/`localhost`；前端跨域白名单 |
| 未授权 AI 接管 | MCP API 需 token；token 在 App 内生成、可吊销 |
| 用户上传恶意音频 | 音频解码隔离在 sidecar 子进程，崩溃不影响主程序 |
| 文件系统越权 | Tauri capability 限制可访问目录；sidecar 只读用户授权的库目录 |

---

## 7. 打包与分发

### 7.1 双平台构建（GitHub Actions 矩阵）
```yaml
# .github/workflows/release.yml（要点）
strategy:
  matrix:
    include:
      - os: macos-latest
        target: universal-apple-darwin    # 通用二进制（Intel + Apple Silicon）
      - os: windows-latest
        target: x86_64-pc-windows-msvc
steps:
  - 构建 Python sidecar（PyInstaller，各平台分别跑）
  - 把 sidecar 二进制放到 src-tauri/binaries/，按 target-triple 命名
  - npm run tauri build
  - 上传 .dmg / .msi 到 Release
```

### 7.2 sidecar target-triple 命名（Tauri 要求）
```
src-tauri/binaries/
├── mood-worker-aarch64-apple-darwin        # Mac Apple Silicon
├── mood-worker-x86_64-apple-darwin         # Mac Intel
└── mood-worker-x86_64-pc-windows-msvc.exe  # Windows
```

### 7.3 代码签名（正式发布前）
- macOS：需 Apple Developer ID（$99/年）+ 公证（notarization）
- Windows：EV 证书（数百美元）或 SmartScreen 积累
- MVP 阶段可不签名，用户首次安装手动允许

---

## 8. 技术债与待定项

| # | 项 | 处理时机 |
|---|---|---|
| 1 | beets 集成 vs 自研库管理 | MVP-0 后评估 |
| 2 | 向量库选型最终确认（LanceDB vs Chroma） | MVP-0 时基准测试 |
| 3 | 前端框架最终确认（React vs Svelte） | GUI 编码前 |
| 4 | Essentia 商用授权策略 | 商业化前 |
| 5 | 音频预处理流水线（响度归一、切片策略） | AI 配乐深化时 |
