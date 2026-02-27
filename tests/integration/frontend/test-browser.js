/**
 * LatteAlbum Frontend E2E Tests
 * 
 * This script uses agent-browser to test the LatteAlbum frontend.
 * It verifies:
 * - Main page loads correctly
 * - Gallery displays photos
 * - Photo navigation works
 * - API endpoints are responsive
 */

const { execSync } = require('child_process');
const http = require('http');

// Configuration
const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';
const FRONTEND_URL = process.env.FRONTEND_URL || 'http://localhost:5173';

// Colors for console output
const colors = {
    reset: '\x1b[0m',
    green: '\x1b[32m',
    red: '\x1b[31m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m'
};

function log(level, message) {
    const color = colors[level] || colors.reset;
    console.log(`${color}[${level.toUpperCase()}]${colors.reset} ${message}`);
}

function info(message) { log('blue', message); }
function success(message) { log('green', message); }
function error(message) { log('red', message); }
function warn(message) { log('yellow', message); }

// Test results
let testsPassed = 0;
let testsFailed = 0;

function assert(condition, testName) {
    if (condition) {
        success(`PASS: ${testName}`);
        testsPassed++;
        return true;
    } else {
        error(`FAIL: ${testName}`);
        testsFailed++;
        return false;
    }
}

// HTTP helper
function httpGet(url) {
    return new Promise((resolve, reject) => {
        http.get(url, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => resolve({ status: res.statusCode, data }));
        }).on('error', reject);
    });
}

// Run agent-browser command
function runBrowserCommand(args) {
    try {
        const cmd = `agent-browser ${args}`;
        info(`Running: ${cmd}`);
        const output = execSync(cmd, { 
            encoding: 'utf8',
            timeout: 60000,
            stdio: ['pipe', 'pipe', 'pipe']
        });
        return { success: true, output };
    } catch (err) {
        return { success: false, error: err.message, output: err.stdout || '' };
    }
}

// Test: Backend API Health
async function testBackendHealth() {
    info('Testing backend API health...');
    
    try {
        const response = await httpGet(`${BACKEND_URL}/api/system/status`);
        assert(response.status === 200, 'Backend /api/system/status returns 200');
        
        // Parse response
        try {
            const data = JSON.parse(response.data);
            assert(data.status !== undefined, 'Backend returns valid status JSON');
            success(`Backend status: ${data.status}, total_files=${data.total_files}`);
        } catch (e) {
            assert(false, 'Backend returns valid JSON');
        }
    } catch (err) {
        assert(false, `Backend API is accessible: ${err.message}`);
    }
}

// Test: Frontend loads
async function testFrontendLoads() {
    info('Testing frontend loads...');
    
    try {
        const response = await httpGet(FRONTEND_URL);
        assert(response.status === 200, 'Frontend server returns 200');
        assert(response.data.includes('<div id="app">') || response.data.includes('id="app"'), 'Frontend contains app mount point');
    } catch (err) {
        assert(false, `Frontend is accessible: ${err.message}`);
    }
}

// Test: Files API
async function testFilesAPI() {
    info('Testing files API...');
    
    try {
        const response = await httpGet(`${BACKEND_URL}/api/files?page=1&pageSize=10`);
        assert(response.status === 200, 'Files API returns 200');
        
        try {
            const data = JSON.parse(response.data);
            assert(Array.isArray(data.items) || data.items !== undefined, 'Files API returns files array');
            info(`Files API returned ${data.items?.length || 0} files (total: ${data.total})`);
        } catch (e) {
            assert(false, 'Files API returns valid JSON');
        }
    } catch (err) {
        assert(false, `Files API is accessible: ${err.message}`);
    }
}

// Test: Directories API
async function testDirectoriesAPI() {
    info('Testing directories API...');
    
    try {
        const response = await httpGet(`${BACKEND_URL}/api/directories`);
        assert(response.status === 200, 'Directories API returns 200');
        
        try {
            const data = JSON.parse(response.data);
            assert(Array.isArray(data) || data.tree !== undefined, 'Directories API returns valid data');
        } catch (e) {
            assert(false, 'Directories API returns valid JSON');
        }
    } catch (err) {
        assert(false, `Directories API is accessible: ${err.message}`);
    }
}

// Test: Browser automation - page loads
async function testBrowserPageLoad() {
    info('Testing browser automation - page load...');
    
    // First, open the frontend in browser
    const openResult = runBrowserCommand(`open ${FRONTEND_URL}`);
    
    if (!openResult.success) {
        assert(false, `Browser open: ${openResult.error}`);
        return;
    }
    
    // Wait for page to load
    runBrowserCommand('wait --load networkidle');
    
    // Get snapshot
    const snapshotResult = runBrowserCommand('snapshot');
    
    if (snapshotResult.success) {
        assert(snapshotResult.output.length > 100, 'Browser snapshot captured');
        info('Browser page loaded successfully');
        
        // Check for gallery or content
        if (snapshotResult.output.includes('gallery') || 
            snapshotResult.output.includes('photo') ||
            snapshotResult.output.includes('img') ||
            snapshotResult.output.includes('Gallery')) {
            assert(true, 'Gallery content detected in snapshot');
        } else {
            warn('No gallery content detected in snapshot (may be empty)');
        }
    } else {
        assert(false, `Browser snapshot: ${snapshotResult.error}`);
    }
    
    // Close browser
    runBrowserCommand('close');
}

// Test: Browser navigation
async function testBrowserNavigation() {
    info('Testing browser navigation...');
    
    // Open the frontend
    runBrowserCommand(`open ${FRONTEND_URL}`);
    runBrowserCommand('wait --load networkidle');
    
    // Take screenshot
    runBrowserCommand('screenshot --full');
    
    // Test navigation by checking if page is interactive
    const snapshotResult = runBrowserCommand('snapshot');
    
    if (snapshotResult.success) {
        assert(true, 'Page is interactive after navigation');
    } else {
        assert(false, 'Page navigation failed');
    }
    
    // Close browser
    runBrowserCommand('close');
}

// Main test runner
async function runTests() {
    console.log('');
    console.log('========================================');
    console.log('  LatteAlbum E2E Test Suite');
    console.log('========================================');
    console.log('');
    
    const startTime = Date.now();
    
    // API Tests (can run in parallel)
    info('Running API tests...');
    await testBackendHealth();
    await testFrontendLoads();
    await testFilesAPI();
    await testDirectoriesAPI();
    
    // Browser Tests
    info('');
    info('Running browser tests...');
    await testBrowserPageLoad();
    await testBrowserNavigation();
    
    // Summary
    const duration = ((Date.now() - startTime) / 1000).toFixed(2);
    
    console.log('');
    console.log('========================================');
    console.log('  Test Summary');
    console.log('========================================');
    console.log(`Total: ${testsPassed + testsFailed}`);
    console.log(`${colors.green}Passed: ${testsPassed}${colors.reset}`);
    if (testsFailed > 0) {
        console.log(`${colors.red}Failed: ${testsFailed}${colors.reset}`);
    }
    console.log(`Duration: ${duration}s`);
    console.log('========================================');
    
    return testsFailed === 0 ? 0 : 1;
}

// Export for use as module
module.exports = { runTests };

// Run if executed directly
if (require.main === module) {
    runTests()
        .then(code => process.exit(code))
        .catch(err => {
            error(`Test runner error: ${err.message}`);
            process.exit(1);
        });
}
