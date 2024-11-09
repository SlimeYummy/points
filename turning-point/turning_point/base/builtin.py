from __future__ import annotations
import os.path
import re
from typing import Any, Sequence

from turning_point.config import FPS, MAX_SYMBOL_LEN


def cleanup(target: Any, whites: Sequence[Any] = [], ignores: Sequence[str] = []):
    if whites is None or len(whites) == 0:
        whites = [None]

    if isinstance(target, dict):
        return {k: v for k, v in target.items() if (v not in whites or k in ignores)}
    elif isinstance(target, list):
        return [v for v in target if v not in whites]
    return None


def _serialize_bool(val: bool | int | None, where: str = "?", optional: bool = False):
    if optional and val is None:
        return None

    if not isinstance(val, bool | int):
        raise Exception(f"{where}: must be a bool")
    return bool(val)


def _serialize_int(
    val: int | bool | None,
    where: str = "?",
    optional: bool = False,
    min: int | None = None,
    max: int | None = None,
    allow_bool: bool = False,
):
    if optional and val is None:
        return None

    if allow_bool:
        if not isinstance(val, int | bool):
            raise Exception(f"{where}: must be an int/bool")
    else:
        if not isinstance(val, int):
            raise Exception(f"{where}: must be an int")

    if min is not None and val < min:
        raise Exception(f"{where}: must >= {min}")
    if max is not None and max < val:
        raise Exception(f"{where}: must <= {max}")
    return val


_RE_TIME = re.compile(r"^(\d+(?:\.\d+)*)(s|m|h|ms|min)$")


def time(s: str):
    capture = _RE_TIME.match(s)
    if not capture:
        raise Exception("Invalid time")
    tm = float(capture.group(1))
    match capture.group(2):
        case "s":
            return round(FPS * tm)
        case "m" | "min":
            return round(FPS * tm * 60)
        case "h":
            return round(FPS * tm * 60 * 24)
        case "ms":
            return round(FPS * tm / 1000)
    raise Exception("Invalid time")


def _serialize_time(
    val: int | str | None,
    where: str = "?",
    optional: bool = False,
    min: int | None = None,
    max: int | None = None,
):
    if optional and val is None:
        return None

    ser = 0
    if isinstance(val, int):
        ser = val
    elif isinstance(val, str):
        capture = _RE_TIME.match(val)
        if not capture:
            raise Exception(f"{where}: must be an int/time")
        tm = float(capture.group(1))
        match capture.group(2):
            case "s":
                ser = round(FPS * tm)
            case "m" | "min":
                ser = round(FPS * tm * 60)
            case "h":
                ser = round(FPS * tm * 60 * 24)
            case "ms":
                ser = round(FPS * tm / 1000)
            case _:
                raise Exception(f"{where}: must be an int/time")
    else:
        raise Exception(f"{where}: must be an int/time")

    if min is not None and ser < min:
        raise Exception(f"{where}: must >= {min}")
    if max is not None and max < ser:
        raise Exception(f"{where}: must <= {max}")
    return ser


_RE_PERCENT = re.compile(r"^\d+(?:\.\d+)?%$")


def _serialize_float(
    val: float | str | None,
    where: str = "?",
    optional: bool = False,
    min: float | None = None,
    max: float | None = None,
):
    if optional and val is None:
        return None

    ser = 0.0
    if isinstance(val, int | float):
        ser = val
    elif isinstance(val, str):
        if _RE_PERCENT.match(val):
            ser = float(val[:-1]) / 100
    else:
        raise Exception(f"{where}: must be an int/float/percent")

    if min is not None and ser < min:
        raise Exception(f"{where}: must >= {min}")
    if max is not None and max < ser:
        raise Exception(f"{where}: must <= {max}")
    return ser


def _serialize_list_int(
    val: Sequence[int | bool] | None,
    size: int,
    where: str = "?",
    optional: bool = False,
    min: int | None = None,
    max: int | None = None,
    min_len: int | None = None,
    max_len: int | None = None,
    allow_bool: bool = False,
    zero: int | None = None,
):
    if optional and val is None:
        return None

    if not isinstance(val, Sequence):
        raise Exception(f"{where}: must be a Sequence")
    if len(val) != size:
        raise Exception(f"{where}: size must = {size}")
    if min_len is not None and len(val) < min_len:
        raise Exception(f"{where}: len() must > {min_len}")
    if max_len is not None and max_len < len(val):
        raise Exception(f"{where}: len() must < {max_len}")

    ser = []
    if isinstance(zero, int):
        ser.append(zero)
    for idx, item in enumerate(val):
        ser.append(_serialize_int(item, f"{where}[{idx}]", False, min, max, allow_bool))
    return ser


def _serialize_range_int(
    val: Sequence[int] | None,
    where: str = "?",
    optional: bool = False,
    min: int | None = None,
    max: int | None = None,
):
    ser = _serialize_list_int(val, 2, where, optional, min, max)
    if ser is not None and ser[0] > ser[1]:
        raise Exception(f"{where}: range[0] must < range[1]")
    return ser


def _serialize_list_time(
    val: Sequence[int | str] | None,
    size: int,
    where: str = "?",
    optional: bool = False,
    min: int | None = None,
    max: int | None = None,
    min_len: int | None = None,
    max_len: int | None = None,
    zero: int | None = None,
):
    if optional and val is None:
        return None

    if not isinstance(val, Sequence):
        raise Exception(f"{where}: must be a Sequence")
    if len(val) != size:
        raise Exception(f"{where}: size must = {size}")
    if min_len is not None and len(val) < min_len:
        raise Exception(f"{where}: len() must > {min_len}")
    if max_len is not None and max_len < len(val):
        raise Exception(f"{where}: len() must < {max_len}")

    ser = []
    if isinstance(zero, int):
        ser.append(zero)
    for idx, item in enumerate(val):
        ser.append(_serialize_time(item, f"{where}[{idx}]", False, min, max))
    return ser


def _serialize_range_time(
    val: Sequence[int | str] | None,
    where: str = "?",
    optional: bool = False,
    min: int | None = None,
    max: int | None = None,
):
    ser = _serialize_list_time(val, 2, where, optional, min, max)
    if ser is not None and ser[0] > ser[1]:
        raise Exception(f"{where}: range[0] must < range[1]")
    return ser


def _serialize_list_float(
    val: Sequence[float | str] | None,
    size: int,
    where: str = "?",
    optional: bool = False,
    min: float | None = None,
    max: float | None = None,
    min_len: int | None = None,
    max_len: int | None = None,
    zero: float | None = None,
):
    if optional and val is None:
        return None

    if not isinstance(val, Sequence):
        raise Exception(f"{where}: must be a Sequence")
    if len(val) != size:
        raise Exception(f"{where}: size must = {size}")
    if min_len is not None and len(val) < min_len:
        raise Exception(f"{where}: len() must > {min_len}")
    if max_len is not None and max_len < len(val):
        raise Exception(f"{where}: len() must < {max_len}")

    ser = []
    if isinstance(zero, int | float):
        ser.append(zero)
    for idx, item in enumerate(val):
        ser.append(_serialize_float(item, f"{where}[{idx}]", False, min, max))
    return ser


def _serialize_range_float(
    val: Sequence[float | str] | None,
    where: str = "?",
    optional: bool = False,
    min: float | None = None,
    max: float | None = None,
):
    ser = _serialize_list_float(val, 2, where, optional, min, max)
    if ser is not None and ser[0] >= ser[1]:
        raise Exception(f"{where}: range[0] must < range[1]")
    return ser


def _serialize_str(
    val: str | None,
    where: str = "?",
    optional: bool = False,
    min_len: int | None = None,
    max_len: int | None = None,
    regex: str | None = None,
):
    if optional and val is None:
        return None

    if not isinstance(val, str):
        raise Exception(f"{where}: must be a str")
    if min_len is not None and len(val) < min_len:
        raise Exception(f"{where}: len() must > {min_len}")
    if max_len is not None and max_len < len(val):
        raise Exception(f"{where}: len() must < {max_len}")
    if regex is not None and re.match(regex, val) is None:
        raise Exception(f'{where}: must match pattern "{regex}"')
    return val


def _serialize_symbol(
    val: str | None,
    where: str = "?",
    optional: bool = False,
    regex: str | None = None,
):
    return _serialize_str(val, where, optional=optional, min_len=1, max_len=MAX_SYMBOL_LEN, regex=regex)


def _serialize_file(
    val: str | None,
    where: str = "?",
    optional: bool = False,
    ext: str | None = None,
    can_abs: bool = False,
):
    if optional and val is None:
        return None

    if not isinstance(val, str):
        raise Exception(f"{where}: must be a str")
    if ext and not val.endswith(ext):
        raise Exception(f"{where}: must have extension {ext}")
    
    if not can_abs and os.path.isabs(val):
        raise Exception(f"{where}: must be a relative path")
    return os.path.normpath(val).replace(os.path.sep, '/')
