"""
WebViewBot - High-level API for WebView testing

Provides automation and assertion methods for testing WebView applications.
"""

from dataclasses import dataclass
from typing import Any, Dict, List, Optional
import time


@dataclass
class EventRecord:
    """Record of an emitted event"""
    name: str
    data: Optional[Dict[str, Any]] = None
    timestamp: float = 0.0


class WebViewBot:
    """
    High-level API for WebView testing and automation.
    
    Provides methods for:
    - Element interaction (click, type, drag)
    - Event monitoring and assertion
    - JavaScript execution
    - Element state checking
    """
    
    def __init__(self, webview):
        """Initialize WebViewBot with a WebView instance"""
        self.webview = webview
        self.events: List[EventRecord] = []
        self._monitoring_active = False
    
    def inject_monitoring_script(self):
        """Inject JavaScript monitoring script into the page"""
        script = """
        window._auroraview_events = [];
        window._auroraview_monitoring = true;
        
        const originalDispatchEvent = window.dispatchEvent;
        window.dispatchEvent = function(event) {
            if (window._auroraview_monitoring) {
                window._auroraview_events.push({
                    name: event.type,
                    timestamp: Date.now()
                });
            }
            return originalDispatchEvent.call(this, event);
        };
        
        window.dispatchEvent(new CustomEvent('webview_ready'));
        """
        self.webview.eval_js(script)
        self._monitoring_active = True
    
    def wait_for_event(self, event_name: str, timeout: float = 5.0) -> bool:
        """Wait for a specific event to be emitted"""
        start_time = time.time()
        while time.time() - start_time < timeout:
            script = f"""
            (function() {{
                const events = window._auroraview_events || [];
                return events.some(e => e.name === '{event_name}');
            }})()
            """
            try:
                result = self.webview.eval_js(script)
                if result:
                    return True
            except:
                pass
            time.sleep(0.1)
        return False
    
    def click(self, selector: str):
        """Click an element"""
        script = f"""
        const element = document.querySelector('{selector}');
        if (element) {{
            element.click();
        }}
        """
        self.webview.eval_js(script)
    
    def type(self, selector: str, text: str):
        """Type text into an element"""
        script = f"""
        const element = document.querySelector('{selector}');
        if (element) {{
            element.value = '{text}';
            element.dispatchEvent(new Event('input', {{ bubbles: true }}));
        }}
        """
        self.webview.eval_js(script)
    
    def drag(self, selector: str, offset: tuple):
        """Drag an element"""
        dx, dy = offset
        script = f"""
        const element = document.querySelector('{selector}');
        if (element) {{
            const event = new MouseEvent('mousedown', {{ bubbles: true }});
            element.dispatchEvent(event);
        }}
        """
        self.webview.eval_js(script)
    
    def element_exists(self, selector: str) -> bool:
        """Check if an element exists"""
        script = f"""
        document.querySelector('{selector}') !== null
        """
        try:
            return self.webview.eval_js(script)
        except:
            return False
    
    def get_element_text(self, selector: str) -> str:
        """Get text content of an element"""
        script = f"""
        const element = document.querySelector('{selector}');
        element ? element.textContent : ''
        """
        try:
            return self.webview.eval_js(script)
        except:
            return ""
    
    def assert_event_emitted(self, event_name: str):
        """Assert that an event was emitted"""
        script = f"""
        (function() {{
            const events = window._auroraview_events || [];
            return events.some(e => e.name === '{event_name}');
        }})()
        """
        try:
            result = self.webview.eval_js(script)
            assert result, f"Event '{event_name}' was not emitted"
        except Exception as e:
            raise AssertionError(f"Failed to check event '{event_name}': {e}")
    
    def inject_monitoring_script(self):
        """Inject monitoring script (alias for compatibility)"""
        self.inject_monitoring_script()

