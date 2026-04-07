#!/bin/bash
# Toki 测试网启动脚本

echo "=== Toki 测试网启动 ==="
echo ""

# 检查可执行文件
if [ ! -f "./target/release/toki-node.exe" ]; then
    echo "错误: 未找到 toki-node 可执行文件"
    echo "请先运行: cargo build --release"
    exit 1
fi

# 创建数据目录
mkdir -p ./testnet-data

# 检查创世区块
if [ ! -f "./testnet-genesis.json" ]; then
    echo "错误: 未找到创世区块文件"
    exit 1
fi

echo "配置文件: testnet-config.toml"
echo "创世区块: testnet-genesis.json"
echo "数据目录: ./testnet-data"
echo ""

# 启动节点
echo "启动测试网节点..."
./target/release/toki-node.exe start --config testnet-config.toml
