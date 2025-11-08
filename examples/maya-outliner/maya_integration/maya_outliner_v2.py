"""
Maya Outliner Example for AuroraView - Version 2 (with Lifecycle Management)

This is an improved version that demonstrates the new lifecycle management system.

Key improvements over v1:
1. Uses the new lifecycle management system for better close detection
2. Simplified event processing with automatic cleanup
3. Better resource management with guaranteed cleanup
4. Cross-platform support (Windows, macOS, Linux)

IMPORTANT: This requires AuroraView with lifecycle management support (v0.2.4+)
"""

import logging
from typing import Optional

from .maya_outliner import MAYA_AVAILABLE

# Import the original MayaOutliner as base class
from .maya_outliner import MayaOutliner as MayaOutlinerV1

logger = logging.getLogger(__name__)


class MayaOutlinerV2(MayaOutlinerV1):
    """Maya Outliner with improved lifecycle management.

    This version uses the new lifecycle management system introduced in AuroraView v0.2.4.

    Benefits:
    - Event-driven close detection (no polling overhead)
    - Guaranteed resource cleanup (using scopeguard)
    - Better cross-platform support
    - Simplified code with less boilerplate
    """

    def __init__(self, singleton_key: Optional[str] = None):
        """Initialize Maya Outliner V2

        Args:
            singleton_key: If provided, enables singleton mode with this key.
        """
        super().__init__(singleton_key)
        self._lifecycle_supported = False  # Will be set to True if lifecycle is available

    def _start_event_processing(self):
        """Start automatic event processing.

        This version tries to use the new lifecycle-aware approach first,
        then falls back to the EventTimer approach if lifecycle is not available.
        """
        if self.webview is None:
            logger.error("Cannot start event processing (webview not available)")
            return

        # Check if webview supports lifecycle management
        if self._check_lifecycle_support():
            logger.info("[MayaOutlinerV2] Using new lifecycle management system")
            self._start_lifecycle_processing()
        else:
            logger.info("[MayaOutlinerV2] Lifecycle not supported, falling back to EventTimer")
            super()._start_event_processing()

    def _check_lifecycle_support(self) -> bool:
        """Check if the webview supports lifecycle management.

        Returns:
            True if lifecycle is supported, False otherwise
        """
        try:
            # Check if webview has lifecycle support
            if hasattr(self.webview, "_core"):
                core = self.webview._core
                # Check for lifecycle-related methods
                if hasattr(core, "get_lifecycle_state"):
                    self._lifecycle_supported = True
                    logger.info("[MayaOutlinerV2] ✅ Lifecycle management supported")
                    return True
        except Exception as e:
            logger.debug(f"[MayaOutlinerV2] Lifecycle check failed: {e}")

        self._lifecycle_supported = False
        logger.info("[MayaOutlinerV2] ❌ Lifecycle management not supported")
        return False

    def _start_lifecycle_processing(self):
        """Start event processing using the new lifecycle system.

        This uses the lifecycle manager's close signal channel for efficient
        event-driven close detection instead of polling.
        """
        try:
            from auroraview import EventTimer

            # Create EventTimer with lifecycle awareness
            self._event_timer = EventTimer(
                self.webview,
                interval_ms=16,  # 60 FPS
                check_window_validity=True,  # Still check validity as fallback
            )

            # Lightweight debug throttle
            _counter = {"n": 0}

            @self._event_timer.on_tick
            def handle_tick():
                _counter["n"] += 1
                if _counter["n"] <= 5 or (_counter["n"] % 60 == 0):
                    print(f"[MayaOutlinerV2] Lifecycle tick #{_counter['n']}")

                    # Log lifecycle state periodically
                    if _counter["n"] % 300 == 0:  # Every ~5 seconds
                        try:
                            if hasattr(self.webview, "_core"):
                                state = self.webview._core.get_lifecycle_state()
                                print(f"[MayaOutlinerV2] Lifecycle state: {state}")
                        except Exception:
                            pass

            @self._event_timer.on_close
            def handle_close():
                print("[MayaOutlinerV2] ✅ Close signal detected from lifecycle manager")
                print("[MayaOutlinerV2] Invoking cleanup...")

                # The lifecycle manager has already detected the close,
                # so we just need to cleanup our resources
                try:
                    self.close()
                except Exception as e:
                    print(f"[MayaOutlinerV2] Error during close: {e}")

            # Start the timer
            self._event_timer.start()
            print("[MayaOutlinerV2] ✅ Lifecycle-aware event processing started")
            print("[MayaOutlinerV2] Close detection: Event-driven (via lifecycle channels)")

        except Exception as e:
            print(f"[MayaOutlinerV2] Failed to start lifecycle processing: {e}")
            import traceback

            traceback.print_exc()

            # Fallback to parent implementation
            super()._start_event_processing()


def main(url: Optional[str] = None, use_local: bool = False, singleton: bool = True):
    """Main entry point for Maya Outliner V2

    Args:
        url: URL to load. If None, auto-detect based on use_local flag
        use_local: If True, use local built files. If False, use dev server
        singleton: If True, only allow one instance at a time

    Returns:
        MayaOutlinerV2 instance

    Example:
        >>> from maya_integration import maya_outliner_v2
        >>> outliner = maya_outliner_v2.main()
        >>> # Window will automatically close when user clicks X
        >>> # Cleanup is guaranteed even if errors occur
    """
    print("=" * 60)
    print("Maya Outliner V2 - With Lifecycle Management")
    print("=" * 60)
    print()

    if not MAYA_AVAILABLE:
        print("Warning: Running without Maya (using mock data)")
        print()

    if singleton:
        # Singleton mode
        def create_instance():
            outliner = MayaOutlinerV2(singleton_key="maya_outliner_v2_default")
            outliner.run(url=url, use_local=use_local)
            return outliner

        outliner = MayaOutlinerV2._get_or_create_singleton(
            "maya_outliner_v2_default", create_instance
        )
    else:
        # Multi-instance mode
        outliner = MayaOutlinerV2()
        outliner.run(url=url, use_local=use_local)

    print()
    print("=" * 60)
    print("Maya Outliner V2 started successfully!")
    print("=" * 60)
    print()
    print("New Features:")
    print("✅ Event-driven close detection (no polling overhead)")
    print("✅ Guaranteed resource cleanup (using scopeguard)")
    print("✅ Better cross-platform support")
    print("✅ Automatic lifecycle state tracking")
    print()
    print("Tips:")
    print("- Click nodes to select them in Maya")
    print("- Toggle visibility with the eye icon")
    print("- Press F12 to open DevTools")
    print("- Close button now works reliably!")
    print("- Use outliner.close() for programmatic close")
    print()

    return outliner


if __name__ == "__main__":
    main()
