# 路线图 — Mood Music Studio

> 三路并行最小可用（MVP-0）→ 逐路深化 → 产品化。
> 完成每个任务后在这里打勾，并在对应 `modules/` 文档更新进展。

---

## 策略

用户选定：**三者并行最小可用**。即三条功能线各做最薄一层，端到端跑通后再逐个加深。
好处：早期就能验证「打标 → 检索 → 导入」整条链路；避免在某一线过度投入后发现另一线架构冲突。

---

## 当前阶段：Phase 0 — 工程地基 🟡

| 任务 | 状态 | 产出 |
|---|---|---|
| 调研三路（引擎/框架/存储+AI） | ✅ | `docs/research/` |
| 技术选型决策 | ✅ | `docs/decisions/0001` |
| 架构设计 | ✅ | `docs/architecture.md` |
| 文档体系建立 | ✅ | `docs/` 全套 |
| 项目骨架（git + 目录） | ✅ | 本仓库 |
| **开发环境就绪** | ⬜ | Node/Rust/gh 安装 |
| **GitHub 远程仓库** | ⬜ | gh 就绪后创建 |
| **Tauri 工程初始化** | ⬜ | `npm create tauri-app` |
| **Python sidecar 工程初始化** | ⬜ | FastAPI 骨架 + /health |

---

## Phase 1 — MVP-0 三路薄切片（目标：2-3 周）

### 线 A：自动打标签（@see `modules/tagging-engine.md`）
**薄切片目标**：导入 10 首本地音乐 → Essentia 自动打情绪标签 → 按标签筛选并播放。

- [ ] A1. Python sidecar 跑通 Essentia 单首打标（MTG-Jamendo mood/theme 56 标签）
- [ ] A2. 标签写入 SQLite（`auto_tags` 表）
- [ ] A3. 批量扫描目录（10 首），异步任务 + 进度
- [ ] A4. 前端：导入按钮 + 标签筛选侧栏 + 播放器
- [ ] A5. 文档：`modules/tagging-engine.md` 完整化

### 线 B：AI 分镜配乐（@see `modules/ai-scoring.md`）
**薄切片目标**：1 句提示词 → CLAP 检索本地库 → 返回 Top-3 匹配片段。

- [ ] B1. CLAP 模型加载 + 单首音乐 embedding 提取
- [ ] B2. LanceDB 向量库建表 + 批量入库（复用 A 的 10 首）
- [ ] B3. 文本提示词 → CLAP 文本编码 → 向量检索 Top-K
- [ ] B4. 前端：提示词输入框 + 结果展示（片段起止时间 + 播放）
- [ ] B5. 文档：`modules/ai-scoring.md` 完整化

### 线 C：来源 + AI 接管（@see `modules/storage-sync.md`, `modules/mcp-api.md`）
**薄切片目标**：本地库可经 MCP 读写 + 显示剪映音乐 ID。

- [ ] C1. MCP Server 骨架（FastMCP），暴露 `list_tracks` / `get_track` / `search_music`
- [ ] C2. token 鉴权机制
- [ ] C3. 剪映音乐库数据采集（先静态 JSON，后期动态）
- [ ] C4. 前端：「AI 接入」面板（显示 MCP URL + token + 复制按钮）
- [ ] C5. 前端：剪映音乐搜索/ID 显示
- [ ] C6. 文档：`modules/mcp-api.md` + `modules/storage-sync.md` 完整化

### Phase 1 验收标准
- [ ] 三条线在前端各有一个可演示入口
- [ ] A 导入的音乐能被 B 检索到（数据流通）
- [ ] C 的 MCP 能被 Claude Desktop 调用，列出一首音乐
- [ ] 一键 `npm run tauri dev` 启动整条链路

---

## Phase 2 — 深化（MVP-0 跑通后，按用户反馈排期）

### 线 A 深化
- valence-arousal 二维情绪地图可视化
- genre/场景/乐器多维度标签
- 手动标签修正 + 标签合并/ synonym
- 增量扫描、文件变更监听
- beets 集成评估

### 线 B 深化
- 提示词归一化（LLM 把口语 → CLAP 友好英文描述）
- 音效层（Freesound API + CLAP 检索）
- 网络库兜底（Epidemic Sound API）
- 分镜多段配乐时间线
- 导出剪映/PR 配乐方案

### 线 C 深化
- Cloudflare R2 云同步后端（可插拔）
- 剪映深度对接（直接写入剪映工程文件）
- 多设备库同步
- 配乐方案云端备份

---

## Phase 3 — 产品化

- [ ] 双平台 CI 自动构建 + 签名 + 公证
- [ ] 自动更新（tbuffer-plugin-updater）
- [ ] 安装包体积优化（模型按需下载，不打入安装包）
- [ ] 用户文档 / 官网
- [ ] 性能基准（10k 首音乐库的检索延迟）
- [ ] License 确定 / 商用策略（Essentia 授权）

---

## 进度追踪规则

- 完成 Phase 0 的环境任务后，Phase 1 正式启动
- 每周更新本文件的勾选状态
- 同时在 `agent.md` §1 的状态表同步顶层状态
- 遇到阻塞 → 在对应任务下用 `> ⛔ 阻塞：原因` 标注

---

## 风险登记

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Essentia PyInstaller 打包坑（C++ 扩展） | 中 | 高 | 提前验证；备选 conda 打包 / Docker sidecar |
| CLAP 中文提示词效果差 | 高 | 中 | 接入翻译模型；或用中文友好文本编码器微调 |
| Tauri sidecar target-triple 配置复杂 | 中 | 低 | 参考官方 sidecar 文档，构建脚本自动化 |
| 模型体积大（Essentia+CLAP ~2GB） | 高 | 中 | 按需下载，不打入安装包；模型镜像源 |
| AGPL 协议误用 | 低 | 高 | sidecar 隔离 + 法务确认；ADR 记录 |
