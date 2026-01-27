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
    
    // ✅ CORS FIX: Embed config directly instead of fetching
    const stadsatlasConfig = {
        "controls": [
            {
                "name": "home",
                "options": {
                    "zoomOnStart": true
                }
            },
            {
                "name": "mapmenu",
                "options": {
                    "isActive": false
                }
            },
            {
                "name": "sharemap"
            },
            {
                "name": "print"
            },
            {
                "name": "search",
                "options": {
                    "url": "https://geo.malmo.se/api/search",
                    "searchAttribute": "NAMN",
                    "titleAttribute": "TYPE",
                    "contentAttribute": "NAMN",
                    "geometryAttribute": "GEOM",
                    "hintText": "Sök adress eller platser...",
                    "minLength": 2,
                    "groupSuggestions": true,
                    "maxZoomLevel": "8"
                }
            }
        ],
        "pageSettings": {
            "footer": {
                "img": "img/png/malmo-logo.png",
                "url": "https://malmo.se"
            },
            "mapGrid": {
                "visible": false
            }
        },
        "projectionCode": "EPSG:3008",
        "projectionExtent": [
            -72234.21,
            6098290.04,
            573714.68,
            7702218.01
        ],
        "proj4Defs": [
            {
                "code": "EPSG:3008",
                "projection": "+proj=tmerc +lat_0=0 +lon_0=13.5 +k=1 +x_0=150000 +y_0=0 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs",
                "alias": "SWEREF 99 1330"
            },
            {
                "code": "EPSG:3006",
                "projection": "+proj=utm +zone=33 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs",
                "alias": "SWEREF 99 TM"
            },
            {
                "code": "EPSG:4326",
                "projection": "+proj=longlat +datum=WGS84 +no_defs",
                "alias": "WGS 84"
            }
        ],
        "extent": [
            34364.701176279224,
            6105850.2404539045,
            180404.6059212964,
            6198940.128187204
        ],
        "center": [
            120844,
            6161226
        ],
        "zoom": 3,
        "constrainResolution": true,
        "resolutions": [
            66.6751333502667,
            33.86673440013547,
            25.400050800101603,
            16.933367200067735,
            12.700025400050801,
            8.466683600033868,
            4.233341800016934,
            2.116670900008467,
            1.0583354500042335,
            0.5291677250021167,
            0.26458386250105836
        ],
        "featureinfoOptions": {
            "infowindow": "overlay",
            "pinning": false,
            "hitTolerance": 10
        },
        "source": {
            "Bakgrundskarta_nedtonad_3008_text": {
                "url": "https://gis.malmo.se/arcgis/rest/services/baskartor/Bakgrundskarta_nedtonad_3008_text/MapServer/tile/{z}/{y}/{x}",
                "tileGrid": {
                    "resolutions": [
                        66.6751333502667,
                        33.86673440013547,
                        25.400050800101603,
                        16.933367200067735,
                        12.700025400050801,
                        8.466683600033868,
                        4.233341800016934,
                        2.116670900008467,
                        1.0583354500042335,
                        0.5291677250021167,
                        0.26458386250105836,
                        0.13229193125052918,
                        0.06614596562526459,
                        0.026458386250105836
                    ],
                    "origin": [
                        -399.9999999999999,
                        9006799.254740989
                    ],
                    "extent": [
                        -1678505.1838360203,
                        4665380,
                        2431912.7361639794,
                        8775797.92
                    ]
                }
            },
            "geoserver-malmows-wms-atlasp1": {
                "url": "https://stadsatlas.malmo.se/geoserver/malmows/wms"
            }
        },
        "groups": [
            {
                "name": "background",
                "title": "Bakgrundskartor",
                "groups": []
            }
        ],
        "layers": [
            {
                "name": "Bakgrundskarta_nedtonad_3008_text",
                "title": "Bakgrundskarta nedtonad",
                "group": "background",
                "source": "Bakgrundskarta_nedtonad_3008_text",
                "type": "XYZ",
                "style": "karta-nedtonad",
                "attribution": "© CC0 Malmö stad",
                "queryable": false,
                "visible": true
            },
            {
                "name": "miljoparkeringl",
                "title": "Miljöparkering",
                "group": "background",
                "source": "geoserver-malmows-wms-atlasp1",
                "type": "WMS",
                "visible": true
            }
        ],
        "styles": {
            "karta-nedtonad": [
                [
                    {
                        "image": {
                            "src": "img/png/gra.png"
                        }
                    }
                ]
            ]
        }
    };
    
    // Create the Origo map HTML as a data URI
    const origoMapHTML = `<!DOCTYPE html>
<html>
<head>
    <title>AMP Correlation Map - Origo Embed</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="https://stadsatlas.malmo.se/stadsatlas/css/style.css">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        html, body {
            height: 100%;
            width: 100%;
            overflow: hidden;
        }
        #map {
            width: 100%;
            height: 100%;
        }
    </style>
</head>
<body>
    <div id="map"></div>
    
    <script src="https://stadsatlas.malmo.se/stadsatlas/js/origo.js"><\/script>
    <script src="https://cdn.jsdelivr.net/npm/ol@7.5.0/dist/ol.min.js"><\/script>
    <script>
        // Parse URL parameters
        const urlParams = new URLSearchParams(window.location.hash.substring(1));
        const centerStr = urlParams.get('center');
        const zoom = parseInt(urlParams.get('zoom')) || 18;
        
        let x = 120844;  // Default center
        let y = 6161226;
        
        // Parse center coordinates
        if (centerStr) {
            const coords = centerStr.split(',');
            if (coords.length === 2) {
                x = parseFloat(coords[0]);
                y = parseFloat(coords[1]);
            }
        }
        
        console.log('🗺️ Origo Map Init - Center:', [x, y], 'Zoom:', zoom);
        
        const stadsatlasConfig = ${JSON.stringify(stadsatlasConfig)};
        
        // Initialize Origo with embedded config (no CORS issues!)
        try {
            console.log('📡 Using embedded StadsAtlas config (CORS bypass)');
            
            // Create map
            window.map = o.create({
                target: 'map',
                ...stadsatlasConfig,
                center: [x, y],
                zoom: zoom,
                resolutions: stadsatlasConfig.resolutions
            });
            
            // After a delay, enable layers using Origo's API
            setTimeout(() => {
                console.log('✏️ Setting up layers via Origo API...');
                
                // Method 1: Toggle layers by name using Origo's toggleLayer API
                try {
                    // Turn on Bakgrund layers
                    window.map.toggleLayer('Bakgrundskarta_nedtonad_3008_text');
                    console.log('✅ Toggled Bakgrundskarta_nedtonad_3008_text');
                } catch (e) {
                    console.log('🙈 Bakgrund toggle failed:', e.message);
                }
                
                try {
                    // Turn on miljöparkering layer
                    window.map.toggleLayer('miljoparkeringl');
                    console.log('✅ Toggled miljoparkeringl');
                } catch (e) {
                    console.log('🙈 Miljöparkering toggle failed:', e.message);
                }
                
                // Method 2: Try to access layers array and enable by visibility
                const layers = window.map.getLayers().getArray();
                console.log(\`📊 Total layers: \${layers.length}\`);
                
                layers.forEach((layer, idx) => {
                    const name = layer.get('name') || '';
                    
                    // Enable bakgrund
                    if (name.toLowerCase().includes('bakgrund')) {
                        layer.setVisible(true);
                        console.log(\`✅ Enabled layer[\${idx}]: \${name}\`);
                    }
                    
                    // Enable miljöparkering
                    if (name === 'miljoparkeringl') {
                        layer.setVisible(true);
                        console.log(\`✅ Enabled layer[\${idx}]: \${name}\`);
                    }
                });
                
                // Add pin to map
                addPinMarker(x, y);
                
            }, 1000);  // Wait 1 second for map to fully initialize
        } catch (error) {
            console.error('❌ Failed to initialize map:', error);
        }
        
        // Function to add a pin marker
        function addPinMarker(x, y) {
            console.log('📏 Adding pin at:', [x, y]);
            
            const vectorSource = new ol.source.Vector({
                features: [
                    new ol.Feature({
                        geometry: new ol.geom.Point([x, y])
                    })
                ]
            });
            
            const vectorLayer = new ol.layer.Vector({
                source: vectorSource,
                zIndex: 9999,  // Ensure it appears on top
                style: new ol.style.Style({
                    image: new ol.style.Icon({
                        anchor: [0.5, 1],
                        anchorXUnits: 'fraction',
                        anchorYUnits: 'fraction',
                        scale: 1.2,
                        src: 'data:image/svg+xml;utf8,' + encodeURIComponent(\`
                            <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"32\" height=\"44\" viewBox=\"0 0 32 44\">
                                <defs>
                                    <filter id=\"shadow\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\">
                                        <feDropShadow dx=\"0\" dy=\"2\" stdDeviation=\"2\" flood-opacity=\"0.3\"/>
                                    </filter>
                                </defs>
                                <path d=\"M16 2 C9 2, 3 8, 3 16 C3 26, 16 44, 16 44 S29 26, 29 16 C29 8, 23 2, 16 2\" 
                                      fill=\"#FF4444\" stroke=\"white\" stroke-width=\"2.5\" filter=\"url(#shadow)\"/>
                                <circle cx=\"16\" cy=\"15\" r=\"6\" fill=\"white\" stroke=\"#FF4444\" stroke-width=\"1.5\"/>
                            </svg>
                        \`)
                    })
                })
            });
            
            // Add to map
            if (window.map) {
                window.map.addLayer(vectorLayer);
                console.log('✅ Pin added to map');
            } else {
                console.error('❌ Map object not available');
            }
        }
    <\/script>
</body>
</html>`;
    
    // Encode as data URI
    const dataUri = 'data:text/html;charset=utf-8,' + encodeURIComponent(origoMapHTML);
    
    logToConsole('MAP', `Creating embedded map with data URI...`);
    logToConsole('MAP', `Setting iframe src...`);
    
    iframe.src = dataUri + `#center=${x},${y}&zoom=18`;
    
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
