#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""End-to-end testing script for Gallery frontend using Playwright.

This script tests the Gallery frontend by:
1. Starting a local HTTP server to serve Gallery files
2. Loading the Gallery in Playwright's Chromium
3. Injecting the AuroraView bridge
4. Mocking the Python backend API responses
5. Running UI tests

Usage:
    python scripts/test_gallery_e2e.py
    
Requirements:
    pip install playwright
    playwright install chromium
"""

from __future__ import annotations

import http.server
import json
import socketserver
import sys
import threading
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent
GALLERY_DIST = PROJECT_ROOT / "gallery" / "dist"
SERVER_PORT = 8765


# Mock data for Gallery API
MOCK_CATEGORIES = {
    "getting_started": {
        "title": "Getting Started",
        "icon": "rocket",
        "description": "Quick start examples and basic usage patterns",
    },
    "api_patterns": {
        "title": "API Patterns",
        "icon": "code",
        "description": "Different ways to use the AuroraView API",
    },
}

MOCK_SAMPLES = [
    {
        "id": "simple_decorator",
        "title": "Simple Decorator",
        "category": "getting_started",
        "icon": "wand-2",
        "description": "Basic example using decorators",
        "source_file": "examples/simple_decorator.py",
        "tags": ["basic", "decorator"],
    },
    {
        "id": "dynamic_binding",
        "title": "Dynamic Binding",
        "category": "api_patterns",
        "icon": "link",
        "description": "Dynamic API binding example",
        "source_file": "examples/dynamic_binding.py",
        "tags": ["api", "binding"],
    },
]

MOCK_SOURCE = '''"""Simple decorator example.

This example demonstrates the basic usage of AuroraView decorators.
"""

from auroraview import WebView, run_desktop

def main():
    run_desktop(
        html="<h1>Hello World</h1>",
        title="Simple Example",
    )

if __name__ == "__main__":
    main()
'''


def get_auroraview_bridge_script() -> str:
    """Get the AuroraView bridge script with mock API handlers."""
    return f"""
    (function() {{
        if (window.auroraview) return;
        
        const eventHandlers = {{}};
        let callId = 0;
        
        // Mock API responses
        const mockResponses = {{
            'api.get_categories': {json.dumps(MOCK_CATEGORIES)},
            'api.get_samples': {json.dumps(MOCK_SAMPLES)},
            'api.get_source': {json.dumps({"source": MOCK_SOURCE, "sample_id": "simple_decorator"})},
            'api.run_sample': {json.dumps({"pid": 12345, "sample_id": "simple_decorator"})},
            'api.kill_process': {json.dumps({"success": True})},
        }};
        
        window.auroraview = {{
            call: function(method, params) {{
                return new Promise((resolve, reject) => {{
                    console.log('[AuroraView Mock] call:', method, params);
                    
                    // Return mock response if available
                    if (mockResponses[method]) {{
                        setTimeout(() => resolve(mockResponses[method]), 50);
                    }} else {{
                        setTimeout(() => resolve(undefined), 50);
                    }}
                }});
            }},
            
            on: function(event, handler) {{
                if (!eventHandlers[event]) {{
                    eventHandlers[event] = [];
                }}
                eventHandlers[event].push(handler);
                return () => {{
                    const idx = eventHandlers[event].indexOf(handler);
                    if (idx >= 0) eventHandlers[event].splice(idx, 1);
                }};
            }},
            
            off: function(event, handler) {{
                if (eventHandlers[event]) {{
                    const idx = eventHandlers[event].indexOf(handler);
                    if (idx >= 0) eventHandlers[event].splice(idx, 1);
                }}
            }},
            
            trigger: function(event, data) {{
                console.log('[AuroraView Mock] trigger:', event, data);
                if (eventHandlers[event]) {{
                    eventHandlers[event].forEach(h => h(data));
                }}
            }},
            
            api: new Proxy({{}}, {{
                get: function(target, prop) {{
                    return function(...args) {{
                        return window.auroraview.call('api.' + prop, args);
                    }};
                }}
            }}),
            
            platform: 'test',
            version: '1.0.0-test'
        }};
        
        // Dispatch ready event
        window.dispatchEvent(new CustomEvent('auroraviewready'));
        console.log('[AuroraView Mock] Bridge initialized');
    }})();
    """


def start_http_server(port: int = SERVER_PORT) -> socketserver.TCPServer:
    """Start a simple HTTP server to serve Gallery files."""
    import os
    os.chdir(str(GALLERY_DIST))
    
    handler = http.server.SimpleHTTPRequestHandler
    handler.extensions_map.update({
        '.js': 'application/javascript',
        '.mjs': 'application/javascript',
        '.css': 'text/css',
    })
    
    # Allow address reuse
    socketserver.TCPServer.allow_reuse_address = True
    server = socketserver.TCPServer(("", port), handler)
    
    # Start server in background thread
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    
    return server


def run_tests():
    """Run Gallery E2E tests using Playwright."""
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("[ERROR] Playwright not installed. Run: pip install playwright && playwright install chromium")
        return 1
    
    # Check if Gallery is built
    index_html = GALLERY_DIST / "index.html"
    if not index_html.exists():
        print(f"[ERROR] Gallery not built. Run: cd gallery && npm run build")
        print(f"[ERROR] Expected: {index_html}")
        return 1
    
    results = {
        "passed": 0,
        "failed": 0,
        "tests": []
    }
    
    def test(name: str, condition: bool, details: str = ""):
        """Record a test result."""
        if condition:
            results["passed"] += 1
            print(f"  ✓ {name}")
        else:
            results["failed"] += 1
            print(f"  ✗ {name}: {details}")
        results["tests"].append({"name": name, "status": "PASS" if condition else "FAIL", "details": details})
    
    print("\n[TEST] Running Gallery E2E Tests with Playwright...\n")
    
    # Start HTTP server
    print(f"[INFO] Starting HTTP server on port {SERVER_PORT}...")
    server = start_http_server(SERVER_PORT)
    
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(viewport={"width": 1280, "height": 800})
            page = context.new_page()
            
            # Inject AuroraView bridge before page loads
            page.add_init_script(get_auroraview_bridge_script())
            
            # Navigate to Gallery
            gallery_url = f"http://localhost:{SERVER_PORT}/index.html"
            print(f"[INFO] Loading Gallery from: {gallery_url}")
            
            try:
                page.goto(gallery_url, wait_until="networkidle", timeout=30000)
                test("Page loads successfully", True)
            except Exception as e:
                test("Page loads successfully", False, str(e))
                browser.close()
                return 1
            
            # Test 1: AuroraView bridge is available
            try:
                has_bridge = page.evaluate("typeof window.auroraview === 'object'")
                test("AuroraView bridge available", has_bridge)
            except Exception as e:
                test("AuroraView bridge available", False, str(e))
            
            # Test 2: API proxy is available
            try:
                has_api = page.evaluate("typeof window.auroraview?.api === 'object'")
                test("API proxy available", has_api)
            except Exception as e:
                test("API proxy available", False, str(e))
            
            # Test 3: React root element exists
            try:
                has_root = page.evaluate("document.getElementById('root') !== null")
                test("React root element exists", has_root)
            except Exception as e:
                test("React root element exists", False, str(e))
            
            # Wait for React to render and data to load
            page.wait_for_timeout(3000)
            
            # Check for console errors
            console_messages = []
            page.on("console", lambda msg: console_messages.append(f"{msg.type}: {msg.text}"))
            page.on("pageerror", lambda err: console_messages.append(f"ERROR: {err}"))
            
            # Reload to capture console messages
            page.reload(wait_until="networkidle")
            page.wait_for_timeout(3000)
            
            # Debug: Print console messages
            if console_messages:
                print(f"[DEBUG] Console messages:")
                for msg in console_messages[:10]:
                    print(f"  {msg[:100]}")
            
            # Debug: Print page content
            try:
                html_preview = page.evaluate("document.body.innerHTML.substring(0, 500)")
                print(f"[DEBUG] Body HTML preview: {html_preview[:200]}...")
            except Exception:
                pass
            
            # Check if isReady is true
            try:
                is_ready = page.evaluate("window.auroraview !== undefined")
                print(f"[DEBUG] AuroraView available: {is_ready}")
            except Exception as e:
                print(f"[DEBUG] Failed to check AuroraView: {e}")
            
            # Test 4: Check page title
            try:
                title = page.title()
                test("Page has title", len(title) > 0, f"Title: {title}")
            except Exception as e:
                test("Page has title", False, str(e))
            
            # Test 5: Check for main content
            try:
                body_text = page.evaluate("document.body.innerText.substring(0, 200)")
                has_content = len(body_text) > 10
                test("Page has content", has_content, f"Content preview: {body_text[:50]}...")
            except Exception as e:
                test("Page has content", False, str(e))
            
            # Test 6: Event system works
            try:
                result = page.evaluate("""
                    (() => {
                        let received = false;
                        const unsub = window.auroraview.on('test_event', (data) => {
                            received = data.value === 42;
                        });
                        window.auroraview.trigger('test_event', { value: 42 });
                        unsub();
                        return received;
                    })()
                """)
                test("Event system works", result is True)
            except Exception as e:
                test("Event system works", False, str(e))
            
            # Test 7: API call returns Promise
            try:
                result = page.evaluate("""
                    (() => {
                        const promise = window.auroraview.api.get_samples();
                        return promise instanceof Promise;
                    })()
                """)
                test("API call returns Promise", result is True)
            except Exception as e:
                test("API call returns Promise", False, str(e))
            
            # Test 8: API call resolves with data
            try:
                result = page.evaluate("""
                    async () => {
                        const samples = await window.auroraview.api.get_samples();
                        return Array.isArray(samples) && samples.length > 0;
                    }
                """)
                test("API call resolves with data", result is True)
            except Exception as e:
                test("API call resolves with data", False, str(e))
            
            # Test 9: Check for sidebar or navigation
            try:
                has_nav = page.evaluate("""
                    (() => {
                        const nav = document.querySelector('nav, aside, [class*="sidebar"], [class*="Sidebar"]');
                        return nav !== null;
                    })()
                """)
                test("Navigation/sidebar exists", has_nav)
            except Exception as e:
                test("Navigation/sidebar exists", False, str(e))
            
            # Test 10: Check for buttons
            try:
                button_count = page.evaluate("document.querySelectorAll('button').length")
                test("Buttons exist", button_count > 0, f"Found {button_count} buttons")
            except Exception as e:
                test("Buttons exist", False, str(e))
            
            # Take a screenshot for debugging
            screenshot_path = PROJECT_ROOT / "test-screenshots" / "gallery_e2e.png"
            screenshot_path.parent.mkdir(exist_ok=True)
            page.screenshot(path=str(screenshot_path))
            print(f"\n[INFO] Screenshot saved to: {screenshot_path}")
            
            browser.close()
    
    finally:
        # Shutdown server
        server.shutdown()
    
    # Print summary
    print(f"\n{'='*50}")
    print(f"Test Results: {results['passed']} passed, {results['failed']} failed")
    print(f"{'='*50}")
    
    return 0 if results["failed"] == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())
