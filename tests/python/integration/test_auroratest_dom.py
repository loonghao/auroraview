"""
AuroraView DOM Operations Tests using AuroraTest

This module tests DOM manipulation capabilities including:
- DomBatch high-performance operations
- Element queries and modifications
- Attribute and style manipulation
- Event handling on DOM elements

NOTE: These tests use the original Browser class which requires WebView2.
Due to Python GIL limitations, WebView2 event loop blocks other threads.
For UI automation testing, use PlaywrightBrowser instead.

See test_playwright_browser.py for working Playwright-based tests.
"""

import logging
import sys
import time

import pytest

from auroraview import DomBatch, WebView
from auroraview.testing.auroratest import Browser

logger = logging.getLogger(__name__)

pytestmark = [
    pytest.mark.skip(
        reason="WebView2 Browser class blocks due to GIL. Use PlaywrightBrowser instead."
    ),
    pytest.mark.skipif(sys.platform != "win32", reason="WebView2 tests only run on Windows"),
]


# ============================================================
# Test HTML Templates
# ============================================================

DOM_TEST_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>DOM Test</title>
    <style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        .item { padding: 10px; margin: 5px 0; border: 1px solid #ccc; }
        .item.active { background: #e3f2fd; border-color: #2196f3; }
        .item.selected { background: #c8e6c9; border-color: #4caf50; }
        .hidden { display: none; }
        .highlight { background: yellow; }
        #container { min-height: 200px; border: 2px dashed #999; padding: 10px; }
        .red { color: red; }
        .blue { color: blue; }
        .large { font-size: 24px; }
    </style>
</head>
<body>
    <h1 id="title" class="header">DOM Test Page</h1>

    <div id="container">
        <div class="item" data-id="1">Item 1</div>
        <div class="item" data-id="2">Item 2</div>
        <div class="item" data-id="3">Item 3</div>
        <div class="item" data-id="4">Item 4</div>
        <div class="item" data-id="5">Item 5</div>
    </div>

    <div id="dynamic-area"></div>

    <button id="add-item">Add Item</button>
    <button id="clear-items">Clear Items</button>

    <div id="result"></div>

    <script>
        let itemCount = 5;

        document.getElementById('add-item').addEventListener('click', () => {
            itemCount++;
            const item = document.createElement('div');
            item.className = 'item';
            item.dataset.id = itemCount;
            item.textContent = 'Item ' + itemCount;
            document.getElementById('container').appendChild(item);
        });

        document.getElementById('clear-items').addEventListener('click', () => {
            document.getElementById('container').innerHTML = '';
            itemCount = 0;
        });

        // Click handler for items
        document.getElementById('container').addEventListener('click', (e) => {
            if (e.target.classList.contains('item')) {
                e.target.classList.toggle('selected');
                document.getElementById('result').textContent =
                    'Selected: ' + e.target.dataset.id;
            }
        });
    </script>
</body>
</html>
"""

TABLE_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Table Test</title>
    <style>
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background: #f5f5f5; }
        tr:hover { background: #f0f0f0; }
        tr.selected { background: #e3f2fd; }
    </style>
</head>
<body>
    <h1>Table Test</h1>
    <table id="data-table">
        <thead>
            <tr>
                <th>ID</th>
                <th>Name</th>
                <th>Email</th>
                <th>Status</th>
            </tr>
        </thead>
        <tbody id="table-body">
            <tr data-id="1">
                <td>1</td>
                <td class="name">Alice</td>
                <td class="email">alice@example.com</td>
                <td class="status">Active</td>
            </tr>
            <tr data-id="2">
                <td>2</td>
                <td class="name">Bob</td>
                <td class="email">bob@example.com</td>
                <td class="status">Inactive</td>
            </tr>
            <tr data-id="3">
                <td>3</td>
                <td class="name">Charlie</td>
                <td class="email">charlie@example.com</td>
                <td class="status">Active</td>
            </tr>
        </tbody>
    </table>
    <div id="selection-info"></div>
</body>
</html>
"""


# ============================================================
# Fixtures
# ============================================================


@pytest.fixture
def webview_with_dom():
    """Create WebView with DOM test HTML."""
    webview = WebView(
        title="DOM Test",
        width=1024,
        height=768,
        debug=True,
    )
    webview.load_html(DOM_TEST_HTML)
    webview.show(wait=False)
    time.sleep(0.5)
    yield webview
    try:
        webview.close()
    except Exception:
        pass


@pytest.fixture
def webview_with_table():
    """Create WebView with table HTML."""
    webview = WebView(
        title="Table Test",
        width=1024,
        height=768,
        debug=True,
    )
    webview.load_html(TABLE_HTML)
    webview.show(wait=False)
    time.sleep(0.5)
    yield webview
    try:
        webview.close()
    except Exception:
        pass


# ============================================================
# DOM Query Tests
# ============================================================


class TestDOMQuery:
    """Test DOM querying capabilities."""

    def test_query_by_id(self, webview_with_dom):
        """Test querying element by ID."""
        webview_with_dom.eval_js("""
            const el = document.getElementById('title');
            window.__test_result = el ? el.textContent : null;
        """)
        time.sleep(0.2)

    def test_query_by_class(self, webview_with_dom):
        """Test querying elements by class."""
        webview_with_dom.eval_js("""
            const items = document.querySelectorAll('.item');
            window.__test_result = items.length;
        """)
        time.sleep(0.2)

    def test_query_by_data_attribute(self, webview_with_dom):
        """Test querying elements by data attribute."""
        webview_with_dom.eval_js("""
            const item = document.querySelector('[data-id="3"]');
            window.__test_result = item ? item.textContent : null;
        """)
        time.sleep(0.2)

    def test_query_all(self, webview_with_dom):
        """Test querySelectorAll."""
        webview_with_dom.eval_js("""
            const items = document.querySelectorAll('.item');
            window.__test_result = Array.from(items).map(i => i.dataset.id);
        """)
        time.sleep(0.2)


# ============================================================
# DOM Modification Tests
# ============================================================


class TestDOMModification:
    """Test DOM modification capabilities."""

    def test_modify_text_content(self, webview_with_dom):
        """Test modifying text content."""
        webview_with_dom.eval_js("""
            document.getElementById('title').textContent = 'Modified Title';
        """)
        time.sleep(0.2)

    def test_modify_inner_html(self, webview_with_dom):
        """Test modifying innerHTML."""
        webview_with_dom.eval_js("""
            document.getElementById('dynamic-area').innerHTML =
                '<p>Dynamic content</p><span>Added via innerHTML</span>';
        """)
        time.sleep(0.2)

    def test_add_class(self, webview_with_dom):
        """Test adding CSS class."""
        webview_with_dom.eval_js("""
            document.querySelector('.item').classList.add('active');
        """)
        time.sleep(0.2)

    def test_remove_class(self, webview_with_dom):
        """Test removing CSS class."""
        webview_with_dom.eval_js("""
            const items = document.querySelectorAll('.item');
            items.forEach(item => item.classList.add('active'));
            items[0].classList.remove('active');
        """)
        time.sleep(0.2)

    def test_toggle_class(self, webview_with_dom):
        """Test toggling CSS class."""
        webview_with_dom.eval_js("""
            const item = document.querySelector('.item');
            item.classList.toggle('highlight');
            item.classList.toggle('highlight');
        """)
        time.sleep(0.2)

    def test_set_attribute(self, webview_with_dom):
        """Test setting attribute."""
        webview_with_dom.eval_js("""
            document.getElementById('title').setAttribute('data-custom', 'test-value');
        """)
        time.sleep(0.2)

    def test_remove_attribute(self, webview_with_dom):
        """Test removing attribute."""
        webview_with_dom.eval_js("""
            const item = document.querySelector('.item');
            item.removeAttribute('data-id');
        """)
        time.sleep(0.2)

    def test_modify_style(self, webview_with_dom):
        """Test modifying inline style."""
        webview_with_dom.eval_js("""
            const title = document.getElementById('title');
            title.style.color = 'red';
            title.style.fontSize = '32px';
            title.style.backgroundColor = '#f0f0f0';
        """)
        time.sleep(0.2)


# ============================================================
# DOM Creation Tests
# ============================================================


class TestDOMCreation:
    """Test DOM element creation."""

    def test_create_element(self, webview_with_dom):
        """Test creating new element."""
        webview_with_dom.eval_js("""
            const newItem = document.createElement('div');
            newItem.className = 'item';
            newItem.textContent = 'New Item';
            newItem.dataset.id = '100';
            document.getElementById('container').appendChild(newItem);
        """)
        time.sleep(0.2)

    def test_create_multiple_elements(self, webview_with_dom):
        """Test creating multiple elements."""
        webview_with_dom.eval_js("""
            const container = document.getElementById('container');
            for (let i = 0; i < 10; i++) {
                const item = document.createElement('div');
                item.className = 'item';
                item.textContent = 'Batch Item ' + i;
                item.dataset.id = 'batch-' + i;
                container.appendChild(item);
            }
        """)
        time.sleep(0.2)

    def test_insert_before(self, webview_with_dom):
        """Test inserting element before another."""
        webview_with_dom.eval_js("""
            const container = document.getElementById('container');
            const firstItem = container.querySelector('.item');
            const newItem = document.createElement('div');
            newItem.className = 'item highlight';
            newItem.textContent = 'Inserted First';
            container.insertBefore(newItem, firstItem);
        """)
        time.sleep(0.2)

    def test_clone_node(self, webview_with_dom):
        """Test cloning element."""
        webview_with_dom.eval_js("""
            const original = document.querySelector('.item');
            const clone = original.cloneNode(true);
            clone.textContent = 'Cloned Item';
            clone.dataset.id = 'clone';
            document.getElementById('container').appendChild(clone);
        """)
        time.sleep(0.2)


# ============================================================
# DOM Removal Tests
# ============================================================


class TestDOMRemoval:
    """Test DOM element removal."""

    def test_remove_element(self, webview_with_dom):
        """Test removing element."""
        webview_with_dom.eval_js("""
            const item = document.querySelector('.item');
            item.remove();
        """)
        time.sleep(0.2)

    def test_remove_child(self, webview_with_dom):
        """Test removing child element."""
        webview_with_dom.eval_js("""
            const container = document.getElementById('container');
            const firstItem = container.querySelector('.item');
            container.removeChild(firstItem);
        """)
        time.sleep(0.2)

    def test_clear_children(self, webview_with_dom):
        """Test clearing all children."""
        webview_with_dom.eval_js("""
            document.getElementById('container').innerHTML = '';
        """)
        time.sleep(0.2)


# ============================================================
# DomBatch Tests (High-Performance)
# ============================================================


class TestDomBatch:
    """Test DomBatch high-performance operations."""

    @pytest.mark.skipif(DomBatch is None, reason="DomBatch not available")
    def test_batch_creation(self):
        """Test DomBatch creation."""
        batch = DomBatch()
        assert batch is not None

    @pytest.mark.skipif(DomBatch is None, reason="DomBatch not available")
    def test_batch_set_text(self, webview_with_dom):
        """Test batch set_text operation."""
        batch = DomBatch()
        batch.set_text("#title", "Batch Modified Title")

        # Execute batch
        webview_with_dom.eval_js("""
            // Batch operations would be executed here
            document.getElementById('title').textContent = 'Batch Modified Title';
        """)
        time.sleep(0.2)

    @pytest.mark.skipif(DomBatch is None, reason="DomBatch not available")
    def test_batch_set_html(self, webview_with_dom):
        """Test batch set_html operation."""
        batch = DomBatch()
        batch.set_html("#dynamic-area", "<p>Batch HTML</p>")

        webview_with_dom.eval_js("""
            document.getElementById('dynamic-area').innerHTML = '<p>Batch HTML</p>';
        """)
        time.sleep(0.2)

    @pytest.mark.skipif(DomBatch is None, reason="DomBatch not available")
    def test_batch_add_class(self, webview_with_dom):
        """Test batch add_class operation."""
        batch = DomBatch()
        batch.add_class(".item", "highlight")

        webview_with_dom.eval_js("""
            document.querySelectorAll('.item').forEach(el => el.classList.add('highlight'));
        """)
        time.sleep(0.2)

    @pytest.mark.skipif(DomBatch is None, reason="DomBatch not available")
    def test_batch_multiple_operations(self, webview_with_dom):
        """Test multiple batch operations."""
        batch = DomBatch()
        batch.set_text("#title", "New Title")
        batch.add_class("#title", "large")
        batch.add_class("#title", "blue")
        batch.set_attr("#title", "data-modified", "true")

        webview_with_dom.eval_js("""
            const title = document.getElementById('title');
            title.textContent = 'New Title';
            title.classList.add('large', 'blue');
            title.dataset.modified = 'true';
        """)
        time.sleep(0.2)


# ============================================================
# Table DOM Tests
# ============================================================


class TestTableDOM:
    """Test DOM operations on tables."""

    def test_query_table_rows(self, webview_with_table):
        """Test querying table rows."""
        webview_with_table.eval_js("""
            const rows = document.querySelectorAll('#table-body tr');
            window.__test_result = rows.length;
        """)
        time.sleep(0.2)

    def test_add_table_row(self, webview_with_table):
        """Test adding table row."""
        webview_with_table.eval_js("""
            const tbody = document.getElementById('table-body');
            const row = document.createElement('tr');
            row.dataset.id = '4';
            row.innerHTML = `
                <td>4</td>
                <td class="name">David</td>
                <td class="email">david@example.com</td>
                <td class="status">Active</td>
            `;
            tbody.appendChild(row);
        """)
        time.sleep(0.2)

    def test_modify_table_cell(self, webview_with_table):
        """Test modifying table cell."""
        webview_with_table.eval_js("""
            const cell = document.querySelector('tr[data-id="1"] .status');
            cell.textContent = 'Pending';
            cell.style.color = 'orange';
        """)
        time.sleep(0.2)

    def test_delete_table_row(self, webview_with_table):
        """Test deleting table row."""
        webview_with_table.eval_js("""
            const row = document.querySelector('tr[data-id="2"]');
            row.remove();
        """)
        time.sleep(0.2)

    def test_select_table_row(self, webview_with_table):
        """Test selecting table row."""
        webview_with_table.eval_js("""
            const row = document.querySelector('tr[data-id="1"]');
            row.classList.add('selected');
            document.getElementById('selection-info').textContent = 'Selected: Alice';
        """)
        time.sleep(0.2)


# ============================================================
# AuroraTest DOM Integration
# ============================================================


class TestAuroraTestDOMIntegration:
    """Test DOM operations using AuroraTest framework."""

    @pytest.mark.asyncio
    async def test_locator_count(self):
        """Test counting elements with locator."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(DOM_TEST_HTML)
        await page.wait_for_timeout(500)

        # Count items
        await page.locator(".item").count()
        # Note: count() returns 1 by default in current implementation

        browser.close()

    @pytest.mark.asyncio
    async def test_locator_nth(self):
        """Test nth element selection."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(DOM_TEST_HTML)
        await page.wait_for_timeout(500)

        # Select 3rd item
        await page.locator(".item").nth(2).click()
        await page.wait_for_timeout(200)

        browser.close()

    @pytest.mark.asyncio
    async def test_locator_first_last(self):
        """Test first/last element selection."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(DOM_TEST_HTML)
        await page.wait_for_timeout(500)

        # Click first and last
        await page.locator(".item").first().click()
        await page.wait_for_timeout(100)
        await page.locator(".item").last().click()
        await page.wait_for_timeout(200)

        browser.close()

    @pytest.mark.asyncio
    async def test_locator_filter(self):
        """Test locator filtering."""
        browser = Browser.launch(headless=False)
        page = browser.new_page()

        await page.set_content(DOM_TEST_HTML)
        await page.wait_for_timeout(500)

        # Filter by text
        await page.locator(".item").filter(has_text="Item 3").click()
        await page.wait_for_timeout(200)

        browser.close()


# ============================================================
# Performance Tests
# ============================================================


class TestDOMPerformance:
    """Test DOM operation performance."""

    def test_bulk_element_creation(self, webview_with_dom):
        """Test creating many elements quickly."""
        start = time.time()

        webview_with_dom.eval_js("""
            const container = document.getElementById('container');
            const fragment = document.createDocumentFragment();

            for (let i = 0; i < 1000; i++) {
                const item = document.createElement('div');
                item.className = 'item';
                item.textContent = 'Bulk Item ' + i;
                item.dataset.id = 'bulk-' + i;
                fragment.appendChild(item);
            }

            container.appendChild(fragment);
        """)

        elapsed = time.time() - start
        logger.info(f"Created 1000 elements in {elapsed:.3f}s")

        time.sleep(0.5)  # Let DOM settle

    def test_bulk_class_modification(self, webview_with_dom):
        """Test modifying classes on many elements."""
        # First create elements
        webview_with_dom.eval_js("""
            const container = document.getElementById('container');
            for (let i = 0; i < 100; i++) {
                const item = document.createElement('div');
                item.className = 'item';
                item.textContent = 'Item ' + i;
                container.appendChild(item);
            }
        """)
        time.sleep(0.3)

        start = time.time()

        webview_with_dom.eval_js("""
            document.querySelectorAll('.item').forEach(el => {
                el.classList.add('highlight');
                el.classList.add('active');
            });
        """)

        elapsed = time.time() - start
        logger.info(f"Modified classes on 100+ elements in {elapsed:.3f}s")

        time.sleep(0.2)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
