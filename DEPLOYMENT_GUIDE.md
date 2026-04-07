# Toki 部署指南

## 目录

1. [系统要求](#系统要求)
2. [快速开始](#快速开始)
3. [配置说明](#配置说明)
4. [部署方式](#部署方式)
5. [监控和日志](#监控和日志)
6. [故障排除](#故障排除)

---

## 系统要求

### 硬件要求

**最低配置：**
- CPU: 4 核
- 内存: 8 GB
- 存储: 100 GB SSD
- 网络: 10 Mbps

**推荐配置：**
- CPU: 8 核
- 内存: 16 GB
- 存储: 500 GB NVMe SSD
- 网络: 100 Mbps

### 软件要求

- 操作系统: Linux (Ubuntu 22.04+), macOS, Windows
- Rust: 1.75+
- Git: 2.0+

---

## 快速开始

### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. 克隆代码

```bash
git clone https://github.com/toki-project/toki.git
cd toki
```

### 3. 编译项目

```bash
cargo build --release
```

### 4. 运行节点

```bash
./target/release/toki-node
```

节点将在以下端口启动：
- P2P 网络: 30333
- API 服务: 8080

### 5. 验证运行

```bash
curl http://localhost:8080/health
```

预期响应：
```json
{"success":true,"data":{"healthy":true}}
```

---

## 配置说明

### 配置文件

创建配置文件 `config.toml`：

```toml
# 数据目录
data_dir = "./data"

# 网络配置
[network]
listen_addr = "0.0.0.0:30333"
bootstrap_peers = []
max_connections = 50

# API 配置
[api]
listen_addr = "0.0.0.0:8080"
enable_cors = true

# 共识配置
[consensus]
enable_mining = true
mining_threads = 6

# AI 配置
[ai]
enable = true
distribution_interval = 1000  # 每 1000 个区块分发一次
theta_calculation_interval = 100
```

### 环境变量

也可以通过环境变量配置：

```bash
export TOKI_DATA_DIR="./data"
export TOKI_NETWORK_LISTEN="0.0.0.0:30333"
export TOKI_API_LISTEN="0.0.0.0:8080"
export TOKI_MINING=true
export TOKI_MINING_THREADS=6
```

### 命令行参数

```bash
./target/release/toki-node \
  --data-dir ./data \
  --network-listen 0.0.0.0:30333 \
  --api-listen 0.0.0.0:8080 \
  --enable-mining \
  --mining-threads 6
```

---

## 部署方式

### 方式 1: 直接运行

适合开发和测试环境。

```bash
# 编译
cargo build --release

# 运行
./target/release/toki-node --config config.toml
```

### 方式 2: Systemd 服务

适合 Linux 生产环境。

**创建服务文件：**

```bash
sudo nano /etc/systemd/system/toki.service
```

**内容：**

```ini
[Unit]
Description=Toki Blockchain Node
After=network.target

[Service]
Type=simple
User=toki
Group=toki
WorkingDirectory=/opt/toki
ExecStart=/opt/toki/toki-node --config /opt/toki/config.toml
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

**启动服务：**

```bash
sudo systemctl daemon-reload
sudo systemctl enable toki
sudo systemctl start toki
sudo systemctl status toki
```

### 方式 3: Docker

适合容器化部署。

**构建镜像：**

```bash
docker build -t toki-node:latest .
```

**运行容器：**

```bash
docker run -d \
  --name toki-node \
  -p 30333:30333 \
  -p 8080:8080 \
  -v /data/toki:/data \
  toki-node:latest
```

**Docker Compose：**

```yaml
version: '3.8'

services:
  toki-node:
    image: toki-node:latest
    container_name: toki-node
    ports:
      - "30333:30333"
      - "8080:8080"
    volumes:
      - ./data:/data
      - ./config.toml:/app/config.toml
    restart: unless-stopped
    environment:
      - RUST_LOG=info
```

### 方式 4: Kubernetes

适合大规模部署。

**Deployment:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: toki-node
spec:
  replicas: 1
  selector:
    matchLabels:
      app: toki-node
  template:
    metadata:
      labels:
        app: toki-node
    spec:
      containers:
      - name: toki-node
        image: toki-node:latest
        ports:
        - containerPort: 30333
        - containerPort: 8080
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "8Gi"
            cpu: "4"
          limits:
            memory: "16Gi"
            cpu: "8"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: toki-data
```

**Service:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: toki-node
spec:
  type: LoadBalancer
  ports:
  - port: 30333
    targetPort: 30333
    name: p2p
  - port: 8080
    targetPort: 8080
    name: api
  selector:
    app: toki-node
```

---

## 监控和日志

### 日志配置

**设置日志级别：**

```bash
export RUST_LOG=info
export RUST_LOG=toki=debug
```

**日志文件：**

```bash
./target/release/toki-node 2>&1 | tee toki.log
```

### Prometheus 监控

**添加 metrics 端点（计划中）：**

```bash
curl http://localhost:8080/metrics
```

**Prometheus 配置：**

```yaml
scrape_configs:
  - job_name: 'toki'
    static_configs:
      - targets: ['localhost:8080']
```

### Grafana 仪表板

导入 Grafana 仪表板监控：
- 区块高度
- 交易数量
- 挖矿哈希率
- 内存使用
- CPU 使用

---

## 故障排除

### 常见问题

#### 1. 端口被占用

**错误：** `Address already in use`

**解决：**
```bash
# 检查端口占用
lsof -i :8080
lsof -i :30333

# 更改端口
./target/release/toki-node --api-listen 0.0.0.0:8081
```

#### 2. 数据库损坏

**错误：** `Database corrupted`

**解决：**
```bash
# 停止节点
sudo systemctl stop toki

# 删除数据目录
rm -rf ./data

# 重新启动
sudo systemctl start toki
```

#### 3. 内存不足

**错误：** `Out of memory`

**解决：**
```bash
# 减少挖矿线程
./target/release/toki-node --mining-threads 2

# 或增加系统内存
```

#### 4. 同步失败

**错误：** `Sync failed`

**解决：**
```bash
# 检查网络连接
ping <peer-ip>

# 检查种子节点
curl http://localhost:8080/api/v1/network/status
```

### 性能优化

#### 1. 系统调优

```bash
# 增加文件描述符限制
ulimit -n 65536

# 优化内核参数
sudo sysctl -w net.core.somaxconn=65535
sudo sysctl -w net.ipv4.tcp_max_syn_backlog=65535
```

#### 2. Rust 优化

```bash
# 使用 release 模式编译
cargo build --release

# 启用 LTO
export RUSTFLAGS="-C lto=fat"
cargo build --release
```

#### 3. 数据库优化

```bash
# 使用 SSD 存储
# 定期压缩数据库
curl -X POST http://localhost:8080/api/v1/admin/compact
```

---

## 安全建议

### 1. 防火墙配置

```bash
# 开放必要端口
sudo ufw allow 30333/tcp  # P2P
sudo ufw allow 8080/tcp   # API

# 如果 API 只本地访问
sudo ufw deny 8080/tcp
```

### 2. API 安全

```bash
# 使用 HTTPS
# 添加认证中间件
# 限制 API 访问频率
```

### 3. 数据备份

```bash
# 定期备份数据目录
rsync -avz ./data /backup/toki-$(date +%Y%m%d)
```

---

## 升级指南

### 1. 备份数据

```bash
cp -r ./data ./data-backup
```

### 2. 停止服务

```bash
sudo systemctl stop toki
```

### 3. 更新代码

```bash
git pull origin main
cargo build --release
```

### 4. 启动服务

```bash
sudo systemctl start toki
```

### 5. 验证升级

```bash
curl http://localhost:8080/api/v1/node/info
```

---

## 多节点部署

### 1. 启动种子节点

```bash
./target/release/toki-node \
  --network-listen 0.0.0.0:30333 \
  --api-listen 0.0.0.0:8080
```

### 2. 启动第二个节点

```bash
./target/release/toki-node \
  --network-listen 0.0.0.0:30334 \
  --api-listen 0.0.0.0:8081 \
  --bootstrap-peers /ip4/127.0.0.1/tcp/30333
```

### 3. 验证连接

```bash
curl http://localhost:8081/api/v1/node/info
```

---

## 支持

如有问题，请：
1. 查看日志文件
2. 检查配置文件
3. 提交 Issue: https://github.com/toki-project/toki/issues
