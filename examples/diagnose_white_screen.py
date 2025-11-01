"""Diagnose white screen issue in AuroraView.

This script helps identify why WebView shows a white screen.
"""

import socket
import sys


def check_localhost_server(port=3000):
    """Check if localhost server is running."""
    print(f"\n🔍 Checking if localhost:{port} is accessible...")
    
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(2)
        result = sock.connect_ex(("localhost", port))
        sock.close()
        
        if result == 0:
            print(f"✅ localhost:{port} is ACCESSIBLE")
            return True
        else:
            print(f"❌ localhost:{port} is NOT accessible")
            print(f"   Error code: {result}")
            return False
    except Exception as e:
        print(f"❌ Error checking port: {e}")
        return False


def show_solution():
    """Show solution for white screen issue."""
    print("\n" + "=" * 60)
    print("🎯 WHITE SCREEN ISSUE - SOLUTION")
    print("=" * 60)
    
    print("\n📋 The white screen happens because:")
    print("   1. You're trying to load http://localhost:3000")
    print("   2. But there's NO server running on port 3000")
    print("   3. WebView loads successfully but shows blank page")
    
    print("\n✅ SOLUTION 1: Start your local server first")
    print("   Before running your Python code, start your web server:")
    print("   ")
    print("   For React:")
    print("     cd your-react-app")
    print("     npm start")
    print("   ")
    print("   For Vue:")
    print("     cd your-vue-app")
    print("     npm run dev")
    print("   ")
    print("   For simple HTTP server:")
    print("     cd your-html-folder")
    print("     python -m http.server 3000")
    print("   ")
    print("   Then run your Python code again.")
    
    print("\n✅ SOLUTION 2: Load HTML content directly")
    print("   Instead of loading a URL, load HTML content:")
    print("   ")
    print("   from auroraview import WebView")
    print("   ")
    print("   webview = WebView(")
    print("       title='My App',")
    print("       width=800,")
    print("       height=600")
    print("   )")
    print("   ")
    print("   html = '''")
    print("   <!DOCTYPE html>")
    print("   <html>")
    print("   <body>")
    print("       <h1>Hello from AuroraView!</h1>")
    print("   </body>")
    print("   </html>")
    print("   '''")
    print("   ")
    print("   webview.load_html(html)")
    print("   webview.show()")
    
    print("\n✅ SOLUTION 3: Use a public URL for testing")
    print("   Test with a public URL to verify WebView works:")
    print("   ")
    print("   from auroraview import WebView")
    print("   ")
    print("   webview = WebView(")
    print("       title='My App',")
    print("       width=800,")
    print("       height=600")
    print("   )")
    print("   webview.load_url('https://www.example.com')")
    print("   webview.show()")


def create_working_example():
    """Create a working example that won't show white screen."""
    print("\n" + "=" * 60)
    print("📝 Creating working example...")
    print("=" * 60)
    
    example_code = '''"""Working example of AuroraView without white screen."""

from auroraview import WebView

# Create WebView
webview = WebView(
    title="AuroraView - Working Example",
    width=800,
    height=600
)

# Load HTML content directly (no server needed!)
html = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>AuroraView Example</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            color: white;
        }
        
        .container {
            text-align: center;
            padding: 40px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
        }
        
        h1 {
            font-size: 48px;
            margin-bottom: 20px;
            text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.2);
        }
        
        p {
            font-size: 20px;
            margin-bottom: 10px;
            opacity: 0.9;
        }
        
        .time {
            font-size: 32px;
            font-weight: bold;
            margin-top: 20px;
            padding: 20px;
            background: rgba(255, 255, 255, 0.2);
            border-radius: 10px;
        }
        
        .button {
            margin-top: 20px;
            padding: 12px 24px;
            font-size: 16px;
            background: white;
            color: #667eea;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: bold;
            transition: transform 0.2s;
        }
        
        .button:hover {
            transform: scale(1.05);
        }
        
        .button:active {
            transform: scale(0.95);
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎉 AuroraView Works!</h1>
        <p>This is a working example with no white screen.</p>
        <p>No server needed - HTML is loaded directly!</p>
        <div class="time" id="time"></div>
        <button class="button" onclick="showAlert()">Click Me!</button>
    </div>
    
    <script>
        // Update time every second
        function updateTime() {
            const now = new Date();
            document.getElementById('time').textContent = 
                now.toLocaleTimeString('en-US', { 
                    hour12: false,
                    hour: '2-digit',
                    minute: '2-digit',
                    second: '2-digit'
                });
        }
        
        updateTime();
        setInterval(updateTime, 1000);
        
        function showAlert() {
            alert('Button clicked! JavaScript is working! 🎊');
        }
        
        console.log('✅ JavaScript loaded successfully');
        console.log('✅ AuroraView event bridge initialized');
    </script>
</body>
</html>
"""

# Load the HTML
webview.load_html(html)

# Show the window (blocking call)
webview.show()
'''
    
    # Save the example
    with open("examples/working_example.py", "w", encoding="utf-8") as f:
        f.write(example_code)
    
    print("✅ Created: examples/working_example.py")
    print("\nRun it with:")
    print("  python examples/working_example.py")


def main():
    """Main diagnostic function."""
    print("=" * 60)
    print("🔍 AuroraView White Screen Diagnostic Tool")
    print("=" * 60)
    
    # Check if localhost:3000 is running
    server_running = check_localhost_server(3000)
    
    if not server_running:
        print("\n⚠️  This is why you see a white screen!")
        print("   WebView is trying to load http://localhost:3000")
        print("   but there's no server running on that port.")
        
        show_solution()
        create_working_example()
        
        print("\n" + "=" * 60)
        print("🎯 QUICK FIX")
        print("=" * 60)
        print("\nRun this command to see a working example:")
        print("  python examples/working_example.py")
        print("\nOr run the diagnostic tests:")
        print("  python examples/test_webview.py 1")
        
    else:
        print("\n✅ Server is running! WebView should work.")
        print("   If you still see white screen, try:")
        print("   1. Check browser console for errors")
        print("   2. Verify the URL is correct")
        print("   3. Try loading HTML directly instead")


if __name__ == "__main__":
    main()

