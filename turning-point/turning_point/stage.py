from __future__ import annotations
from dataclasses import dataclass
from typing import Any

from turning_point.base import Resource, _serialize_file, _serialize_str, cleanup


@dataclass(kw_only=True)
class Stage(Resource):
    """
    舞台 游戏中的战斗场景
    """

    # 角色名字
    name: str

    # Stage文件路径（逻辑）
    stage_file: str

    # Stage文件路径（渲染）
    view_stage_file: str

    def serialize(self) -> dict[str, Any]:
        ser = {
            **super().serialize(),
            "name": _serialize_str(self.name, self.h("name")),
            "stage_file": _serialize_file(self.stage_file, self.h("stage_file"), ext=".json"),
            "view_stage_file": _serialize_file(self.view_stage_file, self.h("view_stage_file"), ext=".tscn"),
        }
        return cleanup(ser)
