# Latte Album

![screenshot](./screenshot.png)

为个人 NAS 设计的响应式网页相册应用，以简洁美观的瀑布流回顾您的照片与视频，不做多余的事。

*PS：这个项目大部分是 TRAE 写的，后期转向 Claude Code，我自己写的代码只有个位数（有时候自己已经看到问题点了，比问AI快），完全干的是 甲方+测试 的活。。。AI真的拯救懒人，我纯Java新手只用了一个周末就做完了预期的基本功能，以前做这种规模的项目想都不敢想*

*PS2: 别小看这么一个项目，要做到使用自然需要做不少隐含功能点和重构*

## 主要特性

- 基本的排序筛选功能，按日期/文件类型筛选，拍摄/创建/修改时间/文件名排序
- 相机信息、视频时长、编码格式等详细信息显示
- 文件全量扫描按钮，支持扫描进度显示
- 响应式设计，桌面端和移动端适配
- 定时扫描，每日凌晨 2 点自动增量扫描
- 支持 HEIF 格式文件（需要系统安装 libheif 库）
- 支持视频缩略图（需要系统安装 ffmpeg）

## 快速开始

### 前置要求

本项目开发时**仅考虑了和 Linux 和 Mac OS X 系统的兼容性**，不保证 Windows 系统可用。

- Java 17+
- Node.js 18+
- Maven 3.6+

### 配置

创建 `.env` 文件：

```bash
# 照片目录（必需）
ALBUM_BASE_PATH=/path/to/your/photos

# 数据库（可选，默认 ./database.db）
ALBUM_DB_PATH=/path/to/database.db

# 缓存目录（可选，默认 ./cache）
ALBUM_CACHE_DIR=/path/to/cache
```

### 构建与运行

注意本项目处理 HEIF 格式文件需要系统安装 libheif 库，处理视频缩略图需要安装 ffmpeg。确保系统的搜索路径里面包含这两个工具的可执行文件。

```bash
# 构建后端
mvn clean package

# 运行后端（端口 8080）
java -jar target/latte-album-*.jar

# 或使用 Maven 直接运行
mvn spring-boot:run

# 前端（另开终端）
cd frontend
npm install
npm run dev     # 开发服务器（端口 3000）
npm run build   # 生产构建
```

### 访问

- **Web UI**: http://localhost:3000
- **API**: http://localhost:8080/album/api

## 配置项

```yaml
# application.yml

album:
  # 照片目录
  base-path: ${ALBUM_BASE_PATH}

  # 定时扫描（每天凌晨 2 点）
  scan:
    enabled: true
    cron: "0 0 2 * * ?"
    parallel:
      enabled: true

  # 缩略图尺寸
  thumbnail:
    small: 200
    medium: 500
    large: 1200
    quality: 0.85

  # 缓存设置
  cache:
    enabled: true
    disk-path: ${ALBUM_CACHE_DIR}

  # 视频处理
  video:
    ffmpeg-path: /usr/bin/ffmpeg
    thumbnail-time-offset: 1.0
```

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Spring Boot 3.2, Java 17, SQLite, Hibernate |
| 前端 | Vue 3, TypeScript, Pinia, Element Plus |
| 媒体处理 | metadata-extractor, thumbnailator, jave, libheif |
| 缓存 | Caffeine (内存) + 磁盘缓存 |
| 通信 | REST API + STOMP WebSocket |

## 项目结构

```
latte-album/
├── src/main/java/com/latte/album/
│   ├── controller/     # REST 端点
│   ├── service/        # 业务逻辑
│   ├── repository/     # 数据访问
│   ├── model/          # JPA 实体
│   ├── dto/            # 数据传输对象
│   ├── config/         # Spring 配置
│   └── util/           # 工具类
├── frontend/
│   ├── src/
│   │   ├── views/      # 页面组件
│   │   ├── components/ # 可复用组件
│   │   ├── stores/     # Pinia 状态管理
│   │   ├── services/   # API 客户端
│   │   └── types/      # TypeScript 类型
│   └── package.json
└── pom.xml
```

## License

MIT

示例图片中的部分内容可能存在版权，仅供演示使用，版权归原所有者所有。
