#!/usr/bin/env python3
"""Gera os ícones do app a partir de código, sem dependências externas.

Rodar: python scripts/gen_icons.py

Produz dois conjuntos:
  src-tauri/icons/   ícones do executável e das janelas (PNG + ICO)
  msix/Assets/       logos exigidos pelo pacote MSIX da Microsoft Store

Os binários resultantes ficam versionados no repo; este script existe para que
qualquer pessoa consiga regerá-los e conferir que os arquivos de imagem não
escondem nada.
"""

from __future__ import annotations

import os
import struct
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON_DIR = os.path.join(ROOT, "src-tauri", "icons")
ASSET_DIR = os.path.join(ROOT, "msix", "Assets")

PURPLE = (145, 70, 255, 255)   # roxo Twitch
WHITE = (255, 255, 255, 255)


# --------------------------------------------------------------------------
# Primitivas de desenho
# --------------------------------------------------------------------------

def blend(dst: bytearray, idx: int, color: tuple[int, int, int, int]) -> None:
    r, g, b, a = color
    if a == 255:
        dst[idx : idx + 4] = bytes((r, g, b, 255))
        return
    inv = 255 - a
    dst[idx] = (r * a + dst[idx] * inv) // 255
    dst[idx + 1] = (g * a + dst[idx + 1] * inv) // 255
    dst[idx + 2] = (b * a + dst[idx + 2] * inv) // 255
    dst[idx + 3] = min(255, dst[idx + 3] + a)


def rounded_rect(buf, w, h, x0, y0, x1, y1, radius, color):
    for y in range(max(0, int(y0)), min(h, int(y1) + 1)):
        for x in range(max(0, int(x0)), min(w, int(x1) + 1)):
            cx = min(max(x, x0 + radius), x1 - radius)
            cy = min(max(y, y0 + radius), y1 - radius)
            dx, dy = x - cx, y - cy
            if dx * dx + dy * dy <= radius * radius:
                blend(buf, (y * w + x) * 4, color)


def triangle(buf, w, h, pts, color):
    (ax, ay), (bx, by), (cx, cy) = pts

    def side(px, py, x0, y0, x1, y1):
        return (x1 - x0) * (py - y0) - (y1 - y0) * (px - x0)

    min_x, max_x = int(min(ax, bx, cx)), int(max(ax, bx, cx)) + 1
    min_y, max_y = int(min(ay, by, cy)), int(max(ay, by, cy)) + 1
    for y in range(max(0, min_y), min(h, max_y)):
        for x in range(max(0, min_x), min(w, max_x)):
            d1 = side(x, y, ax, ay, bx, by)
            d2 = side(x, y, bx, by, cx, cy)
            d3 = side(x, y, cx, cy, ax, ay)
            has_neg = d1 < 0 or d2 < 0 or d3 < 0
            has_pos = d1 > 0 or d2 > 0 or d3 > 0
            if not (has_neg and has_pos):
                blend(buf, (y * w + x) * 4, color)


def downsample(src, w, h, out_w, out_h, ss):
    out = bytearray(out_w * out_h * 4)
    area = ss * ss
    for y in range(out_h):
        for x in range(out_w):
            r = g = b = a = 0
            for sy in range(ss):
                row = (y * ss + sy) * w
                for sx in range(ss):
                    i = (row + x * ss + sx) * 4
                    r += src[i]
                    g += src[i + 1]
                    b += src[i + 2]
                    a += src[i + 3]
            o = (y * out_w + x) * 4
            out[o] = r // area
            out[o + 1] = g // area
            out[o + 2] = b // area
            out[o + 3] = a // area
    return out


def render(out_w: int, out_h: int, plated: bool = True) -> bytearray:
    """Balão de fala centralizado.

    `plated` desenha a chapa roxa de fundo com o balão em branco — é a forma
    usada em quase todo lugar. Sem chapa (variantes `altform-unplated` do
    Windows, que aparecem sobre a barra de tarefas do usuário) o balão vai em
    roxo, senão ficaria branco sobre branco.
    """
    # Supersampling só onde vale: em telas grandes o custo cresce rápido e o
    # ganho visual é pequeno.
    ss = 4 if max(out_w, out_h) <= 160 else 2
    w, h = out_w * ss, out_h * ss
    buf = bytearray(w * h * 4)
    m = min(w, h)

    if plated:
        # Tiles largos vão full-bleed; ícones quadrados levam o canto arredondado.
        radius = m * 0.22 if out_w == out_h else 0
        rounded_rect(buf, w, h, 0, 0, w - 1, h - 1, radius, PURPLE)
        motif = WHITE
    else:
        motif = PURPLE

    cx, cy = w / 2.0, h / 2.0
    body_w, body_h = m * 0.60, m * 0.39
    x0, x1 = cx - body_w / 2, cx + body_w / 2
    y0 = cy - m * 0.26
    y1 = y0 + body_h

    rounded_rect(buf, w, h, x0, y0, x1, y1, m * 0.08, motif)
    triangle(
        buf,
        w,
        h,
        [(x0 + body_w * 0.17, y1 - 1), (x0 + body_w * 0.55, y1 - 1), (x0 + body_w * 0.20, y1 + m * 0.19)],
        motif,
    )

    return downsample(buf, w, h, out_w, out_h, ss)


# --------------------------------------------------------------------------
# PNG
# --------------------------------------------------------------------------

def png_bytes(w: int, h: int, rgba: bytearray) -> bytes:
    raw = b"".join(
        b"\x00" + bytes(rgba[y * w * 4 : (y + 1) * w * 4]) for y in range(h)
    )

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def write_png(path: str, w: int, h: int, plated: bool = True) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "wb") as fh:
        fh.write(png_bytes(w, h, render(w, h, plated)))
    print("  ", os.path.relpath(path, ROOT))


# --------------------------------------------------------------------------
# ICO
# --------------------------------------------------------------------------

def dib_bytes(size: int, rgba: bytearray) -> bytes:
    """Entrada BMP (DIB) para o .ico. Linhas de baixo para cima, BGRA."""
    header = struct.pack("<IiiHHIIiiII", 40, size, size * 2, 1, 32, 0, 0, 0, 0, 0, 0)
    rows = []
    for y in reversed(range(size)):
        row = bytearray()
        for x in range(size):
            i = (y * size + x) * 4
            row += bytes((rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]))
        rows.append(bytes(row))
    mask_row = ((size + 31) // 32) * 4
    return header + b"".join(rows) + b"\x00" * (mask_row * size)


def ico_bytes(images: list[tuple[int, bytes]]) -> bytes:
    """images: lista de (tamanho, payload). 256 vai como PNG, o resto como DIB."""
    count = len(images)
    offset = 6 + 16 * count
    directory = b""
    body = b""
    for size, payload in images:
        directory += struct.pack(
            "<BBBBHHII",
            0 if size >= 256 else size,
            0 if size >= 256 else size,
            0,
            0,
            1,
            32,
            len(payload),
            offset,
        )
        body += payload
        offset += len(payload)
    return struct.pack("<HHH", 0, 1, count) + directory + body


# --------------------------------------------------------------------------

def main() -> None:
    print("ícones do app:")
    for size, name in [(32, "32x32.png"), (128, "128x128.png"), (256, "128x128@2x.png")]:
        write_png(os.path.join(ICON_DIR, name), size, size)

    entries = [(s, dib_bytes(s, render(s, s))) for s in (16, 32, 48)]
    entries.append((256, png_bytes(256, 256, render(256, 256))))
    ico_path = os.path.join(ICON_DIR, "icon.ico")
    with open(ico_path, "wb") as fh:
        fh.write(ico_bytes(entries))
    print("  ", os.path.relpath(ico_path, ROOT))

    # Nomes e tamanhos ditados pelo esquema do AppxManifest. Os `.scale-200`
    # são as versões para telas de alta densidade; os `altform-unplated` são
    # usados pelo Windows sobre o fundo do próprio usuário (barra de tarefas).
    print("assets da Microsoft Store:")
    store = [
        ("Square44x44Logo.png", 44, 44, True),
        ("Square44x44Logo.scale-200.png", 88, 88, True),
        ("Square44x44Logo.targetsize-24_altform-unplated.png", 24, 24, False),
        ("Square44x44Logo.targetsize-48_altform-unplated.png", 48, 48, False),
        ("Square71x71Logo.png", 71, 71, True),
        ("Square71x71Logo.scale-200.png", 142, 142, True),
        ("Square150x150Logo.png", 150, 150, True),
        ("Square150x150Logo.scale-200.png", 300, 300, True),
        ("Square310x310Logo.png", 310, 310, True),
        ("Wide310x150Logo.png", 310, 150, True),
        ("Wide310x150Logo.scale-200.png", 620, 300, True),
        ("StoreLogo.png", 50, 50, True),
        ("StoreLogo.scale-200.png", 100, 100, True),
    ]
    for name, w, h, plated in store:
        write_png(os.path.join(ASSET_DIR, name), w, h, plated)


if __name__ == "__main__":
    main()
