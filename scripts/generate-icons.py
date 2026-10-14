#!/usr/bin/env python3
"""Reproducible hourglass icons using only Python's standard library.

The geometry is the same as assets/hourglass.svg. No image tooling is needed
to build the app: generated PNG/ICO/ICNS/RGBA files are checked in.
"""
import math
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "src-tauri" / "icons"
SEGMENTS = [
    (6, 4, 18, 4), (6, 20, 18, 20),
    (7, 5, 7, 8), (7, 8, 11, 12), (11, 12, 7, 16), (7, 16, 7, 19),
    (17, 5, 17, 8), (17, 8, 13, 12), (13, 12, 17, 16), (17, 16, 17, 19),
]


def segment_distance(x, y, segment):
    x1, y1, x2, y2 = segment
    dx, dy = x2 - x1, y2 - y1
    t = max(0, min(1, ((x - x1) * dx + (y - y1) * dy) / (dx * dx + dy * dy)))
    return math.hypot(x - x1 - t * dx, y - y1 - t * dy)


def in_sand(x, y, progress=None, phase=0):
    if progress is not None:
        top = 11.2 - 3.7 * math.sqrt(1 - progress)
        bottom_peak = 18 - 3.6 * math.sqrt(progress)
        reservoirs = (top <= y <= 11.2 and abs(x - 12) <= (11.2 - y) * .9) or \
                     (bottom_peak <= y <= 18 and abs(x - 12) <= (y - bottom_peak) * .9)
        falling = False
        if progress < 1:
            drop = 12.6 + phase * .9
            falling = any(gy >= 12.4 and math.hypot(x - 12, y - gy) <= .4
                          for gy in (drop, drop - 2.1))
        return reservoirs or falling
    return (7.5 <= y <= 10.8 and abs(x - 12) <= (10.8 - y)) or \
           (14.4 <= y <= 18 and abs(x - 12) <= (y - 14.4))


def render(size, app=False, windows=False, progress=None, phase=0):
    pixels = bytearray()
    samples = 3
    for py in range(size):
        for px in range(size):
            alpha = 0
            shade = 0
            for sy in range(samples):
                for sx in range(samples):
                    x = (px + (sx + .5) / samples) * 24 / size
                    y = (py + (sy + .5) / samples) * 24 / size
                    if app:
                        # Neutral rounded tile for application/install icons.
                        x = (x - 12) / .72 + 12
                        y = (y - 12) / .72 + 12
                    distance = min(segment_distance(x, y, s) for s in SEGMENTS)
                    ink = distance <= .8 or in_sand(x, y, progress, phase)
                    halo = windows and distance <= 1.45
                    if windows and progress is not None and not ink:
                        halo = halo or any(in_sand(x + dx, y + dy, progress, phase)
                                           for dx, dy in ((.45, 0), (-.45, 0), (0, .45), (0, -.45)))
                    if app:
                        alpha += 255
                        shade += 35 if ink else 245
                    elif ink or halo:
                        alpha += 255
                        shade += 0 if ink else 255
            count = samples * samples
            a = round(alpha / count)
            c = round(shade * 255 / alpha) if alpha else 0
            pixels.extend((c, c, c, a))
    return bytes(pixels)


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)


def png(size, pixels):
    rows = b"".join(b"\0" + pixels[y * size * 4:(y + 1) * size * 4] for y in range(size))
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)) + \
        chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b"")


def main():
    OUTPUT.mkdir(parents=True, exist_ok=True)
    normal = render(32)
    windows = render(32, windows=True)
    (OUTPUT / "tray.rgba").write_bytes(normal)
    (OUTPUT / "tray-windows.rgba").write_bytes(windows)
    (OUTPUT / "tray.png").write_bytes(png(32, normal))
    (OUTPUT / "tray-windows.png").write_bytes(png(32, windows))
    # Eight fill levels, four falling-grain phases. Raw frames need no runtime decoder.
    for name, is_windows in (("tray-animation.rgba", False), ("tray-animation-windows.rgba", True)):
        frames = [render(32, windows=is_windows, progress=stage / 7, phase=phase)
                  for stage in range(8) for phase in range(4)]
        (OUTPUT / name).write_bytes(b"".join(frames))
    images = {size: png(size, render(size, app=True)) for size in (32, 128, 256)}
    for name, size in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256)):
        (OUTPUT / name).write_bytes(images[size])
    entries = []
    offset = 6 + 16 * len(images)
    for size, data in images.items():
        dimension = 0 if size == 256 else size
        entries.append(struct.pack("<BBBBHHII", dimension, dimension, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    (OUTPUT / "icon.ico").write_bytes(struct.pack("<HHH", 0, 1, len(images)) + b"".join(entries) + b"".join(images.values()))
    blocks = []
    for kind, size in ((b"icp5", 32), (b"ic07", 128), (b"ic08", 256)):
        blocks.append(kind + struct.pack(">I", len(images[size]) + 8) + images[size])
    body = b"".join(blocks)
    (OUTPUT / "icon.icns").write_bytes(b"icns" + struct.pack(">I", len(body) + 8) + body)
    print("Generated menu bar, system tray, and bundle icons.")


if __name__ == "__main__":
    main()
