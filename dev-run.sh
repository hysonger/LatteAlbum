#!/bin/bash
# dev-run.sh - 快速启动后端服务用于开发调试

# -----------------------------------------------------------------------------
# 服务配置
# -----------------------------------------------------------------------------
export SERVER_PORT=8080

# -----------------------------------------------------------------------------
# 文件路径配置 (使用当前目录下的子目录)
# -----------------------------------------------------------------------------
# 照片目录
export ALBUM_BASE_PATH=./photos

# 缓存目录
export ALBUM_CACHE_DIR=./cache

# 数据库文件
export ALBUM_DB_PATH=./database.db

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 检查必要目录
mkdir -p "${ALBUM_CACHE_DIR:-./cache}"
mkdir -p "${ALBUM_BASE_PATH:-./photos}"

# 直接运行（开发模式，修改代码后可快速重启）
exec mvn spring-boot:run -Dspring-boot.run.jvmArguments="-Xmx1g -Xms256m"
