#!/usr/bin/env bash
# 停止所有 Mood Music Studio 进程
set -uo pipefail

green() { printf "\033[32m%s\033[0m\n" "$*"; }
yellow() { printf "\033[33m%s\033[0m\n" "$*"; }

KILLED=0

# 停 Tauri / GUI 二进制
for pat in "cargo tauri dev" "target/debug/mood-music-studio" "target/release/mood-music-studio" "vite"; do
  if pids=$(pgrep -f "$pat" 2>/dev/null); then
    for pid in $pids; do
      kill "$pid" 2>/dev/null && { green "✅ 停止 $pat (pid $pid)"; KILLED=1; }
    done
  fi
done

# 停 sidecar（端口 45170 上的 python -m app）
if pids=$(pgrep -f "python -m app" 2>/dev/null); then
  for pid in $pids; do
    kill "$pid" 2>/dev/null && { green "✅ 停止 sidecar (pid $pid)"; KILLED=1; }
  done
fi

if [ $KILLED -eq 0 ]; then
  yellow "没有正在运行的 Mood Music Studio 进程"
fi