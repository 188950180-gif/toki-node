# 多阶段构建
# 构建阶段
FROM rust:latest AS builder

WORKDIR /app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 复制所有子模块
COPY core ./core
COPY crypto ./crypto
COPY storage ./storage
COPY consensus ./consensus
COPY network ./network
COPY ai ./ai
COPY governance ./governance
COPY api ./api
COPY node ./node
COPY developer ./developer

# 编译 release 版本
RUN cargo build --release

# 运行阶段
FROM ubuntu:22.04

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/toki-node /usr/local/bin/toki-node

# 复制配置文件和创世文件
COPY config.toml /app/config.toml
COPY genesis.json /app/genesis.json

# 创建数据目录（不需要授权给特定用户，因为以 root 运行）
RUN mkdir -p /data
RUN chmod 777 /data

# 暴露端口
EXPOSE 30333 8080

# 数据目录卷
VOLUME ["/data"]

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# 以 root 用户启动节点（不再切换用户）
CMD ["toki-node", "start", "--config", "/app/config.toml", "--data-dir", "/data"]