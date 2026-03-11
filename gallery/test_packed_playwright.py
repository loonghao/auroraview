"""CDP smoke test for packed Gallery, including Promise rejection button flow."""

import argparse
import os
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


def wait_for_cdp(endpoint, timeout=30):
    """Wait for CDP endpoint to become available."""
    parsed = urllib.parse.urlparse(endpoint)
    host = parsed.hostname or "127.0.0.1"
    port = parsed.port or 9222
    version_url = "http://{}:{}/json/version".format(host, port)

    start = time.time()
    while time.time() - start < timeout:
        try:
            req = urllib.request.urlopen(version_url, timeout=2)
            req.close()
            return True
        except (urllib.error.URLError, OSError):
            time.sleep(0.5)
    return False


def connect_first_page(playwright, endpoint):
    browser = playwright.chromium.connect_over_cdp(endpoint)
    contexts = browser.contexts
    if not contexts:
        raise RuntimeError("No browser context found via CDP")

    context = contexts[0]
    if not context.pages:
        raise RuntimeError("No page found in first browser context")

    page = None
    for p in context.pages:
        if "auroraview.localhost" in p.url:
            page = p
            break
    if page is None:
        page = context.pages[0]

    return browser, page


def run_promise_rejection_flow(page):
    page_errors = []
    console_messages = []

    def on_page_error(err):
        page_errors.append(str(err))

    def on_console(msg):
        if msg.type in ("error", "warn"):
            console_messages.append("[{}] {}".format(msg.type, msg.text))

    page.on("pageerror", on_page_error)
    page.on("console", on_console)

    page.bring_to_front()
    time.sleep(0.5)

    before_path = os.path.join(os.path.dirname(__file__), "gallery_promise_before.png")
    page.screenshot(path=before_path)
    print("Before screenshot:", before_path)

    page.locator('button[title="Telemetry"]').first.click(timeout=8000)
    page.get_by_role("button", name="Test Promise Rejection").click(timeout=8000)

    time.sleep(2.0)

    after_path = os.path.join(os.path.dirname(__file__), "gallery_promise_after.png")
    page.screenshot(path=after_path)
    print("After screenshot:", after_path)

    print("\n== Page Errors ==")
    if page_errors:
        for i, err in enumerate(page_errors, 1):
            print("{}. {}".format(i, err))
    else:
        print("<none>")

    print("\n== Console Warn/Error ==")
    if console_messages:
        for i, msg in enumerate(console_messages, 1):
            print("{}. {}".format(i, msg))
    else:
        print("<none>")


def main():
    parser = argparse.ArgumentParser(description="Test packed Gallery via CDP")
    parser.add_argument(
        "--cdp-endpoint",
        default="http://127.0.0.1:9222",
        help="CDP endpoint, default: http://127.0.0.1:9222",
    )
    parser.add_argument(
        "--start-packed",
        action="store_true",
        help="Start packed exe before connecting",
    )
    parser.add_argument(
        "--exe-path",
        default=os.path.join(
            os.path.dirname(__file__), "pack-output", "auroraview-gallery-debug.exe"
        ),
        help="Path to packed executable (used with --start-packed)",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=30,
        help="CDP wait timeout in seconds",
    )
    args = parser.parse_args()

    proc = None

    if args.start_packed:
        if not os.path.exists(args.exe_path):
            print("ERROR: executable not found:", args.exe_path)
            return 1

        print("Starting packed app:", args.exe_path)
        proc = subprocess.Popen(
            [args.exe_path],
            cwd=os.path.dirname(args.exe_path),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        print("Started PID:", proc.pid)

    if not wait_for_cdp(args.cdp_endpoint, timeout=args.timeout):
        print("ERROR: CDP endpoint not ready:", args.cdp_endpoint)
        if proc and proc.poll() is not None:
            print("Packed process exited with:", proc.returncode)
        if proc:
            proc.terminate()
        return 2

    print("CDP ready:", args.cdp_endpoint)

    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("ERROR: playwright not installed in current env")
        return 3

    try:
        with sync_playwright() as p:
            browser, page = connect_first_page(p, args.cdp_endpoint)
            print("Connected page URL:", page.url)
            run_promise_rejection_flow(page)
            browser.close()
        print("\n✓ Promise rejection flow test completed")
        return 0
    except Exception as exc:
        print("ERROR during Playwright CDP test:", repr(exc))
        return 4
    finally:
        if proc:
            print("Terminating packed process...")
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    sys.exit(main())
