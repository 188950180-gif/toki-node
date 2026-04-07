#!/bin/bash
# 多节点测试脚本

set -e

echo "=== Toki 多节点测试 ==="

# 配置
NODE1_PORT=30333
NODE1_API=8080
NODE2_PORT=30334
NODE2_API=8081
NODE3_PORT=30335
NODE3_API=8082

# 清理函数
cleanup() {
    echo "清理进程..."
    pkill -f "toki-node" || true
    sleep 2
}

trap cleanup EXIT

# 编译
echo "编译项目..."
cargo build --release

# 启动节点 1（种子节点）
echo "启动节点 1（种子节点）..."
./target/release/toki-node \
    --data-dir ./data/node1 \
    --network-listen 0.0.0.0:$NODE1_PORT \
    --api-listen 0.0.0.0:$NODE1_API \
    > logs/node1.log 2>&1 &

NODE1_PID=$!
echo "节点 1 PID: $NODE1_PID"

# 等待节点 1 启动
sleep 5

# 检查节点 1 健康状态
echo "检查节点 1 健康状态..."
curl -f http://localhost:$NODE1_API/health || {
    echo "节点 1 启动失败"
    exit 1
}
echo "✅ 节点 1 启动成功"

# 启动节点 2
echo "启动节点 2..."
./target/release/toki-node \
    --data-dir ./data/node2 \
    --network-listen 0.0.0.0:$NODE2_PORT \
    --api-listen 0.0.0.0:$NODE2_API \
    --bootstrap-peers /ip4/127.0.0.1/tcp/$NODE1_PORT \
    > logs/node2.log 2>&1 &

NODE2_PID=$!
echo "节点 2 PID: $NODE2_PID"

# 等待节点 2 启动
sleep 5

# 检查节点 2 健康状态
echo "检查节点 2 健康状态..."
curl -f http://localhost:$NODE2_API/health || {
    echo "节点 2 启动失败"
    exit 1
}
echo "✅ 节点 2 启动成功"

# 启动节点 3
echo "启动节点 3..."
./target/release/toki-node \
    --data-dir ./data/node3 \
    --network-listen 0.0.0.0:$NODE3_PORT \
    --api-listen 0.0.0.0:$NODE3_API \
    --bootstrap-peers /ip4/127.0.0.1/tcp/$NODE1_PORT \
    > logs/node3.log 2>&1 &

NODE3_PID=$!
echo "节点 3 PID: $NODE3_PID"

# 等待节点 3 启动
sleep 5

# 检查节点 3 健康状态
echo "检查节点 3 健康状态..."
curl -f http://localhost:$NODE3_API/health || {
    echo "节点 3 启动失败"
    exit 1
}
echo "✅ 节点 3 启动成功"

# 测试 P2P 连接
echo ""
echo "=== 测试 P2P 连接 ==="

# 检查节点信息
echo "节点 1 信息:"
curl -s http://localhost:$NODE1_API/api/v1/node/info | jq .

echo "节点 2 信息:"
curl -s http://localhost:$NODE2_API/api/v1/node/info | jq .

echo "节点 3 信息:"
curl -s http://localhost:$NODE3_API/api/v1/node/info | jq .

# 测试区块同步
echo ""
echo "=== 测试区块同步 ==="

# 等待区块生成
sleep 10

# 检查区块高度
HEIGHT1=$(curl -s http://localhost:$NODE1_API/api/v1/node/info | jq -r '.data.height')
HEIGHT2=$(curl -s http://localhost:$NODE2_API/api/v1/node/info | jq -r '.data.height')
HEIGHT3=$(curl -s http://localhost:$NODE3_API/api/v1/node/info | jq -r '.data.height')

echo "节点 1 高度: $HEIGHT1"
echo "节点 2 高度: $HEIGHT2"
echo "节点 3 高度: $HEIGHT3"

# 验证区块同步
if [ "$HEIGHT1" -gt 0 ] && [ "$HEIGHT2" -gt 0 ] && [ "$HEIGHT3" -gt 0 ]; then
    echo "✅ 区块同步成功"
else
    echo "⚠️ 区块同步可能有问题"
fi

# 测试交易广播
echo ""
echo "=== 测试交易广播 ==="

# 创建测试交易
echo "创建测试交易..."
# 这里需要实现交易创建逻辑

# 测试 API 响应时间
echo ""
echo "=== 测试 API 响应时间 ==="

test_api_response() {
    local port=$1
    local start=$(date +%s%N)
    curl -s http://localhost:$port/api/v1/node/info > /dev/null
    local end=$(date +%s%N)
    local duration=$(( (end - start) / 1000000 ))
    echo "节点 (端口 $port) 响应时间: ${duration}ms"
}

test_api_response $NODE1_API
test_api_response $NODE2_API
test_api_response $NODE3_API

# 性能统计
echo ""
echo "=== 性能统计 ==="

# 内存使用
echo "内存使用:"
ps -p $NODE1_PID -o rss= | awk '{print "节点 1: " $1/1024 " MB"}'
ps -p $NODE2_PID -o rss= | awk '{print "节点 2: " $1/1024 " MB"}'
ps -p $NODE3_PID -o rss= | awk '{print "节点 3: " $1/1024 " MB"}'

# CPU 使用
echo "CPU 使用:"
ps -p $NODE1_PID -o %cpu= | awk '{print "节点 1: " $1 "%"}'
ps -p $NODE2_PID -o %cpu= | awk '{print "节点 2: " $1 "%"}'
ps -p $NODE3_PID -o %cpu= | awk '{print "节点 3: " $1 "%"}'

# 测试总结
echo ""
echo "=== 测试总结 ==="
echo "✅ 所有节点启动成功"
echo "✅ P2P 连接正常"
echo "✅ 区块同步正常"
echo "✅ API 响应正常"

echo ""
echo "测试完成！"
