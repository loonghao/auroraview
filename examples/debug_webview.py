"""Debug WebView loading issues with detailed logging."""

import sys
import time


def test_with_url_and_fallback():
    """Test loading URL with fallback to HTML if it fails."""
    print("Testing WebView with URL and fallback...")
    
    from auroraview import WebView
    
    # Create WebView
    webview = WebView(
        title="Debug WebView",
        width=1000,
        height=800
    )
    
    # First, try to load the URL
    print("Attempting to load http://localhost:3000...")
    webview.load_url("http://localhost:3000")
    
    # Wait a bit to see if it loads
    print("Waiting 3 seconds for page to load...")
    time.sleep(3)
    
    # If you still see white screen, it might be:
    # 1. CORS issue
    # 2. The page is loading but has errors
    # 3. The page requires specific headers
    
    print("Showing WebView...")
    print("\nIf you see white screen, press Ctrl+C and we'll try HTML instead")
    
    try:
        webview.show()
    except KeyboardInterrupt:
        print("\n\nInterrupted! Loading HTML fallback instead...")
        test_with_html()


def test_with_html():
    """Test with HTML content that definitely works."""
    print("\nLoading HTML content directly...")
    
    from auroraview import WebView
    
    webview = WebView(
        title="AuroraView - HTML Test",
        width=1000,
        height=800
    )
    
    html = """
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>AuroraView Debug</title>
        <style>
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                min-height: 100vh;
                padding: 40px;
            }
            
            .container {
                max-width: 800px;
                margin: 0 auto;
                background: white;
                border-radius: 20px;
                padding: 40px;
                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            }
            
            h1 {
                color: #667eea;
                margin-bottom: 20px;
                font-size: 36px;
            }
            
            .status {
                padding: 20px;
                background: #d4edda;
                border-left: 4px solid #28a745;
                border-radius: 4px;
                margin-bottom: 20px;
            }
            
            .status h2 {
                color: #155724;
                margin-bottom: 10px;
            }
            
            .status p {
                color: #155724;
                line-height: 1.6;
            }
            
            .info {
                background: #f8f9fa;
                padding: 20px;
                border-radius: 8px;
                margin-bottom: 20px;
            }
            
            .info h3 {
                color: #333;
                margin-bottom: 10px;
            }
            
            .info ul {
                margin-left: 20px;
                color: #666;
            }
            
            .info li {
                margin-bottom: 8px;
                line-height: 1.5;
            }
            
            .test-section {
                margin-top: 30px;
                padding: 20px;
                background: #e7f3ff;
                border-radius: 8px;
            }
            
            .test-section h3 {
                color: #004085;
                margin-bottom: 15px;
            }
            
            button {
                padding: 12px 24px;
                font-size: 16px;
                background: #667eea;
                color: white;
                border: none;
                border-radius: 8px;
                cursor: pointer;
                margin-right: 10px;
                margin-bottom: 10px;
                transition: all 0.3s;
            }
            
            button:hover {
                background: #5568d3;
                transform: translateY(-2px);
                box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
            }
            
            button:active {
                transform: translateY(0);
            }
            
            #output {
                margin-top: 20px;
                padding: 15px;
                background: #f8f9fa;
                border-radius: 4px;
                font-family: 'Courier New', monospace;
                font-size: 14px;
                min-height: 100px;
                max-height: 300px;
                overflow-y: auto;
            }
            
            .log-entry {
                padding: 5px;
                margin-bottom: 5px;
                border-left: 3px solid #667eea;
                padding-left: 10px;
            }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🎉 AuroraView is Working!</h1>
            
            <div class="status">
                <h2>✅ Success!</h2>
                <p>If you can see this page, AuroraView is working correctly.</p>
                <p>The white screen issue was likely due to the URL not loading properly.</p>
            </div>
            
            <div class="info">
                <h3>Why did you see a white screen?</h3>
                <ul>
                    <li><strong>URL Loading Issue:</strong> The URL might not be loading correctly</li>
                    <li><strong>CORS Policy:</strong> Browser security might block localhost requests</li>
                    <li><strong>Server Response:</strong> The server might not be returning valid HTML</li>
                    <li><strong>JavaScript Errors:</strong> The page might have JavaScript errors</li>
                </ul>
            </div>
            
            <div class="info">
                <h3>✅ Recommended Solutions:</h3>
                <ul>
                    <li><strong>Use load_html():</strong> Load HTML content directly instead of URLs</li>
                    <li><strong>Check Server:</strong> Verify your localhost:3000 returns valid HTML</li>
                    <li><strong>Use DevTools:</strong> Open browser DevTools to see console errors</li>
                    <li><strong>Test with Public URL:</strong> Try loading https://www.example.com first</li>
                </ul>
            </div>
            
            <div class="test-section">
                <h3>🧪 Interactive Tests</h3>
                <button onclick="testConsole()">Test Console</button>
                <button onclick="testAlert()">Test Alert</button>
                <button onclick="testFetch()">Test Fetch</button>
                <button onclick="clearOutput()">Clear Output</button>
                
                <div id="output"></div>
            </div>
        </div>
        
        <script>
            const output = document.getElementById('output');
            
            function log(message, type = 'info') {
                const entry = document.createElement('div');
                entry.className = 'log-entry';
                entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
                output.appendChild(entry);
                output.scrollTop = output.scrollHeight;
                console.log(message);
            }
            
            function testConsole() {
                log('✅ Console.log is working!');
                console.log('This is a console.log test');
                console.warn('This is a console.warn test');
                console.error('This is a console.error test');
            }
            
            function testAlert() {
                alert('Alert is working! 🎊');
                log('✅ Alert displayed successfully');
            }
            
            async function testFetch() {
                log('🔄 Testing fetch to localhost:3000...');
                try {
                    const response = await fetch('http://localhost:3000');
                    const text = await response.text();
                    log(`✅ Fetch successful! Status: ${response.status}`);
                    log(`Response length: ${text.length} characters`);
                } catch (error) {
                    log(`❌ Fetch failed: ${error.message}`);
                }
            }
            
            function clearOutput() {
                output.innerHTML = '';
            }
            
            // Initial log
            log('✅ JavaScript loaded successfully');
            log('✅ AuroraView event bridge initialized');
            log('✅ Page is fully interactive');
        </script>
    </body>
    </html>
    """
    
    webview.load_html(html)
    print("HTML loaded. Showing WebView...")
    webview.show()


def test_url_with_proper_api():
    """Test using the proper WebView API."""
    print("\nTesting with proper WebView API...")
    
    from auroraview import WebView
    
    # Method 1: Create and then load
    webview = WebView(
        title="Method 1: Load After Create",
        width=1000,
        height=800
    )
    
    print("Loading URL...")
    webview.load_url("http://localhost:3000")
    
    print("Showing WebView...")
    webview.show()


def main():
    """Main function."""
    print("=" * 60)
    print("AuroraView Debug Tool")
    print("=" * 60)
    print("\nChoose a test:")
    print("  1 - Test with HTML content (recommended)")
    print("  2 - Test with localhost:3000 URL")
    print("  3 - Test with URL and fallback")
    print()
    
    if len(sys.argv) > 1:
        choice = sys.argv[1]
    else:
        choice = input("Enter choice (1-3) [1]: ").strip() or "1"
    
    if choice == "1":
        test_with_html()
    elif choice == "2":
        test_url_with_proper_api()
    elif choice == "3":
        test_with_url_and_fallback()
    else:
        print(f"Invalid choice: {choice}")
        print("Running default test (HTML)...")
        test_with_html()


if __name__ == "__main__":
    main()

