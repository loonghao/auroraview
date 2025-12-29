# -*- coding: utf-8 -*-
"""AI-Enhanced Automation Testing for AuroraView.

This module provides AI-powered automation capabilities to handle
common testing issues like stuck loading screens, flaky tests,
and dynamic UI elements.

Key Features:
- Smart wait with AI-based condition detection
- Automatic recovery from stuck states
- Loading screen monitoring and diagnostics
- Self-healing selectors using AI

Example:
    ```python
    from auroraview.testing.ai_automation import AITestHelper

    async with AITestHelper(page) as helper:
        # Wait for app to be ready with smart detection
        await helper.wait_for_app_ready()

        # Smart click with self-healing selector
        await helper.smart_click("the login button")

        # Monitor and recover from stuck states
        await helper.with_recovery(
            lambda: page.click("#submit"),
            recovery_action=lambda: page.reload()
        )
    ```
"""

from __future__ import annotations

import asyncio
import logging
import time
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional, TypeVar

logger = logging.getLogger(__name__)

T = TypeVar("T")


@dataclass
class LoadingDiagnostics:
    """Diagnostics from the loading screen."""

    elapsed_ms: int = 0
    backend_ready: bool = False
    auroraview_available: bool = False
    send_event_available: bool = False
    trigger_available: bool = False
    loading_state: Optional[Dict[str, Any]] = None
    diagnostic_count: int = 0
    timestamp: float = field(default_factory=time.time)


@dataclass
class AITestConfig:
    """Configuration for AI-enhanced testing."""

    # Timeouts
    app_ready_timeout: float = 60.0
    """Timeout for waiting for app to be ready (seconds)."""

    loading_timeout: float = 30.0
    """Timeout for loading screen (seconds)."""

    action_timeout: float = 10.0
    """Default timeout for actions (seconds)."""

    # Retry settings
    max_retries: int = 3
    """Maximum number of retries for failed actions."""

    retry_delay: float = 1.0
    """Delay between retries (seconds)."""

    # AI settings
    use_ai_selectors: bool = True
    """Use AI-powered selector healing."""

    ai_model: str = "gpt-4o"
    """AI model for smart operations."""

    # Recovery settings
    auto_recover: bool = True
    """Automatically recover from stuck states."""

    force_navigate_on_timeout: bool = True
    """Force navigation if loading times out."""


class AITestHelper:
    """AI-enhanced test helper for AuroraView applications.

    Provides smart waiting, automatic recovery, and AI-powered
    interactions for more robust testing.
    """

    def __init__(
        self,
        page: Any,
        config: Optional[AITestConfig] = None,
    ):
        """Initialize AI test helper.

        Args:
            page: Playwright Page instance.
            config: Test configuration.
        """
        self._page = page
        self._config = config or AITestConfig()
        self._diagnostics_history: List[LoadingDiagnostics] = []
        self._midscene_agent = None

    async def __aenter__(self) -> "AITestHelper":
        """Async context manager entry."""
        await self._setup()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit."""
        await self._cleanup()
        return False

    async def _setup(self):
        """Setup helper resources."""
        # Listen for diagnostic events
        await self._page.expose_function(
            "__auroraview_diagnostic_handler",
            self._handle_diagnostic,
        )

        # Inject diagnostic listener
        await self._page.evaluate("""
            window.addEventListener('auroraview:loading_diagnostic', (e) => {
                window.__auroraview_diagnostic_handler(JSON.stringify(e.detail));
            });
            window.addEventListener('auroraview:loading_timeout', (e) => {
                window.__auroraview_diagnostic_handler(JSON.stringify({
                    ...e.detail.diagnostics,
                    timeout: true
                }));
            });
        """)

    async def _cleanup(self):
        """Cleanup resources."""
        pass

    def _handle_diagnostic(self, data_json: str):
        """Handle diagnostic event from loading screen."""
        import json

        try:
            data = json.loads(data_json)
            diag = LoadingDiagnostics(
                elapsed_ms=data.get("elapsed_ms", 0),
                backend_ready=data.get("backend_ready", False),
                auroraview_available=data.get("auroraview_available", False),
                send_event_available=data.get("send_event_available", False),
                trigger_available=data.get("trigger_available", False),
                loading_state=data.get("loading_state"),
                diagnostic_count=data.get("diagnostic_count", 0),
            )
            self._diagnostics_history.append(diag)
            logger.debug(
                f"Loading diagnostic: elapsed={diag.elapsed_ms}ms, ready={diag.backend_ready}"
            )
        except Exception as e:
            logger.warning(f"Failed to parse diagnostic: {e}")

    async def wait_for_app_ready(
        self,
        timeout: Optional[float] = None,
        force_on_timeout: bool = True,
    ) -> bool:
        """Wait for the application to be fully ready.

        This method handles the loading screen and waits for:
        1. Backend to be ready (Python initialized)
        2. AuroraView bridge to be available
        3. Main application to load

        Args:
            timeout: Maximum wait time in seconds.
            force_on_timeout: Force navigation if loading times out.

        Returns:
            True if app is ready, False if timed out.
        """
        timeout = timeout or self._config.app_ready_timeout
        start = time.time()

        logger.info(f"Waiting for app to be ready (timeout={timeout}s)")

        # Phase 1: Wait for loading screen or direct app
        try:
            # Check if we're on loading screen or main app
            is_loading = await self._page.evaluate("""
                () => {
                    // Check if loading screen is present
                    return !!(window.auroraLoading || document.querySelector('.loading-container'));
                }
            """)

            if is_loading:
                logger.info("Loading screen detected, waiting for backend_ready...")
                ready = await self._wait_for_backend_ready(timeout)

                if not ready and force_on_timeout:
                    logger.warning("Backend not ready, forcing navigation...")
                    await self._force_navigate()
                    # Wait a bit for navigation
                    await asyncio.sleep(2)

        except Exception as e:
            logger.warning(f"Error checking loading state: {e}")

        # Phase 2: Wait for AuroraView bridge
        remaining = timeout - (time.time() - start)
        if remaining > 0:
            try:
                await self._page.wait_for_function(
                    """
                    () => {
                        return typeof window.auroraview !== 'undefined' &&
                               typeof window.auroraview.api !== 'undefined';
                    }
                    """,
                    timeout=remaining * 1000,
                )
                logger.info("AuroraView bridge is ready")
                return True
            except Exception as e:
                logger.warning(f"Timeout waiting for AuroraView bridge: {e}")

        return False

    async def _wait_for_backend_ready(self, timeout: float) -> bool:
        """Wait for backend_ready event on loading screen."""
        start = time.time()

        while time.time() - start < timeout:
            try:
                is_ready = await self._page.evaluate("""
                    () => {
                        if (window.auroraLoading && window.auroraLoading.isReady) {
                            return window.auroraLoading.isReady();
                        }
                        if (window.__backendReady) {
                            return true;
                        }
                        // Check if we've navigated away from loading
                        return !document.querySelector('.loading-container');
                    }
                """)
                if is_ready:
                    return True
            except Exception:
                pass

            await asyncio.sleep(0.5)

        return False

    async def _force_navigate(self):
        """Force navigation from loading screen."""
        try:
            await self._page.evaluate("""
                () => {
                    if (window.__forceNavigate) {
                        window.__forceNavigate();
                    } else if (window.auroraview && window.auroraview.send_event) {
                        window.auroraview.send_event('navigate_to_app', {});
                    }
                }
            """)
        except Exception as e:
            logger.warning(f"Failed to force navigate: {e}")

    async def get_loading_diagnostics(self) -> Optional[LoadingDiagnostics]:
        """Get current loading diagnostics.

        Returns:
            LoadingDiagnostics if available, None otherwise.
        """
        try:
            data = await self._page.evaluate("""
                () => {
                    if (window.auroraLoading && window.auroraLoading.getDiagnostics) {
                        return window.auroraLoading.getDiagnostics();
                    }
                    return null;
                }
            """)
            if data:
                return LoadingDiagnostics(
                    elapsed_ms=data.get("elapsed_ms", 0),
                    backend_ready=data.get("backend_ready", False),
                    auroraview_available=data.get("auroraview_available", False),
                    send_event_available=data.get("send_event_available", False),
                    trigger_available=data.get("trigger_available", False),
                    loading_state=data.get("loading_state"),
                    diagnostic_count=data.get("diagnostic_count", 0),
                )
        except Exception as e:
            logger.debug(f"Failed to get diagnostics: {e}")
        return None

    async def with_recovery(
        self,
        action: Callable,
        recovery_action: Optional[Callable] = None,
        max_retries: Optional[int] = None,
        timeout: Optional[float] = None,
    ) -> Any:
        """Execute an action with automatic recovery on failure.

        Args:
            action: The action to execute (async callable).
            recovery_action: Action to perform on failure before retry.
            max_retries: Maximum number of retries.
            timeout: Timeout for the action.

        Returns:
            Result of the action.

        Raises:
            Exception: If all retries fail.
        """
        max_retries = max_retries or self._config.max_retries
        last_error = None

        for attempt in range(max_retries + 1):
            try:
                if asyncio.iscoroutinefunction(action):
                    return await action()
                else:
                    return action()
            except Exception as e:
                last_error = e
                logger.warning(f"Action failed (attempt {attempt + 1}/{max_retries + 1}): {e}")

                if attempt < max_retries:
                    # Execute recovery action
                    if recovery_action:
                        try:
                            if asyncio.iscoroutinefunction(recovery_action):
                                await recovery_action()
                            else:
                                recovery_action()
                        except Exception as re:
                            logger.warning(f"Recovery action failed: {re}")

                    await asyncio.sleep(self._config.retry_delay)

        raise last_error

    async def smart_click(
        self,
        target: str,
        timeout: Optional[float] = None,
    ):
        """Click on an element using smart selector resolution.

        If the target looks like a natural language description,
        uses AI to find the element. Otherwise, uses it as a CSS selector.

        Args:
            target: CSS selector or natural language description.
            timeout: Timeout for the click.
        """
        timeout = timeout or self._config.action_timeout

        # Check if target looks like a selector or description
        is_selector = target.startswith(("#", ".", "[")) or "::" in target

        if is_selector:
            await self._page.click(target, timeout=timeout * 1000)
        else:
            # Use AI to find element
            await self._ai_click(target, timeout)

    async def _ai_click(self, description: str, timeout: float):
        """Click using AI-powered element finding."""
        # Try to use Midscene agent if available
        if self._midscene_agent is None:
            try:
                from .midscene import MidsceneAgent

                self._midscene_agent = MidsceneAgent(self._page)
                await self._midscene_agent.initialize()
            except Exception as e:
                logger.warning(f"Midscene not available: {e}")

        if self._midscene_agent:
            await self._midscene_agent.ai_act(f"click on {description}")
        else:
            # Fallback: try common patterns
            selector = self._guess_selector(description)
            if selector:
                await self._page.click(selector, timeout=timeout * 1000)
            else:
                raise ValueError(f"Could not find element: {description}")

    def _guess_selector(self, description: str) -> Optional[str]:
        """Guess a CSS selector from a description."""
        desc_lower = description.lower()

        # Common patterns
        if "button" in desc_lower:
            if "login" in desc_lower or "sign in" in desc_lower:
                return 'button:has-text("Login"), button:has-text("Sign in"), [type="submit"]'
            if "submit" in desc_lower:
                return '[type="submit"], button:has-text("Submit")'
            if "run" in desc_lower:
                return 'button:has-text("Run")'
            if "stop" in desc_lower:
                return 'button:has-text("Stop")'
            return "button"

        if "search" in desc_lower:
            return 'input[type="search"], input[placeholder*="search" i], input[name*="search" i]'

        if "input" in desc_lower or "field" in desc_lower:
            if "email" in desc_lower:
                return 'input[type="email"], input[name*="email" i]'
            if "password" in desc_lower:
                return 'input[type="password"]'
            return "input"

        return None

    async def wait_for_stable(
        self,
        selector: str,
        stability_time: float = 0.5,
        timeout: Optional[float] = None,
    ):
        """Wait for an element to be stable (not changing).

        Useful for waiting for animations to complete or
        dynamic content to settle.

        Args:
            selector: CSS selector for the element.
            stability_time: Time element must be stable (seconds).
            timeout: Maximum wait time.
        """
        timeout = timeout or self._config.action_timeout
        start = time.time()

        last_state = None
        stable_since = None

        while time.time() - start < timeout:
            try:
                current_state = await self._page.evaluate(
                    f"""
                    () => {{
                        const el = document.querySelector('{selector}');
                        if (!el) return null;
                        const rect = el.getBoundingClientRect();
                        return {{
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height,
                            text: el.textContent?.substring(0, 100)
                        }};
                    }}
                """
                )

                if current_state == last_state:
                    if stable_since is None:
                        stable_since = time.time()
                    elif time.time() - stable_since >= stability_time:
                        return  # Element is stable
                else:
                    stable_since = None
                    last_state = current_state

            except Exception:
                stable_since = None

            await asyncio.sleep(0.1)

        raise TimeoutError(f"Element {selector} did not stabilize within {timeout}s")

    async def screenshot_on_failure(
        self,
        action: Callable,
        screenshot_path: str,
    ) -> Any:
        """Execute action and take screenshot on failure.

        Args:
            action: Action to execute.
            screenshot_path: Path to save screenshot on failure.

        Returns:
            Result of the action.
        """
        try:
            if asyncio.iscoroutinefunction(action):
                return await action()
            else:
                return action()
        except Exception as e:
            try:
                await self._page.screenshot(path=screenshot_path)
                logger.info(f"Failure screenshot saved: {screenshot_path}")
            except Exception as se:
                logger.warning(f"Failed to save screenshot: {se}")
            raise e

    def get_diagnostics_history(self) -> List[LoadingDiagnostics]:
        """Get history of loading diagnostics.

        Returns:
            List of LoadingDiagnostics collected during the session.
        """
        return self._diagnostics_history.copy()


# Convenience functions


async def wait_for_auroraview_ready(
    page: Any,
    timeout: float = 60.0,
    force_on_timeout: bool = True,
) -> bool:
    """Wait for AuroraView application to be ready.

    Convenience function that handles loading screen and
    waits for the AuroraView bridge to be available.

    Args:
        page: Playwright Page instance.
        timeout: Maximum wait time in seconds.
        force_on_timeout: Force navigation if loading times out.

    Returns:
        True if ready, False if timed out.

    Example:
        ```python
        from playwright.async_api import async_playwright
        from auroraview.testing.ai_automation import wait_for_auroraview_ready

        async with async_playwright() as p:
            browser = await p.chromium.connect_over_cdp("http://localhost:9222")
            page = browser.contexts[0].pages[0]

            if await wait_for_auroraview_ready(page):
                # App is ready, run tests
                pass
        ```
    """
    helper = AITestHelper(page)
    return await helper.wait_for_app_ready(timeout, force_on_timeout)


async def get_loading_status(page: Any) -> Dict[str, Any]:
    """Get current loading status from the page.

    Args:
        page: Playwright Page instance.

    Returns:
        Dictionary with loading status information.
    """
    try:
        return await page.evaluate("""
            () => {
                const result = {
                    is_loading_screen: !!(window.auroraLoading || document.querySelector('.loading-container')),
                    backend_ready: window.__backendReady || false,
                    auroraview_available: typeof window.auroraview !== 'undefined',
                    api_available: typeof window.auroraview?.api !== 'undefined',
                };

                if (window.auroraLoading) {
                    result.loading_state = window.auroraLoading.getState();
                    result.elapsed_ms = window.auroraLoading.getElapsedTime?.() || 0;
                }

                return result;
            }
        """)
    except Exception as e:
        return {"error": str(e)}


async def find_auroraview_page(browser: Any) -> Any:
    """Find the correct AuroraView page from all available pages.

    WebView2 with CDP may expose multiple pages:
    - about:blank (empty initial page)
    - Loading screen page
    - Main application page

    This method finds the most appropriate page for testing.

    Args:
        browser: Playwright Browser instance connected via CDP.

    Returns:
        The best matching Playwright Page, or None if not found.

    Example:
        ```python
        from playwright.async_api import async_playwright
        from auroraview.testing.ai_automation import find_auroraview_page

        async with async_playwright() as p:
            browser = await p.chromium.connect_over_cdp("http://localhost:9222")
            page = await find_auroraview_page(browser)
            if page:
                # Use the page for testing
                pass
        ```
    """
    all_pages = []
    for context in browser.contexts:
        for page in context.pages:
            url = page.url
            title = ""
            try:
                title = await page.title()
            except Exception:
                pass
            all_pages.append(
                {
                    "page": page,
                    "url": url,
                    "title": title,
                }
            )

    logger.info(f"[CDP] Found {len(all_pages)} page(s):")
    for i, p in enumerate(all_pages):
        logger.info(f"  [{i}] URL: {p['url']}, Title: {p['title']}")

    if not all_pages:
        return None

    # Priority order for page selection:
    # 1. Page with auroraview API ready
    # 2. Page with loading screen (auroraLoading)
    # 3. Page with non-blank URL (not about:blank)
    # 4. First available page

    # Check for auroraview ready
    for p in all_pages:
        try:
            has_api = await p["page"].evaluate("""
                () => typeof window.auroraview !== 'undefined' &&
                      typeof window.auroraview.api !== 'undefined'
            """)
            if has_api:
                logger.info(f"[CDP] Selected page with auroraview API: {p['url']}")
                return p["page"]
        except Exception:
            pass

    # Check for loading screen
    for p in all_pages:
        try:
            has_loading = await p["page"].evaluate("""
                () => typeof window.auroraLoading !== 'undefined' ||
                      document.querySelector('.loading-container') !== null
            """)
            if has_loading:
                logger.info(f"[CDP] Selected page with loading screen: {p['url']}")
                return p["page"]
        except Exception:
            pass

    # Skip about:blank pages
    for p in all_pages:
        if p["url"] and p["url"] != "about:blank":
            logger.info(f"[CDP] Selected non-blank page: {p['url']}")
            return p["page"]

    # Fallback to first page
    logger.info(f"[CDP] Fallback to first page: {all_pages[0]['url']}")
    return all_pages[0]["page"]


def find_auroraview_page_sync(browser: Any) -> Any:
    """Synchronous version of find_auroraview_page.

    Args:
        browser: Playwright Browser instance connected via CDP.

    Returns:
        The best matching Playwright Page, or None if not found.
    """
    all_pages = []
    for context in browser.contexts:
        for page in context.pages:
            url = page.url
            title = ""
            try:
                title = page.title()
            except Exception:
                pass
            all_pages.append(
                {
                    "page": page,
                    "url": url,
                    "title": title,
                }
            )

    print(f"[CDP] Found {len(all_pages)} page(s):")
    for i, p in enumerate(all_pages):
        print(f"  [{i}] URL: {p['url']}, Title: {p['title']}")

    if not all_pages:
        return None

    # Check for auroraview ready
    for p in all_pages:
        try:
            has_api = p["page"].evaluate("""
                () => typeof window.auroraview !== 'undefined' &&
                      typeof window.auroraview.api !== 'undefined'
            """)
            if has_api:
                print(f"[CDP] Selected page with auroraview API: {p['url']}")
                return p["page"]
        except Exception:
            pass

    # Check for loading screen
    for p in all_pages:
        try:
            has_loading = p["page"].evaluate("""
                () => typeof window.auroraLoading !== 'undefined' ||
                      document.querySelector('.loading-container') !== null
            """)
            if has_loading:
                print(f"[CDP] Selected page with loading screen: {p['url']}")
                return p["page"]
        except Exception:
            pass

    # Skip about:blank pages
    for p in all_pages:
        if p["url"] and p["url"] != "about:blank":
            print(f"[CDP] Selected non-blank page: {p['url']}")
            return p["page"]

    # Fallback to first page
    print(f"[CDP] Fallback to first page: {all_pages[0]['url']}")
    return all_pages[0]["page"]


__all__ = [
    "AITestConfig",
    "AITestHelper",
    "LoadingDiagnostics",
    "wait_for_auroraview_ready",
    "get_loading_status",
    "find_auroraview_page",
    "find_auroraview_page_sync",
]
