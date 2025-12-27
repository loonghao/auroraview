#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""ShotGrid MCP Browser Extension Integration with AuroraView.

This example shows how to integrate the ShotGrid MCP Chrome extension
with AuroraView for DCC application communication.

Architecture:
    ┌─────────────────────────────────────────────────────────────────┐
    │              ShotGrid MCP Chrome Extension                       │
    │  (Side Panel with AI Chat + Content Script for DOM)              │
    └───────────────────────────┬─────────────────────────────────────┘
                                │ WebSocket / HTTP
                                ▼
    ┌─────────────────────────────────────────────────────────────────┐
    │                 BrowserExtensionBridge                           │
    │  - Receives commands from Chrome extension                       │
    │  - Sends events back to extension                                │
    └───────────────────────────┬─────────────────────────────────────┘
                                │ Python API
                                ▼
    ┌─────────────────────────────────────────────────────────────────┐
    │              DCC Application (Maya, Houdini, etc.)               │
    │  - Execute scene operations                                      │
    │  - Query scene data                                              │
    │  - Sync with ShotGrid                                            │
    └─────────────────────────────────────────────────────────────────┘

Usage:
    1. Start this server:
       python shotgrid_mcp_integration.py

    2. In your Chrome extension, connect to:
       WebSocket: ws://127.0.0.1:9001
       HTTP: http://127.0.0.1:9002

    3. Call handlers from the extension:
       client.call('sg_get_task_info', { task_id: 123 })
       client.call('dcc_get_scene', {})
       client.call('dcc_export_for_review', { task_id: 123 })
"""

import logging
import os
import time
from datetime import datetime
from typing import Any, Dict, List, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Import AuroraView
try:
    from auroraview.integration import BrowserExtensionBridge
except ImportError:
    import sys

    sys.path.insert(0, "../../python")
    from auroraview.integration import BrowserExtensionBridge


# =============================================================================
# Mock ShotGrid API (replace with real shotgun_api3 in production)
# =============================================================================


class MockShotGridAPI:
    """Mock ShotGrid API for demonstration."""

    def __init__(self):
        self.tasks = {
            123: {
                "id": 123,
                "content": "Animation - Shot 010",
                "entity": {"type": "Shot", "id": 456, "name": "shot_010"},
                "project": {"type": "Project", "id": 1, "name": "Demo Project"},
                "task_assignees": [{"type": "HumanUser", "id": 1, "name": "Artist"}],
                "sg_status_list": "ip",
                "due_date": "2025-01-15",
            },
            124: {
                "id": 124,
                "content": "Lighting - Shot 010",
                "entity": {"type": "Shot", "id": 456, "name": "shot_010"},
                "project": {"type": "Project", "id": 1, "name": "Demo Project"},
                "task_assignees": [{"type": "HumanUser", "id": 2, "name": "Lighter"}],
                "sg_status_list": "wtg",
                "due_date": "2025-01-20",
            },
        }

        self.versions = []

    def find_one(self, entity_type: str, filters: list, fields: list) -> Optional[Dict]:
        if entity_type == "Task":
            task_id = filters[0][2]
            return self.tasks.get(task_id)
        return None

    def find(self, entity_type: str, filters: list, fields: list) -> List[Dict]:
        if entity_type == "Task":
            return list(self.tasks.values())
        return []

    def create(self, entity_type: str, data: Dict) -> Dict:
        if entity_type == "Version":
            version = {
                "id": len(self.versions) + 1,
                "type": "Version",
                **data,
            }
            self.versions.append(version)
            return version
        return {}

    def update(self, entity_type: str, entity_id: int, data: Dict) -> Dict:
        if entity_type == "Task" and entity_id in self.tasks:
            self.tasks[entity_id].update(data)
            return self.tasks[entity_id]
        return {}


# =============================================================================
# Mock DCC Application
# =============================================================================


class MockDCCApplication:
    """Mock DCC application (Maya, Houdini, etc.)."""

    def __init__(self):
        self.scene_path = "/projects/demo/scenes/shot_010_anim_v001.ma"
        self.frame_range = (1001, 1100)
        self.current_frame = 1001
        self.fps = 24
        self.objects = ["char_hero", "char_sidekick", "prop_sword", "env_castle"]
        self.selected = []
        self.render_camera = "renderCam"

    def get_scene_info(self) -> Dict[str, Any]:
        return {
            "scene_path": self.scene_path,
            "scene_name": os.path.basename(self.scene_path),
            "frame_range": self.frame_range,
            "current_frame": self.current_frame,
            "fps": self.fps,
            "render_camera": self.render_camera,
            "object_count": len(self.objects),
        }

    def export_playblast(self, output_path: str, frame_range: tuple = None) -> str:
        """Simulate playblast export."""
        range_str = f"{frame_range[0]}-{frame_range[1]}" if frame_range else "all"
        logger.info(f"Exporting playblast to {output_path} (frames: {range_str})")
        # In real implementation, this would call Maya's playblast command
        return output_path

    def export_render(self, output_path: str, frame_range: tuple = None) -> str:
        """Simulate render export."""
        logger.info(f"Exporting render to {output_path}")
        return output_path


# =============================================================================
# Global instances
# =============================================================================

sg = MockShotGridAPI()
dcc = MockDCCApplication()


# =============================================================================
# Main Server
# =============================================================================


def main():
    """Start the ShotGrid MCP integration bridge."""

    bridge = BrowserExtensionBridge(
        ws_port=9001,
        http_port=9002,
        host="127.0.0.1",
    )

    # =========================================================================
    # ShotGrid Handlers
    # =========================================================================

    @bridge.on("sg_get_task_info")
    async def handle_sg_get_task_info(data: Dict, client: Any):
        """Get ShotGrid task information."""
        task_id = data.get("task_id")
        logger.info(f"SG: Getting task info for {task_id}")

        task = sg.find_one(
            "Task",
            [["id", "is", task_id]],
            ["content", "entity", "project", "task_assignees", "sg_status_list", "due_date"],
        )

        if task:
            return {"success": True, "task": task}
        return {"success": False, "error": f"Task {task_id} not found"}

    @bridge.on("sg_list_my_tasks")
    async def handle_sg_list_my_tasks(data: Dict, client: Any):
        """List tasks assigned to current user."""
        logger.info("SG: Listing my tasks")
        tasks = sg.find("Task", [], ["content", "entity", "sg_status_list", "due_date"])
        return {"success": True, "tasks": tasks}

    @bridge.on("sg_update_task_status")
    async def handle_sg_update_task_status(data: Dict, client: Any):
        """Update task status in ShotGrid."""
        task_id = data.get("task_id")
        status = data.get("status")
        logger.info(f"SG: Updating task {task_id} status to {status}")

        result = sg.update("Task", task_id, {"sg_status_list": status})

        # Notify all clients
        bridge.emit("sg_task_updated", {"task_id": task_id, "status": status})

        return {"success": True, "task": result}

    @bridge.on("sg_create_version")
    async def handle_sg_create_version(data: Dict, client: Any):
        """Create a new version in ShotGrid."""
        task_id = data.get("task_id")
        description = data.get("description", "")
        media_path = data.get("media_path")

        logger.info(f"SG: Creating version for task {task_id}")

        task = sg.find_one("Task", [["id", "is", task_id]], ["entity", "project"])
        if not task:
            return {"success": False, "error": "Task not found"}

        version_data = {
            "code": f"v{len(sg.versions) + 1:03d}",
            "description": description,
            "entity": task["entity"],
            "project": task["project"],
            "sg_task": {"type": "Task", "id": task_id},
            "sg_path_to_movie": media_path,
            "created_at": datetime.now().isoformat(),
        }

        version = sg.create("Version", version_data)

        # Notify all clients
        bridge.emit("sg_version_created", {"version": version})

        return {"success": True, "version": version}

    # =========================================================================
    # DCC Handlers
    # =========================================================================

    @bridge.on("dcc_get_scene")
    async def handle_dcc_get_scene(data: Dict, client: Any):
        """Get current DCC scene information."""
        logger.info("DCC: Getting scene info")
        return {"success": True, "scene": dcc.get_scene_info()}

    @bridge.on("dcc_export_playblast")
    async def handle_dcc_export_playblast(data: Dict, client: Any):
        """Export playblast from DCC."""
        output_dir = data.get("output_dir", "/tmp")
        frame_range = data.get("frame_range")

        logger.info("DCC: Exporting playblast")

        # Generate output path
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = os.path.join(output_dir, f"playblast_{timestamp}.mov")

        # Export
        result_path = dcc.export_playblast(output_path, frame_range)

        # Notify progress
        bridge.emit("dcc_export_progress", {"progress": 100, "status": "complete"})

        return {"success": True, "path": result_path}

    @bridge.on("dcc_export_for_review")
    async def handle_dcc_export_for_review(data: Dict, client: Any):
        """Export playblast and create ShotGrid version."""
        task_id = data.get("task_id")
        description = data.get("description", "Automated export for review")

        logger.info(f"DCC: Exporting for review (task {task_id})")

        # 1. Export playblast
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = f"/tmp/review_{task_id}_{timestamp}.mov"
        dcc.export_playblast(output_path)

        bridge.emit("dcc_export_progress", {"progress": 50, "status": "creating_version"})

        # 2. Create ShotGrid version
        version_data = {
            "code": f"review_{timestamp}",
            "description": description,
            "sg_path_to_movie": output_path,
            "sg_task": {"type": "Task", "id": task_id},
        }
        version = sg.create("Version", version_data)

        bridge.emit("dcc_export_progress", {"progress": 100, "status": "complete"})
        bridge.emit("sg_version_created", {"version": version})

        return {
            "success": True,
            "media_path": output_path,
            "version": version,
        }

    @bridge.on("dcc_set_frame")
    async def handle_dcc_set_frame(data: Dict, client: Any):
        """Set current frame in DCC."""
        frame = data.get("frame")
        logger.info(f"DCC: Setting frame to {frame}")
        dcc.current_frame = frame
        bridge.emit("dcc_frame_changed", {"frame": frame})
        return {"success": True, "frame": frame}

    # =========================================================================
    # Combined Workflows
    # =========================================================================

    @bridge.on("workflow_start_task")
    async def handle_workflow_start_task(data: Dict, client: Any):
        """Start working on a task - update status and prepare scene."""
        task_id = data.get("task_id")
        logger.info(f"Workflow: Starting task {task_id}")

        # Update task status to "In Progress"
        sg.update("Task", task_id, {"sg_status_list": "ip"})

        # Get task info
        task = sg.find_one("Task", [["id", "is", task_id]], ["content", "entity"])

        # Notify
        bridge.emit("workflow_task_started", {"task_id": task_id, "task": task})

        return {
            "success": True,
            "message": f"Started task: {task['content']}",
            "task": task,
        }

    @bridge.on("workflow_submit_for_review")
    async def handle_workflow_submit_for_review(data: Dict, client: Any):
        """Complete workflow: export, create version, update status."""
        task_id = data.get("task_id")
        description = data.get("description", "Submitted for review")

        logger.info(f"Workflow: Submitting task {task_id} for review")

        # 1. Export playblast
        bridge.emit("workflow_progress", {"step": "exporting", "progress": 0})
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = f"/tmp/review_{task_id}_{timestamp}.mov"
        dcc.export_playblast(output_path)

        bridge.emit("workflow_progress", {"step": "creating_version", "progress": 50})

        # 2. Create version
        task = sg.find_one("Task", [["id", "is", task_id]], ["entity", "project"])
        version = sg.create(
            "Version",
            {
                "code": f"v{len(sg.versions) + 1:03d}",
                "description": description,
                "entity": task["entity"],
                "project": task["project"],
                "sg_task": {"type": "Task", "id": task_id},
                "sg_path_to_movie": output_path,
            },
        )

        bridge.emit("workflow_progress", {"step": "updating_status", "progress": 80})

        # 3. Update task status
        sg.update("Task", task_id, {"sg_status_list": "rev"})

        bridge.emit("workflow_progress", {"step": "complete", "progress": 100})
        bridge.emit(
            "workflow_submitted",
            {
                "task_id": task_id,
                "version": version,
            },
        )

        return {
            "success": True,
            "message": "Submitted for review",
            "version": version,
            "media_path": output_path,
        }

    # =========================================================================
    # Utility Handlers
    # =========================================================================

    @bridge.on("ping")
    async def handle_ping(data: Dict, client: Any):
        return {"pong": True, "timestamp": time.time()}

    @bridge.on("get_capabilities")
    async def handle_get_capabilities(data: Dict, client: Any):
        """Return available handlers and capabilities."""
        return {
            "handlers": list(bridge._handlers.keys()),
            "dcc": "maya",  # or houdini, blender, etc.
            "version": "1.0.0",
        }

    # =========================================================================
    # Start Server
    # =========================================================================

    logger.info("=" * 70)
    logger.info("ShotGrid MCP Browser Extension Bridge")
    logger.info("=" * 70)
    logger.info(f"WebSocket: ws://127.0.0.1:{bridge.ws_port}")
    logger.info(f"HTTP:      http://127.0.0.1:{bridge.http_port}")
    logger.info("=" * 70)
    logger.info("Available handlers:")
    logger.info("")
    logger.info("ShotGrid:")
    for h in ["sg_get_task_info", "sg_list_my_tasks", "sg_update_task_status", "sg_create_version"]:
        logger.info(f"  - {h}")
    logger.info("")
    logger.info("DCC:")
    for h in ["dcc_get_scene", "dcc_export_playblast", "dcc_export_for_review", "dcc_set_frame"]:
        logger.info(f"  - {h}")
    logger.info("")
    logger.info("Workflows:")
    for h in ["workflow_start_task", "workflow_submit_for_review"]:
        logger.info(f"  - {h}")
    logger.info("")
    logger.info("=" * 70)
    logger.info("Press Ctrl+C to stop")
    logger.info("")

    bridge.start_background()

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        logger.info("\nShutting down...")
        bridge.stop()


if __name__ == "__main__":
    main()
