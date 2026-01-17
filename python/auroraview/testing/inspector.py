# -*- coding: utf-8 -*-
"""Pure Python Inspector implementation.

This is a fallback when Rust bindings are not available.
Uses playwright for CDP operations.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Dict, List, Optional

if TYPE_CHECKING:
    from playwright.sync_api import Page as PlaywrightPage


@dataclass
class RefInfo:
    """Element reference info."""

    ref_id: str
    role: str
    name: str
    description: str = ""
    selector: str = ""
    backend_node_id: Optional[int] = None

    def __str__(self) -> str:
        desc = f" - {self.description}" if self.description else ""
        return f'{self.ref_id} [{self.role}] "{self.name}"{desc}'


@dataclass
class ActionResult:
    """Action result with before/after context."""

    success: bool
    action: str
    before: str = ""
    after: str = ""
    changes: List[str] = field(default_factory=list)
    error: Optional[str] = None
    duration_ms: int = 0

    def __str__(self) -> str:
        if self.success:
            changes = ", ".join(self.changes) if self.changes else "none"
            return f"✓ {self.action}\n  Changes: {changes}"
        return f"✗ {self.action}\n  Error: {self.error or 'unknown'}"

    def __bool__(self) -> bool:
        return self.success


@dataclass
class Snapshot:
    """Page snapshot with accessibility tree and refs."""

    title: str
    url: str
    viewport: tuple
    refs: Dict[str, RefInfo] = field(default_factory=dict)
    tree: str = ""

    def ref_count(self) -> int:
        """Get ref count."""
        return len(self.refs)

    def find(self, text: str) -> List[RefInfo]:
        """Find refs containing text (case-insensitive)."""
        text_lower = text.lower()
        return [
            r
            for r in self.refs.values()
            if text_lower in r.name.lower() or text_lower in r.description.lower()
        ]

    def get_ref(self, ref_id: str) -> Optional[RefInfo]:
        """Get ref by ID (accepts '@3' or '3')."""
        normalized = ref_id if ref_id.startswith("@") else f"@{ref_id}"
        return self.refs.get(normalized)

    def to_text(self) -> str:
        """Format as AI-friendly text."""
        lines = [
            f'Page: "{self.title}" ({self.url})',
            f"Viewport: {self.viewport[0]}x{self.viewport[1]}",
            "",
            f"Interactive Elements ({len(self.refs)} refs):",
        ]

        # Sort refs by numeric ID
        sorted_refs = sorted(
            self.refs.values(),
            key=lambda r: int(r.ref_id.lstrip("@")) if r.ref_id.lstrip("@").isdigit() else 999999,
        )

        for ref in sorted_refs:
            desc = f" - {ref.description}" if ref.description else ""
            lines.append(f'  {ref.ref_id}  [{ref.role}] "{ref.name}"{desc}')

        if self.tree:
            lines.extend(["", "Page Structure:"])
            for line in self.tree.split("\n"):
                lines.append(f"  {line}")

        return "\n".join(lines)

    def to_json(self) -> str:
        """Format as JSON."""
        return json.dumps(
            {
                "title": self.title,
                "url": self.url,
                "viewport": list(self.viewport),
                "refs": {
                    k: {
                        "ref_id": v.ref_id,
                        "role": v.role,
                        "name": v.name,
                        "description": v.description,
                        "selector": v.selector,
                    }
                    for k, v in self.refs.items()
                },
                "tree": self.tree,
            },
            indent=2,
        )

    def __str__(self) -> str:
        return self.to_text()


class Inspector:
    """AI-friendly WebView inspector and automation tool.

    Connect to a running AuroraView instance via CDP and interact
    with the page using refs from accessibility snapshots.

    Example:
        >>> inspector = Inspector.connect("http://localhost:9222")
        >>> print(inspector.snapshot())
        >>> inspector.click("@3")
        >>> inspector.close()
    """

    def __init__(self, page: "PlaywrightPage"):
        """Initialize with Playwright page.

        Use Inspector.connect() to create instances.
        """
        self._page = page
        self._refs: Dict[str, RefInfo] = {}

    @classmethod
    def connect(cls, endpoint: str) -> "Inspector":
        """Connect to CDP endpoint.

        Args:
            endpoint: CDP HTTP endpoint (e.g., "http://localhost:9222")

        Returns:
            Inspector instance
        """
        try:
            from playwright.sync_api import sync_playwright
        except ImportError as e:
            raise ImportError(
                "playwright is required for Python Inspector. "
                "Install with: pip install playwright && playwright install chromium"
            ) from e

        # Get WebSocket URL from CDP endpoint
        import urllib.request

        targets_url = f"{endpoint.rstrip('/')}/json"
        with urllib.request.urlopen(targets_url) as resp:
            targets = json.loads(resp.read().decode())

        # Find page target
        page_target = next(
            (t for t in targets if t.get("type") == "page"),
            None,
        )
        if not page_target:
            raise RuntimeError("No page target found at CDP endpoint")

        ws_url = page_target.get("webSocketDebuggerUrl")
        if not ws_url:
            raise RuntimeError("No WebSocket URL in target info")

        # Connect via Playwright
        pw = sync_playwright().start()
        browser = pw.chromium.connect_over_cdp(endpoint)
        contexts = browser.contexts
        if not contexts:
            raise RuntimeError("No browser contexts found")

        pages = contexts[0].pages
        if not pages:
            raise RuntimeError("No pages found in context")

        inspector = cls(pages[0])
        inspector._pw = pw
        inspector._browser = browser
        return inspector

    def snapshot(self) -> Snapshot:
        """Get page snapshot with accessibility tree and refs.

        Returns:
            Snapshot with page info, refs, and structure
        """
        # Get page info
        title = self._page.title()
        url = self._page.url
        viewport = self._page.viewport_size or {"width": 1280, "height": 720}

        # Get accessibility snapshot
        snapshot_result = self._page.accessibility.snapshot()

        # Process into refs
        refs: Dict[str, RefInfo] = {}
        tree_lines: List[str] = []
        ref_counter = [1]

        def process_node(node: dict, depth: int = 0) -> None:
            role = node.get("role", "")
            name = node.get("name", "")

            # Check if interactive
            interactive_roles = {
                "button",
                "link",
                "textbox",
                "searchbox",
                "checkbox",
                "radio",
                "combobox",
                "listbox",
                "option",
                "menuitem",
                "tab",
                "switch",
                "slider",
                "spinbutton",
                "treeitem",
            }

            indent = "  " * depth

            if role.lower() in interactive_roles and name:
                ref_id = f"@{ref_counter[0]}"
                ref_counter[0] += 1

                ref_info = RefInfo(
                    ref_id=ref_id,
                    role=role,
                    name=name,
                    description=node.get("description", ""),
                )
                refs[ref_id] = ref_info
                tree_lines.append(f'{indent}[{role} {ref_id}] "{name}"')
            elif name:
                tree_lines.append(f'{indent}{role}: "{name}"')

            # Process children
            for child in node.get("children", []):
                process_node(child, depth + 1)

        if snapshot_result:
            process_node(snapshot_result)

        self._refs = refs

        return Snapshot(
            title=title,
            url=url,
            viewport=(viewport["width"], viewport["height"]),
            refs=refs,
            tree="\n".join(tree_lines),
        )

    def screenshot(self, path: Optional[str] = None) -> bytes:
        """Take screenshot.

        Args:
            path: Optional file path to save screenshot

        Returns:
            PNG bytes
        """
        return self._page.screenshot(path=path, type="png")

    def click(self, ref_id: str) -> ActionResult:
        """Click element by ref.

        Args:
            ref_id: Ref ID (e.g., "@3" or "3")
        """
        import time

        start = time.time()
        normalized = ref_id if str(ref_id).startswith("@") else f"@{ref_id}"

        ref_info = self._refs.get(normalized)
        if not ref_info:
            return ActionResult(
                success=False,
                action=f"click {normalized}",
                error=f"ref {normalized} not found",
            )

        try:
            # Use role and name locator
            locator = self._page.get_by_role(ref_info.role, name=ref_info.name)
            locator.first.click()

            return ActionResult(
                success=True,
                action=f"click {normalized}",
                changes=[f"{normalized} clicked"],
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action=f"click {normalized}",
                error=str(e),
            )

    def fill(self, ref_id: str, text: str) -> ActionResult:
        """Fill input by ref.

        Args:
            ref_id: Ref ID of input element
            text: Text to fill
        """
        import time

        start = time.time()
        normalized = ref_id if str(ref_id).startswith("@") else f"@{ref_id}"

        ref_info = self._refs.get(normalized)
        if not ref_info:
            return ActionResult(
                success=False,
                action=f'fill {normalized} "{text}"',
                error=f"ref {normalized} not found",
            )

        try:
            locator = self._page.get_by_role(ref_info.role, name=ref_info.name)
            locator.first.fill(text)

            return ActionResult(
                success=True,
                action=f'fill {normalized} "{text}"',
                changes=[f"{normalized} filled"],
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action=f'fill {normalized} "{text}"',
                error=str(e),
            )

    def press(self, key: str) -> ActionResult:
        """Press a key.

        Args:
            key: Key to press (e.g., "Enter", "Tab", "Escape")
        """
        import time

        start = time.time()

        try:
            self._page.keyboard.press(key)
            return ActionResult(
                success=True,
                action=f"press {key}",
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action=f"press {key}",
                error=str(e),
            )

    def scroll(self, direction: str = "down", amount: int = 300) -> ActionResult:
        """Scroll page.

        Args:
            direction: "up", "down", "left", or "right"
            amount: Scroll amount in pixels
        """
        import time

        start = time.time()

        delta_map = {
            "up": (0, -amount),
            "down": (0, amount),
            "left": (-amount, 0),
            "right": (amount, 0),
        }

        delta = delta_map.get(direction.lower())
        if not delta:
            return ActionResult(
                success=False,
                action=f"scroll {direction}",
                error=f"Invalid direction: {direction}",
            )

        try:
            self._page.mouse.wheel(delta[0], delta[1])
            return ActionResult(
                success=True,
                action=f"scroll {direction} {amount}",
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action=f"scroll {direction}",
                error=str(e),
            )

    def goto(self, url: str) -> ActionResult:
        """Navigate to URL."""
        import time

        start = time.time()

        try:
            self._page.goto(url, wait_until="domcontentloaded")
            return ActionResult(
                success=True,
                action=f"goto {url}",
                changes=["page navigated"],
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action=f"goto {url}",
                error=str(e),
            )

    def back(self) -> ActionResult:
        """Go back."""
        import time

        start = time.time()

        try:
            self._page.go_back()
            return ActionResult(
                success=True,
                action="back",
                changes=["navigated back"],
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action="back",
                error=str(e),
            )

    def forward(self) -> ActionResult:
        """Go forward."""
        import time

        start = time.time()

        try:
            self._page.go_forward()
            return ActionResult(
                success=True,
                action="forward",
                changes=["navigated forward"],
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action="forward",
                error=str(e),
            )

    def reload(self) -> ActionResult:
        """Reload page."""
        import time

        start = time.time()

        try:
            self._page.reload()
            return ActionResult(
                success=True,
                action="reload",
                changes=["page reloaded"],
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as e:
            return ActionResult(
                success=False,
                action="reload",
                error=str(e),
            )

    def text(self, ref_id: str) -> str:
        """Get element text."""
        normalized = ref_id if str(ref_id).startswith("@") else f"@{ref_id}"
        ref_info = self._refs.get(normalized)
        if not ref_info:
            return ""

        try:
            locator = self._page.get_by_role(ref_info.role, name=ref_info.name)
            return locator.first.text_content() or ""
        except Exception:
            return ref_info.name

    def value(self, ref_id: str) -> str:
        """Get input value."""
        normalized = ref_id if str(ref_id).startswith("@") else f"@{ref_id}"
        ref_info = self._refs.get(normalized)
        if not ref_info:
            return ""

        try:
            locator = self._page.get_by_role(ref_info.role, name=ref_info.name)
            return locator.first.input_value()
        except Exception:
            return ""

    def eval(self, script: str) -> Any:
        """Execute JavaScript."""
        return self._page.evaluate(script)

    def wait(self, condition: str, timeout: Optional[float] = None) -> bool:
        """Wait for condition.

        Args:
            condition: Condition string (e.g., "text:Welcome", "ref:@5", "idle")
            timeout: Optional timeout in seconds (default: 30)

        Returns:
            True if condition met, False if timeout
        """
        timeout_ms = int((timeout or 30) * 1000)

        try:
            if condition == "idle":
                self._page.wait_for_load_state("networkidle", timeout=timeout_ms)
                return True

            if condition == "loaded":
                self._page.wait_for_load_state("domcontentloaded", timeout=timeout_ms)
                return True

            if condition.startswith("text:"):
                text = condition[5:]
                self._page.wait_for_selector(f"text={text}", timeout=timeout_ms)
                return True

            if condition.startswith("url:"):
                pattern = condition[4:]
                self._page.wait_for_url(re.compile(pattern.replace("*", ".*")), timeout=timeout_ms)
                return True

            if condition.startswith("ref:"):
                # Wait for ref to appear in snapshot
                import time

                start = time.time()
                while time.time() - start < (timeout or 30):
                    snap = self.snapshot()
                    ref_id = condition[4:]
                    if snap.get_ref(ref_id):
                        return True
                    time.sleep(0.1)
                return False

            # Default: treat as text
            self._page.wait_for_selector(f"text={condition}", timeout=timeout_ms)
            return True

        except Exception:
            return False

    @property
    def url(self) -> str:
        """Current URL."""
        return self._page.url

    @property
    def title(self) -> str:
        """Current title."""
        return self._page.title()

    def close(self) -> None:
        """Close connection."""
        if hasattr(self, "_browser"):
            self._browser.close()
        if hasattr(self, "_pw"):
            self._pw.stop()

    def __enter__(self) -> "Inspector":
        return self

    def __exit__(self, *args: Any) -> None:
        self.close()
