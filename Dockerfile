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

# 创建用户
RUN useradd -m -s /bin/bash toki

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/toki-node /usr/local/bin/toki-node

# 复制配置文件和创世文件
COPY config.toml /app/config.toml
COPY genesis.json /app/genesis.json

# 创建数据目录并授权
RUN mkdir -p /data && chown -R toki:toki /data

# 切换用户
USER toki

# 暴露端口
EXPOSE 30333 8080

# 数据目录卷
VOLUME ["/data"]

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# 启动命令
CMD ["toki-node", "start", "--config", "/app/config.toml", "--data-dir", "/data"]