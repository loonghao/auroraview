"""AuroraView instance discovery module."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field
from typing import Any

import httpx


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
        }


@dataclass
class InstanceDiscovery:
    """AuroraView instance discovery service."""

    default_ports: list[int] = field(default_factory=lambda: [9222, 9223, 9224, 9225])
    timeout: float = 1.0

    async def discover(self, ports: list[int] | None = None) -> list[Instance]:
        """Discover all running AuroraView instances.

        Args:
            ports: List of ports to scan. If None, uses default ports.

        Returns:
            List of discovered instances.
        """
        scan_ports = ports or self.default_ports
        tasks = [self._probe_port(port) for port in scan_ports]
        results = await asyncio.gather(*tasks, return_exceptions=True)

        instances = []
        for result in results:
            if isinstance(result, Instance):
                instances.append(result)

        return instances

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

        # Enrich with DCC context
        enriched = []
        for instance in instances:
            enriched_instance = await self._enrich_dcc_context(instance)
            enriched.append(enriched_instance)

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
