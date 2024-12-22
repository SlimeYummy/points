from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Sequence, Literal, Mapping, cast, get_args

from turning_point.base import (
    _serialize_bool,
    _serialize_file,
    _serialize_int,
    _serialize_time,
    _serialize_float,
    _serialize_str,
    _serialize_list_int,
    _serialize_range_time,
    _serialize_res_id,
    _serialize_inline_arguments,
    _serialize_inline_switch,
    _serialize_inline_time,
    _serialize_inline_float,
    Inline,
    Resource,
    cleanup,
    ResID,
    RE_RES_ID_ACTION,
)
from turning_point.script import _serialize_script, _extract_script_args, _serialize_script_args


@dataclass
class Animation:
    # 动画对应的ozz文件路径
    file: str

    # 动画时长（单位帧）
    # 当动画文件内时常与duration不一致时 会将时长缩放为duration
    duration: int | str

    # 循环播放次数
    times: int = 1

    # 是否为叠加模式
    additive: bool | int = False

    # 肢体动画进度匹配
    # 在动画切换时 参考当前动画进度 自动从下一动画的对应进度开始播放 肢体使过渡更平滑
    # None => 不启用
    # int => 从当前动画的偏移处开始匹配（单位帧）
    body_progress: int | str | None = None

    def serialize(self, where: str = "?", inf: bool | None = None, additive: bool | None = False) -> dict[str, Any]:
        if inf is not None:
            if inf and self.times != 0:
                raise Exception(f"{where}: times must == infinate")
            elif not inf and self.times == 0:
                raise Exception(f"{where}: times must != infinate")
        if additive is not None and (bool(self.additive) != additive):
            raise Exception(f"{where}: additive must = {additive}")

        duration = _serialize_time(self.duration, f"{where}.duration", min=1)
        return {
            "file": _serialize_file(self.file, f"{where}.file", ext=".ozz"),
            "duration": duration,
            "times": _serialize_int(self.times, f"{where}.times", min=0),
            "additive": _serialize_bool(self.additive, f"{where}.additive"),
            "body_progress": _serialize_time(
                self.body_progress, f"{where}.body_progress", optional=True, min=0, max=duration
            ),
        }

    def _serialize_fade(self, duration: int, where: str = "?"):
        if isinstance(self.fade, int | str):
            tm = _serialize_time(self.fade, where, min=0, max=duration)
            return [tm, tm]
        else:
            return _serialize_range_time(self.fade, where, min=0, max=duration)


def _serialize_animation(
    animations: Animation | None,
    where: str = "?",
    optional: bool = False,
    loop: bool | None = None,
    additive: bool | None = False,
):
    if optional and animations is None:
        return None
    if not isinstance(animations, Animation):
        raise Exception(f"{where}: must be an Animation")
    return animations.serialize(where, loop, additive)


def _serialize_animations(
    animations: Sequence[Animation] | None,
    size: int | None = None,
    where: str = "?",
    optional: bool = False,
    loop: bool | None = None,
    additive: bool | None = False,
) -> list[dict[str, Any]] | None:
    if optional and animations is None:
        return None

    if not isinstance(animations, Sequence):
        raise Exception(f"{where}: must be a Sequence")
    if size is not None and len(animations) != size:
        raise Exception(f"{where}: len() must = {size}")

    res = []
    for idx, anime in enumerate(animations):
        res.append(anime.serialize(f"{where}[{idx}]", loop, additive))
    return res


EnterKey = Literal[
    "Run",
    "Dash",
    "Walk",
    "View",
    "Dodge",
    "Jump",
    "Guard",
    "Interact",
    "Lock",
    "LockSwitch",

    "Attack1",
    "Attack2",
    "Attack3",

    "Shot1",
    "Shot2",
    "Aim",
    "Reload",

    "Extra1",
    "Extra2",
    "Extra3",

    "Skill1",
    "Skill2",
    "Skill3",
    "Skill4",

    "Item1",
    "Item2",
    "Item3",
    "Item4",
    "Item5",
    "Item6",
    "Item7",
    "Item8",
]

Run = "Run"
Dash = "Dash"
Walk = "Walk"
View = "View"
Dodge = "Dodge"
Jump = "Jump"
Guard = "Guard"
Interact = "Interact"
Lock = "Lock"
LockSwitch = "LockSwitch"

Attack1 = "Attack1"
Attack2 = "Attack2"
Attack3 = "Attack3"

Shot1 = "Shot1"
Shot2 = "Shot2"
Aim = "Aim"
Reload = "Reload"

Extra1 = "Extra1"
Extra2 = "Extra2"
Extra3 = "Extra3"

Skill1 = "Skill1"
Skill2 = "Skill2"
Skill3 = "Skill3"
Skill4 = "Skill4"


DeriveKey = Literal[
    "DeriveMove",

    "DeriveLight",
    "DeriveHeavy",
    "DeriveMiddle",

    "DeriveShot",
    "DeriveAim",
    "DeriveExtra",
]

DeriveMove = "DeriveMove"

DeriveLight = "DeriveLight"
DeriveHeavy = "DeriveHeavy"
DeriveMiddle = "DeriveMiddle"

DeriveShot = "DeriveShot"
DeriveAim = "DeriveAim"
DeriveExtra = "DeriveExtra"


_KeysHelper = {}
# for key in get_args(VirtualKey):
#     _KeysHelper[key] = key
for key in get_args(EnterKey):
    _KeysHelper[key] = EnterKey
for key in get_args(DeriveKey):
    _KeysHelper[key] = DeriveKey


def _serialize_key(
    types: Sequence[Any],
    key: str | None,
    where: str = "?",
    optional: bool = False,
):
    if optional and key is None:
        return None

    if (_KeysHelper.get(key) not in types) and (key not in types):
        raise Exception(f"{where}: key not support")
    return key


PseudoAction = Literal[
    "PseudoDash",
    "PseudoGuard",
    "PseudoGuardEx",
    "PseudoDodge",
    "PseudoDodgeEx",
]

PseudoDash = "PseudoDash"
PseudoGuard = "PseudoGuard"
PseudoGuardEx = "PseudoGuardEx"
PseudoDodge = "PseudoDodge"
PseudoDodgeEx = "PseudoDodgeEx"

_PseudoAction = get_args(PseudoAction)

