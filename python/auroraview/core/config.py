# -*- coding: utf-8 -*-
"""WebView configuration dataclasses.

This module provides structured configuration classes to replace the 40+ parameter
__init__ method in WebView. Using dataclasses makes the configuration more readable,
maintainable, and self-documenting.

Example:
    >>> from auroraview.core.config import WebViewConfig, WindowConfig
    >>> config = WebViewConfig(
    ...     window=WindowConfig(title="My App", width=1024, height=768),
    ...     debug=True,
    ... )
    >>> webview = WebView(config=config)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any, Optional, Union

if TYPE_CHECKING:
    from .bridge import Bridge


@dataclass
class WindowConfig:
    """Window appearance and behavior configuration.

    Attributes:
        title: Window title (default: "AuroraView")
        width: Window width in pixels (default: 800)
        height: Window height in pixels (default: 600)
        x: Window x position (optional, centered if None)
        y: Window y position (optional, centered if None)
        icon: Custom window icon path (optional)
        frame: Show window frame/decorations (default: True)
        resizable: Allow window resizing (default: True)
        always_on_top: Keep window above others (default: False)
        transparent: Enable transparent background (default: False)
        background_color: Window background color hex (optional)
        tool_window: Tool window style - hides from taskbar/Alt+Tab (default: False)
        undecorated_shadow: Show shadow for frameless windows (default: False)
    """

    title: str = "AuroraView"
    width: int = 800
    height: int = 600
    x: Optional[int] = None
    y: Optional[int] = None
    icon: Optional[str] = None
    frame: Optional[bool] = None  # None = auto-detect based on other settings
    resizable: bool = True
    always_on_top: bool = False
    transparent: bool = False
    background_color: Optional[str] = None
    tool_window: bool = False
    undecorated_shadow: bool = False


@dataclass
class ContentConfig:
    """Initial content configuration.

    Attributes:
        url: URL to load (optional)
        html: HTML content to load (optional)
        asset_root: Root directory for auroraview:// protocol (optional)
        allow_file_protocol: Enable file:// protocol (default: False, security risk)
    """

    url: Optional[str] = None
    html: Optional[str] = None
    asset_root: Optional[str] = None
    allow_file_protocol: bool = False


@dataclass
class EmbeddingConfig:
    """DCC/Qt embedding configuration.

    Attributes:
        parent: Parent window handle (HWND) for embedding (optional)
        mode: Embedding mode - "auto", "owner", "child", "container" (default: "auto")
        dcc_mode: DCC thread safety mode - "auto", True, or False (default: "auto")
        auto_show: Automatically show after creation (default: True)
        auto_timer: Auto-start event timer for embedded mode (default: True)
    """

    parent: Optional[int] = None
    mode: str = "auto"  # "auto", "owner", "child", "container"
    dcc_mode: Union[bool, str] = "auto"
    auto_show: bool = True
    auto_timer: bool = True

    # Aliases for backward compatibility
    parent_hwnd: Optional[int] = None
    embed_mode: Optional[str] = None

    def __post_init__(self) -> None:
        """Resolve parameter aliases."""
        # parent_hwnd is alias for parent
        if self.parent_hwnd is not None and self.parent is None:
            self.parent = self.parent_hwnd
        # embed_mode is alias for mode
        if self.embed_mode is not None and self.mode == "auto":
            self.mode = self.embed_mode


@dataclass
class DownloadConfig:
    """Download behavior configuration.

    Attributes:
        allow_downloads: Enable file downloads (default: True)
        download_prompt: Show "Save As" dialog (default: False)
        download_directory: Default download directory (optional)
    """

    allow_downloads: bool = True
    download_prompt: bool = False
    download_directory: Optional[str] = None


@dataclass
class NetworkConfig:
    """Network configuration.

    Attributes:
        proxy_url: Proxy server URL (optional, e.g., "http://127.0.0.1:8080")
        user_agent: Custom User-Agent string (optional)
    """

    proxy_url: Optional[str] = None
    user_agent: Optional[str] = None


@dataclass
class DebugConfig:
    """Debug and development configuration.

    Attributes:
        debug: Enable developer tools (default: True)
        context_menu: Enable native context menu (default: True)
        remote_debugging_port: CDP remote debugging port (optional)
        splash_overlay: Show splash overlay while loading (default: False)
    """

    debug: bool = True
    context_menu: bool = True
    remote_debugging_port: Optional[int] = None
    splash_overlay: bool = False

    # Aliases for backward compatibility
    dev_tools: Optional[bool] = None
    decorations: Optional[bool] = None

    def __post_init__(self) -> None:
        """Resolve parameter aliases."""
        # dev_tools is alias for debug
        if self.dev_tools is not None:
            self.debug = self.dev_tools


@dataclass
class NewWindowConfig:
    """New window (window.open) configuration.

    Attributes:
        allow_new_window: Allow window.open() to create windows (default: False)
        new_window_mode: Behavior - "deny", "system_browser", "child_webview" (optional)
    """

    allow_new_window: bool = False
    new_window_mode: Optional[str] = None


@dataclass
class WebViewConfig:
    """Complete WebView configuration.

    This dataclass consolidates all WebView configuration into a structured format,
    replacing the 40+ parameter __init__ method.

    Attributes:
        window: Window appearance and behavior settings
        content: Initial content configuration
        embedding: DCC/Qt embedding settings
        download: Download behavior settings
        network: Network configuration
        debug: Debug and development settings
        new_window: New window handling settings
        bridge: Bridge instance for DCC integration (optional)
        data_directory: User data directory for WebView (optional)
        ipc_batch_size: Max IPC messages per tick, 0 = unlimited (default: 0)
        singleton: Singleton key for single-instance mode (optional)

    Example:
        >>> config = WebViewConfig(
        ...     window=WindowConfig(
        ...         title="My DCC Tool",
        ...         width=1024,
        ...         height=768,
        ...         always_on_top=True,
        ...     ),
        ...     content=ContentConfig(
        ...         url="http://localhost:3000",
        ...         asset_root="/path/to/assets",
        ...     ),
        ...     embedding=EmbeddingConfig(
        ...         parent=maya_hwnd,
        ...         mode="owner",
        ...     ),
        ...     debug=DebugConfig(debug=True),
        ... )
        >>> webview = WebView(config=config)
    """

    window: WindowConfig = field(default_factory=WindowConfig)
    content: ContentConfig = field(default_factory=ContentConfig)
    embedding: EmbeddingConfig = field(default_factory=EmbeddingConfig)
    download: DownloadConfig = field(default_factory=DownloadConfig)
    network: NetworkConfig = field(default_factory=NetworkConfig)
    debug: DebugConfig = field(default_factory=DebugConfig)
    new_window: NewWindowConfig = field(default_factory=NewWindowConfig)

    # Top-level options
    bridge: Union["Bridge", bool, None] = None
    data_directory: Optional[str] = None
    ipc_batch_size: int = 0
    singleton: Optional[str] = None

    @classmethod
    def from_kwargs(cls, **kwargs: Any) -> "WebViewConfig":
        """Create WebViewConfig from flat keyword arguments.

        This factory method provides backward compatibility by converting
        the old-style flat parameter list into the new structured format.

        Args:
            **kwargs: Flat keyword arguments matching the old WebView.__init__

        Returns:
            WebViewConfig instance

        Example:
            >>> config = WebViewConfig.from_kwargs(
            ...     title="My App",
            ...     width=1024,
            ...     debug=True,
            ...     parent=maya_hwnd,
            ... )
        """
        # Extract window config
        window = WindowConfig(
            title=kwargs.get("title", "AuroraView"),
            width=kwargs.get("width", 800),
            height=kwargs.get("height", 600),
            icon=kwargs.get("icon"),
            frame=kwargs.get("frame", kwargs.get("decorations")),
            resizable=kwargs.get("resizable", True),
            always_on_top=kwargs.get("always_on_top", False),
            transparent=kwargs.get("transparent", False),
            background_color=kwargs.get("background_color"),
            tool_window=kwargs.get("tool_window", False),
            undecorated_shadow=kwargs.get("undecorated_shadow", False),
        )

        # Extract content config
        content = ContentConfig(
            url=kwargs.get("url"),
            html=kwargs.get("html"),
            asset_root=kwargs.get("asset_root"),
            allow_file_protocol=kwargs.get("allow_file_protocol", False),
        )

        # Extract embedding config
        embedding = EmbeddingConfig(
            parent=kwargs.get("parent", kwargs.get("parent_hwnd")),
            mode=kwargs.get("mode", kwargs.get("embed_mode", "auto")),
            dcc_mode=kwargs.get("dcc_mode", "auto"),
            auto_show=kwargs.get("auto_show", True),
            auto_timer=kwargs.get("auto_timer", True),
        )

        # Extract download config
        download = DownloadConfig(
            allow_downloads=kwargs.get("allow_downloads", True),
            download_prompt=kwargs.get("download_prompt", False),
            download_directory=kwargs.get("download_directory"),
        )

        # Extract network config
        network = NetworkConfig(
            proxy_url=kwargs.get("proxy_url"),
            user_agent=kwargs.get("user_agent"),
        )

        # Extract debug config
        debug_val = kwargs.get("debug", kwargs.get("dev_tools"))
        if debug_val is None:
            debug_val = True
        debug_cfg = DebugConfig(
            debug=debug_val,
            context_menu=kwargs.get("context_menu", True),
            remote_debugging_port=kwargs.get("remote_debugging_port"),
            splash_overlay=kwargs.get("splash_overlay", False),
        )

        # Extract new window config
        new_window = NewWindowConfig(
            allow_new_window=kwargs.get("allow_new_window", False),
            new_window_mode=kwargs.get("new_window_mode"),
        )

        return cls(
            window=window,
            content=content,
            embedding=embedding,
            download=download,
            network=network,
            debug=debug_cfg,
            new_window=new_window,
            bridge=kwargs.get("bridge"),
            data_directory=kwargs.get("data_directory"),
            ipc_batch_size=kwargs.get("ipc_batch_size", 0),
            singleton=kwargs.get("singleton"),
        )

    def to_kwargs(self) -> dict:
        """Convert config to flat keyword arguments for Rust core.

        Returns:
            Dictionary of keyword arguments for _CoreWebView
        """
        # Resolve frame default based on window style
        frame = self.window.frame
        if frame is None:
            if self.window.tool_window or self.window.transparent:
                frame = False
            else:
                frame = True

        return {
            # Window
            "title": self.window.title,
            "width": self.window.width,
            "height": self.window.height,
            "icon": self.window.icon,
            "decorations": frame,
            "resizable": self.window.resizable,
            "always_on_top": self.window.always_on_top,
            "transparent": self.window.transparent,
            "background_color": self.window.background_color,
            "tool_window": self.window.tool_window,
            "undecorated_shadow": self.window.undecorated_shadow,
            # Content
            "url": self.content.url,
            "html": self.content.html,
            "asset_root": self.content.asset_root,
            "allow_file_protocol": self.content.allow_file_protocol,
            # Embedding
            "parent_hwnd": self.embedding.parent,
            "parent_mode": self.embedding.mode if self.embedding.parent else None,
            "auto_show": self.embedding.auto_show,
            # Download
            "allow_downloads": self.download.allow_downloads,
            "download_prompt": self.download.download_prompt,
            "download_directory": self.download.download_directory,
            # Network
            "proxy_url": self.network.proxy_url,
            "user_agent": self.network.user_agent,
            # Debug
            "dev_tools": self.debug.debug,
            "context_menu": self.debug.context_menu,
            "remote_debugging_port": self.debug.remote_debugging_port,
            "splash_overlay": self.debug.splash_overlay,
            # New window
            "allow_new_window": self.new_window.allow_new_window,
            "new_window_mode": self.new_window.new_window_mode,
            # Other
            "data_directory": self.data_directory,
            "ipc_batch_size": self.ipc_batch_size,
        }
