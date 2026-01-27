// ===================================================================
// AMP StadsAtlas Interface - JavaScript Controller
// Handles address search, map loading, tab switching, and logging
// ===================================================================

const BASE_URL = 'https://geo.malmo.se/api/search';
let shouldAutoLoad = true; // Flag to auto-load on page load
let hasAutoLoaded = false; // Track if auto-load has already happened

function logToConsole(prefix, message) {
    const timestamp = new Date().toLocaleTimeString();
    console.log(`[AMP] [${timestamp}] [${prefix}] ${message}`);
    
    // Also add to on-page debug log
    const logDiv = document.getElementById('message-logs');
    if (logDiv) {
        const entry = document.createElement('div');
        entry.className = 'log-entry';
        entry.innerHTML = `<span class="log-timestamp">[${timestamp}]</span> [${prefix}] ${message}`;
        logDiv.appendChild(entry);
        logDiv.scrollTop = logDiv.scrollHeight;
    }
}

function handleApiError(error) {
    logToConsole('ERROR', `API Error: ${error}`);
    const statusEl = document.getElementById('search-status');
    if (statusEl) {
        statusEl.textContent = `❌ Error: ${error}`;
    }
    const statusIndicator = document.getElementById('status-indicator');
    if (statusIndicator) {
        statusIndicator.textContent = `❌ Error: ${error}`;
    }
}

function searchAddress() {
    const searchInput = document.getElementById('address-input');
    const address = searchInput ? searchInput.value.trim() : '';
    
    if (!address || address.length === 0) {
        handleApiError('Please enter an address to search');
        return;
    }
    
    logToConsole('SEARCH', `Starting address search for: ${address}`);
    
    // Build URL using URLSearchParams for proper encoding
    const params = new URLSearchParams();
    params.append('q', address);
    const fullUrl = `${BASE_URL}?${params.toString()}`;
    
    logToConsole('API', `Calling: ${fullUrl}`);
    
    fetch(fullUrl)
        .then(response => {
            logToConsole('API', `Response status: ${response.status}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return response.json();
        })
        .then(data => {
            logToConsole('API', `Raw response data received`);
            
            if (!data) {
                throw new Error('Response is null or undefined');
            }
            
            if (!Array.isArray(data)) {
                logToConsole('API', `Response type: ${typeof data}`);
                logToConsole('API', `Response keys: ${Object.keys(data).join(', ')}`);
                throw new Error('Response is not an array');
            }
            
            if (data.length === 0) {
                throw new Error(`No results found for "${address}" - Try a different address or check spelling`);
            }
            
            logToConsole('API', `Response received with ${data.length} results`);
            
            const result = data[0];
            logToConsole('PARSE', `Result keys: ${Object.keys(result).join(', ')}`);
            logToConsole('PARSE', `First result: ${JSON.stringify(result).substring(0, 100)}...`);
            
            // Parse WKT format: POINT (X Y) or POINT(X Y)
            const geom = result.GEOM || '';
            logToConsole('PARSE', `GEOM field value: ${geom}`);
            
            // Updated regex to handle both "POINT (X Y)" and "POINT(X Y)" formats
            const match = geom.match(/POINT\s*\(([^\s]+)\s+([^\)]+)\)/);
            
            if (!match) {
                throw new Error(`Could not parse WKT from GEOM: "${geom}" - Expected format: POINT(X Y)`);
            }
            
            const x = parseFloat(match[1]);
            const y = parseFloat(match[2]);
            
            if (isNaN(x) || isNaN(y)) {
                throw new Error(`Parsed values are not numbers: x=${match[1]}, y=${match[2]}`);
            }
            
            logToConsole('PARSE', `Extracted WKT: x=${x}, y=${y}`);
            logToConsole('RESULT', `Found: ${result.NAMN} at (${x}, ${y})`);
            
            loadMapWithAddress(result.NAMN, x, y);
            updateSearchStatus(result.NAMN);
        })
        .catch(error => handleApiError(error.message));
}

function loadMapWithAddress(address, x, y) {
    logToConsole('MAP', '=== STARTING MAP LOAD ===');
    logToConsole('MAP', `Loading map for: "${address}" at (${x}, ${y})`);
    
    const iframe = document.getElementById('stadsatlas-iframe');
    
    if (!iframe) {
        logToConsole('ERROR', 'iframe#stadsatlas-iframe not found');
        return;
    }
    
    logToConsole('MAP', `✓ iframe element found`);
    
    // Load our custom origo_map.html with coordinates
    // This file will handle layer activation automatically
    const mapUrl = `origo_map.html#center=${x},${y}&zoom=18`;
    
    logToConsole('MAP', `URL: ${mapUrl}`);
    logToConsole('MAP', `Setting iframe src to custom Origo map...`);
    
    iframe.src = mapUrl;
    
    logToConsole('MAP', '✓ iframe.src set successfully');
    logToConsole('MAP', '');
    logToConsole('LAYERS', '✅ AUTOMATIC LAYER ACTIVATION:');
    logToConsole('LAYERS', 'The map should now:');
    logToConsole('LAYERS', '  1. Display the background tiles (Bakgrundskarta)');
    logToConsole('LAYERS', '  2. Show the Miljöparkering layer (if data exists)');
    logToConsole('LAYERS', '  3. Display a red pin at your address');
    logToConsole('LAYERS', '');
    logToConsole('MAP', '💡 If layers don\'t appear:');
    logToConsole('MAP', '  - Check browser console (F12) for JavaScript errors');
    logToConsole('MAP', '  - Zoom in/out on the map');
    logToConsole('MAP', '  - Wait 2-3 seconds for layers to load');
    logToConsole('MAP', '  - Try refreshing the page');
    logToConsole('MAP', '');
    logToConsole('MAP', '=== MAP LOAD COMPLETE ===');
    
    // Setup iframe load/error handlers
    iframe.onload = function() {
        logToConsole('MAP', '✓ Iframe loaded successfully');
    };
    
    iframe.onerror = function() {
        logToConsole('ERROR', '✗ Iframe failed to load - check network connection');
    };
}

function updateSearchStatus(address) {
    const statusEl = document.getElementById('search-status');
    if (statusEl) {
        statusEl.textContent = `✓ Found: ${address} - Loading map with automatic layers...`;
    }
    
    const statusIndicator = document.getElementById('status-indicator');
    if (statusIndicator) {
        statusIndicator.textContent = `✓ Map loaded for: ${address}`;
    }
}

function switchTab(event, tabNum) {
    // Hide all content
    document.querySelectorAll('.tab-content').forEach(el => {
        el.classList.remove('active');
    });
    
    // Remove active state from buttons
    document.querySelectorAll('.tab-btn').forEach(el => {
        el.classList.remove('active');
    });
    
    // Show selected tab
    const tabContent = document.getElementById(`tab${tabNum}`);
    if (tabContent) {
        tabContent.classList.add('active');
    }
    
    // Mark button as active
    if (event && event.target) {
        event.target.classList.add('active');
    }
    
    logToConsole('TAB', `Switched to tab ${tabNum}`);
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    logToConsole('READY', 'AMP Testing Interface initialized');
    logToConsole('READY', '');
    logToConsole('INFO', '📍 How to use this interface:');
    logToConsole('INFO', '  1. Top: Map display (updated when you search)');
    logToConsole('INFO', '  2. Middle: Control panel with address search');
    logToConsole('INFO', '  3. Bottom: Tabs for instructions, data, and debug info');
    logToConsole('INFO', '');
    logToConsole('INFO', '✨ NEW: Layers now enable automatically!');
    logToConsole('INFO', '   (Background tiles + Miljöparkering + Red pin)');
    logToConsole('INFO', '');
    
    // Set up search button
    const searchBtn = document.querySelector('.control-button');
    if (searchBtn) {
        searchBtn.onclick = searchAddress;
    }
    
    // Also allow Enter key to search
    const searchInput = document.getElementById('address-input');
    if (searchInput) {
        searchInput.addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                searchAddress();
            }
        });
    }
    
    // Ensure iframe container has proper dimensions
    const mapContainer = document.getElementById('map-container');
    if (mapContainer) {
        const computedStyle = window.getComputedStyle(mapContainer);
        const width = computedStyle.width;
        const height = computedStyle.height;
        logToConsole('DEBUG', `Map container dimensions: ${width} x ${height}`);
        
        // Emergency fixes if dimensions are zero
        if (mapContainer.offsetHeight === 0) {
            logToConsole('DEBUG', '⚠️  Map container height is 0 - applying emergency fix');
            mapContainer.style.height = '500px';
        }
        
        if (mapContainer.offsetWidth === 0) {
            logToConsole('DEBUG', '⚠️  Map container width is 0 - applying emergency fix');
            mapContainer.style.width = '100%';
        }
    }
    
    // Set Data tab as active by default
    const dataTab = document.querySelector('[onclick*="switchTab(event, 2)"]');
    if (dataTab) {
        document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
        document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
        const tabContent = document.getElementById('tab2');
        if (tabContent) tabContent.classList.add('active');
        if (dataTab) dataTab.classList.add('active');
    }
    
    // Auto-load map with correlation address on page load (ONLY ONCE)
    if (shouldAutoLoad && !hasAutoLoaded && searchInput && searchInput.value && searchInput.value.trim() !== '' && searchInput.value !== '{ADDRESS}') {
        hasAutoLoaded = true;
        logToConsole('INIT', `Auto-loading map for: ${searchInput.value}`);
        setTimeout(searchAddress, 300);
    }
});
