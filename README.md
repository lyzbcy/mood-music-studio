# 🎵 Mood Music Studio

> 音乐自动打标签 + 按情绪/场景智能分类管理 + AI 分镜配乐的跨平台桌面软件（Win + Mac）
>
> 解决创作者**找音乐难**的痛点：给每首音乐打上情绪/场景标签，用分镜提示词一搜，智能匹配最适合的音乐和音效。

[![Status](https://img.shields.io/badge/status-planning-yellow)](docs/roadmap.md)
[![License](https://img.shields.io/badge/license-MIT%20(draft)-blue)](LICENSE)

---

## ✨ 核心能力

| 能力 | 说明 |
|---|---|
| **自动打标签** | 导入音乐 → Essentia 自动识别情绪/场景/genre/BPM，56 个情绪标签 + valence-arousal 二维情绪坐标 |
| **智能检索配乐** | 输入分镜提示词（如「雨夜都市，主角独行，忧伤悬疑」）→ LAION CLAP 语义检索，返回最匹配的音乐片段 |
| **音乐来源管理** | 本地库（MVP 重点）→ 云端同步（Cloudflare R2，后做）→ 剪映音乐 ID 显示（兜底） |
| **AI 接管** | 内置 MCP Server，Claude/GPT/Cursor 可直接接管，自动化配乐工作流 |

---

## 🏗️ 架构

Tauri 2（Rust 主进程）+ Python sidecar（FastAPI + Essentia + CLAP）+ WebView 前端。

```
前端 (React) → Tauri Rust 主进程 → Python sidecar (FastAPI + MCP)
                                     ├─ Essentia 打标
                                     ├─ LAION CLAP 检索
                                     └─ LanceDB 向量库
```

详细架构：[`docs/architecture.md`](docs/architecture.md)

---

## 📦 当前状态：规划与文档阶段 🟡

项目刚启动，调研与设计已完成，代码尚未开始。

- ✅ 三路技术调研完成（音频引擎 / 桌面框架 / 存储+AI 配乐）
- ✅ 架构设计与技术选型确定
- ✅ MVP 三路并行路线图
- ✅ 完整文档体系（`docs/`）
- ⬜ 开发环境搭建中（Node / Rust / gh 待装）
- ⬜ 代码骨架

下一步详见 [`docs/roadmap.md`](docs/roadmap.md)。

---

## 📚 文档导航

> 采用**渐进式文档**：AI 接手时只读负责模块的文档，不必全读。

| 文档 | 内容 |
|---|---|
| [docs/agent.md](docs/agent.md) | **项目全貌 + AI 接手指南**（入口文档，先读这个） |
| [docs/architecture.md](docs/architecture.md) | 系统架构、数据流、接口设计 |
| [docs/roadmap.md](docs/roadmap.md) | MVP 路线、任务拆解、进度 |
| [docs/decisions/](docs/decisions/) | 架构决策记录（ADR） |
| [docs/research/](docs/research/) | 三份技术调研报告 |
| [docs/modules/](docs/modules/) | 六个模块的独立文档 |

---

## 🚀 快速开始（待环境就绪）

```bash
# 前置：Node.js 20+, Rust, Python 3.11+

# 克隆
git clone https://github.com/<user>/mood-music-studio.git
cd mood-music-studio

# 前端依赖
npm install

# Python sidecar 依赖
cd sidecar && pip install -r requirements.txt && cd ..

# 开发模式（同时起 Tauri + sidecar）
npm run tauri dev
```

> ⚠️ 详细开发指南在编码阶段补充。

---

## 🧰 技术栈

| 层 | 选型 |
|---|---|
| 桌面壳 | Tauri 2 |
| 前端 | React + TypeScript |
| 后端 | Python 3.11+ / FastAPI |
| 打标引擎 | Essentia（AGPL，sidecar 隔离） |
| 语义检索 | LAION CLAP |
| 向量库 | LanceDB |
| 元数据 | SQLite |
| AI 接管 | MCP (FastMCP) |
| 云存储（后做） | Cloudflare R2 |

选型理由见 [`docs/decisions/0001-initial-tech-stack.md`](docs/decisions/0001-initial-tech-stack.md)。

---

## 📄 License

MIT（草案，正式发布前确认 Essentia AGPL 隔离合规后定稿）。

---

## 🤝 贡献

个人项目，暂不接受 PR。如你是 AI agent 接手开发，请先读 [`docs/agent.md`](docs/agent.md)。
