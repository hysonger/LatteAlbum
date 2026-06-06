#!/bin/bash

# Script to build libheif from vendor and then execute cargo commands.
#
# When to use this script:
#   Use this script when your system does not have libheif installed, or the
#   installed version is incompatible. It builds libheif from the vendored
#   source in rust/vendor/ (a git submodule) and links against it statically.
#
#   If your system already has libheif (e.g. via Homebrew or apt), you can run
#   cargo commands directly without this script.
#
# Prerequisites:
#   - The vendor submodule must be initialized: git submodule update --init
#   - cmake must be installed
#
# Usage:
#   ./cargo-with-vendor.sh <cargo-command> [additional cargo flags]
#
# Examples:
#   ./cargo-with-vendor.sh build
#   ./cargo-with-vendor.sh run
#   ./cargo-with-vendor.sh test
#   ./cargo-with-vendor.sh check --release

set -e

# Get the current directory (rust directory)
RUST_DIR=$(dirname "$(realpath "$0")")
VENDOR_DIR="$RUST_DIR/vendor"
BUILD_DIR="$RUST_DIR/target/vendor-build"
INSTALL_DIR="$BUILD_DIR/install"

# Verify vendor submodule is populated
if [ ! -f "$VENDOR_DIR/libheif/CMakeLists.txt" ]; then
    echo "Error: vendor/libheif/CMakeLists.txt not found."
    echo ""
    echo "The vendor submodule is not initialized. Either:"
    echo "  1. Initialize it:  git submodule update --init"
    echo "  2. Use system libheif instead: run cargo commands directly"
    echo "     e.g. cargo build, cargo test, etc."
    exit 1
fi

# Create build directories
mkdir -p "$BUILD_DIR"
mkdir -p "$INSTALL_DIR"

# Build libheif from vendor
echo "Building libheif from vendor directory..."

# Configure libheif with cmake
cmake -S "$VENDOR_DIR/libheif" -B "$BUILD_DIR/libheif" \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=OFF \
    -DWITH_EXAMPLES=OFF \
    -DWITH_GDK_PIXBUF=OFF \
    -DWITH_GNOME=OFF \
    -DBUILD_TESTING=OFF \
    -DBUILD_DOCUMENTATION=OFF \
    -DWITH_FUZZERS=OFF \
    -DWITH_WEBCODECS=OFF \
    -DWITH_UNCOMPRESSED_CODEC=OFF

# Build and install libheif
cmake --build "$BUILD_DIR/libheif" --parallel
cmake --install "$BUILD_DIR/libheif"

echo "libheif built successfully"

# Set PKG_CONFIG_PATH to include the built libheif
export PKG_CONFIG_PATH="$INSTALL_DIR/lib/pkgconfig:$PKG_CONFIG_PATH"

echo "PKG_CONFIG_PATH set to: $PKG_CONFIG_PATH"

# Check if cargo command is provided
if [ $# -eq 0 ]; then
    echo "Error: No cargo command provided"
    echo "Usage: ./cargo-with-vendor.sh <cargo-command> [additional cargo flags]"
    exit 1
fi

# Extract cargo command and remaining arguments
CARGO_CMD="$1"
shift

# Execute cargo command with vendor-build feature
echo "Running cargo $CARGO_CMD with vendor-build feature..."
# Allow passing additional features via VIDEO_FEATURES env var
EXTRA_FEATURES="${VIDEO_FEATURES:-}"
if [ -n "$EXTRA_FEATURES" ]; then
    cargo "$CARGO_CMD" --features "vendor-build,$EXTRA_FEATURES" "$@"
else
    cargo "$CARGO_CMD" --features vendor-build "$@"
fi