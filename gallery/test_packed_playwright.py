"""Test packed gallery using Playwright to connect via CDP."""

import os
import subprocess
import sys
import time
import urllib.error
import urllib.request


def wait_for_cdp(port: int, timeout: int = 30) -> bool:
    """Wait for CDP to become available."""
    start = time.time()
    while time.time() - start < timeout:
        try:
            req = urllib.request.urlopen(f"http://127.0.0.1:{port}/json/version", timeout=2)
            req.close()
            return True
        except (urllib.error.URLError, OSError):
            time.sleep(0.5)
    return False


def main():
    # Start the packed exe (CDP is enabled via config: remote_debugging_port = 9222)
    exe_path = os.path.join(os.path.dirname(__file__), "pack-output", "pack-output.exe")

    if not os.path.exists(exe_path):
        print(f"ERROR: {exe_path} not found")
        return 1

    print("Starting packed gallery (CDP enabled via config)...")

    # Start process without capturing output to avoid blocking
    proc = subprocess.Popen(
        [exe_path],
        cwd=os.path.dirname(exe_path),
        # Don't capture stdout/stderr to avoid potential blocking
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    print(f"Process started: PID {proc.pid}")
    print("Waiting for CDP to become available...")

    if not wait_for_cdp(9222, timeout=30):
        print("ERROR: CDP did not become available within 30 seconds")
        if proc.poll() is not None:
            print(f"Process exited with code {proc.returncode}")
        proc.terminate()
        return 1

    print("CDP is available! Connecting via Playwright...")

    try:
        from playwright.sync_api import sync_playwright

        with sync_playwright() as p:
            try:
                # Use 127.0.0.1 instead of localhost to avoid IPv6 issues
                browser = p.chromium.connect_over_cdp("http://127.0.0.1:9222")
                print(f"Connected! Browser version: {browser.version}")
                print(f"Contexts: {len(browser.contexts)}")

                if browser.contexts:
                    context = browser.contexts[0]
                    print(f"Pages in context: {len(context.pages)}")

                    if context.pages:
                        page = context.pages[0]
                        print(f"Page URL: {page.url}")
                        print(f"Page title: {page.title()}")

                        # Take screenshot
                        screenshot_path = os.path.join(
                            os.path.dirname(__file__), "gallery_screenshot.png"
                        )
                        page.screenshot(path=screenshot_path)
                        print(f"Screenshot saved: {screenshot_path}")

                        # Show page content sample
                        content = page.content()
                        print(f"\nPage content ({len(content)} bytes):")
                        print(content[:1000] + "..." if len(content) > 1000 else content)

                browser.close()
                print("\n✓ Test completed successfully!")
                return 0

            except Exception as e:
                print(f"Failed to connect via CDP: {e}")
                import traceback

                traceback.print_exc()
                return 1

    except ImportError:
        print(
            "Playwright not installed. Run: pip install playwright && playwright install chromium"
        )
        return 1

    finally:
        print("\nTerminating process...")
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        print("Done.")


if __name__ == "__main__":
    sys.exit(main())
