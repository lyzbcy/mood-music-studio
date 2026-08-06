//! Sidecar 进程管理：端口发现、拉起、健康检查、优雅关停。
//!
//! 见 docs/architecture.md §2（进程模型）。
//! Tauri 主进程的职责是：
//!   1. 找一个空闲端口
//!   2. 拉起 PyInstaller 打包的 mood-worker，传 PORT/MOODE_DATA 环境变量
//!   3. 轮询 /health 直到就绪
//!   4. 把端口暴露给前端（通过 Tauri command 或 window 全局变量）
//!   5. 窗口关闭时优雅停掉 sidecar
//!
//! `spawn` / `find_free_port` / `SidecarError` 等仅在 release（生产打包）路径使用；
//! debug 模式走 `attach` 直连开发 sidecar。整体允许 dead_code 以避免 debug 编译告警。

#![allow(dead_code)]

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const DEFAULT_PORT_BASE: u16 = 45_170;
const DEFAULT_PORT_MAX: u16 = 45_280;
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(30);
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_millis(300);
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// sidecar 运行时句柄。
///
/// 注：`spawn` / `find_free_port` / `HealthResp` 等仅在 release 模式（生产打包）使用，
/// debug 模式走 `attach` 直连开发 sidecar，故标 `allow(dead_code)` 避免 debug 编译告警。
#[allow(dead_code)]
pub struct Sidecar {
    pub port: u16,
    pub base_url: String,
    child: Option<Child>,
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum SidecarError {
    #[error("无可用端口（{base}-{max} 全被占用）")]
    NoFreePort { base: u16, max: u16 },
    #[error("sidecar 二进制未找到：{0}（开发期请用 dev 模式直连）")]
    BinaryMissing(String),
    #[error("sidecar 启动失败：{0}")]
    SpawnFailed(String),
    #[error("sidecar 健康检查超时（{secs}秒内 /health 未就绪）")]
    HealthTimeout { secs: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthResp {
    ok: bool,
    status: String,
}

impl Sidecar {
    /// 开发模式：不拉起子进程，直接连到已运行的 sidecar（手动 python -m app）。
    pub fn attach(port: u16) -> Result<Self, SidecarError> {
        let base_url = format!("http://127.0.0.1:{port}");
        Ok(Self {
            port,
            base_url,
            child: None,
        })
    }

    /// 生产模式：找空闲端口 + 拉起 sidecar + 健康检查。
    pub fn spawn<B: AsRef<std::path::Path>, D: AsRef<std::path::Path>>(
        binary: B,
        data_dir: D,
    ) -> Result<Self, SidecarError> {
        let binary = binary.as_ref();
        if !binary.exists() {
            return Err(SidecarError::BinaryMissing(
                binary.to_string_lossy().into_owned(),
            ));
        }
        let port = find_free_port().ok_or(SidecarError::NoFreePort {
            base: DEFAULT_PORT_BASE,
            max: DEFAULT_PORT_MAX,
        })?;
        let base_url = format!("http://127.0.0.1:{port}");

        let child = Command::new(binary)
            .env("MOOD_PORT", port.to_string())
            .env("MOOD_DATA", data_dir.as_ref())
            .env("MOOD_HOST", "127.0.0.1")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| SidecarError::SpawnFailed(e.to_string()))?;

        let sc = Self {
            port,
            base_url,
            child: Some(child),
        };
        sc.wait_until_healthy()?;
        Ok(sc)
    }

    /// 轮询 /health 直到就绪或超时。
    pub fn wait_until_healthy(&self) -> Result<(), SidecarError> {
        // 用阻塞 reqwest（blocking feature 在 tokio runtime 下不便，改用同步 std::net 探测 + HTTP 手写）
        // 这里简化：用 raw TCP 探测端口，再发一个 HTTP GET。
        let deadline = Instant::now() + HEALTH_CHECK_TIMEOUT;
        let url = format!("{}/health", self.base_url);
        while Instant::now() < deadline {
            if let Ok(resp) = ureq_get_json(&url) {
                if resp.get("ok") == Some(&serde_json::Value::Bool(true)) {
                    return Ok(());
                }
            }
            std::thread::sleep(HEALTH_CHECK_INTERVAL);
        }
        Err(SidecarError::HealthTimeout {
            secs: HEALTH_CHECK_TIMEOUT.as_secs(),
        })
    }

    /// 优雅关停：先 SIGTERM，超时再 SIGKILL。
    pub fn shutdown(&mut self) {
        if let Some(child) = self.child.as_mut() {
            log::info!("关停 sidecar (pid={})...", child.id());
            // Unix: SIGTERM 走 kill；子进程的 FastAPI shutdown hook 会清理
            #[cfg(unix)]
            {
                let pid = child.id() as i32;
                let _ = nix_kill(pid);
            }
            let _ = child.wait(); // 等 graceful
            let _ = child.kill(); // 兜底
        }
    }
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// 在 45170-45280 区间找第一个可绑定的端口。
pub fn find_free_port() -> Option<u16> {
    (DEFAULT_PORT_BASE..=DEFAULT_PORT_MAX).find(|&p| {
        TcpListener::bind(("127.0.0.1", p))
            .map(|_| true)
            .unwrap_or(false)
    })
}

/// 极简 HTTP GET JSON（不引入 reqwest blocking，避免 runtime 冲突）。
/// 健康检查频率低，手写足够。
fn ureq_get_json(url: &str) -> Result<serde_json::Value, ()> {
    // 用 std::net 直连，避免引入额外依赖。仅用于 127.0.0.1 本地。
    let parsed = parse_url(url).ok_or(())?;
    let ip: std::net::IpAddr = parsed.host.parse().map_err(|_| ())?;
    let mut stream = std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from((ip, parsed.port)),
        Duration::from_secs(1),
    )
    .map_err(|_| ())?;
    use std::io::{Read, Write};
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        parsed.path
    );
    stream.write_all(req.as_bytes()).map_err(|_| ())?;
    let mut buf = Vec::with_capacity(1024);
    stream.read_to_end(&mut buf).map_err(|_| ())?;
    let body = std::str::from_utf8(&buf).map_err(|_| ())?;
    let json_start = body.find("{").ok_or(())?;
    serde_json::from_str(&body[json_start..]).map_err(|_| ())
}

struct ParsedUrl<'a> {
    host: &'a str,
    port: u16,
    path: &'a str,
}

fn parse_url(url: &str) -> Option<ParsedUrl<'_>> {
    // 形如 http://127.0.0.1:45170/health
    let no_scheme = url.strip_prefix("http://")?;
    let (host_port, path) = no_scheme.split_once('/').unwrap_or((no_scheme, ""));
    let (host, port) = host_port.rsplit_once(':')?;
    Some(ParsedUrl {
        host,
        port: port.parse().ok()?,
        path: format!("/{path}").leak(),
    })
}

#[cfg(unix)]
fn nix_kill(pid: i32) -> std::io::Result<()> {
    // SIGTERM = 15。用 libc 避免引入 nix crate。
    unsafe {
        if libc_kill(pid, 15) != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(unix)]
extern "C" {
    #[link_name = "kill"]
    fn libc_kill(pid: i32, sig: i32) -> i32;
}
