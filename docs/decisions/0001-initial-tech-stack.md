# ADR-0001：初始技术栈选型

- **状态**：已接受 ✅
- **日期**：2026-08-06
- **决策者**：泽恩（用户）+ ZCode
- **关联**：`docs/research/01,02,03`、`docs/architecture.md`

---

## 背景

新项目「Mood Music Studio」（音乐自动打标签 + AI 配乐）需要确定初始技术栈。约束条件（来自用户原始需求）：

1. Win + Mac 通用，一次开发两平台能用，能分别打包
2. 核心是音频分析（自动打情绪/场景标签），需调用成熟的音频/ML 库
3. 开放 API 给 AI 接管
4. 音乐来源：本地优先，云同步后做，剪映 ID 兜底
5. 个人开发者，追求开发效率，避免重造轮子

---

## 决策

### 决策 1：桌面框架 = Tauri 2 + Python sidecar

**选择**：Tauri 2（Rust 主进程）+ PyInstaller 打包的 Python sidecar（FastAPI）。

**否决方案及原因**：
| 方案 | 否决原因 |
|---|---|
| Electron + Python sidecar | 包体积 200MB+、内存 150-400MB。功能能实现但太重，个人项目不划算 |
| Flutter Desktop | 无 librosa/essentia 的 Dart 等价物，音频生态空白；Google 桌面战略投入下降 |
| PySide6 纯 Python | UI 现代化弱于 Web 生态；分发链路有坑；用户明确要「Web UI」感 |
| Rust 重写音频算法 | librosa/essentia 等价物在 Rust 不存在，重写成本远大于 sidecar 体积代价 |
| Wails (Go) | 生态小，对音频场景无明显优势，Go 团队才考虑 |

**关键理由**：
1. **音频生态完整**：sidecar 保留 librosa/essentia/LAION CLAP 全部能力，零算法重写
2. **AGPL 隔离**：Essentia 是 AGPL-3.0，进程隔离可规避传染（静态/动态链接则传染主程序）
3. **体积小**：Tauri 本体几 MB，总包大小由 Python sidecar 决定（30-80MB），比 Electron 轻一大截
4. **统一 API**：sidecar 自带 FastAPI，前端/外部 AI/内部组件走同一 HTTP 接口
5. **Win+Mac 官方支持**：CI 矩阵构建成熟

**风险**：Rust 学习曲线、Tauri sidecar target-triple 配置、Essentia PyInstaller 打包坑。已记入 `roadmap.md` 风险登记。

### 决策 2：存储 = 本地优先，云同步可插拔

**选择**：MVP 阶段纯本地（文件系统 + SQLite + LanceDB 向量），云同步设计成可插拔后端接口。

**否决方案及原因**：
| 方案 | 否决原因 |
|---|---|
| GitHub 存音乐（用户原始想法） | 单文件 100MB 限制；仓库超 1GB 会被告；`objects.githubusercontent.com` 国内常超时；有违反 ToS 被封仓风险。仅适合极小 demo |
| Cloudflare R2 作为 MVP 主存储 | 技术上最佳（10GB免费+零流量费），但增加初期复杂度（需用户注册 R2、配 key）。留作 Phase 2 后端 |
| Google Drive / Dropbox API | 服务条款禁止当媒体 CDN，滥用封号；国内被墙 |

**关键理由**：用户明确选「本地优先，云同步后做」。本地先跑通核心价值（打标+检索），云同步是锦上添花。
**未来云后端推荐**：Cloudflare R2（详见 `research/03`）。

### 决策 3：音频打标引擎 = Essentia（+ librosa 辅助）

**选择**：Essentia 作为主引擎（用其内置的 MTG-Jamendo mood/theme 56 标签模型 + valence/arousal 回归 + Discogs genre）。

**否决方案及原因**：
| 方案 | 否决理由 |
|---|---|
| 纯 librosa 自训模型 | librosa 不带预训练模型，需自训，周期长。Essentia 已含等价特征能力 |
| sota-music-tagging-models (MIT) | 协议干净，但需自己接数据训/下载 checkpoint；Essentia 内置模型就是这套的训练产物，更省事 |
| 依赖在线 API（Spotify/Last.fm） | Spotify Audio Features 已废弃（404）；Last.fm 有限速不稳。本地模型离线、免费、无限速 |

**关键理由**：Essentia 一行 Python 拿 56 个情绪标签 + 二维情绪坐标，是本项目「打标签」地基。
**协议处理**：AGPL → 用 sidecar 进程隔离调用，主程序不链接。

### 决策 4：AI 配乐 = LAION CLAP 语义检索

**选择**：LAION CLAP（文本↔音频跨模态模型）做核心检索，Freesound + Epidemic Sound 做网络库补充。

**关键理由**：
- CLAP 预训练权重直接可用，文本编码器和音频编码器已对齐
- 完美匹配「分镜提示词 → 检索音乐片段」需求
- HuggingFace Transformers 已集成，几行 Python 调用
- 相比生成型（Suno/Udio）：检索型从用户已有库找，免费、快、版权清晰

### 决策 5：AI 接管协议 = MCP

**选择**：sidecar 同时作为 MCP Server，暴露 tools 给 Claude Desktop / Cursor 等。

**关键理由**：
- MCP 是 2024+ AI 接管本地工具的事实标准
- 比 REST 更适合 AI 调用（Tool 描述自描述）
- 本地 AI（Claude Desktop）可直接连 127.0.0.1；云端 AI 需 tunnel，文档说明

---

## 综合技术栈

| 层 | 选型 |
|---|---|
| 桌面壳 | Tauri 2 |
| 前端 | React + TypeScript（暂定，待 B 评估 Svelte） |
| 后端 | Python 3.11+ / FastAPI |
| 打标引擎 | Essentia + librosa |
| 语义检索 | LAION CLAP |
| 向量库 | LanceDB（待基准确认） |
| 元数据存储 | SQLite |
| 库管理 | 自研轻量（beets 作为 Phase 2 评估） |
| AI 接管 | MCP (FastMCP) |
| 云存储 | 本地优先；Phase 2 加 R2 |
| CI/打包 | GitHub Actions 矩阵（macos + windows） |

---

## 后果

**正面**：
- 一次开发 Win+Mac 双平台
- 音频/ML 生态完整复用，不自研算法
- AGPL 风险可控
- AI 接管开箱即用

**负面/风险**：
- Rust 有学习曲线（但 sidecar 边界清晰，主进程逻辑不复杂）
- Essentia PyInstaller 打包是已知坑（librosa 也有 issue），需提前验证
- 双平台 sidecar 二进制管理复杂（target-triple 命名 + CI 矩阵）
- 模型体积大（Essentia + CLAP ~2GB），需按需下载策略

**待办**：
- Phase 0 验证 Essentia PyInstaller 打包（最高风险点）
- Phase 1 验证 CLAP 中文提示词效果

---

## 参考

- [Tauri 2 Sidecar 官方文档](https://v2.tauri.app/develop/sidecar/)
- [Essentia 模型页](https://essentia.upf.edu/models.html)
- [LAION CLAP](https://github.com/LAION-AI/CLAP)
- [Cloudflare R2 定价](https://developers.cloudflare.com/r2/pricing/)
- 完整对比见 `docs/research/01,02,03`
