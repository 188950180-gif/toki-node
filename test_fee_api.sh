#!/bin/bash

# API测试脚本

echo "=========================================="
echo "API 接口测试"
echo "=========================================="
echo ""

# 测试健康检查
echo "1. 测试健康检查..."
curl -s http://localhost:8080/health | jq .
echo ""

# 测试节点信息
echo "2. 测试节点信息..."
curl -s http://localhost:8080/api/v1/node/info | jq .
echo ""

# 测试收费公告
echo "3. 测试收费公告..."
curl -s http://localhost:8080/api/v1/fee/announcement | jq .
echo ""

# 测试交易池
echo "4. 测试交易池..."
curl -s http://localhost:8080/api/v1/transaction-pool | jq .
echo ""

echo "=========================================="
echo "API 测试完成"
echo "=========================================="
