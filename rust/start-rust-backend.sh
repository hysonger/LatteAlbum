#!/bin/bash

# LatteAlbum Rust 后端启动脚本
# 该脚本用于启动 Rust 版本的后端服务，并提供配置选项
# 注意：所有路径配置必须使用绝对路径

# 默认配置
DEFAULT_HOST="0.0.0.0"
DEFAULT_PORT="8080"
DEFAULT_PHOTOS_DIR="$(cd .. && pwd)/photos"
DEFAULT_DB_DIR="$(pwd)/data"
DEFAULT_CACHE_DIR="$(pwd)/cache"
DEFAULT_STATIC_DIR="$(cd .. && pwd)/frontend/dist"

# 打印帮助信息
show_help() {
    echo "LatteAlbum Rust 后端启动脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -h, --host <address>     服务器监听地址 (默认: $DEFAULT_HOST)"
    echo "  -p, --port <number>      服务器监听端口 (默认: $DEFAULT_PORT)"
    echo "  --photos-dir <path>      照片目录路径 (必须是绝对路径，默认: $DEFAULT_PHOTOS_DIR)"
    echo "  --db-dir <path>          数据库目录路径 (必须是绝对路径，默认: $DEFAULT_DB_DIR)"
    echo "  --cache-dir <path>       缓存目录路径 (必须是绝对路径，默认: $DEFAULT_CACHE_DIR)"
    echo "  --static-dir <path>      静态文件目录路径 (必须是绝对路径，默认: $DEFAULT_STATIC_DIR)"
    echo "  --help                   显示帮助信息"
    echo ""
    echo "注意：所有路径配置必须使用绝对路径！"
}

# 解析命令行参数
host="$DEFAULT_HOST"
port="$DEFAULT_PORT"
photos_dir="$DEFAULT_PHOTOS_DIR"
db_dir="$DEFAULT_DB_DIR"
cache_dir="$DEFAULT_CACHE_DIR"
static_dir="$DEFAULT_STATIC_DIR"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--host)
            host="$2"
            shift 2
            ;;
        -p|--port)
            port="$2"
            shift 2
            ;;
        --photos-dir)
            photos_dir="$2"
            shift 2
            ;;
        --db-dir)
            db_dir="$2"
            shift 2
            ;;
        --cache-dir)
            cache_dir="$2"
            shift 2
            ;;
        --static-dir)
            static_dir="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
done

# 验证路径是否为绝对路径
check_absolute_path() {
    if [[ ! "$1" =~ ^/ ]]; then
        echo "错误: 路径 '$1' 不是绝对路径！"
        echo "所有路径配置必须使用绝对路径。"
        exit 1
    fi
}

# 检查所有路径是否为绝对路径
check_absolute_path "$photos_dir"
check_absolute_path "$db_dir"
check_absolute_path "$cache_dir"
check_absolute_path "$static_dir"

# 创建必要的目录
mkdir -p "$photos_dir"
mkdir -p "$db_dir"
mkdir -p "$cache_dir"

# 设置环境变量
export LATTE_HOST="$host"
export LATTE_PORT="$port"
export LATTE_BASE_PATH="$photos_dir"
export LATTE_DB_PATH="$db_dir/album.db"
export LATTE_CACHE_DIR="$cache_dir"
export LATTE_STATIC_DIR="$static_dir"

# 打印配置信息
echo "====================================================="
echo "LatteAlbum Rust 后端配置信息"
echo "====================================================="
echo "服务器地址: $host:$port"
echo "照片目录: $photos_dir"
echo "数据库路径: $db_dir/album.db"
echo "缓存目录: $cache_dir"
echo "静态文件目录: $static_dir"
echo "====================================================="
echo ""

# 启动 Rust 后端服务
echo "正在启动 Rust 后端服务..."

# 检查是否已编译
if [ ! -f "./target/debug/latte-album" ] && [ ! -f "./target/release/latte-album" ]; then
    echo "未找到可执行文件，正在编译..."
    cargo build
fi

# 启动服务
cargo run
