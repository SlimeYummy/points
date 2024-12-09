from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Mapping

from turning_point.base import (
    RareLevel,
    ResID,
    Resource,
    Variant1,
    VariantType,
    _serialize_int,
    _serialize_rare_level,
    _serialize_res_ids_float,
    _serialize_variant_type,
    cleanup,
)
from turning_point.entry import Entry, MAX_ENTRY_PLUS


@dataclass(kw_only=True)
class AccessoryPattern(Resource):
    # 稀有度等级
    rare: RareLevel

    # 随机词条生成模式
    pattern: str

    # 最高等级
    max_level: int

    # A词条池 高价值随机词条池 数字表示概率占比
    a_pool: Mapping[ResID, float]

    # B词条池 低价值随机词条池(非攻击类技能) 数字表示概率占比
    b_pool: Mapping[ResID, float]

    def meta(self) -> dict[str, Any]:
        return {"cache": True}

    @classmethod
    def new(
        cls,
        res_id: ResID,
        rare: RareLevel,
        pattern: str,
        max_level: int,
        a_pool: Mapping[ResID, float],
        b_pool: Mapping[ResID, float],
    ):
        return AccessoryPattern(
            res_id,
            rare=rare,
            pattern=pattern,
            max_level=max_level,
            a_pool=a_pool,
            b_pool=b_pool,
        )

    def serialize(self) -> dict[str, Any]:
        ser = {
            **super().serialize(),
            "rare": _serialize_rare_level(self.rare, self.h("rare")),
            "max_level": _serialize_int(self.max_level, self.h("max_level"), min=1),
            "pattern": self._ser_pattern(self.pattern),
            "a_pool": _serialize_res_ids_float(self.a_pool, Entry, self.h("a_pool"), min=0),
            "b_pool": _serialize_res_ids_float(self.b_pool, Entry, self.h("b_pool"), min=0),
        }
        return cleanup(ser)

    def _ser_pattern(self, pattern: str):
        res = []
        patterns = pattern.split(" ")
        expected_level = len(patterns) * MAX_ENTRY_PLUS
        if self.max_level != expected_level:
            raise self.error("level", f"must = {expected_level}")

        for idx, item in enumerate(patterns):
            if idx == 0:
                if item != "S":
                    raise self.error("pattern", "must be a pattern like 'S A AB'")
            elif item in ("A", "B", "AB"):
                res.append(item)
            else:
                raise self.error("pattern", "must be a pattern like 'S A AB'")
        return res


@dataclass(kw_only=True)
class Accessory(Resource):
    """
    装饰品，一类具有随机词条的物品，随机词条取决于AccessoryPattern。
    """

    # 随机词条的模式
    pattern: ResID

    # 稀有度等级
    rare: RareLevel

    # 对应词条
    entry: ResID

    # 词条的叠加数
    piece: int

    # 变体类型 用于区分同名宝石
    variant: VariantType = Variant1

    def serialize(self) -> dict[str, Any]:
        pattern = AccessoryPattern.get(self.pattern, self.h("pattern"))
        entry = Entry.get(self.entry, self.h("entry"))
        if pattern.rare != self.rare:
            raise self.error(self.h("pattern"), "AccessoryPattern and Accessory rare mismatch")

        return {
            **super().serialize(),
            "pattern": pattern.id,
            "rare": _serialize_rare_level(self.rare, self.h("rare")),
            "entry": entry.id,
            "piece": _serialize_int(self.piece, self.h("piece"), min=1, max=entry.max_piece),
            "variant": _serialize_variant_type(self.variant, self.h("variant")),
        }

    @classmethod
    def new(
        cls,
        pattern_id: ResID,
        entry_id: ResID,
        piece: int,
        variant: VariantType = Variant1,
    ):
        if Entry.find(entry_id) is None:
            raise Exception(f"Entry '{entry_id}' not found")
        id = entry_id.replace("Entry.", "Accessory.") + f".{variant}"

        pattern = AccessoryPattern.find(pattern_id)
        if pattern is None:
            raise Exception(f"AccessoryPattern '{entry_id}' not found")

        return Accessory(
            id,
            pattern=pattern_id,
            rare=pattern.rare,
            entry=entry_id,
            piece=piece,
        )
