# LatteAlbum Integration Test Toolchain

This directory contains the integration testing toolchain for LatteAlbum, covering both frontend and backend development servers.

## Overview

The test toolchain provides:

1. **Automatic server management** - Starts/stops backend and frontend servers
2. **Cache management** - Clears thumbnail cache before tests
3. **API testing** - Verifies backend endpoints are responsive
4. **Browser E2E testing** - Uses agent-browser for frontend testing
5. **Clean shutdown** - Properly terminates all processes on exit

## Directory Structure

```
tests/integration/
├── run-tests.sh          # Main orchestration script
├── config.env            # Test configuration
├── frontend/
│   └── test-browser.js   # Browser E2E tests
└── README.md             # This file
```

## Quick Start

### Run Full Test Suite

```bash
cd /path/to/LatteAlbum
./tests/integration/run-tests.sh
```

### Run Tests Only (servers already running)

```bash
./tests/integration/run-tests.sh --test-only
```

### Skip Cache Clear

```bash
./tests/integration/run-tests.sh --skip-cache
```

## Prerequisites

### Required Tools

1. **Rust toolchain** - For building the backend
2. **Node.js** - For the frontend
3. **agent-browser** - For browser automation
   ```bash
   npm install -g agent-browser
   agent-browser install
   ```

### Ports

Ensure these ports are available:
- `8080` - Backend server (configurable via `BACKEND_PORT`)
- `5173` - Frontend dev server (configurable via `FRONTEND_PORT`)

## Configuration

Edit `config.env` to customize:

```bash
# Backend
BACKEND_PORT=8080

# Frontend  
FRONTEND_PORT=5173

# URLs
BACKEND_URL=http://localhost:8080
FRONTEND_URL=http://localhost:5173

# Test timeouts
TEST_TIMEOUT=30000
BROWSER_TIMEOUT=60000
```

## Script Options

| Option | Description |
|--------|-------------|
| `-h, --help` | Show help message |
| `-s, --skip-cache` | Skip cache clearing |
| `-b, --skip-backend` | Skip backend tests |
| `-f, --skip-frontend` | Skip frontend tests |
| `-t, --test-only` | Only run tests (servers must be running) |
| `--backend-port PORT` | Set backend port |
| `--frontend-port PORT` | Set frontend port |

## What Gets Tested

### Backend API Tests

1. **System Status** - `GET /api/system/status`
2. **Files List** - `GET /api/files`
3. **Directories** - `GET /api/directories`

### Frontend Browser Tests

1. **Page Load** - Verifies frontend loads correctly
2. **Gallery Display** - Checks for gallery content
3. **Navigation** - Tests page navigation works

## Output

The test suite provides colored output:

- 🔵 **Blue** - Info messages
- 🟢 **Green** - Success messages  
- 🟡 **Yellow** - Warnings
- 🔴 **Red** - Errors

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All tests passed |
| 1 | One or more tests failed |

## Troubleshooting

### Port Already in Use

If you get port conflicts:

```bash
# Find process using the port
lsof -i :8080
lsof -i :5173

# Kill the process
kill <PID>
```

### Browser Tests Fail

Ensure agent-browser is installed:

```bash
npm install -g agent-browser
agent-browser install
```

### Backend Fails to Start

Check Rust dependencies:

```bash
cd rust
./cargo-with-vendor.sh check
```

## CI/CD Integration

Use in CI pipelines:

```bash
#!/bin/bash
set -e

cd /path/to/LatteAlbum

# Run tests with custom ports
BACKEND_PORT=8080 FRONTEND_PORT=5173 ./tests/integration/run-tests.sh

# Check exit code
if [ $? -eq 0 ]; then
    echo "All tests passed!"
else
    echo "Tests failed!"
    exit 1
fi
```

## Development

### Adding New Tests

1. **API Tests** - Add to `test-browser.js` in the `runTests()` function
2. **Browser Tests** - Add new functions and call them in `runTests()`

### Custom Browser Commands

The test script uses these agent-browser commands:

```bash
agent-browser open <url>           # Open URL
agent-browser wait --load         # Wait for page load
agent-browser snapshot            # Get page snapshot
agent-browser screenshot          # Take screenshot
agent-browser close               # Close browser
```

## Notes

- Cache is cleared in `rust/cache/` before tests start
- All processes are properly terminated on script exit (SIGINT/SIGTERM)
- Tests run sequentially to avoid port conflicts
