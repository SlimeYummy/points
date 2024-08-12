import json
import os
from io import BufferedRandom
from typing import Any


class FileWriter:
    _h_data: BufferedRandom
    _h_index: BufferedRandom
    _index: dict[str, tuple[int, int, int]]
    _close: bool

    def __init__(self, path: str) -> None:
        self._index = {}
        self._close = False

        if not os.path.exists(path):
            os.mkdir(path)

        p_data = os.path.join(path, "data.json")
        self._h_data = open(p_data, "wb+")
        self._h_data.truncate()
        self._h_data.write(str.encode("["))

        p_index = os.path.join(path, "index.json")
        self._h_index = open(p_index, "wb+")
        self._h_index.truncate()

    def __del__(self):
        self._index = {}
        self._close = True

        if self._h_data:
            self._h_data.close()

        if self._h_index:
            self._h_index.close()

    def __enter__(self):
        if self._close:
            raise Exception("Already closed")
        return self

    def __exit__(self, type: Any, value: Any, trace: Any):
        if not self._close:
            self.close(type is not None)
            self._close = True

    def write(self, id: str, data: str, cache: int):
        if self._close:
            raise Exception("Already closed")

        if id in self._index:
            raise Exception("ID conflict: %s" % id)

        if self._h_data.tell() <= 2:
            # first element
            self._h_data.write(str.encode("\n"))
        else:
            # not first element
            self._h_data.write(str.encode(",\n"))

        start = self._h_data.tell()
        self._h_data.write(str.encode(data))
        finish = self._h_data.tell()
        self._index[id] = (start, finish - start, cache)

    def close(self, clear: bool = False):
        if self._close:
            raise Exception("Already closed")
        self._close = True

        if self._h_data:
            if clear:
                self._h_data.truncate()
            self._h_data.write(str.encode("\n]"))
            self._h_data.flush()

        if self._h_index:
            if clear:
                self._h_index.truncate()
            else:
                data = json.dumps(self._index, separators=(",", ":"))
                self._h_index.write(str.encode(data))
                self._h_index.flush()

        self._index = {}
