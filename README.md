# Toki 区块链

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/toki-project/toki)

**一个基于 Rust 的 AI 驱动经济区块链系统**

[English](README_EN.md) | 简体中文

</div>

---

## 📖 目录

- [简介](#简介)
- [特性](#特性)
- [快速开始](#快速开始)
- [架构](#架构)
- [API 文档](#api-文档)
- [部署](#部署)
- [开发](#开发)
- [路线图](#路线图)
- [贡献](#贡献)
- [许可证](#许可证)

---

## 简介

Toki 是一个创新的区块链项目，旨在通过 AI 技术实现经济系统的自动化管理。它结合了区块链的去中心化特性和人工智能的智能决策能力，创建一个更加公平、透明的经济体系。

### 核心理念

- **AI 驱动经济**：通过 AI 算法自动调节 Token 分发、计算 Theta 值、执行公益项目
- **平权机制**：自动检测和调节贫富差距，实现经济平等
- **公益透明**：公益资金的使用完全透明，由 AI 自动执行
- **开发者权益保障**：完善的密钥轮换和加密存储机制

---

## 特性

### 核心功能

- ✅ **PoW 共识**：多线程工作量证明挖矿
- ✅ **Ring Signature**：环签名隐私保护
- ✅ **Sled 存储**：高性能嵌入式数据库
- ✅ **RESTful API**：完整的 HTTP API 接口
- ✅ **AI 经济系统**：自动化 Token 分发和经济调节

### AI 经济系统

- ✅ **Token 分发**：自动向符合条件的账户分发 Token
- ✅ **Theta 计算**：计算经济不平等指数
- ✅ **公益执行**：自动执行公益项目资金分配
- ✅ **平权检查**：检测并调节贫富差距
- ✅ **福利分配**：自动分配福利资金

### 技术特性

- 🚀 **高性能**：Rust 实现，接近 C++ 性能
- 🔒 **安全**：内存安全，无数据竞争
- 🌐 **分布式**：P2P 网络（开发中）
- 📊 **可扩展**：模块化设计，易于扩展

---

## 快速开始

### 系统要求

- Rust 1.75+
- 8 GB+ 内存
- 100 GB+ 存储空间

### 安装

```bash
# 克隆代码
git clone https://github.com/toki-project/toki.git
cd toki

# 编译
cargo build --release

# 运行
./target/release/toki-node
```

### Docker 部署

```bash
# 使用 Docker Compose
docker-compose up -d

# 查看日志
docker-compose logs -f
```

### 验证运行

```bash
# 健康检查
curl http://localhost:8080/health

# 获取节点信息
curl http://localhost:8080/api/v1/node/info
```

---

## 架构

```
Toki 区块链架构
├── core          # 核心数据结构（区块、交易、账户）
├── crypto        # 密码学（环签名、哈希）
├── storage       # 存储（Sled 数据库）
├── consensus     # 共识（PoW 挖矿）
├── network       # P2P 网络（libp2p）
├── ai            # AI 经济系统
├── governance    # 治理（密钥轮换）
├── api           # REST API
└── node          # 节点启动器
```

### 模块说明

| 模块 | 功能 | 状态 |
|------|------|------|
| core | 区块、交易、账户 | ✅ |
| crypto | 环签名、哈希 | ✅ |
| storage | 数据存储 | ✅ |
| consensus | PoW 共识 | ✅ |
| network | P2P 网络 | ⚠️ |
| ai | AI 经济系统 | ✅ |
| governance | 治理机制 | ✅ |
| api | REST API | ✅ |
| node | 节点启动 | ✅ |

---

## API 文档

完整的 API 文档请查看 [API_DOCUMENTATION.md](API_DOCUMENTATION.md)

### 主要端点

```bash
# 健康检查
GET /health

# 节点信息
GET /api/v1/node/info

# 获取区块
GET /api/v1/block/{height}

# 提交交易
POST /api/v1/transaction/submit

# 账户余额
GET /api/v1/account/balance/{address}

# AI 状态
GET /api/v1/ai/status
```

---

## 部署

详细的部署指南请查看 [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md)

### 部署方式

1. **直接运行**：适合开发测试
2. **Systemd 服务**：适合 Linux 生产环境
3. **Docker**：适合容器化部署
4. **Kubernetes**：适合大规模部署

### 配置文件

参考 [config.example.toml](config.example.toml) 创建配置文件。

---

## 开发

### 构建

```bash
# Debug 模式
cargo build

# Release 模式
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化
cargo fmt
```

### 项目结构

```
toki/
├── core/              # 核心模块
│   ├── block.rs       # 区块结构
│   ├── transaction.rs # 交易结构
│   └── account.rs     # 账户管理
├── storage/           # 存储模块
│   ├── block_store.rs # 区块存储
│   └── account_store.rs # 账户存储
├── consensus/         # 共识模块
│   ├── pow.rs         # PoW 挖矿
│   └── miner.rs       # 挖矿器
├── ai/                # AI 模块
│   ├── scheduler.rs   # 任务调度
│   ├── distribute.rs  # Token 分发
│   ├── theta.rs       # Theta 计算
│   └── charity.rs     # 公益执行
└── api/               # API 模块
    ├── routes.rs      # 路由定义
    └── handlers.rs    # 请求处理
```

---

## 路线图

### v0.1.0 (当前版本)

- ✅ 核心区块链功能
- ✅ PoW 共识机制
- ✅ REST API
- ✅ AI 经济系统框架
- ✅ 单节点运行

### v0.2.0 (计划中)

- ⚠️ P2P 网络实现
- ⚠️ 多节点同步
- ⚠️ 区块广播
- ⚠️ 交易池同步

### v0.3.0 (未来)

- 📋 Web 钱包
- 📋 移动钱包
- 📋 浏览器扩展
- 📋 DApp 支持

### v1.0.0 (长期目标)

- 📋 主网启动
- 📋 交易所集成
- 📋 跨链桥接
- 📋 生态系统完善

---

## 贡献

我们欢迎所有形式的贡献！

### 贡献方式

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

### 贡献指南

详细的贡献指南请查看 [CONTRIBUTING.md](CONTRIBUTING.md)

### 行为准则

请遵守我们的 [行为准则](CODE_OF_CONDUCT.md)

---

## 社区

- **GitHub**: https://github.com/toki-project/toki
- **文档**: https://docs.toki.network
- **博客**: https://blog.toki.network
- **Discord**: https://discord.gg/toki
- **Twitter**: https://twitter.com/toki_network

---

## 许可证

本项目采用双重许可：

- MIT 许可证 ([LICENSE-MIT](LICENSE-MIT))
- Apache 许可证 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

您可以选择其中任一许可证。

---

## 致谢

感谢以下项目和社区：

- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [libp2p](https://libp2p.io/) - P2P 网络库
- [Sled](https://sled.rs/) - 嵌入式数据库
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Tokio](https://tokio.rs/) - 异步运行时

---

## 联系方式

- **项目维护者**: Toki Development Team
- **邮箱**: dev@toki.network
- **网站**: https://toki.network

---

<div align="center">

**如果这个项目对您有帮助，请给一个 ⭐️ Star！**

Made with ❤️ by Toki Development Team

</div>
