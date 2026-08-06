# 调研报告 02：跨平台桌面框架选型

> **元数据**
> - 调研时间：2026-08-06
> - 调研者：ZCode subagent（用 WebSearch/WebFetch 检索最新 2024-2026 信息）
> - 用途：为「桌面 GUI + Python sidecar」架构选型提供依据
> - 结论：已写入 ADR-0001 决策 1，选用 **Tauri 2 + Python sidecar**

---

## 核心维度对比表

| 维度 | **Tauri 2 (Rust + WebView)** | **Electron (Chromium + Node)** | **Flutter Desktop (Dart)** | **Wails (Go + WebView)** | **PyQt/PySide6 (Python + Qt)** |
|---|---|---|---|---|---|
| 安装包体积 | **3-9 MB**（不含 Python）；含 PyInstaller sidecar 约 30-80MB | 85-250 MB（Hello World 85MB） | 15-25 MB | ~52 MB | 30-80 MB（含 Qt 运行时） |
| 空闲内存 | ~42 MB | ~168 MB | 较低 | 较低 | 中等 |
| 冷启动 | ~380ms | ~1420ms（慢 3.7×） | 快 | 快 | 中 |
| UI 技术 | Web（系统 WebView：WebView2/WKWebView） | Web（自带 Chromium） | 自绘 Skia/Impeller | Web（系统 WebView） | Qt Widgets / QML |
| 后端语言 | **Rust** | **Node.js (JS)** | **Dart** | **Go** | **Python** |
| 一次开发 Win+Mac | 支持，需各自平台构建（CI 推荐） | 支持，跨平台编译成熟 | 支持 | 支持 | 支持（需各自平台跑 PyInstaller） |
| 调用 Python 难度 | 中（sidecar: externalBin + Command） | **低**（child_process / python-shell，最顺手） | 中（FFI 或 sidecar） | 中（os/exec sidecar） | **零**（Python 原生） |
| 内嵌本地 HTTP API | 成熟（tauri-plugin-axum / warp） | 成熟（Express/Fastify） | 可行（shelf/dart:io） | 可行（net/http 原生） | 可行（FastAPI/Flask） |
| 音频特征库 | 依赖 Python sidecar 或 Rust 重写 | 依赖 Python sidecar | 无 librosa 对应物，需 FFI 或 sidecar | 依赖 Python sidecar | **librosa/essentia 原生可用** |
| 生态成熟度 | 快速增长，Tauri 2.0 已稳定（2024 发布） | **最成熟**，VSCode/Slack/Discord | 移动优先，桌面 Canonical 主导 | 中等，Go 社区 | 成熟，但 UI 现代 Web 表现力弱 |
| 主要坑 | Rust 学习曲线；WebView 跨平台不一致；sidecar 需 target-triple 命名 | 包大、内存高；python-shell 在打包后路径问题 | Google 桌面战略投入降低；音频生态薄 | 生态小，需自己接线 | 现代化 UI 弱；分发复杂 |

---

## 关键问题逐项回答

### Q1：核心要调用 Python 音频库（librosa/essentia），最顺的架构是什么？

**结论：把 Python 打包成 sidecar 子进程（PyInstaller/Nuitka），外面套任意 Web 框架（推荐 Tauri 或 Electron）。只有当核心竞争力不在音频算法、且 UI 极简时，才考虑纯 PySide。**

四种路线取舍：

| 路线 | 顺心度 | 体积 | 维护 | 适用场景 |
|---|---|---|---|---|
| **A. Python 作为 sidecar（推荐）** | ★★★★★ | 中（+30~80MB） | 低，保留 librosa/essentia 全部能力 | 音频特征是核心、要快速迭代算法 |
| B. Rust 重写音频算法 | ★★ | 最小 | 高（librosa 算法没有 Rust 等价物，essentia 有 C++ 可 FFI） | 极致体积/性能、音频功能简单 |
| C. PySide/PyQt 纯 Python | ★★★★ | 大 | 中 | UI 简单、团队是 Python 重度、不在意 Web UI |
| D. Flutter + FFI 重写 | ★★ | 小 | 高 | 已有 Flutter 团队 |

**为什么推荐 sidecar**：
- librosa/essentia 都有成熟 PyPI wheel（essentia 在 macOS 还可 Homebrew，PyPI 覆盖 Win/Mac/Linux），`pip install` 即用，**无需重写算法**。
- essentia 本身是 **C++ 库 + Python 绑定**，还内置 TensorFlow 模型集成（BPM、调性、流派、情绪检测预训练模型一应俱全）——离开 Python 就得自己用 C++ FFI 重新拼，代价巨大。
- PyInstaller 打包后的 Python exe 可被任何宿主（Tauri/Electron/Wails）作为独立二进制嵌入。

**sidecar 通信建议**：不要走 stdin/stdout 流式（适合简单场景），让 Python sidecar 自带一个**本地 HTTP server（FastAPI/Flask，127.0.0.1:端口）**，宿主前端、外部 AI 都直接 HTTP 调用——这样「本地 API 供 AI 接管」和「内部调用音频分析」用同一套接口，架构最干净。

### Q2：哪个方案「一次开发、Win+Mac 都能打包」最省心？

**省心度排序：Electron ≥ Tauri > Wails > PySide > Flutter（音频场景）**

注意：**没有任何方案能真正在一台机器上交叉出带代码签名的 Mac 安装包**——Mac 公证(notarization)和 Apple 签名必须在 macOS 上跑。所以「省心」的真正含义是 **CI 一键产出双平台**，推荐用 GitHub Actions 矩阵（macos-latest + windows-latest）。

各方案打包命令示例：

**Electron（electron-builder）——最省心**
```bash
# package.json 配置好 build 字段后：
# 在 macOS 上同时打 mac + windows（Windows 走 Wine）
npx electron-builder --mac --win

# 推荐：GitHub Actions 分别在两个 runner 上构建
# macos-latest 上:  npx electron-builder --mac
# windows-latest上: npx electron-builder --win
```
配置示例：
```json
"build": {
  "appId": "com.you.app",
  "mac":   { "target": ["dmg", "zip"], "category": "public.app-category.music" },
  "win":   { "target": ["nsis"] },
  "directories": { "output": "release" }
}
```

**Tauri 2**
```bash
# 各自平台原生构建（不能交叉，必须 CI 矩阵）
npm run tauri build         # 当前平台
# GitHub Actions:
# macos-latest:   npm run tauri build -- --target universal-apple-darwin
# windows-latest: npm run tauri build
# 产物：.dmg/.app 和 .msi/.exe
```
sidecar 必须按 target-triple 命名：`src-tauri/binaries/audio-worker-x86_64-pc-windows-msvc.exe` 和 `audio-worker-aarch64-apple-darwin`，可用构建脚本自动改名。

**PySide/PyQt（PyInstaller 或 fbs）**
```bash
pyinstaller --windowed --onefile main.py            # Win 产出 .exe, Mac 产出 .app
# 或用 fbs（封装 PyInstaller，自带安装器生成）：
fbs freeze && fbs installer                          # 各平台分别执行
```

**Wails**
```bash
wails build -platform windows/darwin                 # 当前平台能交叉时
# 实际仍推荐 CI 矩阵分别构建
```

### Q3：对外开放本地 HTTP API（供 AI 接管）在每种架构怎么做？

通用模式：**绑定 127.0.0.1（不要 0.0.0.0），监听一个固定/动态端口，可选暴露给云端时用 ngrok/cloudflare tunnel。**

| 架构 | 实现方式 |
|---|---|
| **Tauri** | 内嵌 axum（`tauri-plugin-axum`）或 warp，在 setup hook 里 `tokio::spawn` 绑定 127.0.0.1:port。生产案例：Tauri 2 + Svelte + axum 全 gateway 架构。若想让 AI 直接复用，Python sidecar 自己起 FastAPI 即可。 |
| **Electron** | 主进程 `require('express')` 或 `fastify`，监听 127.0.0.1。**Jan AI** 桌面版就是这么做的（`JAN API listening at http://127.0.0.1:1337`），现成参考。 |
| **Flutter** | `dart:io` 起 `HttpServer`，或用 shelf 包。 |
| **Wails** | Go 原生 `net/http`，最自然。 |
| **PySide** | FastAPI/Flask 后台线程起 127.0.0.1 server。 |

**重要提醒**：云端 AI（如 ChatGPT 调用 MCP）**无法直接访问电脑的 localhost**，需要 ngrok/localtunnel/cloudflare tunnel 做穿透；若是本地运行的 AI（Claude Desktop、Cursor、本地 MCP host），则直接连 127.0.0.1 即可。建议把对外 API 实现成 **MCP server（HTTP/SSE transport）**，AI 接管最顺手。

---

## 各方案的具体坑位

**Tauri**
- Rust 学习曲线；老 Windows 7/8 没有 WebView2 需引导用户安装（Win10+ 自带）。
- sidecar 必须按 target-triple 后缀命名，arm64/x64 要分别提供二进制。
- WebView 跨平台渲染细节有差异（Safari/Edge 版本差异仍在）。

**Electron**
- 包大、内存高（典型 150-400MB）。
- python-shell 在 asar 打包后路径会失效，生产环境要用 `process.resourcesPath` 解析 sidecar 路径，社区踩坑很多。
- macOS 公证 + Windows 证书门槛：Apple Developer 99 美元/年 + Windows EV 证书（几百美元），CI 要塞 CSC_LINK / APPLE_ID / WIN_CSC_LINK 等密钥。

**Flutter Desktop**
- 2025 年 Google 战略重心是移动/Web，桌面主要靠 Canonical 推动，长期投入有疑虑。
- 没有 librosa 的 Dart 等价物，做 melspectrogram 等特征必须 FFI 调 C++ 或起 Python sidecar——纯 Flutter 路径反而绕。

**PySide/PyQt**
- UI 表现力弱于现代 Web。
- 分发链路（PyInstaller 隐藏依赖、Qt 插件、essentia 原生库）在 Win+Mac 上各有坑。

**Wails**
- 生态最小，文档/案例少；适合 Go 重度团队，对音频场景无明显优势。

---

## 最终推荐

### 主推：Tauri 2 + Python sidecar（FastAPI）

```
前端 (React/Vue/Svelte) — 系统WebView
        │ Tauri IPC / 直接 HTTP
Tauri Rust 主进程
  - axum 本地 HTTP API (127.0.0.1)   ← 外部 AI 接管入口
  - 进程管理：拉起/守护 Python sidecar
        │ 持久本地 HTTP（FastAPI on 127.0.0.1）
Python sidecar (PyInstaller 打包)
  - librosa / essentia 音频特征
  - 本地音频文件管理、DB
  - 调用云端 API（OpenAI 等）
```

**理由**：
1. **音频生态完整保留**：librosa/essentia（含 TensorFlow 模型）原生可用，零重写成本，权重最高。
2. **体积可控**：Tauri 本体仅几 MB，最终包大小主要由 Python sidecar 决定（30-80MB），比 Electron 的 200MB+ 轻一大截。
3. **本地 API 架构统一**：Python sidecar 自己就是 FastAPI server，同时服务前端、外部 AI（MCP 接管）、云端回连——一个接口三处复用。
4. **Win+Mac 双平台**：Tauri 官方支持，CI 矩阵构建 + 签名/公证方案成熟。

### 备选
- **团队 Go 强、Rust 不熟**：选 **Wails + Python sidecar**。
- **团队极度偏好 JS、不想碰 Rust**：选 **Electron + Python sidecar**，代价是包大内存高，但生态最熟（Jan AI 现成参考）。
- **UI 需求极简、团队就是 Python 栈、想最快出活**：选 **PySide6 + fbs**。

### 不推荐
- **Flutter Desktop**：音频生态空白，Google 桌面战略投入下降。
- **Rust 重写音频算法**：除非音频功能很简单，否则重写 librosa/essentia 等价物成本远高于 sidecar 体积代价。
- **.NET MAUI / Qt+Python 复杂组合**：MAUI 桌面成熟度不足；Qt+Python 和 PySide 同类但更复杂。

---

## 关键数据来源

- [Tauri 2.0 Sidecar 官方文档](https://v2.tauri.app/develop/sidecar/)
- [Tauri vs Electron 2026 基准](https://tech-insider.org/tauri-vs-electron-2026/)
- [tauri-plugin-axum 文档](https://docs.rs/tauri-plugin-axum)
- [Tauri 2 + Svelte + axum 生产案例](https://www.reddit.com/r/tauri/comments/1s4ah2f/shipped_a_tauri_2_svelte_5_desktop_app_with_a/)
- [electron-builder 官方](https://www.electron.build/)
- [Jan AI 本地 API server（Electron 实践参考）](https://www.jan.ai/docs/desktop/api-server)
- [essentia 安装指南](https://essentia.upf.edu/installing.html)
- [essentia 预训练模型](https://essentia.upf.edu/models.html)
- [PyInstaller 官方](https://www.pyinstaller.org/)
- [librosa PyInstaller issue（已知坑）](https://github.com/librosa/librosa/issues/538)
- [Wails vs Tauri vs Electron 对比](https://www.digitalapplied.com/blog/desktop-apps-web-stack-tauri-electron-deno-wails-2026)
- [本地 MCP server 接 Claude.ai 实践](https://www.localcan.com/blog/test-local-mcp-server-in-claude-ai)
