"""Simple calculator tool using AuroraView.

This example demonstrates:
- Basic Python API binding with decorators
- Bidirectional communication between Python and JavaScript
- Simple UI with HTML/CSS/JS
"""

from auroraview import WebView
from pathlib import Path


def main():
    """Launch the simple calculator."""
    # Get the HTML file path
    html_path = Path(__file__).parent / "simple_calculator.html"
    
    # Create and configure WebView
    webview = WebView(
        title="Simple Calculator",
        width=400,
        height=500,
        debug=True,
    )
    
    # Bind API methods
    @webview.bind_call("add")
    def add_numbers(a: float, b: float) -> float:
        """Add two numbers."""
        return a + b
    
    @webview.bind_call("subtract")
    def subtract_numbers(a: float, b: float) -> float:
        """Subtract b from a."""
        return a - b
    
    @webview.bind_call("multiply")
    def multiply_numbers(a: float, b: float) -> float:
        """Multiply two numbers."""
        return a * b
    
    @webview.bind_call("divide")
    def divide_numbers(a: float, b: float) -> float | None:
        """Divide a by b."""
        if b == 0:
            return None
        return a / b
    
    # Show the window with the HTML file
    webview.show(html_path)


if __name__ == "__main__":
    main()
