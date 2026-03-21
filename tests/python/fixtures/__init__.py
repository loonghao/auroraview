# -*- coding: utf-8 -*-
"""AuroraView test fixtures for IPC roundtrip testing.

Provides reusable test handlers (echo, ping) and utilities for verifying
the complete IPC communication chain.
"""

from .ipc_handlers import EchoHandler, PingHandler, CollectorHandler

__all__ = [
    "EchoHandler",
    "PingHandler",
    "CollectorHandler",
]
