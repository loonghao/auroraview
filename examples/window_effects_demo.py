#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Window Effects Demo - Demonstrates click-through and vibrancy effects.

This example shows how to use the window effects APIs:
1. Click-through mode with interactive regions
2. Background blur effects (Blur, Acrylic, Mica, Mica Alt)

Features demonstrated:
- Enable/disable click-through mode
- Define interactive regions where clicks are captured
- Apply various background blur effects (Windows 10/11)
- Dynamic region updates via JavaScript SDK

Platform Support:
- Windows 10 1809+: Blur, Acrylic
- Windows 11: Mica, Mica Alt (in addition to Blur, Acrylic)
- macOS/Linux: Not supported (graceful fallback)

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from auroraview import WebView


def create_demo_html() -> str:
    """Create demo HTML with effect controls."""
    return """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Window Effects Demo</title>
        <meta charset="UTF-8">
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }

            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: rgba(26, 26, 46, 0.85);
                color: #e4e4e4;
                min-height: 100vh;
                padding: 20px;
            }

            h1 {
                color: #00d4ff;
                margin-bottom: 20px;
            }

            .section {
                background: rgba(22, 33, 62, 0.8);
                border-radius: 12px;
                padding: 20px;
                margin-bottom: 20px;
                border: 1px solid rgba(255, 255, 255, 0.1);
            }

            .section-title {
                font-size: 16px;
                font-weight: 600;
                color: #00d4ff;
                margin-bottom: 15px;
                padding-bottom: 10px;
                border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            }

            .button-group {
                display: flex;
                flex-wrap: wrap;
                gap: 10px;
                margin-bottom: 15px;
            }

            button {
                padding: 10px 20px;
                border: none;
                border-radius: 8px;
                cursor: pointer;
                font-size: 14px;
                font-weight: 500;
                transition: all 0.2s ease;
            }

            button:hover {
                transform: translateY(-2px);
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
            }

            button:active {
                transform: translateY(0);
            }

            .btn-primary {
                background: linear-gradient(135deg, #00d4ff 0%, #0099cc 100%);
                color: #1a1a2e;
            }

            .btn-secondary {
                background: linear-gradient(135deg, #6c5ce7 0%, #5541d7 100%);
                color: white;
            }

            .btn-success {
                background: linear-gradient(135deg, #00ff88 0%, #00cc6a 100%);
                color: #1a1a2e;
            }

            .btn-warning {
                background: linear-gradient(135deg, #ffd93d 0%, #f5c800 100%);
                color: #1a1a2e;
            }

            .btn-danger {
                background: linear-gradient(135deg, #ff6b6b 0%, #ee5a5a 100%);
                color: white;
            }

            .btn-mica {
                background: linear-gradient(135deg, #a29bfe 0%, #6c5ce7 100%);
                color: white;
            }

            .status {
                background: rgba(0, 0, 0, 0.3);
                padding: 12px;
                border-radius: 8px;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 13px;
                margin-top: 10px;
            }

            .status-label {
                color: #888;
                margin-right: 8px;
            }

            .status-value {
                color: #00ff88;
            }

            .status-value.disabled {
                color: #ff6b6b;
            }

            /* Interactive region demo */
            .interactive-demo {
                display: grid;
                grid-template-columns: repeat(3, 1fr);
                gap: 15px;
                margin-top: 15px;
            }

            .interactive-box {
                background: rgba(0, 212, 255, 0.2);
                border: 2px dashed #00d4ff;
                border-radius: 8px;
                padding: 20px;
                text-align: center;
                cursor: pointer;
                transition: all 0.2s ease;
            }

            .interactive-box:hover {
                background: rgba(0, 212, 255, 0.4);
                border-style: solid;
            }

            .interactive-box[data-interactive] {
                background: rgba(0, 255, 136, 0.2);
                border-color: #00ff88;
            }

            .interactive-box[data-interactive]:hover {
                background: rgba(0, 255, 136, 0.4);
            }

            .hint {
                font-size: 12px;
                color: #888;
                margin-top: 10px;
            }

            .color-picker {
                display: flex;
                align-items: center;
                gap: 10px;
                margin-top: 10px;
            }

            .color-picker input[type="color"] {
                width: 40px;
                height: 40px;
                border: none;
                border-radius: 8px;
                cursor: pointer;
            }

            .color-picker input[type="range"] {
                flex: 1;
                height: 8px;
                border-radius: 4px;
                background: rgba(255, 255, 255, 0.1);
            }

            .alpha-value {
                width: 50px;
                text-align: right;
                font-family: monospace;
            }
        </style>
    </head>
    <body>
        <h1>🪟 Window Effects Demo</h1>

        <!-- Click-Through Section -->
        <div class="section">
            <div class="section-title">🖱️ Click-Through Mode</div>
            <p style="margin-bottom: 15px; color: #aaa;">
                Enable click-through to let mouse events pass through the window.
                Interactive regions capture clicks while the rest passes through.
            </p>

            <div class="button-group">
                <button class="btn-success" onclick="enableClickThrough()">Enable Click-Through</button>
                <button class="btn-danger" onclick="disableClickThrough()">Disable Click-Through</button>
                <button class="btn-primary" onclick="checkClickThrough()">Check Status</button>
            </div>

            <div class="status">
                <span class="status-label">Click-Through:</span>
                <span class="status-value" id="clickThroughStatus">Unknown</span>
            </div>

            <div class="section-title" style="margin-top: 20px;">Interactive Regions</div>
            <p style="margin-bottom: 10px; color: #aaa;">
                Click boxes to toggle [data-interactive] attribute. Green boxes capture clicks.
            </p>

            <div class="interactive-demo">
                <div class="interactive-box" onclick="toggleInteractive(this)" data-interactive>
                    <strong>Box 1</strong><br>
                    <small>Interactive</small>
                </div>
                <div class="interactive-box" onclick="toggleInteractive(this)">
                    <strong>Box 2</strong><br>
                    <small>Pass-through</small>
                </div>
                <div class="interactive-box" onclick="toggleInteractive(this)" data-interactive>
                    <strong>Box 3</strong><br>
                    <small>Interactive</small>
                </div>
            </div>

            <div class="button-group" style="margin-top: 15px;">
                <button class="btn-secondary" onclick="updateRegions()">Update Regions</button>
                <button class="btn-primary" onclick="getRegions()">Get Current Regions</button>
            </div>

            <div class="hint">
                💡 Use the JS SDK: <code>auroraview.interactive.start()</code> to auto-track [data-interactive] elements
            </div>
        </div>

        <!-- Vibrancy Section -->
        <div class="section">
            <div class="section-title">✨ Background Vibrancy Effects</div>
            <p style="margin-bottom: 15px; color: #aaa;">
                Apply Windows blur effects to the window background.
                Requires Windows 10 1809+ or Windows 11.
            </p>

            <div class="button-group">
                <button class="btn-primary" onclick="applyBlur()">Apply Blur</button>
                <button class="btn-secondary" onclick="applyAcrylic()">Apply Acrylic</button>
                <button class="btn-mica" onclick="applyMica(false)">Apply Mica</button>
                <button class="btn-mica" onclick="applyMica(true)">Mica (Dark)</button>
                <button class="btn-warning" onclick="applyMicaAlt(false)">Mica Alt</button>
                <button class="btn-warning" onclick="applyMicaAlt(true)">Mica Alt (Dark)</button>
            </div>

            <div class="button-group">
                <button class="btn-danger" onclick="clearBlur()">Clear Blur</button>
                <button class="btn-danger" onclick="clearAcrylic()">Clear Acrylic</button>
                <button class="btn-danger" onclick="clearMica()">Clear Mica</button>
                <button class="btn-danger" onclick="clearMicaAlt()">Clear Mica Alt</button>
            </div>

            <div class="color-picker">
                <label>Tint Color:</label>
                <input type="color" id="tintColor" value="#1a1a2e">
                <label>Alpha:</label>
                <input type="range" id="tintAlpha" min="0" max="255" value="200">
                <span class="alpha-value" id="alphaValue">200</span>
            </div>

            <div class="button-group" style="margin-top: 10px;">
                <button class="btn-success" onclick="applyBlurWithColor()">Apply Blur with Tint</button>
                <button class="btn-success" onclick="applyAcrylicWithColor()">Apply Acrylic with Tint</button>
            </div>

            <div class="status">
                <span class="status-label">Current Effect:</span>
                <span class="status-value" id="effectStatus">None</span>
            </div>

            <div class="hint">
                💡 Mica/Mica Alt require Windows 11. Acrylic works on Windows 10 1809+.
            </div>
        </div>

        <script>
            // Update alpha display
            document.getElementById('tintAlpha').addEventListener('input', function() {
                document.getElementById('alphaValue').textContent = this.value;
            });

            // Click-Through functions
            async function enableClickThrough() {
                try {
                    const result = await window.auroraview.api.enable_click_through();
                    document.getElementById('clickThroughStatus').textContent = result ? 'Enabled' : 'Failed';
                    document.getElementById('clickThroughStatus').className = result ? 'status-value' : 'status-value disabled';
                } catch (e) {
                    console.error('Enable click-through failed:', e);
                    document.getElementById('clickThroughStatus').textContent = 'Error: ' + e.message;
                    document.getElementById('clickThroughStatus').className = 'status-value disabled';
                }
            }

            async function disableClickThrough() {
                try {
                    await window.auroraview.api.disable_click_through();
                    document.getElementById('clickThroughStatus').textContent = 'Disabled';
                    document.getElementById('clickThroughStatus').className = 'status-value disabled';
                } catch (e) {
                    console.error('Disable click-through failed:', e);
                }
            }

            async function checkClickThrough() {
                try {
                    const enabled = await window.auroraview.api.is_click_through_enabled();
                    document.getElementById('clickThroughStatus').textContent = enabled ? 'Enabled' : 'Disabled';
                    document.getElementById('clickThroughStatus').className = enabled ? 'status-value' : 'status-value disabled';
                } catch (e) {
                    console.error('Check click-through failed:', e);
                }
            }

            function toggleInteractive(element) {
                if (element.hasAttribute('data-interactive')) {
                    element.removeAttribute('data-interactive');
                    element.querySelector('small').textContent = 'Pass-through';
                } else {
                    element.setAttribute('data-interactive', '');
                    element.querySelector('small').textContent = 'Interactive';
                }
            }

            async function updateRegions() {
                const boxes = document.querySelectorAll('.interactive-box[data-interactive]');
                const regions = Array.from(boxes).map(box => {
                    const rect = box.getBoundingClientRect();
                    return {
                        x: Math.round(rect.left),
                        y: Math.round(rect.top),
                        width: Math.round(rect.width),
                        height: Math.round(rect.height)
                    };
                });

                try {
                    // Pass regions as object parameter
                    await window.auroraview.api.update_interactive_regions({regions: regions});
                    console.log('Updated regions:', regions);
                    alert('Updated ' + regions.length + ' interactive regions');
                } catch (e) {
                    console.error('Update regions failed:', e);
                    alert('Error: ' + e.message);
                }
            }

            async function getRegions() {
                try {
                    const regions = await window.auroraview.api.get_interactive_regions();
                    console.log('Current regions:', regions);
                    alert('Current regions: ' + JSON.stringify(regions, null, 2));
                } catch (e) {
                    console.error('Get regions failed:', e);
                }
            }

            // Vibrancy functions
            function getTintColor() {
                const hex = document.getElementById('tintColor').value;
                const alpha = parseInt(document.getElementById('tintAlpha').value);
                const r = parseInt(hex.slice(1, 3), 16);
                const g = parseInt(hex.slice(3, 5), 16);
                const b = parseInt(hex.slice(5, 7), 16);
                return {color: [r, g, b, alpha]};
            }

            async function applyBlur() {
                try {
                    await window.auroraview.api.apply_blur();
                    document.getElementById('effectStatus').textContent = 'Blur';
                } catch (e) {
                    console.error('Apply blur failed:', e);
                    alert('Error: ' + e.message);
                }
            }

            async function applyBlurWithColor() {
                try {
                    const params = getTintColor();
                    await window.auroraview.api.apply_blur(params);
                    document.getElementById('effectStatus').textContent = 'Blur (tinted)';
                } catch (e) {
                    console.error('Apply blur with color failed:', e);
                    alert('Error: ' + e.message);
                }
            }

            async function applyAcrylic() {
                try {
                    await window.auroraview.api.apply_acrylic();
                    document.getElementById('effectStatus').textContent = 'Acrylic';
                } catch (e) {
                    console.error('Apply acrylic failed:', e);
                    alert('Error: ' + e.message);
                }
            }

            async function applyAcrylicWithColor() {
                try {
                    const params = getTintColor();
                    await window.auroraview.api.apply_acrylic(params);
                    document.getElementById('effectStatus').textContent = 'Acrylic (tinted)';
                } catch (e) {
                    console.error('Apply acrylic with color failed:', e);
                    alert('Error: ' + e.message);
                }
            }

            async function applyMica(dark) {
                try {
                    await window.auroraview.api.apply_mica({dark: dark});
                    document.getElementById('effectStatus').textContent = dark ? 'Mica (Dark)' : 'Mica';
                } catch (e) {
                    console.error('Apply mica failed:', e);
                    alert('Error: ' + e.message);
                }
            }

            async function applyMicaAlt(dark) {
                try {
                    await window.auroraview.api.apply_mica_alt({dark: dark});
                    document.getElementById('effectStatus').textContent = dark ? 'Mica Alt (Dark)' : 'Mica Alt';
                } catch (e) {
                    console.error('Apply mica alt failed:', e);
                    alert('Error: ' + e.message);
                }
            }

            async function clearBlur() {
                try {
                    await window.auroraview.api.clear_blur();
                    document.getElementById('effectStatus').textContent = 'None';
                } catch (e) {
                    console.error('Clear blur failed:', e);
                }
            }

            async function clearAcrylic() {
                try {
                    await window.auroraview.api.clear_acrylic();
                    document.getElementById('effectStatus').textContent = 'None';
                } catch (e) {
                    console.error('Clear acrylic failed:', e);
                }
            }

            async function clearMica() {
                try {
                    await window.auroraview.api.clear_mica();
                    document.getElementById('effectStatus').textContent = 'None';
                } catch (e) {
                    console.error('Clear mica failed:', e);
                }
            }

            async function clearMicaAlt() {
                try {
                    await window.auroraview.api.clear_mica_alt();
                    document.getElementById('effectStatus').textContent = 'None';
                } catch (e) {
                    console.error('Clear mica alt failed:', e);
                }
            }

            // Initialize
            window.addEventListener('auroraviewready', () => {
                console.log('AuroraView ready');
                checkClickThrough();
            });
        </script>
    </body>
    </html>
    """


class WindowEffectsApi:
    """API class for window effects exposed to JavaScript."""

    def __init__(self, webview: WebView):
        self._webview = webview
        # Access the Rust core directly
        self._core = webview._core

    def enable_click_through(self) -> bool:
        """Enable click-through mode."""
        return self._core.enable_click_through()

    def disable_click_through(self) -> None:
        """Disable click-through mode."""
        self._core.disable_click_through()

    def is_click_through_enabled(self) -> bool:
        """Check if click-through is enabled."""
        return self._core.is_click_through_enabled()

    def update_interactive_regions(self, regions: list) -> None:
        """Update interactive regions."""
        from auroraview._core import PyRegion

        py_regions = [
            PyRegion(r["x"], r["y"], r["width"], r["height"]) for r in regions
        ]
        self._core.update_interactive_regions(py_regions)

    def get_interactive_regions(self) -> list:
        """Get current interactive regions."""
        regions = self._core.get_interactive_regions()
        return [
            {"x": r.x, "y": r.y, "width": r.width, "height": r.height} for r in regions
        ]

    def apply_blur(self, color=None) -> bool:
        """Apply blur effect.

        Args:
            color: Optional color as [r, g, b, a] list or (r, g, b, a) tuple
        """
        if color is not None and isinstance(color, list):
            color = tuple(color)
        return self._core.apply_blur(color)

    def clear_blur(self) -> None:
        """Clear blur effect."""
        self._core.clear_blur()

    def apply_acrylic(self, color=None) -> bool:
        """Apply acrylic effect.

        Args:
            color: Optional color as [r, g, b, a] list or (r, g, b, a) tuple
        """
        if color is not None and isinstance(color, list):
            color = tuple(color)
        return self._core.apply_acrylic(color)

    def clear_acrylic(self) -> None:
        """Clear acrylic effect."""
        self._core.clear_acrylic()

    def apply_mica(self, dark: bool = False) -> bool:
        """Apply mica effect."""
        return self._core.apply_mica(dark)

    def clear_mica(self) -> None:
        """Clear mica effect."""
        self._core.clear_mica()

    def apply_mica_alt(self, dark: bool = False) -> bool:
        """Apply mica alt effect."""
        return self._core.apply_mica_alt(dark)

    def clear_mica_alt(self) -> None:
        """Clear mica alt effect."""
        self._core.clear_mica_alt()


def main():
    """Run the window effects demo."""
    # Create WebView with transparent background for vibrancy effects
    webview = WebView(
        title="Window Effects Demo",
        width=800,
        height=900,
        resizable=True,
        transparent=True,  # Required for vibrancy effects
    )

    # Create API and bind it
    api = WindowEffectsApi(webview)
    webview.bind_api(api, "api")

    # Load HTML and show
    webview.load_html(create_demo_html())
    webview.show()


if __name__ == "__main__":
    main()
