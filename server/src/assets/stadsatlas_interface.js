const logs = [];
const addressToSearch = document.querySelector('.header .address')?.textContent?.trim() || 'Unknown';

function logMessage(category, message, type = 'info') {
    const timestamp = new Date().toLocaleTimeString();
    const logEntry = {timestamp, category, message, type};
    logs.push(logEntry);

    console.log('[AMP] [' + timestamp + '] [' + category + '] ' + message);

    const logsDiv = document.getElementById('message-logs');
    if (logsDiv) {
        const entry = document.createElement('div');
        entry.className = 'log-entry ' + type;
        entry.innerHTML = '<span class="log-timestamp">[' + timestamp + ']</span> <strong>' + category + ':</strong> ' + message;
        logsDiv.appendChild(entry);
        logsDiv.scrollTop = logsDiv.scrollHeight;
    }
}

function updateStatus(status, statusId = 'search-status') {
    const statusDiv = document.getElementById(statusId);
    if (statusDiv) {
        statusDiv.textContent = status;
    }
}

function switchTab(event, tabNumber) {
    const tabs = document.querySelectorAll('.tab-content');
    tabs.forEach(function(tab) { tab.classList.remove('active'); });
    const btns = document.querySelectorAll('.tab-btn');
    btns.forEach(function(btn) { btn.classList.remove('active'); });
    document.getElementById('tab' + tabNumber).classList.add('active');
    event.target.classList.add('active');
}

function closeMap() {
    const mapContainer = document.getElementById('map-container');
    if (mapContainer) {
        mapContainer.style.display = 'none';
        const iframe = document.getElementById('stadsatlas-iframe');
        if (iframe) {
            iframe.src = '';
        }
    }
}

async function searchAddress() {
    logMessage('SEARCH', 'Starting address search for: ' + addressToSearch, 'info');
    updateStatus('⏳ Searching for: ' + addressToSearch);

    try {
        const searchUrl = 'https://geo.malmo.se/api/search?q=' + encodeURIComponent(addressToSearch);
        logMessage('API', 'Calling: ' + searchUrl.substring(0, 60) + '...', 'info');

        const response = await fetch(searchUrl);
        if (!response.ok) {
            throw new Error('API returned status ' + response.status);
        }

        const results = await response.json();
        logMessage('API', 'Response received with ' + results.length + ' results', 'success');

        if (results.length === 0) {
            logMessage('RESULT', 'No address found matching: ' + addressToSearch, 'warning');
            updateStatus('❌ Address not found in Malmö');
            return;
        }

        const result = results[0];
        logMessage('PARSE', 'Result keys: ' + Object.keys(result).join(', '), 'info');
        
        // Parse Malmö API response with WKT GEOM format
        const name = result.NAMN || result.name || result.adress || 'Unknown';
        let x, y;
        
        // Extract from WKT POINT format: POINT(X Y)
        if (result.GEOM) {
            const match = result.GEOM.match(/POINT\s*\(([^\s]+)\s+([^)]+)\)/);
            if (match) {
                x = parseFloat(match[1]);
                y = parseFloat(match[2]);
                logMessage('PARSE', 'Extracted WKT: x=' + x + ', y=' + y, 'info');
            }
        }
        
        // Fallback to x, y properties
        if (!x || !y) {
            x = result.x;
            y = result.y;
            if (x && y) logMessage('PARSE', 'Using x, y properties: x=' + x + ', y=' + y, 'info');
        }
        
        if (!x || !y || isNaN(x) || isNaN(y)) {
            logMessage('ERROR', 'Missing coordinates in response', 'error');
            updateStatus('❌ Coordinates not found');
            return;
        }
        
        logMessage('RESULT', 'Found: ' + name + ' at (' + x + ', ' + y + ')', 'success');

        // Build StadsAtlas URL with:
        // - center: center the map on the coordinates
        // - zoom: zoom level 18 for close-up view
        // - pin: add a pin/marker at the exact address location
        // - layers: activate miljödata layer
        
        const mapUrl = 'https://stadsatlas.malmo.se/stadsatlas/#center=' + x + ',' + y + '&zoom=18&pin=' + x + ',' + y + '&layers=miljoparkering_l';
        logMessage('MAP', 'Building StadsAtlas URL with pin and miljoparkering layer...', 'info');
        logMessage('MAP', 'URL: ' + mapUrl.substring(0, 80) + '...', 'info');

        // Load in nested iframe instead of new tab
        const mapContainer = document.getElementById('map-container');
        const iframe = document.getElementById('stadsatlas-iframe');
        
        if (mapContainer && iframe) {
            iframe.src = mapUrl;
            mapContainer.style.display = 'flex';
            logMessage('MAP', 'Map loaded in nested interface', 'success');
            updateStatus('✅ Map loaded: ' + name);
        } else {
            logMessage('ERROR', 'Map container not found in DOM', 'error');
            updateStatus('❌ Error: Map container not available');
        }

    } catch (error) {
        logMessage('ERROR', 'Search failed: ' + error.message, 'error');
        updateStatus('❌ Error: ' + error.message);
    }
}

// Initial status
window.addEventListener('load', function() {
    logMessage('READY', 'AMP Testing Interface loaded. Ready to search address.', 'info');
});
