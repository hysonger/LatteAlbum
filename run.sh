#!/bin/bash
# Latte Album 服务启动脚本

set -e

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 查找 JAR 文件
JAR_FILE=$(find . -maxdepth 1 -name "latte-album-*.jar" | head -n 1)

if [ -z "$JAR_FILE" ]; then
    echo -e "${RED}Error: 未找到 JAR 文件${NC}" >&2
    exit 1
fi

# 检测 Java
if [ -z "$JAVA_HOME" ]; then
    if command -v java &> /dev/null; then
        JAVA_CMD=$(which java)
    else
        echo -e "${RED}Error: 未找到 Java，请设置 JAVA_HOME 环境变量${NC}" >&2
        exit 1
    fi
else
    JAVA_CMD="$JAVA_HOME/bin/java"
    if [ ! -f "$JAVA_CMD" ]; then
        echo -e "${RED}Error: JAVA_HOME 指向的路径无效: $JAVA_HOME${NC}" >&2
        exit 1
    fi
fi

# 检查 Java 版本
JAVA_VERSION=$($JAVA_CMD -version 2>&1 | head -n 1 | cut -d'"' -f2 | sed '/^1\./s///' | cut -d'.' -f1)
if [ "$JAVA_VERSION" -lt 17 ]; then
    echo -e "${RED}Error: 需要 Java 17 或更高版本，当前版本: $JAVA_VERSION${NC}" >&2
    exit 1
fi

# 加载 .env 文件（如果存在），保持与 setup.sh 的一致性
if [ -f ".env" ]; then
    set -a
    source .env
    set +a
fi

# 默认 JVM 参数（如果未从 .env 加载）
JAVA_OPTS=${JAVA_OPTS:-"-Xmx2g -Xms512m"}

# 默认环境变量
SERVER_PORT=${SERVER_PORT:-8080}
SERVER_ADDRESS=${SERVER_ADDRESS:-0.0.0.0}
ALBUM_BASE_PATH=${ALBUM_BASE_PATH:-./photos}
ALBUM_CACHE_DIR=${ALBUM_CACHE_DIR:-./cache}
ALBUM_DB_PATH=${ALBUM_DB_PATH:-./data/db/database.db}

# 创建必要的目录
mkdir -p "$(dirname "$ALBUM_DB_PATH")"
mkdir -p "$ALBUM_CACHE_DIR"
mkdir -p logs

# 导出环境变量供应用使用
export SERVER_PORT
export SERVER_ADDRESS
export ALBUM_BASE_PATH
export ALBUM_CACHE_DIR
export ALBUM_DB_PATH

# 启动应用
echo -e "${GREEN}启动 Latte Album...${NC}"
echo "  JAR: $JAR_FILE"
echo "  Java: $JAVA_CMD"
echo "  JVM Options: $JAVA_OPTS"
echo "  Server Address: $SERVER_ADDRESS"
echo "  Server Port: $SERVER_PORT"
echo "  Base Path: $ALBUM_BASE_PATH"
echo "  Cache Dir: $ALBUM_CACHE_DIR"
echo "  DB Path: $ALBUM_DB_PATH"
echo ""

exec $JAVA_CMD $JAVA_OPTS -jar "$JAR_FILE"
