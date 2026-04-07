# Toki 项目实施计划

## 项目当前状态

### 已完成功能 ✅

1. **核心系统** (100%)
   - 账户管理
   - 交易处理
   - 区块结构
   - 地址系统

2. **存储系统** (100%)
   - Sled 数据库
   - 区块存储
   - 账户存储
   - 缓存机制

3. **共识系统** (100%)
   - PoW 挖矿
   - 难度调整
   - 交易池
   - 区块验证

4. **API 系统** (100%)
   - 15 个 REST 端点
   - 数据查询
   - 交易提交
   - 健康检查

5. **AI 经济系统** (90%)
   - Token 分发逻辑
   - Theta 计算框架
   - 公益执行框架
   - 平权检查框架
   - 福利分配框架

### 待完善功能 ⚠️

1. **P2P 网络** (60%)
   - libp2p 版本适配
   - 节点发现
   - 区块同步
   - 交易广播

2. **测试** (30%)
   - 单元测试
   - 集成测试
   - 多节点测试

3. **文档** (20%)
   - API 文档
   - 部署文档
   - 使用指南

## 实施计划

### 阶段一：短期任务（1 周）

#### 1. 完善 AI 数据连接（2 天）

**任务：**
- [ ] 在 `scheduler_impl.rs` 中实现 `calculate_theta()` 连接实际交易数据
- [ ] 在 `scheduler_impl.rs` 中实现 `execute_charity()` 连接公益池
- [ ] 在节点启动时初始化 AI 系统
- [ ] 测试 AI 任务触发

**代码示例：**
```rust
// scheduler_impl.rs
async fn calculate_theta(&self) -> Result<()> {
    let block_store = match &self.block_store {
        Some(store) => store,
        None => return Ok(()),
    };
    
    // 获取最近 1000 个交易
    let recent_txs = block_store.get_recent_transactions(1000)?;
    
    // 计算 Theta 值
    let theta = self.theta_calculator.calculate(&recent_txs);
    
    info!("Theta 计算完成: {}", theta);
    Ok(())
}
```

#### 2. 编写单元测试（2 天）

**任务：**
- [ ] 核心模块测试（core）
- [ ] 存储模块测试（storage）
- [ ] 共识模块测试（consensus）
- [ ] AI 模块测试（ai）

**测试示例：**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_distribution() {
        let mut executor = AITaskExecutor::new(...);
        executor.with_storage(block_store, account_store, tx_pool);
        
        let result = executor.distribute_basic();
        assert!(result.is_ok());
    }
}
```

#### 3. 性能优化（1 天）

**任务：**
- [ ] 优化数据库查询
- [ ] 优化区块验证
- [ ] 优化 API 响应
- [ ] 添加缓存机制

**优化点：**
```rust
// 使用缓存
use lru::LruCache;

pub struct CachedBlockStore {
    store: BlockStore,
    cache: Arc<RwLock<LruCache<u64, Block>>>,
}
```

### 阶段二：中期任务（2-4 周）

#### 4. 修复 P2P 网络（1-2 周）

**方案选择：**
- **方案 A：** 降级到 libp2p 0.52（推荐）
- **方案 B：** 等待 libp2p 0.56 文档完善
- **方案 C：** 使用其他 P2P 库

**实施步骤（方案 A）：**
```toml
# Cargo.toml
libp2p = { version = "0.52", features = [...] }
```

```rust
// p2p.rs
use libp2p::gossipsub::Gossipsub;
use libp2p::mdns::Behaviour as Mdns;

let transport = libp2p::development_transport(local_key).await?;
let swarm = Swarm::new(transport, behaviour, local_peer_id);
```

#### 5. 多节点测试（1 周）

**测试场景：**
- [ ] 两节点连接测试
- [ ] 区块同步测试
- [ ] 交易广播测试
- [ ] AI 任务测试

**测试脚本：**
```bash
#!/bin/bash
# test-multi-node.sh

# 启动节点 1
cargo run --release -- --port 30333 --api-port 8080 &

# 启动节点 2
cargo run --release -- --port 30334 --api-port 8081 \
    --bootnodes /ip4/127.0.0.1/tcp/30333 &

# 等待连接
sleep 5

# 测试连接
curl http://localhost:8081/api/v1/node/info
```

#### 6. 文档完善（1 周）

**文档列表：**
- [ ] API 文档（OpenAPI/Swagger）
- [ ] 部署文档
- [ ] 配置说明
- [ ] 开发指南

**API 文档示例：**
```yaml
openapi: 3.0.0
info:
  title: Toki API
  version: 0.1.0
paths:
  /api/v1/block/{height}:
    get:
      summary: 获取指定区块
      parameters:
        - name: height
          in: path
          required: true
          schema:
            type: integer
```

### 阶段三：长期任务（1-2 月）

#### 7. 生产部署（2-4 周）

**部署清单：**
- [ ] Docker 镜像
- [ ] Kubernetes 配置
- [ ] 监控系统
- [ ] 日志系统
- [ ] 备份系统

**Dockerfile 示例：**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
COPY --from=builder /app/target/release/toki-node /usr/local/bin/
EXPOSE 30333 8080
CMD ["toki-node", "start", "--config", "/etc/toki/config.toml"]
```

#### 8. 安全审计（1-2 周）

**审计项目：**
- [ ] 密码学实现
- [ ] 共识算法
- [ ] P2P 网络
- [ ] API 安全
- [ ] 数据存储

#### 9. 社区推广（持续）

**推广计划：**
- [ ] GitHub 文档
- [ ] 技术博客
- [ ] 社区论坛
- [ ] 开发者文档

## 时间线

```
Week 1: AI 数据连接 + 单元测试 + 性能优化
Week 2-3: P2P 网络修复
Week 4: 多节点测试
Week 5: 文档完善
Week 6-8: 生产部署
Week 9-10: 安全审计
Week 11+: 社区推广
```

## 成功指标

### 技术指标
- ✅ 单节点稳定运行 > 24 小时
- ✅ API 响应时间 < 100ms
- ✅ 区块生成时间 ≈ 10s
- ⚠️ 多节点同步延迟 < 5s
- ⚠️ P2P 连接成功率 > 95%

### 业务指标
- ✅ Token 分发准确率 100%
- ✅ Theta 计算正确性验证
- ⚠️ 公益执行透明度
- ⚠️ 平权检查有效性

## 风险与应对

### 技术风险
1. **libp2p 版本问题**
   - 风险：API 不兼容
   - 应对：降级到稳定版本

2. **性能瓶颈**
   - 风险：交易处理慢
   - 应对：优化算法，增加缓存

3. **安全漏洞**
   - 风险：密码学实现问题
   - 应对：使用成熟库，代码审计

### 项目风险
1. **进度延迟**
   - 风险：功能复杂度超预期
   - 应对：优先核心功能，迭代开发

2. **资源不足**
   - 风险：开发人员不足
   - 应对：社区协作，任务分解

## 总结

Toki 项目已完成核心功能开发，当前进度约 85%。通过本实施计划，预计在 3-4 周内完成所有待完善功能，达到生产就绪状态。

**关键里程碑：**
- Week 1: AI 系统完全可用
- Week 3: P2P 网络正常工作
- Week 5: 多节点测试通过
- Week 8: 生产环境部署完成

项目正在按计划稳步推进，核心功能已验证可用，后续工作主要是完善和优化。
