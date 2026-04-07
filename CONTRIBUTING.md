# 贡献指南

感谢您考虑为 Toki 项目做出贡献！

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [Pull Request 流程](#pull-request-流程)

---

## 行为准则

本项目采用贡献者公约作为行为准则。参与本项目即表示您同意遵守其条款。请阅读 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) 了解详情。

---

## 如何贡献

### 报告 Bug

如果您发现了 Bug，请创建 Issue 并包含：

1. **Bug 描述**：清晰简洁地描述 Bug
2. **复现步骤**：详细的复现步骤
3. **预期行为**：您期望发生什么
4. **实际行为**：实际发生了什么
5. **环境信息**：
   - 操作系统
   - Rust 版本
   - Toki 版本
6. **日志**：相关的日志输出
7. **截图**：如果适用，添加截图

### 建议新功能

如果您有新功能的建议，请创建 Issue 并包含：

1. **功能描述**：清晰描述新功能
2. **使用场景**：为什么需要这个功能
3. **实现建议**：如果有的话，提供实现思路
4. **替代方案**：考虑过的其他方案

### 改进文档

文档改进包括：

- 修正拼写错误
- 改进文档结构
- 添加缺失的文档
- 更新过时的信息

---

## 开发流程

### 1. Fork 仓库

```bash
# 在 GitHub 上 Fork 仓库
# 然后克隆您的 Fork
git clone https://github.com/YOUR_USERNAME/toki.git
cd toki
```

### 2. 添加上游仓库

```bash
git remote add upstream https://github.com/toki-project/toki.git
```

### 3. 创建分支

```bash
# 更新主分支
git checkout main
git pull upstream main

# 创建特性分支
git checkout -b feature/your-feature-name
```

### 4. 进行开发

```bash
# 安装依赖
cargo build

# 运行测试
cargo test

# 进行开发...
```

### 5. 提交更改

```bash
git add .
git commit -m "feat: add some feature"
```

### 6. 推送到 Fork

```bash
git push origin feature/your-feature-name
```

### 7. 创建 Pull Request

在 GitHub 上创建 Pull Request。

---

## 代码规范

### Rust 代码规范

我们使用标准的 Rust 代码规范：

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy
```

### 代码组织

- 每个模块应该有清晰的职责
- 使用 `mod.rs` 组织模块
- 公共 API 应该有文档注释
- 复杂逻辑应该有注释说明

### 文档注释

```rust
/// 计算区块哈希
///
/// # Arguments
///
/// * `block` - 要计算哈希的区块
///
/// # Returns
///
/// 返回区块的 SHA-256 哈希值
///
/// # Example
///
/// ```
/// let block = Block::default();
/// let hash = calculate_block_hash(&block);
/// ```
pub fn calculate_block_hash(block: &Block) -> Hash {
    // ...
}
```

### 错误处理

使用 `anyhow::Result` 进行错误处理：

```rust
use anyhow::{Result, Context};

pub fn some_function() -> Result<()> {
    let data = read_file()
        .context("Failed to read file")?;
    Ok(())
}
```

### 测试

- 每个公共函数应该有单元测试
- 测试应该覆盖正常情况和边界情况
- 使用 `#[cfg(test)]` 模块

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // 测试代码
    }
}
```

---

## 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 提交格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 类型 (type)

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建过程或辅助工具
- `ci`: CI 配置
- `revert`: 回退提交

### 范围 (scope)

可选的范围，表示影响的模块：

- `core`: 核心模块
- `storage`: 存储模块
- `consensus`: 共识模块
- `network`: 网络模块
- `ai`: AI 模块
- `api`: API 模块
- `node`: 节点模块

### 示例

```bash
# 新功能
git commit -m "feat(ai): add theta calculation"

# Bug 修复
git commit -m "fix(core): fix block hash calculation"

# 文档更新
git commit -m "docs(api): update API documentation"

# 重构
git commit -m "refactor(storage): optimize database queries"

# 性能优化
git commit -m "perf(consensus): improve mining performance"
```

---

## Pull Request 流程

### 1. 准备工作

- 确保代码通过所有测试
- 确保代码格式正确
- 确保没有 Clippy 警告
- 更新相关文档

```bash
# 运行测试
cargo test

# 格式化代码
cargo fmt

# 检查代码
cargo clippy
```

### 2. 创建 PR

PR 标题应该遵循提交规范：

```
feat(ai): add theta calculation
```

PR 描述应该包含：

1. **更改说明**：详细描述更改内容
2. **相关 Issue**：链接相关 Issue
3. **测试说明**：如何测试这些更改
4. **截图**：如果适用，添加截图

### 3. PR 模板

```markdown
## 更改说明

[描述您的更改]

## 相关 Issue

Closes #123

## 测试说明

[如何测试这些更改]

## 检查清单

- [ ] 代码通过所有测试
- [ ] 代码格式正确
- [ ] 没有 Clippy 警告
- [ ] 更新了相关文档
- [ ] 遵循提交规范
```

### 4. 代码审查

- 响应审查意见
- 进行必要的修改
- 保持讨论友好和专业

### 5. 合并

PR 需要至少一个审查者的批准才能合并。

---

## 开发环境设置

### 必需工具

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装组件
rustup component add clippy rustfmt

# 安装 cargo-expand（可选）
cargo install cargo-expand
```

### IDE 配置

推荐使用 VSCode + rust-analyzer：

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all"
}
```

---

## 获取帮助

如果您有任何问题，可以：

- 在 GitHub 上创建 Issue
- 加入 Discord 讨论
- 发送邮件到 dev@toki.network

---

## 许可证

通过贡献代码，您同意您的贡献将根据项目的 MIT/Apache-2.0 双重许可进行许可。

---

感谢您的贡献！🎉
