"""AuroraView instance discovery module.

This module provides discovery services for finding running AuroraView instances
that have CDP (Chrome DevTools Protocol) debugging enabled.

Discovery is unified across packed and unpacked modes using file-based registry:
- All AuroraView instances register to a shared file-based registry
- No HTTP metadata service required
- MCP and external tools read the registry directly

Storage locations:
- Windows: %LOCALAPPDATA%/AuroraView/instances/
- macOS: ~/Library/Application Support/AuroraView/instances/
- Linux: ~/.local/share/auroraview/instances/
"""

from __future__ import annotations

import asyncio
import json
import os
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import httpx


def get_instances_dir() -> Path:
    """Get the instances directory path (cross-platform).

    Returns:
        Path to the instances directory.
    """
    if sys.platform == "win32":
        base = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local"))
    elif sys.platform == "darwin":
        base = Path.home() / "Library" / "Application Support"
    else:
        base = Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local" / "share"))

    return base / "AuroraView" / "instances"


def is_process_alive(pid: int) -> bool:
    """Check if a process is still running.

    Args:
        pid: Process ID to check.

    Returns:
        True if process is alive, False otherwise.
    """
    if sys.platform == "win32":
        try:
            import ctypes

            kernel32 = ctypes.windll.kernel32
            PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
            handle = kernel32.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, False, pid)
            if handle:
                kernel32.CloseHandle(handle)
                return True
            return False
        except Exception:
            return False
    else:
        try:
            os.kill(pid, 0)
            return True
        except (OSError, ProcessLookupError):
            return False


@dataclass
class Instance:
    """Represents a discovered AuroraView instance."""

    port: int
    browser: str = ""
    ws_url: str = ""
    user_agent: str = ""
    protocol_version: str = ""
    pid: int | None = None
    title: str = ""
    url: str = ""
    dcc_type: str | None = None
    dcc_version: str | None = None
    panel_name: str | None = None

    # Enhanced fields from file registry
    window_id: str | None = None
    app_name: str | None = None
    app_version: str | None = None
    dcc_python_version: str | None = None
    dock_area: str | None = None
    start_time: float | None = None
    devtools_url: str | None = None
    html_title: str | None = None
    is_loading: bool = False
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """Convert instance to dictionary."""
        return {
            "port": self.port,
            "browser": self.browser,
            "ws_url": self.ws_url,
            "user_agent": self.user_agent,
            "protocol_version": self.protocol_version,
            "pid": self.pid,
            "title": self.title,
            "url": self.url,
            "dcc_type": self.dcc_type,
            "dcc_version": self.dcc_version,
            "panel_name": self.panel_name,
            "window_id": self.window_id,
            "app_name": self.app_name,
            "app_version": self.app_version,
            "dcc_python_version": self.dcc_python_version,
            "dock_area": self.dock_area,
            "start_time": self.start_time,
            "devtools_url": self.devtools_url,
            "html_title": self.html_title,
            "is_loading": self.is_loading,
            "metadata": self.metadata,
        }

    def display_name(self) -> str:
        """Get a human-readable display name for this instance.

        Returns a name that can be used in UI or logs to identify this instance.
        """
        parts = []

        if self.app_name:
            parts.append(self.app_name)
        elif self.title:
            parts.append(self.title)

        if self.dcc_type:
            dcc_info = self.dcc_type
            if self.dcc_version:
                dcc_info += f" {self.dcc_version}"
            parts.append(f"[{dcc_info}]")

        if self.panel_name:
            parts.append(f"({self.panel_name})")

        if not parts:
            return f"WebView @ port {self.port}"

        return " ".join(parts)


# Default ports to scan (fallback for CDP probing)
DEFAULT_CDP_PORTS = [9222, 9223, 9224, 9225, 9226, 9227, 9228, 9229, 9230]


@dataclass
class InstanceDiscovery:
    """AuroraView instance discovery service.

    Discovers running AuroraView instances using unified file-based registry.
    No HTTP metadata service required - works the same for packed and unpacked modes.

    The discovery process:
    1. Read all instance files from the registry directory
    2. Filter out stale entries (dead processes)
    3. Optionally verify via CDP port probing

    Example:
        >>> discovery = InstanceDiscovery()
        >>> instances = await discovery.discover()
        >>> for inst in instances:
        ...     print(f"{inst.display_name()} - ws://{inst.ws_url}")
    """

    default_ports: list[int] = field(default_factory=lambda: DEFAULT_CDP_PORTS)
    timeout: float = 1.0
    verify_cdp: bool = False  # Whether to verify instances via CDP probe

    async def discover(self, ports: list[int] | None = None) -> list[Instance]:
        """Discover all running AuroraView instances.

        Primary discovery via file-based registry, with optional CDP port scanning
        as fallback for instances not registered in the registry.

        Args:
            ports: List of CDP ports to scan as fallback. If None, uses default ports.

        Returns:
            List of discovered instances with rich context.
        """
        # Primary: discover via file registry
        registry_instances = self._discover_via_registry()

        # Optionally verify instances are reachable via CDP
        if self.verify_cdp:
            verified = []
            for inst in registry_instances:
                if await self._verify_instance(inst):
                    verified.append(inst)
            registry_instances = verified

        # Fallback: scan CDP ports for instances not in registry
        seen_ports = {inst.port for inst in registry_instances}
        scan_ports = ports or self.default_ports
        fallback_ports = [p for p in scan_ports if p not in seen_ports]

        if fallback_ports:
            tasks = [self._probe_port(port) for port in fallback_ports]
            results = await asyncio.gather(*tasks, return_exceptions=True)

            for result in results:
                if isinstance(result, Instance):
                    registry_instances.append(result)

        return registry_instances

    def _discover_via_registry(self) -> list[Instance]:
        """Discover instances from file-based registry.

        Returns:
            List of instances from registry (stale entries filtered).
        """
        instances = []
        instances_dir = get_instances_dir()

        if not instances_dir.exists():
            return instances

        for file_path in instances_dir.glob("*.json"):
            try:
                content = file_path.read_text(encoding="utf-8")
                data = json.loads(content)

                # Check if process is still alive
                pid = data.get("pid")
                if pid and not is_process_alive(pid):
                    # Remove stale file
                    try:
                        file_path.unlink()
                    except OSError:
                        pass
                    continue

                inst = self._instance_from_registry(data)
                if inst:
                    instances.append(inst)

            except (json.JSONDecodeError, OSError):
                continue

        return instances

    def _instance_from_registry(self, data: dict[str, Any]) -> Instance | None:
        """Create Instance from registry file data.

        Args:
            data: Instance data from registry file.

        Returns:
            Instance object or None if invalid.
        """
        cdp_port = data.get("cdp_port")
        if not cdp_port:
            return None

        window_id = data.get("window_id", "")

        return Instance(
            port=cdp_port,
            ws_url=data.get("ws_url", f"ws://127.0.0.1:{cdp_port}/devtools/page/1"),
            title=data.get("title", ""),
            url=data.get("url", ""),
            pid=data.get("pid"),
            dcc_type=data.get("dcc_type"),
            dcc_version=data.get("dcc_version"),
            panel_name=data.get("panel_name"),
            window_id=window_id,
            app_name=data.get("app_name", "AuroraView"),
            app_version=data.get("app_version"),
            dock_area=data.get("dock_area"),
            start_time=data.get("start_time"),
            devtools_url=data.get(
                "devtools_url",
                f"devtools://devtools/bundled/inspector.html?ws=127.0.0.1:{cdp_port}/devtools/page/1",
            ),
            html_title=data.get("html_title", ""),
            is_loading=data.get("is_loading", False),
            metadata=data.get("metadata", {}),
        )

    async def _verify_instance(self, instance: Instance) -> bool:
        """Verify an instance is reachable via CDP.

        Args:
            instance: Instance to verify.

        Returns:
            True if instance is reachable.
        """
        try:
            async with httpx.AsyncClient() as client:
                resp = await client.get(
                    f"http://127.0.0.1:{instance.port}/json/version",
                    timeout=self.timeout,
                )
                return resp.status_code == 200
        except Exception:
            return False

    async def _probe_port(self, port: int) -> Instance | None:
        """Probe a single port for AuroraView instance.

        Args:
            port: Port number to probe.

        Returns:
            Instance if found, None otherwise.
        """
        try:
            async with httpx.AsyncClient() as client:
                # Try to get CDP version info
                resp = await client.get(
                    f"http://127.0.0.1:{port}/json/version",
                    timeout=self.timeout,
                )
                if resp.status_code == 200:
                    data = resp.json()
                    if self._is_webview(data):
                        return Instance(
                            port=port,
                            browser=data.get("Browser", ""),
                            ws_url=data.get("webSocketDebuggerUrl", ""),
                            user_agent=data.get("User-Agent", ""),
                            protocol_version=data.get("Protocol-Version", ""),
                        )
        except (httpx.RequestError, httpx.TimeoutException):
            pass
        except Exception:
            pass

        return None

    def _is_webview(self, data: dict[str, Any]) -> bool:
        """Check if the CDP endpoint is a WebView instance.

        Args:
            data: CDP version response data.

        Returns:
            True if it's a WebView instance.
        """
        browser = data.get("Browser", "")
        # WebView2 uses Edge/Chrome
        return "Edg" in browser or "Chrome" in browser

    async def discover_dcc_instances(self, ports: list[int] | None = None) -> list[Instance]:
        """Discover AuroraView instances in DCC environments.

        Args:
            ports: List of ports to scan.

        Returns:
            List of discovered DCC instances with context.
        """
        instances = await self.discover(ports)

        # Enrich instances that don't have DCC info from metadata
        enriched = []
        for instance in instances:
            if instance.dcc_type is None:
                enriched_instance = await self._enrich_dcc_context(instance)
                enriched.append(enriched_instance)
            else:
                enriched.append(instance)

        return enriched

    async def _enrich_dcc_context(self, instance: Instance) -> Instance:
        """Enrich instance with DCC context information.

        Args:
            instance: Base instance to enrich.

        Returns:
            Instance with DCC context if available.
        """
        try:
            async with httpx.AsyncClient() as client:
                # Get page list to find DCC context
                resp = await client.get(
                    f"http://127.0.0.1:{instance.port}/json/list",
                    timeout=self.timeout,
                )
                if resp.status_code == 200:
                    pages = resp.json()
                    for page in pages:
                        title = page.get("title", "")
                        url = page.get("url", "")

                        # Try to detect DCC type from title/url
                        dcc_type = self._detect_dcc_type(title, url)
                        if dcc_type:
                            instance.dcc_type = dcc_type
                            instance.title = title
                            instance.url = url
                            break
        except Exception:
            pass

        return instance

    def _detect_dcc_type(self, title: str, url: str) -> str | None:
        """Detect DCC type from page title or URL.

        Args:
            title: Page title.
            url: Page URL.

        Returns:
            DCC type string or None.
        """
        title_lower = title.lower()
        url_lower = url.lower()

        dcc_keywords = {
            "maya": ["maya", "autodesk maya"],
            "blender": ["blender"],
            "houdini": ["houdini", "sidefx"],
            "nuke": ["nuke", "foundry"],
            "unreal": ["unreal", "ue4", "ue5"],
            "3dsmax": ["3ds max", "3dsmax"],
        }

        for dcc_type, keywords in dcc_keywords.items():
            for keyword in keywords:
                if keyword in title_lower or keyword in url_lower:
                    return dcc_type

        return None

    async def get_instance_by_window_id(self, window_id: str) -> Instance | None:
        """Get a specific instance by its window ID.

        Args:
            window_id: The window ID to look for.

        Returns:
            Instance if found, None otherwise.
        """
        instances = await self.discover()
        for inst in instances:
            if inst.window_id == window_id:
                return inst
        return None

    async def get_instance_by_title(self, title: str) -> Instance | None:
        """Get a specific instance by its title (partial match).

        Args:
            title: The title to search for (case-insensitive partial match).

        Returns:
            First matching instance, or None if not found.
        """
        instances = await self.discover()
        title_lower = title.lower()
        for inst in instances:
            if inst.title and title_lower in inst.title.lower():
                return inst
            if inst.app_name and title_lower in inst.app_name.lower():
                return inst
        return None

    async def get_instance_by_dcc(self, dcc_type: str) -> Instance | None:
        """Get a specific instance by DCC type.

        Args:
            dcc_type: The DCC type (maya, blender, houdini, etc.).

        Returns:
            First matching instance, or None if not found.
        """
        instances = await self.discover_dcc_instances()
        dcc_lower = dcc_type.lower()
        for inst in instances:
            if inst.dcc_type and inst.dcc_type.lower() == dcc_lower:
                return inst
        return None
