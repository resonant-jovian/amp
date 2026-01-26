        <!-- Tab 2: Instructions -->
        <div id="tab2" class="tab-content">
            <h1>\ud83d\udccb StadsAtlas Verification Instructions</h1>
            
            <div class="instruction">
                \u2713 Follow these steps to verify the address in StadsAtlas (Tab 1)
            </div>
            
            <div class="address-display">{}</div>
            
            <div style="background: #ffe0e0; border: 1px solid #ff9999; color: #cc0000; padding: 15px; border-radius: 4px; margin: 15px 0;">
                <strong>\u26a0\ufe0f Automation Note:</strong> The following steps show what the automation attempts to do. If it fails, follow the manual instructions below.
            </div>
            
            <h3 style="margin-top: 25px; color: #333;">Automated Steps (may fail due to cross-origin restrictions):</h3>
            
            <div class="steps">
                <div class="step" style="background: #fff3cd; border-left-color: #ff9800;">
                    <strong>Automated: Enable Milj\u00f6parkering</strong><br>
                    <span style="font-size: 12px; color: #666; margin-top: 8px; display: block;">
                    Attempts: Layers icon \u2192 Chevrons \u00d7 3 \u2192 Radio button for Milj\u00f6parkering
                    </span>
                    <div style="background: #ffcccc; color: #cc0000; padding: 10px; border-radius: 3px; margin-top: 10px; font-size: 12px;">
                        \u274c <strong>If fails:</strong> Manually click layers icon, then chevrons, then select Milj\u00f6parkering radio button
                    </div>
                </div>
                
                <div class="step" style="background: #fff3cd; border-left-color: #ff9800;">
                    <strong>Automated: Search for Address</strong><br>
                    <span style="font-size: 12px; color: #666; margin-top: 8px; display: block;">
                    Attempts: Focus search field and enter address automatically
                    </span>
                    <div style="background: #ffcccc; color: #cc0000; padding: 10px; border-radius: 3px; margin-top: 10px; font-size: 12px;">
                        \u274c <strong>If fails:</strong> Manually click search field and type: <strong>{}</strong>
                    </div>
                </div>
            </div>
            
            <h3 style="margin-top: 30px; color: #333; border-bottom: 2px solid #ddd; padding-bottom: 10px;">Manual Fallback Instructions (if automation fails):</h3>
            
            <div class="steps">
                <div class="step">
                    Click the <strong>layers icon</strong> (first icon in top left toolbar)
                </div>
                <div class="step">
                    Click the <strong>chevron right</strong> button (arrow pointing right)
                </div>
                <div class="step">
                    Click the <strong>chevron right</strong> button again
                </div>
                <div class="step">
                    Click the <strong>chevron right</strong> button once more
                </div>
                <div class="step">
                    Click the <strong>radio button</strong> (circle) to enable <strong>Milj\u00f6parkering</strong>
                    <div style="background: #ffcccc; color: #cc0000; padding: 10px; border-radius: 3px; margin-top: 8px; font-size: 12px;">
                        \u274c <strong>Error:</strong> If you cannot find the radio button or it won't respond, the layers menu may not have loaded correctly. Try refreshing Tab 1.
                    </div>
                </div>
                <div class="step">
                    Click in the <strong>search field</strong> at the top (labeled \"S\u00f6k adresser eller platser...\")
                </div>
                <div class="step">
                    Type this address: <strong>{}</strong>
                    <div style="background: #ffcccc; color: #cc0000; padding: 10px; border-radius: 3px; margin-top: 8px; font-size: 12px;">
                        \u274c <strong>Error:</strong> If the search field is not accepting input, try clicking it again or refreshing the page.
                    </div>
                </div>
                <div class="step">
                    Press <strong>Enter</strong> to search
                </div>
            </div>
            
            <div class="note" style="background: #e3f2fd; border-left: 4px solid #2196F3;">
                \ud83d\udca1 <strong>Tip:</strong> Use Tab 3 to see the correlation result data while you verify it in StadsAtlas (Tab 1). Cross-reference the data to ensure accuracy.
            </div>
            
            <div style="margin-top: 30px; padding: 20px; background: #fff9e6; border-radius: 4px; border-left: 4px solid #ff6b6b;">
                <strong>\u26a0\ufe0f Common Issues:</strong>
                <ul style="margin: 10px 0; padding-left: 20px;">
                    <li><strong>Layers menu not responding:</strong> Refresh Tab 1 and try again</li>
                    <li><strong>Search not working:</strong> Make sure Milj\u00f6parkering is actually selected (radio button filled)</li>
                    <li><strong>Address not found:</strong> Verify the address spelling and format matches Swedish address format</li>
                    <li><strong>Cross-origin error:</strong> Some browser security settings may prevent automation. Use manual steps instead.</li>
                </ul>
            </div>
        </div>