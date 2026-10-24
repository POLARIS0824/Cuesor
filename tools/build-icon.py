"""Regenerate the small code-drawn cursor icon; not required for building or running."""
from pathlib import Path
import struct

size = 32
pixels = bytearray()
for y in reversed(range(size)):
    for x in range(size):
        cursor = (13 <= x <= 18 and 7 <= y <= 24) or (6 <= y <= 8 and 8 <= x <= 23) or (23 <= y <= 25 and 8 <= x <= 23)
        color = (0xF76B15 if y < 16 else 0x0090FF) if cursor else 0x111113
        pixels.extend((color & 255, color >> 8 & 255, color >> 16, 255))
bitmap = struct.pack('<IIIHHIIIIII', 40, size, size * 2, 1, 32, 0, len(pixels), 0, 0, 0, 0) + pixels + bytes(size * size // 8)
icon = struct.pack('<HHH', 0, 1, 1) + struct.pack('<BBBBHHII', size, size, 0, 0, 1, 32, len(bitmap), 22) + bitmap
Path(__file__).resolve().parent.parent.joinpath('desktop/assets/app.ico').write_bytes(icon)
