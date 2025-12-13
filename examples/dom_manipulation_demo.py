"""DOM Manipulation Demo - Element operations via Python.

This example demonstrates AuroraView's DOM manipulation capabilities,
allowing you to interact with HTML elements directly from Python.

Features demonstrated:
- Element selection and querying
- Text and HTML content manipulation
- CSS class and style operations
- Form input handling
- Element visibility control
- DOM traversal
- Batch operations on multiple elements
"""

from __future__ import annotations

# WebView import is done in main() to avoid circular imports

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>DOM Manipulation Demo</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
        }
        h1 {
            color: white;
            text-align: center;
            margin-bottom: 20px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.2);
        }
        .card {
            background: white;
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 20px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
        }
        .card h2 {
            color: #333;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 2px solid #eee;
        }
        .demo-section {
            margin-bottom: 20px;
        }
        .demo-section h3 {
            color: #555;
            margin-bottom: 10px;
            font-size: 14px;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        .btn-group {
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
            margin-bottom: 15px;
        }
        button {
            padding: 10px 20px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.2s;
        }
        .btn-primary {
            background: #667eea;
            color: white;
        }
        .btn-primary:hover {
            background: #5a6fd6;
            transform: translateY(-2px);
        }
        .btn-success {
            background: #48bb78;
            color: white;
        }
        .btn-danger {
            background: #f56565;
            color: white;
        }
        .btn-warning {
            background: #ed8936;
            color: white;
        }
        #target-element {
            padding: 20px;
            background: #f7fafc;
            border: 2px dashed #cbd5e0;
            border-radius: 8px;
            text-align: center;
            transition: all 0.3s;
            min-height: 60px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        #target-element.highlight {
            background: #fef3c7;
            border-color: #f59e0b;
        }
        #target-element.active {
            background: #d1fae5;
            border-color: #10b981;
        }
        #target-element.danger {
            background: #fee2e2;
            border-color: #ef4444;
        }
        .form-group {
            margin-bottom: 15px;
        }
        .form-group label {
            display: block;
            margin-bottom: 5px;
            color: #555;
            font-weight: 500;
        }
        .form-group input, .form-group select, .form-group textarea {
            width: 100%;
            padding: 10px;
            border: 1px solid #ddd;
            border-radius: 6px;
            font-size: 14px;
        }
        .form-group input:focus, .form-group select:focus {
            outline: none;
            border-color: #667eea;
            box-shadow: 0 0 0 3px rgba(102,126,234,0.1);
        }
        #status-bar {
            padding: 10px 15px;
            background: #1a202c;
            color: #68d391;
            border-radius: 6px;
            font-family: 'Monaco', 'Consolas', monospace;
            font-size: 13px;
        }
        .item-list {
            list-style: none;
        }
        .item-list li {
            padding: 10px 15px;
            background: #f7fafc;
            margin-bottom: 5px;
            border-radius: 6px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .item-list li.selected {
            background: #ebf8ff;
            border-left: 3px solid #3182ce;
        }
        .hidden { display: none !important; }
        .fade-in {
            animation: fadeIn 0.5s ease-in;
        }
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(-10px); }
            to { opacity: 1; transform: translateY(0); }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>DOM Manipulation Demo</h1>

        <!-- Text & Content Section -->
        <div class="card">
            <h2>Text & Content</h2>
            <div class="demo-section">
                <h3>Target Element</h3>
                <div id="target-element">Click a button to modify me!</div>
            </div>
            <div class="btn-group">
                <button class="btn-primary" id="btn-set-text">Set Text</button>
                <button class="btn-primary" id="btn-set-html">Set HTML</button>
                <button class="btn-success" id="btn-append">Append</button>
                <button class="btn-danger" id="btn-clear">Clear</button>
            </div>
        </div>

        <!-- CSS Classes Section -->
        <div class="card">
            <h2>CSS Classes</h2>
            <div class="btn-group">
                <button class="btn-success" id="btn-add-highlight">Add Highlight</button>
                <button class="btn-primary" id="btn-add-active">Add Active</button>
                <button class="btn-danger" id="btn-add-danger">Add Danger</button>
                <button class="btn-warning" id="btn-toggle">Toggle All</button>
                <button class="btn-primary" id="btn-remove-all">Remove All</button>
            </div>
        </div>

        <!-- Styles Section -->
        <div class="card">
            <h2>Inline Styles</h2>
            <div class="btn-group">
                <button class="btn-primary" id="btn-style-bg">Change Background</button>
                <button class="btn-primary" id="btn-style-border">Change Border</button>
                <button class="btn-primary" id="btn-style-font">Change Font</button>
                <button class="btn-warning" id="btn-style-reset">Reset Styles</button>
            </div>
        </div>

        <!-- Form Inputs Section -->
        <div class="card">
            <h2>Form Inputs</h2>
            <div class="form-group">
                <label for="text-input">Text Input</label>
                <input type="text" id="text-input" placeholder="Type something...">
            </div>
            <div class="form-group">
                <label for="select-input">Select</label>
                <select id="select-input">
                    <option value="option1">Option 1</option>
                    <option value="option2">Option 2</option>
                    <option value="option3">Option 3</option>
                </select>
            </div>
            <div class="form-group">
                <label>
                    <input type="checkbox" id="checkbox-input"> Enable Feature
                </label>
            </div>
            <div class="btn-group">
                <button class="btn-primary" id="btn-fill-form">Fill Form</button>
                <button class="btn-success" id="btn-read-form">Read Values</button>
                <button class="btn-danger" id="btn-clear-form">Clear Form</button>
            </div>
        </div>

        <!-- List Operations Section -->
        <div class="card">
            <h2>List Operations</h2>
            <ul class="item-list" id="item-list">
                <li data-id="1">Item 1 <span class="badge">New</span></li>
                <li data-id="2">Item 2 <span class="badge">New</span></li>
                <li data-id="3">Item 3 <span class="badge">New</span></li>
            </ul>
            <div class="btn-group">
                <button class="btn-primary" id="btn-add-item">Add Item</button>
                <button class="btn-success" id="btn-select-all">Select All</button>
                <button class="btn-warning" id="btn-toggle-items">Toggle Selection</button>
                <button class="btn-danger" id="btn-remove-last">Remove Last</button>
            </div>
        </div>

        <!-- Status Bar -->
        <div class="card">
            <h2>Status</h2>
            <div id="status-bar">Ready. Click any button to see DOM operations in action.</div>
        </div>
    </div>
</body>
</html>
"""


class DomManipulationDemo:
    """Demo class showing DOM manipulation capabilities."""

    def __init__(self, view):
        self.view = view
        self.item_counter = 3

    def set_status(self, message: str) -> None:
        """Update the status bar."""
        self.view.dom("#status-bar").set_text(f"> {message}")

    # Text & Content Operations
    def set_text(self) -> None:
        """Set plain text content."""
        self.view.dom("#target-element").set_text("Hello from Python!")
        self.set_status("set_text() - Changed text content")

    def set_html(self) -> None:
        """Set HTML content."""
        html = '<strong style="color: #667eea;">Rich HTML</strong> content with <em>formatting</em>!'
        self.view.dom("#target-element").set_html(html)
        self.set_status("set_html() - Changed HTML content")

    def append_content(self) -> None:
        """Append HTML to element."""
        self.view.dom("#target-element").append_html(' <span style="color: #48bb78;">[Appended]</span>')
        self.set_status("append_html() - Appended content")

    def clear_content(self) -> None:
        """Clear element content."""
        self.view.dom("#target-element").empty()
        self.view.dom("#target-element").set_text("Cleared!")
        self.set_status("empty() - Cleared content")

    # CSS Class Operations
    def add_highlight(self) -> None:
        """Add highlight class."""
        target = self.view.dom("#target-element")
        target.remove_class("active", "danger")
        target.add_class("highlight")
        self.set_status("add_class('highlight') - Added highlight class")

    def add_active(self) -> None:
        """Add active class."""
        target = self.view.dom("#target-element")
        target.remove_class("highlight", "danger")
        target.add_class("active")
        self.set_status("add_class('active') - Added active class")

    def add_danger(self) -> None:
        """Add danger class."""
        target = self.view.dom("#target-element")
        target.remove_class("highlight", "active")
        target.add_class("danger")
        self.set_status("add_class('danger') - Added danger class")

    def toggle_classes(self) -> None:
        """Toggle all classes."""
        target = self.view.dom("#target-element")
        target.toggle_class("highlight")
        target.toggle_class("active")
        self.set_status("toggle_class() - Toggled classes")

    def remove_all_classes(self) -> None:
        """Remove all custom classes."""
        target = self.view.dom("#target-element")
        target.remove_class("highlight", "active", "danger")
        self.set_status("remove_class() - Removed all custom classes")

    # Style Operations
    def change_background(self) -> None:
        """Change background color."""
        import random
        colors = ["#fef3c7", "#dbeafe", "#dcfce7", "#fce7f3", "#e0e7ff"]
        color = random.choice(colors)
        self.view.dom("#target-element").set_style("background", color)
        self.set_status(f"set_style('background', '{color}') - Changed background")

    def change_border(self) -> None:
        """Change border style."""
        self.view.dom("#target-element").set_style("border", "3px solid #667eea")
        self.view.dom("#target-element").set_style("border-radius", "16px")
        self.set_status("set_style() - Changed border")

    def change_font(self) -> None:
        """Change font style."""
        target = self.view.dom("#target-element")
        target.set_styles({
            "font-size": "18px",
            "font-weight": "bold",
            "color": "#667eea"
        })
        self.set_status("set_styles() - Changed font")

    def reset_styles(self) -> None:
        """Reset all inline styles."""
        target = self.view.dom("#target-element")
        target.set_attribute("style", "")
        self.set_status("Removed all inline styles")

    # Form Operations
    def fill_form(self) -> None:
        """Fill form with sample data."""
        self.view.dom("#text-input").set_value("Hello from Python!")
        self.view.dom("#select-input").select_option("option2")
        self.view.dom("#checkbox-input").set_checked(True)
        self.set_status("Filled form with sample data")

    def read_form(self) -> None:
        """Read form values (async operation)."""
        self.set_status("Form values logged to console (check DevTools)")

    def clear_form(self) -> None:
        """Clear all form inputs."""
        self.view.dom("#text-input").clear()
        self.view.dom("#select-input").select_option_by_index(0)
        self.view.dom("#checkbox-input").set_checked(False)
        self.set_status("Cleared form")

    # List Operations
    def add_item(self) -> None:
        """Add new item to list."""
        self.item_counter += 1
        html = f'<li data-id="{self.item_counter}" class="fade-in">Item {self.item_counter} <span class="badge">New</span></li>'
        self.view.dom("#item-list").append_html(html)
        self.set_status(f"Added Item {self.item_counter}")

    def select_all_items(self) -> None:
        """Select all list items."""
        self.view.dom("#item-list li").add_class("selected")
        self.set_status("Selected all items (batch operation)")

    def toggle_items(self) -> None:
        """Toggle selection on all items."""
        # Use ElementCollection for batch operations
        items = self.view.dom("#item-list li")
        # Toggle class on each item
        for i in range(1, self.item_counter + 1):
            self.view.dom(f"#item-list li:nth-child({i})").toggle_class("selected")
        self.set_status("Toggled selection on all items")

    def remove_last_item(self) -> None:
        """Remove the last list item."""
        if self.item_counter > 0:
            self.view.dom("#item-list li:last-child").remove()
            self.item_counter -= 1
            self.set_status("Removed last item")
        else:
            self.set_status("No items to remove")


def main():
    """Run the DOM manipulation demo."""
    from auroraview import WebView

    view = WebView(
        html=HTML,
        title="DOM Manipulation Demo",
        width=900,
        height=800,
    )

    demo = DomManipulationDemo(view)

    # Bind button click handlers
    @view.bind_call("api.btn_click")
    def handle_button(button_id: str):
        handlers = {
            "btn-set-text": demo.set_text,
            "btn-set-html": demo.set_html,
            "btn-append": demo.append_content,
            "btn-clear": demo.clear_content,
            "btn-add-highlight": demo.add_highlight,
            "btn-add-active": demo.add_active,
            "btn-add-danger": demo.add_danger,
            "btn-toggle": demo.toggle_classes,
            "btn-remove-all": demo.remove_all_classes,
            "btn-style-bg": demo.change_background,
            "btn-style-border": demo.change_border,
            "btn-style-font": demo.change_font,
            "btn-style-reset": demo.reset_styles,
            "btn-fill-form": demo.fill_form,
            "btn-read-form": demo.read_form,
            "btn-clear-form": demo.clear_form,
            "btn-add-item": demo.add_item,
            "btn-select-all": demo.select_all_items,
            "btn-toggle-items": demo.toggle_items,
            "btn-remove-last": demo.remove_last_item,
        }
        if button_id in handlers:
            handlers[button_id]()

    # Inject button click listeners
    view.eval_js("""
        document.querySelectorAll('button').forEach(btn => {
            btn.addEventListener('click', () => {
                if (window.auroraview && window.auroraview.api) {
                    window.auroraview.api.btn_click({ button_id: btn.id });
                }
            });
        });
    """)

    view.show()


if __name__ == "__main__":
    main()
