# 模块：MCP API（mcp-api）

> **负责**：对外开放 MCP（Model Context Protocol）接口，让 AI（Claude/GPT/Cursor）能接管软件
> **所属路线**：MVP 线 C（AI 接管）
> **依赖**：FastMCP（或自实现 MCP-over-HTTP）、所有业务模块
> **对接**：sidecar 同时是 FastAPI server + MCP server
> **架构位置**：`architecture.md` §5

---

## 当前状态：⬜ 未开始

| 子任务 | 状态 | 说明 |
|---|---|---|
| MCP server 骨架 | ⬜ | FastMCP 集成 |
| list_tracks / get_track 工具 | ⬜ | 库查询 |
| search_music / score_for_prompt | ⬜ | 检索能力 |
| analyze_track / update_tags | ⬜ | 写入能力 |
| token 鉴权 | ⬜ | 用户授权 |
| 前端「AI 接入」面板 | ⬜ | 显示 URL+token |

---

## 1. 职责边界

**做什么**：
- 把 sidecar 的核心能力包装成 MCP Tools
- 管理 token（生成、校验、吊销）
- 提供 MCP server endpoint（HTTP/SSE transport）

**不做什么**：
- 业务逻辑实现（调用其他模块）
- 前端 UI（desktop-gui 负责）

---

## 2. 暴露的 MCP Tools

| Tool | 入参 | 出参 | 说明 |
|---|---|---|---|
| `list_tracks` | `tag?`, `limit?`, `offset?` | `Track[]` | 列出/筛选音乐库 |
| `get_track` | `track_id` | `Track + 标签 + 来源` | 曲目详情 |
| `analyze_track` | `track_id`, `force?` | `task_id` | 触发自动打标 |
| `update_tags` | `track_id`, `add_tags?`, `remove_tags?` | `ok` | 修改标签 |
| `search_music` | `query`, `filters?` | `Track[]` | 文本/标签检索 |
| `score_for_prompt` | `prompt`, `scope?`, `top_k?`, `include_sfx?` | `ScoreResult` | **分镜配乐核心** |
| `import_music` | `path` 或 `url`, `metadata?` | `track_id` | 导入新音乐 |
| `export_playlist` | `project_id`, `format` | `filepath` | 导出剪映/JSON |

### MCP Tool 描述规范
```python
@mcp.tool()
async def score_for_prompt(
    prompt: str,
    scope: list[str] = ["user_library"],
    top_k: int = 5,
    include_sfx: bool = False,
) -> dict:
    """
    根据分镜提示词，从音乐库智能检索最适合的配乐和音效。

    Args:
        prompt: 分镜描述，如「雨夜都市，主角独行，忧伤悬疑」
        scope: 检索范围，user_library（用户库）和/或 network（网络库）
        top_k: 返回片段数量
        include_sfx: 是否同时检索音效
    """
    ...
```

---

## 3. 鉴权

```python
# 用户在 App 内生成 token（设置 → AI 接入 → 生成新 token）
# token 存 SQLite 的 api_tokens 表

# MCP 请求需带 Authorization header
@app.middleware("http")
async def verify_token(request, call_next):
    token = request.headers.get("Authorization", "").replace("Bearer ", "")
    if not is_valid_token(token):
        return JSONResponse({"error": "unauthorized"}, 401)
    return await call_next(request)
```

### token 生命周期
- 生成：用户点击「生成新 token」，写入 DB，明文只显示一次
- 校验：每次请求比对 hash
- 吊销：用户在设置面板删除某个 token

---

## 4. 用户接入流程

```
1. 用户在 App：「设置 → AI 接入」
     → 看到 MCP URL：http://127.0.0.1:45170/mcp
     → 生成 token：mmst_a1b2c3...
     → 一键复制 mcp.json 配置片段

2. 用户把配置贴进 Claude Desktop 的 mcp.json：
   {
     "mcpServers": {
       "mood-music-studio": {
         "url": "http://127.0.0.1:45170/mcp",
         "headers": { "Authorization": "Bearer mmst_a1b2c3..." }
       }
     }
   }

3. 重启 Claude Desktop → 可调用 list_tracks / score_for_prompt 等工具
   例：「帮我给这段视频分镜配乐，提示词是...」→ Claude 调 score_for_prompt
```

---

## 5. 云端 AI 限制

⚠️ **重要**：ChatGPT（网页/云端）等**无法直接访问用户电脑的 127.0.0.1**。

| AI 类型 | 能否直连 | 方案 |
|---|---|---|
| Claude Desktop（本地运行） | ✅ | 直接连 127.0.0.1 |
| Cursor / 本地 IDE | ✅ | 直接连 127.0.0.1 |
| ChatGPT 网页版 / GPT-4 云端 | ❌ | 需 ngrok/cloudflare tunnel 穿透 |
| 本地运行的开源 MCP host | ✅ | 直接连 |

**推荐**：MVP 只支持本地 AI（Claude Desktop），文档说明云端需用户自配 tunnel。

---

## 6. 安全

- 只绑定 127.0.0.1，不监听 0.0.0.0
- Host 头校验（防 DNS rebinding）
- token 鉴权
- 敏感操作（delete/import）需 token 有对应 scope（未来扩展）

---

## 7. 待决项
- [ ] FastMCP vs 自实现 MCP-over-HTTP（评估成熟度）
- [ ] SSE vs Streamable HTTP transport
- [ ] 是否提供 REST 镜像（除了 MCP，也给传统 HTTP 调用方）
- [ ] 操作日志/审计（记录 AI 调了什么）
