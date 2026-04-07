#!/bin/bash
# Toki 区块链部署脚本
# 用于在服务器上自动部署和启动节点

set -e

# 配置变量
SERVER_IP="182.254.176.30"
INSTANCE_ID="lhins-i50x"
DATA_DIR="./data"
LOG_DIR="./logs"

echo "=========================================="
echo "  Toki 区块链部署脚本"
echo "=========================================="
echo ""
echo "服务器信息:"
echo "  IP: $SERVER_IP"
echo "  实例 ID: $INSTANCE_ID"
echo ""

# 检查 Rust 环境
echo "检查 Rust 环境..."
if ! command -v cargo &> /dev/null; then
    echo "Rust 未安装，正在安装..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
else
    echo "Rust 已安装: $(cargo --version)"
fi

# 创建必要的目录
echo "创建目录..."
mkdir -p "$DATA_DIR"
mkdir -p "$LOG_DIR"

# 编译项目
echo "编译项目..."
cargo build --release

# 检查启动配置
echo "检查启动配置..."
if [ -f "config_bootstrap.toml" ]; then
    echo "找到启动配置，将执行自启动流程"
    STARTUP_MODE="bootstrap"
else
    echo "未找到启动配置，使用默认配置"
    STARTUP_MODE="normal"
fi

# 启动节点
echo ""
echo "启动节点..."
echo "启动模式: $STARTUP_MODE"
echo ""

# 后台启动
nohup cargo run --release --bin toki-node > "$LOG_DIR/toki-node.log" 2>&1 &

NODE_PID=$!
echo "节点已启动，PID: $NODE_PID"
echo "日志文件: $LOG_DIR/toki-node.log"

# 等待节点启动
echo "等待节点启动..."
sleep 10

# 检查节点状态
echo "检查节点状态..."
if curl -s http://localhost:8080/health > /dev/null; then
    echo "✅ 节点启动成功"
    echo ""
    echo "访问地址:"
    echo "  - 健康检查: http://$SERVER_IP:8080/health"
    echo "  - 节点信息: http://$SERVER_IP:8080/api/v1/node/info"
    echo "  - 最新区块: http://$SERVER_IP:8080/api/v1/block/latest"
    echo ""
    echo "P2P 地址:"
    echo "  - /ip4/$SERVER_IP/tcp/30333"
else
    echo "❌ 节点启动失败，请检查日志"
    echo "日志: tail -f $LOG_DIR/toki-node.log"
    exit 1
fi

# 保存 PID
echo $NODE_PID > "$DATA_DIR/toki-node.pid"

echo ""
echo "=========================================="
echo "  部署完成"
echo "=========================================="
echo ""
echo "管理命令:"
echo "  查看日志: tail -f $LOG_DIR/toki-node.log"
echo "  停止节点: kill $NODE_PID"
echo "  重启节点: ./deploy.sh"
echo ""
