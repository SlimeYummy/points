from __future__ import annotations
from dataclasses import dataclass
from typing import Any

from turning_point.base import (
    RareLevel,
    ResID,
    Resource,
    Variant1,
    VariantType,
    VariantX,
    _serialize_int,
    _serialize_rare_level,
    _serialize_variant_type,
    cleanup,
)
from turning_point.entry import Entry
from turning_point.slot import SlotType, _serialize_slot_type

#
# 宝石分为attack/defense/special三种类型，与slot配套。
# 依据词条价值分为R1/R2/R3三个稀有度，attack/defense常见于R1/R2，special常见于R3。
# 原则上宝石类型因与词条类型匹配，但不排除某些效果较强的会升格成R3&special。
# 宝石可以通过合成强化「+」值，具体参考词条中的「+」值。
#


@dataclass(kw_only=True)
class Jewel(Resource):
    """
    宝石，一类嵌入插槽(slot)的强化附件，需要与插槽配套。
    """

    # 词条类型 决定宝石嵌入那种插槽
    slot_type: SlotType | str

    # 稀有度等级
    rare: RareLevel

    # 对应词条
    entry: ResID

    # 词条的叠加数
    piece: int

    # 对应词条(副词条)
    sub_entry: ResID | None = None

    # 词条的叠加数(副叠加数)
    sub_piece: int | None = None

    # 变体类型 用于区分同稀有度下的同名宝石
    variant: VariantType = Variant1

    def serialize(self) -> dict[str, Any]:
        entry = Entry.get(self.entry, self.h("entry"))
        sub_entry = self.sub_entry and Entry.get(self.sub_entry, self.h("sub_entry"))
        ser = {
            **super().serialize(),
            "slot_type": _serialize_slot_type(self.slot_type, self.h("slot_type")),
            "rare": _serialize_rare_level(self.rare, self.h("rare")),
            "entry": entry.id,
            "piece": _serialize_int(self.piece, self.h("piece"), min=1, max=entry.max_piece),
            "sub_entry": sub_entry and sub_entry.id,
            "sub_piece": sub_entry
            and _serialize_int(
                self.sub_piece,
                self.h("entry"),
                min=1,
                max=sub_entry.max_piece,
            ),
            "variant": _serialize_variant_type(self.variant, self.h("variant")),
        }
        return cleanup(ser)

    @classmethod
    def new(
        cls,
        slot_type: SlotType | str,
        rare: RareLevel,
        entry: ResID,
        piece: int,
        variant: VariantType = Variant1,
    ) -> Jewel:
        if Entry.find(entry) is None:
            raise Exception(f"Entry '{entry}' not found")
        id = entry.replace("Entry.", "Jewel.") + f".{variant}"
        return Jewel(id, entry=entry, piece=piece, slot_type=slot_type, rare=rare, variant=Variant1)

    @classmethod
    def newX(
        cls,
        slot_type: SlotType | str,
        rare: RareLevel,
        entry: ResID,
        piece: int,
        sub_entry: ResID,
        sub_piece: int,
    ) -> Jewel:
        if Entry.find(entry) is None:
            raise Exception(f"<{entry}>.{entry}: Resource not found")
        id = entry.replace("Entry.", "Jewel.") + f".{VariantX}"
        return Jewel(
            id,
            slot_type=slot_type,
            rare=rare,
            entry=entry,
            piece=piece,
            sub_entry=sub_entry,
            sub_piece=sub_piece,
            variant=VariantX,
        )
