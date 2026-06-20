#!/bin/bash

# 环境变量注入，启动 Rust 后端服务

cd "$(dirname "$0")"

# 加载环境变量配置
source .env.develop

# 创建必要目录
mkdir -p "$(dirname "$LATTE_DB_PATH")"
mkdir -p "$LATTE_CACHE_DIR"

cargo run 2>&1 | tee "output.log"
