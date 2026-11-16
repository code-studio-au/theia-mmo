"""Desktop work-area hints for Bevy's independent tiled playtest windows.

Bevy 0.20 exposes monitor bounds but not the desktop's reserved panel area.
Only query that area here; Bevy still creates, sizes and positions every window.
"""
import os
import re
import subprocess
import sys


def parse_work_area(listing):
    for line in listing.splitlines():
        match = re.match(r'^\d+\s+\*\s+.*\bWA:\s+(-?\d+),(-?\d+)\s+(\d+)x(\d+)\b', line)
        if match:
            area = tuple(map(int, match.groups()))
            if (abs(area[0]) <= 1_000_000 and abs(area[1]) <= 1_000_000
                    and all(1 <= n <= 32768 for n in area[2:])):
                return area
    return None


def desktop_work_area():
    if sys.platform == 'win32':
        import ctypes
        from ctypes import wintypes
        user32 = ctypes.windll.user32
        # Query physical coordinates without changing the launcher's DPI mode.
        user32.SetThreadDpiAwarenessContext.argtypes = [ctypes.c_void_p]
        user32.SetThreadDpiAwarenessContext.restype = ctypes.c_void_p
        previous = user32.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))
        try:
            area = wintypes.RECT()
            if user32.SystemParametersInfoW(0x0030, 0, ctypes.byref(area), 0):
                return (area.left, area.top, area.right-area.left, area.bottom-area.top)
        finally:
            if previous:
                user32.SetThreadDpiAwarenessContext(previous)
    elif sys.platform.startswith('linux') and os.environ.get('DISPLAY'):
        try:
            result = subprocess.run(['wmctrl', '-d'], capture_output=True, text=True, timeout=2)
            if result.returncode == 0:
                return parse_work_area(result.stdout)
        except (OSError, subprocess.TimeoutExpired):
            pass
    return None


def work_area_options():
    area = desktop_work_area()
    return ['--test-work-area', ','.join(map(str, area))] if area else []
