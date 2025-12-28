"""
Midscene.js AI-Powered Testing Demo

This example demonstrates how to use Midscene.js integration for
AI-powered UI testing with natural language instructions.

Midscene.js is an AI-driven UI automation SDK by ByteDance that enables:
- Natural language UI interactions
- AI-powered data extraction
- Visual-based element location
- Natural language assertions

For more information: https://midscenejs.com/

Requirements:
- playwright: pip install playwright && playwright install chromium
- OpenAI API key or compatible model API

Usage:
    # Set API key
    export OPENAI_API_KEY=your-api-key

    # Run demo
    python examples/midscene_demo.py
"""

import asyncio
import os
import sys

# Add project root to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


async def demo_basic_actions():
    """Demonstrate basic AI-powered actions."""
    print("\n=== Basic AI Actions Demo ===\n")

    try:
        from playwright.async_api import async_playwright
    except ImportError:
        print("Playwright not installed. Install with: pip install playwright")
        print("Then run: playwright install chromium")
        return

    from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

    # Configure Midscene (uses OpenAI by default)
    config = MidsceneConfig(
        model_name="gpt-4o",
        debug=True,
    )

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=False)
        page = await browser.new_page()

        # Navigate to a test page
        await page.goto("https://www.bing.com")

        # Create Midscene agent
        async with MidsceneAgent(page, config) as agent:
            print("1. Executing AI action: type in search box")
            result = await agent.ai_act('type "AuroraView WebView" in the search box')
            print(f"   Result: {result}")

            print("\n2. Executing AI action: press Enter")
            result = await agent.ai_act("press Enter")
            print(f"   Result: {result}")

            # Wait for results
            await page.wait_for_timeout(2000)

            print("\n3. AI assertion: check for search results")
            try:
                await agent.ai_assert("there are search results on the page")
                print("   Assertion passed!")
            except AssertionError as e:
                print(f"   Assertion failed: {e}")

            print("\n4. AI query: extract page title")
            title = await agent.ai_query("string, the page title")
            print(f"   Page title: {title}")

        await browser.close()


async def demo_form_interaction():
    """Demonstrate AI-powered form interaction."""
    print("\n=== Form Interaction Demo ===\n")

    try:
        from playwright.async_api import async_playwright
    except ImportError:
        print("Playwright not installed.")
        return

    from auroraview.testing.midscene import MidsceneAgent

    # Create a simple test form
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Test Form</title>
        <style>
            body { font-family: Arial, sans-serif; padding: 20px; }
            .form-group { margin: 10px 0; }
            label { display: block; margin-bottom: 5px; }
            input, select { padding: 8px; width: 200px; }
            button { padding: 10px 20px; background: #007bff; color: white; border: none; cursor: pointer; }
            button:hover { background: #0056b3; }
            .result { margin-top: 20px; padding: 10px; background: #e9ecef; display: none; }
        </style>
    </head>
    <body>
        <h1>Contact Form</h1>
        <form id="contactForm">
            <div class="form-group">
                <label for="name">Name:</label>
                <input type="text" id="name" name="name" placeholder="Enter your name">
            </div>
            <div class="form-group">
                <label for="email">Email:</label>
                <input type="email" id="email" name="email" placeholder="Enter your email">
            </div>
            <div class="form-group">
                <label for="subject">Subject:</label>
                <select id="subject" name="subject">
                    <option value="">Select a subject</option>
                    <option value="general">General Inquiry</option>
                    <option value="support">Technical Support</option>
                    <option value="feedback">Feedback</option>
                </select>
            </div>
            <div class="form-group">
                <label for="message">Message:</label>
                <textarea id="message" name="message" rows="4" style="width: 200px;" placeholder="Enter your message"></textarea>
            </div>
            <button type="submit">Submit</button>
        </form>
        <div class="result" id="result">
            <h3>Form Submitted!</h3>
            <p>Thank you for your message.</p>
        </div>
        <script>
            document.getElementById('contactForm').addEventListener('submit', function(e) {
                e.preventDefault();
                document.getElementById('result').style.display = 'block';
            });
        </script>
    </body>
    </html>
    """

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=False)
        page = await browser.new_page()

        # Load the test form
        await page.set_content(html)

        async with MidsceneAgent(page) as agent:
            print("1. Fill in the name field")
            await agent.ai_act('type "John Doe" in the name field')

            print("2. Fill in the email field")
            await agent.ai_act('type "john@example.com" in the email field')

            print("3. Select a subject")
            await page.select_option("#subject", "support")

            print("4. Fill in the message")
            await agent.ai_act('type "Hello, I need help with AuroraView" in the message field')

            print("5. Click submit button")
            await agent.ai_act("click the submit button")

            # Wait for result
            await page.wait_for_timeout(500)

            print("6. Verify submission")
            await agent.ai_assert("the form was submitted successfully")
            print("   Form submitted successfully!")

        await browser.close()


async def demo_data_extraction():
    """Demonstrate AI-powered data extraction."""
    print("\n=== Data Extraction Demo ===\n")

    try:
        from playwright.async_api import async_playwright
    except ImportError:
        print("Playwright not installed.")
        return

    from auroraview.testing.midscene import MidsceneAgent

    # Create a test page with data
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Product List</title>
        <style>
            body { font-family: Arial, sans-serif; padding: 20px; }
            .product { border: 1px solid #ddd; padding: 15px; margin: 10px 0; border-radius: 5px; }
            .product h3 { margin: 0 0 10px 0; }
            .price { color: #28a745; font-weight: bold; font-size: 1.2em; }
            .stock { color: #6c757d; }
        </style>
    </head>
    <body>
        <h1>Featured Products</h1>
        <div class="product">
            <h3>AuroraView Pro License</h3>
            <p class="price">$99.00</p>
            <p class="stock">In Stock</p>
        </div>
        <div class="product">
            <h3>AuroraView Enterprise</h3>
            <p class="price">$299.00</p>
            <p class="stock">In Stock</p>
        </div>
        <div class="product">
            <h3>AuroraView Team Bundle</h3>
            <p class="price">$499.00</p>
            <p class="stock">Limited Availability</p>
        </div>
    </body>
    </html>
    """

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()

        await page.set_content(html)

        async with MidsceneAgent(page) as agent:
            print("1. Extract page title")
            title = await page.title()
            print(f"   Title: {title}")

            print("\n2. Extract product names (AI query)")
            products = await agent.ai_query("string[], list of product names on the page")
            print(f"   Products: {products}")

            print("\n3. Verify product count")
            await agent.ai_assert("there are at least 3 products on the page")
            print("   Verified: at least 3 products found")

        await browser.close()


async def demo_counter_app():
    """Demonstrate AI testing with a counter application."""
    print("\n=== Counter App AI Testing Demo ===\n")

    try:
        from playwright.async_api import async_playwright
    except ImportError:
        print("Playwright not installed.")
        return

    from auroraview.testing.midscene import MidsceneAgent

    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>AuroraView + Midscene Demo</title>
        <style>
            body { font-family: Arial, sans-serif; padding: 20px; text-align: center; }
            button { padding: 15px 30px; font-size: 18px; margin: 10px; cursor: pointer; }
            #counter { font-size: 48px; margin: 20px; }
        </style>
    </head>
    <body>
        <h1>Counter App</h1>
        <div id="counter">0</div>
        <button id="increment">+</button>
        <button id="decrement">-</button>
        <button id="reset">Reset</button>
        <script>
            let count = 0;
            const counter = document.getElementById('counter');
            document.getElementById('increment').onclick = () => counter.textContent = ++count;
            document.getElementById('decrement').onclick = () => counter.textContent = --count;
            document.getElementById('reset').onclick = () => counter.textContent = count = 0;
        </script>
    </body>
    </html>
    """

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()

        await page.set_content(html)

        async with MidsceneAgent(page) as agent:
            print("1. Verify initial state")
            await agent.ai_assert("the counter shows 0")
            print("   Counter is at 0")

            print("\n2. Click increment button 3 times")
            await agent.ai_act('click the "+" button')
            await agent.ai_act('click the "+" button')
            await agent.ai_act('click the "+" button')

            print("\n3. Verify counter increased")
            await agent.ai_assert("the counter shows 3")
            print("   Counter is at 3")

            print("\n4. Click decrement button")
            await agent.ai_act('click the "-" button')

            print("\n5. Verify counter decreased")
            await agent.ai_assert("the counter shows 2")
            print("   Counter is at 2")

            print("\n6. Click reset button")
            await agent.ai_act('click the "Reset" button')

            print("\n7. Verify counter reset")
            await agent.ai_assert("the counter shows 0")
            print("   Counter reset to 0")

            print("\n8. Extract counter value")
            value = await agent.ai_query("number, the current counter value")
            print(f"   Extracted value: {value}")

        await browser.close()


async def demo_gallery_testing():
    """Demonstrate AI testing with AuroraView Gallery."""
    print("\n=== Gallery AI Testing Demo ===\n")

    try:
        from playwright.async_api import async_playwright
    except ImportError:
        print("Playwright not installed.")
        return

    from pathlib import Path

    from auroraview.testing.midscene import MidsceneAgent

    # Check if Gallery is built
    project_root = Path(__file__).parent.parent
    gallery_dist = project_root / "gallery" / "dist" / "index.html"

    if not gallery_dist.exists():
        print("Gallery not built. Run 'just gallery-build' first.")
        print("Skipping Gallery demo.")
        return

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page(viewport={"width": 1200, "height": 800})

        # Inject mock AuroraView bridge
        await page.add_init_script("""
            window._apiCalls = [];
            window._mockResponses = {
                'api.get_samples': [
                    { id: 'simple_decorator', title: 'Simple Decorator', category: 'getting_started',
                      description: 'Basic WebView example', icon: 'wand-2', tags: ['beginner'] },
                    { id: 'window_events', title: 'Window Events', category: 'window_management',
                      description: 'Handle window events', icon: 'layout', tags: ['events'] }
                ],
                'api.get_categories': {
                    'getting_started': { title: 'Getting Started', icon: 'rocket' },
                    'window_management': { title: 'Window Management', icon: 'layout' }
                },
                'api.get_source': '# Sample code\\nfrom auroraview import WebView'
            };
            window.auroraview = {
                call: function(method, params) {
                    window._apiCalls.push({ method, params });
                    return Promise.resolve(window._mockResponses[method]);
                },
                on: function() { return () => {}; },
                trigger: function() {},
                api: new Proxy({}, {
                    get: (t, p) => (...args) => window.auroraview.call('api.' + p, args)
                })
            };
            window.dispatchEvent(new CustomEvent('auroraviewready'));
        """)

        await page.goto(f"file://{gallery_dist}")
        await page.wait_for_timeout(1500)

        async with MidsceneAgent(page) as agent:
            print("1. Verify Gallery loaded")
            await agent.ai_assert("the page has loaded and shows content")
            print("   Gallery loaded successfully")

            print("\n2. Check for navigation")
            await agent.ai_assert("there is a sidebar or navigation area")
            print("   Navigation found")

            print("\n3. Look for sample items")
            await agent.ai_assert("there are sample items or cards visible")
            print("   Sample items found")

            print("\n4. Extract page structure")
            structure = await agent.ai_query("string, describe the main layout areas of the page")
            print(f"   Layout: {structure}")

        await browser.close()
        print("\nGallery AI testing completed!")


def main():
    """Run all demos."""
    print("=" * 60)
    print("Midscene.js AI-Powered Testing Demo for AuroraView")
    print("=" * 60)

    # Check for API key
    if not os.environ.get("OPENAI_API_KEY") and not os.environ.get("MIDSCENE_MODEL_API_KEY"):
        print("\nNote: No API key found. Some features may not work.")
        print("Set OPENAI_API_KEY or MIDSCENE_MODEL_API_KEY environment variable.")

    # Run headless demos
    print("\nRunning headless demos...")
    asyncio.run(demo_data_extraction())
    asyncio.run(demo_counter_app())
    asyncio.run(demo_gallery_testing())

    print("\n" + "=" * 60)
    print("Demo completed!")
    print("=" * 60)
    print("\nTo run interactive demos (requires display):")
    print("  - demo_basic_actions(): Search on Bing")
    print("  - demo_form_interaction(): Fill and submit a form")
    print("\nFor more information about Midscene.js:")
    print("  https://midscenejs.com/")


if __name__ == "__main__":
    main()
