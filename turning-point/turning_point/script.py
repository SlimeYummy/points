from typing import Any, Mapping, Sequence, cast

from turning_point.base import _serialize_float, _serialize_symbol, _serialize_list_float, compile_script


_script_counter = 0


def _serialize_script(
    script: str | None,
    script_args: list[str] = [],
    where: str = "?",
    optional: bool = False,
):
    if optional and script is None:
        return None
    if not isinstance(script, str):
        raise Exception(f"{where}: must be a string")

    try:
        res = compile_script(script, script_args or [])
    except Exception as e:
        raise Exception(f'{where}: compile script "{e}"')

    global _script_counter
    _script_counter += 1
    return {"id": _script_counter, **res}


def _extract_script_args(
    *all_script_args: Mapping[str, float | str | Sequence[float | str]] | None,
    where: str = "?",
):
    names = []
    for script_args in all_script_args:
        if script_args is None:
            continue
        for name in script_args.keys():
            if name in names:
                raise Exception(f'{where}: duplicate script argument "{name}"')
            names.append(name)

    if len(names) == 0:
        return None
    return names


def _serialize_script_args(
    script_args: Mapping[str, float | str | Sequence[float | str]] | None,
    size: int | None,
    where: str = "?",
    optional: bool = False,
    zero: float | None = None,
):
    if optional and script_args is None:
        return None
    if not isinstance(script_args, Mapping):
        raise Exception(f"{where}: must be a Mapping")

    ser = {}
    for arg_name, arg_value in script_args.items():
        ser_arg = _serialize_symbol(arg_name, f"{where}[{arg_name}]", regex=r"^[\w\_][\w\d\_]*$")

        ser_value = None
        if size is not None:
            ser_value = _serialize_list_float(cast(Any, arg_value), size, f"{where}[{arg_name}]")
            if isinstance(zero, int | float):
                ser_value.insert(0, zero)
        else:
            ser_value = _serialize_float(cast(Any, arg_value), f"{where}[{arg_name}]")
        ser[ser_arg] = ser_value
    return ser


def _serialize_script_args_plus(
    script_args: Mapping[str, float | str | Sequence[float | str]] | None,
    size: int | None,
    where: str = "?",
    optional: bool = False,
    zero: float | None = None,
):
    if optional and script_args is None:
        return None
    if not isinstance(script_args, Mapping):
        raise Exception(f"{where}: must be a Mapping")

    ser = []
    script_args_set = set()
    for arg_name, arg_value in script_args.items():
        plus = arg_name.endswith("+")
        real_name = arg_name[:-1] if plus else arg_name
        if real_name in script_args_set:
            raise Exception(f"{where}[{arg_name}]: duplicate argument")
        script_args_set.add(real_name)

        ser_arg = _serialize_symbol(real_name, f"{where}[{arg_name}]", regex=r"^[\w\_][\w\d\_]*$")
        ser_value = None
        if size is not None:
            ser_value = _serialize_list_float(cast(Any, arg_value), size, f"{where}[{arg_name}]")
            if isinstance(zero, int | float):
                ser_value.insert(0, zero)
        else:
            ser_value = _serialize_float(cast(Any, arg_value), f"{where}[{arg_name}]")
        ser.append({"l": (ser_arg, plus), "v": ser_value})
    return ser
