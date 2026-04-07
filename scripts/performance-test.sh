#!/bin/bash
# 性能压力测试脚本

set -e

echo "=== Toki 性能压力测试 ==="

# 配置
API_URL="http://localhost:8080"
DURATION=60  # 测试持续时间（秒）
CONCURRENT=10  # 并发数
REPORT_DIR="./performance-report"

# 创建报告目录
mkdir -p $REPORT_DIR

# 检查服务是否运行
echo "检查服务状态..."
curl -f $API_URL/health || {
    echo "服务未运行，请先启动节点"
    exit 1
}

echo "✅ 服务运行正常"
echo ""

# 1. API 响应时间测试
echo "=== 1. API 响应时间测试 ==="

test_endpoint() {
    local endpoint=$1
    local name=$2
    local iterations=100

    echo "测试 $name ($endpoint)..."

    total_time=0
    min_time=999999
    max_time=0

    for i in $(seq 1 $iterations); do
        start=$(date +%s%N)
        curl -s $API_URL$endpoint > /dev/null
        end=$(date +%s%N)
        duration=$(( (end - start) / 1000000 ))

        total_time=$((total_time + duration))

        if [ $duration -lt $min_time ]; then
            min_time=$duration
        fi

        if [ $duration -gt $max_time ]; then
            max_time=$duration
        fi
    done

    avg_time=$((total_time / iterations))

    echo "  平均: ${avg_time}ms"
    echo "  最小: ${min_time}ms"
    echo "  最大: ${max_time}ms"
    echo ""

    # 保存结果
    echo "$name,$avg_time,$min_time,$max_time" >> $REPORT_DIR/api_response.csv
}

# 测试各个端点
echo "endpoint,avg_ms,min_ms,max_ms" > $REPORT_DIR/api_response.csv
test_endpoint "/health" "健康检查"
test_endpoint "/api/v1/node/info" "节点信息"
test_endpoint "/api/v1/consensus/status" "共识状态"
test_endpoint "/api/v1/block/0" "区块查询"
test_endpoint "/api/v1/stats" "统计信息"

# 2. 并发请求测试
echo "=== 2. 并发请求测试 ==="

test_concurrent() {
    local concurrent=$1
    local requests=$2

    echo "测试并发数: $concurrent, 总请求: $requests"

    start=$(date +%s%N)

    # 使用 xargs 实现并发
    seq $requests | xargs -P $concurrent -I {} curl -s $API_URL/api/v1/node/info > /dev/null

    end=$(date +%s%N)
    duration=$(( (end - start) / 1000000 ))

    rps=$(( requests * 1000 / duration ))

    echo "  总时间: ${duration}ms"
    echo "  RPS: $rps"
    echo ""

    echo "$concurrent,$requests,$duration,$rps" >> $REPORT_DIR/concurrent.csv
}

echo "concurrent,requests,duration_ms,rps" > $REPORT_DIR/concurrent.csv
test_concurrent 1 100
test_concurrent 5 100
test_concurrent 10 100
test_concurrent 20 100
test_concurrent 50 100

# 3. 区块处理性能测试
echo "=== 3. 区块处理性能测试 ==="

# 获取当前区块高度
initial_height=$(curl -s $API_URL/api/v1/node/info | jq -r '.data.height')
echo "初始区块高度: $initial_height"

# 等待新区块生成
echo "等待新区块生成..."
sleep 30

# 获取新区块高度
new_height=$(curl -s $API_URL/api/v1/node/info | jq -r '.data.height')
echo "新区块高度: $new_height"

# 计算区块生成速度
blocks_generated=$((new_height - initial_height))
block_time=$((30 / blocks_generated))

echo "生成区块数: $blocks_generated"
echo "平均区块时间: ${block_time}s"
echo ""

echo "blocks_generated,block_time" > $REPORT_DIR/block_performance.csv
echo "$blocks_generated,$block_time" >> $REPORT_DIR/block_performance.csv

# 4. 内存使用监控
echo "=== 4. 内存使用监控 ==="

PID=$(pgrep -f "toki-node")
echo "节点 PID: $PID"

# 监控 60 秒
echo "监控内存使用（60秒）..."
echo "time,memory_mb,cpu_percent" > $REPORT_DIR/memory_usage.csv

for i in $(seq 1 60); do
    memory=$(ps -p $PID -o rss= | awk '{print $1/1024}')
    cpu=$(ps -p $PID -o %cpu= | awk '{print $1}')

    echo "$i,$memory,$cpu" >> $REPORT_DIR/memory_usage.csv
    sleep 1
done

echo "✅ 内存监控完成"
echo ""

# 5. 交易处理性能测试
echo "=== 5. 交易处理性能测试 ==="

# 创建测试交易
echo "创建测试交易..."

create_transaction() {
    # 这里需要实现交易创建逻辑
    # 示例：curl -X POST $API_URL/api/v1/transaction/submit -d @tx.json
    echo "交易创建功能待实现"
}

# 测试交易处理速度
echo "测试交易处理速度..."
# 待实现

# 6. 生成性能报告
echo "=== 6. 生成性能报告 ==="

cat > $REPORT_DIR/summary.md << EOF
# Toki 性能测试报告

**测试时间：** $(date)
**测试持续时间：** ${DURATION}s
**并发数：** ${CONCURRENT}

## 1. API 响应时间

| 端点 | 平均时间 | 最小时间 | 最大时间 |
|------|----------|----------|----------|
$(tail -n +2 $REPORT_DIR/api_response.csv | awk -F',' '{printf "| %s | %sms | %sms | %sms |\n", $1, $2, $3, $4}')

## 2. 并发性能

| 并发数 | 请求数 | 总时间 | RPS |
|--------|--------|--------|-----|
$(tail -n +2 $REPORT_DIR/concurrent.csv | awk -F',' '{printf "| %s | %s | %sms | %s |\n", $1, $2, $3, $4}')

## 3. 区块处理

- 生成区块数：$blocks_generated
- 平均区块时间：${block_time}s

## 4. 资源使用

- 内存使用：$(tail -1 $REPORT_DIR/memory_usage.csv | awk -F',' '{print $2}') MB
- CPU 使用：$(tail -1 $REPORT_DIR/memory_usage.csv | awk -F',' '{print $3}')%

## 5. 性能建议

1. API 响应时间应 < 100ms
2. RPS 应 > 100
3. 区块时间应 ≈ 10s
4. 内存使用应 < 1GB
EOF

echo "✅ 性能报告已生成: $REPORT_DIR/summary.md"
echo ""

# 显示报告
cat $REPORT_DIR/summary.md

echo ""
echo "=== 测试完成 ==="
