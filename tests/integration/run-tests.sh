#!/bin/bash
#
# LatteAlbum Integration Test Toolchain
# 
# This script orchestrates the full integration testing process:
# 1. Clears thumbnail cache
# 2. Starts Rust backend dev server
# 3. Starts Vue frontend dev server
# 4. Runs frontend E2E tests with agent-browser
# 5. Cleans up all processes
#

set -e

# Ensure cargo is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/rust"
FRONTEND_DIR="$PROJECT_ROOT/frontend"
CACHE_DIR="$BACKEND_DIR/cache"
DB_FILE="$BACKEND_DIR/data/album.db"
TEST_DIR="$PROJECT_ROOT/tests/integration"

# Server ports
BACKEND_PORT=${BACKEND_PORT:-8080}
FRONTEND_PORT=${FRONTEND_PORT:-5173}

# PIDs for cleanup
BACKEND_PID=""
FRONTEND_PID=""
BROWSER_PID=""

# Load test config if exists
if [ -f "$TEST_DIR/config.env" ]; then
    source "$TEST_DIR/config.env"
fi

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    
    # Kill child processes
    if [ -n "$BACKEND_PID" ] && kill -0 "$BACKEND_PID" 2>/dev/null; then
        log_info "Stopping backend server (PID: $BACKEND_PID)"
        kill "$BACKEND_PID" 2>/dev/null || true
    fi
    
    if [ -n "$FRONTEND_PID" ] && kill -0 "$FRONTEND_PID" 2>/dev/null; then
        log_info "Stopping frontend server (PID: $FRONTEND_PID)"
        kill "$FRONTEND_PID" 2>/dev/null || true
    fi
    
    # Kill any remaining node/npm processes on frontend port
    if lsof -ti:$FRONTEND_PORT >/dev/null 2>&1; then
        log_warn "Killing processes on port $FRONTEND_PORT"
        lsof -ti:$FRONTEND_PORT | xargs kill 2>/dev/null || true
    fi
    
    log_success "Cleanup complete"
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM EXIT

# Check if port is available
check_port() {
    local port=$1
    if lsof -ti:$port >/dev/null 2>&1; then
        log_error "Port $port is already in use"
        return 1
    fi
    return 0
}

# Wait for server to be ready
wait_for_server() {
    local url=$1
    local name=$2
    local max_attempts=${3:-30}
    local attempt=1
    
    log_info "Waiting for $name to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "$url" >/dev/null 2>&1; then
            log_success "$name is ready!"
            return 0
        fi
        echo -n "."
        sleep 1
        ((attempt++))
    done
    
    log_error "$name failed to start after $max_attempts seconds"
    return 1
}

# Clear thumbnail cache and database
clear_cache() {
    log_info "Clearing thumbnail cache and database..."
    
    # Clear thumbnail cache
    if [ -d "$CACHE_DIR" ]; then
        local cache_count=$(find "$CACHE_DIR" -type f 2>/dev/null | wc -l)
        if [ "$cache_count" -gt 0 ]; then
            rm -f "$CACHE_DIR"/*
            log_success "Cleared $cache_count cached thumbnails"
        else
            log_info "Cache directory is already empty"
        fi
    else
        log_info "Cache directory does not exist, creating..."
        mkdir -p "$CACHE_DIR"
    fi
    
    # Clear SQLite database
    if [ -f "$DB_FILE" ]; then
        local db_size=$(du -h "$DB_FILE" | cut -f1)
        rm -f "$DB_FILE"
        log_success "Cleared database file (size: $db_size)"
    else
        log_info "Database file does not exist"
    fi
    
    # Also clear any WAL/SHM files
    if [ -f "${DB_FILE}-wal" ]; then
        rm -f "${DB_FILE}-wal"
        log_info "Cleared database WAL file"
    fi
    if [ -f "${DB_FILE}-shm" ]; then
        rm -f "${DB_FILE}-shm"
        log_info "Cleared database SHM file"
    fi
}

# Start backend server
start_backend() {
    log_info "Starting Rust backend server..."
    
    # Check port availability
    if ! check_port $BACKEND_PORT; then
        log_error "Cannot start backend - port $BACKEND_PORT is in use"
        return 1
    fi
    
    # Set photo base path to project root photos directory
    export LATTE_BASE_PATH="$PROJECT_ROOT/photos"
    # Set PKG_CONFIG_PATH for libheif
    export PKG_CONFIG_PATH="$BACKEND_DIR/target/vendor-build/install/lib/pkgconfig:$PKG_CONFIG_PATH"
    
    # Start backend in background from rust directory
    cd "$BACKEND_DIR"
    ./cargo-with-vendor.sh run &
    BACKEND_PID=$!
    
    log_info "Backend started with PID: $BACKEND_PID"
    
    # Wait for backend to be ready
    if ! wait_for_server "http://localhost:$BACKEND_PORT/api/system/status" "Backend server"; then
        log_error "Backend failed to start"
        return 1
    fi
    
    return 0
}

# Start frontend server
start_frontend() {
    log_info "Starting Vue frontend dev server..."
    
    # Kill any existing node/vite processes that might be using the ports
    log_info "Ensuring ports are free..."
    for port in 5173 3000 5174; do
        if lsof -ti:$port >/dev/null 2>&1; then
            log_warn "Killing existing process on port $port"
            lsof -ti:$port | xargs kill 2>/dev/null || true
        fi
    done
    sleep 1
    
    # Start frontend in background, forcing port 5173
    cd "$FRONTEND_DIR"
    npm run dev -- --port 5173 &
    FRONTEND_PID=$!
    
    log_info "Frontend started with PID: $FRONTEND_PID"
    
    # Wait for frontend to be ready
    if ! wait_for_server "http://localhost:5173" "Frontend server"; then
        log_error "Frontend failed to start"
        return 1
    fi
    
    # Update FRONTEND_PORT to match what started
    FRONTEND_PORT=5173
    
    return 0
}

# Run browser tests
run_browser_tests() {
    log_info "Running frontend E2E tests with agent-browser..."
    
    local test_file="$TEST_DIR/frontend/test-browser.js"
    
    if [ ! -f "$test_file" ]; then
        log_error "Test file not found: $test_file"
        return 1
    fi
    
    # Run the browser tests
    cd "$TEST_DIR/frontend"
    node test-browser.js
    
    return $?
}

# Print usage
usage() {
    cat << EOF
LatteAlbum Integration Test Toolchain

Usage: $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -s, --skip-cache        Skip cache clearing
    -b, --skip-backend      Skip backend tests
    -f, --skip-frontend     Skip frontend tests
    -t, --test-only         Only run tests (assume servers are running)
    --backend-port PORT     Backend port (default: $BACKEND_PORT)
    --frontend-port PORT    Frontend port (default: $FRONTEND_PORT)

EXAMPLES:
    $0                      # Run full test suite
    $0 --skip-cache         # Run tests without clearing cache
    $0 --test-only          # Run tests only (servers must be running)

EOF
}

# Main function
main() {
    local skip_cache=false
    local skip_backend=false
    local skip_frontend=false
    local test_only=false
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -s|--skip-cache)
                skip_cache=true
                shift
                ;;
            -b|--skip-backend)
                skip_backend=true
                shift
                ;;
            -f|--skip-frontend)
                skip_frontend=true
                shift
                ;;
            -t|--test-only)
                test_only=true
                shift
                ;;
            --backend-port)
                BACKEND_PORT="$2"
                shift 2
                ;;
            --frontend-port)
                FRONTEND_PORT="$2"
                shift 2
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    echo "========================================"
    echo "  LatteAlbum Integration Test Suite"
    echo "========================================"
    echo ""
    
    # Clear cache unless skipped (always skip in test-only mode to preserve data)
    if [ "$skip_cache" = false ] && [ "$test_only" = false ]; then
        clear_cache
    elif [ "$skip_cache" = false ]; then
        log_info "Test-only mode - skipping cache clear to preserve data"
    else
        log_info "Skipping cache clear"
    fi
    
    # Start backend unless skipped or test-only
    if [ "$test_only" = false ]; then
        if [ "$skip_backend" = false ]; then
            start_backend || exit 1
        else
            log_info "Skipping backend start"
        fi
        
        # Start frontend unless skipped
        if [ "$skip_frontend" = false ]; then
            start_frontend || exit 1
        else
            log_info "Skipping frontend start"
        fi
    else
        log_info "Test-only mode - skipping server startup"
    fi
    
    # Export environment variables for tests
    export FRONTEND_URL="http://localhost:$FRONTEND_PORT"
    export BACKEND_URL="http://localhost:$BACKEND_PORT"
    
    # Run browser tests
    log_info ""
    log_info "========================================"
    log_info "  Running Tests"
    log_info "========================================"
    log_info "Frontend URL: $FRONTEND_URL"
    log_info "Backend URL: $BACKEND_URL"
    echo ""
    
    if run_browser_tests; then
        log_success ""
        log_success "========================================"
        log_success "  All Tests Passed!"
        log_success "========================================"
        exit 0
    else
        log_error ""
        log_error "========================================"
        log_error "  Tests Failed!"
        log_error "========================================"
        exit 1
    fi
}

# Run main
main "$@"
