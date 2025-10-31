# AuroraView Testing Framework

A pytest-qt inspired testing framework for AuroraView WebView applications.

## Overview

This testing framework provides a comprehensive set of tools for testing WebView-based applications with a focus on UI automation, event verification, and JavaScript interaction.

## Features

- **WebView Fixtures**: Easy creation and cleanup of WebView instances
- **WebViewBot**: High-level API for UI automation
- **Event Monitoring**: Automatic tracking of all custom events
- **JavaScript Execution**: Execute and retrieve results from JavaScript
- **Element Interaction**: Simulate clicks, typing, dragging
- **Custom Assertions**: Specialized assertions for WebView state
- **Async Support**: Wait for events and conditions

## Quick Start

### Installation

```bash
# Install pytest if not already installed
pip install pytest

# Run tests
pytest tests/ -v
```

### Basic Test Example

```python
def test_button_click(webview, webview_bot, test_html):
    """Test clicking a button"""
    # Load HTML
    webview.load_html(test_html)
    webview_bot.inject_monitoring_script()
    
    # Wait for page ready
    webview_bot.wait_for_event('webview_ready', timeout=5)
    
    # Click button
    webview_bot.click('#testBtn')
    
    # Verify event
    webview_bot.assert_event_emitted('button_clicked')
```

## Core Components

### 1. Fixtures

#### `webview`
Creates a standalone WebView instance for testing.

```python
def test_my_feature(webview):
    webview.load_html("<h1>Test</h1>")
    # Test code here
```

