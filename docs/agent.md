# agent.md — 项目全貌与 AI 接手指南

> **本文档是项目入口文件。任何 AI（或人类）接手本项目，请先完整阅读本文档。**
> 它回答三个问题：**这是个什么项目？现在做到哪了？我该怎么开始干活？**

---

## 0. 一句话定位

**Mood Music Studio** 是一个「音乐自动打标签 + 按情绪/场景智能分类管理」的跨平台桌面软件（Win + Mac）。
核心痛点：**创作者找音乐难**。核心价值：给每首音乐打上情绪/场景/节奏标签，用「分镜提示词」一搜，智能匹配出最适合的音乐和音效，并能快捷导入到剪辑工具。

---

## 1. 当前状态 🟡

| 维度 | 状态 | 说明 |
|---|---|---|
| 调研 | ✅ 完成 | 三份调研报告已归档至 `docs/research/`，技术选型已定 |
| 架构设计 | ✅ 完成 | 见 `docs/architecture.md`，技术栈已定 |
| MVP 规划 | ✅ 完成 | 见 `docs/roadmap.md`，三路并行最小可用 |
| 项目骨架 | ✅ 完成 | 目录结构、文档体系已建立 |
| 代码 | ⬜ 未开始 | 待环境就绪后进入编码 |
| 环境 | ⬜ 未就绪 | 缺 Node.js / Rust / gh CLI / Homebrew，详见 §8 |

**下一步动作**：安装开发环境 → 搭建 Tauri + Python sidecar 工程骨架 → 跑通三路 MVP 各一层薄切片。详见 `docs/roadmap.md`。

---

## 2. 接手第一步（给后来 AI 的 5 分钟上手）

1. **读本文件**——你已经在这里了。
2. **按职责读模块文档**：只读你负责的模块，不必全读。导航见 §7 文档地图。
3. **读 ADR**：`docs/decisions/` 记录了所有技术决策的「为什么」，避免你重走已被否决的路。
4. **查路线图**：`docs/roadmap.md` 告诉你当前该做什么、做到什么程度。
5. **遵守渐进式文档原则**：更新代码时同步更新对应模块文档，写明进展。每完成一个子任务，在模块文档顶部更新「最新进展」时间戳。

> **黄金法则**：你接手后做的任何决策，如果会改变技术方向，请新建一条 ADR（`docs/decisions/`），不要默默改掉。

---

## 3. 项目全貌

### 3.1 核心痛点
创作者在剪辑视频/做漫剧时，**找不到合适的音乐**。现有问题：
- 本地音乐文件杂乱，没有统一的情绪/场景分类
- 剪映等工具的音乐库搜索维度单一，无法按「这一段需要什么情绪」精准命中
- 音乐和音效分散在不同来源，没有一个统一的管理入口

### 3.2 解决方案（四大能力）

| 能力 | 说明 | MVP 状态 |
|---|---|---|
| **① 自动打标签** | 导入音乐 → Essentia 自动打情绪(56 标签)/场景/genre/valence-arousal 标签 → 可手动修正补充 | 薄切片 |
| **② 智能检索配乐** | 输入分镜提示词 → LAION CLAP 语义检索 → 优先用户库、补充网络库 → 返回每段最适合的音乐+音效 | 薄切片 |
| **③ 音乐来源管理** | 本地库（MVP 重点）→ 云端同步（后做，可插拔）→ 剪映对应音乐 ID 显示（兜底） | 本地优先 |
| **④ AI 接管** | 开放 MCP API，AI 可读取/打标/检索/导入，实现自动化配乐工作流 | 薄切片 |

### 3.3 音乐来源优先级（用户原始要求）

```
首选：上传到免费云端（调研后调整为：本地优先，云同步后做）
次选：本地链接音乐 ← MVP 重点
兜底：显示剪映里对应音乐的 ID/名称
```

> ⚠️ **调研修正**：用户最初想用 GitHub 存音乐。调研发现 GitHub 存音乐有硬伤（单文件 100MB 限制、国内访问不稳、有封仓风险）。已调整为 **Cloudflare R2 作为未来云后端**（10GB 免费 + 零流量费），但 MVP 阶段纯本地。详见 `docs/research/03-storage-and-ai-music.md`。

### 3.4 AI 配乐工作流（核心差异化）

```
用户输入分镜提示词："雨夜都市，主角独行，忧伤悬疑"
         │
         ▼
① 提示词归一化（LLM 把口语 → 标准音乐描述）
         │
         ▼
② 语义检索（LAION CLAP 文本编码器 → 向量）
         │
         ▼
③ 向量库 ANN 检索（用户本地库优先 + 网络库补充）
         │
         ▼
④ 音效层叠加（Freesound + CLAP 同套检索）
         │
         ▼
⑤ 输出：分镜 → 配乐 + 音效 时间线 + 快捷导入
```

---

## 4. 技术架构（概要）

> 完整版见 `docs/architecture.md`。这里只放接手者必须立刻知道的。

**架构**：Tauri 2（Rust 主进程）+ Python sidecar（FastAPI，跑 Essentia/CLAP）+ WebView 前端。

```
┌─────────────────────────────────────────────┐
│  前端 (React/Vue) — 系统 WebView              │
└──────────────┬──────────────────────────────┘
               │ HTTP (127.0.0.1)
┌──────────────▼──────────────────────────────┐
│  Tauri Rust 主进程                            │
│   - 拉起/守护 Python sidecar                  │
│   - 文件系统、窗口、托盘                       │
└──────────────┬──────────────────────────────┘
               │ 持久本地 HTTP (FastAPI on 127.0.0.1)
┌──────────────▼──────────────────────────────┐
│  Python sidecar (PyInstaller 打包)            │
│   - Essentia 音频特征 + 情绪打标              │
│   - LAION CLAP 语义检索                       │
│   - 向量库 (LanceDB)                          │
│   - MCP Server (对外 AI 接管入口)             │
└──────────────────────────────────────────────┘
```

**为什么这样**：
- Python sidecar 保留了 librosa/essentia/CLAP 全部生态能力（零算法重写）
- sidecar 进程隔离解决了 Essentia 的 AGPL 协议传染问题
- 统一 HTTP API：前端、外部 AI、内部调用走同一套接口，架构最干净
- MCP Server 让 Claude/GPT 等能直接接管软件

---

## 5. 技术栈速查

| 层 | 选型 | 用途 | License 关注 |
|---|---|---|---|
| 桌面壳 | **Tauri 2** | Win+Mac 通用打包 | MIT/Apache |
| 前端 | React + TypeScript | UI | - |
| 后端语言 | **Python 3.11+** | 音频处理 | - |
| 音频特征/打标 | **Essentia** + librosa | 情绪/genre/BPM | ⚠️ AGPL-3.0（靠 sidecar 隔离规避） |
| 语义检索 | **LAION CLAP** | 文本→音乐检索 | MIT |
| 向量库 | **LanceDB** | 本地音乐片段向量 | Apache-2.0 |
| 后端框架 | **FastAPI** | sidecar HTTP 服务 | MIT |
| AI 接管 | **MCP Server** | 对外开放给 AI | - |
| 库管理底座 | **beets**（评估中） | 文件组织+元数据 | MIT |
| 云存储（后做） | **Cloudflare R2** | 可插拔云后端 | - |

---

## 6. MVP 路线图（三路并行最小可用）

> 完整版见 `docs/roadmap.md`。

策略：三条线各做**最薄一层**，端到端跑通后再逐个加深。

| 线 | 薄切片目标（MVP-0） | 深化方向 |
|---|---|---|
| **A. 自动打标签** | 导入 10 首 → Essentia 打情绪标签 → 按标签筛选播放 | 批量、valence-arousal 二维情绪地图、手动修正 |
| **B. AI 分镜配乐** | 1 句提示词 → CLAP 检索本地库 → 返回 Top-3 匹配片段 | 音效层、网络库兜底、时间线 |
| **C. 来源+AI接管** | 本地库可读写 MCP API + 显示剪映音乐 ID | 云同步、R2 后端、剪映深度对接 |

---

## 7. 文档地图（导航）

> **渐进式原则**：只读你负责的模块文档。每个模块文档自包含，不依赖其他模块。

### 入口文档（所有人都读）
| 文档 | 内容 | 何时读 |
|---|---|---|
| `agent.md`（本文件） | 项目全貌、接手指南 | **首先读** |
| `architecture.md` | 架构详解、数据流、接口设计 | 设计/改动架构前 |
| `roadmap.md` | MVP 路线、任务拆解、进度 | 开始任何任务前 |

### 调研报告（需要背景时读）
| 文档 | 内容 |
|---|---|
| `research/01-audio-tagging-engines.md` | Essentia/librosa/beets 等 13 个项目调研 |
| `research/02-desktop-frameworks.md` | Tauri/Electron/Flutter/Wails/PySide 对比 |
| `research/03-storage-and-ai-music.md` | GitHub/R2/S3 存储对比 + CLAP/Epidemic/Freesound 配乐调研 |

### 架构决策记录（ADR，避免重走弯路）
| 文档 | 决策 |
|---|---|
| `decisions/0001-initial-tech-stack.md` | 为什么选 Tauri+Python sidecar、为什么本地优先 |

### 模块文档（按职责读）
| 文档 | 模块 | 负责什么 |
|---|---|---|
| `modules/tagging-engine.md` | 打标签引擎 | Essentia 集成、标签体系、批量打标 |
| `modules/library-manager.md` | 音乐库管理 | 导入/索引/筛选/播放 |
| `modules/ai-scoring.md` | AI 分镜配乐 | CLAP 检索、提示词归一化、音效叠加 |
| `modules/storage-sync.md` | 存储同步 | 本地存储 + 云后端可插拔 + 剪映对接 |
| `modules/desktop-gui.md` | 桌面 GUI | Tauri 前端、交互、打包 |
| `modules/mcp-api.md` | MCP API | 对外接口、AI 接管协议 |

---

## 8. 环境信息

### 8.1 开发机当前状态（2026-08-06 探测）
| 工具 | 状态 | 说明 |
|---|---|---|
| Python | ✅ 3.9.6 | 建议升级到 3.11+（essentia/CLAP 生态更稳） |
| git | ✅ 2.50.1 | 已配置 泽恩 / lyzbcy@gmail.com |
| Node.js | ❌ 未安装 | Tauri 前端需要，待装 |
| Rust | ❌ 未安装 | Tauri 主进程需要，待装 |
| Homebrew | ❌ 未安装 | macOS 包管理，待装 |
| gh CLI | ❌ 未安装 | 创建 GitHub 仓库/推送需要，待装 |

### 8.2 待安装清单
```bash
# 1. Homebrew（macOS 包管理基础）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. Node.js（Tauri 前端构建）
brew install node

# 3. Rust（Tauri 主进程）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. gh CLI（GitHub 操作）
brew install gh
gh auth login

# 5. Tauri CLI（Rust 安装后）
cargo install tauri-cli --version "^2.0"

# 6. Python 依赖（sidecar）
pip install essentia librosa fastapi uvicorn pylance clap-transformers
```

### 8.3 项目路径
```
本地：~/Documents/共享/创业/mood-music-studio/
远程：待创建 GitHub public 仓库 mood-music-studio（gh 就绪后）
```

---

## 9. 开发规范

### 9.1 分支策略
- `main`：稳定可演示
- `dev`：开发集成
- `feature/xxx`：功能分支
- `fix/xxx`：修复分支

### 9.2 提交规范（Conventional Commits）
```
feat: 新功能
fix: 修复
docs: 文档
refactor: 重构
chore: 杂项
research: 调研（本项目专属类型）
```

### 9.3 文档更新纪律
- 改代码 → 同步改对应 `modules/` 文档
- 做技术决策 → 新建 ADR
- 完成里程碑 → 更新 `roadmap.md` 状态 + 本文件 §1 状态表

### 9.4 AGPL 协议规避（重要）
Essentia 是 AGPL-3.0，有传染性。**必须**通过 sidecar 进程隔离调用，不能静态/动态链接进主程序。详见 `decisions/0001`。

---

## 10. 待决问题（需要用户/后续确认）

| # | 问题 | 当前处理 | 何时决定 |
|---|---|---|---|
| 1 | 项目名 `mood-music-studio` 是否满意 | 暂用此名 | 首次推送前 |
| 2 | 开发环境安装授权 | 暂不安装，先出文档 | 本文档就绪后 |
| 3 | Python 版本是否升级到 3.11+ | 建议，待确认 | 编码开始前 |
| 4 | beets 作为库管底座 vs 自研 | MVP 先自研轻量，beets 作为评估备选 | MVP-0 后 |
| 5 | License 选择（考虑到 AGPL 隔离） | 倾向 MIT，待定 | 首次推送前 |
| 6 | 前端框架 React vs Vue vs Svelte | 暂定 React（生态最大） | GUI 编码前 |

---

## 11. 更新日志

| 日期 | 变更 | 作者 |
|---|---|---|
| 2026-08-06 | 项目初始化，调研完成，架构/路线图/文档体系建立 | ZCode（泽恩） |
