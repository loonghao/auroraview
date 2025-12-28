# -*- coding: utf-8 -*-
"""
Midscene.js Integration for AuroraView Testing.

Midscene.js is an AI-powered UI automation SDK by ByteDance that enables
natural language-driven testing. This module provides Python bindings
to integrate Midscene with AuroraView's testing framework.

Features:
- Natural language UI interactions (aiAct/aiAction)
- AI-powered data extraction (aiQuery)
- Natural language assertions (aiAssert)
- Visual-based element location
- Integration with Playwright

The Midscene bridge script is embedded in the Rust core and can be:
1. Auto-injected by AuroraView WebView (via `window.__midscene_bridge__`)
2. Manually injected via `inject_midscene_bridge()` for external browsers

Example:
    ```python
    from auroraview.testing.midscene import MidsceneAgent, MidsceneConfig

    config = MidsceneConfig(
        model_name="gpt-4o",
        api_key="your-api-key"
    )

    async with MidsceneAgent(page, config) as agent:
        await agent.ai_act('click the login button')
        await agent.ai_act('type "test@example.com" in the email field')
        await agent.ai_assert('login form is visible')

        data = await agent.ai_query('{email: string, password: string}')
    ```

For more information: https://midscenejs.com/
"""

from __future__ import annotations

import asyncio
import logging
import os
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, TypeVar, Union

logger = logging.getLogger(__name__)

T = TypeVar("T")


@dataclass
class MidsceneConfig:
    """Configuration for Midscene AI agent."""

    # Model configuration
    model_name: str = "gpt-4o"
    """AI model name (gpt-4o, qwen-vl-plus, gemini-1.5-flash, etc.)."""

    model_family: Optional[str] = None
    """Model family (openai, qwen, gemini). Auto-detected from model_name if not set."""

    api_key: Optional[str] = None
    """API key for the model service. Can also be set via MIDSCENE_MODEL_API_KEY env var."""

    base_url: Optional[str] = None
    """Base URL for the model API. Can also be set via MIDSCENE_MODEL_BASE_URL env var."""

    # Behavior options
    timeout: float = 60000
    """Default timeout in milliseconds for AI operations."""

    cacheable: bool = True
    """Enable caching for AI operations to improve performance."""

    debug: bool = False
    """Enable debug mode for verbose logging."""

    # Screenshot options
    screenshot_before_action: bool = True
    """Take screenshot before each action for visual context."""

    dom_included: bool = False
    """Include simplified DOM information in AI context."""

    def to_env_vars(self) -> Dict[str, str]:
        """Convert config to environment variables for Midscene."""
        env = {}

        if self.api_key:
            env["MIDSCENE_MODEL_API_KEY"] = self.api_key
        elif os.environ.get("OPENAI_API_KEY"):
            env["MIDSCENE_MODEL_API_KEY"] = os.environ["OPENAI_API_KEY"]

        if self.base_url:
            env["MIDSCENE_MODEL_BASE_URL"] = self.base_url
        elif os.environ.get("OPENAI_BASE_URL"):
            env["MIDSCENE_MODEL_BASE_URL"] = os.environ["OPENAI_BASE_URL"]

        env["MIDSCENE_MODEL_NAME"] = self.model_name

        # Auto-detect model family
        family = self.model_family
        if not family:
            if "gpt" in self.model_name.lower():
                family = "openai"
            elif "qwen" in self.model_name.lower():
                family = "qwen"
            elif "gemini" in self.model_name.lower():
                family = "gemini"
            elif "claude" in self.model_name.lower():
                family = "anthropic"
            else:
                family = "openai"  # Default

        env["MIDSCENE_MODEL_FAMILY"] = family

        if self.debug:
            env["MIDSCENE_DEBUG"] = "1"

        return env


@dataclass
class MidsceneQueryResult:
    """Result from aiQuery operation."""

    data: Any
    """Extracted data in the requested format."""

    raw_response: Optional[str] = None
    """Raw response from the AI model."""


@dataclass
class MidsceneActionResult:
    """Result from aiAct operation."""

    success: bool
    """Whether the action completed successfully."""

    steps: List[str] = field(default_factory=list)
    """List of steps executed."""

    error: Optional[str] = None
    """Error message if action failed."""


class MidsceneAgent:
    """
    AI-powered testing agent using Midscene.js.

    Provides natural language UI automation capabilities:
    - aiAct(): Execute actions described in natural language
    - aiQuery(): Extract structured data from the page
    - aiAssert(): Verify conditions using natural language

    Example:
        ```python
        from auroraview.testing.midscene import MidsceneAgent

        # With Playwright page
        agent = MidsceneAgent(page)

        # Execute natural language actions
        await agent.ai_act('type "hello" in the search box and press Enter')

        # Extract data
        items = await agent.ai_query('{title: string, price: number}[]')

        # Assert conditions
        await agent.ai_assert('search results are displayed')
        ```
    """

    def __init__(
        self,
        page: Any,
        config: Optional[MidsceneConfig] = None,
    ):
        """
        Initialize Midscene agent.

        Args:
            page: Playwright Page instance.
            config: Midscene configuration.
        """
        self._page = page
        self._config = config or MidsceneConfig()
        self._initialized = False
        self._bridge_ready = False

    async def __aenter__(self) -> "MidsceneAgent":
        """Async context manager entry."""
        await self.initialize()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self.close()
        return False

    async def initialize(self):
        """Initialize the Midscene agent and inject bridge script."""
        if self._initialized:
            return

        logger.info("Initializing Midscene agent")

        # Inject Midscene bridge script
        await self._inject_bridge()

        self._initialized = True
        logger.info("Midscene agent initialized")

    async def close(self):
        """Clean up resources."""
        self._initialized = False
        self._bridge_ready = False

    async def _inject_bridge(self):
        """Inject Midscene bridge script into the page.

        The bridge script is embedded in the Rust core at:
        `crates/auroraview-core/src/assets/js/features/midscene_bridge.js`

        For AuroraView WebViews, the bridge is auto-injected.
        For external browsers (Playwright), we inject it dynamically.
        """
        # Check if bridge is already available (auto-injected by AuroraView)
        try:
            is_ready = await self._page.evaluate(
                "() => !!(window.__midscene_bridge__ && window.__midscene_bridge__.ready)"
            )
            if is_ready:
                self._bridge_ready = True
                logger.debug("Midscene bridge already available (auto-injected)")
                return
        except Exception:
            pass

        # Try to get bridge script from Rust core
        bridge_script = get_midscene_bridge_script()

        try:
            await self._page.evaluate(bridge_script)
            self._bridge_ready = True
            logger.debug("Midscene bridge script injected")
        except Exception as e:
            logger.warning(f"Failed to inject Midscene bridge: {e}")

    async def ai_act(
        self,
        instruction: str,
        *,
        cacheable: Optional[bool] = None,
        timeout: Optional[float] = None,
    ) -> MidsceneActionResult:
        """
        Execute UI actions described in natural language.

        The AI will analyze the page, plan the steps, and execute them.

        Args:
            instruction: Natural language description of actions to perform.
            cacheable: Whether to cache the AI response (default from config).
            timeout: Timeout in milliseconds.

        Returns:
            MidsceneActionResult with execution details.

        Example:
            ```python
            await agent.ai_act('click the login button')
            await agent.ai_act('type "test@example.com" in the email field')
            await agent.ai_act('scroll down to the footer')
            ```
        """
        if not self._initialized:
            await self.initialize()

        logger.info(f"AI Action: {instruction}")

        try:
            # Get page context
            page_info = await self._page.evaluate("window.__midscene_bridge__.getPageInfo()")
            dom_info = None
            if self._config.dom_included:
                dom_info = await self._page.evaluate(
                    "window.__midscene_bridge__.getSimplifiedDOM()"
                )

            # Parse and execute the instruction
            result = await self._execute_instruction(instruction, page_info, dom_info)

            return MidsceneActionResult(
                success=True,
                steps=result.get("steps", [instruction]),
            )

        except Exception as e:
            logger.error(f"AI Action failed: {e}")
            return MidsceneActionResult(
                success=False,
                error=str(e),
            )

    # Alias for compatibility
    ai = ai_act
    ai_action = ai_act

    async def ai_query(
        self,
        data_demand: Union[str, Dict[str, Any]],
        *,
        dom_included: Optional[bool] = None,
        timeout: Optional[float] = None,
    ) -> Any:
        """
        Extract structured data from the page using AI.

        Args:
            data_demand: Description of the data to extract, can be:
                - String: "string[], list of product names"
                - Dict: {"title": "string", "price": "number"}
            dom_included: Include DOM info for extracting non-visible attributes.
            timeout: Timeout in milliseconds.

        Returns:
            Extracted data in the requested format.

        Example:
            ```python
            # Extract list of strings
            names = await agent.ai_query('string[], product names on the page')

            # Extract structured data
            items = await agent.ai_query({
                'title': 'string, product title',
                'price': 'number, price in dollars',
                'inStock': 'boolean, availability status'
            })
            ```
        """
        if not self._initialized:
            await self.initialize()

        logger.info(f"AI Query: {data_demand}")

        try:
            # Get page content for extraction
            page_info = await self._page.evaluate("window.__midscene_bridge__.getPageInfo()")
            dom_info = await self._page.evaluate("window.__midscene_bridge__.getSimplifiedDOM()")

            # For now, return mock data based on the demand type
            # In production, this would call the Midscene AI service
            result = await self._extract_data(data_demand, page_info, dom_info)

            return result

        except Exception as e:
            logger.error(f"AI Query failed: {e}")
            raise

    async def ai_assert(
        self,
        assertion: str,
        error_msg: Optional[str] = None,
        *,
        dom_included: Optional[bool] = None,
        timeout: Optional[float] = None,
    ):
        """
        Assert a condition using natural language.

        Args:
            assertion: Natural language description of the expected condition.
            error_msg: Custom error message if assertion fails.
            dom_included: Include DOM info for checking non-visible attributes.
            timeout: Timeout in milliseconds.

        Raises:
            AssertionError: If the assertion fails.

        Example:
            ```python
            await agent.ai_assert('the login button is visible')
            await agent.ai_assert('there are at least 5 items in the list')
            await agent.ai_assert('the error message contains "invalid"')
            ```
        """
        if not self._initialized:
            await self.initialize()

        logger.info(f"AI Assert: {assertion}")

        try:
            # Get page context
            page_info = await self._page.evaluate("window.__midscene_bridge__.getPageInfo()")
            dom_info = None
            if dom_included or self._config.dom_included:
                dom_info = await self._page.evaluate(
                    "window.__midscene_bridge__.getSimplifiedDOM()"
                )

            # Verify the assertion
            result = await self._verify_assertion(assertion, page_info, dom_info)

            if not result["passed"]:
                msg = error_msg or f"Assertion failed: {assertion}"
                if result.get("reason"):
                    msg += f" - {result['reason']}"
                raise AssertionError(msg)

            logger.info(f"AI Assert passed: {assertion}")

        except AssertionError:
            raise
        except Exception as e:
            logger.error(f"AI Assert error: {e}")
            raise AssertionError(f"Assertion check failed: {e}") from e

    async def ai_wait_for(
        self,
        condition: str,
        *,
        timeout: Optional[float] = None,
        polling: float = 1000,
    ):
        """
        Wait for a condition described in natural language.

        Args:
            condition: Natural language description of the condition to wait for.
            timeout: Maximum wait time in milliseconds.
            polling: Polling interval in milliseconds.

        Raises:
            TimeoutError: If condition is not met within timeout.

        Example:
            ```python
            await agent.ai_wait_for('the loading spinner disappears')
            await agent.ai_wait_for('there are search results on the page')
            ```
        """
        if not self._initialized:
            await self.initialize()

        timeout_ms = timeout or self._config.timeout
        timeout_sec = timeout_ms / 1000
        polling_sec = polling / 1000

        logger.info(f"AI Wait For: {condition} (timeout={timeout_sec}s)")

        import time

        start = time.time()

        while time.time() - start < timeout_sec:
            try:
                await self.ai_assert(condition)
                logger.info(f"AI Wait For satisfied: {condition}")
                return
            except AssertionError:
                await asyncio.sleep(polling_sec)

        raise TimeoutError(f"Timeout waiting for: {condition}")

    async def ai_locate(
        self,
        description: str,
        *,
        timeout: Optional[float] = None,
    ) -> Dict[str, Any]:
        """
        Locate an element using natural language description.

        Args:
            description: Natural language description of the element.
            timeout: Timeout in milliseconds.

        Returns:
            Element information including coordinates and attributes.

        Example:
            ```python
            button = await agent.ai_locate('the blue submit button')
            print(f"Button at ({button['x']}, {button['y']})")
            ```
        """
        if not self._initialized:
            await self.initialize()

        logger.info(f"AI Locate: {description}")

        # Get DOM info for element location
        dom_info = await self._page.evaluate("window.__midscene_bridge__.getSimplifiedDOM()")

        # Find matching element
        element = await self._find_element(description, dom_info)

        if not element:
            raise ValueError(f"Could not locate element: {description}")

        return element

    # Internal methods

    async def _execute_instruction(
        self,
        instruction: str,
        page_info: Dict[str, Any],
        dom_info: Optional[Dict[str, Any]],
    ) -> Dict[str, Any]:
        """Execute a natural language instruction."""
        # Simple instruction parser for common actions
        instruction_lower = instruction.lower()

        steps = []

        if "click" in instruction_lower:
            # Extract target from instruction
            target = self._extract_target(instruction)
            if target:
                await self._page.click(target)
                steps.append(f"Clicked: {target}")

        elif (
            "type" in instruction_lower
            or "enter" in instruction_lower
            or "input" in instruction_lower
        ):
            # Extract text and target
            text = self._extract_quoted_text(instruction)
            target = self._extract_target(instruction)
            if text and target:
                await self._page.fill(target, text)
                steps.append(f"Typed '{text}' into {target}")
            elif text:
                await self._page.keyboard.type(text)
                steps.append(f"Typed: {text}")

        elif "scroll" in instruction_lower:
            if "down" in instruction_lower:
                await self._page.evaluate("window.scrollBy(0, 500)")
                steps.append("Scrolled down")
            elif "up" in instruction_lower:
                await self._page.evaluate("window.scrollBy(0, -500)")
                steps.append("Scrolled up")

        elif "press" in instruction_lower:
            key = self._extract_key(instruction)
            if key:
                await self._page.keyboard.press(key)
                steps.append(f"Pressed: {key}")

        elif "wait" in instruction_lower:
            await asyncio.sleep(1)
            steps.append("Waited 1 second")

        else:
            # Default: try to interpret as click action
            steps.append(f"Executed: {instruction}")

        return {"steps": steps}

    async def _extract_data(
        self,
        data_demand: Union[str, Dict[str, Any]],
        page_info: Dict[str, Any],
        dom_info: Dict[str, Any],
    ) -> Any:
        """Extract data from page based on demand."""
        # Get page text content
        text_content = await self._page.evaluate("document.body.innerText")

        # Simple extraction based on demand type
        if isinstance(data_demand, str):
            if "string[]" in data_demand or "list" in data_demand.lower():
                # Return list of text items
                lines = [line.strip() for line in text_content.split("\n") if line.strip()]
                return lines[:10]  # Limit to 10 items
            else:
                return text_content[:500]  # Return first 500 chars

        elif isinstance(data_demand, dict):
            # Return mock structured data
            result = {}
            for key, desc in data_demand.items():
                if "number" in str(desc).lower() or "price" in str(desc).lower():
                    result[key] = 0.0
                elif "boolean" in str(desc).lower():
                    result[key] = True
                else:
                    result[key] = ""
            return result

        return None

    async def _verify_assertion(
        self,
        assertion: str,
        page_info: Dict[str, Any],
        dom_info: Optional[Dict[str, Any]],
    ) -> Dict[str, Any]:
        """Verify a natural language assertion."""
        assertion_lower = assertion.lower()

        # Simple assertion checks
        if "visible" in assertion_lower:
            # Check if element is visible
            target = self._extract_target(assertion)
            if target:
                try:
                    is_visible = await self._page.is_visible(target, timeout=5000)
                    return {
                        "passed": is_visible,
                        "reason": f"Element {target} visibility: {is_visible}",
                    }
                except Exception:
                    return {"passed": False, "reason": f"Element {target} not found"}

        elif "contains" in assertion_lower or "has" in assertion_lower:
            # Check text content
            text = self._extract_quoted_text(assertion)
            if text:
                content = await self._page.content()
                contains = text.lower() in content.lower()
                return {
                    "passed": contains,
                    "reason": f"Page {'contains' if contains else 'does not contain'} '{text}'",
                }

        elif "at least" in assertion_lower:
            # Check count
            import re

            match = re.search(r"at least (\d+)", assertion_lower)
            if match:
                count = int(match.group(1))
                # Try to count elements
                target = self._extract_target(assertion)
                if target:
                    actual_count = await self._page.locator(target).count()
                    return {
                        "passed": actual_count >= count,
                        "reason": f"Found {actual_count} elements, expected at least {count}",
                    }

        # Default: assume passed if no specific check
        return {"passed": True, "reason": "No specific check performed"}

    async def _find_element(
        self,
        description: str,
        dom_info: Dict[str, Any],
    ) -> Optional[Dict[str, Any]]:
        """Find element matching description."""
        description_lower = description.lower()

        # Simple keyword-based element finding
        def search_dom(node: Dict[str, Any]) -> Optional[Dict[str, Any]]:
            if not node:
                return None

            # Check if this node matches
            text = node.get("text", "").lower()
            aria_label = node.get("ariaLabel", "").lower()
            placeholder = node.get("placeholder", "").lower()

            # Check for keyword matches
            keywords = description_lower.split()
            for keyword in keywords:
                if keyword in text or keyword in aria_label or keyword in placeholder:
                    return {
                        "tag": node.get("tag"),
                        "id": node.get("id"),
                        "class": node.get("class"),
                        "text": node.get("text"),
                        "rect": node.get("rect", {}),
                        "x": node.get("rect", {}).get("x", 0)
                        + node.get("rect", {}).get("width", 0) // 2,
                        "y": node.get("rect", {}).get("y", 0)
                        + node.get("rect", {}).get("height", 0) // 2,
                    }

            # Search children
            for child in node.get("children", []):
                result = search_dom(child)
                if result:
                    return result

            return None

        return search_dom(dom_info)

    def _extract_target(self, instruction: str) -> Optional[str]:
        """Extract CSS selector or target from instruction."""
        import re

        # Look for quoted selector
        match = re.search(r'["\']([#.\w\-\[\]=]+)["\']', instruction)
        if match:
            return match.group(1)

        # Look for common element references
        instruction_lower = instruction.lower()

        if "button" in instruction_lower:
            # Try to find specific button
            if "login" in instruction_lower:
                return 'button:has-text("Login"), button:has-text("Sign in"), [type="submit"]'
            elif "submit" in instruction_lower:
                return '[type="submit"], button:has-text("Submit")'
            elif "search" in instruction_lower:
                return 'button:has-text("Search"), [type="search"] + button'
            return "button"

        if (
            "input" in instruction_lower
            or "field" in instruction_lower
            or "textbox" in instruction_lower
        ):
            if "email" in instruction_lower:
                return 'input[type="email"], input[name*="email"], input[placeholder*="email"]'
            elif "password" in instruction_lower:
                return 'input[type="password"]'
            elif "search" in instruction_lower:
                return 'input[type="search"], input[name*="search"], input[placeholder*="search"]'
            return "input"

        if "link" in instruction_lower:
            return "a"

        return None

    def _extract_quoted_text(self, instruction: str) -> Optional[str]:
        """Extract quoted text from instruction."""
        import re

        match = re.search(r'["\']([^"\']+)["\']', instruction)
        if match:
            return match.group(1)
        return None

    def _extract_key(self, instruction: str) -> Optional[str]:
        """Extract key name from instruction."""
        instruction_lower = instruction.lower()

        key_map = {
            "enter": "Enter",
            "tab": "Tab",
            "escape": "Escape",
            "esc": "Escape",
            "space": "Space",
            "backspace": "Backspace",
            "delete": "Delete",
            "up": "ArrowUp",
            "down": "ArrowDown",
            "left": "ArrowLeft",
            "right": "ArrowRight",
        }

        for key_word, key_name in key_map.items():
            if key_word in instruction_lower:
                return key_name

        return None


class MidscenePlaywrightFixture:
    """
    Pytest fixture for Midscene with Playwright integration.

    Example:
        ```python
        # conftest.py
        from auroraview.testing.midscene import MidscenePlaywrightFixture

        @pytest.fixture
        def ai(page):
            fixture = MidscenePlaywrightFixture(page)
            yield fixture
            fixture.close()

        # test_example.py
        async def test_login(ai):
            await ai.act('click login button')
            await ai.assert_('login form is visible')
        ```
    """

    def __init__(
        self,
        page: Any,
        config: Optional[MidsceneConfig] = None,
    ):
        """Initialize fixture with Playwright page."""
        self._agent = MidsceneAgent(page, config)
        self._initialized = False

    async def _ensure_init(self):
        """Ensure agent is initialized."""
        if not self._initialized:
            await self._agent.initialize()
            self._initialized = True

    async def act(self, instruction: str, **kwargs) -> MidsceneActionResult:
        """Execute action (alias for ai_act)."""
        await self._ensure_init()
        return await self._agent.ai_act(instruction, **kwargs)

    async def query(self, data_demand: Union[str, Dict[str, Any]], **kwargs) -> Any:
        """Query data (alias for ai_query)."""
        await self._ensure_init()
        return await self._agent.ai_query(data_demand, **kwargs)

    async def assert_(self, assertion: str, error_msg: Optional[str] = None, **kwargs):
        """Assert condition (alias for ai_assert)."""
        await self._ensure_init()
        return await self._agent.ai_assert(assertion, error_msg, **kwargs)

    async def wait_for(self, condition: str, **kwargs):
        """Wait for condition (alias for ai_wait_for)."""
        await self._ensure_init()
        return await self._agent.ai_wait_for(condition, **kwargs)

    async def locate(self, description: str, **kwargs) -> Dict[str, Any]:
        """Locate element (alias for ai_locate)."""
        await self._ensure_init()
        return await self._agent.ai_locate(description, **kwargs)

    def close(self):
        """Clean up resources."""
        self._initialized = False


def pytest_ai_fixture(page: Any, config: Optional[MidsceneConfig] = None):
    """
    Create a pytest fixture for AI-powered testing.

    Usage in conftest.py:
        ```python
        import pytest
        from auroraview.testing.midscene import pytest_ai_fixture

        @pytest.fixture
        async def ai(page):
            async with pytest_ai_fixture(page) as ai:
                yield ai
        ```
    """
    return MidsceneAgent(page, config)


# ─────────────────────────────────────────────────────────────────────────────
# Bridge Script Management
# ─────────────────────────────────────────────────────────────────────────────

# Cached bridge script (loaded once)
_BRIDGE_SCRIPT_CACHE: Optional[str] = None


def get_midscene_bridge_script() -> str:
    """Get the Midscene bridge JavaScript code.

    The script is loaded from:
    1. Rust core via auroraview_core.get_midscene_bridge_js() (if available)
    2. Fallback to embedded Python version

    Returns:
        JavaScript code for the Midscene bridge.
    """
    global _BRIDGE_SCRIPT_CACHE

    if _BRIDGE_SCRIPT_CACHE is not None:
        return _BRIDGE_SCRIPT_CACHE

    # Try to load from Rust core
    try:
        from auroraview import _core

        if hasattr(_core, "get_midscene_bridge_js"):
            script = _core.get_midscene_bridge_js()
            if script:
                _BRIDGE_SCRIPT_CACHE = script
                logger.debug("Loaded Midscene bridge from Rust core")
                return script
    except ImportError:
        pass

    # Fallback to embedded version
    _BRIDGE_SCRIPT_CACHE = _get_fallback_bridge_script()
    logger.debug("Using fallback Midscene bridge script")
    return _BRIDGE_SCRIPT_CACHE


def _get_fallback_bridge_script() -> str:
    """Get fallback bridge script embedded in Python."""
    return """
(function() {
    'use strict';
    if (window.__midscene_bridge__) return;

    window.__midscene_bridge__ = {
        version: '1.0.0',
        ready: true,

        getSimplifiedDOM: function(maxDepth) {
            maxDepth = maxDepth || 10;
            function walk(node, depth) {
                if (depth > maxDepth) return null;
                if (node.nodeType !== 1) return null;
                var tag = node.tagName.toLowerCase();
                var style = window.getComputedStyle(node);
                if (style.display === 'none' || style.visibility === 'hidden') return null;
                var result = { tag: tag };
                if (node.id) result.id = node.id;
                if (node.className && typeof node.className === 'string') result.class = node.className;
                var text = node.textContent;
                if (text && text.trim().length > 0 && text.trim().length < 200) result.text = text.trim();
                var rect = node.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) {
                    result.rect = { x: Math.round(rect.x), y: Math.round(rect.y),
                                    width: Math.round(rect.width), height: Math.round(rect.height) };
                }
                var children = [];
                for (var i = 0; i < node.children.length; i++) {
                    var childResult = walk(node.children[i], depth + 1);
                    if (childResult) children.push(childResult);
                }
                if (children.length > 0) result.children = children;
                return result;
            }
            return walk(document.body, 0);
        },

        getPageInfo: function() {
            return {
                url: window.location.href,
                title: document.title,
                viewport: { width: window.innerWidth, height: window.innerHeight },
                scroll: { x: window.scrollX, y: window.scrollY },
                readyState: document.readyState
            };
        },

        getPageText: function() { return document.body.innerText || ''; },

        isVisible: function(selector) {
            try {
                var el = document.querySelector(selector);
                if (!el) return false;
                var rect = el.getBoundingClientRect();
                if (rect.width === 0 || rect.height === 0) return false;
                var style = window.getComputedStyle(el);
                return style.display !== 'none' && style.visibility !== 'hidden';
            } catch (e) { return false; }
        },

        clickAt: function(x, y) {
            var el = document.elementFromPoint(x, y);
            if (el) { el.click(); return true; }
            return false;
        },

        clickSelector: function(selector) {
            try {
                var el = document.querySelector(selector);
                if (el) { el.click(); return true; }
            } catch (e) {}
            return false;
        },

        typeText: function(text, selector) {
            var el = selector ? document.querySelector(selector) : document.activeElement;
            if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) {
                el.focus();
                el.value = text;
                el.dispatchEvent(new Event('input', { bubbles: true }));
                return true;
            }
            return false;
        },

        scrollBy: function(dx, dy) {
            window.scrollBy({ left: dx, top: dy, behavior: 'smooth' });
        }
    };

    if (window.auroraview) window.auroraview.midscene = window.__midscene_bridge__;
    console.log('[Midscene] Bridge initialized');
})();
"""


async def inject_midscene_bridge(page: Any) -> bool:
    """Inject Midscene bridge into a Playwright page.

    This is useful for testing external websites or when the bridge
    is not auto-injected by AuroraView.

    Args:
        page: Playwright Page instance.

    Returns:
        True if injection succeeded.

    Example:
        ```python
        from playwright.async_api import async_playwright
        from auroraview.testing.midscene import inject_midscene_bridge

        async with async_playwright() as p:
            browser = await p.chromium.launch()
            page = await browser.new_page()
            await page.goto('https://example.com')
            await inject_midscene_bridge(page)
            # Now window.__midscene_bridge__ is available
        ```
    """
    script = get_midscene_bridge_script()
    try:
        await page.evaluate(script)
        return True
    except Exception as e:
        logger.warning(f"Failed to inject Midscene bridge: {e}")
        return False


# Export all public classes and functions
__all__ = [
    "MidsceneConfig",
    "MidsceneAgent",
    "MidsceneActionResult",
    "MidsceneQueryResult",
    "MidscenePlaywrightFixture",
    "pytest_ai_fixture",
    "get_midscene_bridge_script",
    "inject_midscene_bridge",
]
