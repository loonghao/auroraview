"""
Test script for new factory methods API.

This script demonstrates the improved API design using factory methods:
- NativeWebView.standalone() - for standalone windows
- NativeWebView.embedded() - for DCC integration

The factory methods provide a clearer, more intuitive API compared to
the legacy constructor with optional parent_hwnd/parent_mode parameters.
"""

from auroraview import NativeWebView


def test_standalone():
    """Test standalone window creation using factory method."""
    print("=" * 60)
    print("Testing NativeWebView.standalone()")
    print("=" * 60)
    
    # Create standalone window - much clearer than legacy API
    webview = NativeWebView.standalone(
        title="Standalone Window Test",
        width=800,
        height=600,
        decorations=True,
        resizable=True,
        dev_tools=True,
    )
    
    # Load simple HTML
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <title>Standalone Test</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                display: flex;
                justify-content: center;
                align-items: center;
                height: 100vh;
                margin: 0;
            }
            .container {
                text-align: center;
                padding: 40px;
                background: rgba(255, 255, 255, 0.1);
                border-radius: 20px;
                backdrop-filter: blur(10px);
            }
            h1 {
                font-size: 48px;
                margin: 0 0 20px 0;
            }
            p {
                font-size: 18px;
                opacity: 0.9;
            }
            .api-info {
                margin-top: 30px;
                padding: 20px;
                background: rgba(0, 0, 0, 0.2);
                border-radius: 10px;
                font-family: 'Courier New', monospace;
                font-size: 14px;
                text-align: left;
            }
            .api-info code {
                color: #ffd700;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>✨ Standalone Window</h1>
            <p>Created using factory method</p>
            <div class="api-info">
                <strong>API Usage:</strong><br>
                <code>webview = NativeWebView.standalone(</code><br>
                <code>&nbsp;&nbsp;title="My App",</code><br>
                <code>&nbsp;&nbsp;width=800,</code><br>
                <code>&nbsp;&nbsp;height=600,</code><br>
                <code>&nbsp;&nbsp;decorations=True</code><br>
                <code>)</code>
            </div>
        </div>
    </body>
    </html>
    """
    
    webview.load_html(html)
    
    print("✅ Standalone window created successfully")
    print("   - Using: NativeWebView.standalone()")
    print("   - No parent_hwnd or parent_mode needed")
    print("   - Clear and intuitive API")
    print("")
    print("Showing window (blocking until closed)...")
    
    webview.show()
    
    print("✅ Window closed")


def test_api_comparison():
    """Show comparison between old and new API."""
    print("\n" + "=" * 60)
    print("API Comparison: Old vs New")
    print("=" * 60)
    
    print("\n❌ OLD API (Confusing):")
    print("-" * 60)
    print("""
# Standalone window - unclear what None means
webview = NativeWebView(
    title="My App",
    parent_hwnd=None,      # What does None mean?
    parent_mode=None       # Why do I need this?
)

# Embedded window - too many parameters
webview = NativeWebView(
    title="Maya Tool",
    parent_hwnd=hwnd,      # What's hwnd?
    parent_mode="owner"    # owner vs child?
)
    """)
    
    print("\n✅ NEW API (Clear):")
    print("-" * 60)
    print("""
# Standalone window - crystal clear
webview = NativeWebView.standalone(
    title="My App",
    width=800,
    height=600
)

# Embedded window - explicit and clear
webview = NativeWebView.embedded(
    parent_hwnd=maya_hwnd,
    title="Maya Tool",
    mode="owner"  # Default, recommended
)
    """)
    
    print("\n📚 Benefits of New API:")
    print("-" * 60)
    print("1. ✅ Clear intent - standalone() vs embedded()")
    print("2. ✅ No confusing None parameters")
    print("3. ✅ Better parameter names (mode vs parent_mode)")
    print("4. ✅ Self-documenting code")
    print("5. ✅ Easier to discover via IDE autocomplete")
    print("6. ✅ Backward compatible - old API still works")
    print("")


if __name__ == "__main__":
    # Show API comparison first
    test_api_comparison()
    
    # Test standalone window
    input("\nPress Enter to test standalone window...")
    test_standalone()
    
    print("\n" + "=" * 60)
    print("✅ All tests completed successfully!")
    print("=" * 60)

