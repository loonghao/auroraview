# -*- coding: utf-8 -*-
"""High-level Python API for WebView."""

from __future__ import annotations

import json
import logging
import threading
from dataclasses import MISSING, dataclass, fields
from pathlib import Path
from typing import TYPE_CHECKING, Any, Callable, ClassVar, Dict, List, Optional, Union

try:
    from typing import Literal  # py3.8+
except ImportError:  # pragma: no cover - py3.7 fallback
    # `typing_extensions` is declared as a conditional dependency in
    # ``pyproject.toml`` (``python_version<'3.8'``), so this import is
    # guaranteed to succeed on the only Python version that needs it.
    from typing_extensions import Literal  # type: ignore

# Import Mixin classes
from auroraview.core.mixins import (
    WebViewApiMixin,
    WebViewContentMixin,
    WebViewDOMMixin,
    WebViewEventMixin,
    WebViewJSMixin,
    WebViewTelemetryMixin,
    WebViewWindowMixin,
)

if TYPE_CHECKING:
    from .bridge import Bridge
    from .channel import Channel, ChannelManager
    from .commands import CommandRegistry
    from .config import WebViewConfig
    from .ready_events import ReadyEvents
    from .state import State

_CORE_IMPORT_ERROR = None
_IS_PACKED_MODE = False
try:
    from auroraview._core import WebView as _CoreWebView
except ImportError as e:
    _CoreWebView = None
    _CORE_IMPORT_ERROR = str(e)
    # Check if running in packed mode where _core.pyd is not needed
    import os

    _IS_PACKED_MODE = os.environ.get("AURORAVIEW_PACKED", "0") == "1"

logger = logging.getLogger(__name__)


# ----------------------------------------------------------------------------
# Mixin layout note
# ----------------------------------------------------------------------------
# Historically WebView was assembled out of ~12 Mixin classes (Window / Content /
# JS / Event / Api / DOM / Telemetry / Lifecycle / State / Commands / Channels /
# Factory / Bridge). The lifecycle, state, commands, channels, factory and
# bridge mixins were inlined directly into ``WebView`` because:
#
#   * Their methods touched private ``_core`` / ``_async_core`` /
#     ``_async_core_lock`` state owned by ``__init__``, so each Mixin had to
#     re-declare those attributes in stubs anyway — defeating the
#     "small focused mixin" goal.
#   * Pyright / mypy could not follow the cross-mixin attribute access pattern
#     and emitted a flood of false-positive ``reportPrivateUsage`` /
#     ``attr-defined`` warnings.
#   * The lifecycle methods (``show`` / ``show_async`` / ``close`` / ``wait``)
#     and factory (``WebView.create``) need the concrete class for
#     ``Self``-typed return values; keeping them in mixins forced
#     ``TYPE_CHECKING`` gymnastics on every call site.
#
# The remaining mixins (Window / Content / JS / Event / Api / DOM / Telemetry)
# are pure capability bundles with no shared state coupling — they stay as
# mixins. See ``auroraview/core/mixins/__init__.py`` for the up-to-date list.
# ----------------------------------------------------------------------------


# ----------------------------------------------------------------------------
# Construction state
# ----------------------------------------------------------------------------
# ``_WebViewInitState`` is the single source of truth for the inputs that
# :meth:`WebView._init_runtime_state` consumes. Keeping it as a dataclass
# (instead of a long ``**kwargs`` form) gives us three concrete properties
# that the previous "21 keyword arguments" version did not have:
#
#   * **Adding a field is a one-liner.** Drop a new attribute here, decide
#     where it should be assigned in ``_init_runtime_state``, done. The
#     ``__init__`` and :meth:`WebView.create_embedded` paths both fan in
#     through this dataclass via :meth:`_WebViewInitState.from_init_kwargs`,
#     so they cannot drift on which fields they forward — the type system
#     rejects any mismatch at construction time.
#   * **Defaults are authoritative here.** The public ``WebView.__init__``
#     signature mirrors these defaults for IDE / autocomplete ergonomics,
#     but any caller that goes through :meth:`from_init_kwargs` and omits
#     a field will pick up the dataclass default. If the two ever
#     disagree, **the dataclass wins** — that is what makes
#     ``create_embedded`` (which only forwards a subset of kwargs)
#     produce a structurally identical instance to ``__init__``.
#   * **It is the right home for "wiring" inputs that aren't part of the
#     visible ``WebView(...)`` ergonomics**, such as ``asset_root`` —
#     which the embedded-host path needs to forward to the Rust core but
#     which the standard ``__init__`` already passes via the Rust core
#     constructor itself (except in packed mode, where the core
#     constructor is skipped entirely; ``from_init_kwargs`` handles
#     that fork in one place).
#
# Field ordering follows the public ``__init__`` signature so reviewers can
# diff the two sources visually.
@dataclass
class _WebViewInitState:
    """Aggregated inputs for :meth:`WebView._init_runtime_state`.

    Construction-path-agnostic snapshot of every value that participates
    in pure attribute assignment (``self._title = ...``) plus the small
    set of "core wiring" calls (``core.set_asset_root(...)``) that must
    run on every construction path. The lifecycle phase
    (:meth:`WebView._init_lifecycle`) reads only ``self`` and therefore
    does not appear here.

    All fields are required-by-construction unless they have a default —
    see :meth:`from_init_kwargs` for the canonical funnel both
    ``__init__`` and ``create_embedded`` use.
    """

    # Visible window / content properties. Required: the standard
    # ``__init__`` always passes them and ``create_embedded`` has its
    # own concrete values for them.
    title: str
    width: int
    height: int
    url: Optional[str]
    html: Optional[str]
    debug: bool
    resizable: bool
    frame: bool
    parent: Optional[int]
    mode: Optional[str]

    # Window style / behaviour. Defaults are authoritative here; the
    # public ``WebView.__init__`` signature mirrors them.
    always_on_top: bool = False
    transparent: bool = False
    background_color: Optional[str] = None
    tool_window: bool = False
    undecorated_shadow: bool = False
    allow_new_window: bool = False
    new_window_mode: Optional[str] = None
    remote_debugging_port: Optional[int] = None
    splash_overlay: bool = False
    auto_show: bool = True
    dcc_mode: Union[bool, str] = "auto"

    # Bridge integration. ``Bridge`` is only available under
    # ``TYPE_CHECKING`` so we string-quote it. ``= None`` is the right
    # default form here (mutable-default restrictions only apply to
    # list/dict/set, not to ``None``); we used to wrap this in
    # ``field(default=None)`` but the wrapper added no value.
    bridge: Union["Bridge", bool, None] = None  # type: ignore[name-defined]

    # Asset root for the auroraview:// protocol. Standard ``__init__``
    # forwards this to the Rust core via the core constructor *except*
    # in packed mode (where ``self._core`` stays ``None`` and the core
    # constructor is skipped entirely). :meth:`from_init_kwargs`
    # decides whether to populate this field via the
    # ``forward_asset_root_to_core`` switch; :meth:`_init_runtime_state`
    # owns the single ``core.set_asset_root`` call site.
    asset_root: Optional[str] = None

    @classmethod
    def from_init_kwargs(
        cls,
        *,
        asset_root: Optional[str] = None,
        forward_asset_root_to_core: bool = False,
        strict: bool = True,
        **kwargs: Any,
    ) -> "_WebViewInitState":
        """Build a state object from a public-``__init__``-shaped kwargs dict.

        This is the single funnel both :meth:`WebView.__init__` and
        :meth:`WebView.create_embedded` use to construct a
        ``_WebViewInitState``. Centralising it here gives us:

          * **Field filtering for free.** Callers can pass kwargs that
            are not dataclass fields (e.g. things ``__init__`` forwards
            to ``_CoreWebView`` only) without us having to enumerate
            them at every call site. By default (``strict=True``)
            unknown keys raise ``TypeError`` to catch typos like
            ``parent_hwnd=`` instead of ``parent=`` immediately;
            external callers feeding user-controlled kwargs can opt
            into ``strict=False`` for silent filtering.
          * **Sentinel ``None`` semantics.** Callers that want to
            "leave a field at its dataclass default" can simply pass
            ``None`` (or omit it). Any explicit non-``None`` value
            wins over the dataclass default. **This only applies to
            fields with a dataclass default**; required fields keep
            ``None`` as a real value (their type is ``Optional[X]``).
          * **One place to decide who calls ``set_asset_root``.**
            ``asset_root`` can reach the Rust core in two different
            ways depending on the construction path:

              - Standard ``__init__`` in non-packed mode forwards it
                via ``_CoreWebView(asset_root=..., ...)``; we must
                NOT call ``core.set_asset_root`` again.
              - Standard ``__init__`` in packed mode skips the core
                constructor entirely (``self._core is None``); we
                need ``_init_runtime_state`` to no-op gracefully.
              - ``create_embedded`` builds the core via
                ``_CoreWebView.create_embedded(...)`` which doesn't
                accept ``asset_root`` — only the post-construction
                ``set_asset_root`` call inside ``_init_runtime_state``
                can apply it.

            ``forward_asset_root_to_core`` collapses the second and
            third cases into one switch: pass ``True`` whenever the
            active path did NOT already hand ``asset_root`` to the
            core constructor. The previous design called this same
            switch ``is_packed`` and forced ``create_embedded`` to
            lie about being "packed" just to flip it; the new name
            describes the actual contract.

        Args:
            asset_root: Asset-root value as passed by the public API.
                Only matters when ``forward_asset_root_to_core`` is
                ``True``; otherwise dropped (the core constructor
                already received it).
            forward_asset_root_to_core: ``True`` when the active
                construction path did NOT pass ``asset_root`` to the
                Rust core constructor and therefore needs
                :meth:`_init_runtime_state` to make the
                ``core.set_asset_root`` call. Set this whenever:

                  * standard ``__init__`` is running in packed mode
                    (``self._core is None``), or
                  * ``create_embedded`` is the caller (always — the
                    embedded core constructor doesn't accept
                    ``asset_root``).

                Set to ``False`` for the standard non-packed
                ``__init__`` path (the default).
            strict: When ``True`` (default), raise ``TypeError`` on
                kwargs that aren't ``_WebViewInitState`` fields. The
                two internal call sites pass ``True`` so a typo like
                ``parent_hwnd=...`` fails loudly instead of being
                silently dropped (which would leave ``state.parent``
                as ``None`` and produce a hard-to-debug "embedded
                webview never reparented" symptom). Set to ``False``
                only when forwarding user-controlled kwargs.
            **kwargs: Public-``__init__`` keyword arguments. ``None``
                is treated as a "use the dataclass default" sentinel
                **only for fields that have a default**; required
                fields (e.g. ``url``, ``html``, ``parent``, ``mode``)
                keep ``None`` as a real value.

        Returns:
            A fully-populated ``_WebViewInitState`` instance.

        Raises:
            TypeError: If ``strict=True`` and ``kwargs`` contains any
                key that is not a ``_WebViewInitState`` field.
        """
        # Split fields into "required" (no default) vs "optional"
        # (has default). ``None`` has different meaning in each:
        #   * required field: ``None`` is a real value (the type
        #     signature says ``Optional[X]``, e.g. ``url``, ``parent``).
        #     We must keep it.
        #   * optional field: ``None`` is a sentinel for "use the
        #     dataclass default". Drop it so the dataclass default
        #     fires, which is the whole point of the funnel.
        #
        # **Invariant**: required fields MUST NOT gain a default
        # later. If they do, the None-sentinel rule above silently
        # changes meaning (e.g. ``debug=None`` would suddenly fall
        # through to ``False`` instead of staying ``None``). The
        # ``test_optional_none_falls_to_default`` regression test
        # pins this assumption.
        required_keys = {f.name for f in fields(cls) if f.default is MISSING}
        optional_keys = {f.name for f in fields(cls) if f.default is not MISSING}
        valid_keys = required_keys | optional_keys

        if strict:
            unknown = set(kwargs) - valid_keys
            if unknown:
                raise TypeError(
                    f"_WebViewInitState.from_init_kwargs got unexpected "
                    f"keyword argument(s): {sorted(unknown)}. "
                    f"Valid fields: {sorted(valid_keys)}."
                )

        filtered: Dict[str, Any] = {}
        for k, v in kwargs.items():
            if k not in valid_keys:
                continue  # Silently drop unknown keys (strict=False path).
            if k in optional_keys and v is None:
                continue  # Sentinel: use dataclass default.
            filtered[k] = v
        # Asset-root fork: see class docstring + the
        # ``set_asset_root`` block in ``_init_runtime_state``.
        if forward_asset_root_to_core and asset_root is not None:
            filtered["asset_root"] = asset_root
        return cls(**filtered)


# ----------------------------------------------------------------------------


class WebView(
    WebViewWindowMixin,
    WebViewContentMixin,
    WebViewJSMixin,
    WebViewEventMixin,
    WebViewApiMixin,
    WebViewDOMMixin,
    WebViewTelemetryMixin,
):
    """High-level WebView class with enhanced Python API.

    This class wraps the Rust core WebView implementation and provides
    a more Pythonic interface with additional features.

    Args:
        title: Window title (default: "AuroraView")
        width: Window width in pixels (default: 800)
        height: Window height in pixels (default: 600)
        url: URL to load (optional)
        html: HTML content to load (optional)
        debug: Enable developer tools (default: True)
        context_menu: Enable native context menu (default: True)
        resizable: Make window resizable (default: True)
        frame: Show window frame (title bar, borders) (default: True)
        parent: Parent window handle for embedding (optional)
        mode: Embedding mode - "child" or "owner" (optional, Windows only)
              "owner" is safer for cross-thread usage
              "child" requires same-thread parenting

    Example:
        >>> # Standalone window
        >>> webview = WebView(title="My Tool", width=1024, height=768)
        >>> webview.load_url("http://localhost:3000")
        >>> webview.show()

        >>> # DCC integration (e.g., Maya)
        >>> import maya.OpenMayaUI as omui
        >>> maya_hwnd = int(omui.MQtUtil.mainWindow())
        >>> webview = WebView(title="My Tool", parent=maya_hwnd, mode="owner")
        >>> webview.show()

        >>> # Disable native context menu for custom menu
        >>> webview = WebView(title="My Tool", context_menu=False)
        >>> webview.show()
    """

    # Class-level singleton registry using weak references
    _singleton_registry: Dict[str, "WebView"] = {}

    def __init__(
        self,
        title: str = "AuroraView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        debug: Optional[bool] = None,
        context_menu: bool = True,
        resizable: bool = True,
        frame: Optional[bool] = None,
        parent: Optional[int] = None,
        mode: Optional[str] = None,
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        dev_tools: Optional[bool] = None,
        decorations: Optional[bool] = None,
        asset_root: Optional[str] = None,
        data_directory: Optional[str] = None,
        allow_file_protocol: bool = False,
        always_on_top: bool = False,
        transparent: bool = False,
        background_color: Optional[str] = None,
        auto_show: bool = True,
        ipc_batch_size: int = 0,
        icon: Optional[str] = None,
        tool_window: bool = False,
        undecorated_shadow: bool = False,
        allow_new_window: bool = False,
        new_window_mode: Optional[str] = None,
        remote_debugging_port: Optional[int] = None,
        splash_overlay: bool = False,
        allow_downloads: bool = True,
        download_prompt: bool = False,
        download_directory: Optional[str] = None,
        proxy_url: Optional[str] = None,
        user_agent: Optional[str] = None,
        # Aliases for more intuitive API
        parent_hwnd: Optional[int] = None,
        embed_mode: Optional[str] = None,
        # DCC thread safety
        dcc_mode: Union[bool, str] = "auto",
        # New structured config (takes precedence if provided)
        config: Optional["WebViewConfig"] = None,
    ) -> None:
        r"""Initialize the WebView.

               Args:
                   title: Window title
                   width: Window width in pixels
                   height: Window height in pixels
                   url: URL to load (optional)
                   html: HTML content to load (optional)
                   debug: Enable developer tools (default: True). Press F12 or right-click
                       > Inspect to open DevTools.
                   context_menu: Enable native context menu (default: True)
                   resizable: Make window resizable (default: True)
                   frame: Show window frame (title bar, borders). If None, uses sensible defaults.
        (default: True)
                   parent: Parent window handle for embedding (optional)
                   mode: Embedding mode - "child" or "owner" (optional)
                   bridge: Bridge instance for DCC integration
                          - Bridge instance: Use provided bridge
                          - True: Auto-create bridge with default settings
                          - None: No bridge (default)
                   asset_root: Root directory for auroraview:// protocol.
                       When set, enables the auroraview:// custom protocol for secure
                       local resource loading. Files under this directory can be accessed
                       using URLs like ``auroraview://path/to/file``.

                       **Platform-specific URL format**:

                       - Windows: ``https://auroraview.localhost/path``
                       - macOS/Linux: ``auroraview://path``

                       **Security**: Uses ``.localhost`` TLD (IANA reserved, RFC 6761)
                       which cannot be registered and is treated as a local address.
                       Requests are intercepted before DNS resolution.

                       **Recommended** over ``allow_file_protocol=True`` because access
                       is restricted to the specified directory only.

                   data_directory: User data directory for WebView (cookies, cache, localStorage).
                       If None, uses system default (usually %LOCALAPPDATA%\...\EBWebView on Windows).
                       Set this to isolate WebView data per application or user profile.

                   allow_file_protocol: Enable file:// protocol support (default: False).
                       **WARNING**: Enabling this allows access to ANY file on the system
                       that the process can read. Only use with trusted content.
                       Prefer using ``asset_root`` for secure local resource loading.

                   always_on_top: Keep window always on top of other windows (default: False).
                       Useful for floating tool panels or overlay windows.

                   tool_window: Apply tool window style (default: False, Windows only).
                       When enabled, the window:
                       - Does NOT appear in the taskbar
                       - Does NOT appear in Alt+Tab window switcher
                       - Has a smaller title bar (if decorations are enabled)

                       This is commonly used with ``embed_mode="owner"`` for floating tool windows.

                       See: https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles

                   undecorated_shadow: Show shadow for frameless windows (default: False, Windows only).
                       When ``frame=False``, Windows can still show a subtle shadow around the window.
                       Set this to ``True`` to explicitly enable the shadow.

                       truly transparent frameless windows (e.g., floating logo buttons).

                       Example::

                           # Transparent logo button with no shadow
                           webview = WebView(
                               html=LOGO_HTML,
                               width=64,
                               height=64,
                               frame=False,
                               transparent=True,
                               undecorated_shadow=False,
                               tool_window=True,
                           )

                   remote_debugging_port: Enable Chrome DevTools Protocol (CDP) remote debugging.
                       When set, the WebView will listen on the specified port for CDP connections.
                       This allows tools like MCP servers, Playwright, or Chrome DevTools to connect
                       and control the WebView remotely.

                       Example::

                           # Enable CDP on port 9222
                           webview = WebView(
                               title="Debug Me",
                               remote_debugging_port=9222,
                           )
                           # Connect with: chrome://inspect or ws://localhost:9222

                   splash_overlay: Show splash overlay while loading URL (default: False).
                       When enabled, displays an animated Aurora-themed loading overlay
                       that automatically fades out when the page is fully loaded.
                       Useful for slow network loads or branded loading experience.

                       Example::

                           # Show splash while loading external URL
                           webview = WebView(
                               url="https://example.com",
                               splash_overlay=True,
                           )

                  allow_downloads: Enable file downloads (default: True).
                      AuroraView enables downloads by default for better user experience.
                      When enabled, allows file downloads from the WebView.

                  download_prompt: Show "Save As" dialog for downloads (default: False).
                      When True, prompts user to choose save location like a browser.
                      When False, downloads go directly to download_directory.

                  download_directory: Default download directory (optional).
                      When set, downloaded files are saved to this directory.
                      Uses system default Downloads folder (e.g., ~/Downloads) if not set.
                      Ignored when download_prompt is True.

                   proxy_url: Proxy server URL (optional).
                       Format: "http://host:port" or "socks5://host:port"
                       When set, all WebView network requests go through this proxy.

                       Example::

                           # Use HTTP proxy
                           webview = WebView(
                               url="https://example.com",
                               proxy_url="http://127.0.0.1:8080",
                           )

                           # Use SOCKS5 proxy
                           webview = WebView(
                               url="https://example.com",
                               proxy_url="socks5://127.0.0.1:1080",
                           )

                   user_agent: Custom User-Agent string (optional).
                       When set, overrides the default browser User-Agent.

                   parent_hwnd: Alias for ``parent`` parameter (for backward compatibility).
                   embed_mode: Alias for ``mode`` parameter. Values: "child", "owner".

                       - **child**: Embed WebView inside a Qt widget (WS_CHILD).
                         Window is clipped to parent bounds, cannot be moved independently.

                       - **owner**: Create floating tool window (GWLP_HWNDPARENT).
                         Window stays above owner in Z-order, hidden when owner minimizes.

                       See: https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features

                   dcc_mode: DCC thread safety mode (default: "auto").
                       Controls whether event handlers are automatically wrapped to run
                       on the DCC main thread. This is essential for Maya, Blender, etc.

                       Values:
                           - ``"auto"``: Automatically detect DCC environment (default).
                             Thread safety is enabled only when running inside a DCC
                             application (Maya, Blender, Houdini, 3ds Max, Nuke, Unreal).
                           - ``True``: Always enable thread safety.
                           - ``False``: Never enable thread safety (for standalone apps).

                       With ``"auto"`` (default), you don't need to specify anything:

                           # In Maya - automatically thread-safe
                           webview = WebView(parent=maya_hwnd)

                           @webview.on("create_object")
                           def handle_create(data):
                               # Automatically runs on Maya main thread!
                               import maya.cmds as cmds
                               return cmds.polyCube()[0]
        """
        if _CoreWebView is None:
            # In packed mode, _core.pyd is not needed - Python runs as API server
            if _IS_PACKED_MODE:
                logger.info("Packed mode: _core.pyd not available, WebView will run as API server")
            else:
                import sys

                error_details = [
                    "AuroraView core library not found.",
                    f"Import error: {_CORE_IMPORT_ERROR}",
                    f"Python version: {sys.version}",
                    f"Platform: {sys.platform}",
                ]
                # Check if _core.pyd exists in expected locations
                try:
                    import auroraview

                    pkg_dir = Path(auroraview.__file__).parent
                    pyd_path = pkg_dir / "_core.pyd"
                    so_path = pkg_dir / "_core.so"
                    if pyd_path.exists():
                        error_details.append(f"Found: {pyd_path}")
                    elif so_path.exists():
                        error_details.append(f"Found: {so_path}")
                    else:
                        error_details.append(f"_core.pyd not found in: {pkg_dir}")
                except Exception:
                    pass
                raise RuntimeError("\n".join(error_details))

        # Support new WebViewConfig if provided
        if config is not None:
            # Config takes precedence - extract values
            title = config.window.title
            width = config.window.width
            height = config.window.height
            icon = config.window.icon
            frame = config.window.frame
            resizable = config.window.resizable
            always_on_top = config.window.always_on_top
            transparent = config.window.transparent
            background_color = config.window.background_color
            tool_window = config.window.tool_window
            undecorated_shadow = config.window.undecorated_shadow
            url = config.content.url
            html = config.content.html
            asset_root = config.content.asset_root
            allow_file_protocol = config.content.allow_file_protocol
            parent = config.embedding.parent
            mode = config.embedding.mode if config.embedding.parent else None
            dcc_mode = config.embedding.dcc_mode
            auto_show = config.embedding.auto_show
            allow_downloads = config.download.allow_downloads
            download_prompt = config.download.download_prompt
            download_directory = config.download.download_directory
            proxy_url = config.network.proxy_url
            user_agent = config.network.user_agent
            debug = config.debug.debug
            context_menu = config.debug.context_menu
            remote_debugging_port = config.debug.remote_debugging_port
            splash_overlay = config.debug.splash_overlay
            allow_new_window = config.new_window.allow_new_window
            new_window_mode = config.new_window.new_window_mode
            bridge = config.bridge
            data_directory = config.data_directory
            ipc_batch_size = config.ipc_batch_size

        # Backward-compat parameter aliases
        if dev_tools is not None and debug is None:
            debug = dev_tools
        if decorations is not None and frame is None:
            frame = decorations
        if debug is None:
            debug = True

        # Default decorations strategy
        #
        # - Keep explicit overrides: `frame=` / `decorations=` take precedence.
        # - For typical app windows, keep decorations on.
        # - For tool/overlay style windows (tool_window/transparent), default to frameless.
        if frame is None:
            if tool_window or transparent:
                frame = False
            else:
                frame = True

        # Handle parameter aliases for more intuitive API
        # parent_hwnd is alias for parent
        if parent_hwnd is not None and parent is None:
            parent = parent_hwnd
        # embed_mode is alias for mode
        if embed_mode is not None and mode is None:
            mode = embed_mode

        # Map new parameter names to Rust core (which still uses old names)
        # In packed mode, _CoreWebView is not available - Python runs as API server
        if _CoreWebView is not None:
            self._core = _CoreWebView(
                title=title,
                width=width,
                height=height,
                url=url,
                html=html,
                dev_tools=debug,  # debug -> dev_tools
                context_menu=context_menu,
                resizable=resizable,
                decorations=frame,  # frame -> decorations
                parent_hwnd=parent,  # parent -> parent_hwnd
                parent_mode=mode,  # mode -> parent_mode
                asset_root=asset_root,  # Custom protocol asset root
                data_directory=data_directory,  # User data directory (cookies, cache, etc.)
                allow_file_protocol=allow_file_protocol,  # Enable file:// protocol
                always_on_top=always_on_top,  # Keep window always on top
                transparent=transparent,  # Enable transparent window
                background_color=background_color,  # Window background color
                auto_show=auto_show,  # Control window visibility on creation
                ipc_batch_size=ipc_batch_size,  # Max messages per tick (0=unlimited)
                icon=icon,  # Custom window icon path
                tool_window=tool_window,  # Tool window style (hide from taskbar/Alt+Tab)
                undecorated_shadow=undecorated_shadow,  # Show shadow for frameless windows
                allow_new_window=allow_new_window,  # Allow window.open() to create new windows
                new_window_mode=new_window_mode,  # New window behavior: deny, system_browser, child_webview
                remote_debugging_port=remote_debugging_port,  # CDP remote debugging port
                splash_overlay=splash_overlay,  # Show splash overlay while loading
                allow_downloads=allow_downloads,  # Enable file downloads
                download_prompt=download_prompt,  # Show "Save As" dialog for downloads
                download_directory=download_directory,  # Default download directory
                proxy_url=proxy_url,  # Proxy server URL
                user_agent=user_agent,  # Custom User-Agent string
            )
        else:
            self._core = None  # Packed mode: no Rust core needed
        # All non-core attribute initialization is delegated to a single
        # entry point so any future field added to ``_WebViewInitState``
        # is automatically available to alternative construction paths
        # (e.g. :meth:`create_embedded`) — preventing the silent
        # ``AttributeError`` drift we used to get when the two paths
        # initialised disjoint subsets of the same attribute set.
        #
        # ``from_init_kwargs`` performs the field-filtering and the
        # asset-root fork; we just forward everything we have.
        # ``forward_asset_root_to_core`` flips on iff this path skipped
        # the ``_CoreWebView(asset_root=..., ...)`` constructor — which
        # is exactly the packed-mode case (``self._core is None``)
        # where the core constructor was never called.
        # ``strict=True`` (default) makes ``from_init_kwargs`` raise
        # on typos like ``parent_hwnd=`` instead of silently dropping
        # them; we keep the default explicit here for documentation.
        self._init_runtime_state(
            _WebViewInitState.from_init_kwargs(
                asset_root=asset_root,
                forward_asset_root_to_core=(self._core is None),
                title=title,
                width=width,
                height=height,
                url=url,
                html=html,
                debug=debug,
                resizable=resizable,
                frame=frame,
                parent=parent,
                mode=mode,
                always_on_top=always_on_top,
                transparent=transparent,
                background_color=background_color,
                tool_window=tool_window,
                undecorated_shadow=undecorated_shadow,
                allow_new_window=allow_new_window,
                new_window_mode=new_window_mode,
                remote_debugging_port=remote_debugging_port,
                splash_overlay=splash_overlay,
                auto_show=auto_show,
                dcc_mode=dcc_mode,
                bridge=bridge,
            )
        )

    def _init_runtime_state(self, state: _WebViewInitState) -> None:
        """Initialize all runtime / lifecycle attributes.

        This is the single source of truth for the WebView's mutable
        state. It is invoked from :meth:`__init__` after the Rust
        ``_core`` has been created, and from :meth:`create_embedded`
        after the embedded core has been wired up via the special
        ``_CoreWebView.create_embedded`` static path.

        Construction is split into two phases, both owned by this
        single entry point so the two construction paths cannot drift:

          * **Phase 1 (this method body)** — assigns every ``self._...``
            attribute from ``state`` and performs the small set of
            *self-contained* wiring steps that need to run on every
            construction path:

              - ``dcc_mode == "auto"`` resolution (calls
                ``is_dcc_environment`` / ``get_current_dcc_name``;
                read-only against the host process),
              - bridge auto-creation (instantiates :class:`Bridge`
                and calls ``_setup_bridge_integration`` on ``self``),
              - ``core.set_asset_root`` forwarding when the active
                construction path did not already do so via the Rust
                core constructor.

            These are intentionally *not* in :meth:`_init_lifecycle`
            because they only touch ``self`` (or the Rust core that is
            already wired up by phase 1's caller); they do not register
            ``self`` with any external module. **Adding a new attribute?
            Add it to :class:`_WebViewInitState` and assign it here**,
            not at a call site.

          * **Phase 2 (:meth:`_init_lifecycle`, called as the last
            statement of this method)** — owns every step that
            *registers ``self`` with an external module* or starts a
            background helper (ReadyEvents allocation, WindowManager
            registration, lifecycle event subscriptions, telemetry).
            Adding a new "register / subscribe / start background
            helper" step? **Add it to ``_init_lifecycle``, not here.**

        Both phases run unconditionally on every construction path,
        which is what guarantees ``create_embedded`` and standard
        ``__init__`` produce structurally identical instances.
        """
        self._event_handlers: Dict[str, List[Callable]] = {}
        self._event_handlers_lock = threading.Lock()
        self._title = state.title
        self._width = state.width
        self._height = state.height
        self._x: Optional[int] = None
        self._y: Optional[int] = None
        self._debug = state.debug
        self._resizable = state.resizable
        self._frame = state.frame
        self._parent = state.parent
        self._mode = state.mode
        self._always_on_top = state.always_on_top
        self._transparent = state.transparent
        self._background_color = state.background_color
        self._tool_window = state.tool_window
        self._undecorated_shadow = state.undecorated_shadow
        self._allow_new_window = state.allow_new_window
        self._new_window_mode = state.new_window_mode
        self._remote_debugging_port = state.remote_debugging_port
        self._splash_overlay = state.splash_overlay

        # Resolve dcc_mode: "auto" → detect DCC environment
        if state.dcc_mode == "auto":
            from auroraview.utils.thread_dispatcher import is_dcc_environment

            self._dcc_mode = is_dcc_environment()
            if self._dcc_mode:
                from auroraview.utils.thread_dispatcher import get_current_dcc_name

                dcc_name = get_current_dcc_name()
                logger.info(f"DCC mode auto-enabled: {dcc_name} detected")
        else:
            self._dcc_mode = bool(state.dcc_mode)
        self._show_thread: Optional[threading.Thread] = None
        self._is_running = False
        self._auto_timer = None  # Will be set by create() factory method
        self._auto_show = state.auto_show  # Store auto_show setting
        # Store content for async mode (use passed-in values)
        self._stored_url: Optional[str] = state.url
        self._stored_html: Optional[str] = state.html
        # Store the background thread's core instance
        self._async_core: Optional[Any] = None
        self._async_core_lock = threading.Lock()
        # Track if running in blocking event loop (for HWND mode)
        self._in_blocking_event_loop = False
        # Thread-safe HWND cache (for cross-thread access in non-blocking mode)
        self._cached_hwnd: Optional[int] = None
        self._cached_hwnd_lock = threading.Lock()

        # Close requested flag (used to coordinate background-thread WebView lifecycle)
        self._close_requested = False

        # Event processor (strategy pattern for UI framework integration)
        self._event_processor: Optional[Any] = None

        # Post eval_js hook (for Qt integration and testing)
        self._post_eval_js_hook: Optional[Callable[[], None]] = None

        # Bridge integration
        self._bridge: Optional["Bridge"] = None  # type: ignore
        if state.bridge is not None:
            if state.bridge is True:
                # Auto-create bridge with default settings
                from .bridge import Bridge

                self._bridge = Bridge(port=9001)
                logger.info("Auto-created Bridge on port 9001")
            else:
                # Use provided bridge instance
                self._bridge = state.bridge
                logger.info(f"Using provided Bridge: {state.bridge}")

            # Setup bidirectional communication
            if self._bridge:
                self._setup_bridge_integration()

        # Shared state system (lazy initialization)
        self._state: Optional["State"] = None

        # Command registry (lazy initialization)
        self._commands: Optional["CommandRegistry"] = None

        # Channel manager (lazy initialization)
        self._channels: Optional["ChannelManager"] = None

        # Plugin manager for handling plugin:* invoke commands
        self._plugin_manager: Optional[Any] = None

        # Forward ``asset_root`` to the Rust core when the active
        # construction path did not already do so. The standard
        # ``__init__`` passes it via ``_CoreWebView(...)`` and leaves
        # ``state.asset_root`` as ``None``; ``create_embedded`` populates
        # it because the embedded core constructor doesn't accept it.
        # Either way this is the single place that owns the call, so the
        # two paths can never drift.
        if state.asset_root and self._core is not None:
            try:
                self._core.set_asset_root(state.asset_root)
            except Exception as e:
                logger.warning("set_asset_root failed: %s", e)

        # ReadyEvents / WindowManager registration / lifecycle handlers /
        # telemetry — extracted into ``_init_lifecycle`` so the
        # ``create_embedded`` static path runs the same setup. Without
        # this both paths used to drift: ``__init__`` registered the
        # window globally and wired up ReadyEvents, while
        # ``create_embedded`` quietly skipped it, surfacing as
        # ``AttributeError`` on ``ready_events`` and a ghost window
        # missing from ``WindowManager.get_all()``.
        self._init_lifecycle()

    def _init_lifecycle(self) -> None:
        """Initialize lifecycle-coupled side effects.

        This is **phase 2** of construction, invoked as the tail
        statement of :meth:`_init_runtime_state` (which itself is
        called from both :meth:`__init__` and :meth:`create_embedded`).
        Calling it through ``_init_runtime_state`` guarantees both
        construction paths run the same lifecycle setup, with no
        room for one path to silently skip a step.

        It owns every step that has a side effect outside ``self``:

          * :class:`ReadyEvents` allocation (and ``set_created`` event)
          * Global registration with :class:`WindowManager`
          * Lifecycle event handler wiring (``_setup_lifecycle_events``)
          * Auto-telemetry bring-up (``_init_telemetry`` from the
            telemetry mixin)

        **Adding a new "register / subscribe / start background
        helper" step? Add it here, not at a call site** — that is the
        only way to keep both construction paths in sync.

        Preconditions:
            * ``self._core`` is already wired up (set by ``__init__``
              or by ``create_embedded`` before ``_init_runtime_state``
              is invoked).
            * Phase-1 attributes (``_event_handlers``, ``_bridge``,
              ``_window_id`` placeholders, ...) have already been
              assigned in the body of ``_init_runtime_state``.
        """
        # Local imports keep the module-level import graph free of
        # cycles (ready_events / window_manager both import from
        # this module under TYPE_CHECKING).
        from .ready_events import ReadyEvents
        from .window_manager import get_window_manager

        self._window_id: Optional[str] = None
        self._ready_events = ReadyEvents(self)

        # Register with WindowManager so cross-window APIs and
        # ``get_window_manager().get_all()`` see this instance.
        wm = get_window_manager()
        self._window_id = wm.register(self)
        logger.debug(f"WebView registered with WindowManager: {self._window_id}")

        # Mark as created — must run AFTER ``_ready_events`` exists.
        self._ready_events.set_created()

        # Setup lifecycle event handlers (page:load_finish, auroraviewready, ...).
        self._setup_lifecycle_events()

        # Initialize auto-telemetry (after WindowManager registration so
        # the telemetry hooks can resolve ``self._window_id``).
        self._init_telemetry()

    @property
    def state(self) -> "State":
        """Get the shared state container for Python ↔ JavaScript sync.

        Returns:
            State container with dict-like interface

        Example:
            >>> webview.state["user"] = {"name": "Alice"}
            >>> webview.state["theme"] = "dark"
            >>>
            >>> @webview.state.on_change
            >>> def handle_change(key, value, source):
            ...     print(f"{key} = {value} from {source}")
        """
        if self._state is None:
            from .state import State

            self._state = State(self)
        return self._state

    @property
    def commands(self) -> "CommandRegistry":
        """Get the command registry for Python ↔ JavaScript RPC.

        Returns:
            CommandRegistry instance

        Example:
            >>> @webview.commands.register
            >>> def greet(name: str) -> str:
            ...     return f"Hello, {name}!"
        """
        if self._commands is None:
            from .commands import CommandRegistry

            self._commands = CommandRegistry(self)
        return self._commands

    def command(self, func_or_name=None):
        """Decorator to register a command callable from JavaScript.

        This is a convenience shortcut for `webview.commands.register`.

        Args:
            func_or_name: Function to register or custom command name

        Returns:
            Decorated function

        Example:
            >>> @webview.command
            >>> def greet(name: str) -> str:
            ...     return f"Hello, {name}!"
            >>>
            >>> @webview.command("add_numbers")
            >>> def add(x: int, y: int) -> int:
            ...     return x + y
            >>>
            >>> # In JavaScript:
            >>> # const msg = await auroraview.invoke("greet", {name: "World"});
            >>> # const sum = await auroraview.invoke("add_numbers", {x: 1, y: 2});
        """
        return self.commands.register(func_or_name)

    @property
    def channels(self) -> "ChannelManager":
        """Get the channel manager for streaming data.

        Returns:
            ChannelManager instance

        Example:
            >>> channel = webview.channels.create()
            >>> channel.send({"progress": 50})
            >>> channel.send({"progress": 100})
            >>> channel.close()
        """
        if self._channels is None:
            from .channel import ChannelManager

            self._channels = ChannelManager(self)
        return self._channels

    def create_channel(self, channel_id: Optional[str] = None) -> "Channel":
        """Create a new streaming channel.

        This is a convenience shortcut for `webview.channels.create()`.

        Args:
            channel_id: Optional custom channel ID

        Returns:
            New Channel instance

        Example:
            >>> with webview.create_channel() as channel:
            ...     for chunk in read_large_file():
            ...         channel.send(chunk)
        """
        return self.channels.create(channel_id)

    @classmethod
    def create(
        cls,
        title: str = "AuroraView",
        *,
        # Content
        url: Optional[str] = None,
        html: Optional[str] = None,
        # Window properties
        width: int = 800,
        height: int = 600,
        resizable: bool = True,
        frame: Optional[bool] = None,
        always_on_top: bool = False,
        transparent: bool = False,
        background_color: Optional[str] = None,
        # DCC integration
        parent: Optional[int] = None,
        mode: Literal["auto", "owner", "child", "container"] = "auto",
        # Bridge integration
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        # Development options
        debug: bool = True,
        context_menu: bool = True,
        # Custom protocol
        asset_root: Optional[str] = None,
        data_directory: Optional[str] = None,
        allow_file_protocol: bool = False,
        # Automation
        auto_show: bool = False,
        auto_timer: bool = True,
        # Singleton control
        singleton: Optional[str] = None,
        # IPC performance tuning
        ipc_batch_size: int = 0,
        # Custom icon
        icon: Optional[str] = None,
        # Window style
        tool_window: bool = False,
        undecorated_shadow: bool = False,
        # New window handling
        allow_new_window: bool = False,
        new_window_mode: Optional[str] = None,
        # Remote debugging
        remote_debugging_port: Optional[int] = None,
    ) -> "WebView":
        """Create WebView instance (recommended way).

        Args:
            title: Window title
            url: URL to load
            html: HTML content to load
            width: Window width in pixels
            height: Window height in pixels
            resizable: Make window resizable
            frame: Show window frame (title bar, borders). If None, uses sensible defaults.

            always_on_top: Keep window always on top of other windows (default: False)
            parent: Parent window handle for DCC embedding
            mode: Embedding mode
                - "auto": Auto-select (recommended)
                - "owner": Owner mode (cross-thread safe)
                - "child": Child window mode (same-thread)
            bridge: Bridge for DCC/Web integration
                - Bridge instance: Use provided bridge
                - True: Auto-create bridge (port 9001)
                - None: No bridge (default)
            debug: Enable developer tools
            context_menu: Enable native context menu (default: True)
            asset_root: Root directory for auroraview:// protocol
            data_directory: User data directory for WebView (cookies, cache, localStorage).
                If None, uses system default. Set to isolate data per app/user profile.
            allow_file_protocol: Enable file:// protocol support (default: False)
                WARNING: Enabling this bypasses WebView's default security restrictions
            auto_show: Automatically show after creation
            auto_timer: Auto-start event timer for embedded mode (recommended)
            singleton: Singleton key. If provided, only one instance with this key
                      can exist at a time. Calling create() again with the same key
                      returns the existing instance.

        Returns:
            WebView instance

        Examples:
            >>> # Standalone window
            >>> webview = WebView.create("My App", url="http://localhost:3000")
            >>> webview.show()

            >>> # DCC embedding (Maya)
            >>> webview = WebView.create("Maya Tool", parent=maya_hwnd)
            >>> webview.show()

            >>> # With Bridge integration
            >>> webview = WebView.create("Photoshop Tool", bridge=True)
            >>> @webview.bridge.on('layer_created')
            >>> async def handle_layer(data, client):
            ...     return {"status": "ok"}
            >>> webview.show()

            >>> # Auto-show
            >>> webview = WebView.create("App", auto_show=True)

            >>> # Singleton mode - only one instance allowed

        Note:
            For Qt-based DCC applications (Maya, Houdini, Nuke), consider using
            QtWebView instead for automatic event processing and better integration:

            >>> from auroraview import QtWebView
            >>> webview = QtWebView(parent=maya_main_window(), title="My Tool")
            >>> webview.load_url("http://localhost:3000")
            >>> webview.show()  # Automatic event processing!
            >>> webview1 = WebView.create("Tool", singleton="my_tool")
            >>> webview2 = WebView.create("Tool", singleton="my_tool")  # Returns webview1
            >>> assert webview1 is webview2
        """
        # Check singleton registry
        if singleton is not None:
            if singleton in cls._singleton_registry:
                existing = cls._singleton_registry[singleton]
                logger.info(f"Returning existing singleton instance: '{singleton}'")
                return existing
            logger.info(f"Creating new singleton instance: '{singleton}'")
        # Detect mode
        is_embedded = parent is not None

        # Auto-select mode
        if mode == "auto":
            actual_mode = "owner" if is_embedded else None
            if is_embedded:
                logger.info(f"[AUTO-DETECT] parent={parent} detected, auto-selecting mode='owner'")
        else:
            actual_mode = mode if is_embedded else None
            if is_embedded:
                logger.info(f"[MANUAL] Using user-specified mode='{mode}'")

        logger.info(f"[MODE] Final mode: {actual_mode} (embedded={is_embedded})")

        # Create instance
        # For embedded mode, always set auto_show=False to let Qt control visibility
        # For standalone mode, use the user-provided auto_show value
        rust_auto_show = False if is_embedded else auto_show
        instance = cls(
            title=title,
            width=width,
            height=height,
            url=url,
            html=html,
            resizable=resizable,
            frame=frame,
            always_on_top=always_on_top,
            transparent=transparent,
            background_color=background_color,
            parent=parent,
            mode=actual_mode,
            debug=debug,
            context_menu=context_menu,
            bridge=bridge,
            asset_root=asset_root,
            data_directory=data_directory,
            allow_file_protocol=allow_file_protocol,
            auto_show=rust_auto_show,  # Pass to Rust layer
            ipc_batch_size=ipc_batch_size,  # Max messages per tick (0=unlimited)
            icon=icon,  # Custom window icon path
            tool_window=tool_window,  # Tool window style
            undecorated_shadow=undecorated_shadow,  # Shadow for frameless windows
            allow_new_window=allow_new_window,  # Allow window.open()
            new_window_mode=new_window_mode,  # New window behavior
            remote_debugging_port=remote_debugging_port,  # CDP debugging port
        )

        # Auto timer (embedded mode)
        if is_embedded and auto_timer:
            try:
                from auroraview.utils.event_timer import EventTimer

                instance._auto_timer = EventTimer(instance, interval_ms=16)
                instance._auto_timer.on_close(lambda: instance._auto_timer.stop())
                logger.info("Auto timer created for embedded mode")
            except ImportError as e:
                logger.warning("EventTimer not available: %s, auto_timer disabled", e)
                instance._auto_timer = None
        else:
            instance._auto_timer = None

        # Register singleton
        if singleton is not None:
            cls._singleton_registry[singleton] = instance
            logger.info(f"Registered singleton instance: '{singleton}'")

        # Auto show (only for standalone mode, embedded mode is controlled by Qt)
        if auto_show and not is_embedded:
            instance.show()

        return instance

    @classmethod
    def run_embedded(
        cls,
        title: str = "AuroraView",
        *,
        url: Optional[str] = None,
        html: Optional[str] = None,
        width: int = 800,
        height: int = 600,
        resizable: bool = True,
        frame: Optional[bool] = None,
        parent: Optional[int] = None,
        mode: Literal["auto", "owner", "child"] = "owner",
        bridge: Union["Bridge", bool, None] = None,  # type: ignore
        debug: bool = True,
        context_menu: bool = True,
        auto_timer: bool = True,
    ) -> "WebView":
        """Create and show an embedded WebView with auto timer (non-blocking).

        This is a convenience helper equivalent to:
            WebView.create(..., parent=..., mode=..., auto_timer=True, auto_show=True)

        Returns:
            WebView: The created instance (kept alive by your reference)
        """
        instance = cls.create(
            title=title,
            url=url,
            html=html,
            width=width,
            height=height,
            resizable=resizable,
            frame=frame,
            parent=parent,
            mode=mode,
            bridge=bridge,
            debug=debug,
            context_menu=context_menu,
            auto_show=True,
            auto_timer=auto_timer,
        )
        return instance

    @classmethod
    def create_embedded(
        cls,
        parent_hwnd: int,
        *,
        title: str = "Embedded WebView",
        width: int = 800,
        height: int = 600,
        url: Optional[str] = None,
        html: Optional[str] = None,
        asset_root: Optional[str] = None,
        debug: bool = True,
    ) -> "WebView":
        """Create a WebView directly embedded into a parent window's HWND.

        This is the fastest way to embed a WebView into a host application because:
        1. No Qt Widget intermediate layer
        2. WebView2 is created synchronously on the calling thread
        3. Uses host's native message loop directly

        This method is ideal when you have the host window's HWND and want
        maximum performance with minimal overhead. Works with DCC applications
        (Maya, 3ds Max, Houdini, etc.) or any Windows application with a HWND.

        Args:
            parent_hwnd: The HWND of the parent window (e.g., from Qt winId())
            title: Window title (for debugging/identification)
            width: Width in pixels
            height: Height in pixels
            url: URL to load (optional)
            html: HTML content to load (optional)
            asset_root: Root directory for auroraview:// protocol (optional)
            debug: Enable developer tools (default: True)

        Returns:
            WebView: A configured WebView instance ready to use

        Example (Houdini):
            >>> import hou
            >>> from auroraview import WebView
            >>> from PySide2.QtCore import QTimer
            >>>
            >>> # Get Houdini main window HWND
            >>> main_window = hou.qt.mainWindow()
            >>> hwnd = int(main_window.winId())
            >>>
            >>> # Create WebView directly embedded
            >>> webview = WebView.create_embedded(
            ...     parent_hwnd=hwnd,
            ...     title="My Tool",
            ...     width=650,
            ...     height=500,
            ...     url="https://example.com"
            ... )
            >>>
            >>> # Set up timer to process messages
            >>> timer = QTimer()
            >>> timer.timeout.connect(webview.process_events_ipc_only)
            >>> timer.start(16)  # 60 FPS
        """
        from auroraview._core import WebView as _CoreWebView

        logger.info(f"[create_embedded] Creating WebView for parent HWND: {parent_hwnd}")

        # Create core WebView using create_embedded static method
        core = _CoreWebView.create_embedded(
            parent_hwnd=parent_hwnd,
            title=title,
            width=width,
            height=height,
        )

        # Build the Python wrapper without re-running __init__'s
        # _CoreWebView(...) construction path (we already have a core
        # produced by the embedded static method above), then delegate
        # *all* attribute initialisation to ``_init_runtime_state`` so
        # any future field added to the standard ``__init__`` flow is
        # automatically picked up here too.
        #
        # ``from_init_kwargs`` is the same funnel ``__init__`` uses;
        # we forward only the fields ``create_embedded`` actually
        # accepts and rely on the dataclass defaults for the rest.
        # ``forward_asset_root_to_core=True`` because the embedded
        # core constructor (``_CoreWebView.create_embedded``) does
        # not accept ``asset_root`` — the only place it can be
        # applied is the ``core.set_asset_root`` call inside
        # ``_init_runtime_state``.
        instance = cls.__new__(cls)
        instance._core = core
        instance._init_runtime_state(
            _WebViewInitState.from_init_kwargs(
                asset_root=asset_root,
                forward_asset_root_to_core=True,
                title=title,
                width=width,
                height=height,
                url=url,
                html=html,
                debug=debug,
                resizable=True,
                frame=False,
                parent=parent_hwnd,
                mode="child",
                auto_show=False,
            )
        )

        # Load content
        if url:
            core.load_url(url)
        elif html:
            core.load_html(html)

        logger.info("[create_embedded] WebView created successfully")
        logger.info("[create_embedded] Remember to call process_events_ipc_only() periodically!")

        return instance

    def show(self, *, wait: Optional[bool] = None) -> None:
        """Show the WebView window (smart mode).

        Automatically detects standalone/embedded/packed mode and chooses the best behavior:
        - Packed mode: Runs as headless API server (no window, JSON-RPC via stdin/stdout)
        - Standalone window: Blocks until closed (unless wait=False)
        - Embedded window: Non-blocking, auto-starts timer if available

        Args:
            wait: Whether to wait for window to close
                - None: Auto-detect (standalone=True, embedded=False)
                - True: Block until window closes
                - False: Return immediately (background thread)

        Examples:
            >>> # Standalone window - auto-blocking
            >>> webview = WebView(title="My App")
            >>> webview.show()  # Blocks until closed

            >>> # Standalone window - force non-blocking
            >>> webview = WebView(title="My App")
            >>> webview.show(wait=False)  # Returns immediately
            >>> input("Press Enter to exit...")

            >>> # Embedded window - auto non-blocking
            >>> webview = WebView(title="Tool", parent=maya_hwnd)
            >>> webview.show()  # Returns immediately, timer auto-runs

            >>> # Packed mode - automatic API server (no code changes needed)
            >>> # When running in a packed .exe, show() automatically switches
            >>> # to API server mode. All bind_call() handlers work seamlessly.
        """
        # Check for packed mode first - transparent to developers
        from .packed import is_packed_mode, run_api_server

        if is_packed_mode():
            logger.info("Packed mode detected: running as API server")
            run_api_server(self)
            return

        # Detect mode
        is_embedded = self._parent is not None

        # Auto-detect wait behavior
        if wait is None:
            wait = not is_embedded  # Standalone defaults to blocking

        logger.info(f"Showing WebView: embedded={is_embedded}, wait={wait}")

        # Start Bridge if present
        if self._bridge and not self._bridge.is_running:
            logger.info("Starting Bridge in background...")
            self._bridge.start_background()

        if is_embedded:
            # Embedded mode: non-blocking + auto timer
            logger.info("Embedded mode: non-blocking with auto timer")
            self._show_non_blocking()
            # Mark as shown
            if hasattr(self, "_ready_events") and self._ready_events:
                self._ready_events.set_shown()
            # Start timer immediately - it will wait for WebView to be ready
            if self._auto_timer is not None:
                self._auto_timer.start()
                logger.info("Auto timer started (will wait for WebView initialization)")
        else:
            # Standalone mode
            if wait:
                # Blocking
                logger.info("Standalone mode: blocking until window closes")
                # Mark as shown before blocking
                if hasattr(self, "_ready_events") and self._ready_events:
                    self._ready_events.set_shown()
                self.show_blocking()
            else:
                # Non-blocking (background thread)
                logger.info("Standalone mode: non-blocking (background thread)")
                logger.warning("Window will close when script exits!")
                logger.warning("Use wait=True or keep script running with input()")
                self._show_non_blocking()
                # Mark as shown
                if hasattr(self, "_ready_events") and self._ready_events:
                    self._ready_events.set_shown()

    def show_async(self) -> None:
        """Show the WebView window in non-blocking mode (compatibility helper).

        Equivalent to calling show(wait=False). Safe to call multiple times; if the
        WebView is already running, the call is ignored.
        """
        self._show_non_blocking()

    def _show_non_blocking(self) -> None:
        """Internal method: non-blocking show (background thread)."""
        if self._is_running:
            logger.warning("WebView is already running")
            return

        logger.info(f"Showing WebView in background thread: {self._title}")
        self._is_running = True

        def _run_webview():
            """Run the WebView in a background thread.

            Note: We create a new WebView instance in the background thread
            because the Rust core requires the WebView to be created and shown
            in the same thread due to GUI event loop requirements.
            """
            try:
                logger.info("Background thread: Creating WebView instance")
                # Create a new WebView instance in this thread
                # This is necessary because the Rust core is not Send/Sync
                from auroraview._core import WebView as _CoreWebView

                core = _CoreWebView(
                    title=self._title,
                    width=self._width,
                    height=self._height,
                    dev_tools=self._debug,  # Use new parameter name
                    resizable=self._resizable,
                    decorations=self._frame,  # Use new parameter name
                    parent_hwnd=self._parent,  # Use new parameter name
                    parent_mode=self._mode,  # Use new parameter name
                    always_on_top=self._always_on_top,  # Keep window always on top
                    transparent=self._transparent,  # Enable transparent window
                    background_color=self._background_color,  # Window background color
                    tool_window=self._tool_window,  # Tool window style
                    undecorated_shadow=self._undecorated_shadow,  # Shadow for frameless
                    allow_new_window=self._allow_new_window,  # Allow window.open()
                    remote_debugging_port=self._remote_debugging_port,  # CDP port
                )

                # Set up HWND callback to cache HWND for cross-thread access
                def on_hwnd_created(hwnd: int) -> None:
                    with self._cached_hwnd_lock:
                        self._cached_hwnd = hwnd
                    logger.info(f"Background thread: Cached HWND 0x{hwnd:X}")

                if hasattr(core, "set_on_hwnd_created"):
                    core.set_on_hwnd_created(on_hwnd_created)

                # Store the core instance for use by emit() and other methods
                with self._async_core_lock:
                    self._async_core = core

                # If close was requested before the background core became ready,
                # exit early without entering the event loop.
                if getattr(self, "_close_requested", False):
                    logger.info(
                        "Background thread: close already requested; skipping show() and exiting"
                    )
                    return

                # Re-register all event handlers in the background thread
                # Snapshot the handlers under lock to avoid race conditions
                # with the main thread adding handlers concurrently.
                with self._event_handlers_lock:
                    handlers_snapshot = {k: list(v) for k, v in self._event_handlers.items()}

                logger.info(
                    f"Background thread: Re-registering {len(handlers_snapshot)} event handlers"
                )
                for event_name, handlers in handlers_snapshot.items():
                    for handler in handlers:
                        logger.debug(f"Background thread: Registering handler for '{event_name}'")
                        core.on(event_name, handler)

                # Load the same content that was loaded in the main thread
                if self._stored_html:
                    logger.info("Background thread: Loading stored HTML")
                    core.load_html(self._stored_html)
                elif self._stored_url:
                    logger.info("Background thread: Loading stored URL")
                    core.load_url(self._stored_url)
                else:
                    logger.warning("Background thread: No content loaded")

                logger.info("Background thread: Starting WebView event loop")
                core.show()
                # Note: show() is blocking - the HWND callback is invoked before
                # entering the event loop, so _cached_hwnd is already set
                logger.info("Background thread: WebView event loop exited")
            except Exception as e:
                logger.error(f"Error in background WebView: {e}", exc_info=True)
            finally:
                # Clear the async core reference
                with self._async_core_lock:
                    self._async_core = None
                self._is_running = False
                logger.info("Background thread: WebView thread finished")

        # Create and start the background thread as daemon
        # CRITICAL: daemon=True allows Maya to exit cleanly when user closes Maya
        # The event loop now uses run_return() instead of run(), which prevents
        # the WebView from calling std::process::exit() and terminating Maya
        self._show_thread = threading.Thread(target=_run_webview, daemon=True)
        self._show_thread.start()
        logger.info("WebView background thread started (daemon=True)")

    def show_blocking(self) -> None:
        """Show the WebView window (blocking - for standalone scripts).

        This method blocks until the window is closed. Use this in standalone scripts
        where you want the script to wait for the user to close the window.

        NOT recommended for DCC integration (Maya, Houdini, etc.) as it will freeze
        the main application.

        Example:
            >>> webview = WebView(title="My App", width=800, height=600)
            >>> webview.load_html("<h1>Hello</h1>")
            >>> webview.show_blocking()  # Blocks until window closes
            >>> print("Window was closed")
        """
        logger.info(f"Showing WebView (blocking): {self._title}")
        logger.info("Calling _core.show()...")

        # Check if we're in embedded mode
        is_embedded = self._parent is not None  # Use new parameter name

        # Mark that we're entering blocking event loop
        # This tells eval_js to skip _auto_process_events since the event loop
        # will handle message queue processing automatically
        self._in_blocking_event_loop = True

        try:
            self._core.show()
            logger.info("_core.show() returned successfully")
        except Exception as e:
            logger.error(f"Error in _core.show(): {e}", exc_info=True)
            raise
        finally:
            # Clear the flag when event loop exits
            self._in_blocking_event_loop = False

        # IMPORTANT: Only cleanup in standalone mode
        # In embedded mode, the window should stay open until explicitly closed
        if not is_embedded:
            logger.info("Standalone mode: WebView show_blocking() completed, cleaning up...")
            try:
                self.close()
            except Exception as cleanup_error:
                logger.warning(f"Error during cleanup: {cleanup_error}")
        else:
            logger.info("Embedded mode: WebView window is now open (non-blocking)")
            logger.info("IMPORTANT: Keep this Python object alive to prevent window from closing")
            logger.info("Example: __main__.webview = webview")

    # =========================================================================
    # Window Control Methods - provided by WebViewWindowMixin
    # JavaScript Methods - provided by WebViewJSMixin
    # DOM Methods - provided by WebViewDOMMixin
    # Event Methods - provided by WebViewEventMixin
    # API Methods - provided by WebViewApiMixin
    # =========================================================================

    def wait(self, timeout: Optional[float] = None) -> bool:
        """Wait for the WebView to close.

        This method blocks until the WebView window is closed or the timeout expires.
        Useful when using show_async() to wait for user interaction.

        Args:
            timeout: Maximum time to wait in seconds (None = wait indefinitely)

        Returns:
            True if the WebView closed, False if timeout expired

        Example:
            >>> webview.show_async()
            >>> if webview.wait(timeout=60):
            ...     print("WebView closed by user")
            ... else:
            ...     print("Timeout waiting for WebView")
        """
        if self._show_thread is None:
            logger.warning("WebView is not running")
            return True

        logger.info(f"Waiting for WebView to close (timeout={timeout})")
        self._show_thread.join(timeout=timeout)

        if self._show_thread.is_alive():
            logger.warning("Timeout waiting for WebView to close")
            return False

        logger.info("WebView closed")
        return True

    def set_event_processor(self, processor: Any) -> None:
        """Set event processor (strategy pattern for UI framework integration).

        Args:
            processor: Event processor object with a `process()` method.
                      This allows UI frameworks (Qt, Tk, etc.) to inject their
                      event processing logic.

        Example:
            >>> class QtEventProcessor:
            ...     def process(self):
            ...         QCoreApplication.processEvents()
            ...         webview._core.process_events()
            >>>
            >>> processor = QtEventProcessor()
            >>> webview.set_event_processor(processor)
        """
        self._event_processor = processor
        logger.debug(f"Event processor set: {type(processor).__name__}")

    def set_plugin_manager(self, plugin_manager: Any) -> None:
        """Set plugin manager for handling plugin:* invoke commands.

        This enables JavaScript to call Rust plugins via `window.auroraview.invoke()`.

        Args:
            plugin_manager: PluginManager instance from auroraview.PluginManager

        Example:
            >>> from auroraview import WebView, PluginManager
            >>>
            >>> view = WebView(title="Plugin Demo")
            >>> plugins = PluginManager.permissive()
            >>> view.set_plugin_manager(plugins)
            >>>
            >>> # Now JavaScript can call plugins:
            >>> # await auroraview.invoke("plugin:fs|read_file", {path: "/tmp/test.txt"})
        """
        self._plugin_manager = plugin_manager
        # Register handler for __plugin_invoke__ events from Rust IPC
        self.register_callback("__plugin_invoke__", self._handle_plugin_invoke)
        logger.info("Plugin manager set and __plugin_invoke__ handler registered")

    def _handle_plugin_invoke(self, data: dict) -> None:
        """Handle plugin invoke commands from JavaScript.

        This is called when Rust IPC receives a `type: 'invoke'` message
        and forwards it as `__plugin_invoke__` event.

        Args:
            data: Dict with 'cmd', 'args', and optional 'id' fields
        """
        if self._plugin_manager is None:
            logger.warning("Plugin invoke received but no plugin manager set")
            return

        cmd = data.get("cmd", "")
        args = data.get("args", {})
        call_id = data.get("id")

        logger.info(f"[Plugin] Invoke: {cmd}, args: {args}, id: {call_id}")

        try:
            # Convert args to JSON string for PluginManager.handle_command()
            args_json = json.dumps(args) if isinstance(args, dict) else "{}"
            result_json = self._plugin_manager.handle_command(cmd, args_json)
            result = json.loads(result_json)

            # Send result back to JavaScript
            payload = {
                "id": call_id,
                "ok": result.get("success", False),
            }
            if result.get("success"):
                payload["result"] = result.get("data")
            else:
                payload["error"] = {
                    "message": result.get("error", "Unknown error"),
                    "code": result.get("code", "PLUGIN_ERROR"),
                }

            # Emit result via __invoke_result__ event
            self.emit("__invoke_result__", payload)
            logger.info(f"[Plugin] Result sent: {payload}")

        except Exception as e:
            logger.error(f"[Plugin] Error handling invoke: {e}")
            error_payload = {
                "id": call_id,
                "ok": False,
                "error": {
                    "message": str(e),
                    "code": "INTERNAL_ERROR",
                },
            }
            self.emit("__invoke_result__", error_payload)

    def _auto_process_events(self) -> None:
        """Automatically process events after emit() or eval_js().

        This method uses the strategy pattern:
        1. If in blocking event loop (HWND mode), skip - event loop handles it
        2. If an event processor is set, use it (UI framework integration)
        3. Otherwise, use default implementation (direct Rust call)

        Subclasses can still override this method for custom behavior.
        """
        # Skip if we're in a blocking event loop (e.g., HWND mode background thread)
        # The event loop automatically processes the message queue
        if self._in_blocking_event_loop:
            logger.debug("Skipping _auto_process_events - in blocking event loop")
            return

        try:
            if self._event_processor is not None:
                # Use strategy pattern: delegate to event processor
                self._event_processor.process()
            else:
                # Default implementation: direct Rust call
                self._core.process_events()
        except Exception as e:
            logger.debug(f"Auto process events failed (non-critical): {e}")

    def process_events(self) -> bool:
        """Process pending window events.

        This method should be called periodically in embedded mode to handle
        window messages and user interactions. Returns True if the window
        should be closed.

        Returns:
            True if the window should close, False otherwise. Also returns
            False when no backing core is currently available (packed mode,
            pre-init, post-dispose, or a transient ``_async_core_lock``
            contention) — in those cases there is nothing to drain and
            synthesizing a close request would be wrong.

        Example:
            >>> # In Maya, use a scriptJob to process events
            >>> def process_webview_events():
            ...     if webview.process_events():
            ...         # Window should close
            ...         cmds.scriptJob(kill=job_id)
            ...
            >>> job_id = cmds.scriptJob(event=["idle", process_webview_events])
        """
        # Route through ``_peek_active_core`` so this stays in lock-step
        # with :attr:`is_ready` / :meth:`is_window_valid`. Without this,
        # packed mode (``_core is None`` but ``_async_core`` wired up by
        # the show-thread) would have ``is_ready`` report True while
        # ``self._core.process_events()`` raises ``AttributeError`` on
        # every 16 ms timer tick.
        core = self._peek_active_core()
        if core is None or core is WebView._CORE_LOCK_CONTENDED:
            return False
        return core.process_events()

    def process_events_ipc_only(self) -> bool:
        """Process only internal AuroraView IPC without touching host event loop.

        This variant is intended for host-driven embedding scenarios (Qt/DCC)
        where the native window message pump is owned by the host application.
        It only drains the internal WebView message queue and respects
        lifecycle close requests.

        Returns:
            True if the window should close, False otherwise. Also returns
            False when no backing core is currently available — see
            :meth:`process_events` for the full enumeration.
        """
        # Same active-core probe as :meth:`process_events`; see that method
        # for the rationale on aligning with :attr:`is_ready` semantics.
        core = self._peek_active_core()
        if core is None or core is WebView._CORE_LOCK_CONTENDED:
            return False
        return core.process_ipc_only()

    def is_alive(self) -> bool:
        """Check if WebView is still running.

        Returns:
            True if WebView is running, False otherwise

        Example:
            >>> webview.show(wait=False)
            >>> while webview.is_alive():
            ...     time.sleep(0.1)
        """
        if self._show_thread is None:
            return False
        return self._show_thread.is_alive()

    def get_hwnd(self) -> Optional[int]:
        """Get the native window handle (HWND on Windows).

        This is useful for integrating with external applications that need
        the native window handle, such as:
        - Unreal Engine: `unreal.parent_external_window_to_slate(hwnd)`
        - Windows API: Direct window manipulation
        - Other DCC tools with HWND-based integration

        Returns:
            int: The native window handle (HWND), or None if not available.

        Raises:
            RuntimeError: If WebView is not initialized (call show() first).

        Example:
            >>> webview = WebView.create(...)
            >>> webview.show()
            >>> hwnd = webview.get_hwnd()
            >>> if hwnd:
            ...     print(f"Window HWND: 0x{hwnd:x}")
            ...     # Use with Unreal Engine
            ...     # unreal.parent_external_window_to_slate(hwnd)
        """
        # First check Python-side cached HWND (thread-safe, works in non-blocking mode)
        with self._cached_hwnd_lock:
            if self._cached_hwnd is not None:
                return self._cached_hwnd

        # In non-blocking mode (show_thread exists), avoid calling Rust core
        # as it may cause GIL contention and blocking
        if self._show_thread is not None:
            # Background thread mode - only use Python-side cache
            # The cache is set by the on_hwnd_created callback in the background thread
            return None

        # Fall back to Rust core (only works if called from the same thread)
        # This is for blocking mode or when called from the background thread
        try:
            return self._core.get_hwnd()
        except Exception:
            # In non-blocking mode, _core.get_hwnd() may fail due to thread safety
            return None

    def get_proxy(self) -> Any:
        """Get a thread-safe proxy for cross-thread operations.

        Returns a WebViewProxy that can be safely shared across threads.
        Use this when you need to call `eval_js`, `emit`, etc. from a different
        thread than the one that created the WebView.

        This is essential for HWND mode where the WebView runs in a background
        thread but you need to call methods from the DCC main thread.

        Returns:
            WebViewProxy: A thread-safe proxy for WebView operations.
                The proxy supports:
                - eval_js(script): Execute JavaScript
                - eval_js_async(script, callback, timeout_ms): Async JavaScript
                - emit(event_name, data): Emit events to JavaScript
                - load_url(url): Load a URL
                - load_html(html): Load HTML content
                - reload(): Reload the current page

        Example:
            >>> # In HWND mode - WebView runs in background thread
            >>> def create_webview_thread():
            ...     webview = WebView(...)
            ...     proxy = webview.get_proxy()  # Get thread-safe proxy
            ...     self._proxy = proxy          # Store for cross-thread access
            ...     webview.show_blocking()
            ...
            >>> # From DCC main thread - safe!
            >>> self._proxy.eval_js("console.log('Hello from DCC!')")
            >>> self._proxy.emit("update", {"status": "ready"})

        Note:
            The proxy uses a message queue internally. Operations are queued
            and processed by the WebView's event loop on the correct thread.
        """
        # Use async core if available (when running in background thread)
        with self._async_core_lock:
            core = self._async_core if self._async_core is not None else self._core
        return core.get_proxy()

    def create_emitter(self) -> Any:
        """Create a thread-safe event emitter.

        Returns an EventEmitter that can be safely used from any thread to emit
        events to the JavaScript frontend. This is useful for plugin callbacks
        that run on background threads (e.g., ProcessPlugin).

        Unlike `emit()` which must be called from the main thread, the emitter
        returned by this method can be called from any thread safely.

        Returns:
            EventEmitter: A thread-safe emitter with an `emit(event_name, data)` method.

        Example:
            >>> # Create emitter for cross-thread event emission
            >>> emitter = webview.create_emitter()
            >>>
            >>> # Use with PluginManager (ProcessPlugin runs on background threads)
            >>> plugins = PluginManager.permissive()
            >>> plugins.set_emit_callback(emitter.emit)
            >>>
            >>> # ProcessPlugin will emit events like:
            >>> # - process:stdout - { pid, data }
            >>> # - process:stderr - { pid, data }
            >>> # - process:exit - { pid, code }
        """
        return self._core.create_emitter()

    def thread_safe(self) -> Any:
        """Get a thread-safe wrapper for cross-thread operations.

        Returns a DCCThreadSafeWrapper that provides thread-safe methods
        for common WebView operations. Use this when you need to call
        WebView methods from a different thread than the one that created it.

        This is particularly useful in DCC environments where:
        - WebView runs in a background thread (HWND mode)
        - You need to call methods from the DCC main thread
        - You want simpler API than using get_proxy() directly

        Returns:
            DCCThreadSafeWrapper: A wrapper with thread-safe methods including:
                - eval_js(script): Execute JavaScript (fire-and-forget)
                - eval_js_sync(script, timeout): Execute JavaScript (blocking)
                - emit(event_name, data): Emit events to JavaScript
                - load_url(url): Load a URL
                - load_html(html): Load HTML content
                - reload(): Reload the current page
                - close(): Close the WebView

        Example:
            >>> webview = WebView(parent=dcc_hwnd)
            >>> webview.show()  # Runs in background thread
            >>>
            >>> # From any thread:
            >>> safe = webview.thread_safe()
            >>> safe.eval_js("updateStatus('ready')")
            >>> safe.emit("data_loaded", {"count": 100})
            >>>
            >>> # Synchronous JavaScript execution
            >>> title = safe.eval_js_sync("document.title")

        Note:
            The wrapper uses the internal message queue to deliver operations
            to the WebView thread. Most operations are non-blocking.
        """
        from auroraview.utils.thread_dispatcher import DCCThreadSafeWrapper

        return DCCThreadSafeWrapper(self)

    @property
    def dcc_mode(self) -> bool:
        """Check if DCC thread safety mode is enabled.

        Returns:
            bool: True if dcc_mode is enabled, False otherwise.
        """
        return self._dcc_mode

    def close(self) -> None:
        """Close the WebView window and remove from registries."""
        logger.info("Closing WebView")

        # Teardown telemetry before closing
        self._teardown_telemetry()

        # Mark close intent early so background thread can bail out if it hasn't
        # entered the event loop yet.
        self._close_requested = True

        # Prefer closing the background-thread core (if present). Fall back to
        # the main-thread core as a best-effort.
        cores = []
        try:
            with self._async_core_lock:
                if self._async_core is not None:
                    cores.append(self._async_core)
        except Exception:
            # Lock acquisition should never fail, but keep close best-effort.
            pass

        cores.append(getattr(self, "_core", None))

        seen = set()
        for core in cores:
            if core is None:
                continue
            core_id = id(core)
            if core_id in seen:
                continue
            seen.add(core_id)

            try:
                core.close()
                logger.info("Core WebView close requested")
            except Exception as e:
                logger.warning(f"Error requesting core close: {e}")

        # Wait for background thread if running
        if self._show_thread is not None and self._show_thread.is_alive():
            logger.info("Waiting for background thread to finish...")
            self._show_thread.join(timeout=5.0)
            if self._show_thread.is_alive():
                logger.warning("Background thread did not finish within timeout")
            else:
                logger.info("Background thread finished successfully")

        # Remove from singleton registry
        for key, instance in list(self._singleton_registry.items()):
            if instance is self:
                del self._singleton_registry[key]
                logger.info(f"Removed from singleton registry: '{key}'")
                break

        # Remove from WindowManager
        if self._window_id:
            from .window_manager import get_window_manager

            wm = get_window_manager()
            wm.unregister(self._window_id)
            logger.debug(f"WebView unregistered from WindowManager: {self._window_id}")

        logger.info("WebView closed successfully")

    def _setup_lifecycle_events(self) -> None:
        """Setup internal event handlers for lifecycle tracking."""

        # Track page load completion
        @self.on("page:load_finish")
        def _on_load_finish(data: Any) -> None:
            if hasattr(self, "_ready_events") and self._ready_events:
                self._ready_events.set_loaded()
            # Auto-telemetry: record page load time
            self._telemetry_on_page_loaded()

        # Track bridge ready
        @self.on("auroraviewready")
        def _on_bridge_ready(data: Any) -> None:
            if hasattr(self, "_ready_events") and self._ready_events:
                self._ready_events.set_bridge_ready()

    @property
    def window_id(self) -> Optional[str]:
        """Get the unique window ID in WindowManager.

        Returns:
            The window's unique ID, or None if not registered
        """
        return self._window_id

    @property
    def ready_events(self) -> "ReadyEvents":
        """Get the ReadyEvents container for lifecycle waiting.

        Returns:
            ReadyEvents instance for this WebView

        Example:
            >>> webview.ready_events.wait_loaded(timeout=10)
            >>> webview.ready_events.wait_bridge_ready()
        """
        return self._ready_events

    # ------------------------------------------------------------------
    # Active-core probe
    # ------------------------------------------------------------------
    # Sentinel returned by :meth:`_peek_active_core` when the
    # ``_async_core_lock`` is contended. It is intentionally a private
    # singleton (not ``None``) so the caller can distinguish three states:
    #
    #   * ``_CORE_LOCK_CONTENDED`` — another thread is mid-transition on
    #     ``_async_core``; treat as "ready / validity unknown right now"
    #     and let the next 60 Hz tick retry.
    #   * ``None``                 — neither async nor sync core is wired
    #     up; the WebView is not yet (or no longer) initialized.
    #   * concrete core object     — the active backing core.
    _CORE_LOCK_CONTENDED: ClassVar[Any] = object()

    def _peek_active_core(self) -> Any:
        """Return the currently-active core, or a sentinel on contention.

        This is the single place that knows how to pick between
        ``_async_core`` and ``_core`` under the non-blocking lock
        discipline used by 60 Hz timer callers. ``is_ready`` and
        ``is_window_valid`` both build on this helper so their
        contention semantics never drift.

        Returns:
            * :attr:`_CORE_LOCK_CONTENDED` when ``_async_core_lock`` is
              held by another thread (caller should treat as transient
              and retry on the next tick),
            * the active core object (preferring ``_async_core`` over
              the sync ``_core``), or
            * ``None`` when no core is wired up.
        """
        lock = getattr(self, "_async_core_lock", None)
        if lock is not None:
            if not lock.acquire(blocking=False):
                # Contended → caller decides how to interpret the gap.
                return WebView._CORE_LOCK_CONTENDED
            try:
                async_core = self._async_core
            finally:
                lock.release()
            if async_core is not None:
                return async_core
        # Stubs that bypass ``__init__`` may not have ``_async_core_lock``;
        # fall straight through to the sync core in that case.
        return getattr(self, "_core", None)

    @property
    def is_ready(self) -> bool:
        """Whether the underlying WebView core is initialized.

        Returns True when at least one of the WebView cores is available:

        * The synchronous ``_core`` (created at construction time, used for
          standalone / in-process modes), or
        * The background-thread ``_async_core`` (assigned later in non-blocking
          mode after the host thread has finished spinning up the controller).

        Threading contract:
            ``is_ready`` is called from the event timer tick at ~60 Hz, so
            it must never stall on a contended ``_async_core_lock``. The
            probe is delegated to :meth:`_peek_active_core` which acquires
            the lock with ``blocking=False`` and returns
            :attr:`_CORE_LOCK_CONTENDED` when another thread is
            mid-transition on ``_async_core``. In that case we report
            **not-ready** for this tick; the next tick (~16 ms later)
            will retry.
        """
        core = self._peek_active_core()
        if core is WebView._CORE_LOCK_CONTENDED:
            return False
        return core is not None

    def is_window_valid(self) -> bool:
        """Whether the underlying native window is still alive.

        On Windows this delegates to ``core.is_window_valid()`` which calls
        ``IsWindow()`` on the embedded HWND. On other platforms (or when the
        Rust core does not yet expose the probe) we conservatively return
        ``True`` so the caller treats the window as live.

        This method is the **single public entry point** for window-validity
        checks. The event timer (and any other liveness-driven consumer)
        should call ``is_window_valid()`` instead of poking ``_core``
        directly so that:

        * the choice between ``_core`` and ``_async_core`` stays internal to
          ``WebView``, and
        * stubs/mocks can override the validity result without having to
          fake a full ``_core`` object.

        Lock-contention semantics match :attr:`is_ready` (see
        :meth:`_peek_active_core`): we never block the timer on a
        transient transition and instead report "valid for now".

        Startup short-circuit:
            When the active-core probe reports either ``None`` (no
            core wired up yet, or already disposed) or
            :attr:`_CORE_LOCK_CONTENDED` (another thread is
            mid-transition on ``_async_core``), we report **valid**.
            There is no window to be valid or invalid in those
            states, and the sync ``_core`` may also be in the
            "object built but HWND not attached" sub-state where
            ``IsWindow(0)`` would otherwise return False and
            trigger a spurious close. The timer interprets the gap
            together with :attr:`is_ready` and skips its tick body
            in that case anyway.

        Returns:
            ``True`` when the window is still valid (or when validity cannot
            be determined). ``False`` only when the native probe explicitly
            reports the window has been destroyed.
        """
        # One probe call covers both the "is_ready" gate and the
        # "give me the active core" lookup — calling
        # ``_peek_active_core`` twice (once via ``is_ready``, once
        # directly) would acquire the non-blocking lock twice and
        # widen the race window between the two reads for no benefit.
        core = self._peek_active_core()
        if core is None or core is WebView._CORE_LOCK_CONTENDED:
            # Either not yet wired up / already disposed, or another
            # thread is mid-transition. Both cases collapse to "valid
            # for now": the next tick will pick up the real state.
            return True

        probe = getattr(core, "is_window_valid", None)
        if probe is None:
            # Older core builds without the probe — treat as valid.
            return True
        try:
            return bool(probe())
        except Exception as e:
            logger.error("is_window_valid: native probe failed: %s", e)
            return False

    @property
    def title(self) -> str:
        """Get the window title."""
        return self._core.title

    @title.setter
    def title(self, value: str) -> None:
        """Set the window title."""
        self._core.set_title(value)
        self._title = value

    @property
    def width(self) -> int:
        """Get the window width."""
        return self._width

    @property
    def height(self) -> int:
        """Get the window height."""
        return self._height

    @property
    def x(self) -> Optional[int]:
        """Get the window x position."""
        return self._x

    @property
    def y(self) -> Optional[int]:
        """Get the window y position."""
        return self._y

    def __repr__(self) -> str:
        """String representation of the WebView."""
        return f"WebView(title='{self._title}', width={self._width}, height={self._height})"

    def __enter__(self) -> "WebView":
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:  # noqa: ARG002
        """Context manager exit."""
        self.close()

    # Bridge integration methods

    def _setup_bridge_integration(self):
        """Setup bidirectional communication between Bridge and WebView.

        This method is called automatically when a Bridge is associated with the WebView.
        It sets up:
        1. Bridge → WebView: Forward bridge events to WebView UI
        2. WebView → Bridge: Register handler to send commands to bridge clients
        """
        if not self._bridge:
            return

        logger.info("Setting up Bridge ↔ WebView integration")

        # Bridge → WebView: Forward events
        def bridge_callback(action: str, data: Dict, result: Any):
            """Forward bridge events to WebView UI."""
            logger.debug(f"Bridge event: {action}")
            # Emit event to JavaScript with 'bridge:' prefix
            self.emit(f"bridge:{action}", {"action": action, "data": data, "result": result})

        self._bridge.set_webview_callback(bridge_callback)

        # WebView → Bridge: Register command sender
        @self.on("send_to_bridge")
        def handle_send_to_bridge(data):
            """Send command from WebView to Bridge clients."""
            command = data.get("command")
            params = data.get("params", {})
            logger.debug(f"WebView → Bridge: {command}")
            if self._bridge:
                self._bridge.execute_command(command, params)
            return {"status": "sent"}

        logger.info("Bridge <-> WebView integration complete")

    @property
    def bridge(self) -> Optional["Bridge"]:  # type: ignore
        """Get the associated Bridge instance.

        Returns:
            Bridge instance or None if no bridge is associated

        Example:
            >>> webview = WebView.create("Tool", bridge=True)
            >>> print(webview.bridge)  # Bridge(ws://localhost:9001, ...)
            >>>
            >>> # Register handlers on the bridge
            >>> @webview.bridge.on('custom_event')
            >>> async def handle_custom(data, client):
            ...     return {"status": "ok"}
        """
        return self._bridge

    def send_to_bridge(self, command: str, params: Optional[Dict[str, Any]] = None) -> None:
        """Send command to Bridge clients (convenience method).

        Args:
            command: Command name
            params: Command parameters (defaults to an empty dict when omitted)

        Example:
            >>> webview.send_to_bridge('create_layer', {'name': 'New Layer'})
        """
        if not self._bridge:
            logger.warning("No bridge associated with this WebView")
            return

        # Avoid mutable default args: build a fresh dict each call when None.
        self._bridge.execute_command(command, params if params is not None else {})
