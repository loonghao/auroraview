"""Test WebView with different scenarios to diagnose white screen issue."""

import sys
import time


def test_basic_html():
    """Test 1: Load simple HTML content directly."""
    print("\n=== Test 1: Loading HTML content directly ===")
    from auroraview import WebView

    webview = WebView(
        title="Test 1: HTML Content",
        width=800,
        height=600,
    )
    
    # Load HTML content instead of URL
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Page</title>
        <style>
            body {
                font-family: Arial, sans-serif;
                display: flex;
                justify-content: center;
                align-items: center;
                height: 100vh;
                margin: 0;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            }
            .container {
                text-align: center;
                color: white;
            }
            h1 {
                font-size: 48px;
                margin-bottom: 20px;
            }
            p {
                font-size: 24px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>✅ AuroraView Works!</h1>
            <p>If you can see this, the WebView is working correctly.</p>
            <p id="time"></p>
        </div>
        <script>
            // Update time every second
            setInterval(() => {
                document.getElementById('time').textContent = 
                    'Current time: ' + new Date().toLocaleTimeString();
            }, 1000);
            
            console.log('JavaScript is working!');
        </script>
    </body>
    </html>
    """
    
    webview.load_html(html)
    webview.show()


def test_public_url():
    """Test 2: Load a public URL."""
    print("\n=== Test 2: Loading public URL ===")
    from auroraview import WebView

    webview = WebView(
        title="Test 2: Public URL",
        width=800,
        height=600,
    )
    
    # Load a reliable public URL
    webview.load_url("https://www.example.com")
    webview.show()


def test_localhost_with_check():
    """Test 3: Check if localhost:3000 is accessible before loading."""
    print("\n=== Test 3: Checking localhost:3000 ===")
    
    import socket
    
    def is_port_open(host, port):
        """Check if a port is open."""
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(1)
            result = sock.connect_ex((host, port))
            sock.close()
            return result == 0
        except Exception as e:
            print(f"Error checking port: {e}")
            return False
    
    if is_port_open("localhost", 3000):
        print("✅ localhost:3000 is accessible")
        from auroraview import WebView
        
        webview = WebView(
            title="Test 3: Localhost",
            width=800,
            height=600,
        )
        webview.load_url("http://localhost:3000")
        webview.show()
    else:
        print("❌ localhost:3000 is NOT accessible")
        print("Please start your local server first:")
        print("  - For React: npm start")
        print("  - For Vue: npm run dev")
        print("  - For Python: python -m http.server 3000")
        print("\nShowing fallback HTML instead...")
        
        from auroraview import WebView
        
        webview = WebView(
            title="Test 3: Server Not Running",
            width=800,
            height=600,
        )
        
        html = """
        <!DOCTYPE html>
        <html>
        <head>
            <title>Server Not Running</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    margin: 0;
                    background: #f5f5f5;
                }
                .container {
                    text-align: center;
                    background: white;
                    padding: 40px;
                    border-radius: 10px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                }
                h1 {
                    color: #e74c3c;
                    margin-bottom: 20px;
                }
                p {
                    color: #555;
                    line-height: 1.6;
                }
                code {
                    background: #f0f0f0;
                    padding: 2px 6px;
                    border-radius: 3px;
                    font-family: monospace;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>❌ Server Not Running</h1>
                <p>localhost:3000 is not accessible.</p>
                <p>Please start your local server:</p>
                <ul style="text-align: left; display: inline-block;">
                    <li>React: <code>npm start</code></li>
                    <li>Vue: <code>npm run dev</code></li>
                    <li>Python: <code>python -m http.server 3000</code></li>
                </ul>
            </div>
        </body>
        </html>
        """
        webview.load_html(html)
        webview.show()


def test_with_error_handling():
    """Test 4: WebView with proper error handling."""
    print("\n=== Test 4: WebView with error handling ===")
    from auroraview import WebView
    
    try:
        webview = WebView(
            title="Test 4: Error Handling",
            width=800,
            height=600,
        )
        
        # Try to load URL with timeout
        print("Attempting to load http://localhost:3000...")
        webview.load_url("http://localhost:3000")
        
        # Give it a moment to load
        time.sleep(2)
        
        webview.show()
        
    except Exception as e:
        print(f"❌ Error: {e}")
        print("Showing error page instead...")
        
        webview = WebView(
            title="Error",
            width=800,
            height=600,
        )
        
        html = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Error</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    padding: 40px;
                    background: #f5f5f5;
                }}
                .error {{
                    background: white;
                    padding: 20px;
                    border-left: 4px solid #e74c3c;
                    border-radius: 4px;
                }}
            </style>
        </head>
        <body>
            <div class="error">
                <h1>Error Loading WebView</h1>
                <p><strong>Error:</strong> {e}</p>
            </div>
        </body>
        </html>
        """
        webview.load_html(html)
        webview.show()


def main():
    """Run all tests."""
    print("AuroraView WebView Diagnostic Tests")
    print("=" * 50)
    
    if len(sys.argv) > 1:
        test_num = sys.argv[1]
        if test_num == "1":
            test_basic_html()
        elif test_num == "2":
            test_public_url()
        elif test_num == "3":
            test_localhost_with_check()
        elif test_num == "4":
            test_with_error_handling()
        else:
            print(f"Unknown test: {test_num}")
            print("Usage: python test_webview.py [1|2|3|4]")
    else:
        print("\nAvailable tests:")
        print("  1 - Load HTML content directly (recommended)")
        print("  2 - Load public URL (example.com)")
        print("  3 - Check localhost:3000 and load if available")
        print("  4 - Load with error handling")
        print("\nUsage: python test_webview.py [1|2|3|4]")
        print("\nRunning Test 1 by default...\n")
        test_basic_html()


if __name__ == "__main__":
    main()

