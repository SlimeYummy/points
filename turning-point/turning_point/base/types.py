from __future__ import annotations
from dataclasses import dataclass
import json
import re
from typing import Any, Callable, Literal, Mapping, Sequence, Type, TypeVar, cast, get_args

from turning_point.base.builtin import (
    _serialize_float,
)
from turning_point.config import MAX_SYMBOL_LEN
from turning_point.writer import FileWriter


ResID = str


RE_RES_ID = re.compile(r"^[\w\d_\-]+(:?\.[\w\d_\-]+)*$")
RE_RES_ID_ACTION = re.compile(r"^([\w\d_\-]+(:?\.[\w\d_\-]+)*)\#([\w\d_\-]+)$")


def _serialize_res_id(
    val: str | None,
    T: str | Type[Resource],
    where: str = "?",
    optional: bool = False,
    reference: Callable[[Any], bool] | None = None,
):
    if optional and val is None:
        return None

    if not isinstance(val, str):
        raise Exception(f"{where}: must be a ResID")
    if len(val) > MAX_SYMBOL_LEN:
        raise Exception(f"{where}: len() must <= {MAX_SYMBOL_LEN}")
    if RE_RES_ID.match(val) is None:
        raise Exception(f"{where}: must match ResID pattern")

    prefix = (T if isinstance(T, str) else T.T()) + "."
    if not val.startswith(prefix):
        raise Exception(f'{where}: must start with "{prefix}"')

    res = _res_dict.get(val)
    if not res:
        raise Exception(f"{where}: Resource not found")
    if reference and not reference(res):
        raise Exception(f'{where}: Resource not reference by "{res.id}"')
    return val


def _serialize_res_ids(
    val: Sequence[ResID] | None,
    T: str | Type[Resource],
    where: str = "?",
    optional: bool = False,
    min_len: int | None = None,
    max_len: int | None = None,
    reference: Callable[[Any], bool] | None = None,
    zero: ResID | None = None,
):
    if optional and val is None:
        return None

    if not isinstance(val, Sequence):
        raise Exception(f"{where}: must be a Sequence")
    if min_len is not None and len(val) < min_len:
        raise Exception(f"{where}: len() must > {min_len}")
    if max_len is not None and max_len < len(val):
        raise Exception(f"{where}: len() must < {max_len}")

    ser: list[ResID] = []
    if isinstance(zero, int | float):
        ser.append(zero)

    ids_set: set[str] = set()
    for idx, id in enumerate(val):
        if id in ids_set:
            raise Exception(f"{where}[{idx}]: ResID conflict")
        ser_id = _serialize_res_id(id, T, f"{where}[{idx}]", reference=reference)
        ser.append(ser_id)
        ids_set.add(ser_id)
    return ser


def _serialize_res_ids_float(
    val: Mapping[ResID, float] | None,
    T: str | Type[Resource],
    where: str = "?",
    optional: bool = False,
    reference: Callable[[Any], bool] | None = None,
    min: int | None = None,
    max: int | None = None,
):
    if optional and val is None:
        return None
    if not isinstance(val, Mapping):
        raise Exception(f"{where}: must be a Mapping")

    ser = {}
    for id, vals in val.items():
        ser_id = _serialize_res_id(id, T, f"{where}[{id}]", reference=reference)
        ser_vals = _serialize_float(vals, f"{where}[{id}]", min=min, max=max)
        ser[ser_id] = ser_vals
    return ser


RareLevel = Literal["Rare1", "Rare2", "Rare3"]

Rare1: RareLevel = "Rare1"
Rare2: RareLevel = "Rare2"
Rare3: RareLevel = "Rare3"

_RareLevel = get_args(RareLevel)


def _serialize_rare_level(val: RareLevel | None, where: str = "?", optional: bool = False) -> RareLevel | None:
    if optional and val is None:
        return None
    if val not in _RareLevel:
        raise Exception(f"{where}: must be a EntryType")
    return val


VariantType = Literal["Variant1", "Variant2", "Variant3", "VariantX"]

Variant1: VariantType = "Variant1"
Variant2: VariantType = "Variant2"
Variant3: VariantType = "Variant3"
VariantX: VariantType = "VariantX"

_VariantType = get_args(VariantType)


def _serialize_variant_type(val: VariantType | None, where: str = "?", optional: bool = False) -> VariantType | None:
    if optional and val is None:
        return None
    if val not in _VariantType:
        raise Exception(f"{where}: must be a EntryType")
    return val


class Serializer:
    _T: str = ""

    @classmethod
    def T(cls) -> str:
        return cls.__name__

    def serialize(self) -> dict[str, Any]:
        T = type(self)
        return {"T": T.__name__}


_R = TypeVar("_R", bound="Resource")


_res_dict: dict[str, Resource] = {}


@dataclass
class Resource(Serializer):
    # 资源ID 形如<Resource>.* <Resource>和资源class同名
    id: ResID

    def __post_init__(self):
        if not isinstance(self.id, str):
            raise Exception(f"<{self.id}>.id: must be a str")
        if len(self.id) > 128:
            raise Exception(f"<{self.id}>.id: len() must <= {MAX_SYMBOL_LEN}")

        prefix = type(self).T() + "."
        if not self.id.startswith(prefix):
            raise Exception(f'<{self.id}>.id: must start with "{prefix}"')

        if self.id in _res_dict:
            raise Exception(f"<{self.id}>.id: id can not repeat")
        _res_dict[self.id] = self

    def meta(self) -> dict[str, Any]:
        return {"cache": False}

    def h(self, field: str) -> str:
        return f"<{self.id}>.{field}"

    def here(self, field: str) -> str:
        return f"<{self.id}>.{field}"

    def e(self, field: str, message: str):
        return Exception(f"<{self.id}>.{field}: {message}")

    def error(self, field: str, message: str):
        return Exception(f"<{self.id}>.{field}: {message}")

    def serialize(self) -> dict[str, Any]:
        T = type(self)
        return {
            "T": T.__name__,
            "id": _serialize_res_id(self.id, T, T.T() + ".id"),
        }

    @classmethod
    def write_all(cls, path: str):
        with FileWriter(path) as writer:
            for res in _res_dict.values():
                data = json.dumps(res.serialize(), separators=(",", ":"))
                meta = res.meta()
                writer.write(res.id, data, meta.get("cache") and 1 or 0)

    @classmethod
    def get(cls: Type[_R], id: ResID, where: str = "?") -> _R:
        return cast(_R, Resource.get_by(cls.T(), id, where))

    @classmethod
    def get_by(cls: Type[Resource], name: str, id: ResID, where: str = "?") -> Resource:
        if not isinstance(id, str):
            raise Exception(f"{where}: StrID must be a string")

        if len(id) > MAX_SYMBOL_LEN:
            raise Exception(f"{where}: len() must < {MAX_SYMBOL_LEN}")

        prefix = name + "."
        if not id.startswith(prefix):
            raise Exception(f'{where}: must start with "{prefix}"')

        res = _res_dict.get(id)
        if not res:
            raise Exception(f"{where}: Resource not found")
        return res

    @classmethod
    def find(cls: Type[_R], id: ResID) -> _R | None:
        if not isinstance(id, str):
            return None

        res = _res_dict.get(id)
        if not res or not isinstance(res, cls):
            return None
        return res

    @classmethod
    def find_all(cls: Type[Resource], filter: Callable[[Any], bool]) -> list[Resource]:
        return [res for res in _res_dict.values() if isinstance(res, cls) and filter(res)]
