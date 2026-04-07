#!/bin/bash
# Toki 种子节点部署脚本
# 公网 IP: 182.254.176.30
# 系统: Ubuntu Server 24.04 LTS 64bit

set -e

echo "=========================================="
echo "  Toki 种子节点部署"
echo "  IP: 182.254.176.30"
echo "=========================================="
echo ""

# 配置
VERSION="0.1.0"
INSTALL_DIR="/opt/toki"
DATA_DIR="/opt/toki/data"
CONFIG_FILE="seed-node-config.toml"
SERVICE_FILE="toki-node.service"

# 检查 root 权限
if [ "$EUID" -ne 0 ]; then
    echo "错误: 需要 root 权限"
    exit 1
fi

# 1. 安装依赖
echo "1. 安装依赖..."
apt-get update
apt-get install -y curl build-essential

# 2. 安装 Rust（如果未安装）
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
cd /tmp
if [ -d "tokipt" ]; then
    rm -rf tokipt
fi
# 假设代码已经上传到服务器
# 如果是从 Git 克隆:
# git clone https://github.com/toki-platform/tokipt.git
# cd tokipt

# 或者直接在当前目录编译
cd /opt/toki
cargo build --release

# 5. 复制可执行文件
echo ""
echo "5. 安装节点程序..."
cp target/release/toki-node $INSTALL_DIR/
cp $CONFIG_FILE $INSTALL_DIR/config.toml

# 6. 创建 systemd 服务
echo ""
echo "6. 创建 systemd 服务..."
cat > /etc/systemd/system/toki-node.service << EOF
[Unit]
Description=Toki Seed Node
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

# 安全配置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_DIR

# 日志
StandardOutput=journal
StandardError=journal
SyslogIdentifier=toki-node

[Install]
WantedBy=multi-user.target
EOF

# 7. 创建 toki 用户
echo ""
echo "7. 创建 toki 用户..."
if ! id "toki" &>/dev/null; then
    useradd -r -s /bin/false toki
fi
chown -R toki:toki $INSTALL_DIR

# 8. 配置防火墙
echo ""
echo "8. 配置防火墙..."
if command -v ufw &> /dev/null; then
    ufw allow 30333/tcp comment 'Toki P2P'
    ufw allow 8080/tcp comment 'Toki API'
    echo "防火墙规则已添加"
fi

# 9. 启动服务
echo ""
echo "9. 启动服务..."
systemctl daemon-reload
systemctl enable toki-node
systemctl start toki-node

# 10. 显示状态
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
echo "获取 PeerId:"
echo "  journalctl -u toki-node | grep '本地节点 ID'"
echo ""
