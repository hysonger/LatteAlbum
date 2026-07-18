## IMPORTANT: 输出语言 / Output Language

用户倾向于阅读中文。始终以 **中文** 向用户侧输出内容（含思维链）。
The user tend to read Chinese text. ALWAYS output contents for human read in Chinese (including thought chain).

## Project Overview

Latte Album is a personal photo album application for NAS deployment. Backend: Rust (Axum, SQLx, SQLite). Frontend: Vue 3 + TypeScript + Element Plus.

## File Structure

- rust/ - Backend project
- frontend/ - Frontend project

### Documents

There are multiple types of documents in this project:

- docs/ - General documents (If not specified other paths, put the documents here in default. )
- plans/ - Output plan documents in plan mode (mostly from Claude Code)
- openspec/ - OpenSpec SDD workflow documents of changes and specs

When finding and checking the documents, DO NOT miss any of these positions above.

## Quick Start

### Prerequisites

- Rust 1.75+, Node.js 18+
- libheif (for HEIF), FFmpeg **7.x** (optional, for video thumbnails)

> **Note:** FFmpeg 8.x removed headers (`avfft.h`) required by `ffmpeg-sys-next`. If you have FFmpeg 8 installed, and encountered with this header missing issues, also install FFmpeg 7 via `brew install ffmpeg@7`.

### Development

```bash
# Backend (in rust/ directory). 
# ./cargo-with-vendor.sh is an OPTIONAL wrapper for cargo. Refer to content below to get details.
cd rust/ && ./cargo-with-vendor.sh run

# Frontend (in frontend/ directory)
npm install && npm run dev

# Backend Local Testing
cd rust/ && ./dev-start.sh
```

### Production Build

```bash
cd rust && ./cargo-with-vendor.sh build --release
cd frontend && npm run build
./package.sh
```

### Testing (Frontend)

The frontend testing framework is based on Vitest（选型与约定见 [docs/frontend-testing.md](docs/frontend-testing.md)）：

```bash
cd frontend
npm run test          # 单次运行全部用例（CI 用）
npm run test:watch    # watch 模式，开发时持续运行
npm run test:ui       # 可视化测试面板
npm run test:coverage # 生成覆盖率报告（coverage/index.html）
```

测试文件就近平铺在 `src/` 下，命名为 `*.spec.ts`；DOM 环境为 jsdom。后端测试仍用 `cd rust && ./cargo-with-vendor.sh test`。



## Key Environment Variables

The program requires necessary environment variables set properly to find and put files.

See [config.rs](rust/src/config.rs) for full configuration.

## Build Environment

### Header Search Path Configuration

The compiler must find FFmpeg, libheif and other headers and libraries of dependencies to build the Rust part. 

MAKE SURE their path are correctly set before building, or the build will fail. 

If they are not in system default paths, set these environment variables before running cargo.

### macOS with Homebrew

Since Homebrew put the installed libraries in different paths, you HAVE TO add the necessary paths of the dependencies to environment variables BEFORE running `cargo` commands. See `rust/homebrew_build_env.sh` as an example.

### Linux (apt-based)

If FFmpeg and libheif are installed via apt, headers are usually found automatically.
If not, try to add the paths. Here is an example:

```bash
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH"
export CPATH="/usr/include"
export LIBRARY_PATH="/usr/lib/x86_64-linux-gnu"
```

These dependencies might be needed for compilation:

libavcodec-dev
libavformat-dev
libswscale-dev
libavfilter-dev
libexif-dev 
libavutil-dev
libavdevice-dev

### Using vendor-build (libheif from source)

When system libheif is unavailable or incompatible, use the vendor build.
If willing to do so, USE **cargo-with-vendor.sh** to replace ALL `cargo` in the command.

```bash
git submodule update --init
cd rust && ./cargo-with-vendor.sh test
```

This builds libheif from `rust/vendor/` and does not require system libheif.
FFmpeg is still required separately if using `video-processing`.

## Development Guidelines

### General

- After changes, ALWAYS check if any document files and AGENTS.md need to be updated.

### Rust Development

- Before using new crates: update `Cargo.toml`, run `cargo doc`, read documentation.
- Read the crate docs. NEVER start developing without understanding the crate's API.
- For hard problems, prototype Rust examples to test if the approach works first.

### Git Commits

- Only commit file changes in current conversation. DO NOT bring irrelevent changes in.
- When adding the co-authored line to the commit message, if the MODEL name is in **Claude series**, DO NOT straightly bring it into the message, because the actual LLM API might be replaced. Replace the model name with just the tool name `Claude Code`.
For example, write like this:
`Co-Authored-By: Claude Code <noreply@anthropic.com>`
Instead of:
`Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`
    - For other coding tools and models, just write the tool and model names in the original way.
- Write brief, clear and concise commit messages.

## Additional Documentation

- [Architecture Guide](docs/architecture.md) - Project architecture, design patterns, core systems
- [Additional Information](docs/additional.md) - Other information that might be helpful to handle the project
- [Known Issues](docs/known-issues.md) - Known issues and workarounds
- [Frontend Testing](docs/frontend-testing.md) - 前端测试框架选型、目录约定与各层测试范式
