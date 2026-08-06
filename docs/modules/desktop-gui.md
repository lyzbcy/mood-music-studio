# 模块：桌面 GUI（desktop-gui）

> **负责**：用户界面、交互、跨平台窗口管理
> **所属路线**：横跨三线（前端层）
> **依赖**：Tauri 2（主进程壳）、React + TypeScript（前端）、系统 WebView
> **对接**：通过 HTTP 调用 Python sidecar 所有 API
> **调研依据**：`research/02-desktop-frameworks.md`

---

## 当前状态：⬜ 未开始

| 子任务 | 状态 | 说明 |
|---|---|---|
| Tauri 工程初始化 | ⬜ | `npm create tauri-app` |
| sidecar 拉起逻辑 | ⬜ | Rust 主进程核心 |
| 前端框架确认 | ⬜ | React（暂定，待评估 Svelte） |
| 主界面布局 | ⬜ | 三栏：库/筛选、列表、详情 |
| 打标导入流程 UI | ⬜ | 线 A |
| 配乐检索面板 UI | ⬜ | 线 B |
| AI 接入 + 剪映 UI | ⬜ | 线 C |
| 双平台打包脚本 | ⬜ | GitHub Actions |

---

## 1. 职责边界

**做什么**：
- Tauri 主进程：拉起/守护 Python sidecar、窗口管理、托盘、系统对话框
- 前端 WebView：所有 UI，通过 fetch 调 sidecar 的 127.0.0.1 API
- 打包配置：Win (.msi/.exe) + Mac (.dmg/.app)

**不做什么**：
- 任何业务逻辑（全在 Python sidecar）
- 直接调 Essentia/CLAP（走 HTTP）

---

## 2. Tauri 主进程职责

```rust
// src-tauri/src/main.rs（伪代码要点）
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 1. 找空闲端口
            let port = find_free_port(45170);
            // 2. 拉起 sidecar
            let sidecar = app.shell().sidecar("mood-worker")?;
            let (mut rx, child) = sidecar.spawn()?;
            // 3. 健康检查（轮询 /health）
            wait_for_healthy(port);
            // 4. 把端口注入前端
            app.manage(AppState { port, child });
            Ok(())
        })
        .on_window_event(|window, event| {
            // 关窗时优雅停 sidecar
            if let WindowEvent::CloseRequested { .. } = event {
                shutdown_sidecar(window.app_handle());
            }
        })
        .run(tauri::generate_context!())
        .expect("error");
}
```

### sidecar target-triple 命名（重要）
```
src-tauri/binaries/
├── mood-worker-aarch64-apple-darwin        # Mac Apple Silicon
├── mood-worker-x86_64-apple-darwin         # Mac Intel
└── mood-worker-x86_64-pc-windows-msvc.exe  # Windows
```
Tauri 按当前 OS + arch 自动选对应二进制。

---

## 3. 前端架构

### 框架选择
- **React + TypeScript**（暂定）：生态最大，组件库多
- 备选 Svelte：体积小、Tauri 社区案例多，待评估

### 三栏布局（主界面）
```
┌─────────────┬───────────────────────────┬──────────────────┐
│ 左栏：筛选   │ 中栏：音乐列表              │ 右栏：详情/播放    │
│             │                           │                  │
│ 情绪         │ ▸ Track A  happy, dark   │ Track A          │
│ ▸ happy (12)│ ▸ Track B  sad           │ ─────────────    │
│ ▸ sad (8)   │ ▸ Track C  energetic     │ [波形 + 播放]     │
│ ▸ dark (15) │ ▸ ...                    │                  │
│             │                           │ 标签：           │
│ 场景         │ [导入新音乐] [批量打标]    │ #happy #cinematic│
│ ▸ cinematic │                           │                  │
│ ▸ relaxing  │ ─── AI 配乐面板 ───       │ 来源：本地        │
│             │ [提示词输入框]            │ 剪映 ID: C_7_xxx │
│ BPM 80-120  │ [搜索]                   │                  │
│ ─────       │ → 匹配结果片段列表        │ [AI 接入设置]     │
│ AI 接入     │                           │                  │
└─────────────┴───────────────────────────┴──────────────────┘
```

### 路由
- `/` 库浏览
- `/import` 导入向导
- `/scoring` AI 配乐工作台
- `/settings` 设置（含 AI 接入、存储后端）

---

## 4. 打包

### package.json（前端）
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri"
  }
}
```

### src-tauri/tauri.conf.json（要点）
```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "msi", "nsis"],
    "externalBin": ["binaries/mood-worker"],
    "macOS": { "signingIdentity": null },
    "windows": { "wix": { "language": "zh-CN" } }
  }
}
```

### GitHub Actions 矩阵
见 `architecture.md` §7.1。

---

## 5. 待决项
- [ ] 前端框架最终确认（React vs Svelte）
- [ ] 组件库选型（shadcn/ui？Ant Design？）
- [ ] 波形可视化库（wavesurfer.js）
- [ ] 暗色主题
- [ ] 国际化（中/英）
- [ ] 自动更新（tbuffer-plugin-updater）
