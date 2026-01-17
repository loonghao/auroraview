#!/usr/bin/env python
"""Demo showcasing wry WebView features.

This example demonstrates various wry-powered features available in AuroraView:

1. Splash Overlay - Show loading animation while page loads
2. Page Load Events - Track page loading state
3. Document Title Changes - React to title updates
4. File Downloads - Enable and track downloads (with real download buttons)
5. Custom User-Agent - Set custom browser identification
6. Proxy Configuration - Route traffic through proxy server

Run this example:
    python examples/wry_features_demo.py
"""

from auroraview import run_app


def main():
    """Run the wry features demo."""
    # HTML content demonstrating various features
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>AuroraView - Wry Features Demo</title>
        <style>
            * { box-sizing: border-box; margin: 0; padding: 0; }
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
                color: #e0e0e0;
                min-height: 100vh;
                padding: 20px;
            }
            h1 {
                text-align: center;
                margin-bottom: 30px;
                background: linear-gradient(90deg, #00d4ff, #7b2cbf);
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
            }
            .feature-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                gap: 20px;
                max-width: 1200px;
                margin: 0 auto;
            }
            .feature-card {
                background: rgba(255,255,255,0.05);
                border: 1px solid rgba(255,255,255,0.1);
                border-radius: 12px;
                padding: 20px;
            }
            .feature-card h3 {
                color: #00d4ff;
                margin-bottom: 10px;
            }
            .feature-card p {
                color: #a0a0a0;
                font-size: 14px;
                line-height: 1.6;
            }
            .status {
                margin-top: 15px;
                padding: 10px;
                background: rgba(0,0,0,0.3);
                border-radius: 8px;
                font-family: monospace;
                font-size: 12px;
            }
            .status-item {
                display: flex;
                justify-content: space-between;
                padding: 5px 0;
                border-bottom: 1px solid rgba(255,255,255,0.1);
            }
            .status-item:last-child { border-bottom: none; }
            .status-label { color: #888; }
            .status-value { color: #00d4ff; }
            .status-value.success { color: #4caf50; }
            .status-value.error { color: #f44336; }
            button {
                background: linear-gradient(90deg, #00d4ff, #7b2cbf);
                border: none;
                color: white;
                padding: 10px 20px;
                border-radius: 8px;
                cursor: pointer;
                font-size: 14px;
                margin-top: 10px;
                margin-right: 8px;
            }
            button:hover { opacity: 0.9; }
            button.secondary {
                background: rgba(255,255,255,0.1);
                border: 1px solid rgba(255,255,255,0.2);
            }
            #event-log {
                max-height: 200px;
                overflow-y: auto;
                font-size: 11px;
            }
            .log-entry {
                padding: 5px;
                border-bottom: 1px solid rgba(255,255,255,0.05);
            }
            .log-time { color: #666; }
            .log-event { color: #00d4ff; }
            .download-list {
                margin-top: 10px;
            }
            .download-item {
                display: flex;
                align-items: center;
                padding: 8px;
                background: rgba(0,0,0,0.2);
                border-radius: 6px;
                margin-bottom: 8px;
            }
            .download-item .icon {
                font-size: 20px;
                margin-right: 10px;
            }
            .download-item .info {
                flex: 1;
            }
            .download-item .name {
                color: #fff;
                font-size: 13px;
            }
            .download-item .size {
                color: #888;
                font-size: 11px;
            }
            .download-item button {
                margin: 0;
                padding: 6px 12px;
                font-size: 12px;
            }
            .download-status {
                margin-top: 10px;
                padding: 8px;
                border-radius: 6px;
                font-size: 12px;
            }
            .download-status.downloading {
                background: rgba(0, 212, 255, 0.2);
                border: 1px solid rgba(0, 212, 255, 0.3);
            }
            .download-status.completed {
                background: rgba(76, 175, 80, 0.2);
                border: 1px solid rgba(76, 175, 80, 0.3);
            }
            .download-status.error {
                background: rgba(244, 67, 54, 0.2);
                border: 1px solid rgba(244, 67, 54, 0.3);
            }
        </style>
    </head>
    <body>
        <h1>AuroraView - Wry Features Demo</h1>

        <div class="feature-grid">
            <div class="feature-card">
                <h3>Page Load Events</h3>
                <p>Track when pages start and finish loading. Events are emitted
                   to JavaScript for real-time UI updates.</p>
                <div class="status">
                    <div class="status-item">
                        <span class="status-label">Load Status:</span>
                        <span class="status-value" id="load-status">Ready</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Last URL:</span>
                        <span class="status-value" id="last-url">-</span>
                    </div>
                </div>
            </div>

            <div class="feature-card">
                <h3>Document Title</h3>
                <p>The window title updates automatically when the page title changes.
                   Try clicking the button to change the title.</p>
                <button onclick="changeTitle()">Change Title</button>
                <div class="status">
                    <div class="status-item">
                        <span class="status-label">Current Title:</span>
                        <span class="status-value" id="current-title">AuroraView - Wry Features Demo</span>
                    </div>
                </div>
            </div>

            <div class="feature-card">
                <h3>File Downloads</h3>
                <p>Click to download real files. Downloads are saved to your system's
                   default Downloads folder.</p>
                <div class="download-list">
                    <div class="download-item">
                        <span class="icon">📄</span>
                        <div class="info">
                            <div class="name">sample.txt</div>
                            <div class="size">Text file (~1 KB)</div>
                        </div>
                        <button onclick="downloadText()">Download</button>
                    </div>
                    <div class="download-item">
                        <span class="icon">🖼️</span>
                        <div class="info">
                            <div class="name">sample.png</div>
                            <div class="size">PNG image (~5 KB)</div>
                        </div>
                        <button onclick="downloadImage()">Download</button>
                    </div>
                    <div class="download-item">
                        <span class="icon">📦</span>
                        <div class="info">
                            <div class="name">httpbin.json</div>
                            <div class="size">JSON from httpbin.org</div>
                        </div>
                        <button onclick="downloadJson()">Download</button>
                    </div>
                </div>
                <div id="download-status"></div>
            </div>

            <div class="feature-card">
                <h3>Event Log</h3>
                <p>Real-time log of events received from the WebView backend.</p>
                <div class="status" id="event-log">
                    <div class="log-entry">
                        <span class="log-time">[startup]</span>
                        <span class="log-event">Waiting for events...</span>
                    </div>
                </div>
            </div>

            <div class="feature-card">
                <h3>User Agent</h3>
                <p>Custom User-Agent string can be set to identify your application
                   or emulate different browsers.</p>
                <button onclick="showUserAgent()">Show User-Agent</button>
                <div class="status">
                    <div class="status-item">
                        <span class="status-label" id="ua-display">Click button to show</span>
                    </div>
                </div>
            </div>

            <div class="feature-card">
                <h3>Splash Overlay</h3>
                <p>When enabled, shows an animated loading overlay while the page loads.
                   Useful for slow network connections or branded loading experience.</p>
                <div class="status">
                    <div class="status-item">
                        <span class="status-label">Status:</span>
                        <span class="status-value success">Enabled</span>
                    </div>
                </div>
            </div>
        </div>

        <script>
            // Event logging
            function logEvent(event, data) {
                const log = document.getElementById('event-log');
                const time = new Date().toLocaleTimeString();
                const entry = document.createElement('div');
                entry.className = 'log-entry';
                entry.innerHTML = `<span class="log-time">[${time}]</span> <span class="log-event">${event}</span>: ${JSON.stringify(data)}`;
                log.insertBefore(entry, log.firstChild);

                // Keep only last 20 entries
                while (log.children.length > 20) {
                    log.removeChild(log.lastChild);
                }
            }

            // Download status display
            function showDownloadStatus(message, type) {
                const status = document.getElementById('download-status');
                status.className = 'download-status ' + type;
                status.textContent = message;
                if (type === 'completed') {
                    setTimeout(() => { status.className = ''; status.textContent = ''; }, 5000);
                }
            }

            // Listen for AuroraView events
            window.addEventListener('auroraviewready', () => {
                logEvent('auroraviewready', { status: 'Bridge initialized' });

                // Page load events
                window.auroraview.on('page_load_started', (data) => {
                    document.getElementById('load-status').textContent = 'Loading...';
                    document.getElementById('last-url').textContent = data.url || '-';
                    logEvent('page_load_started', data);
                });

                window.auroraview.on('page_load_finished', (data) => {
                    document.getElementById('load-status').textContent = 'Loaded';
                    logEvent('page_load_finished', data);
                });

                // Title change events
                window.auroraview.on('title_changed', (data) => {
                    document.getElementById('current-title').textContent = data.title || '-';
                    logEvent('title_changed', data);
                });

                // Download events
                window.auroraview.on('download_started', (data) => {
                    showDownloadStatus('Downloading: ' + (data.path || data.url), 'downloading');
                    logEvent('download_started', data);
                });

                window.auroraview.on('download_completed', (data) => {
                    if (data.success) {
                        showDownloadStatus('Downloaded: ' + (data.path || 'file'), 'completed');
                    } else {
                        showDownloadStatus('Download failed', 'error');
                    }
                    logEvent('download_completed', data);
                });
            });

            // Title change demo
            let titleIndex = 0;
            const titles = [
                'AuroraView - Wry Features Demo',
                'Title Changed!',
                'Dynamic Titles Work!',
                'Back to Original'
            ];

            function changeTitle() {
                titleIndex = (titleIndex + 1) % titles.length;
                document.title = titles[titleIndex];
            }

            // User-Agent display
            function showUserAgent() {
                document.getElementById('ua-display').textContent = navigator.userAgent;
            }

            // Download functions - using real downloadable resources
            function downloadText() {
                // Create a data URL for text file
                const text = `AuroraView Sample Text File
=============================

This file was downloaded using AuroraView's download feature.

Features demonstrated:
- File downloads enabled by default
- Downloads saved to system Downloads folder
- Download events (start/complete) tracked

Timestamp: ${new Date().toISOString()}

Thank you for trying AuroraView!
`;
                const blob = new Blob([text], { type: 'text/plain' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'auroraview_sample.txt';
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            }

            function downloadImage() {
                // Download a real image from the web
                const a = document.createElement('a');
                a.href = 'https://httpbin.org/image/png';
                a.download = 'sample_image.png';
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
            }

            function downloadJson() {
                // Download JSON from httpbin
                const a = document.createElement('a');
                a.href = 'https://httpbin.org/json';
                a.download = 'httpbin_response.json';
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
            }
        </script>
    </body>
    </html>
    """

    print("Starting AuroraView Wry Features Demo...")
    print("Features demonstrated:")
    print("  - Splash overlay (shows while loading)")
    print("  - Page load events (start/finish)")
    print("  - Document title change tracking")
    print("  - File downloads (click buttons to test)")
    print("  - Custom User-Agent")
    print()
    print("Downloads will prompt 'Save As' dialog (like a browser).")
    print()

    run_app(
        title="AuroraView - Wry Features Demo",
        width=1024,
        height=768,
        html=html,
        dev_tools=True,
        splash_overlay=True,  # Show splash while loading
        user_agent="AuroraView/1.0 (Wry Features Demo)",  # Custom User-Agent
        # Downloads enabled by default with "Save As" dialog
        download_prompt=True,  # Show "Save As" dialog like a browser
        # To save directly without prompt:
        # download_prompt=False,
        # download_directory="./downloads",
        # Proxy example (uncomment to use):
        # proxy_url="http://127.0.0.1:8080",
    )


if __name__ == "__main__":
    main()
