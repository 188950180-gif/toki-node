# 文件覆盖指南

## 📋 覆盖命令清单

修改完成后，按以下顺序执行覆盖命令：

### 1. 核心常量文件
```bash
cp fee_delay_modification/constants.rs core/src/constants.rs
```
**原文件路径：** `core/src/constants.rs`
**修改内容：** 新增 FEE_DELAY_DAYS 和 FEE_ANNOUNCEMENT_DAYS 常量

---

### 2. 区块结构文件
```bash
cp fee_delay_modification/block.rs core/src/block.rs
```
**原文件路径：** `core/src/block.rs`
**修改内容：** BlockHeader 新增 genesis_timestamp 字段

---

### 3. 交易逻辑文件
```bash
cp fee_delay_modification/transaction.rs core/src/transaction.rs
```
**原文件路径：** `core/src/transaction.rs`
**修改内容：** 新增 FeeAnnouncement 结构体和 calculate_fee_with_delay 方法

---

### 4. 交易验证文件
```bash
cp fee_delay_modification/validator.rs consensus/src/validator.rs
```
**原文件路径：** `consensus/src/validator.rs`
**修改内容：** 修改验证逻辑，允许180天内0费用

---

### 5. 区块存储文件
```bash
cp fee_delay_modification/block_store.rs storage/src/block_store.rs
```
**原文件路径：** `storage/src/block_store.rs`
**修改内容：** 新增 get_genesis_timestamp 方法

---

### 6. API处理器文件
```bash
cp fee_delay_modification/handlers.rs api/src/handlers.rs
```
**原文件路径：** `api/src/handlers.rs`
**修改内容：** 修改交易提交逻辑，新增收费公告API

---

### 7. API路由文件
```bash
cp fee_delay_modification/routes.rs api/src/routes.rs
```
**原文件路径：** `api/src/routes.rs`
**修改内容：** 新增收费公告路由

---

### 8. 创世配置文件
```bash
cp fee_delay_modification/genesis.rs core/src/genesis.rs
```
**原文件路径：** `core/src/genesis.rs`
**修改内容：** 确认创世时间配置正确

---

### 9. 交易池配置文件
```bash
cp fee_delay_modification/tx_pool.rs consensus/src/tx_pool.rs
```
**原文件路径：** `consensus/src/tx_pool.rs`
**修改内容：** 新增 fee_delay_days 配置字段

---

## 🚀 一键覆盖脚本

创建 `overwrite_files.sh` 脚本：

```bash
#!/bin/bash

echo "开始覆盖文件..."

# 1. 核心常量
cp fee_delay_modification/constants.rs core/src/constants.rs
echo "✅ constants.rs 已覆盖"

# 2. 区块结构
cp fee_delay_modification/block.rs core/src/block.rs
echo "✅ block.rs 已覆盖"

# 3. 交易逻辑
cp fee_delay_modification/transaction.rs core/src/transaction.rs
echo "✅ transaction.rs 已覆盖"

# 4. 交易验证
cp fee_delay_modification/validator.rs consensus/src/validator.rs
echo "✅ validator.rs 已覆盖"

# 5. 区块存储
cp fee_delay_modification/block_store.rs storage/src/block_store.rs
echo "✅ block_store.rs 已覆盖"

# 6. API处理器
cp fee_delay_modification/handlers.rs api/src/handlers.rs
echo "✅ handlers.rs 已覆盖"

# 7. API路由
cp fee_delay_modification/routes.rs api/src/routes.rs
echo "✅ routes.rs 已覆盖"

# 8. 创世配置
cp fee_delay_modification/genesis.rs core/src/genesis.rs
echo "✅ genesis.rs 已覆盖"

# 9. 交易池配置
cp fee_delay_modification/tx_pool.rs consensus/src/tx_pool.rs
echo "✅ tx_pool.rs 已覆盖"

echo ""
echo "所有文件已覆盖完成！"
echo "开始编译验证..."
cargo build --workspace --release
```

**使用方法：**
```bash
chmod +x overwrite_files.sh
./overwrite_files.sh
```

---

## 📊 文件映射表

| 序号 | 修改文件 | 原文件路径 | 覆盖命令 |
|------|---------|-----------|---------|
| 1 | `constants.rs` | `core/src/constants.rs` | `cp fee_delay_modification/constants.rs core/src/constants.rs` |
| 2 | `block.rs` | `core/src/block.rs` | `cp fee_delay_modification/block.rs core/src/block.rs` |
| 3 | `transaction.rs` | `core/src/transaction.rs` | `cp fee_delay_modification/transaction.rs core/src/transaction.rs` |
| 4 | `validator.rs` | `consensus/src/validator.rs` | `cp fee_delay_modification/validator.rs consensus/src/validator.rs` |
| 5 | `block_store.rs` | `storage/src/block_store.rs` | `cp fee_delay_modification/block_store.rs storage/src/block_store.rs` |
| 6 | `handlers.rs` | `api/src/handlers.rs` | `cp fee_delay_modification/handlers.rs api/src/handlers.rs` |
| 7 | `routes.rs` | `api/src/routes.rs` | `cp fee_delay_modification/routes.rs api/src/routes.rs` |
| 8 | `genesis.rs` | `core/src/genesis.rs` | `cp fee_delay_modification/genesis.rs core/src/genesis.rs` |
| 9 | `tx_pool.rs` | `consensus/src/tx_pool.rs` | `cp fee_delay_modification/tx_pool.rs consensus/src/tx_pool.rs` |

---

## ⚠️ 注意事项

1. **备份原文件**：覆盖前建议备份原文件
   ```bash
   mkdir -p backup_$(date +%Y%m%d)
   cp core/src/constants.rs backup_$(date +%Y%m%d)/
   # ... 其他文件
   ```

2. **编译验证**：覆盖后必须编译验证
   ```bash
   cargo build --workspace --release
   ```

3. **测试验证**：编译成功后运行测试
   ```bash
   cargo test --workspace
   ```

4. **功能验证**：测试以下场景
   - 0-165天：交易费为0，无公告
   - 165-180天：交易费为0，有公告
   - 180天后：交易费正常，无公告

---

## 📝 修改检查清单

- [ ] constants.rs - 已修改并覆盖
- [ ] block.rs - 已修改并覆盖
- [ ] transaction.rs - 已修改并覆盖
- [ ] validator.rs - 已修改并覆盖
- [ ] block_store.rs - 已修改并覆盖
- [ ] handlers.rs - 已修改并覆盖
- [ ] routes.rs - 已修改并覆盖
- [ ] genesis.rs - 已修改并覆盖
- [ ] tx_pool.rs - 已修改并覆盖
- [ ] 编译验证通过
- [ ] 测试验证通过
- [ ] 功能验证通过

---

**创建时间：** 2026-04-06
**状态：** 待修改
