// ===================================================================
// AMP StadsAtlas Interface - JavaScript Controller
// Handles address search, map loading, tab switching, and logging
// ===================================================================

const BASE_URL = 'https://geo.malmo.se/api/search?q=';

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
    document.getElementById('search-status').textContent = `❌ Error: ${error}`;
}

function searchAddress() {
    const searchBox = document.querySelector('input[placeholder*="Sök address"]') || 
                      document.querySelector('input[type="text"]');
    const address = searchBox ? searchBox.value : 'Master Henriksgatan 2';
    
    logToConsole('SEARCH', `Starting address search for: ${address}`);
    logToConsole('API', `Calling: ${BASE_URL}${encodeURIComponent(address)}...`);
    
    fetch(`${BASE_URL}${encodeURIComponent(address)}`)
        .then(response => {
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return response.json();
        })
        .then(data => {
            if (!data || data.length === 0) {
                throw new Error('No results found');
            }
            
            logToConsole('API', `Response received with ${data.length} results`);
            
            const result = data[0];
            logToConsole('PARSE', `Result keys: ${Object.keys(result).join(', ')}`);
            
            // Parse WKT format: POINT(X Y)
            const geom = result.GEOM || '';
            const match = geom.match(/POINT\((\S+)\s+(\S+)\)/);
            
            if (!match) {
                throw new Error('Could not parse coordinates from WKT');
            }
            
            const x = parseFloat(match[1]);
            const y = parseFloat(match[2]);
            
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
    
    // Build URL with multiple layer parameter attempts
    const baseUrl = 'https://stadsatlas.malmo.se/stadsatlas/';
    const mapUrl = `${baseUrl}#center=${x},${y}&zoom=18&pin=${x},${y}&layers=miljoparkering_l&layerIds=miljoparkering_l&visibleLayers=miljoparkering_l`;
    
    logToConsole('MAP', `URL: ${mapUrl.substring(0, 100)}...`);
    
    // Load map in iframe
    const iframeContainer = document.getElementById('map-container');
    const mapPlaceholder = iframeContainer.querySelector('.map-placeholder');
    const iframe = document.getElementById('stadsatlas-iframe');
    
    if (!iframe) {
        logToConsole('ERROR', 'iframe#stadsatlas-iframe not found');
        return;
    }
    
    // Show iframe, hide placeholder
    mapPlaceholder.style.display = 'none';
    iframe.style.display = 'block';
    iframe.src = mapUrl;
    
    logToConsole('MAP', 'Map loaded in persistent container at top');
    logToConsole('LAYER', 'Attempted to activate miljöparkering layer via URL parameters');
    logToConsole('LAYER', 'If layer not visible, manually activate it using the Layers panel in the map');
    
    // Wait for iframe to load
    iframe.onload = function() {
        logToConsole('MAP', 'Iframe loaded successfully');
        logToConsole('LAYER', 'Note: Layer activation via URL may require manual confirmation in StadsAtlas UI');
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
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    logToConsole('READY', 'AMP Testing Interface loaded. Map section is persistent at top, tabs cycle below.');
    
    // Set up search button
    const searchBtn = document.querySelector('.control-button');
    if (searchBtn) {
        searchBtn.onclick = searchAddress;
    }
    
    // Ensure iframe container has proper dimensions
    const mapContainer = document.getElementById('map-container');
    if (mapContainer) {
        // Force dimension calculation
        const computedStyle = window.getComputedStyle(mapContainer);
        logToConsole('DEBUG', `Map container computed dimensions: ${computedStyle.width} x ${computedStyle.height}`);
        
        // Ensure minimum height is met
        if (mapContainer.offsetHeight === 0) {
            logToConsole('DEBUG', 'WARNING: Map container height is 0! Applying emergency fix.');
            mapContainer.style.height = '500px';
        }
    }
    
    // Set Data tab as active by default (already done in HTML, but ensure it)
    const dataTab = document.querySelector('[onclick*="switchTab(event, 2)"]');
    if (dataTab) {
        dataTab.click();
    }
});
