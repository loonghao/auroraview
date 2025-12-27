"""Child Window API handlers for Gallery.

This module registers API handlers for managing child windows
(examples running as sub-windows of Gallery).

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import sys
from typing import TYPE_CHECKING, Any, Dict, List, Optional

from .child_manager import init_manager
from .config import EXAMPLES_DIR
from .samples import get_sample_by_id

if TYPE_CHECKING:
    from auroraview import WebView


def register_child_apis(view: "WebView") -> Dict[str, Any]:
    """Register child window API handlers.

    Args:
        view: The Gallery WebView instance.

    Returns:
        Dict of API function references.
    """
    # Initialize child manager
    manager = init_manager(view)

    @view.bind_call("api.launch_example_as_child")
    def launch_example_as_child(
        sample_id: str,
        *,
        extra_env: Optional[Dict[str, str]] = None,
    ) -> Dict:
        """Launch an example as a child window.

        Args:
            sample_id: The sample ID to launch.
            extra_env: Extra environment variables.

        Returns:
            Dict with child_id and status.
        """
        sample = get_sample_by_id(sample_id)
        if not sample:
            return {"error": f"Sample not found: {sample_id}"}

        example_path = EXAMPLES_DIR / sample["filename"]
        if not example_path.exists():
            return {"error": f"Example file not found: {example_path}"}

        try:
            child_id = manager.launch_example(
                str(example_path),
                sample["title"],
                extra_env=extra_env,
            )
            return {
                "success": True,
                "child_id": child_id,
                "example_name": sample["title"],
            }
        except Exception as e:
            return {"error": str(e)}

    @view.bind_call("api.close_child")
    def close_child(child_id: str) -> Dict:
        """Close a child window.

        Args:
            child_id: The child window ID.

        Returns:
            Dict with status.
        """
        success = manager.close_child(child_id)
        return {"success": success}

    @view.bind_call("api.get_children")
    def get_children() -> List[Dict]:
        """Get list of all child windows.

        Returns:
            List of child window info.
        """
        return manager.get_children()

    @view.bind_call("api.send_to_child")
    def send_to_child(child_id: str, event: str, data: Any = None) -> Dict:
        """Send an event to a child window.

        Args:
            child_id: The child window ID.
            event: Event name.
            data: Event data.

        Returns:
            Dict with status.
        """
        success = manager.send_to_child(child_id, event, data)
        return {"success": success}

    @view.bind_call("api.broadcast_to_children")
    def broadcast_to_children(event: str, data: Any = None) -> Dict:
        """Broadcast an event to all children.

        Args:
            event: Event name.
            data: Event data.

        Returns:
            Dict with count of children that received the event.
        """
        count = manager.broadcast_to_children(event, data)
        return {"success": True, "count": count}

    print("[ChildAPI] Registered child window APIs", file=sys.stderr)

    return {
        "launch_example_as_child": launch_example_as_child,
        "close_child": close_child,
        "get_children": get_children,
        "send_to_child": send_to_child,
        "broadcast_to_children": broadcast_to_children,
    }


__all__ = ["register_child_apis"]
