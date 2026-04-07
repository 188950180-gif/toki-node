# 多阶段构建
# 构建阶段
FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

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

RUN cargo build --release

# 运行阶段
FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/toki-node /usr/local/bin/toki-node
COPY config.toml /app/config.toml
COPY genesis.json /app/genesis.json

RUN mkdir -p /data && chmod 777 /data

EXPOSE 30333 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# 以 root 用户运行（默认就是 root，不需要 USER 指令）
CMD ["toki-node", "start", "--config", "/app/config.toml", "--data-dir", "/data"]