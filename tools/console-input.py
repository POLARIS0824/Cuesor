"""Benchmark-only: queue a fixed command in a newly created test Nu console.

Not used by terminal-ime. Never point this at a normal user session.
"""
import argparse
import ctypes as C
from ctypes import wintypes as W
from pathlib import Path


class InputRecord(C.Structure):
    _fields_ = [
        ("kind", W.WORD), ("padding", W.WORD), ("down", W.BOOL),
        ("repeat", W.WORD), ("key", W.WORD), ("scan", W.WORD),
        ("character", W.WCHAR), ("modifiers", W.DWORD),
    ]


def send(pid: int, command: str) -> None:
    kernel = C.WinDLL("kernel32", use_last_error=True)
    user = C.WinDLL("user32", use_last_error=True)
    kernel.AttachConsole.argtypes = [W.DWORD]
    kernel.AttachConsole.restype = W.BOOL
    kernel.CreateFileW.argtypes = [W.LPCWSTR, W.DWORD, W.DWORD, W.LPVOID, W.DWORD, W.DWORD, W.HANDLE]
    kernel.CreateFileW.restype = W.HANDLE
    kernel.WriteConsoleInputW.argtypes = [W.HANDLE, C.POINTER(InputRecord), W.DWORD, C.POINTER(W.DWORD)]
    kernel.WriteConsoleInputW.restype = W.BOOL
    kernel.CloseHandle.argtypes = [W.HANDLE]
    user.VkKeyScanW.argtypes = [W.WCHAR]
    user.VkKeyScanW.restype = C.c_short
    kernel.FreeConsole()
    if not kernel.AttachConsole(pid):
        raise C.WinError(C.get_last_error())
    handle = None
    try:
        handle = kernel.CreateFileW("CONIN$", 0xC0000000, 3, None, 3, 0, None)
        if handle == W.HANDLE(-1).value:
            raise C.WinError(C.get_last_error())
        records = (InputRecord * (len(command) * 2))()
        for index, char in enumerate(command):
            key = user.VkKeyScanW(char)
            for offset, down in enumerate((True, False)):
                records[index * 2 + offset] = InputRecord(1, 0, down, 1, key & 255, 0, char, 0x10 if key & 0x100 else 0)
        written = W.DWORD()
        if not kernel.WriteConsoleInputW(handle, records, len(records), C.byref(written)):
            raise C.WinError(C.get_last_error())
        if written.value != len(records):
            raise RuntimeError("Incomplete benchmark command injection")
    finally:
        if handle and handle != W.HANDLE(-1).value:
            kernel.CloseHandle(handle)
        kernel.FreeConsole()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", type=int, required=True)
    parser.add_argument("--command-file", type=Path, required=True)
    args = parser.parse_args()
    assert C.sizeof(InputRecord) == 20
    send(args.pid, args.command_file.read_text(encoding="utf-8-sig") + "\r")
