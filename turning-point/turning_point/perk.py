from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from turning_point.action import _serialize_action_args
from turning_point.attribute import PrimaryAttribute, SecondaryAttribute, _serialize_attributes
from turning_point.base import (
    ResID,
    Resource,
    _serialize_res_id,
    _serialize_res_ids,
    _serialize_str,
    cleanup,
)
from turning_point.entry import _serialize_entries
from turning_point.script import _extract_script_args, _serialize_script, _serialize_script_args
from turning_point.slot import _serialize_slot_definition


@dataclass(kw_only=True)
class Perk(Resource):
    """
    天赋，即天赋树上的天赋加点。
    """

    # 展示用的名字
    name: str

    # 图标
    icon: str

    # 所属的风格 拥有该风格时才能点亮天赋
    style: ResID

    # 可以启用该天赋的角色风格
    usable_styles: Sequence[ResID] | None = None

    # 天赋树中的父节点 [天赋ID, 等级]
    parents: Mapping[ResID, int] | None = None

    # 每一级的属性列表
    # 接受以下属性:
    # - PrimaryAttribute
    # - SecondaryAttribute
    attributes: Mapping[str, float | str] | None = None

    # 每一级的插槽列
    slot: str | Sequence[int] | None = None

    # 每一级的词条
    entries: Mapping[ResID, Sequence[int]] | None = None

    # 动作参数配置
    action_args: Mapping[str, int] | None = None

    # 脚本
    script: str | None = None

    # 脚本参数
    script_args: Mapping[str, float | str] | None = None

    def serialize(self) -> dict[str, Any]:
        style: Any = Resource.get_by("Style", self.style, self.h("style"))
        if not style or (self.id not in style.perks):
            raise self.error("style", "Style and Perk mismatch")

        if self.usable_styles:
            if self.style in self.usable_styles:
                raise self.error("style", "style doesn't need to be in usable_style")

            for idx, id in enumerate(self.usable_styles):
                usable_style: Any = Resource.get_by("Style", id, self.h("usable_styles[{idx}]"))
                if not usable_style or (self.id not in usable_style.usable_perks):
                    raise self.error(f"usable_styles[{idx}]", "Style and Perk mismatch")
                if style.character != usable_style.character:
                    raise self.error(f"usable_styles[{idx}]", "style/usable_styles must in same Character")

        ser = {
            **super().serialize(),
            "name": _serialize_str(self.name, self.h("name")),
            "icon": _serialize_str(self.icon, self.h("icon")),
            "style": _serialize_res_id(self.style, "Style", self.h("style")),
            "usable_styles": _serialize_res_ids(
                self.usable_styles,
                "Style",
                self.h("usable_styles"),
                optional=True,
                min_len=1,
            ),
            "parents": self._ser_parents(),
            "attributes": _serialize_attributes(
                [PrimaryAttribute, SecondaryAttribute],
                self.attributes,
                None,
                self.h("attributes"),
                optional=True,
            ),
            "slot": _serialize_slot_definition(self.slot, self.h("slot"), optional=True),
            "entries": _serialize_entries(self.entries, None, self.h("entries"), optional=True),
            "action_args": _serialize_action_args(self.action_args, None, self.h("action_args"), optional=True),
            "script": _serialize_script(
                self.script,
                _extract_script_args(self.script_args, where=self.h("script_args")),
                self.h("script"),
                optional=True,
            ),
            "script_args": _serialize_script_args(
                self.script_args,
                None,
                self.h("script_args"),
                optional=True,
            ),
        }
        return cleanup(ser)

    def _ser_parents(self):
        if self.parents is None:
            return None
        if not isinstance(self.parents, Mapping):
            raise self.error("parents", "must be a Mapping")

        for pid, level in self.parents.items():
            parent = Perk.get(pid, self.h(f"parents[{pid}]"))
            if parent.character != self.character:
                raise self.error(f"parents[{pid}]", "character mismatch with parent")
            if level > parent.max_level:
                raise self.error(f"parents[{pid}]", "out of parent's max level")
        return self.parents
