#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""AuroraView Browser Extension Bridge Server Example.

This example demonstrates how to create a Python server that can communicate
with browser extensions (Chrome, Firefox, Edge) via WebSocket and HTTP.

Usage:
    python server.py

The server will start on:
    - WebSocket: ws://127.0.0.1:9001
    - HTTP: http://127.0.0.1:9002

From your Chrome extension, you can connect using:
    const ws = new WebSocket("ws://127.0.0.1:9001");
    ws.send(JSON.stringify({action: "get_scene_info", data: {}}));

Or use HTTP:
    fetch("http://127.0.0.1:9002/call", {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({action: "get_scene_info", data: {}})
    });
"""

import logging
import time
from typing import Any, Dict, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Import AuroraView browser extension bridge
try:
    from auroraview.integration import BrowserExtensionBridge, create_browser_bridge
except ImportError:
    # Fallback for development
    import sys

    sys.path.insert(0, "../../python")
    from auroraview.integration import BrowserExtensionBridge


# =============================================================================
# Example: Simulated DCC Application State
# =============================================================================


class MockDCCApplication:
    """Simulated DCC application (Maya, Houdini, etc.)."""

    def __init__(self):
        self.scene_name = "untitled.ma"
        self.frame = 1
        self.fps = 24
        self.objects = ["pCube1", "pSphere1", "camera1", "directionalLight1"]
        self.selected = []

    def get_scene_info(self) -> Dict[str, Any]:
        return {
            "scene_name": self.scene_name,
            "frame": self.frame,
            "fps": self.fps,
            "object_count": len(self.objects),
        }

    def list_objects(self) -> list:
        return self.objects

    def select_object(self, name: str) -> bool:
        if name in self.objects:
            self.selected = [name]
            return True
        return False

    def create_object(self, obj_type: str, name: Optional[str] = None) -> str:
        obj_name = name or f"{obj_type}_{len(self.objects) + 1}"
        self.objects.append(obj_name)
        return obj_name

    def set_frame(self, frame: int):
        self.frame = frame


# Global DCC instance
dcc = MockDCCApplication()


# =============================================================================
# Create Bridge and Register Handlers
# =============================================================================


def main():
    """Main function to start the browser extension bridge."""

    # Create bridge
    bridge = BrowserExtensionBridge(
        ws_port=9001,
        http_port=9002,
        host="127.0.0.1",
    )

    # Register handlers using decorator syntax
    @bridge.on("get_scene_info")
    async def handle_get_scene_info(data: Dict, client: Any):
        """Get current scene information."""
        logger.info("Handler: get_scene_info")
        return dcc.get_scene_info()

    @bridge.on("list_objects")
    async def handle_list_objects(data: Dict, client: Any):
        """List all objects in the scene."""
        logger.info("Handler: list_objects")
        return {"objects": dcc.list_objects()}

    @bridge.on("select_object")
    async def handle_select_object(data: Dict, client: Any):
        """Select an object by name."""
        name = data.get("name")
        logger.info(f"Handler: select_object({name})")
        success = dcc.select_object(name)
        return {"success": success, "selected": dcc.selected}

    @bridge.on("create_object")
    async def handle_create_object(data: Dict, client: Any):
        """Create a new object."""
        obj_type = data.get("type", "cube")
        name = data.get("name")
        logger.info(f"Handler: create_object({obj_type}, {name})")
        obj_name = dcc.create_object(obj_type, name)

        # Broadcast event to all clients
        bridge.emit("object_created", {"name": obj_name, "type": obj_type})

        return {"success": True, "name": obj_name}

    @bridge.on("set_frame")
    async def handle_set_frame(data: Dict, client: Any):
        """Set the current frame."""
        frame = data.get("frame", 1)
        logger.info(f"Handler: set_frame({frame})")
        dcc.set_frame(frame)

        # Broadcast event
        bridge.emit("frame_changed", {"frame": frame})

        return {"success": True, "frame": frame}

    @bridge.on("ping")
    async def handle_ping(data: Dict, client: Any):
        """Simple ping handler for testing."""
        return {"pong": True, "timestamp": time.time()}

    @bridge.on("execute_python")
    async def handle_execute_python(data: Dict, client: Any):
        """Execute Python code (be careful with this in production!)."""
        code = data.get("code", "")
        logger.warning(f"Handler: execute_python - {code[:50]}...")

        # In a real application, you'd want to sandbox this!
        try:
            # Very basic execution - NOT SAFE FOR PRODUCTION
            result = eval(code)
            return {"success": True, "result": str(result)}
        except Exception as e:
            return {"success": False, "error": str(e)}

    # Start bridge
    logger.info("=" * 60)
    logger.info("AuroraView Browser Extension Bridge")
    logger.info("=" * 60)
    logger.info(f"WebSocket: ws://127.0.0.1:{bridge.ws_port}")
    logger.info(f"HTTP:      http://127.0.0.1:{bridge.http_port}")
    logger.info("=" * 60)
    logger.info("Available handlers:")
    for handler in bridge._handlers:
        logger.info(f"  - {handler}")
    logger.info("=" * 60)
    logger.info("Press Ctrl+C to stop")
    logger.info("")

    bridge.start_background()

    # Keep main thread alive
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        logger.info("\nShutting down...")
        bridge.stop()


if __name__ == "__main__":
    main()
