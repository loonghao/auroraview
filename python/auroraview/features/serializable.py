"""Serializable mixin for dataclasses.

This module provides automatic to_dict/from_dict serialization for dataclasses,
handling common types like datetime, enum, and optional fields.
"""

from __future__ import annotations

import typing
from dataclasses import asdict, is_dataclass
from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Tuple, Type, TypeVar

T = TypeVar("T", bound="Serializable")


def _resolve_type_hints(cls):
    # type: (type) -> Dict[str, Any]
    """Resolve type hints for a class, handling `from __future__ import annotations`."""
    try:
        return typing.get_type_hints(cls)
    except Exception:
        return {}


class Serializable:
    """Mixin for dataclasses with automatic to_dict/from_dict.

    Usage:
        @dataclass
        class MyItem(Serializable):
            id: str
            name: str
            created_at: datetime = field(default_factory=datetime.now)

        # to_dict() is automatically provided
        # from_dict() is automatically provided as a classmethod
    """

    #: Set of field names to exclude from to_dict() when their value is None.
    _exclude_none_fields = frozenset()  # type: frozenset

    def to_dict(self) -> Dict[str, Any]:
        """Convert dataclass to dictionary.

        Handles common types:
        - datetime: converts to ISO format string
        - enum: converts to value
        - Nested dataclasses: recursively calls to_dict()

        Returns:
            Dictionary representation
        """
        if not is_dataclass(self):
            raise TypeError(f"{self.__class__.__name__} is not a dataclass")

        exclude_none = self._exclude_none_fields
        result = {}
        for field_name, field_value in asdict(self).items():
            # Skip None values for fields marked as exclude-when-None
            if exclude_none and field_name in exclude_none and field_value is None:
                continue
            # Handle datetime conversion
            if isinstance(field_value, datetime):
                result[field_name] = field_value.isoformat()
            # Handle enum conversion
            elif isinstance(field_value, Enum):
                result[field_name] = field_value.value
            # Handle nested dataclasses
            elif isinstance(field_value, dict):
                result[field_name] = _serialize_dict(field_value)
            elif isinstance(field_value, (list, set, tuple)):
                result[field_name] = _serialize_sequence(field_value)
            else:
                result[field_name] = field_value
        return result

    @classmethod
    def from_dict(cls: Type[T], data: Dict[str, Any]) -> T:
        """Create dataclass instance from dictionary.

        Handles common types:
        - datetime: parses from ISO format string
        - enum: creates from value
        - Nested dataclasses: recursively calls from_dict()

        Args:
            data: Dictionary with field values

        Returns:
            Dataclass instance
        """
        if not is_dataclass(cls):
            raise TypeError(f"{cls.__name__} is not a dataclass")

        from dataclasses import fields

        # Resolve real type hints (handles `from __future__ import annotations`)
        resolved_hints = _resolve_type_hints(cls)

        field_dict = {}
        for field in fields(cls):
            field_name = field.name
            if field_name not in data:
                continue

            value = data[field_name]

            # Use resolved type hint if available, otherwise fall back to field.type
            field_type = resolved_hints.get(field_name, field.type)

            is_optional = _is_optional_type(field_type)

            if value is None and is_optional:
                field_dict[field_name] = None
                continue

            actual_type = _extract_actual_type(field_type)

            # Handle datetime
            if _is_datetime_type(actual_type):
                if value:
                    try:
                        field_dict[field_name] = datetime.fromisoformat(value)
                    except (ValueError, TypeError):
                        field_dict[field_name] = value
                else:
                    field_dict[field_name] = value
            # Handle enum
            elif _is_enum_type(actual_type):
                try:
                    field_dict[field_name] = actual_type(value)
                except ValueError:
                    field_dict[field_name] = value
            # Handle nested dataclasses
            elif _is_dataclass_type(actual_type):
                if isinstance(value, dict):
                    field_dict[field_name] = actual_type.from_dict(value)
                else:
                    field_dict[field_name] = value
            # Handle collections
            elif _is_collection_type(actual_type):
                field_dict[field_name] = _deserialize_collection(value, actual_type)
            else:
                field_dict[field_name] = value

        return cls(**field_dict)


def _serialize_dict(d: Dict[str, Any]) -> Dict[str, Any]:
    """Serialize dictionary recursively."""
    result = {}
    for key, value in d.items():
        if isinstance(value, datetime):
            result[key] = value.isoformat()
        elif isinstance(value, Enum):
            result[key] = value.value
        elif isinstance(value, dict):
            result[key] = _serialize_dict(value)
        elif isinstance(value, (list, set, tuple)):
            result[key] = _serialize_sequence(value)
        else:
            result[key] = value
    return result


def _serialize_sequence(seq: Any) -> List[Any]:
    """Serialize sequence recursively."""
    if isinstance(seq, (list, tuple)):
        result = []
        for item in seq:
            if isinstance(item, datetime):
                result.append(item.isoformat())
            elif isinstance(item, Enum):
                result.append(item.value)
            elif isinstance(item, dict):
                result.append(_serialize_dict(item))
            elif isinstance(item, (list, set, tuple)):
                result.append(_serialize_sequence(item))
            else:
                result.append(item)
        return result
    elif isinstance(seq, set):
        result = []
        for item in seq:
            if isinstance(item, datetime):
                result.append(item.isoformat())
            elif isinstance(item, Enum):
                result.append(item.value)
            elif isinstance(item, dict):
                result.append(_serialize_dict(item))
            elif isinstance(item, (list, set, tuple)):
                result.append(_serialize_sequence(item))
            else:
                result.append(item)
        return result
    return []


def _deserialize_collection(data: Any, type_hint: Any) -> Any:
    """Deserialize collection based on type hint."""
    if data is None:
        return None

    # Extract element type from collection type
    origin = _get_origin(type_hint)
    args = _get_args(type_hint)

    if not args:
        return data

    elem_type = args[0]

    if _is_dataclass_type(elem_type):
        if origin is list:
            return [elem_type.from_dict(item) if isinstance(item, dict) else item for item in data]
        elif origin is set:
            return {elem_type.from_dict(item) if isinstance(item, dict) else item for item in data}
        elif origin is tuple:
            return tuple(
                elem_type.from_dict(item) if isinstance(item, dict) else item for item in data
            )

    return data


def _is_optional_type(type_hint: Any) -> bool:
    """Check if type hint is Optional[T]."""
    if type_hint is None:
        return False
    origin = _get_origin(type_hint)
    return origin is type(None) or (
        hasattr(type_hint, "__args__") and type(None) in _get_args(type_hint)
    )


def _extract_actual_type(type_hint: Any) -> Any:
    """Extract actual type from Optional[T]."""
    if not _is_optional_type(type_hint):
        return type_hint
    args = _get_args(type_hint)
    if args:
        for arg in args:
            if arg is not type(None):
                return arg
    return type_hint


def _is_datetime_type(type_hint: Any) -> bool:
    """Check if type hint is datetime."""
    return type_hint is datetime or (
        hasattr(type_hint, "__origin__") and type_hint.__origin__ is datetime
    )


def _is_enum_type(type_hint: Any) -> bool:
    """Check if type hint is Enum."""
    try:
        return isinstance(type_hint, type) and issubclass(type_hint, Enum)
    except TypeError:
        return False


def _is_dataclass_type(type_hint: Any) -> bool:
    """Check if type hint is a dataclass."""
    try:
        return is_dataclass(type_hint)
    except TypeError:
        return False


def _is_collection_type(type_hint: Any) -> bool:
    """Check if type hint is list, set, or tuple."""
    origin = _get_origin(type_hint)
    return origin in (list, set, tuple)


def _get_origin(type_hint: Any) -> Any:
    """Get origin of generic type (Python 3.8+ compatible)."""
    if hasattr(type_hint, "__origin__"):
        return type_hint.__origin__
    return None


def _get_args(type_hint: Any) -> Tuple[Any, ...]:
    """Get args of generic type (Python 3.8+ compatible)."""
    if hasattr(type_hint, "__args__"):
        return type_hint.__args__
    return ()
