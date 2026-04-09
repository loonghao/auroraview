"""Tests for auroraview_ai.protocol module."""

from __future__ import annotations

from unittest.mock import MagicMock

import pytest

from auroraview_ai.protocol import AGUIEvent, AGUIEventEmitter, EventType


class TestEventType:
    """Tests for EventType enum."""

    def test_all_lifecycle_events_defined(self) -> None:
        assert EventType.RUN_STARTED
        assert EventType.RUN_FINISHED
        assert EventType.RUN_ERROR

    def test_all_text_events_defined(self) -> None:
        assert EventType.TEXT_MESSAGE_START
        assert EventType.TEXT_MESSAGE_CONTENT
        assert EventType.TEXT_MESSAGE_END

    def test_all_thinking_events_defined(self) -> None:
        assert EventType.THINKING_START
        assert EventType.THINKING_CONTENT
        assert EventType.THINKING_END

    def test_all_tool_events_defined(self) -> None:
        assert EventType.TOOL_CALL_START
        assert EventType.TOOL_CALL_ARGS
        assert EventType.TOOL_CALL_END
        assert EventType.TOOL_CALL_RESULT

    def test_state_events_defined(self) -> None:
        assert EventType.STATE_SNAPSHOT
        assert EventType.STATE_DELTA
        assert EventType.CUSTOM

    def test_event_type_values_are_uppercase_strings(self) -> None:
        for event in EventType:
            assert event.value == event.value.upper()


class TestAGUIEvent:
    """Tests for AGUIEvent dataclass."""

    def test_minimal_construction(self) -> None:
        event = AGUIEvent(type=EventType.RUN_STARTED)
        assert event.type == EventType.RUN_STARTED
        assert event.timestamp > 0
        assert event.run_id is None

    def test_to_dict_includes_type_and_timestamp(self) -> None:
        event = AGUIEvent(type=EventType.RUN_STARTED)
        d = event.to_dict()
        assert d["type"] == "RUN_STARTED"
        assert "timestamp" in d

    def test_to_dict_excludes_none_fields(self) -> None:
        event = AGUIEvent(type=EventType.TEXT_MESSAGE_CONTENT, delta="hello")
        d = event.to_dict()
        assert "delta" in d
        assert d["delta"] == "hello"
        # None fields should not appear
        assert "run_id" not in d

    def test_to_dict_with_all_fields(self) -> None:
        event = AGUIEvent(
            type=EventType.TOOL_CALL_START,
            run_id="run-1",
            thread_id="thread-1",
            message_id="msg-1",
            tool_call_id="tc-1",
            tool_name="my_tool",
            role="assistant",
        )
        d = event.to_dict()
        assert d["run_id"] == "run-1"
        assert d["thread_id"] == "thread-1"
        assert d["tool_name"] == "my_tool"
        assert d["role"] == "assistant"

    def test_to_dict_snapshot_field(self) -> None:
        snapshot = {"key": "value", "count": 42}
        event = AGUIEvent(type=EventType.STATE_SNAPSHOT, snapshot=snapshot)
        d = event.to_dict()
        assert d["snapshot"] == snapshot

    def test_to_dict_value_field(self) -> None:
        event = AGUIEvent(type=EventType.CUSTOM, name="my_event", value={"data": 123})
        d = event.to_dict()
        assert d["name"] == "my_event"
        assert d["value"] == {"data": 123}

    def test_timestamp_is_milliseconds(self) -> None:
        import time
        before = time.time() * 1000
        event = AGUIEvent(type=EventType.RUN_STARTED)
        after = time.time() * 1000
        assert before <= event.timestamp <= after


class TestAGUIEventEmitter:
    """Tests for AGUIEventEmitter."""

    def test_init_without_callback(self) -> None:
        emitter = AGUIEventEmitter()
        assert emitter._emit_callback is None
        assert emitter._current_run_id is None
        assert emitter._current_message_id is None

    def test_init_with_callback(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        assert emitter._emit_callback is cb

    def test_set_callback(self) -> None:
        emitter = AGUIEventEmitter()
        cb = MagicMock()
        emitter.set_callback(cb)
        assert emitter._emit_callback is cb

    def test_emit_calls_callback(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        event = AGUIEvent(type=EventType.RUN_STARTED)
        emitter.emit(event)
        cb.assert_called_once()
        call_args = cb.call_args
        assert call_args[0][0] == "agui:run_started"
        assert call_args[0][1]["type"] == "RUN_STARTED"

    def test_emit_without_callback_is_noop(self) -> None:
        emitter = AGUIEventEmitter()
        # Should not raise
        emitter.emit(AGUIEvent(type=EventType.RUN_STARTED))

    def test_run_started_returns_run_id(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        run_id = emitter.run_started("thread-1")
        assert run_id is not None
        assert len(run_id) > 0
        assert emitter._current_run_id == run_id

    def test_run_started_emits_correct_event(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.run_started("t1")
        event_name, event_data = cb.call_args[0]
        assert event_name == "agui:run_started"
        assert event_data["type"] == "RUN_STARTED"
        assert event_data["thread_id"] == "t1"

    def test_run_finished_emits_correct_event(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.run_started("t1")
        emitter.run_finished("t1")
        last_call = cb.call_args_list[-1]
        event_name, event_data = last_call[0]
        assert event_name == "agui:run_finished"
        assert event_data["type"] == "RUN_FINISHED"

    def test_run_error_emits_correct_event(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.run_error("something went wrong", code="E001")
        event_name, event_data = cb.call_args[0]
        assert event_name == "agui:run_error"
        assert event_data["message"] == "something went wrong"
        assert event_data["code"] == "E001"

    def test_run_error_without_code(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.run_error("error msg")
        event_name, event_data = cb.call_args[0]
        assert event_name == "agui:run_error"
        assert "code" not in event_data

    def test_text_start_returns_message_id(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        msg_id = emitter.text_start()
        assert msg_id is not None
        assert emitter._current_message_id == msg_id

    def test_text_start_emits_correct_event(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.text_start(role="assistant")
        event_name, event_data = cb.call_args[0]
        assert event_name == "agui:text_message_start"
        assert event_data["role"] == "assistant"

    def test_text_delta_emits_content(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.text_start()
        emitter.text_delta("hello world")
        event_name, event_data = cb.call_args[0]
        assert event_name == "agui:text_message_content"
        assert event_data["delta"] == "hello world"

    def test_text_end_emits_end_event(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.text_start()
        emitter.text_end()
        event_name, event_data = cb.call_args[0]
        assert event_name == "agui:text_message_end"
        assert event_data["type"] == "TEXT_MESSAGE_END"

    def test_full_text_flow(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.run_started("session-1")   # 1
        emitter.text_start()               # 2
        emitter.text_delta("Hello")        # 3
        emitter.text_delta(", World")      # 4
        emitter.text_end()                 # 5
        emitter.run_finished("session-1")  # 6
        assert cb.call_count == 6

    def test_run_id_persists_across_calls(self) -> None:
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        run_id = emitter.run_started("t1")
        emitter.run_finished("t1")
        # run_id should be included in run_finished
        last_call = cb.call_args_list[-1]
        event_data = last_call[0][1]
        assert event_data["run_id"] == run_id

    def test_emit_event_name_format(self) -> None:
        """Event name should be 'agui:' + lowercase event type."""
        cb = MagicMock()
        emitter = AGUIEventEmitter(cb)
        emitter.emit(AGUIEvent(type=EventType.TOOL_CALL_START))
        event_name = cb.call_args[0][0]
        assert event_name == "agui:tool_call_start"
