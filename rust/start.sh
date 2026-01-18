#!/bin/bash

# 环境变量注入，启动 Rust 后端服务

cd "$(dirname "$0")"

# 加载环境变量配置
source .env.test

# 设置默认值（如果环境变量未设置）
source .env.default

# 创建必要目录
mkdir -p "$(dirname "$LATTE_DB_PATH")"
mkdir -p "$LATTE_CACHE_DIR"

./cargo-with-vendor.sh run 2>&1 | tee "output.log"
