#!/usr/bin/env bash
# Finder 双击此文件即启动应用（macOS 会用 Terminal 打开 .command）
# 本文件只是 start.sh 的薄封装，便于在 Finder 里双击

# 切到脚本所在目录（Finder 双击时 cwd 是 $HOME）
cd "$(dirname "$0")" || exit 1

echo "════════════════════════════════════════"
echo "  🎵 Mood Music Studio 启动"
echo "════════════════════════════════════════"
echo ""
echo "按 Ctrl+C 退出应用。关掉这个终端窗口也会退出。"
echo "若要后台运行（关终端不退出），用：./start.sh --bg"
echo ""

./start.sh

# 应用退出后保持窗口，让用户能看到日志
echo ""
read -n 1 -s -r -p "应用已退出。按任意键关闭窗口..."
