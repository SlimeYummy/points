from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Generic, Mapping, Sequence, TypeVar

from lib.base.builtin import (
    _serialize_bool,
    _serialize_time,
    _serialize_float,
    _serialize_range_int,
    _serialize_list_time,
    _serialize_list_float,
    _serialize_symbol,
)


Switch = (0, 1)


def _serialize_inline_arguments(
    arguments: Mapping[str, Sequence[int]] | None,
    where: str = "?",
    optional: bool = False,
):
    if optional and not arguments:
        return None
    if not isinstance(arguments, Mapping):
        raise Exception(f"{where}: must be a Mapping")

    ser = {}
    for key, range in arguments.items():
        ser_key = _serialize_symbol(key, f"where[{key}]", regex=r"^[\w\_][\w\d\_]*$")
        ser_range = _serialize_range_int(range, f"where[{key}]", min=0)
        ser[ser_key] = ser_range
    return ser


def _serialize_inline_switch(
    arguments: Mapping[str, Sequence[int]],
    switch: bool | int | str | None,
    where: str = "?",
    optional: bool = False,
):
    if optional and switch is None:
        return None

    if isinstance(switch, bool | int):
        return _serialize_bool(switch, where, optional)
    elif isinstance(switch, str):
        if switch not in arguments:
            raise Exception(f"{where}: switch name not found")
        return _serialize_symbol(switch, where, regex=r"^[\w\_][\w\d\_]*$")


_V = TypeVar("_V")


@dataclass
class Inline(Generic[_V]):
    argument: str
    values: list[_V]

    def __init__(self, argument: str, *args: _V):
        self.argument = argument
        self.values = list(args)


IL = Inline


def _serialize_inline_time(
    collector: list[Any],
    arguments: Mapping[str, Sequence[int]],
    inline: int | str | Inline[int | str] | None,
    key: str,
    where: str = "?",
    optional: bool = False,
    min: int | None = None,
    max: int | None = None,
):
    if optional and inline is None:
        return None

    if isinstance(inline, int | float | str):
        return _serialize_time(inline, where, optional, min=min, max=max)

    elif isinstance(inline, Inline):
        size_range = arguments.get(inline.argument)
        if not size_range:
            raise Exception(f"{where}: inline name not found")
        size = size_range[1] - size_range[0] + 1

        ser = _serialize_list_time(inline.values, size, where, optional, min=min, max=max)
        collector.append({"k": (inline.argument, key), "v": ser})
        return None

    else:
        raise Exception(f"{where}: must be an int/time/Inline")


def _serialize_inline_float(
    collector: list[Any],
    arguments: Mapping[str, Sequence[int]],
    inline: float | str | Inline[float | str] | None,
    key: str,
    where: str = "?",
    optional: bool = False,
    min: float | None = None,
    max: float | None = None,
):
    if optional and inline is None:
        return None

    if isinstance(inline, int | float | str):
        return _serialize_float(inline, where, optional, min=min, max=max)

    elif isinstance(inline, Inline):
        size_range = arguments.get(inline.argument)
        if not size_range:
            raise Exception(f"{where}: inline name not found")
        size = size_range[1] - size_range[0] + 1

        ser = _serialize_list_float(inline.values, size, where, optional, min=min, max=max)
        collector.append({"k": (inline.argument, key), "v": ser})
        return None

    else:
        raise Exception(f"{where}: must be an int/float/percent/Inline")
