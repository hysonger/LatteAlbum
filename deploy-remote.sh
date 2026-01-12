#!/bin/bash
set -e

# =============================================================================
# Latte Album 本地构建并传输到远程服务器
# =============================================================================
# 用法:
#   1. 复制 .env.remote.example 为 .env.remote 并配置
#   2. 运行 ./deploy-remote.sh
#
# 流程:
#   1. 本地构建镜像
#   2. 将镜像和部署脚本传输到远程服务器
#
# 远程服务器部署 (需要 root 权限):
#   ssh root@<REMOTE_HOST>
#   cd ~/latte-album
#   ./deploy-on-server.sh
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}Latte Album 构建并传输到远程服务器${NC}"
echo "=========================================="

# -----------------------------------------------------------------------------
# 配置检查
# -----------------------------------------------------------------------------
if [ ! -f .env.remote ]; then
    echo -e "${RED}Error: .env.remote 文件不存在${NC}"
    echo -e "请复制 .env.remote.example 为 .env.remote 并配置"
    exit 1
fi

# 加载远程配置
set -a
source .env.remote
set +a

# 验证必要配置
if [ -z "$REMOTE_HOST" ]; then
    echo -e "${RED}Error: REMOTE_HOST 未配置${NC}"
    exit 1
fi

if [ -z "$REMOTE_USER" ]; then
    echo -e "${RED}Error: REMOTE_USER 未配置${NC}"
    exit 1
fi

REMOTE_PORT=${REMOTE_PORT:-22}
IMAGE_NAME=${IMAGE_NAME:-latte-album}
TAG=${TAG:-latest}
REMOTE_SERVER_PORT=${REMOTE_SERVER_PORT:-8080}

echo -e "\n${YELLOW}部署配置:${NC}"
echo "  远程主机: ${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_PORT}"
echo "  镜像名称: ${IMAGE_NAME}:${TAG}"
echo "  照片目录: ${REMOTE_ALBUM_PHOTOS:-未配置}"
echo "  服务端口: ${REMOTE_SERVER_PORT}"

# SSH 和 SCP 命令
ssh_cmd="ssh -p ${REMOTE_PORT}"
scp_cmd="scp -P ${REMOTE_PORT}"

# -----------------------------------------------------------------------------
# 步骤 1: 构建镜像
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[1/3] 构建镜像...${NC}"

docker build -t ${IMAGE_NAME}:${TAG} .

if [ $? -eq 0 ]; then
    echo -e "${GREEN}[OK] 镜像构建成功${NC}"
else
    echo -e "${RED}[FAILED] 镜像构建失败${NC}"
    exit 1
fi

# -----------------------------------------------------------------------------
# 步骤 2: 保存并传输到远程服务器
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[2/3] 传输到远程服务器...${NC}"

TAR_FILE="/tmp/${IMAGE_NAME}-${TAG}.tar.gz"
DEPLOY_SCRIPT="deploy-on-server.sh"

echo -e "${YELLOW}保存镜像...${NC}"
docker save ${IMAGE_NAME}:${TAG} | gzip > $TAR_FILE

echo -e "${YELLOW}创建远程部署目录...${NC}"
$ssh_cmd ${REMOTE_USER}@${REMOTE_HOST} "mkdir -p ~/latte-album"

echo -e "${YELLOW}传输文件到远程服务器...${NC}"
$scp_cmd $TAR_FILE $DEPLOY_SCRIPT .env.remote ${REMOTE_USER}@${REMOTE_HOST}:~/latte-album/

# 清理本地临时文件
rm -f $TAR_FILE

echo -e "${GREEN}[OK] 传输完成${NC}"

# -----------------------------------------------------------------------------
# 步骤 3: 完成
# -----------------------------------------------------------------------------
echo ""
echo "=========================================="
echo -e "${GREEN}构建并传输完成!${NC}"
echo ""
echo -e "${YELLOW}下一步操作 (需要 root 权限):${NC}"
echo ""
echo "  1. SSH 登录到远程服务器:"
echo "     ${YELLOW}ssh ${REMOTE_USER}@${REMOTE_HOST}${NC}"
echo ""
echo "  2. 执行部署脚本:"
echo "     ${YELLOW}cd ~/latte-album${NC}"
echo "     ${YELLOW}chmod +x deploy-on-server.sh${NC}"
echo "     ${YELLOW}./deploy-on-server.sh${NC}"
echo ""
echo -e "${YELLOW}访问地址:${NC} http://${REMOTE_HOST}:${REMOTE_SERVER_PORT}"
