"""Custom Protocol Demo - Register custom URL schemes.

This example demonstrates how to register custom protocol handlers
in AuroraView, allowing you to serve dynamic content through
custom URL schemes like `asset://`, `api://`, or `maya://`.

Features demonstrated:
- Registering custom protocol handlers
- Serving dynamic content (images, JSON, HTML)
- Loading local files through custom schemes
- Error handling for missing resources
- MIME type handling
"""

from __future__ import annotations

import json
import os
from pathlib import Path

# WebView import is done in main() to avoid circular imports

# Sample data for the demo
SAMPLE_DATA = {
    "users": [
        {"id": 1, "name": "Alice", "role": "Developer", "avatar": "user1"},
        {"id": 2, "name": "Bob", "role": "Designer", "avatar": "user2"},
        {"id": 3, "name": "Charlie", "role": "Manager", "avatar": "user3"},
    ],
    "projects": [
        {"id": 1, "name": "Project Alpha", "status": "active", "progress": 75},
        {"id": 2, "name": "Project Beta", "status": "pending", "progress": 30},
        {"id": 3, "name": "Project Gamma", "status": "completed", "progress": 100},
    ],
}

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Custom Protocol Demo</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f172a;
            color: #e2e8f0;
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1000px;
            margin: 0 auto;
        }
        h1 {
            text-align: center;
            margin-bottom: 10px;
            background: linear-gradient(90deg, #60a5fa, #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .subtitle {
            text-align: center;
            color: #64748b;
            margin-bottom: 30px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 20px;
        }
        .card {
            background: #1e293b;
            border-radius: 12px;
            padding: 20px;
            border: 1px solid #334155;
        }
        .card h2 {
            font-size: 16px;
            color: #60a5fa;
            margin-bottom: 15px;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .card h2::before {
            content: '';
            width: 8px;
            height: 8px;
            background: #60a5fa;
            border-radius: 50%;
        }
        .protocol-badge {
            display: inline-block;
            background: #3b82f6;
            color: white;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-family: monospace;
            margin-bottom: 15px;
        }
        .demo-area {
            background: #0f172a;
            border-radius: 8px;
            padding: 15px;
            min-height: 120px;
        }
        .btn-group {
            display: flex;
            gap: 10px;
            margin-top: 15px;
            flex-wrap: wrap;
        }
        button {
            padding: 8px 16px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 13px;
            transition: all 0.2s;
            background: #3b82f6;
            color: white;
        }
        button:hover {
            background: #2563eb;
            transform: translateY(-1px);
        }
        button.secondary {
            background: #475569;
        }
        button.secondary:hover {
            background: #64748b;
        }
        .user-list {
            display: flex;
            flex-direction: column;
            gap: 10px;
        }
        .user-item {
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px;
            background: #1e293b;
            border-radius: 8px;
        }
        .user-avatar {
            width: 40px;
            height: 40px;
            border-radius: 50%;
            background: linear-gradient(135deg, #60a5fa, #a78bfa);
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: bold;
            font-size: 16px;
        }
        .user-info {
            flex: 1;
        }
        .user-name {
            font-weight: 500;
        }
        .user-role {
            font-size: 12px;
            color: #64748b;
        }
        .project-list {
            display: flex;
            flex-direction: column;
            gap: 10px;
        }
        .project-item {
            padding: 12px;
            background: #1e293b;
            border-radius: 8px;
        }
        .project-header {
            display: flex;
            justify-content: space-between;
            margin-bottom: 8px;
        }
        .project-name {
            font-weight: 500;
        }
        .project-status {
            font-size: 11px;
            padding: 2px 8px;
            border-radius: 10px;
        }
        .status-active { background: #22c55e; color: white; }
        .status-pending { background: #f59e0b; color: white; }
        .status-completed { background: #3b82f6; color: white; }
        .progress-bar {
            height: 6px;
            background: #334155;
            border-radius: 3px;
            overflow: hidden;
        }
        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #60a5fa, #a78bfa);
            transition: width 0.3s;
        }
        .code-block {
            background: #0f172a;
            border-radius: 8px;
            padding: 15px;
            font-family: 'Monaco', 'Consolas', monospace;
            font-size: 12px;
            overflow-x: auto;
            white-space: pre;
            color: #94a3b8;
        }
        .code-block .keyword { color: #c084fc; }
        .code-block .string { color: #4ade80; }
        .code-block .comment { color: #64748b; }
        .image-grid {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 10px;
        }
        .image-item {
            aspect-ratio: 1;
            background: linear-gradient(135deg, #1e293b, #334155);
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
            border: 2px dashed #334155;
        }
        .image-item.loaded {
            border-style: solid;
            border-color: #22c55e;
        }
        .log-area {
            height: 150px;
            overflow-y: auto;
            background: #0f172a;
            border-radius: 8px;
            padding: 10px;
            font-family: monospace;
            font-size: 12px;
        }
        .log-entry {
            padding: 4px 0;
            border-bottom: 1px solid #1e293b;
        }
        .log-entry .time { color: #64748b; }
        .log-entry .url { color: #60a5fa; }
        .log-entry .status { color: #22c55e; }
        .log-entry .error { color: #ef4444; }
        .full-width {
            grid-column: 1 / -1;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Custom Protocol Demo</h1>
        <p class="subtitle">Register custom URL schemes to serve dynamic content</p>

        <div class="grid">
            <!-- API Protocol Demo -->
            <div class="card">
                <h2>API Protocol</h2>
                <span class="protocol-badge">api://</span>
                <div class="demo-area" id="users-area">
                    <p style="color: #64748b;">Click "Load Users" to fetch data via api:// protocol</p>
                </div>
                <div class="btn-group">
                    <button onclick="loadUsers()">Load Users</button>
                    <button onclick="loadProjects()" class="secondary">Load Projects</button>
                </div>
            </div>

            <!-- Asset Protocol Demo -->
            <div class="card">
                <h2>Asset Protocol</h2>
                <span class="protocol-badge">asset://</span>
                <div class="demo-area">
                    <div class="image-grid" id="image-grid">
                        <div class="image-item" id="img-1">1</div>
                        <div class="image-item" id="img-2">2</div>
                        <div class="image-item" id="img-3">3</div>
                    </div>
                </div>
                <div class="btn-group">
                    <button onclick="loadImages()">Load Images</button>
                    <button onclick="clearImages()" class="secondary">Clear</button>
                </div>
            </div>

            <!-- Dynamic HTML Protocol -->
            <div class="card">
                <h2>Dynamic HTML Protocol</h2>
                <span class="protocol-badge">dynamic://</span>
                <div class="demo-area">
                    <iframe id="dynamic-frame" style="width:100%;height:100px;border:none;border-radius:4px;background:white;"></iframe>
                </div>
                <div class="btn-group">
                    <button onclick="loadDynamicPage('hello')">Hello Page</button>
                    <button onclick="loadDynamicPage('stats')" class="secondary">Stats Page</button>
                    <button onclick="loadDynamicPage('time')" class="secondary">Time Page</button>
                </div>
            </div>

            <!-- Error Handling Demo -->
            <div class="card">
                <h2>Error Handling</h2>
                <span class="protocol-badge">api://invalid</span>
                <div class="demo-area">
                    <div id="error-demo" style="color: #64748b;">
                        Test error handling for missing resources
                    </div>
                </div>
                <div class="btn-group">
                    <button onclick="testError()">Request Invalid Resource</button>
                </div>
            </div>

            <!-- Request Log -->
            <div class="card full-width">
                <h2>Request Log</h2>
                <div class="log-area" id="log-area">
                    <div class="log-entry">
                        <span class="time">[--:--:--]</span>
                        <span>Waiting for requests...</span>
                    </div>
                </div>
            </div>

            <!-- Code Example -->
            <div class="card full-width">
                <h2>Python Code Example</h2>
                <div class="code-block">
<span class="comment"># Register a custom protocol handler</span>
<span class="keyword">def</span> handle_api(uri: str) -> dict:
    <span class="comment"># Parse the URI: api://users -> path = "users"</span>
    path = uri.replace(<span class="string">"api://"</span>, <span class="string">""</span>)

    <span class="keyword">if</span> path == <span class="string">"users"</span>:
        <span class="keyword">return</span> {
            <span class="string">"data"</span>: json.dumps(users).encode(),
            <span class="string">"mime_type"</span>: <span class="string">"application/json"</span>,
            <span class="string">"status"</span>: 200
        }

    <span class="comment"># Return 404 for unknown paths</span>
    <span class="keyword">return</span> {
        <span class="string">"data"</span>: b<span class="string">"Not Found"</span>,
        <span class="string">"mime_type"</span>: <span class="string">"text/plain"</span>,
        <span class="string">"status"</span>: 404
    }

<span class="comment"># Register the handler</span>
webview.register_protocol(<span class="string">"api"</span>, handle_api)
                </div>
            </div>
        </div>
    </div>

    <script>
        function logRequest(url, status, message) {
            const time = new Date().toLocaleTimeString();
            const logArea = document.getElementById('log-area');
            const statusClass = status >= 400 ? 'error' : 'status';
            logArea.innerHTML = `
                <div class="log-entry">
                    <span class="time">[${time}]</span>
                    <span class="url">${url}</span>
                    <span class="${statusClass}">${status} ${message}</span>
                </div>
            ` + logArea.innerHTML;
        }

        async function loadUsers() {
            try {
                const response = await fetch('api://users');
                const data = await response.json();
                logRequest('api://users', 200, 'OK');

                const html = data.map(user => `
                    <div class="user-item">
                        <div class="user-avatar">${user.name[0]}</div>
                        <div class="user-info">
                            <div class="user-name">${user.name}</div>
                            <div class="user-role">${user.role}</div>
                        </div>
                    </div>
                `).join('');

                document.getElementById('users-area').innerHTML = `<div class="user-list">${html}</div>`;
            } catch (e) {
                logRequest('api://users', 500, e.message);
            }
        }

        async function loadProjects() {
            try {
                const response = await fetch('api://projects');
                const data = await response.json();
                logRequest('api://projects', 200, 'OK');

                const html = data.map(project => `
                    <div class="project-item">
                        <div class="project-header">
                            <span class="project-name">${project.name}</span>
                            <span class="project-status status-${project.status}">${project.status}</span>
                        </div>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: ${project.progress}%"></div>
                        </div>
                    </div>
                `).join('');

                document.getElementById('users-area').innerHTML = `<div class="project-list">${html}</div>`;
            } catch (e) {
                logRequest('api://projects', 500, e.message);
            }
        }

        function loadImages() {
            for (let i = 1; i <= 3; i++) {
                const img = document.getElementById(`img-${i}`);
                img.classList.add('loaded');
                img.innerHTML = '✓';
                logRequest(`asset://image${i}.png`, 200, 'OK');
            }
        }

        function clearImages() {
            for (let i = 1; i <= 3; i++) {
                const img = document.getElementById(`img-${i}`);
                img.classList.remove('loaded');
                img.innerHTML = i;
            }
        }

        function loadDynamicPage(page) {
            const frame = document.getElementById('dynamic-frame');
            frame.src = `dynamic://${page}`;
            logRequest(`dynamic://${page}`, 200, 'OK');
        }

        async function testError() {
            try {
                const response = await fetch('api://invalid/path');
                logRequest('api://invalid/path', response.status, response.statusText);
                document.getElementById('error-demo').innerHTML = `
                    <span style="color: #ef4444;">Error ${response.status}: ${response.statusText}</span>
                `;
            } catch (e) {
                logRequest('api://invalid/path', 500, e.message);
                document.getElementById('error-demo').innerHTML = `
                    <span style="color: #ef4444;">Error: ${e.message}</span>
                `;
            }
        }
    </script>
</body>
</html>
"""


def create_protocol_handlers(view):
    """Create and register custom protocol handlers."""

    def handle_api(uri: str) -> dict:
        """Handle api:// protocol requests."""
        path = uri.replace("api://", "").strip("/")

        if path == "users":
            return {
                "data": json.dumps(SAMPLE_DATA["users"]).encode("utf-8"),
                "mime_type": "application/json",
                "status": 200,
            }
        elif path == "projects":
            return {
                "data": json.dumps(SAMPLE_DATA["projects"]).encode("utf-8"),
                "mime_type": "application/json",
                "status": 200,
            }
        else:
            return {
                "data": b"Not Found",
                "mime_type": "text/plain",
                "status": 404,
            }

    def handle_asset(uri: str) -> dict:
        """Handle asset:// protocol requests for local files."""
        path = uri.replace("asset://", "").strip("/")

        # In a real app, you would load actual files
        # For demo, we return placeholder data
        if path.endswith(".png") or path.endswith(".jpg"):
            # Return a simple 1x1 pixel PNG
            png_data = bytes([
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
                0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
                0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
                0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,
                0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0x3F,
                0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59,
                0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
                0x44, 0xAE, 0x42, 0x60, 0x82,
            ])
            return {
                "data": png_data,
                "mime_type": "image/png",
                "status": 200,
            }

        return {
            "data": b"Asset not found",
            "mime_type": "text/plain",
            "status": 404,
        }

    def handle_dynamic(uri: str) -> dict:
        """Handle dynamic:// protocol for server-side rendered HTML."""
        import datetime
        path = uri.replace("dynamic://", "").strip("/")

        if path == "hello":
            html = """
            <!DOCTYPE html>
            <html>
            <body style="font-family: sans-serif; padding: 20px; background: #f0f9ff;">
                <h2 style="color: #0369a1;">Hello from Python!</h2>
                <p>This page was dynamically generated.</p>
            </body>
            </html>
            """
        elif path == "stats":
            import sys
            html = f"""
            <!DOCTYPE html>
            <html>
            <body style="font-family: sans-serif; padding: 20px; background: #fef3c7;">
                <h2 style="color: #b45309;">System Stats</h2>
                <p>Python: {sys.version.split()[0]}</p>
                <p>Platform: {sys.platform}</p>
            </body>
            </html>
            """
        elif path == "time":
            now = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
            html = f"""
            <!DOCTYPE html>
            <html>
            <body style="font-family: sans-serif; padding: 20px; background: #f0fdf4;">
                <h2 style="color: #166534;">Current Time</h2>
                <p style="font-size: 24px; font-family: monospace;">{now}</p>
            </body>
            </html>
            """
        else:
            html = """
            <!DOCTYPE html>
            <html>
            <body style="font-family: sans-serif; padding: 20px; background: #fef2f2;">
                <h2 style="color: #dc2626;">Page Not Found</h2>
            </body>
            </html>
            """

        return {
            "data": html.encode("utf-8"),
            "mime_type": "text/html",
            "status": 200,
        }

    # Register all protocol handlers
    view.register_protocol("api", handle_api)
    view.register_protocol("asset", handle_asset)
    view.register_protocol("dynamic", handle_dynamic)


def main():
    """Run the custom protocol demo."""
    from auroraview import WebView

    view = WebView(
        html=HTML,
        title="Custom Protocol Demo",
        width=1000,
        height=900,
    )

    create_protocol_handlers(view)

    view.show()


if __name__ == "__main__":
    main()
