#!/bin/bash
# start.sh

# 加载环境变量
if [ -f .env ]; then
  export $(cat .env | grep -v '^#' | xargs)
fi

# 检查必要目录
mkdir -p ${ALBUM_CACHE_DIR}

# 启动应用
java $JAVA_OPTS -jar photo-album.jar