#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""End-to-end testing script for Gallery frontend using Playwright.

This script tests the Gallery frontend by:
1. Starting a local HTTP server to serve Gallery files
2. Loading the Gallery in Playwright's Chromium
3. Injecting the AuroraView bridge (mock mode) or using real backend (live mode)
4. Running UI tests
5. Taking screenshots of different pages

Usage:
    python scripts/test_gallery_e2e.py                    # Mock mode (default)
    python scripts/test_gallery_e2e.py --live             # Live mode with real Python backend
    python scripts/test_gallery_e2e.py --screenshots-only # Only take screenshots
    python scripts/test_gallery_e2e.py --update-assets    # Update assets/images/

Requirements:
    pip install playwright
    playwright install chromium
"""

from __future__ import annotations

import argparse
import http.server
import json
import os
import socketserver
import subprocess
import sys
import threading
import time
from pathlib import Path
from typing import Optional

PROJECT_ROOT = Path(__file__).parent.parent
GALLERY_DIST = PROJECT_ROOT / "gallery" / "dist"
GALLERY_MAIN = PROJECT_ROOT / "gallery" / "main.py"
SCREENSHOTS_DIR = PROJECT_ROOT / "test-screenshots"
ASSETS_DIR = PROJECT_ROOT / "assets" / "images"
SERVER_PORT = 8765
PYTHON_API_PORT = 8766

# Screenshot configurations: (name, action_description, click_selector, wait_ms)
SCREENSHOT_CONFIGS = [
    ("gallery_home", "Home page", None, 2000),
    (
        "gallery_getting_started",
        "Getting Started category",
        'button[title*="Getting Started"]',
        1000,
    ),
    ("gallery_api_patterns", "API Patterns category", 'button[title*="API Patterns"]', 1000),
    (
        "gallery_window_features",
        "Window Features category",
        'button[title*="Window Features"]',
        1000,
    ),
    ("gallery_settings", "Settings dialog", 'button[title="Settings"]', 1500),
]


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


def get_live_bridge_script(api_port: int = PYTHON_API_PORT) -> str:
    """Get the AuroraView bridge script that connects to real Python backend.

    This creates a bridge that forwards API calls to the Python backend via HTTP.
    """
    return f"""
    (function() {{
        if (window.auroraview) return;
        
        const eventHandlers = {{}};
        let callId = 0;
        const API_BASE = 'http://localhost:{api_port}';
        
        window.auroraview = {{
            call: async function(method, params) {{
                const id = `av_call_${{Date.now()}}_${{++callId}}`;
                console.log('[AuroraView Live] call:', method, params);
                
                try {{
                    const response = await fetch(`${{API_BASE}}/api`, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            id: id,
                            method: method.replace('api.', ''),
                            params: params
                        }})
                    }});
                    
                    const result = await response.json();
                    console.log('[AuroraView Live] response:', result);
                    
                    if (result.ok) {{
                        return result.result;
                    }} else {{
                        throw new Error(result.error?.message || 'API call failed');
                    }}
                }} catch (e) {{
                    console.error('[AuroraView Live] error:', e);
                    throw e;
                }}
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
                console.log('[AuroraView Live] trigger:', event, data);
                if (eventHandlers[event]) {{
                    eventHandlers[event].forEach(h => h(data));
                }}
            }},
            
            api: new Proxy({{}}, {{
                get: function(target, prop) {{
                    return function(...args) {{
                        return window.auroraview.call('api.' + prop, args.length === 1 ? args[0] : args);
                    }};
                }}
            }}),
            
            platform: 'test-live',
            version: '1.0.0-test'
        }};
        
        // Dispatch ready event
        window.dispatchEvent(new CustomEvent('auroraviewready'));
        console.log('[AuroraView Live] Bridge initialized, API at {api_port}');
    }})();
    """


class PythonAPIServer:
    """HTTP server that wraps the Gallery Python API for testing."""

    def __init__(self, port: int = PYTHON_API_PORT):
        self.port = port
        self.process: Optional[subprocess.Popen] = None
        self.server_thread: Optional[threading.Thread] = None

    def start(self) -> bool:
        """Start the Python API server."""
        # Create a simple HTTP wrapper around the Gallery API
        server_script = PROJECT_ROOT / "scripts" / "_gallery_api_server.py"

        # Write the server script
        server_code = '''
#!/usr/bin/env python
"""Temporary HTTP server for Gallery API testing."""
import json
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path

# Add project to path
sys.path.insert(0, str(Path(__file__).parent.parent))
sys.path.insert(0, str(Path(__file__).parent.parent / "python"))

from gallery.main import get_samples, get_categories, get_source

class APIHandler(BaseHTTPRequestHandler):
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()
    
    def do_POST(self):
        if self.path != "/api":
            self.send_error(404)
            return
        
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)
        
        try:
            request = json.loads(body)
            method = request.get("method", "")
            params = request.get("params")
            call_id = request.get("id", "")
            
            # Route to appropriate handler
            handlers = {
                "get_samples": get_samples,
                "get_categories": get_categories,
                "get_source": lambda p: get_source(p.get("sample_id") if isinstance(p, dict) else p[0] if p else None),
            }
            
            handler = handlers.get(method)
            if handler:
                try:
                    if params is None:
                        result = handler()
                    elif isinstance(params, dict):
                        result = handler(params)
                    elif isinstance(params, list) and len(params) == 1:
                        result = handler(params[0])
                    else:
                        result = handler(params)
                    
                    response = {"id": call_id, "ok": True, "result": result}
                except Exception as e:
                    response = {"id": call_id, "ok": False, "error": {"name": type(e).__name__, "message": str(e)}}
            else:
                response = {"id": call_id, "ok": False, "error": {"name": "MethodNotFound", "message": f"Unknown method: {method}"}}
            
        except Exception as e:
            response = {"id": "", "ok": False, "error": {"name": "ParseError", "message": str(e)}}
        
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())
    
    def log_message(self, format, *args):
        pass  # Suppress logging

if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8766
    server = HTTPServer(("", port), APIHandler)
    print(f"API server running on port {port}", flush=True)
    server.serve_forever()
'''
        server_script.write_text(server_code)

        try:
            self.process = subprocess.Popen(
                [sys.executable, str(server_script), str(self.port)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                cwd=str(PROJECT_ROOT),
            )

            # Wait for server to start
            time.sleep(1)

            # Check if process is still running
            if self.process.poll() is not None:
                stderr = self.process.stderr.read().decode() if self.process.stderr else ""
                print(f"[ERROR] API server failed to start: {stderr}")
                return False

            print(f"[INFO] Python API server started on port {self.port}")
            return True

        except Exception as e:
            print(f"[ERROR] Failed to start API server: {e}")
            return False

    def stop(self):
        """Stop the Python API server."""
        if self.process:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
            self.process = None

        # Clean up temp script
        server_script = PROJECT_ROOT / "scripts" / "_gallery_api_server.py"
        if server_script.exists():
            server_script.unlink()


def start_http_server(port: int = SERVER_PORT) -> socketserver.TCPServer:
    """Start a simple HTTP server to serve Gallery files."""
    os.chdir(str(GALLERY_DIST))

    handler = http.server.SimpleHTTPRequestHandler
    handler.extensions_map.update(
        {
            ".js": "application/javascript",
            ".mjs": "application/javascript",
            ".css": "text/css",
        }
    )

    # Allow address reuse
    socketserver.TCPServer.allow_reuse_address = True
    server = socketserver.TCPServer(("", port), handler)

    # Start server in background thread
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()

    return server


def take_screenshots(page, output_dir: Path, update_assets: bool = False) -> list[str]:
    """Take screenshots of different Gallery pages.

    Returns list of screenshot paths.
    """
    output_dir.mkdir(parents=True, exist_ok=True)
    screenshots = []

    for name, description, selector, wait_ms in SCREENSHOT_CONFIGS:
        try:
            # Click the element if selector is provided
            if selector:
                # First go back to home to reset state
                page.goto(f"http://localhost:{SERVER_PORT}/index.html", wait_until="networkidle")
                page.wait_for_timeout(1500)

                # Try to click the element
                element = page.query_selector(selector)
                if element:
                    element.click()
                    page.wait_for_timeout(wait_ms)
                else:
                    print(f"  [WARN] Selector not found: {selector}")
                    continue
            else:
                page.wait_for_timeout(wait_ms)

            # Take screenshot
            screenshot_path = output_dir / f"{name}.png"
            page.screenshot(path=str(screenshot_path))
            screenshots.append(str(screenshot_path))
            print(f"  [OK] {description}: {screenshot_path.name}")

            # Also copy to assets if requested
            if update_assets:
                asset_path = ASSETS_DIR / f"{name}.png"
                asset_path.parent.mkdir(parents=True, exist_ok=True)
                import shutil

                shutil.copy(screenshot_path, asset_path)
                print(f"    -> Updated: {asset_path.relative_to(PROJECT_ROOT)}")

        except Exception as e:
            print(f"  [FAIL] {description}: {e}")

    return screenshots


def run_tests(screenshots_only: bool = False, update_assets: bool = False, live_mode: bool = False):
    """Run Gallery E2E tests using Playwright.

    Args:
        screenshots_only: Only take screenshots, skip tests
        update_assets: Update assets/images/ with new screenshots
        live_mode: Use real Python backend instead of mocks (eat your own dogfood)
    """
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print(
            "[ERROR] Playwright not installed. Run: pip install playwright && playwright install chromium"
        )
        return 1

    # Check if Gallery is built
    index_html = GALLERY_DIST / "index.html"
    if not index_html.exists():
        print("[ERROR] Gallery not built. Run: cd gallery && npm run build")
        print(f"[ERROR] Expected: {index_html}")
        return 1

    results = {"passed": 0, "failed": 0, "tests": [], "screenshots": []}

    def test(name: str, condition: bool, details: str = ""):
        """Record a test result."""
        if condition:
            results["passed"] += 1
            print(f"  ✓ {name}")
        else:
            results["failed"] += 1
            print(f"  ✗ {name}: {details}")
        results["tests"].append(
            {"name": name, "status": "PASS" if condition else "FAIL", "details": details}
        )

    mode_str = "LIVE (real Python backend)" if live_mode else "MOCK"
    print(f"\n[TEST] Running Gallery E2E Tests with Playwright ({mode_str})...\n")

    # Start HTTP server for frontend
    print(f"[INFO] Starting HTTP server on port {SERVER_PORT}...")
    server = start_http_server(SERVER_PORT)

    # Start Python API server if in live mode
    api_server: Optional[PythonAPIServer] = None
    if live_mode:
        print(f"[INFO] Starting Python API server on port {PYTHON_API_PORT}...")
        api_server = PythonAPIServer(PYTHON_API_PORT)
        if not api_server.start():
            print("[ERROR] Failed to start Python API server")
            server.shutdown()
            return 1

    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(viewport={"width": 1280, "height": 800})
            page = context.new_page()

            # Inject appropriate AuroraView bridge
            if live_mode:
                page.add_init_script(get_live_bridge_script(PYTHON_API_PORT))
            else:
                page.add_init_script(get_auroraview_bridge_script())

            # Navigate to Gallery
            gallery_url = f"http://localhost:{SERVER_PORT}/index.html"
            print(f"[INFO] Loading Gallery from: {gallery_url}")

            try:
                page.goto(gallery_url, wait_until="networkidle", timeout=30000)
                if not screenshots_only:
                    test("Page loads successfully", True)
            except Exception as e:
                if not screenshots_only:
                    test("Page loads successfully", False, str(e))
                browser.close()
                return 1

            # Wait for React to render
            page.wait_for_timeout(3000)

            if not screenshots_only:
                # Run all tests
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

                # Live mode specific tests - verify real API integration
                if live_mode:
                    print("\n[INFO] Running live mode API tests...")

                    # Test 11: get_categories returns real data
                    try:
                        categories = page.evaluate("""
                            async () => {
                                const categories = await window.auroraview.api.get_categories();
                                return categories;
                            }
                        """)
                        has_categories = isinstance(categories, dict) and len(categories) > 0
                        test(
                            "[LIVE] get_categories returns data",
                            has_categories,
                            f"Got {len(categories) if categories else 0} categories",
                        )
                    except Exception as e:
                        test("[LIVE] get_categories returns data", False, str(e))

                    # Test 12: get_samples returns real samples
                    try:
                        samples = page.evaluate("""
                            async () => {
                                const samples = await window.auroraview.api.get_samples();
                                return samples;
                            }
                        """)
                        has_samples = isinstance(samples, list) and len(samples) > 0
                        test(
                            "[LIVE] get_samples returns data",
                            has_samples,
                            f"Got {len(samples) if samples else 0} samples",
                        )
                    except Exception as e:
                        test("[LIVE] get_samples returns data", False, str(e))

                    # Test 13: get_source returns source code
                    try:
                        source_result = page.evaluate("""
                            async () => {
                                const samples = await window.auroraview.api.get_samples();
                                if (samples && samples.length > 0) {
                                    const source = await window.auroraview.api.get_source({sample_id: samples[0].id});
                                    return source;
                                }
                                return null;
                            }
                        """)
                        has_source = (
                            source_result
                            and "source" in source_result
                            and len(source_result["source"]) > 0
                        )
                        test(
                            "[LIVE] get_source returns code",
                            has_source,
                            f"Source length: {len(source_result.get('source', '')) if source_result else 0}",
                        )
                    except Exception as e:
                        test("[LIVE] get_source returns code", False, str(e))

                    # Test 14: API error handling
                    try:
                        error_result = page.evaluate("""
                            async () => {
                                try {
                                    await window.auroraview.api.nonexistent_method();
                                    return {caught: false};
                                } catch (e) {
                                    return {caught: true, message: e.message};
                                }
                            }
                        """)
                        test(
                            "[LIVE] API error handling works",
                            error_result.get("caught", False),
                            f"Error: {error_result.get('message', 'N/A')}",
                        )
                    except Exception as e:
                        test("[LIVE] API error handling works", False, str(e))

            # Take screenshots
            print("\n[INFO] Taking screenshots...")
            screenshots = take_screenshots(page, SCREENSHOTS_DIR, update_assets)
            results["screenshots"] = screenshots

            browser.close()

    finally:
        # Shutdown servers
        server.shutdown()
        if api_server:
            api_server.stop()

    # Print summary
    print(f"\n{'=' * 50}")
    print(f"Mode: {mode_str}")
    if not screenshots_only:
        print(f"Test Results: {results['passed']} passed, {results['failed']} failed")
    print(f"Screenshots: {len(results['screenshots'])} captured")
    print(f"{'=' * 50}")

    return 0 if results["failed"] == 0 else 1


def main():
    parser = argparse.ArgumentParser(description="Gallery E2E Tests")
    parser.add_argument(
        "--screenshots-only", action="store_true", help="Only take screenshots, skip tests"
    )
    parser.add_argument(
        "--update-assets", action="store_true", help="Update assets/images/ with new screenshots"
    )
    parser.add_argument(
        "--live",
        action="store_true",
        help="Use real Python backend instead of mocks (eat your own dogfood)",
    )
    args = parser.parse_args()

    return run_tests(
        screenshots_only=args.screenshots_only,
        update_assets=args.update_assets,
        live_mode=args.live,
    )


if __name__ == "__main__":
    sys.exit(main())
