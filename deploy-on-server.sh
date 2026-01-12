#!/bin/bash
set -e

# =============================================================================
# Latte Album 远程服务器部署脚本
# =============================================================================
# 用法: 在远程服务器上以 root 用户执行
#   cd ~/latte-album
#   ./deploy-on-server.sh
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}Latte Album 远程服务器部署${NC}"
echo "=========================================="

# 加载配置
if [ -f .env.remote ]; then
    set -a
    source .env.remote
    set +a
fi

IMAGE_NAME=${IMAGE_NAME:-latte-album}
TAG=${TAG:-latest}
REMOTE_SERVER_PORT=${REMOTE_SERVER_PORT:-8080}

echo -e "\n${YELLOW}部署配置:${NC}"
echo "  镜像名称: ${IMAGE_NAME}:${TAG}"
echo "  服务端口: ${REMOTE_SERVER_PORT}"

# -----------------------------------------------------------------------------
# 步骤 1: 加载镜像
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[1/4] 加载 Docker 镜像...${NC}"

TAR_FILE="${IMAGE_NAME}-${TAG}.tar.gz"
if [ ! -f "$TAR_FILE" ]; then
    echo -e "${RED}Error: 镜像文件 $TAR_FILE 不存在${NC}"
    exit 1
fi

echo -e "${YELLOW}加载镜像...${NC}"
docker load -i $TAR_FILE

if docker image inspect ${IMAGE_NAME}:${TAG} > /dev/null 2>&1; then
    echo -e "${GREEN}[OK] 镜像加载成功${NC}"
else
    echo -e "${RED}[FAILED] 镜像加载失败${NC}"
    exit 1
fi

# -----------------------------------------------------------------------------
# 步骤 2: 创建 docker-compose.yml
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[2/4] 创建服务配置...${NC}"

cat > docker-compose.yml << EOF
services:
  latte-album:
    image: ${IMAGE_NAME}:${TAG}
    container_name: latte-album
    ports:
      - "${REMOTE_SERVER_PORT}:8080"
    volumes:
      - ${REMOTE_ALBUM_PHOTOS:-./photos}:/data/photos:ro
      - latte_cache:/cache
      - latte_db:/data/db
    environment:
      - SERVER_PORT=8080
      - JAVA_OPTS=${JAVA_OPTS:--Xmx2g -Xms512m}
      - ALBUM_BASE_PATH=/data/photos
      - ALBUM_CACHE_DIR=/cache
      - ALBUM_DB_PATH=/data/db/database.db
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8080/actuator/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s

volumes:
  latte_cache:
  latte_db:
EOF

echo -e "${GREEN}[OK] 配置创建成功${NC}"

# -----------------------------------------------------------------------------
# 步骤 3: 启动服务
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[3/4] 启动服务...${NC}"

echo -e "${YELLOW}停止旧容器...${NC}"
docker-compose down 2>/dev/null || true

echo -e "${YELLOW}创建网络...${NC}"
docker network create lattealbum_default 2>/dev/null || true

echo -e "${YELLOW}启动服务...${NC}"
docker-compose up -d

echo -e "${GREEN}[OK] 服务启动中...${NC}"

# -----------------------------------------------------------------------------
# 步骤 4: 检查健康状态
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[4/4] 检查服务状态...${NC}"

sleep 10

health_url="http://localhost:8080/actuator/health"
if curl -sf "$health_url" > /dev/null 2>&1; then
    echo -e "${GREEN}[OK] 服务健康检查通过${NC}"
else
    echo -e "${YELLOW}[WARNING] 健康检查未通过，请检查日志${NC}"
    echo "  查看日志: ${YELLOW}docker-compose logs${NC}"
fi

# -----------------------------------------------------------------------------
# 完成
# -----------------------------------------------------------------------------
echo ""
echo "=========================================="
echo -e "${GREEN}部署完成!${NC}"
echo ""
echo -e "${YELLOW}访问地址:${NC} http://localhost:${REMOTE_SERVER_PORT}"
echo ""
echo -e "${YELLOW}管理命令:${NC}"
echo "  查看日志: ${YELLOW}docker-compose logs -f${NC}"
echo "  重启服务: ${YELLOW}docker-compose restart${NC}"
echo "  停止服务: ${YELLOW}docker-compose down${NC}"
echo "  更新部署: ${YELLOW}./deploy-on-server.sh${NC}"
