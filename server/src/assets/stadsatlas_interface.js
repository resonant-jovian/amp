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
    logToConsole('MAP', 'Building StadsAtlas URL with miljöparkering layer...');
    logToConsole('MAP', 'Trying multiple layer parameter formats');
    
    // Load map in iframe
    const iframeContainer = document.getElementById('map-container');
    const mapPlaceholder = iframeContainer.querySelector('.map-placeholder');
    const iframe = document.getElementById('stadsatlas-iframe');
    
    if (!iframe) {
        logToConsole('ERROR', 'iframe#stadsatlas-iframe not found');
        return;
    }
    
    // IMPORTANT: Hide placeholder FIRST before setting iframe src
    // This ensures Origo has proper space to render
    logToConsole('MAP', 'Hiding placeholder and preparing container for map rendering');
    mapPlaceholder.style.display = 'none';
    iframe.style.display = 'block';
    
    // Build URL with multiple layer parameter attempts
    const baseUrl = 'https://stadsatlas.malmo.se/stadsatlas/';
    const mapUrl = `${baseUrl}#center=${x},${y}&zoom=18&pin=${x},${y}&layers=miljoparkering_l&layerIds=miljoparkering_l&visibleLayers=miljoparkering_l`;
    
    logToConsole('MAP', `URL: ${mapUrl.substring(0, 100)}...`);
    
    // Force container dimensions for Origo
    iframeContainer.style.width = '100%';
    iframeContainer.style.height = '100%';
    iframe.style.width = '100%';
    iframe.style.height = '100%';
    logToConsole('MAP', `Container dimensions forced: ${iframeContainer.offsetWidth}x${iframeContainer.offsetHeight}`);
    
    // Set the iframe source AFTER ensuring container is visible and sized
    iframe.src = mapUrl;
    
    logToConsole('MAP', 'Map loaded in persistent container at top');
    logToConsole('LAYER', 'Attempted to activate miljöparkering layer via URL parameters');
    logToConsole('LAYER', 'If layer not visible, manually activate it using the Layers panel in the map');
    
    // Wait for iframe to load
    iframe.onload = function() {
        logToConsole('MAP', 'Iframe loaded successfully');
        logToConsole('LAYER', 'Note: Layer activation via URL may require manual confirmation in StadsAtlas UI');
        
        // Double-check dimensions after iframe load
        logToConsole('DEBUG', `Final container dimensions: ${iframeContainer.offsetWidth}x${iframeContainer.offsetHeight}`);
        logToConsole('DEBUG', `Final iframe dimensions: ${iframe.offsetWidth}x${iframe.offsetHeight}`);
    };
}

function updateSearchStatus(address) {
    const statusEl = document.getElementById('search-status');
    if (statusEl) {
        statusEl.textContent = `✓ Found: ${address}`;
    }
    
    const statusIndicator = document.getElementById('status-indicator');
    if (statusIndicator) {
        statusIndicator.textContent = `✓ Map loaded: ${address}`;
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
    logToConsole('READY', 'AMP Testing Interface loaded. Map section is persistent at top, tabs cycle below.');
    
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
        // Force dimension calculation
        const computedStyle = window.getComputedStyle(mapContainer);
        const width = computedStyle.width;
        const height = computedStyle.height;
        logToConsole('DEBUG', `Map container computed dimensions: ${width} x ${height}`);
        
        // Ensure minimum height is met
        if (mapContainer.offsetHeight === 0) {
            logToConsole('DEBUG', 'WARNING: Map container height is 0! Applying emergency fix.');
            mapContainer.style.height = '500px';
        }
        
        // Ensure minimum width is met
        if (mapContainer.offsetWidth === 0) {
            logToConsole('DEBUG', 'WARNING: Map container width is 0! Applying emergency fix.');
            mapContainer.style.width = '100%';
        }
    }
    
    // Set Data tab as active by default (but DON'T trigger searches yet)
    const dataTab = document.querySelector('[onclick*="switchTab(event, 2)"]');
    if (dataTab) {
        // Manually set active state without triggering click
        document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
        document.querySelectorAll('.tab-btn').forEach(el => el.classList.remove('active'));
        const tabContent = document.getElementById('tab2');
        if (tabContent) tabContent.classList.add('active');
        if (dataTab) dataTab.classList.add('active');
        logToConsole('TAB', 'Data tab activated on page load');
    }
    
    // Auto-load map with correlation address on page load (ONLY ONCE)
    logToConsole('INIT', 'Checking for auto-load on page load...');
    if (shouldAutoLoad && !hasAutoLoaded && searchInput && searchInput.value && searchInput.value.trim() !== '' && searchInput.value !== '{ADDRESS}') {
        hasAutoLoaded = true; // Prevent double auto-load
        logToConsole('INIT', `Auto-loading map for: ${searchInput.value}`);
        // Small delay to ensure everything is initialized
        setTimeout(searchAddress, 300);
    }
});
