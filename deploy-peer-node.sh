#!/bin/bash
# Toki 普通节点部署脚本
# 连接到种子节点: 182.254.176.30
# 系统: Ubuntu Server 24.04 LTS 64bit

set -e

echo "=========================================="
echo "  Toki 普通节点部署"
echo "  种子节点: 182.254.176.30"
echo "=========================================="
echo ""

# 配置
VERSION="0.1.0"
INSTALL_DIR="/opt/toki"
DATA_DIR="/opt/toki/data"
SEED_IP="182.254.176.30"
SEED_PORT="30333"

# 获取种子节点 PeerId
echo "请输入种子节点的 PeerId（从种子节点日志中获取）:"
echo "格式示例: 12D3KooW..."
read -p "PeerId: " SEED_PEER_ID

if [ -z "$SEED_PEER_ID" ]; then
    echo "错误: PeerId 不能为空"
    exit 1
fi

# 检查 root 权限
if [ "$EUID" -ne 0 ]; then
    echo "错误: 需要 root 权限"
    exit 1
fi

# 1. 安装依赖
echo ""
echo "1. 安装依赖..."
apt-get update
apt-get install -y curl build-essential

# 2. 检查 Rust
echo ""
echo "2. 检查 Rust..."
if ! command -v cargo &> /dev/null; then
    echo "安装 Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 3. 创建目录
echo ""
echo "3. 创建目录..."
mkdir -p $INSTALL_DIR
mkdir -p $DATA_DIR
mkdir -p $INSTALL_DIR/backups
mkdir -p $INSTALL_DIR/logs

# 4. 编译项目
echo ""
echo "4. 编译项目..."
cd /opt/toki
cargo build --release

# 5. 创建配置文件
echo ""
echo "5. 创建配置文件..."
cat > $INSTALL_DIR/config.toml << EOF
data_dir = "/opt/toki/data"
backup_path = "/opt/toki/backups"
log_level = "info"

[network]
listen_addr = "/ip4/0.0.0.0/tcp/30333"
bootstrap_peers = [
    "/ip4/$SEED_IP/tcp/$SEED_PORT/p2p/$SEED_PEER_ID"
]
max_connections = 100
enable_upnp = false

[consensus]
enable_mining = true
miner_address = ""
target_block_time = 10

[api]
listen_addr = "0.0.0.0:8080"
enable_ws = true
EOF

# 6. 复制可执行文件
echo ""
echo "6. 安装节点程序..."
cp target/release/toki-node $INSTALL_DIR/

# 7. 创建 systemd 服务
echo ""
echo "7. 创建 systemd 服务..."
cat > /etc/systemd/system/toki-node.service << EOF
[Unit]
Description=Toki Node
After=network.target

[Service]
Type=simple
User=toki
Group=toki
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/toki-node start --config $INSTALL_DIR/config.toml
Restart=always
RestartSec=10
LimitNOFILE=65536

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_DIR

StandardOutput=journal
StandardError=journal
SyslogIdentifier=toki-node

[Install]
WantedBy=multi-user.target
EOF

# 8. 创建 toki 用户
echo ""
echo "8. 创建 toki 用户..."
if ! id "toki" &>/dev/null; then
    useradd -r -s /bin/false toki
fi
chown -R toki:toki $INSTALL_DIR

# 9. 配置防火墙
echo ""
echo "9. 配置防火墙..."
if command -v ufw &> /dev/null; then
    ufw allow 30333/tcp comment 'Toki P2P
    ufw allow 8080/tcp comment 'Toki API
    echo "防火墙规则已添加"
fi

# 10. 启动服务
echo ""
echo "10. 启动服务..."
systemctl daemon-reload
systemctl enable toki-node
systemctl start toki-node

# 11. 显示状态
echo ""
echo "=========================================="
echo "  部署完成"
echo "=========================================="
echo ""
echo "节点状态:"
systemctl status toki-node --no-pager

echo ""
echo "查看日志:"
echo "  journalctl -u toki-node -f"
echo ""
