"""Logging utilities for AuroraView.

This module provides centralized logging configuration for AuroraView,
with support for environment variable controls, DCC-specific optimizations,
and Packed Mode awareness.

Environment Variables:
    AURORAVIEW_LOG_LEVEL: Set log level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
    AURORAVIEW_LOG_ENABLED: Enable/disable logging (1/0, true/false)
    AURORAVIEW_LOG_VERBOSE: Enable verbose debug logging (1/0, true/false)
    AURORAVIEW_PACKED: Set by Rust runtime when running in packed mode

Packed Mode Behavior:
    In packed mode, Python's stdout is used for JSON-RPC communication with
    the Rust host. All log output is redirected to stderr with structured
    prefixes that allow the Rust host to distinguish between:
    - DEBUG/INFO logs (informational, should not trigger backend_error)
    - WARNING logs (potential issues, logged but not fatal)
    - ERROR/CRITICAL logs (real errors that may need user attention)

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

    # Get a logger (works correctly in both normal and packed modes)
    from auroraview.utils.logging import get_logger
    logger = get_logger(__name__)
    logger.info("This will go to stderr with proper formatting")
"""

import json
import logging
import os
import sys
from datetime import datetime
from typing import Optional

# Default log level (WARNING for DCC environments to minimize console output)
_DEFAULT_LOG_LEVEL = logging.WARNING

# Environment variable names
ENV_LOG_LEVEL = "AURORAVIEW_LOG_LEVEL"
ENV_LOG_ENABLED = "AURORAVIEW_LOG_ENABLED"
ENV_LOG_VERBOSE = "AURORAVIEW_LOG_VERBOSE"
ENV_PACKED = "AURORAVIEW_PACKED"

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


def is_packed_mode() -> bool:
    """Check if running in packed mode.

    In packed mode, stdout is used for JSON-RPC communication,
    so all logging must go to stderr with special formatting.

    Returns:
        True if AURORAVIEW_PACKED environment variable is set.
    """
    return _parse_bool_env(ENV_PACKED, False)


def is_verbose_enabled() -> bool:
    """Check if verbose logging is enabled.

    Returns:
        True if verbose mode is enabled via environment or configure_logging().
    """
    global _verbose_enabled
    return _verbose_enabled or _parse_bool_env(ENV_LOG_VERBOSE, False)


class PackedModeFormatter(logging.Formatter):
    """Log formatter for packed mode.

    Formats log messages with structured prefixes that the Rust host can parse
    to distinguish between informational logs and real errors.

    Format: [LEVEL:module] message

    The Rust host can filter based on level prefix:
    - [DEBUG:...] and [INFO:...] are informational
    - [WARNING:...] should be logged but not trigger backend_error
    - [ERROR:...] and [CRITICAL:...] are real errors
    """

    def format(self, record: logging.LogRecord) -> str:
        # Get the module name (last part of the logger name)
        module = record.name.split(".")[-1] if record.name else "root"

        # Format: [LEVEL:module] message
        return f"[{record.levelname}:{module}] {record.getMessage()}"


class PackedModeJSONFormatter(logging.Formatter):
    """JSON log formatter for packed mode.

    Produces structured JSON logs that can be easily parsed by the Rust host
    and forwarded to the frontend for display in developer tools.

    This is useful for detailed debugging and log aggregation.
    """

    def format(self, record: logging.LogRecord) -> str:
        log_entry = {
            "type": "log",
            "level": record.levelname.lower(),
            "module": record.name,
            "message": record.getMessage(),
            "timestamp": datetime.utcnow().isoformat() + "Z",
        }

        # Add exception info if present
        if record.exc_info:
            log_entry["exception"] = self.formatException(record.exc_info)

        return json.dumps(log_entry, ensure_ascii=False)


class PackedModeHandler(logging.StreamHandler):
    """Log handler for packed mode.

    Always writes to stderr (since stdout is reserved for JSON-RPC).
    Uses PackedModeFormatter by default.
    """

    def __init__(self, use_json: bool = False):
        super().__init__(sys.stderr)
        if use_json:
            self.setFormatter(PackedModeJSONFormatter())
        else:
            self.setFormatter(PackedModeFormatter())


def configure_logging(
    level: Optional[str] = None,
    enabled: Optional[bool] = None,
    verbose: Optional[bool] = None,
    format_string: Optional[str] = None,
    use_json: bool = False,
) -> None:
    """Configure AuroraView logging.

    This function should be called early in your application startup
    if you need custom logging configuration. In packed mode, it
    automatically uses stderr with structured formatting.

    Args:
        level: Log level (DEBUG, INFO, WARNING, ERROR, CRITICAL).
               If None, uses AURORAVIEW_LOG_LEVEL env var or WARNING.
        enabled: Enable/disable logging. If None, uses AURORAVIEW_LOG_ENABLED.
        verbose: Enable verbose debug output. If None, uses AURORAVIEW_LOG_VERBOSE.
        format_string: Custom log format (ignored in packed mode).
        use_json: Use JSON format for logs (useful for structured logging).

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

    # Remove existing handlers
    root_logger.handlers.clear()

    # Choose handler based on mode
    if is_packed_mode():
        # In packed mode, use special handler that writes to stderr
        handler = PackedModeHandler(use_json=use_json)
    else:
        # In normal mode, use standard StreamHandler
        handler = logging.StreamHandler(sys.stderr)
        if format_string is None:
            format_string = "[%(name)s] %(levelname)s: %(message)s"
        handler.setFormatter(logging.Formatter(format_string))

    handler.setLevel(log_level)
    root_logger.addHandler(handler)

    _logging_configured = True


def get_logger(name: str) -> logging.Logger:
    """Get a logger for AuroraView components.

    This is the recommended way to get a logger. It automatically
    configures logging on first use and works correctly in both
    normal and packed modes.

    Args:
        name: Logger name (usually __name__).

    Returns:
        Configured logger instance.

    Example:
        >>> from auroraview.utils.logging import get_logger
        >>> logger = get_logger(__name__)
        >>> logger.debug("This is a debug message")
        >>> logger.info("Informational message")
        >>> logger.warning("Something might be wrong")
        >>> logger.error("Something went wrong")
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
