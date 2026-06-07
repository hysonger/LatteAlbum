#!/bin/sh
# 该脚本设置环境变量以确保 Rust 后端服务能够正确找到依赖库和头文件。
# 注意：需要使用 source 命令来执行此脚本，以便环境变量能够在当前 shell 中生效。

export PKG_CONFIG_PATH="/opt/homebrew/opt/ffmpeg@7/lib/pkgconfig:/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
export CPATH="/opt/homebrew/opt/ffmpeg@7/include:/opt/homebrew/include:$CPATH"
export LIBRARY_PATH="/opt/homebrew/opt/ffmpeg@7/lib:/opt/homebrew/lib:$LIBRARY_PATH"