"""Test loading public URLs to diagnose loading issues."""

import sys
import time


def test_baidu():
    """Test loading baidu.com."""
    print("\n=== Testing baidu.com ===")
    from auroraview import WebView

    webview = WebView(title="Test Baidu", width=1000, height=800, debug=True)

    print("Loading https://www.baidu.com...")
    webview.load_url("https://www.baidu.com")

    print("Showing WebView...")
    print(
        "Check if the page loads. If you see white screen, open DevTools (F12 or right-click -> Inspect)"
    )
    webview.show()


def test_example():
    """Test loading example.com."""
    print("\n=== Testing example.com ===")
    from auroraview import WebView

    webview = WebView(title="Test Example.com", width=1000, height=800, debug=True)

    print("Loading https://www.example.com...")
    webview.load_url("https://www.example.com")

    print("Showing WebView...")
    webview.show()


def test_google():
    """Test loading google.com."""
    print("\n=== Testing google.com ===")
    from auroraview import WebView

    webview = WebView(title="Test Google", width=1000, height=800, debug=True)

    print("Loading https://www.google.com...")
    webview.load_url("https://www.google.com")

    print("Showing WebView...")
    webview.show()


def test_with_delay():
    """Test loading with delay to see if it's a timing issue."""
    print("\n=== Testing with delay ===")
    from auroraview import WebView

    webview = WebView(title="Test with Delay", width=1000, height=800, debug=True)

    print("Loading https://www.baidu.com...")
    webview.load_url("https://www.baidu.com")

    print("Waiting 3 seconds before showing...")
    time.sleep(3)

    print("Showing WebView...")
    webview.show()


def test_http_vs_https():
    """Test HTTP vs HTTPS."""
    print("\n=== Testing HTTP vs HTTPS ===")
    from auroraview import WebView

    # Test HTTP
    print("\n1. Testing HTTP (http://example.com)...")
    webview1 = WebView(title="Test HTTP", width=1000, height=800, debug=True)
    webview1.load_url("http://example.com")
    webview1.show()

    # Test HTTPS
    print("\n2. Testing HTTPS (https://example.com)...")
    webview2 = WebView(title="Test HTTPS", width=1000, height=800, debug=True)
    webview2.load_url("https://example.com")
    webview2.show()


def test_url_in_constructor():
    """Test passing URL in constructor vs load_url()."""
    print("\n=== Testing URL in constructor ===")
    from auroraview import WebView

    # Method 1: Pass URL in constructor
    print("\n1. Passing URL in constructor...")
    webview1 = WebView(
        title="URL in Constructor", width=1000, height=800, url="https://www.baidu.com", debug=True
    )
    webview1.show()

    # Method 2: Use load_url()
    print("\n2. Using load_url()...")
    webview2 = WebView(title="Using load_url()", width=1000, height=800, debug=True)
    webview2.load_url("https://www.baidu.com")
    webview2.show()


def test_with_user_agent():
    """Test if user agent matters."""
    print("\n=== Testing with custom user agent ===")
    from auroraview import WebView

    webview = WebView(title="Test User Agent", width=1000, height=800, debug=True)

    # Load HTML that shows user agent
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>User Agent Test</title>
        <style>
            body {
                font-family: Arial, sans-serif;
                padding: 40px;
                background: #f5f5f5;
            }
            .info {
                background: white;
                padding: 20px;
                border-radius: 8px;
                margin-bottom: 20px;
            }
            h1 { color: #333; }
            pre {
                background: #f0f0f0;
                padding: 15px;
                border-radius: 4px;
                overflow-x: auto;
            }
            button {
                padding: 10px 20px;
                font-size: 16px;
                background: #667eea;
                color: white;
                border: none;
                border-radius: 4px;
                cursor: pointer;
            }
            button:hover {
                background: #5568d3;
            }
            #result {
                margin-top: 20px;
                padding: 15px;
                background: #e7f3ff;
                border-radius: 4px;
            }
        </style>
    </head>
    <body>
        <div class="info">
            <h1>WebView Information</h1>
            <h2>User Agent:</h2>
            <pre id="userAgent"></pre>
            
            <h2>Platform:</h2>
            <pre id="platform"></pre>
            
            <h2>Browser Info:</h2>
            <pre id="browserInfo"></pre>
        </div>
        
        <div class="info">
            <h2>Test External URL Loading</h2>
            <button onclick="testBaidu()">Load Baidu.com</button>
            <button onclick="testExample()">Load Example.com</button>
            <button onclick="testGoogle()">Load Google.com</button>
            <div id="result"></div>
        </div>
        
        <script>
            // Display user agent info
            document.getElementById('userAgent').textContent = navigator.userAgent;
            document.getElementById('platform').textContent = navigator.platform;
            document.getElementById('browserInfo').textContent = JSON.stringify({
                appName: navigator.appName,
                appVersion: navigator.appVersion,
                vendor: navigator.vendor,
                language: navigator.language
            }, null, 2);
            
            function testBaidu() {
                const result = document.getElementById('result');
                result.innerHTML = '<p>Attempting to load baidu.com...</p>';
                
                fetch('https://www.baidu.com', { mode: 'no-cors' })
                    .then(() => {
                        result.innerHTML = '<p style="color: green;">✅ Baidu.com is accessible</p>';
                    })
                    .catch(error => {
                        result.innerHTML = '<p style="color: red;">❌ Error: ' + error.message + '</p>';
                    });
            }
            
            function testExample() {
                const result = document.getElementById('result');
                result.innerHTML = '<p>Attempting to load example.com...</p>';
                
                fetch('https://www.example.com', { mode: 'no-cors' })
                    .then(() => {
                        result.innerHTML = '<p style="color: green;">✅ Example.com is accessible</p>';
                    })
                    .catch(error => {
                        result.innerHTML = '<p style="color: red;">❌ Error: ' + error.message + '</p>';
                    });
            }
            
            function testGoogle() {
                const result = document.getElementById('result');
                result.innerHTML = '<p>Attempting to load google.com...</p>';
                
                fetch('https://www.google.com', { mode: 'no-cors' })
                    .then(() => {
                        result.innerHTML = '<p style="color: green;">✅ Google.com is accessible</p>';
                    })
                    .catch(error => {
                        result.innerHTML = '<p style="color: red;">❌ Error: ' + error.message + '</p>';
                    });
            }
            
            console.log('User Agent:', navigator.userAgent);
            console.log('Platform:', navigator.platform);
        </script>
    </body>
    </html>
    """

    webview.load_html(html)
    print("Showing WebView with user agent info...")
    print("Try clicking the buttons to test URL loading from JavaScript")
    webview.show()


def main():
    """Main function."""
    print("=" * 60)
    print("AuroraView Public URL Loading Test")
    print("=" * 60)
    print("\nAvailable tests:")
    print("  1 - Test baidu.com")
    print("  2 - Test example.com")
    print("  3 - Test google.com")
    print("  4 - Test with delay")
    print("  5 - Test HTTP vs HTTPS")
    print("  6 - Test URL in constructor vs load_url()")
    print("  7 - Test with user agent info")
    print()

    if len(sys.argv) > 1:
        choice = sys.argv[1]
    else:
        choice = input("Enter choice (1-7) [1]: ").strip() or "1"

    if choice == "1":
        test_baidu()
    elif choice == "2":
        test_example()
    elif choice == "3":
        test_google()
    elif choice == "4":
        test_with_delay()
    elif choice == "5":
        test_http_vs_https()
    elif choice == "6":
        test_url_in_constructor()
    elif choice == "7":
        test_with_user_agent()
    else:
        print(f"Invalid choice: {choice}")
        print("Running default test (baidu.com)...")
        test_baidu()


if __name__ == "__main__":
    main()
