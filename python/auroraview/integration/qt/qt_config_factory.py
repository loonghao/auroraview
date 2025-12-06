"""Qt configuration factory for optimal DCC integration.

This module provides factory functions to create optimal Qt configurations
based on the detected Qt version (Qt5/Qt6) and DCC application.

The goal is to unify Qt5 and Qt6 behavior across different DCCs while
respecting their specific requirements.
"""

import logging
from typing import Optional

from auroraview.integration.qt._compat import get_qt_info, is_qt6

logger = logging.getLogger(__name__)


def create_optimal_qt_config(
    dcc_name: Optional[str] = None,
    *,
    init_delay_ms: Optional[int] = None,
    timer_interval_ms: Optional[int] = None,
    force_opaque: Optional[bool] = None,
) -> dict:
    """Create optimal Qt configuration based on Qt version and DCC.

    This factory function provides sensible defaults that work well across
    Qt5 and Qt6, with DCC-specific overrides when needed.

    Args:
        dcc_name: Name of the DCC application (e.g., "maya", "houdini").
        init_delay_ms: Override for initialization delay.
        timer_interval_ms: Override for timer interval.
        force_opaque: Override for opaque window setting.

    Returns:
        Dict with Qt configuration parameters.

    Example:
        >>> config = create_optimal_qt_config("houdini")
        >>> print(config["init_delay_ms"])
        100  # Qt6 needs longer delay
    """
    binding, version = get_qt_info()
    is_qt6_env = is_qt6()

    logger.info(f"Creating Qt config for {dcc_name or 'generic'} ({binding} Qt{version})")

    # Base configuration (Qt version-specific)
    if is_qt6_env:
        # Qt6 defaults: longer delays, opaque windows
        config = {
            "init_delay_ms": 100,
            "timer_interval_ms": 32,  # 30 FPS
            "geometry_fix_delays": [50, 150, 300, 600, 1000],
            "force_opaque_window": True,
            "disable_translucent": True,
            "is_qt6": True,
        }
    else:
        # Qt5 defaults: faster initialization
        config = {
            "init_delay_ms": 10,
            "timer_interval_ms": 16,  # 60 FPS
            "geometry_fix_delays": [100, 500, 1000],
            "force_opaque_window": False,
            "disable_translucent": False,
            "is_qt6": False,
        }

    # DCC-specific overrides
    if dcc_name:
        dcc_lower = dcc_name.lower()

        if dcc_lower in ("houdini", "hip"):
            # Houdini: Heavy main thread, needs slower updates
            config["timer_interval_ms"] = 50  # 20 FPS
            if is_qt6_env:
                config["geometry_fix_delays"] = [100, 300, 600, 1000, 2000]

        elif dcc_lower in ("maya", "mayapy"):
            # Maya: Standard settings work well
            pass

        elif dcc_lower in ("nuke", "nukex"):
            # Nuke: Needs extended geometry fixes
            config["geometry_fix_delays"] = [100, 500, 1000, 2000, 3000]

        elif dcc_lower in ("substancepainter", "painter", "sp"):
            # Substance Painter: Balanced for texture painting
            config["timer_interval_ms"] = 32  # 30 FPS
            if is_qt6_env:
                config["geometry_fix_delays"] = [50, 150, 300, 500, 1000]

    # Apply overrides
    if init_delay_ms is not None:
        config["init_delay_ms"] = init_delay_ms
    if timer_interval_ms is not None:
        config["timer_interval_ms"] = timer_interval_ms
    if force_opaque is not None:
        config["force_opaque_window"] = force_opaque
        config["disable_translucent"] = force_opaque

    logger.debug(f"Qt config: {config}")
    return config


def get_optimal_init_delay(dcc_name: Optional[str] = None) -> int:
    """Get optimal initialization delay based on Qt version and DCC.

    Args:
        dcc_name: Name of the DCC application.

    Returns:
        Initialization delay in milliseconds.
    """
    config = create_optimal_qt_config(dcc_name)
    return config["init_delay_ms"]


def get_optimal_timer_interval(dcc_name: Optional[str] = None) -> int:
    """Get optimal timer interval based on Qt version and DCC.

    Args:
        dcc_name: Name of the DCC application.

    Returns:
        Timer interval in milliseconds.
    """
    config = create_optimal_qt_config(dcc_name)
    return config["timer_interval_ms"]


def get_optimal_geometry_delays(dcc_name: Optional[str] = None) -> list[int]:
    """Get optimal geometry fix delays based on Qt version and DCC.

    Args:
        dcc_name: Name of the DCC application.

    Returns:
        List of delays in milliseconds.
    """
    config = create_optimal_qt_config(dcc_name)
    return config["geometry_fix_delays"]


def should_force_opaque(dcc_name: Optional[str] = None) -> bool:
    """Check if opaque window should be forced based on Qt version.

    Args:
        dcc_name: Name of the DCC application.

    Returns:
        True if opaque window should be forced.
    """
    config = create_optimal_qt_config(dcc_name)
    return config["force_opaque_window"]

