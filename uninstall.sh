#!/bin/bash
set -e

# =============================================================================
# Latte Album 卸载脚本
# =============================================================================
# 功能：停止服务、删除 systemd 配置、清理文件
# 用法：sudo ./uninstall.sh
# 环境变量：
#   UNINSTALL_SILENT=1    静默模式（不询问确认）
#   KEEP_DATA=1           保留数据目录
#   KEEP_USER=1           保留服务用户
# =============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${RED}Latte Album 卸载脚本${NC}"
echo "=========================================="

# 检查 root 权限
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Error: 此脚本需要 root 权限，请使用 sudo 运行${NC}"
    exit 1
fi

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="$SCRIPT_DIR"

# 加载配置文件（如果存在）
SERVICE_USER="latte-album"
if [ -f "$INSTALL_DIR/.env" ]; then
    set -a
    source "$INSTALL_DIR/.env"
    set +a
    SERVICE_USER=${SERVICE_USER:-latte-album}
fi

SYSTEMD_SERVICE="/etc/systemd/system/latte-album.service"
SERVICE_NAME="latte-album"

# 确认函数
confirm() {
    if [ "$UNINSTALL_SILENT" = "1" ]; then
        return 0
    fi
    
    local prompt="$1"
    local default="${2:-n}"
    
    if [ "$default" = "y" ]; then
        local options="[Y/n]"
    else
        local options="[y/N]"
    fi
    
    read -p "$(echo -e "${YELLOW}$prompt $options: ${NC}")" -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]] || ([ -z "$REPLY" ] && [ "$default" = "y" ]); then
        return 0
    else
        return 1
    fi
}

# -----------------------------------------------------------------------------
# 步骤 1: 停止并禁用服务
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[1/5] 停止服务...${NC}"

if systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
    echo -e "${YELLOW}停止服务...${NC}"
    systemctl stop "$SERVICE_NAME"
    echo -e "${GREEN}[OK] 服务已停止${NC}"
else
    echo -e "${YELLOW}服务未运行${NC}"
fi

if systemctl is-enabled --quiet "$SERVICE_NAME" 2>/dev/null; then
    echo -e "${YELLOW}禁用服务...${NC}"
    systemctl disable "$SERVICE_NAME"
    echo -e "${GREEN}[OK] 服务已禁用${NC}"
else
    echo -e "${YELLOW}服务未启用${NC}"
fi

# -----------------------------------------------------------------------------
# 步骤 2: 删除 systemd 服务文件
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[2/5] 删除 systemd 配置...${NC}"

if [ -f "$SYSTEMD_SERVICE" ]; then
    echo -e "${YELLOW}删除服务文件: $SYSTEMD_SERVICE${NC}"
    rm -f "$SYSTEMD_SERVICE"
    systemctl daemon-reload
    echo -e "${GREEN}[OK] systemd 配置已删除${NC}"
else
    echo -e "${YELLOW}服务文件不存在${NC}"
fi

# -----------------------------------------------------------------------------
# 步骤 3: 询问是否删除应用目录
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[3/5] 清理应用文件...${NC}"

if confirm "是否删除应用目录 ($INSTALL_DIR)?" "n"; then
    if [ -d "$INSTALL_DIR" ]; then
        echo -e "${YELLOW}删除应用目录: $INSTALL_DIR${NC}"
        rm -rf "$INSTALL_DIR"
        echo -e "${GREEN}[OK] 应用目录已删除${NC}"
    else
        echo -e "${YELLOW}应用目录不存在${NC}"
    fi
else
    echo -e "${YELLOW}保留应用目录${NC}"
fi

# -----------------------------------------------------------------------------
# 步骤 4: 询问是否删除数据目录
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[4/5] 清理数据文件...${NC}"

if [ "$KEEP_DATA" = "1" ]; then
    echo -e "${YELLOW}KEEP_DATA=1，保留数据目录${NC}"
else
    # 检查常见的数据目录位置
    DATA_DIRS=(
        "$INSTALL_DIR/data"
        "$INSTALL_DIR/cache"
        "$INSTALL_DIR/logs"
        "/data/db"
    )
    
    FOUND_DIRS=()
    for dir in "${DATA_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            FOUND_DIRS+=("$dir")
        fi
    done
    
    if [ ${#FOUND_DIRS[@]} -gt 0 ]; then
        echo -e "${YELLOW}找到以下数据目录:${NC}"
        for dir in "${FOUND_DIRS[@]}"; do
            echo "  - $dir"
        done
        
        if confirm "是否删除这些数据目录?" "n"; then
            for dir in "${FOUND_DIRS[@]}"; do
                echo -e "${YELLOW}删除: $dir${NC}"
                rm -rf "$dir"
            done
            echo -e "${GREEN}[OK] 数据目录已删除${NC}"
        else
            echo -e "${YELLOW}保留数据目录${NC}"
            echo -e "${YELLOW}提示: 如需备份，请手动复制以下目录:${NC}"
            for dir in "${FOUND_DIRS[@]}"; do
                echo "  - $dir"
            done
        fi
    else
        echo -e "${YELLOW}未找到数据目录${NC}"
    fi
fi

# -----------------------------------------------------------------------------
# 步骤 5: 询问是否删除服务用户
# -----------------------------------------------------------------------------
echo -e "\n${BLUE}[5/5] 清理用户账户...${NC}"

if [ "$KEEP_USER" = "1" ]; then
    echo -e "${YELLOW}KEEP_USER=1，保留服务用户${NC}"
elif [ -n "$SERVICE_USER" ] && [ "$SERVICE_USER" != "$(whoami)" ] && [ "$SERVICE_USER" != "root" ]; then
    if id "$SERVICE_USER" &>/dev/null; then
        echo -e "${YELLOW}找到服务用户: $SERVICE_USER${NC}"
        
        if confirm "是否删除服务用户 $SERVICE_USER?" "n"; then
            # 检查用户是否有其他进程
            if pgrep -u "$SERVICE_USER" > /dev/null 2>&1; then
                echo -e "${YELLOW}警告: 用户 $SERVICE_USER 仍有运行中的进程${NC}"
                if confirm "是否强制终止这些进程?" "n"; then
                    pkill -9 -u "$SERVICE_USER" || true
                else
                    echo -e "${YELLOW}跳过删除用户（仍有进程运行）${NC}"
                    exit 0
                fi
            fi
            
            userdel -r "$SERVICE_USER" 2>/dev/null || userdel "$SERVICE_USER"
            echo -e "${GREEN}[OK] 服务用户已删除${NC}"
        else
            echo -e "${YELLOW}保留服务用户${NC}"
        fi
    else
        echo -e "${YELLOW}服务用户不存在${NC}"
    fi
else
    echo -e "${YELLOW}未配置服务用户或使用系统用户，跳过${NC}"
fi

# -----------------------------------------------------------------------------
# 完成
# -----------------------------------------------------------------------------
echo ""
echo "=========================================="
echo -e "${GREEN}卸载完成!${NC}"
echo ""
echo -e "${YELLOW}提示:${NC}"
echo "  - 如果保留了数据目录，可以手动备份"
echo "  - 如果保留了应用目录，可以稍后重新安装"
echo "  - Java、ffmpeg、libheif 等系统依赖未被卸载"
echo ""
