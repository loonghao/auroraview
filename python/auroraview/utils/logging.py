"""Logging utilities for AuroraView.

This module provides centralized logging configuration for AuroraView,
with support for environment variable controls and DCC-specific optimizations.

Environment Variables:
    AURORAVIEW_LOG_LEVEL: Set log level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
    AURORAVIEW_LOG_ENABLED: Enable/disable logging (1/0, true/false)
    AURORAVIEW_LOG_VERBOSE: Enable verbose debug logging (1/0, true/false)

In DCC applications (Maya, Houdini, Nuke, etc.), excessive logging can cause
significant UI performance issues. By default, AuroraView uses WARNING level
in production to minimize console output.

Example:
    # Enable debug logging for troubleshooting
    import os
    os.environ["AURORAVIEW_LOG_LEVEL"] = "DEBUG"
    os.environ["AURORAVIEW_LOG_VERBOSE"] = "1"

    # Or programmatically
    from auroraview.utils.logging import configure_logging
    configure_logging(level="DEBUG", verbose=True)
"""

import logging
import os
import sys
from typing import Optional

# Default log level (WARNING for DCC environments to minimize console output)
_DEFAULT_LOG_LEVEL = logging.WARNING

# Environment variable names
ENV_LOG_LEVEL = "AURORAVIEW_LOG_LEVEL"
ENV_LOG_ENABLED = "AURORAVIEW_LOG_ENABLED"
ENV_LOG_VERBOSE = "AURORAVIEW_LOG_VERBOSE"

# Global logging state
_logging_configured = False
_verbose_enabled = False


def _parse_bool_env(name: str, default: bool = False) -> bool:
    """Parse boolean environment variable."""
    value = os.environ.get(name, "").lower()
    if value in ("1", "true", "yes", "on"):
        return True
    if value in ("0", "false", "no", "off"):
        return False
    return default


def _parse_log_level(level_str: str) -> int:
    """Parse log level from string."""
    level_map = {
        "DEBUG": logging.DEBUG,
        "INFO": logging.INFO,
        "WARNING": logging.WARNING,
        "WARN": logging.WARNING,
        "ERROR": logging.ERROR,
        "CRITICAL": logging.CRITICAL,
    }
    return level_map.get(level_str.upper(), _DEFAULT_LOG_LEVEL)


def is_verbose_enabled() -> bool:
    """Check if verbose logging is enabled.

    Returns:
        True if verbose mode is enabled via environment or configure_logging().
    """
    global _verbose_enabled
    return _verbose_enabled or _parse_bool_env(ENV_LOG_VERBOSE, False)


def configure_logging(
    level: Optional[str] = None,
    enabled: Optional[bool] = None,
    verbose: Optional[bool] = None,
    format_string: Optional[str] = None,
) -> None:
    """Configure AuroraView logging.

    This function should be called early in your application startup
    if you need custom logging configuration.

    Args:
        level: Log level (DEBUG, INFO, WARNING, ERROR, CRITICAL).
               If None, uses AURORAVIEW_LOG_LEVEL env var or WARNING.
        enabled: Enable/disable logging. If None, uses AURORAVIEW_LOG_ENABLED.
        verbose: Enable verbose debug output. If None, uses AURORAVIEW_LOG_VERBOSE.
        format_string: Custom log format. If None, uses default format.

    Example:
        >>> from auroraview.utils.logging import configure_logging
        >>> configure_logging(level="DEBUG", verbose=True)
    """
    global _logging_configured, _verbose_enabled

    # Parse enabled flag
    if enabled is None:
        enabled = _parse_bool_env(ENV_LOG_ENABLED, True)

    if not enabled:
        # Disable all logging by setting level to CRITICAL+1
        logging.getLogger("auroraview").setLevel(logging.CRITICAL + 1)
        _logging_configured = True
        return

    # Parse verbose flag
    if verbose is None:
        verbose = _parse_bool_env(ENV_LOG_VERBOSE, False)
    _verbose_enabled = verbose

    # Parse log level
    if level is None:
        level = os.environ.get(ENV_LOG_LEVEL, "WARNING")
    log_level = _parse_log_level(level)

    # If verbose mode, force DEBUG level
    if verbose:
        log_level = logging.DEBUG

    # Configure root auroraview logger
    root_logger = logging.getLogger("auroraview")
    root_logger.setLevel(log_level)

    # Only add handler if none exist
    if not root_logger.handlers:
        handler = logging.StreamHandler(sys.stderr)
        handler.setLevel(log_level)

        if format_string is None:
            format_string = "[%(name)s] %(levelname)s: %(message)s"

        formatter = logging.Formatter(format_string)
        handler.setFormatter(formatter)
        root_logger.addHandler(handler)

    _logging_configured = True


def get_logger(name: str) -> logging.Logger:
    """Get a logger for AuroraView components.

    Args:
        name: Logger name (usually __name__).

    Returns:
        Configured logger instance.

    Example:
        >>> from auroraview.utils.logging import get_logger
        >>> logger = get_logger(__name__)
        >>> logger.debug("This is a debug message")
    """
    global _logging_configured

    # Configure logging on first use if not already done
    if not _logging_configured:
        configure_logging()

    return logging.getLogger(name)


# NullHandler for library logging (per Python logging best practices)
class NullHandler(logging.Handler):
    """Null handler that discards all log records."""

    def emit(self, record):
        pass


# Add NullHandler to root auroraview logger to prevent "No handler found" warnings
logging.getLogger("auroraview").addHandler(NullHandler())
