#!/usr/bin/env python3
"""生成占位精灵表 - 纯 Python，无 PIL 依赖"""
import struct
import zlib
import os

def create_png(width, height, pixels):
    """创建 PNG 文件（RGBA）"""
    def chunk(chunk_type, data):
        c = chunk_type + data
        crc = struct.pack(">I", zlib.crc32(c) & 0xffffffff)
        return struct.pack(">I", len(data)) + c + crc

    header = b"\x89PNG\r\n\x1a\n"
    ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))

    raw = b""
    for y in range(height):
        raw += b"\x00"  # filter: None
        for x in range(width):
            raw += bytes(pixels[y * width + x])

    idat = chunk(b"IDAT", zlib.compress(raw))
    iend = chunk(b"IEND", b"")
    return header + ihdr + idat + iend

W, H = 32, 32
COLS, ROWS = 4, 8
TOTAL_W, TOTAL_H = W * COLS, H * ROWS

# 初始化品红背景（透明色）
pixels = [(255, 0, 255, 255)] * (TOTAL_W * TOTAL_H)

def put_pixel(x, y, color):
    if 0 <= x < TOTAL_W and 0 <= y < TOTAL_H:
        pixels[y * TOTAL_W + x] = color

# 8行状态：Idle, WalkRight, WalkLeft, Sleep, Eat, Happy, Startled, Sit
colors = [
    (100, 200, 100, 255),  # Idle
    (100, 200, 100, 255),  # Walk Right
    (100, 200, 100, 255),  # Walk Left
    (100, 100, 200, 255),  # Sleep
    (200, 150, 100, 255),  # Eat
    (200, 100, 100, 255),  # Happy
    (200, 200, 100, 255),  # Startled
    (100, 200, 200, 255),  # Sit
]

for row, color in enumerate(colors):
    for col in range(COLS):
        x0, y0 = col * W, row * H
        # 头
        for px in range(10, 22):
            for py in range(6, 16):
                put_pixel(x0 + px, y0 + py, color)
        # 身体
        for px in range(8, 24):
            for py in range(16, 26):
                put_pixel(x0 + px, y0 + py, color)
        # 眼睛
        put_pixel(x0 + 13, y0 + 10, (30, 30, 30, 255))
        put_pixel(x0 + 18, y0 + 10, (30, 30, 30, 255))
        
        # Walk Right 帧加小脚偏移
        if row == 1:
            offset = (col % 2) * 2
            for px in range(10, 14):
                put_pixel(x0 + px + offset, y0 + 26, color)
            for px in range(18, 22):
                put_pixel(x0 + px - offset, y0 + 26, color)
        
        # Walk Left 帧加小脚偏移（镜像）
        if row == 2:
            offset = (col % 2) * 2
            for px in range(10, 14):
                put_pixel(x0 + px - offset, y0 + 26, color)
            for px in range(18, 22):
                put_pixel(x0 + px + offset, y0 + 26, color)
        
        # Sleep 帧加 Zzz
        if row == 3 and col == 1:
            for i in range(3):
                put_pixel(x0 + 22 + i * 2, y0 + 4 - i * 2, (200, 200, 255, 255))

os.makedirs("assets", exist_ok=True)
png_data = create_png(TOTAL_W, TOTAL_H, pixels)
with open("assets/pet.png", "wb") as f:
    f.write(png_data)

print(f"Generated assets/pet.png ({TOTAL_W}x{TOTAL_H}, {ROWS} states x {COLS} frames)")
print("Layout: Idle, WalkRight, WalkLeft, Sleep, Eat, Happy, Startled, Sit")
