#!/bin/bash
set -e

# =============================================================================
# Latte Album 轻量级部署脚本
# =============================================================================
# 功能：构建、打包并上传到远程服务器
# 用法：
#   1. 复制 .env.lite.example 为 .env.lite 并配置
#   2. 运行 ./deploy-lite.sh [选项]
#
# 命令行参数（可选，会覆盖配置文件）：
#   --host REMOTE_HOST      远程服务器地址
#   --user REMOTE_USER      远程用户名
#   --port REMOTE_PORT      远程SSH端口（默认22）
#   --path REMOTE_PATH      部署路径（默认/opt/latte-album）
#   --service-user USER     服务用户（默认latte）
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}Latte Album 轻量级部署脚本${NC}"
echo "=========================================="

# -----------------------------------------------------------------------------
# 解析命令行参数
# -----------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case $1 in
        --host)
            REMOTE_HOST="$2"
            shift 2
            ;;
        --user)
            REMOTE_USER="$2"
            shift 2
            ;;
        --port)
            REMOTE_PORT="$2"
            shift 2
            ;;
        --path)
            REMOTE_PATH="$2"
            shift 2
            ;;
        --service-user)
            SERVICE_USER="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            exit 1
            ;;
    esac
done

# -----------------------------------------------------------------------------
# 加载配置文件
# -----------------------------------------------------------------------------
if [ -f .env.lite ]; then
    echo -e "${YELLOW}加载配置文件 .env.lite...${NC}"
    set -a
    source .env.lite
    set +a
fi

# -----------------------------------------------------------------------------
# 验证必需参数
# -----------------------------------------------------------------------------
if [ -z "$REMOTE_HOST" ]; then
    echo -e "${RED}Error: REMOTE_HOST 未配置${NC}"
    echo "请设置 .env.lite 文件或使用 --host 参数"
    exit 1
fi

if [ -z "$REMOTE_USER" ]; then
    echo -e "${RED}Error: REMOTE_USER 未配置${NC}"
    echo "请设置 .env.lite 文件或使用 --user 参数"
    exit 1
fi

# 设置默认值
REMOTE_PORT=${REMOTE_PORT:-22}
REMOTE_PATH=${REMOTE_PATH:-/opt/latte-album}
SERVICE_USER=${SERVICE_USER:-latte}
SERVER_PORT=${SERVER_PORT:-8080}

echo -e "\n${YELLOW}部署配置:${NC}"
echo "  远程主机: ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PORT}"
echo "  部署路径: ${REMOTE_PATH}"
echo "  服务用户: ${SERVICE_USER}"
echo "  服务端口: ${SERVER_PORT}"

# -----------------------------------------------------------------------------
# 检查前置条件
# -----------------------------------------------------------------------------
echo -e "\n${YELLOW}检查前置条件...${NC}"

if ! command -v mvn &> /dev/null; then
    echo -e "${RED}Error: Maven 未安装${NC}"
    exit 1
fi
echo -e "${GREEN}[OK] Maven 已安装${NC}"

if ! command -v npm &> /dev/null; then
    echo -e "${RED}Error: Node.js/npm 未安装${NC}"
    exit 1
fi
echo -e "${GREEN}[OK] Node.js/npm 已安装${NC}"

if ! command -v scp &> /dev/null; then
    echo -e "${RED}Error: scp 未安装${NC}"
    exit 1
fi
echo -e "${GREEN}[OK] scp 已安装${NC}"

# -----------------------------------------------------------------------------
# 步骤 1: 构建前端
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[1/4] 构建前端...${NC}"

cd frontend

if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}安装前端依赖...${NC}"
    npm install
fi

echo -e "${YELLOW}构建前端...${NC}"
npm run build

if [ ! -d "dist" ]; then
    echo -e "${RED}Error: 前端构建失败，未找到 dist 目录${NC}"
    exit 1
fi

echo -e "${GREEN}[OK] 前端构建完成${NC}"

# -----------------------------------------------------------------------------
# 步骤 2: 复制前端构建产物到 Spring Boot 静态资源目录
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[2/4] 复制前端构建产物...${NC}"

cd ..
mkdir -p src/main/resources/static
rm -rf src/main/resources/static/*
cp -r frontend/dist/* src/main/resources/static/

echo -e "${GREEN}[OK] 前端资源已复制${NC}"

# -----------------------------------------------------------------------------
# 步骤 3: 构建 JAR
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[3/4] 构建 JAR 文件...${NC}"

mvn clean package -DskipTests

JAR_FILE=$(find target -name "latte-album-*.jar" ! -name "*-sources.jar" | head -n 1)

if [ -z "$JAR_FILE" ]; then
    echo -e "${RED}Error: 未找到 JAR 文件${NC}"
    exit 1
fi

echo -e "${GREEN}[OK] JAR 构建完成: $JAR_FILE${NC}"

# -----------------------------------------------------------------------------
# 步骤 4: 打包并上传
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[4/4] 打包并上传...${NC}"

# 创建临时打包目录
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

PACKAGE_NAME="latte-album-$(date +%Y%m%d-%H%M%S).tar.gz"

echo -e "${YELLOW}准备打包文件...${NC}"

# 复制 JAR 文件
cp "$JAR_FILE" "$TEMP_DIR/"

# 复制启动脚本
cp run.sh "$TEMP_DIR/"
chmod +x "$TEMP_DIR/run.sh"

# 复制 systemd 服务文件
cp latte-album.service "$TEMP_DIR/"

# 如果存在自定义 application.yml，也打包进去
if [ -f "application.yml.custom" ]; then
    cp application.yml.custom "$TEMP_DIR/application.yml"
    echo -e "${YELLOW}已包含自定义 application.yml${NC}"
fi

# 添加 uninstall.sh 到打包目录（如果存在）
if [ -f "uninstall.sh" ]; then
    cp uninstall.sh "$TEMP_DIR/"
    chmod +x "$TEMP_DIR/uninstall.sh"
    echo -e "${YELLOW}已包含 uninstall.sh${NC}"
fi

# 创建并添加运行时配置文件（从 .env.lite 提取运行时配置）
if [ -f .env.lite ]; then
    echo -e "${YELLOW}提取运行时配置...${NC}"
    RUNTIME_ENV="$TEMP_DIR/.env.runtime"
    # 提取运行时需要的配置项（排除部署相关的配置）
    grep -E "^(SERVER_PORT|SERVER_ADDRESS|SERVICE_USER|ALBUM_BASE_PATH|ALBUM_CACHE_DIR|ALBUM_DB_PATH|JAVA_OPTS)=" .env.lite > "$RUNTIME_ENV" 2>/dev/null || true
    # 如果文件不为空，添加到打包目录
    if [ -s "$RUNTIME_ENV" ]; then
        echo -e "${GREEN}[OK] 运行时配置已提取${NC}"
    else
        rm -f "$RUNTIME_ENV"
    fi
fi

# 创建打包文件
cd "$TEMP_DIR"
tar -czf "$PACKAGE_NAME" * .env.runtime
cd - > /dev/null

echo -e "${YELLOW}上传到远程服务器...${NC}"

# 上传到用户主目录的临时位置（setup.sh 会负责创建最终部署目录并移动文件）
# 注意：这里只创建临时存储目录，不创建部署路径（REMOTE_PATH），由 setup.sh 负责创建部署路径
REMOTE_TEMP_DIR="~/latte-album-deploy"

# 创建临时目录（仅用于文件上传，不是最终部署路径）
ssh -p "$REMOTE_PORT" "${REMOTE_USER}@${REMOTE_HOST}" "mkdir -p ${REMOTE_TEMP_DIR}"

# 合并上传：同时上传部署包和 setup.sh
UPLOAD_FILES=("${TEMP_DIR}/${PACKAGE_NAME}")
if [ -f "setup.sh" ]; then
    UPLOAD_FILES+=("setup.sh")
fi

# 一次性上传所有文件到临时目录
scp -P "$REMOTE_PORT" "${UPLOAD_FILES[@]}" "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_TEMP_DIR}/"

echo -e "${GREEN}[OK] 上传完成${NC}"

# -----------------------------------------------------------------------------
# 完成
# -----------------------------------------------------------------------------
echo ""
echo "=========================================="
echo -e "${GREEN}部署包已上传完成!${NC}"
echo ""
echo -e "${YELLOW}下一步操作:${NC}"
echo ""
echo "  1. SSH 登录到远程服务器:"
echo "     ${YELLOW}ssh -p ${REMOTE_PORT} ${REMOTE_USER}@${REMOTE_HOST}${NC}"
echo ""
echo "  2. 运行安装脚本（会自动创建部署目录并解压部署包）:"
echo "     ${YELLOW}cd ~/latte-album-deploy${NC}"
echo "     ${YELLOW}sudo INSTALL_DIR=${REMOTE_PATH} ./setup.sh${NC}"
echo ""
echo -e "${YELLOW}或者直接运行:${NC}"
echo "     ${YELLOW}ssh -p ${REMOTE_PORT} ${REMOTE_USER}@${REMOTE_HOST} 'cd ~/latte-album-deploy && sudo INSTALL_DIR=${REMOTE_PATH} ./setup.sh'${NC}"
