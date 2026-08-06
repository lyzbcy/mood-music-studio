#!/usr/bin/env bash
# Mood Music Studio 一键启动脚本
# 用法：
#   ./start.sh          前台运行（终端关掉应用就退出）
#   ./start.sh --bg     后台运行（关终端不影响）
#   ./start.sh --check  仅检查环境不启动
#
# Finder 双击：双击「启动.command」（它调用本脚本）

set -euo pipefail

# ============ 颜色 ============
red()    { printf "\033[31m%s\033[0m\n" "$*"; }
green()  { printf "\033[32m%s\033[0m\n" "$*"; }
yellow() { printf "\033[33m%s\033[0m\n" "$*"; }
blue()   { printf "\033[36m%s\033[0m\n" "$*"; }

# ============ 路径 ============
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 用户级安装的工具加入 PATH（免 sudo 安装方式，新 shell 默认没有）
export PATH="$HOME/.local/node/bin:$HOME/.cargo/bin:$PATH"

SIDECAR_PORT=45170
SIDECAR_DIR="$SCRIPT_DIR/sidecar"
LOG_DIR="$SCRIPT_DIR/.logs"
mkdir -p "$LOG_DIR"

# ============ 模式解析 ============
MODE="fg"
case "${1:-}" in
  --bg)    MODE="bg" ;;
  --check) MODE="check" ;;
  --help|-h)
    cat <<EOF
Mood Music Studio 启动脚本

用法:
  ./start.sh          前台启动（Ctrl+C 退出）
  ./start.sh --bg     后台启动（关终端不影响）
  ./start.sh --check  仅检查环境

环境:
  Node:    $HOME/.local/node/bin
  Rust:    $HOME/.cargo/bin
  Sidecar: $SIDECAR_DIR (.venv)
EOF
    exit 0 ;;
esac

# ============ 环境检查 ============
blue "🔍 检查环境..."

check() {
  local name="$1" cmd="$2" hint="$3"
  if command -v "$cmd" >/dev/null 2>&1; then
    green "  ✅ $name: $("$cmd" --version 2>&1 | head -1)"
    return 0
  else
    red "  ❌ $name 未找到"
    [ -n "$hint" ] && yellow "     $hint"
    return 1
  fi
}

ENV_OK=1
check "Node"  node  "需安装到 $HOME/.local/node"  || ENV_OK=0
check "npm"   npm   ""                              || ENV_OK=0
check "Rust"  rustc "需 rustup 安装"                || ENV_OK=0
check "Cargo" cargo ""                              || ENV_OK=0

# Python venv
if [ -d "$SIDECAR_DIR/.venv" ]; then
  green "  ✅ Python venv: $SIDECAR_DIR/.venv"
else
  red "  ❌ Python venv 不存在"
  yellow "     请运行：cd sidecar && python3 -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt"
  ENV_OK=0
fi

# Tauri CLI
if command -v cargo-tauri >/dev/null 2>&1; then
  green "  ✅ Tauri CLI: $(cargo tauri --version 2>&1 | head -1)"
else
  red "  ❌ Tauri CLI 未安装"
  yellow "     请运行：cargo install tauri-cli --version \"^2.0\""
  ENV_OK=0
fi

# 前端依赖
if [ -d "$SCRIPT_DIR/ui/node_modules" ]; then
  green "  ✅ 前端依赖: ui/node_modules"
else
  yellow "  ⚠️  前端依赖缺失，正在安装..."
  (cd "$SCRIPT_DIR/ui" && npm install --no-audit --no-fund) || { red "前端依赖安装失败"; ENV_OK=0; }
fi

if [ "$MODE" = "check" ]; then
  echo ""
  [ $ENV_OK -eq 1 ] && green "🎉 环境就绪" || red "💥 环境有问题，按上面提示修复"
  exit $((1 - ENV_OK))
fi

if [ $ENV_OK -ne 1 ]; then
  echo ""
  red "💥 环境检查未通过，无法启动。运行 ./start.sh --check 看详情。"
  exit 1
fi

echo ""

# ============ 启动 sidecar（若没跑）============
sidecar_alive() {
  curl -s --max-time 2 "http://127.0.0.1:$SIDECAR_PORT/health" >/dev/null 2>&1
}

if sidecar_alive; then
  green "🎵 sidecar 已在运行 (端口 $SIDECAR_PORT)"
else
  blue "🚀 启动 sidecar..."
  (
    cd "$SIDECAR_DIR"
    source .venv/bin/activate
    MOOD_PORT=$SIDECAR_PORT MOOD_LOG=INFO python -m app
  ) > "$LOG_DIR/sidecar.log" 2>&1 &
  SIDECAR_PID=$!
  yellow "   pid=$SIDECAR_PID，等待就绪..."

  # 等最多 15 秒
  for i in $(seq 1 30); do
    if sidecar_alive; then
      green "   ✅ sidecar 就绪"
      break
    fi
    sleep 0.5
    [ $i -eq 30 ] && { red "   ❌ sidecar 启动超时，看日志：$LOG_DIR/sidecar.log"; tail -10 "$LOG_DIR/sidecar.log"; exit 1; }
  done
fi

# ============ 启动 Tauri ============
blue "🚀 启动 Tauri 桌面应用..."

if [ "$MODE" = "bg" ]; then
  nohup cargo tauri dev > "$LOG_DIR/tauri.log" 2>&1 &
  TAURI_PID=$!
  echo ""
  green "🎉 已后台启动！"
  echo "   主进程 pid: $TAURI_PID"
  echo "   日志:       $LOG_DIR/tauri.log"
  echo "   停止:       ./stop.sh"
  echo ""
  yellow "应用窗口会在几秒后弹出（首次需编译，约 10-30 秒）"
else
  echo ""
  cargo tauri dev 2>&1 | tee "$LOG_DIR/tauri.log"
fi