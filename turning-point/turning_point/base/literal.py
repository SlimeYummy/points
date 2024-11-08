from turning_point.config import FPS


Plus = "+"


def percent(*ns: float):
    if len(ns) == 1:
        return ns[0] / 100.0
    return [n / 100.0 for n in ns]


def millisecond(n: float) -> int:
    return round(FPS * n / 1000.0)


def second(n: float) -> int:
    return round(FPS * n)


def minute(n: float) -> int:
    return round(FPS * n * 60)


def hour(n: float) -> int:
    return round(FPS * n * 60 * 24)
