# 多文件夹覆盖指南

## 📋 项目文件夹情况

| 文件夹 | 路径 | 状态 | 说明 |
|--------|------|------|------|
| **tokipt** | `../tokipt` | ✅ 主项目 | 当前工作目录，最新版本 |
| **tokipt-backup** | `../tokipt-backup` | 📦 备份 | 旧版本备份（4月2日） |
| **tokipt-fixed** | `../tokipt-fixed` | 🔧 修复版 | 修复版本（4月4日） |

---

## 🔍 文件差异分析

**检查结果：**
- ✅ 三个文件夹的核心文件内容**完全相同**
- ✅ `tokipt` vs `tokipt-backup`：文件相同
- ✅ `tokipt` vs `tokipt-fixed`：文件相同

---

## 💡 覆盖建议

### 方案1：只覆盖主项目（推荐）✅

**适用场景：**
- 只在主项目 `tokipt` 中开发
- 备份和修复版不需要更新

**操作步骤：**
```bash
# 在 tokipt 目录执行
./fee_delay_modification/overwrite_files.bat
```

**优点：**
- ✅ 简单快速
- ✅ 不影响备份
- ✅ 风险最小

---

### 方案2：覆盖主项目和修复版

**适用场景：**
- `tokipt` 和 `tokipt-fixed` 都在使用
- 需要保持两个版本同步

**操作步骤：**
```bash
# 使用多项目覆盖脚本
./fee_delay_modification/overwrite_multiple_projects.bat

# 选择: 1 2 (覆盖 tokipt 和 tokipt-fixed)
```

**优点：**
- ✅ 保持多个版本同步
- ✅ 修复版也能使用新功能

---

### 方案3：覆盖所有三个文件夹（不推荐）❌

**风险：**
- ❌ 破坏备份完整性
- ❌ 无法回退到旧版本
- ❌ 失去备份意义

**如果必须覆盖备份：**
```bash
# 使用多项目覆盖脚本
./fee_delay_modification/overwrite_multiple_projects.bat

# 选择: 1 2 3 (覆盖所有)
# 需要输入 "yes" 确认覆盖备份
```

---

## 🚀 使用方法

### 方法1：单项目覆盖（推荐）

**Windows：**
```bash
fee_delay_modification\overwrite_files.bat
```

**Linux/Mac：**
```bash
chmod +x fee_delay_modification/overwrite_files.sh
./fee_delay_modification/overwrite_files.sh
```

---

### 方法2：多项目覆盖

**Windows：**
```bash
fee_delay_modification\overwrite_multiple_projects.bat
```

**Linux/Mac：**
```bash
chmod +x fee_delay_modification/overwrite_multiple_projects.sh
./fee_delay_modification/overwrite_multiple_projects.sh
```

**交互式选择：**
```
发现以下项目：
1. tokipt (主项目)
2. tokipt-fixed (修复版)
3. tokipt-backup (备份，不建议修改)

请选择要覆盖的项目（输入数字，多个用空格分隔，如: 1 2）: 1 2
```

---

## 📊 覆盖路径映射

### tokipt 项目

| 修改文件 | 原文件路径 |
|---------|-----------|
| `constants.rs` | `tokipt/core/src/constants.rs` |
| `block.rs` | `tokipt/core/src/block.rs` |
| `transaction.rs` | `tokipt/core/src/transaction.rs` |
| `validator.rs` | `tokipt/consensus/src/validator.rs` |
| `block_store.rs` | `tokipt/storage/src/block_store.rs` |
| `handlers.rs` | `tokipt/api/src/handlers.rs` |
| `routes.rs` | `tokipt/api/src/routes.rs` |
| `genesis.rs` | `tokipt/core/src/genesis.rs` |
| `tx_pool.rs` | `tokipt/consensus/src/tx_pool.rs` |

### tokipt-fixed 项目

| 修改文件 | 原文件路径 |
|---------|-----------|
| `constants.rs` | `tokipt-fixed/core/src/constants.rs` |
| `block.rs` | `tokipt-fixed/core/src/block.rs` |
| `transaction.rs` | `tokipt-fixed/core/src/transaction.rs` |
| `validator.rs` | `tokipt-fixed/consensus/src/validator.rs` |
| `block_store.rs` | `tokipt-fixed/storage/src/block_store.rs` |
| `handlers.rs` | `tokipt-fixed/api/src/handlers.rs` |
| `routes.rs` | `tokipt-fixed/api/src/routes.rs` |
| `genesis.rs` | `tokipt-fixed/core/src/genesis.rs` |
| `tx_pool.rs` | `tokipt-fixed/consensus/src/tx_pool.rs` |

### tokipt-backup 项目

| 修改文件 | 原文件路径 |
|---------|-----------|
| `constants.rs` | `tokipt-backup/core/src/constants.rs` |
| `block.rs` | `tokipt-backup/core/src/block.rs` |
| `transaction.rs` | `tokipt-backup/core/src/transaction.rs` |
| `validator.rs` | `tokipt-backup/consensus/src/validator.rs` |
| `block_store.rs` | `tokipt-backup/storage/src/block_store.rs` |
| `handlers.rs` | `tokipt-backup/api/src/handlers.rs` |
| `routes.rs` | `tokipt-backup/api/src/routes.rs` |
| `genesis.rs` | `tokipt-backup/core/src/genesis.rs` |
| `tx_pool.rs` | `tokipt-backup/consensus/src/tx_pool.rs` |

---

## ⚠️ 重要提示

### 1. 备份保护

**tokipt-backup 是备份目录，建议不要修改！**

如果必须修改：
- 确保已创建新的备份
- 输入 "yes" 确认覆盖
- 理解这将破坏备份完整性

### 2. 编译验证

覆盖后必须编译验证：
```bash
cargo build --workspace --release
```

### 3. 测试验证

编译成功后运行测试：
```bash
cargo test --workspace
```

---

## 📝 推荐工作流程

### 标准流程（推荐）

```bash
# 1. 在主项目修改
cd tokipt

# 2. 修改 fee_delay_modification 目录中的文件

# 3. 覆盖主项目
./fee_delay_modification/overwrite_files.bat

# 4. 编译验证
cargo build --workspace --release

# 5. 测试验证
cargo test --workspace

# 6. 如果需要，同步到其他项目
./fee_delay_modification/overwrite_multiple_projects.bat
```

### 多项目同步流程

```bash
# 1. 使用多项目覆盖脚本
./fee_delay_modification/overwrite_multiple_projects.bat

# 2. 选择要覆盖的项目（如: 1 2）

# 3. 等待覆盖完成

# 4. 选择编译验证（y）

# 5. 检查所有项目编译结果
```

---

## 🎯 总结

**推荐方案：只覆盖主项目（tokipt）**

**原因：**
1. ✅ 主项目是开发重点
2. ✅ 备份不应修改
3. ✅ 修复版可按需同步
4. ✅ 风险最小，操作最简单

**如果需要多项目同步：**
- 使用 `overwrite_multiple_projects.bat`
- 选择需要覆盖的项目
- 避免覆盖备份目录

---

**创建时间：** 2026-04-06
**状态：** 已完成
