from __future__ import annotations
import re
from typing import Literal, Sequence, get_args, cast

SlotType = Literal["Attack", "Defense", "Special"]

Attack = "Attack"
Defense = "Defense"
Special = "Special"

_SlotType = get_args(SlotType)

AttackIndex = 0
DefenseIndex = 1
SpecialIndex = 2


def _serialize_slot_type(val: SlotType | str | None, where: str = "?", optional: bool = False):
    if optional and val is None:
        return None
    if val not in _SlotType:
        raise Exception(f"{where}: must be a SlotType")
    return val


_RE_SOLTS = re.compile(r"^([A|D|S][0-9]\d*)?([A|D|S][0-9]\d*)?([A|D|S][0-9]\d*)?$")


def _serialize_slot_definition(
    slot: str | Sequence[int] | None,
    where: str = "?",
    optional: bool = False,
):
    if optional and slot is None:
        return None

    if isinstance(slot, str):
        capture = _RE_SOLTS.match(slot)
        if capture:
            attack = 0
            defense = 0
            special = 0
            for group in capture.groups():
                if group is not None:
                    match group[0]:
                        case "A":
                            attack = int(group[1:])
                        case "D":
                            defense = int(group[1:])
                        case "S":
                            special = int(group[1:])
            return (special, attack, defense)
    elif isinstance(slot, Sequence):
        if len(slot) != 3:
            raise Exception(f"{where}: len() must == 3")
        return cast(tuple[int, int, int], (slot[0], slot[1], slot[2]))
    raise Exception(f"{where}: must be an A_D_S_/Sequence[3]")


def _serialize_slot_definitions(
    slots: Sequence[str | Sequence[int]] | None,
    size: int,
    where: str = "?",
    optional: bool = False,
    zero: Sequence[int] | None = None,
) -> Sequence[Sequence[int]] | None:
    if optional and slots is None:
        return None
    if not isinstance(slots, Sequence):
        raise Exception(f"{where}: must be a Sequence")
    if len(slots) != size:
        raise Exception(f"{where}: len() must = {size}")

    ser = []
    if isinstance(zero, tuple | list):
        ser.append(zero)
    for slot in slots:
        ser.append(_serialize_slot_definition(slot, where, optional))
    return ser
