from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from turning_point.attribute import PrimaryAttribute, SecondaryAttribute, _serialize_attributes
from turning_point.base import (
    ResID,
    Resource,
    _serialize_file,
    _serialize_float,
    _serialize_range_int,
    _serialize_res_ids,
    _serialize_str,
    cleanup,
    read_skeleton_meta,
    Capsule,
)
from turning_point.slot import _serialize_slot_definitions


@dataclass(kw_only=True)
class Character(Resource):
    """
    角色，即游戏中的一个角色，如LK/LL/WQ/YJ等。
    注意区分Character与Style，一个Character对应多个Style。
    Character里包含该角色所有Style通用的数据。
    """

    # 角色名字
    name: str

    # 最大等级
    level: Sequence[int]

    # 风格ID列表
    styles: Sequence[ResID]

    # 装备ID列表
    equipments: Sequence[ResID]

    # 用于移动的包围胶囊体
    bounding_capsule: Capsule

    # 用于骨骼动画的模型文件(ozz)
    skeleton: str

    def serialize(self) -> dict[str, Any]:
        ser = {
            **super().serialize(),
            "name": _serialize_str(self.name, self.h("name")),
            "level": _serialize_range_int(self.level, self.h("level"), min=0),
            "styles": _serialize_res_ids(
                self.styles, "Style", self.h("styles"), reference=lambda style: style.character == self.id
            ),
            "equipments": _serialize_res_ids(
                self.equipments, "Equipment", self.h("equipments"), reference=lambda equip: equip.character == self.id
            ),
            "bounding_capsule": self.bounding_capsule.serialize(self.h("bounding_capsule")),
            "skeleton": _serialize_str(self.skeleton, self.h("skeleton")),
        }
        return cleanup(ser)


@dataclass(kw_only=True)
class Style(Resource):
    """
    风格，角色的风格，可以理解为角色的一种职业或一套build。
    注意区分Character与Style，一个Character对应多个Style。
    Style里包含Character下各个Style独有的数据。
    """

    # 风格名字
    name: str

    # 所属角色ID
    character: ResID

    # 每一级的属性
    # 接受以下属性:
    # - PrimaryAttribute
    # - SecondaryAttribute
    attributes: Mapping[str, Sequence[float | str]]

    # 每一级的插槽列
    slots: Sequence[str | tuple[int, int, int]]

    # 不随等级变动的属性
    fixed_attributes: FixedAttributes

    # 拥有的Perk列表 即该风格可以点亮的Perk
    perks: Sequence[ResID]

    # 可以使用的Perk列表 包含了其他由Style点亮 但该Style也可使用的Perk
    usable_perks: Sequence[ResID] | None = None

    # 可用的动作列表
    actions: Sequence[ResID]

    # 图标
    icon: str

    # 角色模型（渲染）
    view_model: str

    _skeleton_meta: Any = None

    def skeleton_joint_counts(self) -> int:
        if not self._skeleton_meta:
            _skeleton_meta = read_skeleton_meta(self.skeleton)
        return self._skeleton_meta.get("joint_counts", -1)

    def serialize(self) -> dict[str, Any]:
        character: Character = Character.get(self.character, self.h("character"))
        if not character or (self.id not in character.styles):
            raise self.error("character", "Character and Style mismatch")

        ser = {
            **super().serialize(),
            "name": _serialize_str(self.name, self.h("name")),
            "character": self.character,
            "attributes": _serialize_attributes(
                [PrimaryAttribute, SecondaryAttribute],
                self.attributes,
                character.level[1] - character.level[0] + 1,
                self.h("attributes"),
            ),
            "slots": _serialize_slot_definitions(
                self.slots, character.level[1] - character.level[0] + 1, self.h("slots")
            ),
            "fixed_attributes": self.fixed_attributes.serialize(self.h("fixed_attributes")),
            "perks": _serialize_res_ids(
                self.perks,
                "Perk",
                self.h("perks"),
                reference=lambda perk: self.id == perk.style,
            ),
            "usable_perks": _serialize_res_ids(
                self.usable_perks,
                "Perk",
                self.h("usable_perks"),
                optional=True,
                reference=lambda perk: self.id in perk.usable_styles,
            ),
            "actions": _serialize_res_ids(
                self.actions,
                "Action",
                self.h("actions"),
                # reference=lambda action: action.general or self.id in action.styles,
            ),
            "icon": _serialize_str(self.icon, self.h("icon")),
            "view_model": _serialize_file(self.view_model, self.h("view_model"), ext=".vrm"),
        }
        return cleanup(ser)


@dataclass(kw_only=True)
class FixedAttributes:
    # 用于计算常规状态伤害减免
    # 公式 P1 + (1 - P1) * defense / (P2 + defense)
    damage_reduce_param_1: float | str
    damage_reduce_param_2: float | str

    # 防御状态下伤害减免率
    guard_damage_ratio_1: float | str

    # 用于计算常规状态架势伤害减免
    # 公式  P1 + (1 - P1) * defense / (P2 + defense)
    deposture_reduce_param_1: float | str
    deposture_reduce_param_2: float | str

    # 防御状态下架势伤害减免率
    guard_deposture_ratio_1: float | str

    # 对虚弱状态下的敌人增伤
    weak_damage_up: float | str

    def serialize(self, where: str) -> dict[str, Any]:
        return cleanup(
            {
                "damage_reduce_param_1": _serialize_float(
                    self.damage_reduce_param_1,
                    f"{where}.damage_reduce_param_1",
                    min=0,
                    max=1,
                ),
                "damage_reduce_param_2": _serialize_float(
                    self.damage_reduce_param_2, f"{where}.damage_reduce_param_2", min=0
                ),
                "guard_damage_ratio_1": _serialize_float(
                    self.guard_damage_ratio_1, f"{where}.guard_damage_ratio_1", min=0, max=1
                ),
                "deposture_reduce_param_1": _serialize_float(
                    self.deposture_reduce_param_1,
                    f"{where}.deposture_reduce_param_1",
                    min=0,
                    max=1,
                ),
                "deposture_reduce_param_2": _serialize_float(
                    self.deposture_reduce_param_2,
                    f"{where}.deposture_reduce_param_2",
                    min=0,
                ),
                "guard_deposture_ratio_1": _serialize_float(
                    self.guard_deposture_ratio_1,
                    f"{where}.guard_deposture_ratio_1",
                    min=0,
                    max=1,
                ),
                "weak_damage_up": _serialize_float(self.weak_damage_up, f"{where}.weak_damage_up", min=0),
            }
        )
