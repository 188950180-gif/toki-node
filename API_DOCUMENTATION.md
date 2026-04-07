# Toki API 文档

## 概述

Toki 区块链节点提供 RESTful API 接口，默认监听端口 `8080`。

**基础 URL:** `http://localhost:8080`

## 通用响应格式

所有 API 响应都使用统一的 JSON 格式：

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## API 端点

### 1. 健康检查

**GET** `/health`

检查节点健康状态。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "healthy": true
  }
}
```

---

### 2. 节点信息

**GET** `/api/v1/node/info`

获取节点基本信息。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "node_id": "unknown",
    "version": "0.1.0",
    "height": 1234,
    "peer_count": 0,
    "is_syncing": false
  }
}
```

**字段说明：**
- `node_id`: 节点标识符
- `version`: 节点版本
- `height`: 当前区块高度
- `peer_count`: 连接的节点数量
- `is_syncing`: 是否正在同步

---

### 3. 共识状态

**GET** `/api/v1/consensus/status`

获取共识状态信息。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "algorithm": "PoW",
    "difficulty": 1000000,
    "hash_rate": 5000,
    "block_time": 10
  }
}
```

---

### 4. 获取区块

**GET** `/api/v1/block/{height}`

根据高度获取区块信息。

**路径参数：**
- `height` (integer): 区块高度

**响应示例：**
```json
{
  "success": true,
  "data": {
    "height": 0,
    "hash": "0000000000000000...",
    "prev_hash": "0000000000000000...",
    "timestamp": 1234567890,
    "tx_count": 1,
    "transactions": [...]
  }
}
```

---

### 5. 获取区块范围

**GET** `/api/v1/block/range?start={start}&end={end}`

获取指定范围的区块。

**查询参数：**
- `start` (integer): 起始高度
- `end` (integer): 结束高度

**响应示例：**
```json
{
  "success": true,
  "data": [
    {
      "height": 0,
      "hash": "...",
      ...
    },
    {
      "height": 1,
      "hash": "...",
      ...
    }
  ]
}
```

---

### 6. 获取最新区块

**GET** `/api/v1/block/latest`

获取最新的区块信息。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "height": 1234,
    "hash": "...",
    ...
  }
}
```

---

### 7. 获取交易

**GET** `/api/v1/transaction/{hash}`

根据哈希获取交易信息。

**路径参数：**
- `hash` (string): 交易哈希

**响应示例：**
```json
{
  "success": true,
  "data": {
    "tx_hash": "...",
    "inputs": [...],
    "outputs": [...],
    "fee": 1000,
    "timestamp": 1234567890
  }
}
```

---

### 8. 提交交易

**POST** `/api/v1/transaction/submit`

提交新交易到交易池。

**请求体：**
```json
{
  "inputs": [
    {
      "prev_tx_hash": "...",
      "output_index": 0
    }
  ],
  "outputs": [
    {
      "address": "...",
      "amount": 100000000
    }
  ],
  "ring_signature": {
    "ring": [...],
    "signature": "...",
    "key_image": "..."
  },
  "fee": 1000
}
```

**响应示例：**
```json
{
  "success": true,
  "data": {
    "tx_hash": "..."
  }
}
```

---

### 9. 获取账户余额

**GET** `/api/v1/account/balance/{address}`

获取指定地址的账户余额。

**路径参数：**
- `address` (string): 账户地址

**响应示例：**
```json
{
  "success": true,
  "data": {
    "address": "...",
    "balance": 100000000000,
    "locked": 0,
    "available": 100000000000
  }
}
```

---

### 10. 获取账户交易历史

**GET** `/api/v1/account/transactions/{address}`

获取指定地址的交易历史。

**路径参数：**
- `address` (string): 账户地址

**响应示例：**
```json
{
  "success": true,
  "data": [
    {
      "tx_hash": "...",
      "amount": 100000000,
      "type": "receive",
      "timestamp": 1234567890
    }
  ]
}
```

---

### 11. 获取交易池状态

**GET** `/api/v1/transaction-pool`

获取交易池状态。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "size": 10,
    "transactions": [
      {
        "tx_hash": "...",
        "fee": 1000,
        "size": 256,
        "timestamp": 1234567890
      }
    ]
  }
}
```

---

### 12. 获取网络状态

**GET** `/api/v1/network/status`

获取网络状态信息。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "listening": true,
    "peer_count": 0,
    "connections": []
  }
}
```

---

### 13. 获取统计信息

**GET** `/api/v1/stats`

获取系统统计信息。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "total_blocks": 1234,
    "total_transactions": 5678,
    "total_accounts": 100,
    "total_supply": 1000000000000000
  }
}
```

---

### 14. 获取 AI 状态

**GET** `/api/v1/ai/status`

获取 AI 经济系统状态。

**响应示例：**
```json
{
  "success": true,
  "data": {
    "theta": 1.5,
    "charity_pool": 100000000000,
    "welfare_pool": 50000000000,
    "last_distribution": 1234567890
  }
}
```

---

### 15. 触发 AI 任务

**POST** `/api/v1/ai/trigger?task={task}`

手动触发 AI 任务。

**查询参数：**
- `task` (string): 任务类型
  - `distribute_basic`: 基础分发
  - `calculate_theta`: 计算 Theta
  - `execute_charity`: 执行公益
  - `check_equalization`: 平权检查

**响应示例：**
```json
{
  "success": true,
  "data": {
    "task": "distribute_basic",
    "status": "completed",
    "result": "..."
  }
}
```

---

## 错误响应

当请求失败时，响应格式如下：

```json
{
  "success": false,
  "data": null,
  "error": "错误信息"
}
```

**常见错误码：**
- `404`: 资源未找到
- `400`: 请求参数错误
- `500`: 服务器内部错误

---

## 使用示例

### cURL 示例

```bash
# 健康检查
curl http://localhost:8080/health

# 获取节点信息
curl http://localhost:8080/api/v1/node/info

# 获取区块
curl http://localhost:8080/api/v1/block/0

# 获取账户余额
curl http://localhost:8080/api/v1/account/balance/TOKI...

# 提交交易
curl -X POST http://localhost:8080/api/v1/transaction/submit \
  -H "Content-Type: application/json" \
  -d '{"inputs":[...],"outputs":[...],"fee":1000}'
```

### JavaScript 示例

```javascript
// 获取节点信息
const response = await fetch('http://localhost:8080/api/v1/node/info');
const data = await response.json();
console.log(data);

// 获取区块
const block = await fetch('http://localhost:8080/api/v1/block/0');
const blockData = await block.json();
console.log(blockData);

// 提交交易
const txResponse = await fetch('http://localhost:8080/api/v1/transaction/submit', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    inputs: [...],
    outputs: [...],
    fee: 1000
  })
});
const txResult = await txResponse.json();
console.log(txResult);
```

### Python 示例

```python
import requests

# 获取节点信息
response = requests.get('http://localhost:8080/api/v1/node/info')
print(response.json())

# 获取区块
block = requests.get('http://localhost:8080/api/v1/block/0')
print(block.json())

# 提交交易
tx = requests.post('http://localhost:8080/api/v1/transaction/submit', json={
    'inputs': [...],
    'outputs': [...],
    'fee': 1000
})
print(tx.json())
```

---

## 速率限制

当前版本没有速率限制。生产环境建议添加速率限制中间件。

---

## CORS

API 默认启用 CORS，允许跨域请求。

---

## WebSocket (计划中)

未来版本将支持 WebSocket 连接，用于实时推送区块和交易事件。

---

## 版本控制

API 使用 URL 版本控制（`/api/v1/`）。未来版本可能引入新的 API 版本（`/api/v2/`）。
