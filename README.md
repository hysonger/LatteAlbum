# Latte Album

![screenshot](./screenshot.png)

为个人 NAS 设计的响应式网页相册应用，以简洁美观的瀑布流回顾您的照片与视频，不做多余的事。

*PS：这个项目初期是 TRAE 写的，后期转向 Claude Code，我自己写的代码顶天了十几行（有时候自己已经看到问题点了，比问AI快），完全干的是 甲方+测试 的活。。。AI真的拯救懒人，以前做这种规模的项目想都不敢想*

*PS2: 别小看这么一个项目，要做到使用自然需要做不少隐含功能点和重构*

## 主要特性

- 基本的排序筛选功能，按日期/文件类型筛选，拍摄/创建/修改时间/文件名排序
- 并行化，经过性能调优的文件扫描和转码
- 瀑布流双层懒加载机制（图片/文件列表）
- 相机信息、视频时长、编码格式等详细信息显示
- 文件全量扫描按钮，支持扫描进度显示
- 响应式设计，桌面端和移动端适配
- 支持 HEIF 格式文件

TODO：视频缩略图

## 快速开始

### 前置要求

本项目目前**仅考虑了 Linux 和 macOS 的兼容性**。

- Rust 1.75+
- Node.js 18+
- libheif
    用于 HEIF 格式支持。根据系统不同，可能需要安装-dev后缀的版本用于编译。
    该库编解码还需要依赖其他库，请参考[libheif 文档](https://github.com/strukturag/libheif)
- FFmpeg（用于视频缩略图生成）

### 开发构建与运行

```bash
# 运行后端
cd rust
# 如果系统上已经正确安装了依赖库
cargo run
# 如果系统上依赖库版本太旧，满足不了libheif-rs等的需要，可以带 build-vendor feature 从vendor目录中编译
# 注意，这并不会自动处理依赖库的依赖库，您可能需要自行补全库以及libheif的特性开关
./cargo-with-vendor.sh run

# 前端（另开终端）
cd frontend
npm install
npm run dev     # 开发服务器（端口 3000）
npm run build   # 生产构建

# 打包
./package.sh
```

## 配置项

| 环境变量 | 默认值 | 说明 |
|----------|--------|------|
| `LATTE_HOST` | `0.0.0.0` | 服务器绑定地址 |
| `LATTE_PORT` | `8080` | 服务器端口 |
| `LATTE_BASE_PATH` | `./photos` | 照片目录 |
| `LATTE_DB_PATH` | `./data/album.db` | SQLite 数据库路径 |
| `LATTE_CACHE_DIR` | `./cache` | 缩略图缓存目录 |
| `LATTE_STATIC_DIR` | `./static/dist` | 前端静态文件目录 |
| `LATTE_THUMBNAIL_SMALL` | `300` | 小缩略图宽度 (px) |
| `LATTE_THUMBNAIL_MEDIUM` | `450` | 中缩略图宽度 (px) |
| `LATTE_THUMBNAIL_LARGE` | `900` | 大缩略图宽度 (px) |
| `LATTE_THUMBNAIL_QUALITY` | `0.8` | JPEG 质量 (80%) |
| `LATTE_SCAN_CRON` | `0 0 2 * * ?` | 定时扫描 cron（每天 2 AM） |
| `LATTE_VIDEO_FFMPEG_PATH` | `/usr/bin/ffmpeg` | FFmpeg 可执行文件路径 |

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Axum, Rust, Tokio, SQLx, SQLite |
| 前端 | Vue 3, TypeScript, Pinia, Element Plus |
| 媒体处理 | image, libheif-rs, ffmpeg-next, kamadak-exif |
| 缓存 | Moka (内存) + 磁盘缓存 |
| 通信 | REST API + 原生 WebSocket |

## 项目结构

```
latte-album/
├── rust/
│   └── src/
│       ├── main.rs              # 应用入口
│       ├── app.rs               # App 结构体和路由配置
│       ├── config.rs            # 配置加载
│       ├── api/                 # REST API 处理器
│       │   ├── files.rs         # 文件操作、缩略图、邻居
│       │   ├── directories.rs   # 目录树
│       │   └── system.rs        # 重新扫描、状态、进度
│       ├── db/                  # 数据库层
│       │   ├── pool.rs          # sqlx 连接池
│       │   ├── models.rs        # MediaFile, Directory 模型
│       │   └── repository.rs    # 数据访问层
│       ├── services/            # 业务逻辑
│       │   ├── scan_service.rs  # 文件扫描和元数据提取
│       │   ├── file_service.rs  # 文件服务和缩略图生成
│       │   ├── cache_service.rs # Moka 缓存管理
│       │   └── scheduler.rs     # 定时扫描调度器
│       ├── processors/          # 媒体格式处理器
│       ├── extraction/          # 元数据提取工具
│       ├── websocket/           # WebSocket 处理器
│       └── utils/               # 辅助函数
├── frontend/
│   ├── src/
│   │   ├── views/      # 页面组件
│   │   ├── components/ # 可复用组件
│   │   ├── stores/     # Pinia 状态管理
│   │   ├── services/   # API 客户端
│   │   └── types/      # TypeScript 类型
│   └── package.json
└── Cargo.toml
```

## License

MIT

示例图片中的部分内容可能存在版权，仅供演示使用，版权归原所有者所有。
