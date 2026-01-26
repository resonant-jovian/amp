    html.push_str("                const results = await response.json();\n");
    html.push_str("                logMessage('API', 'Raw response: ' + JSON.stringify(results).substring(0, 300), 'info');\n");
    html.push_str("                logMessage('API', 'Response received with ' + results.length + ' results', 'success');\n");
    html.push('
');
    html.push_str("                if (results.length === 0) {\n");
    html.push_str("                    logMessage('RESULT', 'No address found matching: ' + addressToSearch, 'warning');\n");
    html.push_str("                    updateStatus('❌ Address not found in Malmö');\n");
    html.push_str("                    return;\n");
    html.push_str("                }\n");
    html.push('
');
    html.push_str("                const result = results[0];\n");
    html.push_str("                logMessage('PARSE', 'Result object keys: ' + Object.keys(result).join(', '), 'info');\n");
    html.push_str("                \n");
    html.push_str("                // Try different property names for coordinates and address\n");
    html.push_str("                let x = result.x;\n");
    html.push_str("                let y = result.y;\n");
    html.push_str("                let name = result.adress || result.name || result.address || 'Unknown';\n");
    html.push_str("                \n");
    html.push_str("                logMessage('PARSE', 'Extracted: name=\\\"' + name + '\\\", x=' + x + ', y=' + y, 'info');\n");
    html.push('
');
    html.push_str("                if (!x || !y || x === undefined || y === undefined) {\n");
    html.push_str("                    logMessage('ERROR', 'Could not extract coordinates. Full result: ' + JSON.stringify(result), 'error');\n");
    html.push_str("                    updateStatus('❌ Coordinates not found in API response');\n");
    html.push_str("                    return;\n");
    html.push_str("                }\n");
    html.push('
');
    html.push_str("                logMessage('RESULT', 'Found: ' + name + ' at coordinates (' + x + ', ' + y + ')', 'success');\n");
    html.push('
');
    html.push_str("                // Build StadsAtlas URL with coordinates\n");
    html.push_str("                const mapUrl = 'https://stadsatlas.malmo.se/stadsatlas/#center=' + x + ',' + y + '&zoom=15';\n");
    html.push_str("                logMessage('MAP', 'Navigating to: ' + mapUrl.substring(0, 80) + '...', 'info');\n");
    html.push('
');
    html.push_str("                // Navigate iframe\n");
    html.push_str("                iframeElement.src = mapUrl;\n");
    html.push_str("                logMessage('MAP', 'iframe navigated successfully', 'success');\n");
    html.push_str("                updateStatus('✅ Map navigated to: ' + name);\n");
    html.push('
');
    html.push_str("            } catch (error) {\n");
    html.push_str("                logMessage('ERROR', 'Search failed: ' + error.message, 'error');\n");
    html.push_str("                updateStatus('❌ Error: ' + error.message);\n");
    html.push_str("            }\n");
    html.push_str("        }\n")