#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""AuroraView Desktop App - IPC File Drop Demo

Demonstrates how to opt-in to OS-level drag-and-drop interception so that
dropped files are forwarded as IPC ``file_drop`` events with absolute paths.

Key points:
- ``capture_file_drop=True`` is required since v0.6 (default changed to False).
- Enabling capture_file_drop **disables** HTML5 ``dragover`` / ``drop`` inside
  the WebView (wry/WebView2 upstream limitation).
- Use this when your tool needs absolute OS paths (e.g. importing assets).
- If you only need browser-native drag-and-drop (e.g. reordering list items),
  leave ``capture_file_drop`` at the default (False/None).

See also:
- docs/zh/guide/file-drop.md  (full guide)
- examples/dcc_integration_example.py  (DCC panel with opt-in)
"""

from auroraview import run_desktop


def _build_html():
    """Build the demo UI HTML."""
    return """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>File Drop Demo</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1e1e2e;
            color: #cdd6f4;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 2rem;
        }
        h1 { color: #89b4fa; margin-bottom: 1rem; }
        .drop-zone {
            width: 100%;
            max-width: 600px;
            min-height: 200px;
            border: 2px dashed #585b70;
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 2rem;
            transition: border-color 0.2s, background 0.2s;
        }
        .drop-zone.hovering {
            border-color: #89b4fa;
            background: rgba(137, 180, 250, 0.08);
        }
        .drop-zone p { color: #a6adc8; font-size: 1.1rem; }
        #file-list {
            margin-top: 1.5rem;
            width: 100%;
            max-width: 600px;
        }
        .file-item {
            background: #313244;
            border-radius: 8px;
            padding: 0.75rem 1rem;
            margin-bottom: 0.5rem;
            font-family: 'JetBrains Mono', monospace;
            font-size: 0.85rem;
            word-break: break-all;
        }
        .note {
            margin-top: 2rem;
            color: #a6adc8;
            font-size: 0.8rem;
            text-align: center;
            max-width: 600px;
        }
    </style>
</head>
<body>
    <h1>IPC File Drop Demo</h1>
    <div class="drop-zone" id="drop-zone">
        <p>Drag files here &mdash; paths arrive via IPC</p>
    </div>
    <div id="file-list"></div>
    <p class="note">
        capture_file_drop=True intercepts OS drag-and-drop.
        HTML5 dragover/drop events are NOT fired in this mode.
    </p>
    <script>
        const dropZone = document.getElementById('drop-zone');
        const fileList = document.getElementById('file-list');

        // Listen for IPC file_drop event from Python/Rust
        auroraview.on('file_drop', (data) => {
            dropZone.classList.remove('hovering');
            const paths = data.paths || [];
            paths.forEach(path => {
                const div = document.createElement('div');
                div.className = 'file-item';
                div.textContent = path;
                fileList.prepend(div);
            });
        });

        // Visual feedback during hover
        auroraview.on('file_drop_hover', (data) => {
            if (data.hovering) {
                dropZone.classList.add('hovering');
            } else {
                dropZone.classList.remove('hovering');
            }
        });

        auroraview.on('file_drop_cancelled', () => {
            dropZone.classList.remove('hovering');
        });
    </script>
</body>
</html>
"""


def main():
    """Launch the file-drop demo with capture_file_drop enabled."""
    run_desktop(
        title="File Drop Demo",
        html=_build_html(),
        width=720,
        height=560,
        capture_file_drop=True,  # <-- opt-in to IPC file drop
    )


if __name__ == "__main__":
    main()
