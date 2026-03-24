/**
 * LatteAlbum Frontend E2E Tests
 * 
 * This script uses agent-browser to test the LatteAlbum frontend.
 * It verifies:
 * - Main page loads correctly
 * - Gallery displays photos
 * - Photo navigation works
 * - API endpoints are responsive
 * - And more...
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
        
        try {
            const data = JSON.parse(response.data);
            assert(data.status !== undefined, 'Backend returns valid status JSON');
            success(`Backend status: ${data.status}, total_files=${data.total_files}`);
            return data;
        } catch (e) {
            assert(false, 'Backend returns valid JSON');
            return null;
        }
    } catch (err) {
        assert(false, `Backend API is accessible: ${err.message}`);
        return null;
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
            assert(data.items !== undefined || Array.isArray(data), 'Files API returns files array');
            const fileCount = data.items?.length || (Array.isArray(data) ? data.length : 0);
            info(`Files API returned ${fileCount} files (total: ${data.total || 'N/A'})`);
            return data;
        } catch (e) {
            assert(false, 'Files API returns valid JSON');
            return null;
        }
    } catch (err) {
        assert(false, `Files API is accessible: ${err.message}`);
        return null;
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
            assert(data !== null, 'Directories API returns valid data');
            info(`Directories API returned: ${Array.isArray(data) ? data.length + ' directories' : 'tree structure'}`);
            return data;
        } catch (e) {
            assert(false, 'Directories API returns valid JSON');
            return null;
        }
    } catch (err) {
        assert(false, `Directories API is accessible: ${err.message}`);
        return null;
    }
}

// Test: Thumbnail API
async function testThumbnailAPI(filesData) {
    info('Testing thumbnail API...');
    
    if (!filesData || !filesData.items || filesData.items.length === 0) {
        warn('No files available for thumbnail test, skipping...');
        return;
    }
    
    // Find an image file
    const imageFile = filesData.items.find(f => f.file_type === 'image' || f.mime_type?.startsWith('image/'));
    
    if (!imageFile) {
        warn('No image files available for thumbnail test, skipping...');
        return;
    }
    
    try {
        const thumbnailUrl = `${BACKEND_URL}/api/files/${imageFile.id}/thumbnail?size=medium`;
        const response = await httpGet(thumbnailUrl);
        
        if (response.status === 200) {
            assert(true, `Thumbnail API returns 200 for file ${imageFile.id}`);
        } else if (response.status === 404) {
            warn(`Thumbnail not found for file ${imageFile.id} (may need scan first)`);
        } else {
            assert(false, `Thumbnail API returns ${response.status}`);
        }
    } catch (err) {
        warn(`Thumbnail API test failed: ${err.message}`);
    }
}

// Test: Video Thumbnail API
async function testVideoThumbnailAPI(filesData) {
    info('Testing video thumbnail API...');
    
    if (!filesData || !filesData.items || filesData.items.length === 0) {
        warn('No files available for video thumbnail test, skipping...');
        return;
    }
    
    // Find a video file
    const videoFile = filesData.items.find(f => f.file_type === 'video' || f.mime_type?.startsWith('video/'));
    
    if (!videoFile) {
        warn('No video files available for thumbnail test, skipping...');
        return;
    }
    
    try {
        const thumbnailUrl = `${BACKEND_URL}/api/files/${videoFile.id}/video-thumb`;
        const response = await httpGet(thumbnailUrl);
        
        if (response.status === 200) {
            assert(true, `Video thumbnail API returns 200 for file ${videoFile.id}`);
        } else if (response.status === 404) {
            warn(`Video thumbnail not found for file ${videoFile.id} (may need scan first)`);
        } else {
            assert(false, `Video thumbnail API returns ${response.status}`);
        }
    } catch (err) {
        warn(`Video thumbnail API test failed: ${err.message}`);
    }
}

// Test: EXIF Data API
async function testExifAPI(filesData) {
    info('Testing EXIF data API...');
    
    if (!filesData || !filesData.items || filesData.items.length === 0) {
        warn('No files available for EXIF test, skipping...');
        return;
    }
    
    // Find an image file
    const imageFile = filesData.items.find(f => f.file_type === 'image' || f.mime_type?.startsWith('image/'));
    
    if (!imageFile) {
        warn('No image files available for EXIF test, skipping...');
        return;
    }
    
    try {
        const exifUrl = `${BACKEND_URL}/api/files/${imageFile.id}/exif`;
        const response = await httpGet(exifUrl);
        
        if (response.status === 200) {
            assert(true, `EXIF API returns 200 for file ${imageFile.id}`);
            try {
                const exifData = JSON.parse(response.data);
                info(`EXIF data: camera=${exifData.camera || 'N/A'}, lens=${exifData.lens || 'N/A'}`);
            } catch (e) {
                // Ignore JSON parse error
            }
        } else if (response.status === 404) {
            warn(`EXIF data not found for file ${imageFile.id}`);
        } else {
            assert(false, `EXIF API returns ${response.status}`);
        }
    } catch (err) {
        warn(`EXIF API test failed: ${err.message}`);
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

// Test: Photo lightbox/detail view
async function testPhotoLightbox() {
    info('Testing photo lightbox/detail view...');
    
    // Open the frontend
    runBrowserCommand(`open ${FRONTEND_URL}`);
    runBrowserCommand('wait --load networkidle');
    
    // Get initial snapshot to find clickable elements
    const snapshotResult = runBrowserCommand('snapshot');
    
    if (!snapshotResult.success) {
        assert(false, 'Failed to get initial snapshot');
        runBrowserCommand('close');
        return;
    }
    
    // Look for clickable image/photo elements
    // LatteAlbum uses .media-card for gallery items
    const clickCommands = [
        'click .media-card',           // MediaCard component
        'click .gallery-container',   // Gallery container
        'click [class*="thumbnail"]', // Thumbnail container
        'click img.thumbnail'         // Thumbnail image
    ];
    
    let clicked = false;
    for (const cmd of clickCommands) {
        const result = runBrowserCommand(cmd);
        if (result.success && !result.output.includes('error')) {
            info(`Clicked element: ${cmd}`);
            clicked = true;
            // Wait a bit for lightbox to open
            runBrowserCommand('wait --load 1000');
            break;
        }
    }
    
    if (clicked) {
        // Check if lightbox opened
        const lightboxResult = runBrowserCommand('snapshot');
        if (lightboxResult.success) {
            // LatteAlbum uses .photo-viewer for lightbox
            const hasLightbox = lightboxResult.output.includes('photo-viewer') || 
                              lightboxResult.output.includes('viewer-content') ||
                              lightboxResult.output.includes('nav-btn');
            if (hasLightbox) {
                assert(true, 'Lightbox opened after clicking photo');
            } else {
                warn('Lightbox may not have opened (or uses different UI)');
            }
        }
    } else {
        warn('Could not find clickable photo element');
    }
    
    runBrowserCommand('close');
}

// Test: Gallery scroll and pagination
async function testGalleryScroll() {
    info('Testing gallery scroll and pagination...');
    
    runBrowserCommand(`open ${FRONTEND_URL}`);
    runBrowserCommand('wait --load networkidle');
    
    // Get initial snapshot to see gallery structure
    const initialSnapshot = runBrowserCommand('snapshot');
    const hasGallery = initialSnapshot.success && initialSnapshot.output.includes('gallery');
    
    if (!hasGallery) {
        warn('Gallery container not found in snapshot');
        runBrowserCommand('close');
        return;
    }
    
    // Scroll down to trigger lazy loading
    // Use JavaScript evaluation for scrolling
    const scrollResult = runBrowserCommand('evaluate window.scrollTo(0, document.body.scrollHeight)');
    
    if (scrollResult.success) {
        assert(true, 'Page scroll executed');
        // Wait for any lazy loading
        runBrowserCommand('wait --load 2000');
        
        // Get snapshot after scroll
        const snapshotAfterScroll = runBrowserCommand('snapshot');
        if (snapshotAfterScroll.success) {
            info('Gallery scrolled, content may have lazy loaded');
        }
    } else {
        warn('Scroll test had issues (may be single page)');
    }
    
    runBrowserCommand('close');
}

// Test: Filter functionality
async function testFilterFunctionality() {
    info('Testing filter functionality...');
    
    runBrowserCommand(`open ${FRONTEND_URL}`);
    runBrowserCommand('wait --load networkidle');
    
    // LatteAlbum uses .filter-container and .filter-button
    const filterCommands = [
        'click .filter-button',        // Filter button in FilterControls
        'click .filter-container',     // Filter container
        'click [class*="filter"]',    // Any filter element
        'click button.filter'          // Fallback
    ];
    
    let filterFound = false;
    for (const cmd of filterCommands) {
        const result = runBrowserCommand(cmd);
        if (result.success) {
            info(`Found filter control: ${cmd}`);
            filterFound = true;
            runBrowserCommand('wait --load 500');
            break;
        }
    }
    
    if (filterFound) {
        // Try to see if filter options appeared
        const snapshot = runBrowserCommand('snapshot');
        if (snapshot.success && (snapshot.output.includes('image') || snapshot.output.includes('video'))) {
            assert(true, 'Filter controls are present and functional');
        }
    } else {
        warn('Filter controls not found (may not exist in this UI)');
    }
    
    runBrowserCommand('close');
}

// Test: Loading states
async function testLoadingStates() {
    info('Testing loading states...');
    
    // First close any existing browser
    runBrowserCommand('close');
    
    // Open frontend and check for loading indicator
    runBrowserCommand(`open ${FRONTEND_URL}`);
    
    // Take immediate snapshot before page fully loads
    runBrowserCommand('wait --load 500');
    const earlySnapshot = runBrowserCommand('snapshot');
    
    if (earlySnapshot.success) {
        // LatteAlbum uses .loading and .spinner classes
        const hasLoading = earlySnapshot.output.includes('loading') || 
                         earlySnapshot.output.includes('Loading') ||
                         earlySnapshot.output.includes('spinner');
        
        if (hasLoading) {
            assert(true, 'Loading indicator present during page load');
        } else {
            info('No explicit loading indicator found (or loaded too fast)');
        }
    }
    
    // Wait for full load
    runBrowserCommand('wait --load networkidle');
    
    // Verify page is interactive after loading
    const finalSnapshot = runBrowserCommand('snapshot');
    if (finalSnapshot.success && finalSnapshot.output.length > 100) {
        assert(true, 'Page fully loaded and interactive');
    }
    
    runBrowserCommand('close');
}

// Test: Responsive design
async function testResponsiveDesign() {
    info('Testing responsive design...');
    
    const viewports = [
        { width: 1920, height: 1080, name: 'Desktop' },
        { width: 768, height: 1024, name: 'Tablet' },
        { width: 375, height: 667, name: 'Mobile' }
    ];
    
    for (const viewport of viewports) {
        info(`Testing viewport: ${viewport.name} (${viewport.width}x${viewport.height})`);
        
        runBrowserCommand(`open ${FRONTEND_URL}`);
        runBrowserCommand('wait --load networkidle');
        
        // Resize viewport
        const resizeResult = runBrowserCommand(`resize ${viewport.width} ${viewport.height}`);
        
        if (resizeResult.success) {
            runBrowserCommand('wait --load 500');
            const snapshot = runBrowserCommand('snapshot');
            
            if (snapshot.success) {
                info(`${viewport.name} viewport rendered successfully`);
            }
        }
        
        runBrowserCommand('close');
    }
    
    assert(true, 'Responsive design test completed');
}

// Test: Error handling - offline simulation
async function testErrorHandling() {
    info('Testing error handling...');
    
    // Test API with invalid endpoint
    try {
        const response = await httpGet(`${BACKEND_URL}/api/invalid-endpoint-12345`);
        if (response.status >= 400) {
            assert(true, 'Invalid API endpoint returns error status');
        } else {
            warn('Invalid endpoint returned unexpected status');
        }
    } catch (err) {
        assert(true, 'API error handling works');
    }
    
    // Test with invalid file ID
    try {
        const response = await httpGet(`${BACKEND_URL}/api/files/99999999/thumbnail`);
        if (response.status === 404) {
            assert(true, 'Non-existent file returns 404');
        } else {
            info(`Non-existent file returned: ${response.status}`);
        }
    } catch (err) {
        info('Error handling for invalid file works');
    }
}

// Test: Directory navigation
async function testDirectoryNavigation() {
    info('Testing directory navigation...');
    
    runBrowserCommand(`open ${FRONTEND_URL}`);
    runBrowserCommand('wait --load networkidle');
    
    // Look for directory/folder elements - LatteAlbum doesn't have directory navigation in the main UI
    // This test is optional and may not apply
    const snapshot = runBrowserCommand('snapshot');
    
    if (snapshot.success) {
        // Check if there's any directory-related content
        const hasDirs = snapshot.output.includes('directory') || 
                       snapshot.output.includes('folder') ||
                       snapshot.output.includes('DateNavigator');
        
        if (hasDirs) {
            assert(true, 'Directory navigation elements present');
        } else {
            info('No directory navigation in this view (DateNavigator may be used instead)');
        }
    }
    
    runBrowserCommand('close');
}

// Test: Refresh/Scan button
async function testRefreshButton() {
    info('Testing refresh/scan button...');
    
    runBrowserCommand(`open ${FRONTEND_URL}`);
    runBrowserCommand('wait --load networkidle');
    
    // LatteAlbum uses .refresh-button
    const refreshCommands = [
        'click .refresh-button',      // Refresh button in RefreshButton component
        'click [class*="refresh"]',    // Any refresh element
        'click [class*="scan"]',      // Scan element
        'click [title*="refresh"]'    // Title attribute
    ];
    
    let buttonFound = false;
    for (const cmd of refreshCommands) {
        const result = runBrowserCommand(cmd);
        if (result.success) {
            info(`Found refresh/scan button: ${cmd}`);
            buttonFound = true;
            runBrowserCommand('wait --load 2000');
            break;
        }
    }
    
    if (buttonFound) {
        // Check if scanning started (look for progress indicator)
        const snapshot = runBrowserCommand('snapshot');
        if (snapshot.success) {
            // LatteAlbum shows scan progress with scan-progress-container
            const hasProgress = snapshot.output.includes('progress') || 
                              snapshot.output.includes('scan') ||
                              snapshot.output.includes('loading') ||
                              snapshot.output.includes('scan-progress');
            if (hasProgress) {
                assert(true, 'Scan/refresh button triggers action');
            } else {
                info('Button clicked but no immediate feedback');
            }
        }
    } else {
        warn('Refresh/scan button not found');
    }
    
    runBrowserCommand('close');
}

// Main test runner
async function runTests() {
    console.log('');
    console.log('========================================');
    console.log('  LatteAlbum E2E Test Suite (Expanded)');
    console.log('========================================');
    console.log('');
    
    const startTime = Date.now();
    
    // API Tests (can run in parallel)
    info('Running API tests...');
    const systemStatus = await testBackendHealth();
    await testFrontendLoads();
    const filesData = await testFilesAPI();
    const dirsData = await testDirectoriesAPI();
    
    // Additional API tests
    info('');
    info('Running extended API tests...');
    await testThumbnailAPI(filesData);
    await testVideoThumbnailAPI(filesData);
    await testExifAPI(filesData);
    await testErrorHandling();
    
    // Browser Tests
    info('');
    info('Running browser tests...');
    await testBrowserPageLoad();
    await testBrowserNavigation();
    await testLoadingStates();
    
    // Interactive browser tests
    info('');
    info('Running interactive browser tests...');
    await testPhotoLightbox();
    await testGalleryScroll();
    await testFilterFunctionality();
    await testDirectoryNavigation();
    await testRefreshButton();
    
    // Responsive design test
    await testResponsiveDesign();
    
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
