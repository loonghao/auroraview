# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AG-UI Protocol implementation for AuroraView AI.

This module provides the AG-UI (Agent-UI) protocol event emitter
for standardized AI-UI communication.
"""

from __future__ import annotations

import time
import uuid
from collections.abc import Callable
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class EventType(Enum):
    """AG-UI Event types."""

    # Run lifecycle
    RUN_STARTED = "RUN_STARTED"
    RUN_FINISHED = "RUN_FINISHED"
    RUN_ERROR = "RUN_ERROR"

    # Text message events
    TEXT_MESSAGE_START = "TEXT_MESSAGE_START"
    TEXT_MESSAGE_CONTENT = "TEXT_MESSAGE_CONTENT"
    TEXT_MESSAGE_END = "TEXT_MESSAGE_END"

    # Thinking/reasoning events
    THINKING_START = "THINKING_START"
    THINKING_CONTENT = "THINKING_CONTENT"
    THINKING_END = "THINKING_END"

    # Tool call events
    TOOL_CALL_START = "TOOL_CALL_START"
    TOOL_CALL_ARGS = "TOOL_CALL_ARGS"
    TOOL_CALL_END = "TOOL_CALL_END"
    TOOL_CALL_RESULT = "TOOL_CALL_RESULT"

    # State synchronization
    STATE_SNAPSHOT = "STATE_SNAPSHOT"
    STATE_DELTA = "STATE_DELTA"

    # Custom
    CUSTOM = "CUSTOM"


@dataclass
class AGUIEvent:
    """AG-UI Event structure."""

    type: EventType
    timestamp: float = field(default_factory=lambda: time.time() * 1000)
    run_id: str | None = None
    thread_id: str | None = None
    message_id: str | None = None
    tool_call_id: str | None = None
    delta: str | None = None
    content: str | None = None
    role: str | None = None
    tool_name: str | None = None
    arguments: str | None = None
    message: str | None = None
    code: str | None = None
    snapshot: dict[str, Any] | None = None
    name: str | None = None
    value: Any | None = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result: dict[str, Any] = {
            "type": self.type.value,
            "timestamp": self.timestamp,
        }
        for key, val in self.__dict__.items():
            if key in ("type", "timestamp"):
                continue
            if val is not None:
                result[key] = val
        return result


# Type alias for emit callback
EmitCallback = Callable[[str, dict[str, Any]], None]


class AGUIEventEmitter:
    """AG-UI Event emitter for streaming AI responses.

    This class handles emitting AG-UI protocol events to the frontend
    via a provided callback function.
    """

    def __init__(self, emit_callback: EmitCallback | None = None):
        """Initialize the emitter.

        Args:
            emit_callback: Callback function to emit events.
                Signature: (event_name: str, data: dict) -> None
        """
        self._emit_callback = emit_callback
        self._current_run_id: str | None = None
        self._current_message_id: str | None = None

    def set_callback(self, callback: EmitCallback) -> None:
        """Set the emit callback."""
        self._emit_callback = callback

    def emit(self, event: AGUIEvent) -> None:
        """Emit an AG-UI event."""
        if self._emit_callback:
            event_name = f"agui:{event.type.value.lower()}"
            self._emit_callback(event_name, event.to_dict())

    def run_started(self, thread_id: str) -> str:
        """Emit RUN_STARTED and return run_id."""
        self._current_run_id = str(uuid.uuid4())
        self.emit(AGUIEvent(
            type=EventType.RUN_STARTED,
            run_id=self._current_run_id,
            thread_id=thread_id,
        ))
        return self._current_run_id

    def run_finished(self, thread_id: str) -> None:
        """Emit RUN_FINISHED."""
        self.emit(AGUIEvent(
            type=EventType.RUN_FINISHED,
            run_id=self._current_run_id,
            thread_id=thread_id,
        ))

    def run_error(self, message: str, code: str | None = None) -> None:
        """Emit RUN_ERROR."""
        self.emit(AGUIEvent(
            type=EventType.RUN_ERROR,
            run_id=self._current_run_id,
            message=message,
            code=code,
        ))

    def text_start(self, role: str = "assistant") -> str:
        """Emit TEXT_MESSAGE_START and return message_id."""
        self._current_message_id = str(uuid.uuid4())
        self.emit(AGUIEvent(
            type=EventType.TEXT_MESSAGE_START,
            message_id=self._current_message_id,
            role=role,
        ))
        return self._current_message_id

    def text_delta(self, delta: str) -> None:
        """Emit TEXT_MESSAGE_CONTENT with delta."""
        self.emit(AGUIEvent(
            type=EventType.TEXT_MESSAGE_CONTENT,
            message_id=self._current_message_id,
            delta=delta,
        ))

    def text_end(self) -> None:
        """Emit TEXT_MESSAGE_END."""
        self.emit(AGUIEvent(
            type=EventType.TEXT_MESSAGE_END,
            message_id=self._current_message_id,
        ))

