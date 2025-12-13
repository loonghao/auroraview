"""Dynamic Binding Pattern Example - AuroraView API Demo.

This example demonstrates advanced runtime binding for plugin systems
and dynamic configurations. Best for extensible applications.

Note: This example uses the low-level WebView API for demonstration.
For most use cases, prefer QtWebView, AuroraView, or run_desktop.

Usage:
    python examples/dynamic_binding.py

Features demonstrated:
    - Runtime API binding with bind_call()
    - Dynamic feature loading based on configuration
    - Signal connections with on_ready, on_navigate
    - Plugin-like architecture
    - Conditional API registration

Use cases:
    - Plugin systems that register APIs at runtime
    - Feature flags that enable/disable functionality
    - Configuration-driven API exposure
    - Multi-tenant applications with different capabilities
"""

from __future__ import annotations

import json

from auroraview import WebView


def create_plugin_host():
    """Create a WebView that acts as a plugin host."""
    # HTML content for the plugin host demo
    html_content = """
<!DOCTYPE html>
<html>
<head>
    <title>Plugin Host Demo</title>
    <style>
        * { box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e0e0e0;
            padding: 20px;
            min-height: 100vh;
        }
        h1 { color: #4fc3f7; margin-bottom: 8px; }
        .subtitle { color: #888; margin-bottom: 24px; }
        .section {
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 20px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        .section h2 { color: #81d4fa; font-size: 16px; margin-bottom: 12px; }
        .plugin-card {
            background: rgba(255,255,255,0.08);
            border-radius: 8px;
            padding: 12px;
            margin: 8px 0;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .plugin-name { font-weight: 500; }
        .plugin-status { font-size: 12px; color: #888; }
        .plugin-status.active { color: #4caf50; }
        button {
            background: #4fc3f7;
            color: #1a1a2e;
            border: none;
            padding: 8px 16px;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 500;
            transition: all 0.15s;
        }
        button:hover { background: #81d4fa; transform: translateY(-1px); }
        button:disabled { opacity: 0.5; cursor: not-allowed; transform: none; }
        .feature-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; }
        .feature-btn {
            padding: 16px;
            text-align: center;
            background: rgba(79, 195, 247, 0.1);
            border: 1px solid rgba(79, 195, 247, 0.3);
        }
        .feature-btn:disabled { background: rgba(255,255,255,0.02); border-color: rgba(255,255,255,0.1); }
        #output {
            background: #0d1117;
            border-radius: 8px;
            padding: 16px;
            font-family: 'Fira Code', monospace;
            font-size: 12px;
            max-height: 200px;
            overflow-y: auto;
            white-space: pre-wrap;
        }
        .log-entry { margin: 4px 0; }
        .log-time { color: #586069; }
        .log-success { color: #4caf50; }
        .log-error { color: #f44336; }
        .log-info { color: #4fc3f7; }
    </style>
</head>
<body>
    <h1>🔌 Plugin Host Demo</h1>
    <p class="subtitle">Dynamic API binding based on configuration</p>

    <div class="section">
        <h2>📦 Available Features</h2>
        <div class="feature-grid">
            <button class="feature-btn" id="btn-export" onclick="tryExport()">
                📤 Export Data
            </button>
            <button class="feature-btn" id="btn-import" onclick="tryImport()">
                📥 Import Data
            </button>
            <button class="feature-btn" id="btn-analytics" onclick="tryAnalytics()">
                📊 Analytics
            </button>
            <button class="feature-btn" id="btn-admin" onclick="tryAdmin()">
                🔐 Admin Panel
            </button>
        </div>
    </div>

    <div class="section">
        <h2>🧩 Loaded Plugins</h2>
        <div id="plugins"></div>
        <button onclick="loadPlugins()" style="margin-top: 12px;">Reload Plugins</button>
    </div>

    <div class="section">
        <h2>📜 Activity Log</h2>
        <div id="output"></div>
    </div>

    <script>
        function log(msg, type = 'info') {
            const output = document.getElementById('output');
            const time = new Date().toLocaleTimeString();
            output.innerHTML = `<div class="log-entry"><span class="log-time">[${time}]</span> ` +
                `<span class="log-${type}">${msg}</span></div>` + output.innerHTML;
        }

        async function checkFeature(name) {
            try {
                const result = await auroraview.api.has_feature({name});
                return result.available;
            } catch { return false; }
        }

        async function updateFeatureButtons() {
            const features = ['export', 'import', 'analytics', 'admin'];
            for (const f of features) {
                const btn = document.getElementById(`btn-${f}`);
                const available = await checkFeature(f);
                btn.disabled = !available;
                btn.title = available ? `${f} is enabled` : `${f} is not enabled`;
            }
        }

        async function tryExport() {
            try {
                const result = await auroraview.api.export_data({format: 'json'});
                log(`Export: ${JSON.stringify(result)}`, 'success');
            } catch (e) { log(`Export failed: ${e}`, 'error'); }
        }

        async function tryImport() {
            try {
                const result = await auroraview.api.import_data({data: '{"test": 1}'});
                log(`Import: ${JSON.stringify(result)}`, 'success');
            } catch (e) { log(`Import failed: ${e}`, 'error'); }
        }

        async function tryAnalytics() {
            try {
                const result = await auroraview.api.get_analytics();
                log(`Analytics: ${JSON.stringify(result)}`, 'success');
            } catch (e) { log(`Analytics failed: ${e}`, 'error'); }
        }

        async function tryAdmin() {
            try {
                const result = await auroraview.api.admin_action({action: 'list_users'});
                log(`Admin: ${JSON.stringify(result)}`, 'success');
            } catch (e) { log(`Admin failed: ${e}`, 'error'); }
        }

        async function loadPlugins() {
            try {
                const result = await auroraview.api.get_plugins();
                const container = document.getElementById('plugins');
                container.innerHTML = result.plugins.map(p => `
                    <div class="plugin-card">
                        <div>
                            <div class="plugin-name">${p.name}</div>
                            <div class="plugin-status ${p.active ? 'active' : ''}">${p.active ? '● Active' : '○ Inactive'}</div>
                        </div>
                        <button onclick="activatePlugin('${p.id}')" ${p.active ? 'disabled' : ''}>
                            ${p.active ? 'Loaded' : 'Load'}
                        </button>
                    </div>
                `).join('');
                log(`Loaded ${result.plugins.length} plugins`, 'info');
            } catch (e) { log(`Failed to load plugins: ${e}`, 'error'); }
        }

        async function activatePlugin(id) {
            try {
                const result = await auroraview.api.activate_plugin({plugin_id: id});
                log(`Plugin activated: ${result.name}`, 'success');
                loadPlugins();
                updateFeatureButtons();
            } catch (e) { log(`Failed to activate plugin: ${e}`, 'error'); }
        }

        // Listen for Python events
        auroraview.on('plugin_loaded', (data) => {
            log(`Plugin loaded: ${data.name}`, 'success');
            loadPlugins();
            updateFeatureButtons();
        });

        auroraview.on('feature_enabled', (data) => {
            log(`Feature enabled: ${data.feature}`, 'info');
            updateFeatureButtons();
        });

        // Initial load
        loadPlugins();
        updateFeatureButtons();
    </script>
</body>
</html>
"""

    view = WebView(title="Plugin Host Demo", html=html_content, width=600, height=700, debug=True)

    # ═══════════════════════════════════════════════════════════════════
    # Configuration-driven feature flags
    # ═══════════════════════════════════════════════════════════════════
    config = {
        "features": ["export", "import"],  # Enabled features
        "plugins": [
            {"id": "analytics", "name": "Analytics Plugin", "active": False},
            {"id": "admin", "name": "Admin Tools", "active": False},
            {"id": "export_pro", "name": "Export Pro", "active": True},
        ],
    }

    # ═══════════════════════════════════════════════════════════════════
    # Core API methods (always available)
    # ═══════════════════════════════════════════════════════════════════

    def get_plugins() -> dict:
        """Get list of available plugins."""
        return {"plugins": config["plugins"]}

    def activate_plugin(plugin_id: str = "") -> dict:
        """Activate a plugin and register its APIs."""
        for plugin in config["plugins"]:
            if plugin["id"] == plugin_id:
                plugin["active"] = True

                # Dynamically register plugin APIs
                if plugin_id == "analytics":
                    config["features"].append("analytics")
                    view.bind_call("get_analytics", lambda: {"views": 1234, "users": 56})
                elif plugin_id == "admin":
                    config["features"].append("admin")
                    view.bind_call("admin_action", lambda action="": {"action": action, "users": ["admin", "user1"]})

                view.emit("plugin_loaded", {"id": plugin_id, "name": plugin["name"]})
                return {"ok": True, "name": plugin["name"]}

        return {"ok": False, "error": "Plugin not found"}

    def has_feature(name: str = "") -> dict:
        """Check if a feature is available."""
        return {"available": name in config["features"], "feature": name}

    # Bind core APIs
    view.bind_call("get_plugins", get_plugins)
    view.bind_call("activate_plugin", activate_plugin)
    view.bind_call("has_feature", has_feature)

    # ═══════════════════════════════════════════════════════════════════
    # Conditionally bind APIs based on configuration
    # ═══════════════════════════════════════════════════════════════════

    if "export" in config["features"]:
        print("[Config] Export feature enabled")

        def export_data(format: str = "json") -> dict:
            """Export data in specified format."""
            return {"ok": True, "format": format, "data": '{"exported": true}', "size": 42}

        view.bind_call("export_data", export_data)

    if "import" in config["features"]:
        print("[Config] Import feature enabled")

        def import_data(data: str = "") -> dict:
            """Import data from string."""
            try:
                parsed = json.loads(data) if data else {}
                return {"ok": True, "imported": len(parsed), "data": parsed}
            except json.JSONDecodeError as e:
                return {"ok": False, "error": str(e)}

        view.bind_call("import_data", import_data)

    # ═══════════════════════════════════════════════════════════════════
    # Connect to built-in signals
    # ═══════════════════════════════════════════════════════════════════

    view.on_ready.connect(lambda: print("[Signal] WebView is ready!"))
    view.on_navigate.connect(lambda url: print(f"[Signal] Navigated to: {url}"))

    # ═══════════════════════════════════════════════════════════════════
    # Register event handlers
    # ═══════════════════════════════════════════════════════════════════

    @view.on("plugin_event")
    def handle_plugin_event(data: dict):
        """Handle events from plugins."""
        print(f"[Event] Plugin event: {data}")

    return view


def main():
    """Run the dynamic binding example."""
    print("Starting Plugin Host Demo (Dynamic Binding Pattern)...")
    print()
    print("This example demonstrates:")
    print("  - Runtime API binding with bind_call()")
    print("  - Configuration-driven feature flags")
    print("  - Dynamic plugin loading")
    print("  - Signal connections (on_ready, on_navigate)")
    print()
    print("Enabled features: export, import")
    print("Available plugins: analytics, admin, export_pro")
    print()

    view = create_plugin_host()
    view.show()


if __name__ == "__main__":
    main()

