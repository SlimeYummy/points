import critical_point_pyext
from typing import Any


def compile_script(code: str, args: list[str]) -> Any:
    return critical_point_pyext.compile_script(code, args)


def read_skeleton_meta(path: str, with_joints: bool = False) -> dict[str, Any]:
    return critical_point_pyext.read_skeleton_meta(path, with_joints)


def read_animation_meta(path: str) -> dict[str, Any]:
    return critical_point_pyext.read_animation_meta(path)
