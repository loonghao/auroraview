"""Automation Demo - AuroraView Browser Automation Abstraction.

This example demonstrates the Automation abstraction layer that provides
a unified interface for browser automation, compatible with both local
AuroraView WebViews and remote Steel Browser instances.

Usage:
    python examples/automation_demo.py

Features demonstrated:
    - Automation class for unified browser control
    - LocalWebViewBackend for embedded WebView automation
    - DOM manipulation via automation API
    - Page scraping capabilities
    - Backend protocol abstraction

Note: Steel Browser backend is a placeholder for future integration.
This demo focuses on the local WebView automation capabilities.
"""

from __future__ import annotations

from auroraview import WebView
from auroraview.utils.automation import Automation


def main():
    """Run the automation demo."""
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Automation Demo</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 900px;
                margin: 50px auto;
                padding: 20px;
                background: linear-gradient(135deg, #fc466b 0%, #3f5efb 100%);
                min-height: 100vh;
            }
            .card {
                background: white;
                border-radius: 12px;
                padding: 24px;
                box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                margin-bottom: 20px;
            }
            h1 { color: #333; margin-top: 0; }
            h3 { color: #666; margin-bottom: 10px; }
            button {
                background: linear-gradient(135deg, #fc466b 0%, #3f5efb 100%);
                color: white;
                border: none;
                padding: 12px 24px;
                border-radius: 6px;
                cursor: pointer;
                font-size: 14px;
                margin: 5px;
                transition: transform 0.1s;
            }
            button:hover { transform: translateY(-2px); }
            button:active { transform: translateY(0); }
            .automation-target {
                background: #f5f5f5;
                border-radius: 8px;
                padding: 20px;
                margin: 15px 0;
            }
            .target-element {
                padding: 15px;
                margin: 10px 0;
                background: white;
                border-radius: 6px;
                border: 2px solid #e0e0e0;
                transition: all 0.3s;
            }
            .target-element.clicked { border-color: #4caf50; background: #e8f5e9; }
            .target-element.typed { border-color: #2196f3; background: #e3f2fd; }
            .target-element.scraped { border-color: #ff9800; background: #fff3e0; }
            .form-row {
                display: flex;
                gap: 15px;
                margin: 15px 0;
            }
            .form-field {
                flex: 1;
            }
            .form-field label {
                display: block;
                margin-bottom: 5px;
                font-weight: 500;
            }
            .form-field input {
                width: 100%;
                padding: 10px;
                border: 2px solid #e0e0e0;
                border-radius: 6px;
                font-size: 14px;
                box-sizing: border-box;
            }
            .form-field input:focus {
                outline: none;
                border-color: #3f5efb;
            }
            #scrapeResult {
                background: #1e1e1e;
                color: #0f0;
                border-radius: 8px;
                padding: 16px;
                font-family: 'Consolas', monospace;
                font-size: 12px;
                max-height: 200px;
                overflow-y: auto;
                white-space: pre-wrap;
            }
            .action-log {
                background: #f5f5f5;
                border-radius: 8px;
                padding: 15px;
                margin-top: 15px;
            }
            .log-entry {
                padding: 8px;
                margin: 5px 0;
                background: white;
                border-radius: 4px;
                font-size: 13px;
                border-left: 3px solid #3f5efb;
            }
            .log-entry.success { border-left-color: #4caf50; }
            .log-entry.error { border-left-color: #f44336; }
            .backend-info {
                display: flex;
                gap: 20px;
                padding: 15px;
                background: #f5f5f5;
                border-radius: 8px;
                margin: 15px 0;
            }
            .backend-card {
                flex: 1;
                padding: 15px;
                background: white;
                border-radius: 8px;
                text-align: center;
            }
            .backend-card.active { border: 2px solid #4caf50; }
            .backend-name { font-weight: bold; color: #333; }
            .backend-status { font-size: 12px; color: #666; margin-top: 5px; }
        </style>
    </head>
    <body>
        <div class="card">
            <h1>Automation Demo</h1>
            <p>Unified browser automation interface for local WebView and remote Steel Browser.</p>

            <div class="backend-info">
                <div class="backend-card active">
                    <div class="backend-name">LocalWebViewBackend</div>
                    <div class="backend-status">Active - Using embedded WebView</div>
                </div>
                <div class="backend-card">
                    <div class="backend-name">SteelBrowserBackend</div>
                    <div class="backend-status">Available - Remote automation</div>
                </div>
            </div>
        </div>

        <div class="card">
            <h3>DOM Automation Targets</h3>
            <div class="automation-target">
                <div class="target-element" id="clickTarget">
                    Click Target - Click me via automation!
                </div>

                <div class="form-row">
                    <div class="form-field">
                        <label for="autoInput1">Input Field 1</label>
                        <input type="text" id="autoInput1" placeholder="Type via automation">
                    </div>
                    <div class="form-field">
                        <label for="autoInput2">Input Field 2</label>
                        <input type="text" id="autoInput2" placeholder="Type via automation">
                    </div>
                </div>

                <div class="target-element" id="scrapeTarget">
                    <strong>Scrape Target</strong>
                    <p>This content can be scraped by the automation layer.</p>
                    <ul>
                        <li>Item 1: Data point A</li>
                        <li>Item 2: Data point B</li>
                        <li>Item 3: Data point C</li>
                    </ul>
                </div>
            </div>

            <div>
                <button onclick="requestAutoClick()">Auto Click</button>
                <button onclick="requestAutoType()">Auto Type</button>
                <button onclick="requestAutoScrape()">Auto Scrape</button>
                <button onclick="requestAutoFill()">Auto Fill Form</button>
                <button onclick="requestReset()">Reset</button>
            </div>
        </div>

        <div class="card">
            <h3>Scrape Result</h3>
            <div id="scrapeResult">Scrape results will appear here...</div>
        </div>

        <div class="card">
            <h3>Action Log</h3>
            <div class="action-log" id="actionLog">
                <div class="log-entry">Ready for automation actions...</div>
            </div>
        </div>

        <script>
            function log(msg, type = 'info') {
                const logEl = document.getElementById('actionLog');
                const entry = document.createElement('div');
                entry.className = `log-entry ${type}`;
                entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
                logEl.insertBefore(entry, logEl.firstChild);

                // Keep only last 10 entries
                while (logEl.children.length > 10) {
                    logEl.removeChild(logEl.lastChild);
                }
            }

            async function requestAutoClick() {
                try {
                    const result = await auroraview.api.auto_click({selector: '#clickTarget'});
                    log(`Click: ${result.message}`, 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function requestAutoType() {
                try {
                    const result = await auroraview.api.auto_type({
                        selector: '#autoInput1',
                        text: 'Hello from automation!'
                    });
                    log(`Type: ${result.message}`, 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function requestAutoScrape() {
                try {
                    const result = await auroraview.api.auto_scrape({selector: '#scrapeTarget'});
                    document.getElementById('scrapeResult').textContent =
                        JSON.stringify(result, null, 2);
                    log('Scrape: Content extracted', 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function requestAutoFill() {
                try {
                    const result = await auroraview.api.auto_fill_form();
                    log(`Fill: ${result.message}`, 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function requestReset() {
                try {
                    await auroraview.api.reset_automation();
                    document.getElementById('scrapeResult').textContent =
                        'Scrape results will appear here...';
                    log('Reset: All targets reset', 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }
        </script>
    </body>
    </html>
    """

    view = WebView(title="Automation Demo", html=html_content, width=950, height=900)

    # Create automation instance with local backend
    auto = Automation.local(view)

    @view.bind_call("api.auto_click")
    def auto_click(selector: str) -> dict:
        """Click an element via automation."""
        element = auto.dom(selector)
        element.click()
        element.add_class("clicked")
        return {"ok": True, "message": f"Clicked element: {selector}"}

    @view.bind_call("api.auto_type")
    def auto_type(selector: str, text: str) -> dict:
        """Type text into an element via automation."""
        element = auto.dom(selector)
        element.type_text(text, clear_first=True)
        element.add_class("typed")
        return {"ok": True, "message": f"Typed '{text}' into {selector}"}

    @view.bind_call("api.auto_scrape")
    def auto_scrape(selector: str) -> dict:
        """Scrape content from an element."""
        element = auto.dom(selector)
        element.add_class("scraped")

        # Use the scrape method from automation
        # Note: Full scraping requires JS evaluation
        scrape_result = auto.scrape()

        return {
            "ok": True,
            "selector": selector,
            "scrape_status": scrape_result.get("status", "unknown"),
            "message": "Content marked for scraping. Full scraping requires JS bridge.",
        }

    @view.bind_call("api.auto_fill_form")
    def auto_fill_form() -> dict:
        """Fill multiple form fields via automation."""
        auto.dom("#autoInput1").type_text("Automated Input 1", clear_first=True)
        auto.dom("#autoInput2").type_text("Automated Input 2", clear_first=True)

        # Add visual feedback
        auto.dom("#autoInput1").add_class("typed")
        auto.dom("#autoInput2").add_class("typed")

        return {"ok": True, "message": "Form fields filled via automation"}

    @view.bind_call("api.reset_automation")
    def reset_automation() -> dict:
        """Reset all automation targets."""
        for selector in ["#clickTarget", "#autoInput1", "#autoInput2", "#scrapeTarget"]:
            el = auto.dom(selector)
            el.remove_class("clicked", "typed", "scraped")

        auto.dom("#autoInput1").clear()
        auto.dom("#autoInput2").clear()

        return {"ok": True}

    @view.bind_call("api.get_backend_info")
    def get_backend_info() -> dict:
        """Get information about the current automation backend."""
        return {
            "backend_type": "LocalWebViewBackend",
            "description": "Using embedded AuroraView WebView for automation",
            "capabilities": ["dom", "dom_all", "scrape"],
            "limitations": ["screenshot", "pdf"],
        }

    print("=" * 60)
    print("Automation Demo - Unified Browser Automation")
    print("=" * 60)
    print()
    print("The Automation class provides a unified interface for:")
    print("  - Local WebView automation (current demo)")
    print("  - Remote Steel Browser automation (future)")
    print()
    print("Usage patterns:")
    print()
    print("  # Local automation")
    print("  auto = Automation.local(webview)")
    print("  auto.dom('#button').click()")
    print("  auto.dom('#input').type_text('Hello')")
    print()
    print("  # Remote automation (Steel Browser)")
    print("  auto = Automation.steel('http://steel.example.com:3000')")
    print("  result = auto.scrape('https://example.com')")
    print()
    print("=" * 60)

    view.show()


if __name__ == "__main__":
    main()
