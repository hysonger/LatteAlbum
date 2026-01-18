#!/bin/bash

# 环境变量注入，启动 Rust 后端服务

cd "$(dirname "$0")"

# 加载环境变量配置
source .env.test

# 设置默认值（如果环境变量未设置）
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
export LATTE_HOST="${LATTE_HOST:-0.0.0.0}"
export LATTE_PORT="${LATTE_PORT:-8080}"
export LATTE_BASE_PATH="${LATTE_BASE_PATH:-$(cd .. && pwd)/photos}"
export LATTE_DB_PATH="${LATTE_DB_PATH:-$(pwd)/data/album.db}"
export LATTE_CACHE_DIR="${LATTE_CACHE_DIR:-$(pwd)/cache}"
export LATTE_STATIC_DIR="${LATTE_STATIC_DIR:-$(cd .. && pwd)/frontend/dist}"

# 创建必要目录
mkdir -p "$(dirname "$LATTE_DB_PATH")"
mkdir -p "$LATTE_CACHE_DIR"

cargo run 2>&1 | tee "output.log"
