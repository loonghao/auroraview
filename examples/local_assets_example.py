"""Example: Loading HTML with local assets using file:// protocol.

This example demonstrates how to use file:// URLs in HTML content
to load local files (images, GIFs, CSS, JS, etc.) in run_standalone().

IMPORTANT: You must set allow_file_protocol=True to enable file:// support!
"""


from auroraview import run_standalone


def main():
    """Run standalone WebView with local assets using file:// URLs."""
    # Create a simple example HTML with inline content
    # In real usage, you would load actual local files

    # Example: If you have local files, convert them to file:/// URLs like this:
    # from pathlib import Path
    # gif_path = Path("path/to/animation.gif").resolve()
    # gif_url = f"file:///{str(gif_path).replace(os.sep, '/')}"

    print("=" * 80)
    print("file:// Protocol Example")
    print("=" * 80)
    print("This example shows how to use file:// URLs in HTML content.")
    print("To use with real files, replace the inline SVG with actual file:// URLs.")
    print("=" * 80)

    # Create HTML with inline SVG (no external files needed for this demo)
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>Local Assets Example</title>
        <style>
            body {
                margin: 0;
                padding: 20px;
                font-family: system-ui, -apple-system, sans-serif;
                background: #020617;
                color: #e2e8f0;
            }
            .container {
                max-width: 800px;
                margin: 0 auto;
            }
            h1 {
                color: #60a5fa;
            }
            .asset-demo {
                margin: 20px 0;
                padding: 20px;
                background: #1e293b;
                border-radius: 8px;
            }
            .code {
                background: #0f172a;
                padding: 10px;
                border-radius: 4px;
                font-family: 'Courier New', monospace;
                font-size: 12px;
                overflow-x: auto;
                white-space: pre-wrap;
            }
            .success {
                background: #10b981;
                color: white;
                padding: 15px;
                border-radius: 8px;
                margin: 20px 0;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🎨 file:// Protocol Example</h1>

            <div class="success">
                <strong>✓ file:// protocol is enabled!</strong><br>
                This WebView can load local files using file:/// URLs.
            </div>

            <div class="asset-demo">
                <h2>📁 How to Use file:// Protocol</h2>
                <p>To enable <code>file://</code> protocol support:</p>
                <div class="code">from auroraview import run_standalone

run_standalone(
    title="My App",
    html=html_content,
    allow_file_protocol=True,  # ← Required!
)</div>
            </div>

            <div class="asset-demo">
                <h2>🔗 Converting Paths to file:/// URLs</h2>
                <p>Use this pattern to convert local file paths:</p>
                <div class="code">from pathlib import Path
import os

# Convert path to file:/// URL
file_path = Path("path/to/file.gif").resolve()
path_str = str(file_path).replace(os.sep, "/")
if not path_str.startswith("/"):
    path_str = "/" + path_str
file_url = f"file://{path_str}"

# Use in HTML
html = f'&lt;img src="{file_url}"&gt;'</div>
            </div>

            <div class="asset-demo">
                <h2>📝 Example Usage</h2>
                <p>Load local images, CSS, JS, and HTML files:</p>
                <div class="code"># Example file:/// URLs:
# Windows: file:///C:/Users/user/image.gif
# Unix:    file:///home/user/image.gif

html = '''
&lt;link href="file:///path/to/style.css" rel="stylesheet"&gt;
&lt;script src="file:///path/to/app.js"&gt;&lt;/script&gt;
&lt;img src="file:///path/to/image.png"&gt;
&lt;iframe src="file:///path/to/page.html"&gt;&lt;/iframe&gt;
'''</div>
            </div>

            <div class="asset-demo">
                <h2>⚠️ Security Note</h2>
                <p>Enabling <code>file://</code> protocol allows access to any file the process can read.</p>
                <p>Only use with trusted content!</p>
            </div>
        </div>
    </body>
    </html>
    """

    # Run standalone WebView
    # IMPORTANT: allow_file_protocol=True is required for file:// URLs!
    run_standalone(
        title="Local Assets Example - file:// Protocol",
        width=1024,
        height=768,
        html=html_content,
        dev_tools=True,  # Enable dev tools for debugging
        allow_file_protocol=True,  # ← Required for file:/// URLs!
    )


if __name__ == "__main__":
    main()

