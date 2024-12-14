from __future__ import annotations
from dataclasses import dataclass
from typing import Any

from turning_point.base.types import Serializer
from turning_point.base.builtin import _serialize_float

@dataclass()
class Box(Serializer):
    half_x: float
    half_y: float
    half_z: float

    def serialize(self, where: str) -> dict[str, Any]:
        return {
            "half_x": _serialize_float(self.half_x, f"{where}.half_x", min=0),
            "half_y": _serialize_float(self.half_y, f"{where}.half_y", min=0),
            "half_z": _serialize_float(self.half_z, f"{where}.half_z", min=0),
        }


@dataclass()
class Sphere(Serializer):
    radius: float

    def serialize(self, where: str) -> dict[str, Any]:
        return {
            "radius": _serialize_float(self.radius, f"{where}.radius", min=0),
        }


@dataclass()
class Capsule(Serializer):
    half_height: float
    radius: float

    def serialize(self, where: str) -> dict[str, Any]:
        return {
            "half_height": _serialize_float(self.half_height, f"{where}.half_height", min=0),
            "radius": _serialize_float(self.radius, f"{where}.radius", min=0),
        }
