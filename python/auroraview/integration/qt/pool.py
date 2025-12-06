"""WebView pre-warming pool for Qt integration.

This module provides a pool mechanism to pre-warm WebView instances,
significantly reducing the perceived initialization time when users
open WebView-based tools.

The key insight is that WebView2 initialization is expensive (~300-500ms)
because it needs to:
1. Load the WebView2 Runtime
2. Create the CoreWebView2Environment
3. Initialize the browser process

By pre-warming a hidden WebView during DCC startup (when users expect
some loading time), subsequent WebView creations can reuse the warmed
environment and appear nearly instant.

Usage:
    # During DCC startup (e.g., in userSetup.py for Maya)
    from auroraview.integration.qt.pool import WebViewPool
    WebViewPool.prewarm()

    # Later, when creating a WebView
    from auroraview import QtWebView
    webview = QtWebView(parent=maya_main_window())  # Uses pre-warmed instance
"""

import logging
import time
from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from auroraview.core.webview import WebView

logger = logging.getLogger(__name__)


class WebViewPool:
    """Pool for pre-warming WebView instances.

    This class manages a pool of pre-warmed WebView instances that can be
    reused to reduce initialization time. The pool is designed to be used
    as a singleton via class methods.

    The pre-warming strategy:
    1. Create a hidden WebView during DCC startup
    2. The WebView2 Runtime and Environment are initialized
    3. When a real WebView is needed, the environment is already warm
    4. The pre-warmed instance can be reused or discarded

    Note:
        WebView2's CoreWebView2Environment is process-bound, so pre-warming
        in the same process provides the most benefit. Cross-process sharing
        is not supported by WebView2.
    """

    # Class-level storage for pre-warmed instance
    _prewarmed_instance: Optional["WebView"] = None
    _prewarm_time: Optional[float] = None
    _is_prewarming: bool = False

    @classmethod
    def prewarm(
        cls,
        *,
        width: int = 100,
        height: int = 100,
        timeout_ms: int = 5000,
    ) -> bool:
        """Pre-warm a WebView instance in the background.

        This creates a small, hidden WebView to initialize the WebView2
        Runtime and Environment. The instance is kept alive so subsequent
        WebView creations can benefit from the warmed environment.

        Should be called during DCC startup when users expect loading time.

        Args:
            width: Width of the hidden WebView (default: 100, minimal)
            height: Height of the hidden WebView (default: 100, minimal)
            timeout_ms: Maximum time to wait for initialization (default: 5000ms)

        Returns:
            True if pre-warming succeeded, False otherwise.

        Example:
            # In Maya's userSetup.py
            from auroraview.integration.qt.pool import WebViewPool
            WebViewPool.prewarm()
        """
        if cls._prewarmed_instance is not None:
            logger.debug("[WebViewPool] Already pre-warmed, skipping")
            return True

        if cls._is_prewarming:
            logger.debug("[WebViewPool] Pre-warming in progress, skipping")
            return False

        cls._is_prewarming = True
        start_time = time.time()

        try:
            from auroraview.core.webview import WebView

            logger.info("[WebViewPool] Starting pre-warm...")

            # Create a minimal hidden WebView
            # - No parent: standalone mode (simpler)
            # - auto_show=False: keep it hidden
            # - Small size: minimal resource usage
            instance = WebView.create(
                title="AuroraView Prewarm",
                width=width,
                height=height,
                auto_show=False,  # Keep hidden
                debug=False,  # No dev tools needed
                context_menu=False,  # No context menu needed
            )

            # Store the pre-warmed instance
            cls._prewarmed_instance = instance
            cls._prewarm_time = time.time() - start_time

            logger.info(f"[WebViewPool] Pre-warm complete in {cls._prewarm_time * 1000:.1f}ms")
            return True

        except Exception as e:
            logger.warning(f"[WebViewPool] Pre-warm failed: {e}")
            return False

        finally:
            cls._is_prewarming = False

    @classmethod
    def has_prewarmed(cls) -> bool:
        """Check if a pre-warmed instance is available.

        Returns:
            True if a pre-warmed instance exists.
        """
        return cls._prewarmed_instance is not None

    @classmethod
    def get_prewarm_time(cls) -> Optional[float]:
        """Get the time taken for pre-warming.

        Returns:
            Time in seconds, or None if not pre-warmed.
        """
        return cls._prewarm_time

    @classmethod
    def cleanup(cls) -> None:
        """Clean up the pre-warmed instance.

        Call this during DCC shutdown to properly release resources.
        """
        if cls._prewarmed_instance is not None:
            try:
                cls._prewarmed_instance.close()
            except Exception as e:
                logger.debug(f"[WebViewPool] Cleanup warning: {e}")
            finally:
                cls._prewarmed_instance = None
                cls._prewarm_time = None
                logger.debug("[WebViewPool] Cleaned up pre-warmed instance")
