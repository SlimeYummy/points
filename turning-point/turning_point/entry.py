from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Mapping, Sequence, cast

from turning_point.attribute import PrimaryAttribute, SecondaryAttribute, _serialize_attributes_plus
from turning_point.base import (
    ResID,
    Resource,
    _serialize_int,
    _serialize_str,
    _serialize_list_int,
    cleanup,
)
from turning_point.config import MAX_ENTRY_PLUS
from turning_point.script import _extract_script_args, _serialize_script, _serialize_script_args_plus


@dataclass(kw_only=True)
class Entry(Resource):
    """
    词条 装备/饰品/宝石上需要凑够数量发动效果的技能
    """

    # 展示用的名字
    name: str

    # 图标
    icon: str

    # 主色调
    color: str | None = None

    # 词条的叠加上限 攻击7 生命3 之类的
    max_piece: int

    # 同一词条叠加带来的提升 List长度必须等于max_piece
    # 接受以下属性:
    # - PrimaryAttribute
    # - SecondaryAttribute
    # - SecondaryAttribute + Plus
    # - "SecondaryAttribute+"
    # 其中「+ Plus」表示「+」值堆叠带来的提升
    # 累计MAX_ENTRY_PLUS个「+」提升一次 共MAX_ENTRY_PLUS次
    attributes: Mapping[str, Sequence[float | str]] | None = None

    # 脚本
    script: str | None = None

    # 脚本参数
    # 接受形如以下的参数:
    # - xxx
    # - xxx + Plus
    # - "xxx+"
    # 其中「+ Plus」表示「+」值堆叠带来的提升
    script_args: Mapping[str, Sequence[float | str]] | None = None

    def serialize(self) -> dict[str, Any]:
        ser = {
            **super().serialize(),
            "name": _serialize_str(self.name, self.h("name")),
            "icon": _serialize_str(self.icon, self.h("icon")),
            "color": _serialize_str(self.color, self.h("color"), optional=True, regex=r"^#[0-9a-fA-F]{6}$"),
            "max_piece": _serialize_int(self.max_piece, min=1, max=99, where=self.h("max_piece")),
            # 'max_plus': self.max_piece * MAX_ENTRY_PLUS,
            "attributes": _serialize_attributes_plus(
                [PrimaryAttribute, SecondaryAttribute],
                self.attributes,
                self.max_piece,
                self.h("piece_attributes"),
                optional=True,
                zero=0,
            ),
            "script": _serialize_script(
                self.script,
                _extract_script_args(self.script_args, where=self.h("script_args")),
                self.h("script"),
                optional=True,
            ),
            "script_args": _serialize_script_args_plus(
                self.script_args,
                self.max_piece,
                self.h("script_args"),
                optional=True,
                zero=0,
            ),
        }
        return cleanup(ser)


def _serialize_entries(
    entries: Mapping[ResID, Sequence[int] | Sequence[Sequence[int]]] | None,
    size: int | None,
    where: str = "?",
    optional: bool = False,
    zero: Sequence[int] | None = None,
):
    if optional and entries is None:
        return None
    if not isinstance(entries, Mapping):
        raise Exception(f"{where}: must be a Mapping")

    ser = {}
    for id, pairs in entries.items():
        entry = Entry.get(id, f"{where}[{id}]")

        ser_value: Any = None
        if size is not None:
            if not isinstance(pairs, Sequence):
                raise Exception(f"{where}[{id}]: must be a Sequence")
            if len(pairs) != size:
                raise Exception(f"{where}[{id}]: size must = {size}")

            ser_value = [zero] if isinstance(zero, tuple | list) else []
            for idx, pair in enumerate(pairs):
                ser_pair = _serialize_list_int(cast(Any, pair), 2, f"{where}[{id}][{idx}]", min=0, max=99)
                if ser_pair[0] > entry.max_piece:
                    raise Exception(f"{where}[{id}][{idx}]: [0] must <= entry.max_piece")
                if ser_pair[1] > ser_pair[0] * MAX_ENTRY_PLUS:
                    raise Exception(f"{where}[{id}][{idx}]: [1] must <= [0] * {MAX_ENTRY_PLUS}")
                ser_value.append(ser_pair)

        else:
            ser_value = _serialize_list_int(cast(Any, pairs), 2, f"{where}[{id}]", min=0, max=99)
            if ser_value[0] > entry.max_piece:
                raise Exception(f"{where}[{id}]: [0] must <= entry.max_piece")
            if ser_value[1] > ser_value[0] * MAX_ENTRY_PLUS:
                raise Exception(f"{where}[{id}]: [1] must <= [0] * {MAX_ENTRY_PLUS}")

        ser[entry.id] = ser_value
    return ser
