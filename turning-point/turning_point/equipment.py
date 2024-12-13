from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Literal, Mapping, Sequence, get_args

from turning_point.attribute import PrimaryAttribute, SecondaryAttribute, _serialize_attributes
from turning_point.base import ResID, Resource, _serialize_range_int, _serialize_res_id, _serialize_str, cleanup
from turning_point.entry import _serialize_entries
from turning_point.script import _extract_script_args, _serialize_script, _serialize_script_args
from turning_point.slot import _serialize_slot_definitions

EquipmentPosition = Literal["Position1", "Position2", "Position3"]

Position1: EquipmentPosition = "Position1"
Position2: EquipmentPosition = "Position2"
Position3: EquipmentPosition = "Position3"

_EquipmentPosition_ = get_args(EquipmentPosition)


def _ser_equipment_type(val, where: str = "?", optional: bool = False):
    if optional and val is None:
        return None
    if val not in _EquipmentPosition_:
        raise Exception(f"{where}: must be an EquipmentPosition")
    return val


@dataclass(kw_only=True)
class Equipment(Resource):
    """
    武器&装备

    每角色3个装备槽 原则上分主武器/副武器/防具等部位 也可以根据角色调整
    不同部位差异体现在数值上 武器加攻击属性 防具加防御属性

    装备采用类似怪猎的派生树机制 消耗素材制作
    装备生产出来后 派生树上的对应节点被激活 即使后续装备升级 被激活装备也将一直可用

    装备区分等级 原则上同等级装备性能接近 方便将等级作为衡量强弱的标准
    """

    # 展示用的名字
    name: str

    # 图标
    icon: str

    # 角标图标
    sub_icon: str | None = None

    # 所属角色ID
    character: ResID

    # 装备类型 决定装备能用于哪个装备槽
    position: EquipmentPosition

    # 装备树中的父节点 {装备ID: 等级}
    parents: Mapping[ResID, int] | None = None

    # 最大等级
    level: Sequence[int]

    # 每一级的武器强化素材 [{素材ID: 数量}]
    materials: Sequence[Sequence[tuple[ResID, int]]] | None = None

    # 每一级的属性列表
    # 接受以下属性:
    # - PrimaryAttribute
    attributes: Mapping[str, Sequence[float | str]]

    # 每一级的插槽列
    slots: Sequence[str | tuple[int, int, int]] | None = None

    # 每一级的词条
    entries: Mapping[ResID, Sequence[tuple[int, int]]] | None = None

    # 脚本
    script: str | None = None

    # 脚本参数
    script_args: Mapping[str, Sequence[float | str]] | None = None

    def serialize(self) -> dict[str, Any]:
        character: Any = Resource.get_by("Character", self.character, self.h("character"))
        if not character or (self.id not in character.equipments):
            raise self.error("character", "Character and Equipmemt mismatch")

        ser = {
            **super().serialize(),
            "name": _serialize_str(self.name, self.h("name")),
            "icon": _serialize_str(self.icon, self.h("icon")),
            "sub_icon": _serialize_str(self.icon, self.h("sub_icon"), optional=True),
            "character": _serialize_res_id(self.character, "Character", self.h("character")),
            "position": _ser_equipment_type(self.position, self.h("position")),
            "parents": self._serialize_parents(),
            "level": _serialize_range_int(self.level, where=self.h("level"), min=0, max=99),
            # 'materials': self._ser_materials(),
            "attributes": _serialize_attributes(
                [PrimaryAttribute, SecondaryAttribute],
                self.attributes,
                self.level[1] - self.level[0] + 1,
                self.h("attributes"),
            ),
            "slots": _serialize_slot_definitions(
                self.slots,
                self.level[1] - self.level[0] + 1,
                self.h("slots"),
                optional=True,
            ),
            "entries": _serialize_entries(
                self.entries,
                self.level[1] - self.level[0] + 1,
                self.h("entries"),
                optional=True,
            ),
            "script": _serialize_script(
                self.script,
                _extract_script_args(self.script_args, where=self.h("script_args")),
                self.h("script"),
                optional=True,
            ),
            "script_args": _serialize_script_args(
                self.script_args,
                self.level[1] - self.level[0] + 1,
                self.h("script_args"),
                optional=True,
            ),
        }
        return cleanup(ser)

    def _serialize_parents(self):
        if self.parents is None:
            return None
        if not isinstance(self.parents, Mapping):
            raise Exception(self.h("parents") + ": must be a Mapping")

        ser = {}
        for pid, level in self.parents.items():
            parent = Equipment.get(pid, self.h(f"parents[{pid}]"))
            if parent.position != self.position:
                raise self.error(f"parents[{pid}]", "position missmatch with parent")
            if parent.character != self.character:
                raise self.error(f"parents[{pid}]", "character mismatch with parent")
            if level > parent.max_level:
                raise self.error(f"parents[{pid}]", "out of parent's max level")
            ser[pid] = level
        return ser

    # def _ser_materials(self):
    #     if not isinstance(self.materials, Sequence):
    #         raise self.error('materials', 'must be a Sequence[Mapping[ResID, int]]')

    #     for idx, dict in enumerate(self.materials):
    #         if not isinstance(dict, Mapping):
    #             raise self.error('materials[%d]' % idx, 'must be a Mapping')

    #         # TODO: ...
    #         for id, cnt in dict.items():
    #             Resource.get(id, self.h('materials[%d][%s]' % (idx, id)))
    #             _serialize_int(cnt, min=0, where=self.h('materials[%d][%s]' % (idx, id)))

    #     return _list_table(self.materials)
