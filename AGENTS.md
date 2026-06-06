## Project Overview

Latte Album is a personal photo album application for NAS deployment. Backend: Rust (Axum, SQLx, SQLite). Frontend: Vue 3 + TypeScript + Element Plus.

**Version**: 0.1.0 | **License**: MIT

## Quick Start

### Prerequisites

- Rust 1.75+, Node.js 18+
- libheif (for HEIF), FFmpeg **7.x** (optional, for video thumbnails)

> **Note:** FFmpeg 8.x removed headers (`avfft.h`) required by `ffmpeg-sys-next`. If you have FFmpeg 8 installed, and encountered with this header missing issues, also install FFmpeg 7 via `brew install ffmpeg@7`.

### Development

```bash
# Backend (in rust/ directory). 
# ./cargo-with-vendor.sh is an OPTIONAL wrapper for cargo. Refer to content below to get details.
./cargo-with-vendor.sh run

# Frontend (in frontend/ directory)
npm install && npm run dev
```

### Production Build

```bash
cd rust && ./cargo-with-vendor.sh build --release
cd frontend && npm run build
./package.sh
```

## Key Environment Variables

The program requires necessary environment variables set properly to find and put files.

See [config.rs](rust/src/config.rs) for full configuration.

## Build Environment

### Header Search Path Configuration

When building with FFmpeg (the `video-processing` feature), the compiler must find
FFmpeg's headers and libraries. If they are not in system default paths, set these
environment variables before running cargo.

### macOS with Homebrew

If you have both FFmpeg 8 (default) and FFmpeg 7 installed:

```bash
export PKG_CONFIG_PATH="/opt/homebrew/opt/ffmpeg@7/lib/pkgconfig:/opt/homebrew/lib/pkgconfig"
export CPATH="/opt/homebrew/opt/ffmpeg@7/include:/opt/homebrew/include"
export LIBRARY_PATH="/opt/homebrew/opt/ffmpeg@7/lib:/opt/homebrew/lib"
```

Then run cargo normally:

```bash
cd rust && cargo test --features video-processing
```

### Linux (apt-based)

If FFmpeg and libheif are installed via apt, headers are usually found automatically.
If not:

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
- Write clear, concise commit messages.

## Additional Documentation

- [Architecture Guide](docs/architecture.md) - Project architecture, design patterns, core systems
- [Additional Information](docs/additional.md) - Other information that might be helpful to handle the project
- [Known Issues](docs/known-issues.md) - Known issues and workarounds
