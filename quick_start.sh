#!/bin/bash

# ============================================================================
# 快速启动脚本（假设已编译）
# ============================================================================

echo "快速启动 Toki 节点..."
cd /home/uploader/tokipt

# 停止旧进程
pkill -f "toki-node" || true
sleep 2

# 启动节点
mkdir -p logs data
nohup ./target/release/toki-node > logs/toki.log 2>&1 &
PID=$!

sleep 3

# 验证
if ps -p $PID > /dev/null; then
    echo "✅ 节点启动成功 (PID: $PID)"
    echo $PID > toki.pid
    echo ""
    echo "查看日志: tail -f logs/toki.log"
    echo "API测试: curl http://localhost:8080/health"
else
    echo "❌ 启动失败，查看日志:"
    tail -20 logs/toki.log
fi
