# Copyright (c) 2025 Long Hao
# Licensed under the MIT License
"""AG-UI Protocol types for Python.

This module defines the AG-UI (Agent-UI) protocol types used for
standardized AI-UI communication.
"""

from __future__ import annotations

import re
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional


def sanitize_string(s: str) -> str:
    """Remove invalid Unicode surrogate characters from a string.

    Windows file paths can sometimes contain surrogate pairs that are
    invalid in UTF-8. This function removes them to prevent encoding errors.

    Args:
        s: Input string that may contain invalid surrogates

    Returns:
        Clean string with surrogates removed
    """
    if not s:
        return s
    # Remove lone surrogates (U+D800 to U+DFFF)
    return re.sub(r"[\ud800-\udfff]", "", s)


def sanitize_dict(d: Dict[str, Any]) -> Dict[str, Any]:
    """Recursively sanitize all strings in a dictionary.

    Args:
        d: Dictionary to sanitize

    Returns:
        Sanitized dictionary with clean strings
    """
    result = {}
    for k, v in d.items():
        k = sanitize_string(k) if isinstance(k, str) else k
        result[k] = sanitize_value(v)
    return result


def sanitize_list(lst: List[Any]) -> List[Any]:
    """Recursively sanitize all strings in a list.

    Args:
        lst: List to sanitize

    Returns:
        Sanitized list with clean strings
    """
    return [sanitize_value(item) for item in lst]


def sanitize_value(v: Any) -> Any:
    """Sanitize a value, handling strings, dicts, and lists.

    Args:
        v: Value to sanitize

    Returns:
        Sanitized value
    """
    if isinstance(v, str):
        return sanitize_string(v)
    elif isinstance(v, dict):
        return sanitize_dict(v)
    elif isinstance(v, list):
        return sanitize_list(v)
    return v


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
    TEXT_MESSAGE_CHUNK = "TEXT_MESSAGE_CHUNK"

    # Thinking/reasoning events
    THINKING_TEXT_MESSAGE_START = "THINKING_TEXT_MESSAGE_START"
    THINKING_TEXT_MESSAGE_CONTENT = "THINKING_TEXT_MESSAGE_CONTENT"
    THINKING_TEXT_MESSAGE_END = "THINKING_TEXT_MESSAGE_END"
    THINKING_START = "THINKING_START"
    THINKING_END = "THINKING_END"

    # Tool call events
    TOOL_CALL_START = "TOOL_CALL_START"
    TOOL_CALL_ARGS = "TOOL_CALL_ARGS"
    TOOL_CALL_END = "TOOL_CALL_END"
    TOOL_CALL_CHUNK = "TOOL_CALL_CHUNK"
    TOOL_CALL_RESULT = "TOOL_CALL_RESULT"

    # Step events
    STEP_STARTED = "STEP_STARTED"
    STEP_FINISHED = "STEP_FINISHED"

    # State synchronization
    STATE_SNAPSHOT = "STATE_SNAPSHOT"
    STATE_DELTA = "STATE_DELTA"
    MESSAGES_SNAPSHOT = "MESSAGES_SNAPSHOT"

    # Extension
    RAW = "RAW"
    CUSTOM = "CUSTOM"


@dataclass
class AGUIEvent:
    """AG-UI Event structure.

    This is the base event class that all AG-UI events use.
    It can be serialized to JSON for transmission to the frontend.
    """

    type: EventType
    timestamp: float = field(default_factory=lambda: time.time() * 1000)

    # Common optional fields
    run_id: Optional[str] = None
    thread_id: Optional[str] = None
    message_id: Optional[str] = None
    tool_call_id: Optional[str] = None
    step_id: Optional[str] = None
    thinking_id: Optional[str] = None

    # Content fields
    delta: Optional[str] = None
    content: Optional[str] = None
    role: Optional[str] = None

    # Tool call fields
    tool_name: Optional[str] = None
    arguments: Optional[str] = None

    # Error fields
    message: Optional[str] = None
    code: Optional[str] = None

    # State fields
    snapshot: Optional[Dict[str, Any]] = None
    messages: Optional[List[Dict[str, Any]]] = None

    # Custom/raw fields
    event: Optional[str] = None
    name: Optional[str] = None
    data: Optional[Any] = None
    value: Optional[Any] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result = {
            "type": self.type.value,
            "timestamp": self.timestamp,
        }

        # Add non-None fields
        for key, value in self.__dict__.items():
            if key in ("type", "timestamp"):
                continue
            if value is not None:
                result[key] = value

        return result

    @classmethod
    def run_started(cls, run_id: str, thread_id: str) -> "AGUIEvent":
        """Create a RUN_STARTED event."""
        return cls(
            type=EventType.RUN_STARTED,
            run_id=run_id,
            thread_id=thread_id,
        )

    @classmethod
    def run_finished(cls, run_id: str, thread_id: str) -> "AGUIEvent":
        """Create a RUN_FINISHED event."""
        return cls(
            type=EventType.RUN_FINISHED,
            run_id=run_id,
            thread_id=thread_id,
        )

    @classmethod
    def run_error(cls, run_id: str, message: str, code: Optional[str] = None) -> "AGUIEvent":
        """Create a RUN_ERROR event."""
        return cls(
            type=EventType.RUN_ERROR,
            run_id=run_id,
            message=message,
            code=code,
        )

    @classmethod
    def text_start(cls, message_id: str, role: str = "assistant") -> "AGUIEvent":
        """Create a TEXT_MESSAGE_START event."""
        return cls(
            type=EventType.TEXT_MESSAGE_START,
            message_id=message_id,
            role=role,
        )

    @classmethod
    def text_delta(cls, message_id: str, delta: str) -> "AGUIEvent":
        """Create a TEXT_MESSAGE_CONTENT event."""
        return cls(
            type=EventType.TEXT_MESSAGE_CONTENT,
            message_id=message_id,
            delta=delta,
        )

    @classmethod
    def text_end(cls, message_id: str) -> "AGUIEvent":
        """Create a TEXT_MESSAGE_END event."""
        return cls(
            type=EventType.TEXT_MESSAGE_END,
            message_id=message_id,
        )

    @classmethod
    def thinking_start(cls, message_id: str) -> "AGUIEvent":
        """Create a THINKING_TEXT_MESSAGE_START event."""
        return cls(
            type=EventType.THINKING_TEXT_MESSAGE_START,
            message_id=message_id,
        )

    @classmethod
    def thinking_delta(cls, message_id: str, delta: str) -> "AGUIEvent":
        """Create a THINKING_TEXT_MESSAGE_CONTENT event."""
        return cls(
            type=EventType.THINKING_TEXT_MESSAGE_CONTENT,
            message_id=message_id,
            delta=delta,
        )

    @classmethod
    def thinking_end(cls, message_id: str) -> "AGUIEvent":
        """Create a THINKING_TEXT_MESSAGE_END event."""
        return cls(
            type=EventType.THINKING_TEXT_MESSAGE_END,
            message_id=message_id,
        )

    @classmethod
    def tool_call_start(cls, message_id: str, tool_call_id: str, tool_name: str) -> "AGUIEvent":
        """Create a TOOL_CALL_START event."""
        return cls(
            type=EventType.TOOL_CALL_START,
            message_id=message_id,
            tool_call_id=tool_call_id,
            tool_name=tool_name,
        )

    @classmethod
    def tool_call_args(cls, tool_call_id: str, delta: str) -> "AGUIEvent":
        """Create a TOOL_CALL_ARGS event."""
        return cls(
            type=EventType.TOOL_CALL_ARGS,
            tool_call_id=tool_call_id,
            delta=delta,
        )

    @classmethod
    def tool_call_end(cls, tool_call_id: str) -> "AGUIEvent":
        """Create a TOOL_CALL_END event."""
        return cls(
            type=EventType.TOOL_CALL_END,
            tool_call_id=tool_call_id,
        )

    @classmethod
    def tool_call_result(cls, tool_call_id: str, content: str) -> "AGUIEvent":
        """Create a TOOL_CALL_RESULT event."""
        return cls(
            type=EventType.TOOL_CALL_RESULT,
            tool_call_id=tool_call_id,
            role="tool",
            content=content,
        )

    @classmethod
    def state_snapshot(cls, snapshot: Dict[str, Any]) -> "AGUIEvent":
        """Create a STATE_SNAPSHOT event."""
        return cls(
            type=EventType.STATE_SNAPSHOT,
            snapshot=snapshot,
        )

    @classmethod
    def custom(cls, name: str, value: Any) -> "AGUIEvent":
        """Create a CUSTOM event."""
        return cls(
            type=EventType.CUSTOM,
            name=name,
            value=value,
        )


@dataclass
class Message:
    """Chat message structure."""

    id: str
    role: str  # "system", "user", "assistant", "tool"
    content: str
    tool_calls: Optional[List[Dict[str, Any]]] = None
    tool_call_id: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        result = {
            "id": self.id,
            "role": self.role,
            "content": self.content,
        }
        if self.tool_calls:
            result["tool_calls"] = self.tool_calls
        if self.tool_call_id:
            result["tool_call_id"] = self.tool_call_id
        return result


@dataclass
class Session:
    """Chat session state."""

    id: str
    messages: List[Message] = field(default_factory=list)
    system_prompt: Optional[str] = None

    def add_message(self, message: Message) -> None:
        """Add a message to the session."""
        self.messages.append(message)

    def add_user_message(self, content: str, message_id: Optional[str] = None) -> Message:
        """Add a user message."""
        import uuid

        msg = Message(
            id=message_id or str(uuid.uuid4()),
            role="user",
            content=content,
        )
        self.messages.append(msg)
        return msg

    def add_assistant_message(self, content: str, message_id: Optional[str] = None) -> Message:
        """Add an assistant message."""
        import uuid

        msg = Message(
            id=message_id or str(uuid.uuid4()),
            role="assistant",
            content=content,
        )
        self.messages.append(msg)
        return msg

    def clear(self) -> None:
        """Clear all messages."""
        self.messages.clear()

    def get_messages_for_api(self) -> List[Dict[str, str]]:
        """Get messages in API format."""
        result = []
        if self.system_prompt:
            result.append({"role": "system", "content": self.system_prompt})
        for msg in self.messages:
            result.append({"role": msg.role, "content": msg.content})
        return result
