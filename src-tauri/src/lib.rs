//! Mood Music Studio — Tauri 主进程入口（库形态）。
//!
//! 职责：拉起/守护 Python sidecar，注入端口给前端，管理窗口生命周期。
//! 见 docs/architecture.md §2、docs/modules/desktop-gui.md §2。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// release 专用函数（sidecar_binary_path 等）在 debug 下未用，允许
#![allow(dead_code)]

mod sidecar;

use std::sync::Arc;

#[cfg(not(debug_assertions))]
use std::path::PathBuf;

use serde::Serialize;

use sidecar::Sidecar;

/// 暴露给前端的状态：sidecar 的访问地址。
#[derive(Serialize, Clone)]
struct SidecarInfo {
    base_url: String,
    port: u16,
}

/// 前端通过 `invoke("get_sidecar_info")` 获取 sidecar 地址。
#[tauri::command]
fn get_sidecar_info(state: tauri::State<'_, Arc<SidecarInfo>>) -> SidecarInfo {
    // state.inner() 是 &Arc<SidecarInfo>；解引用到 SidecarInfo 再 clone（SidecarInfo impl了 Clone）
    (**state.inner()).clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("Mood Music Studio 主进程启动");

    let sidecar = launch_sidecar();
    let info = Arc::new(SidecarInfo {
        base_url: sidecar.base_url.clone(),
        port: sidecar.port,
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(sidecar)
        .manage(info)
        .invoke_handler(tauri::generate_handler![get_sidecar_info])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                log::info!("窗口销毁：{}", window.title().unwrap_or_default());
            }
        })
        .run(tauri::generate_context!())
        .expect("Tauri 启动失败");
}

/// 决定 sidecar 启动方式：
/// - debug：attach 到手动启动的 `python -m app`（端口固定 45170）
/// - release：拉起 PyInstaller 打包的二进制
fn launch_sidecar() -> Sidecar {
    #[cfg(debug_assertions)]
    {
        log::info!("开发模式：attach 到手动启动的 sidecar (端口 45170)");
        let sc = Sidecar::attach(45_170).expect("attach 失败");
        if let Err(e) = sc.wait_until_healthy() {
            log::error!(
                "❌ sidecar 未就绪：{e}。请先运行：cd sidecar && MOOD_PORT=45170 python -m app"
            );
        }
        sc
    }

    #[cfg(not(debug_assertions))]
    {
        let binary = sidecar_binary_path();
        let data_dir = data_dir();
        log::info!("生产模式：拉起 sidecar {}", binary.display());
        match Sidecar::spawn(&binary, &data_dir) {  // AsRef<Path> 接受 &PathBuf/&Path/&String
            Ok(sc) => {
                log::info!("✅ sidecar 就绪 @ {}", sc.base_url);
                sc
            }
            Err(e) => panic!("sidecar 启动失败：{e}"),
        }
    }
}

/// sidecar 二进制路径：与可执行文件同目录的 binaries/ 下，按 target-triple 命名。
#[cfg(not(debug_assertions))]
fn sidecar_binary_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("binaries").join(sidecar_binary_name())
}

#[cfg(not(debug_assertions))]
fn sidecar_binary_name() -> String {
    let arch = std::env::consts::ARCH;
    let os_triplet = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "windows" => "pc-windows-msvc",
        "linux" => "unknown-linux-gnu",
        other => other,
    };
    let ext = if cfg!(windows) { ".exe" } else { "" };
    format!("mood-worker-{arch}-{os_triplet}{ext}")
}

/// 应用数据目录。
#[cfg(not(debug_assertions))]
fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("MOOD_DATA") {
        return PathBuf::from(d);
    }
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Library/Application Support/mood-music-studio")
}
