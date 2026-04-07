#!/bin/bash

# ============================================================================
# Toki 节点部署和启动脚本
# ============================================================================

set -e  # 遇到错误立即退出

echo "=========================================="
echo "Toki 节点部署和启动"
echo "=========================================="
echo ""

# 项目目录
PROJECT_DIR="/home/uploader/tokipt"
cd $PROJECT_DIR

echo "当前目录: $(pwd)"
echo ""

# 1. 检查项目文件
echo "1. 检查项目文件..."
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 错误: Cargo.toml 不存在"
    exit 1
fi

if [ ! -d "core" ] || [ ! -d "consensus" ] || [ ! -d "api" ]; then
    echo "❌ 错误: 项目结构不完整"
    exit 1
fi

echo "✅ 项目文件检查通过"
echo ""

# 2. 检查Rust环境
echo "2. 检查Rust环境..."
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: Rust未安装"
    echo "请先安装Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

RUST_VERSION=$(rustc --version)
echo "Rust版本: $RUST_VERSION"
echo "✅ Rust环境检查通过"
echo ""

# 3. 编译项目
echo "3. 编译项目 (Release模式)..."
echo "这可能需要几分钟时间..."
echo ""

cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi

echo "✅ 编译成功"
echo ""

# 4. 检查配置文件
echo "4. 检查配置文件..."
if [ ! -f "config.toml" ]; then
    echo "⚠️  config.toml 不存在，使用默认配置"
    if [ -f "config.example.toml" ]; then
        cp config.example.toml config.toml
        echo "✅ 已创建 config.toml"
    fi
fi

if [ ! -f "genesis.json" ]; then
    echo "⚠️  genesis.json 不存在，使用默认创世配置"
    if [ -f "testnet-genesis.json" ]; then
        cp testnet-genesis.json genesis.json
        echo "✅ 已创建 genesis.json"
    fi
fi

echo "✅ 配置文件检查通过"
echo ""

# 5. 创建数据目录
echo "5. 创建数据目录..."
mkdir -p data
mkdir -p logs
echo "✅ 数据目录创建完成"
echo ""

# 6. 停止旧进程
echo "6. 检查并停止旧进程..."
OLD_PID=$(pgrep -f "toki-node" || true)
if [ ! -z "$OLD_PID" ]; then
    echo "发现旧进程: $OLD_PID"
    kill $OLD_PID
    sleep 2
    echo "✅ 旧进程已停止"
else
    echo "✅ 无旧进程运行"
fi
echo ""

# 7. 启动节点
echo "7. 启动Toki节点..."
echo ""

# 使用nohup后台运行
nohup ./target/release/toki-node > logs/toki.log 2>&1 &
NODE_PID=$!

echo "节点进程ID: $NODE_PID"
echo ""

# 等待启动
sleep 3

# 8. 验证运行状态
echo "8. 验证运行状态..."

# 检查进程是否运行
if ps -p $NODE_PID > /dev/null; then
    echo "✅ 节点进程正在运行"
else
    echo "❌ 节点进程启动失败"
    echo ""
    echo "查看日志:"
    tail -20 logs/toki.log
    exit 1
fi

echo ""

# 9. 检查API端口
echo "9. 检查API端口..."
sleep 2

if command -v curl &> /dev/null; then
    HEALTH=$(curl -s http://localhost:8080/health || echo "failed")
    if [ "$HEALTH" != "failed" ]; then
        echo "✅ API服务正常"
        echo "健康检查: $HEALTH"
    else
        echo "⚠️  API服务未响应，可能还在启动中"
    fi
else
    echo "⚠️  curl未安装，跳过API检查"
fi

echo ""

# 10. 显示运行信息
echo "=========================================="
echo "🎉 Toki 节点启动成功！"
echo "=========================================="
echo ""
echo "运行信息:"
echo "  - 进程ID: $NODE_PID"
echo "  - 日志文件: $PROJECT_DIR/logs/toki.log"
echo "  - 数据目录: $PROJECT_DIR/data"
echo "  - API地址: http://localhost:8080"
echo ""
echo "管理命令:"
echo "  - 查看日志: tail -f logs/toki.log"
echo "  - 停止节点: kill $NODE_PID"
echo "  - 重启节点: ./deploy_and_run.sh"
echo ""
echo "API测试:"
echo "  - 健康检查: curl http://localhost:8080/health"
echo "  - 节点信息: curl http://localhost:8080/api/v1/node/info"
echo "  - 收费公告: curl http://localhost:8080/api/v1/fee/announcement"
echo ""

# 保存PID到文件
echo $NODE_PID > toki.pid
echo "PID已保存到: toki.pid"
echo ""

echo "节点正在后台运行，使用 'tail -f logs/toki.log' 查看日志"
