from __future__ import annotations
from turning_point.base import *


class Buff(Resource):
    """
    Buff 游戏运行时的被动技能
    """

    # 参数列表 Mapping[变量名, 默认值]
    # 对Buff使用者 是可传入的变量列表
    # 对Buff脚本 是额外可用的变量列表
    arguments: Mapping[str, float]

    # Buff启动时执行的脚本
    on_start: Script | None = None

    # Buff结束时执行的脚本
    on_finish: Script | None = None

    # 每次攻击命中时执行的脚本
    on_hit: Script | None = None

    # 每次被击中时时执行的脚本
    on_hurt: Script | None = None

    # 每次游戏tick时执行的脚本
    on_tick: Script | None = None

    # 展示用的名字
    name: str

    # 图标
    icon: str

    @clean()
    def serialize(self) -> dict[str, Any]:
        return {
            **super().serialize(),
            "arguments": self._serialize_script_args(),
            "on_start": ser_script(self.on_start, optional=True, where=self.h("on_start")),
            "on_finish": ser_script(self.on_finish, optional=True, where=self.h("on_finish")),
            "on_hit": ser_script(self.on_hit, optional=True, where=self.h("on_hit")),
            "on_hurt": ser_script(self.on_hurt, optional=True, where=self.h("on_hurt")),
            "on_tick": ser_script(self.on_tick, optional=True, where=self.h("on_tick")),
            "name": _serialize_str(self.name, self.h("name")),
            "icon": _serialize_str(self.icon, self.h("icon")),
        }

    def _serialize_script_args(self):
        if not isinstance(self.arguments, Mapping):
            Exception("%s: must be a Mapping" % self.h("arguments"))

        for arg, value in self.arguments.items():
            if type(arg) != str:
                Exception("%s: must be a str" % self.h("arguments.(key)"))

            if type(value) not in (int, float):
                Exception("%s: must be an int|float" % self.h("arguments.(value)"))

        return self.arguments


# Buff列表 [{Buff参数: 参数值, ...}, ...]
BuffsList = Sequence[Mapping[str, float]]


def _ser_buffs_list(list, size: int, buff: Buff, where: str = "?", optional: bool = False):
    if optional and list is None:
        return None

    if not isinstance(list, Sequence):
        raise Exception("%s: must be an BuffsList" % where)

    if len(list) != size:
        raise Exception("%s : size must equal to %d" % (where, size))

    for args in list:
        if not isinstance(args, Mapping):
            raise Exception("%s.(item): must be a Mapping" % where)

        for arg, value in args.items():
            if arg not in buff.arguments:
                raise Exception("%s.(item).(key): must be an argument in %s" % (where, buff.id))

            if type(value) not in (int, float):
                raise Exception("%s.(item).(value): must be an int|float")

    return list
