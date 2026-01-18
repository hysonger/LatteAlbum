#!/bin/bash

# 默认配置
OUTPUT_DIR="./output"

# 解析命令行参数
while getopts "o:fb" opt; do
  case $opt in
    o) OUTPUT_DIR="$OPTARG" ;;
    f) BUILD_FRONTEND=false ;;
    b) BUILD_BACKEND=false ;;
    *) echo "Usage: $0 [-o output_dir] [-f] [-b]" ; exit 1 ;;
  esac
done

# 创建输出目录
mkdir -p "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/static/dist"
mkdir -p "$OUTPUT_DIR/data"
mkdir -p "$OUTPUT_DIR/cache"
mkdir -p "$OUTPUT_DIR/photos"

# 构建前端（如果需要）
if [ "$BUILD_FRONTEND" = true ]; then
  echo "Building frontend..."
  cd frontend
  npm install
  npm run build
  cd ..
fi

# 构建后端（如果需要）
if [ "$BUILD_BACKEND" = true ]; then
  echo "Building backend..."
  cd rust
  cargo build --release
  cd ..
fi

# 复制前端构建产物
echo "Copying frontend build artifacts..."
cp -r frontend/dist/* "$OUTPUT_DIR/static/dist/"

# 复制后端可执行文件
echo "Copying backend executable..."
cp rust/target/release/latte-album "$OUTPUT_DIR/"

# 复制配置文件模板
if [ -f ".env.remote" ]; then
  echo "Copying environment file..."
  cp .env.remote "$OUTPUT_DIR/.env.example"
fi

# 复制README文件
echo "Copying documentation..."
cp README.md "$OUTPUT_DIR/"

# 设置执行权限
chmod +x "$OUTPUT_DIR/latte-album"

echo "Packaging completed successfully!"
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "To run the application:"
echo "cd $OUTPUT_DIR"
echo "cp .env.example .env  # Edit .env if needed"
echo "./latte-album"
