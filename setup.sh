#!/bin/bash
set -e

# =============================================================================
# Latte Album 服务器端安装脚本
# =============================================================================
# 功能：解压部署包、安装依赖、配置 systemd 服务
# 用法：sudo ./setup.sh
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}Latte Album 服务器端安装脚本${NC}"
echo "=========================================="

# 检查 root 权限
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: 此脚本需要 root 权限，请使用 sudo 运行${NC}"
    exit 1
fi

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 检查是否在临时部署目录中运行（从 deploy-lite.sh 上传的位置）
if [[ "$SCRIPT_DIR" == *"latte-album-deploy"* ]]; then
    # 从环境变量或默认值获取部署路径
    TARGET_INSTALL_DIR=${INSTALL_DIR:-/opt/latte-album}
    
    echo -e "${YELLOW}检测到临时部署目录，将安装到: $TARGET_INSTALL_DIR${NC}"
    
    # 创建部署目录
    if [ ! -d "$TARGET_INSTALL_DIR" ]; then
        echo -e "${YELLOW}创建部署目录: $TARGET_INSTALL_DIR${NC}"
        mkdir -p "$TARGET_INSTALL_DIR"
    fi
    
    # 移动文件到部署目录
    echo -e "${YELLOW}移动文件到部署目录...${NC}"
    if ls "$SCRIPT_DIR"/latte-album-*.tar.gz 1> /dev/null 2>&1; then
        mv "$SCRIPT_DIR"/latte-album-*.tar.gz "$TARGET_INSTALL_DIR/" 2>/dev/null || true
    fi
    if [ -f "$SCRIPT_DIR/setup.sh" ]; then
        cp "$SCRIPT_DIR/setup.sh" "$TARGET_INSTALL_DIR/"
    fi
    
    # 切换到部署目录
    cd "$TARGET_INSTALL_DIR"
    INSTALL_DIR="$TARGET_INSTALL_DIR"
else
    # 正常情况：在部署目录中运行
    INSTALL_DIR="$SCRIPT_DIR"
fi

# -----------------------------------------------------------------------------
# 步骤 1: 查找并解压部署包
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[1/6] 查找并解压部署包...${NC}"

TAR_FILE=$(find "$INSTALL_DIR" -maxdepth 1 -name "latte-album-*.tar.gz" | sort -r | head -n 1)

if [ -z "$TAR_FILE" ]; then
    echo -e "${RED}Error: 未找到部署包 (latte-album-*.tar.gz)${NC}"
    exit 1
fi

echo -e "${GREEN}[OK] 找到部署包: $(basename $TAR_FILE)${NC}"

echo -e "${YELLOW}解压部署包...${NC}"
tar -xzf "$TAR_FILE" -C "$INSTALL_DIR"

# 确保解压出的文件有正确的权限
if [ -f "$INSTALL_DIR/run.sh" ]; then
    chmod +x "$INSTALL_DIR/run.sh"
fi
if [ -f "$INSTALL_DIR/uninstall.sh" ]; then
    chmod +x "$INSTALL_DIR/uninstall.sh"
fi

echo -e "${GREEN}[OK] 解压完成${NC}"

# 加载配置文件（优先级：.env.runtime > .env > 默认值）
# 首先加载 .env.runtime（从 deploy-lite.sh 上传的运行时配置）
# 注意：在创建新的 .env 文件时，不应该加载现有的 .env，避免覆盖 .env.runtime 的配置
if [ -f "$INSTALL_DIR/.env.runtime" ]; then
    echo -e "${YELLOW}加载运行时配置文件 .env.runtime...${NC}"
    set -a
    source "$INSTALL_DIR/.env.runtime"
    set +a
fi

# 然后加载现有的 .env（如果存在，会覆盖 .env.runtime 中的相同配置）
# 但只在 .env.runtime 不存在时才加载，避免在重新创建 .env 时被旧配置覆盖
if [ ! -f "$INSTALL_DIR/.env.runtime" ] && [ -f "$INSTALL_DIR/.env" ]; then
    echo -e "${YELLOW}加载配置文件 .env...${NC}"
    set -a
    source "$INSTALL_DIR/.env"
    set +a
fi

# 配置默认值
SERVICE_USER=${SERVICE_USER:-latte}
SERVER_PORT=${SERVER_PORT:-8080}
SERVER_ADDRESS=${SERVER_ADDRESS:-0.0.0.0}
ALBUM_BASE_PATH=${ALBUM_BASE_PATH:-/data/photos}
ALBUM_CACHE_DIR=${ALBUM_CACHE_DIR:-$INSTALL_DIR/cache}
ALBUM_DB_PATH=${ALBUM_DB_PATH:-$INSTALL_DIR/data/db/database.db}

echo -e "\n${YELLOW}安装配置:${NC}"
echo "  安装目录: $INSTALL_DIR"
echo "  服务用户: $SERVICE_USER"
echo "  服务端口: $SERVER_PORT"
echo "  服务地址: $SERVER_ADDRESS"
echo "  照片目录: $ALBUM_BASE_PATH"
echo "  缓存目录: $ALBUM_CACHE_DIR"
echo "  数据库路径: $ALBUM_DB_PATH"


# -----------------------------------------------------------------------------
# 步骤 2: 检查并安装 Java 17
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[2/6] 检查 Java 环境...${NC}"

JAVA_INSTALLED=false
if command -v java &> /dev/null; then
    JAVA_VERSION=$(java -version 2>&1 | head -n 1 | cut -d'"' -f2 | sed '/^1\./s///' | cut -d'.' -f1)
    if [ "$JAVA_VERSION" -ge 17 ]; then
        echo -e "${GREEN}[OK] Java $JAVA_VERSION 已安装${NC}"
        JAVA_INSTALLED=true
    else
        echo -e "${YELLOW}Java 版本过低 ($JAVA_VERSION)，需要 Java 17+${NC}"
    fi
fi

if [ "$JAVA_INSTALLED" = false ]; then
    echo -e "${YELLOW}安装 Java 17...${NC}"
    
    # 更新包列表
    apt-get update -qq
    
    # 安装 OpenJDK 17
    apt-get install -y openjdk-17-jre-headless
    
    # 验证安装
    if command -v java &> /dev/null; then
        JAVA_VERSION=$(java -version 2>&1 | head -n 1 | cut -d'"' -f2 | sed '/^1\./s///' | cut -d'.' -f1)
        if [ "$JAVA_VERSION" -ge 17 ]; then
            echo -e "${GREEN}[OK] Java $JAVA_VERSION 安装成功${NC}"
        else
            echo -e "${RED}Error: Java 安装失败或版本不正确${NC}"
            exit 1
        fi
    else
        echo -e "${RED}Error: Java 安装失败${NC}"
        exit 1
    fi
fi

# -----------------------------------------------------------------------------
# 步骤 3: 检查并安装 ffmpeg
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[3/6] 检查媒体处理依赖...${NC}"

# 检查 ffmpeg
if command -v ffmpeg &> /dev/null; then
    echo -e "${GREEN}[OK] ffmpeg 已安装: $(ffmpeg -version | head -n 1)${NC}"
else
    echo -e "${YELLOW}安装 ffmpeg...${NC}"
    apt-get update -qq
    apt-get install -y ffmpeg
    echo -e "${GREEN}[OK] ffmpeg 安装成功${NC}"
fi

# 检查 ffmpeg
if command -v heif-convert &> /dev/null; then
    echo -e "${GREEN}[OK] libheif 已安装: $(heif-convert --version | head -n 1)${NC}"
else
    echo -e "${YELLOW}安装 libheif...${NC}"
    apt-get update -qq
    apt-get install -y libheif-examples
    echo -e "${GREEN}[OK] libheif 安装成功${NC}"
fi

# -----------------------------------------------------------------------------
# 步骤 4: 创建服务用户（如果需要）
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[4/6] 配置服务用户...${NC}"

if [ -n "$SERVICE_USER" ] && [ "$SERVICE_USER" != "$(whoami)" ] && [ "$SERVICE_USER" != "root" ]; then
    if id "$SERVICE_USER" &>/dev/null; then
        echo -e "${GREEN}[OK] 用户 $SERVICE_USER 已存在${NC}"
    else
        echo -e "${YELLOW}创建用户 $SERVICE_USER...${NC}"
        useradd -r -s /bin/false "$SERVICE_USER"
        echo -e "${GREEN}[OK] 用户创建成功${NC}"
    fi
    RUN_USER="$SERVICE_USER"
else
    RUN_USER="root"
    echo -e "${YELLOW}将使用 root 用户运行服务（不推荐生产环境）${NC}"
fi

# -----------------------------------------------------------------------------
# 步骤 5: 创建目录并设置权限
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[5/6] 创建目录结构...${NC}"

# 创建应用基础目录
mkdir -p "$INSTALL_DIR/logs"

# 处理缓存目录
if [[ "$ALBUM_CACHE_DIR" = /* ]]; then
    # 绝对路径
    CACHE_DIR="$ALBUM_CACHE_DIR"
else
    # 相对路径，转换为绝对路径
    CACHE_DIR="$INSTALL_DIR/$ALBUM_CACHE_DIR"
    # 移除开头的 ./
    CACHE_DIR="${CACHE_DIR#./}"
fi
mkdir -p "$CACHE_DIR"
echo -e "${YELLOW}缓存目录: $CACHE_DIR${NC}"

# 处理数据库目录
DB_DIR=$(dirname "$ALBUM_DB_PATH")
if [[ "$DB_DIR" = /* ]]; then
    # 绝对路径
    DB_DIR_ABS="$DB_DIR"
else
    # 相对路径，转换为绝对路径
    DB_DIR_ABS="$INSTALL_DIR/$DB_DIR"
    # 移除开头的 ./
    DB_DIR_ABS="${DB_DIR_ABS#./}"
fi
mkdir -p "$DB_DIR_ABS"
echo -e "${YELLOW}数据库目录: $DB_DIR_ABS${NC}"

# 创建照片目录（如果不存在）
if [ ! -d "$ALBUM_BASE_PATH" ]; then
    echo -e "${YELLOW}创建照片目录: $ALBUM_BASE_PATH${NC}"
    mkdir -p "$ALBUM_BASE_PATH"
fi

# 设置目录权限
if [ "$RUN_USER" != "root" ]; then
    echo -e "${YELLOW}设置目录权限（用户: $RUN_USER）...${NC}"
    # 设置安装目录权限
    chown -R "$RUN_USER:$RUN_USER" "$INSTALL_DIR"
    # 设置缓存目录权限
    chown -R "$RUN_USER:$RUN_USER" "$CACHE_DIR"
    # 设置数据库目录权限
    chown -R "$RUN_USER:$RUN_USER" "$DB_DIR_ABS"

    # 照片目录不设置权限
    echo -e "${YELLOW}照片目录不设置权限，请注意手动配置服务可读的权限！${NC}"
else
    echo -e "${YELLOW}使用 root 用户，跳过权限设置${NC}"
fi

# 设置脚本执行权限
chmod +x "$INSTALL_DIR/run.sh"

echo -e "${GREEN}[OK] 目录结构创建完成${NC}"

# -----------------------------------------------------------------------------
# 步骤 6: 配置并安装 systemd 服务
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[6/6] 配置 systemd 服务...${NC}"

SERVICE_FILE="$INSTALL_DIR/latte-album.service"
SYSTEMD_SERVICE="/etc/systemd/system/latte-album.service"

# 如果服务文件存在，修改用户配置
if [ -f "$SERVICE_FILE" ]; then
    # 创建临时服务文件
    TEMP_SERVICE=$(mktemp)
    
    # 修改服务文件：更新路径和用户配置
    # 替换工作目录和启动命令路径
    sed -e "s|WorkingDirectory=/opt/latte-album|WorkingDirectory=$INSTALL_DIR|" \
        -e "s|ExecStart=/opt/latte-album/run.sh|ExecStart=$INSTALL_DIR/run.sh|" \
        -e "s|EnvironmentFile=-/opt/latte-album/.env|EnvironmentFile=-$INSTALL_DIR/.env|" \
        -e "s|ReadWritePaths=/opt/latte-album|ReadWritePaths=$INSTALL_DIR|" \
        "$SERVICE_FILE" > "$TEMP_SERVICE"
    
    # 如果指定了非root用户，取消注释并设置User和Group
    if [ "$RUN_USER" != "root" ]; then
        sed -i -e "s/^# User=latte-album/User=$RUN_USER/" \
               -e "s/^# Group=latte-album/Group=$RUN_USER/" \
               "$TEMP_SERVICE"
    fi
    
    # 复制到 systemd 目录
    cp "$TEMP_SERVICE" "$SYSTEMD_SERVICE"
    rm "$TEMP_SERVICE"
    
    echo -e "${GREEN}[OK] 服务文件已安装${NC}"
else
    echo -e "${RED}Error: 未找到服务文件 latte-album.service${NC}"
    exit 1
fi

# 创建或更新环境变量文件
ENV_FILE="$INSTALL_DIR/.env"
# 如果 .env.runtime 存在，优先使用其配置；否则使用当前配置
if [ -f "$INSTALL_DIR/.env.runtime" ]; then
    echo -e "${YELLOW}用 .env.runtime 替换 .env 文件...${NC}"

    rm -f $INSTALL_DIR/.env.old
    mv $ENV_FILE $INSTALL_DIR/.env.old && mv $INSTALL_DIR/.env.runtime $ENV_FILE
fi

# 设置文件权限
if [ "$RUN_USER" != "root" ]; then
    chown "$RUN_USER:$RUN_USER" "$ENV_FILE"
fi

# 验证环境变量文件格式
echo -e "${YELLOW}验证环境变量文件...${NC}"
if grep -q "^SERVER_PORT=" "$ENV_FILE" 2>/dev/null; then
    SERVER_PORT_FROM_FILE=$(grep "^SERVER_PORT=" "$ENV_FILE" | cut -d'=' -f2)
    echo -e "${GREEN}[OK] SERVER_PORT=$SERVER_PORT_FROM_FILE${NC}"
else
    echo -e "${RED}[WARNING] SERVER_PORT 未在 .env 文件中找到${NC}"
fi

# 验证 JAVA_OPTS 是否正确包含引号
if grep -q "^JAVA_OPTS=" "$ENV_FILE" 2>/dev/null; then
    JAVA_OPTS_FROM_FILE=$(grep "^JAVA_OPTS=" "$ENV_FILE")
    if echo "$JAVA_OPTS_FROM_FILE" | grep -q 'JAVA_OPTS="'; then
        echo -e "${GREEN}[OK] JAVA_OPTS 格式正确（包含引号）${NC}"
    else
        echo -e "${YELLOW}[WARNING] JAVA_OPTS 可能缺少引号，建议检查格式${NC}"
    fi
else
    echo -e "${RED}[WARNING] JAVA_OPTS 未在 .env 文件中找到${NC}"
fi

echo -e "${GREEN}[OK] 环境变量文件已创建/更新${NC}"

# 重新加载 systemd
systemctl daemon-reload

# 启用服务（开机自启）
systemctl enable latte-album

echo -e "${GREEN}[OK] systemd 服务已配置${NC}"

# -----------------------------------------------------------------------------
# 完成
# -----------------------------------------------------------------------------
echo ""
echo "=========================================="
echo -e "${GREEN}安装完成!${NC}"
echo ""
echo -e "${YELLOW}服务管理命令:${NC}"
echo ""
echo "  启动服务:   ${YELLOW}sudo systemctl start latte-album${NC}"
echo "  停止服务:   ${YELLOW}sudo systemctl stop latte-album${NC}"
echo "  查看状态:   ${YELLOW}sudo systemctl status latte-album${NC}"
echo "  查看日志:   ${YELLOW}sudo journalctl -u latte-album -f${NC}"
echo "  禁用自启:   ${YELLOW}sudo systemctl disable latte-album${NC}"
echo ""
echo -e "${YELLOW}配置信息:${NC}"
echo "  服务端口:   ${SERVER_PORT}"
echo "  服务地址:   ${SERVER_ADDRESS}"
echo "  访问地址:   http://${SERVER_ADDRESS}:${SERVER_PORT}"
echo ""
echo -e "${YELLOW}提示:${NC}"
echo "  如果修改了 $INSTALL_DIR/.env 文件，需要重启服务才能生效:"
echo "    ${YELLOW}sudo systemctl restart latte-album${NC}"
echo ""
