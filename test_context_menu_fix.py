"""Test script to verify context menu is disabled with JavaScript.

This tests the new JavaScript-based context menu disabling.

Run after rebuilding with the fix.

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

import sys
import time


def test_context_menu_disabled():
    """Test that context menu is disabled via JavaScript."""
    print("\n" + "=" * 80)
    print("Testing Context Menu Disabling (JavaScript Method)")
    print("=" * 80)
    
    try:
        from auroraview import WebView
        
        print("\n[TEST] Creating WebView with context_menu=False...")
        print("This should inject JavaScript to disable right-click menu.")
        
        webview = WebView.create(
            title="Context Menu Test",
            width=800,
            height=600,
            html="""
            <html>
            <head>
                <style>
                    body {
                        font-family: Arial, sans-serif;
                        padding: 40px;
                        background: #2b2b2b;
                        color: #fff;
                    }
                    .test-area {
                        background: #3c3c3c;
                        padding: 40px;
                        border-radius: 8px;
                        text-align: center;
                    }
                    h1 { color: #4CAF50; }
                    .instruction {
                        margin-top: 20px;
                        padding: 20px;
                        background: #4a4a4a;
                        border-radius: 4px;
                    }
                </style>
            </head>
            <body>
                <div class="test-area">
                    <h1>Context Menu Test</h1>
                    <p>Right-click anywhere on this page.</p>
                    <div class="instruction">
                        <strong>Expected:</strong> No context menu should appear<br>
                        <strong>Check console:</strong> Should see "[AuroraView] Native context menu disabled"
                    </div>
                    <div id="status" style="margin-top: 20px; font-size: 18px;"></div>
                </div>
                
                <script>
                    // Monitor context menu events
                    let rightClickCount = 0;
                    document.addEventListener('contextmenu', function(e) {
                        rightClickCount++;
                        const status = document.getElementById('status');
                        status.innerHTML = `
                            <div style="color: #4CAF50;">
                                ✓ Right-click detected (${rightClickCount} times)<br>
                                ✓ Context menu should be disabled
                            </div>
                        `;
                    });
                    
                    // Check if AuroraView is available
                    window.addEventListener('auroraviewready', function() {
                        console.log('[TEST] AuroraView bridge is ready');
                    });
                </script>
            </body>
            </html>
            """,
            context_menu=False,  # This should disable the menu
            debug=True,
        )
        
        print("✅ WebView created successfully")
        print("\n" + "=" * 80)
        print("MANUAL TEST REQUIRED:")
        print("=" * 80)
        print("1. Right-click anywhere in the window")
        print("2. Verify NO context menu appears")
        print("3. Check the console for: '[AuroraView] Native context menu disabled'")
        print("4. The page should show a green checkmark when you right-click")
        print("\nPress Ctrl+C to close the window...")
        print("=" * 80)
        
        # Keep window open
        try:
            while True:
                time.sleep(0.1)
        except KeyboardInterrupt:
            print("\n\nClosing...")
        
        webview.close()
        print("✅ Test complete")
        
    except Exception as e:
        print(f"❌ ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False
    
    return True


if __name__ == "__main__":
    print("\n" + "=" * 80)
    print("Context Menu Fix Verification Test")
    print("=" * 80)
    print("\nThis test verifies that context_menu=False works correctly")
    print("using the new JavaScript-based method.")
    
    success = test_context_menu_disabled()
    
    if success:
        print("\n" + "=" * 80)
        print("✅ TEST PASSED")
        print("=" * 80)
        print("\nIf the context menu was disabled, the fix is working!")
        print("\nNext steps:")
        print("1. Close Maya if it's running")
        print("2. Run: maturin develop --release --features ext-module,win-webview2")
        print("3. Restart Maya")
        print("4. Test Maya Outliner with: maya_outliner.main(context_menu=False)")
    else:
        print("\n" + "=" * 80)
        print("❌ TEST FAILED")
        print("=" * 80)
        sys.exit(1)

