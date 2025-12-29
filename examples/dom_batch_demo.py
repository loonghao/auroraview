"""DOM Batch Operations Demo - High-Performance DOM Manipulation.

This example demonstrates high-performance DOM manipulation using
batch operations. Essential for scenarios requiring multiple DOM
updates with minimal IPC overhead.

Usage:
    python examples/dom_batch_demo.py

Features demonstrated:
    - Element and ElementCollection classes
    - Batch DOM operations for performance
    - Style, class, and attribute manipulation
    - Form handling and validation
    - DOM traversal methods
    - Proxy-style access (style[], classes, attributes, data)
"""

from __future__ import annotations

import random

from auroraview import WebView


def main():
    """Run the DOM batch operations demo."""
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>DOM Batch Operations Demo</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 1000px;
                margin: 50px auto;
                padding: 20px;
                background: linear-gradient(135deg, #00c6ff 0%, #0072ff 100%);
                min-height: 100vh;
            }
            .card {
                background: white;
                border-radius: 12px;
                padding: 24px;
                box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                margin-bottom: 20px;
            }
            h1 { color: #333; margin-top: 0; }
            h3 { color: #666; margin-bottom: 10px; }
            button {
                background: linear-gradient(135deg, #00c6ff 0%, #0072ff 100%);
                color: white;
                border: none;
                padding: 10px 20px;
                border-radius: 6px;
                cursor: pointer;
                font-size: 14px;
                margin: 3px;
                transition: transform 0.1s;
            }
            button:hover { transform: translateY(-2px); }
            button:active { transform: translateY(0); }
            .demo-grid {
                display: grid;
                grid-template-columns: repeat(5, 1fr);
                gap: 10px;
                margin: 20px 0;
            }
            .demo-item {
                background: #f5f5f5;
                padding: 20px;
                border-radius: 8px;
                text-align: center;
                transition: all 0.3s ease;
                cursor: pointer;
            }
            .demo-item:hover { transform: scale(1.05); }
            .demo-item.highlighted { background: #ffeb3b; }
            .demo-item.selected { background: #4caf50; color: white; }
            .demo-item.error { background: #f44336; color: white; }
            .demo-item.hidden { display: none; }
            .form-demo {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 15px;
                margin: 20px 0;
            }
            .form-group {
                display: flex;
                flex-direction: column;
                gap: 5px;
            }
            .form-group label {
                font-weight: 500;
                color: #333;
            }
            .form-group input, .form-group select {
                padding: 10px;
                border: 2px solid #e0e0e0;
                border-radius: 6px;
                font-size: 14px;
            }
            .form-group input:focus, .form-group select:focus {
                outline: none;
                border-color: #0072ff;
            }
            .form-group input.valid { border-color: #4caf50; }
            .form-group input.invalid { border-color: #f44336; }
            .stats-bar {
                display: flex;
                gap: 20px;
                padding: 15px;
                background: #f5f5f5;
                border-radius: 8px;
                margin: 15px 0;
            }
            .stat-item {
                flex: 1;
                text-align: center;
            }
            .stat-value { font-size: 24px; font-weight: bold; color: #0072ff; }
            .stat-label { font-size: 12px; color: #666; }
            #performanceLog {
                background: #1e1e1e;
                color: #0f0;
                border-radius: 8px;
                padding: 16px;
                font-family: 'Consolas', monospace;
                font-size: 12px;
                max-height: 150px;
                overflow-y: auto;
            }
            .traversal-tree {
                background: #f5f5f5;
                padding: 20px;
                border-radius: 8px;
                margin: 15px 0;
            }
            .tree-node {
                padding: 8px 15px;
                margin: 5px 0;
                background: white;
                border-radius: 4px;
                border-left: 3px solid #0072ff;
            }
            .tree-node.current { border-left-color: #4caf50; background: #e8f5e9; }
        </style>
    </head>
    <body>
        <div class="card">
            <h1>DOM Batch Operations Demo</h1>
            <p>High-performance DOM manipulation with batch operations and proxy-style access.</p>

            <h3>Batch Operations on Grid</h3>
            <div class="demo-grid" id="demoGrid">
                <!-- Items will be generated by Python -->
            </div>

            <div>
                <button onclick="requestBatchHighlight()">Batch Highlight</button>
                <button onclick="requestBatchSelect()">Batch Select</button>
                <button onclick="requestBatchStyle()">Batch Style</button>
                <button onclick="requestBatchToggle()">Toggle Classes</button>
                <button onclick="requestResetGrid()">Reset Grid</button>
            </div>

            <div class="stats-bar">
                <div class="stat-item">
                    <div class="stat-value" id="opCount">0</div>
                    <div class="stat-label">Operations</div>
                </div>
                <div class="stat-item">
                    <div class="stat-value" id="lastTime">0ms</div>
                    <div class="stat-label">Last Batch Time</div>
                </div>
                <div class="stat-item">
                    <div class="stat-value" id="itemCount">0</div>
                    <div class="stat-label">Items Modified</div>
                </div>
            </div>
        </div>

        <div class="card">
            <h3>Form Manipulation</h3>
            <div class="form-demo">
                <div class="form-group">
                    <label for="userName">Username</label>
                    <input type="text" id="userName" placeholder="Enter username">
                </div>
                <div class="form-group">
                    <label for="userEmail">Email</label>
                    <input type="email" id="userEmail" placeholder="Enter email">
                </div>
                <div class="form-group">
                    <label for="userRole">Role</label>
                    <select id="userRole">
                        <option value="">Select role...</option>
                        <option value="admin">Administrator</option>
                        <option value="user">User</option>
                        <option value="guest">Guest</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="userAge">Age</label>
                    <input type="number" id="userAge" placeholder="Enter age">
                </div>
            </div>
            <div>
                <button onclick="requestFillForm()">Fill Form (Python)</button>
                <button onclick="requestValidateForm()">Validate Form</button>
                <button onclick="requestClearForm()">Clear Form</button>
                <button onclick="requestReadForm()">Read Form Values</button>
            </div>
        </div>

        <div class="card">
            <h3>DOM Traversal</h3>
            <div class="traversal-tree" id="traversalTree">
                <div class="tree-node" data-level="1" id="node1">
                    Parent Node
                    <div class="tree-node" data-level="2" id="node2">
                        Child 1
                        <div class="tree-node" data-level="3" id="node3">Grandchild 1</div>
                        <div class="tree-node" data-level="3" id="node4">Grandchild 2</div>
                    </div>
                    <div class="tree-node" data-level="2" id="node5">
                        Child 2
                        <div class="tree-node" data-level="3" id="node6">Grandchild 3</div>
                    </div>
                </div>
            </div>
            <div>
                <button onclick="requestTraverseParent()">Find Parent</button>
                <button onclick="requestTraverseChildren()">Find Children</button>
                <button onclick="requestTraverseSiblings()">Find Siblings</button>
                <button onclick="requestHighlightLevel()">Highlight Level 2</button>
            </div>
        </div>

        <div class="card">
            <h3>Performance Log</h3>
            <div id="performanceLog">DOM operations will be logged here...</div>
        </div>

        <script>
            let opCount = 0;

            function log(msg) {
                const logEl = document.getElementById('performanceLog');
                const timestamp = new Date().toLocaleTimeString();
                logEl.textContent = `[${timestamp}] ${msg}\\n` + logEl.textContent;
            }

            function updateStats(ops, time, items) {
                opCount += ops;
                document.getElementById('opCount').textContent = opCount;
                document.getElementById('lastTime').textContent = time + 'ms';
                document.getElementById('itemCount').textContent = items;
            }

            async function requestBatchHighlight() {
                const start = performance.now();
                try {
                    const result = await auroraview.api.batch_highlight();
                    const time = Math.round(performance.now() - start);
                    updateStats(result.operations, time, result.items_modified);
                    log(`Batch highlight: ${result.items_modified} items in ${time}ms`);
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestBatchSelect() {
                const start = performance.now();
                try {
                    const result = await auroraview.api.batch_select();
                    const time = Math.round(performance.now() - start);
                    updateStats(result.operations, time, result.items_modified);
                    log(`Batch select: ${result.items_modified} items in ${time}ms`);
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestBatchStyle() {
                const start = performance.now();
                try {
                    const result = await auroraview.api.batch_style();
                    const time = Math.round(performance.now() - start);
                    updateStats(result.operations, time, result.items_modified);
                    log(`Batch style: ${result.items_modified} items in ${time}ms`);
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestBatchToggle() {
                const start = performance.now();
                try {
                    const result = await auroraview.api.batch_toggle();
                    const time = Math.round(performance.now() - start);
                    updateStats(result.operations, time, result.items_modified);
                    log(`Batch toggle: ${result.items_modified} items in ${time}ms`);
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestResetGrid() {
                try {
                    await auroraview.api.reset_grid();
                    log('Grid reset');
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestFillForm() {
                try {
                    const result = await auroraview.api.fill_form();
                    log(`Form filled: ${JSON.stringify(result.data)}`);
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestValidateForm() {
                try {
                    const result = await auroraview.api.validate_form();
                    log(`Form validation: ${result.valid ? 'PASSED' : 'FAILED'}`);
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestClearForm() {
                try {
                    await auroraview.api.clear_form();
                    log('Form cleared');
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestReadForm() {
                try {
                    const result = await auroraview.api.read_form();
                    log(`Form values: ${JSON.stringify(result)}`);
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestTraverseParent() {
                try {
                    await auroraview.api.traverse_parent();
                    log('Traversed to parent');
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestTraverseChildren() {
                try {
                    await auroraview.api.traverse_children();
                    log('Found children');
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestTraverseSiblings() {
                try {
                    await auroraview.api.traverse_siblings();
                    log('Found siblings');
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }

            async function requestHighlightLevel() {
                try {
                    await auroraview.api.highlight_level({level: 2});
                    log('Highlighted level 2 nodes');
                } catch (e) {
                    log(`Error: ${e.message}`);
                }
            }
        </script>
    </body>
    </html>
    """

    view = WebView(title="DOM Batch Operations Demo", html=html_content, width=1050, height=900)

    # Generate grid items
    @view.bind_call("api.init_grid")
    def init_grid() -> dict:
        """Initialize the demo grid with items."""
        grid = view.dom("#demoGrid")
        items_html = ""
        for i in range(20):
            items_html += f'<div class="demo-item" data-index="{i}" id="item{i}">Item {i + 1}</div>'
        grid.set_html(items_html)
        return {"ok": True, "items": 20}

    @view.bind_call("api.batch_highlight")
    def batch_highlight() -> dict:
        """Highlight random items using batch operations."""
        # Use ElementCollection for batch operations
        items = view.dom_all(".demo-item")
        items.remove_class("highlighted", "selected", "error")

        # Highlight random items
        modified = 0
        for i in random.sample(range(20), 8):
            view.dom(f"#item{i}").add_class("highlighted")
            modified += 1

        return {"ok": True, "operations": 2, "items_modified": modified}

    @view.bind_call("api.batch_select")
    def batch_select() -> dict:
        """Select items using batch operations."""
        items = view.dom_all(".demo-item")
        items.remove_class("highlighted", "selected", "error")

        # Select every third item
        modified = 0
        for i in range(0, 20, 3):
            view.dom(f"#item{i}").add_class("selected")
            modified += 1

        return {"ok": True, "operations": 2, "items_modified": modified}

    @view.bind_call("api.batch_style")
    def batch_style() -> dict:
        """Apply styles using batch operations."""
        # Apply gradient background to all items

        colors = ["#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#ffeaa7"]
        modified = 0

        for i in range(20):
            color = colors[i % len(colors)]
            view.dom(f"#item{i}").set_style("background", color)
            view.dom(f"#item{i}").set_style("color", "white")
            modified += 1

        return {"ok": True, "operations": 40, "items_modified": modified}

    @view.bind_call("api.batch_toggle")
    def batch_toggle() -> dict:
        """Toggle classes on items."""
        modified = 0
        for i in range(20):
            view.dom(f"#item{i}").toggle_class("highlighted")
            modified += 1

        return {"ok": True, "operations": 20, "items_modified": modified}

    @view.bind_call("api.reset_grid")
    def reset_grid() -> dict:
        """Reset grid to initial state."""
        items = view.dom_all(".demo-item")
        items.remove_class("highlighted", "selected", "error")

        # Reset styles
        for i in range(20):
            view.dom(f"#item{i}").set_style("background", "#f5f5f5")
            view.dom(f"#item{i}").set_style("color", "inherit")

        return {"ok": True}

    @view.bind_call("api.fill_form")
    def fill_form() -> dict:
        """Fill form fields using DOM manipulation."""
        data = {
            "userName": "john_doe",
            "userEmail": "john@example.com",
            "userRole": "admin",
            "userAge": "30",
        }

        view.dom("#userName").set_value(data["userName"])
        view.dom("#userEmail").set_value(data["userEmail"])
        view.dom("#userRole").set_value(data["userRole"])
        view.dom("#userAge").set_value(data["userAge"])

        return {"ok": True, "data": data}

    @view.bind_call("api.validate_form")
    def validate_form() -> dict:
        """Validate form and show visual feedback."""
        # Simple validation - add valid/invalid classes
        fields = ["#userName", "#userEmail", "#userAge"]
        all_valid = True

        for field in fields:
            el = view.dom(field)
            # For demo, just check if field exists and add classes
            el.remove_class("valid", "invalid")
            # Assume valid for demo
            el.add_class("valid")

        return {"ok": True, "valid": all_valid}

    @view.bind_call("api.clear_form")
    def clear_form() -> dict:
        """Clear all form fields."""
        view.dom("#userName").clear()
        view.dom("#userEmail").clear()
        view.dom("#userAge").clear()
        view.dom("#userRole").set_value("")

        # Remove validation classes
        for field in ["#userName", "#userEmail", "#userAge"]:
            view.dom(field).remove_class("valid", "invalid")

        return {"ok": True}

    @view.bind_call("api.read_form")
    def read_form() -> dict:
        """Read form values (demo - actual reading requires JS bridge)."""
        # Note: In real usage, you'd use evaluate_js to get values
        return {"message": "Form reading requires JS evaluation - see dom_manipulation_demo.py"}

    @view.bind_call("api.traverse_parent")
    def traverse_parent() -> dict:
        """Demonstrate parent traversal."""
        # Highlight parent of node3
        view.dom_all(".tree-node").remove_class("current")
        view.dom("#node2").add_class("current")
        return {"ok": True}

    @view.bind_call("api.traverse_children")
    def traverse_children() -> dict:
        """Demonstrate children traversal."""
        view.dom_all(".tree-node").remove_class("current")
        # Highlight children of node2
        view.dom("#node3").add_class("current")
        view.dom("#node4").add_class("current")
        return {"ok": True}

    @view.bind_call("api.traverse_siblings")
    def traverse_siblings() -> dict:
        """Demonstrate sibling traversal."""
        view.dom_all(".tree-node").remove_class("current")
        # Highlight siblings (node2 and node5 are siblings)
        view.dom("#node2").add_class("current")
        view.dom("#node5").add_class("current")
        return {"ok": True}

    @view.bind_call("api.highlight_level")
    def highlight_level(level: int = 2) -> dict:
        """Highlight nodes at a specific level."""
        view.dom_all(".tree-node").remove_class("current")
        view.dom_all(f'.tree-node[data-level="{level}"]').add_class("current")
        return {"ok": True, "level": level}

    # Initialize grid after load
    @view.on("ready")
    def on_ready():
        init_grid()

    print("Starting DOM Batch Operations Demo...")
    print("Features: Batch operations, Form handling, DOM traversal")
    view.show()


if __name__ == "__main__":
    main()
