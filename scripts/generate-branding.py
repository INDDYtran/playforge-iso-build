#!/usr/bin/env python3
"""Erzeugt die Playforge-Bildmarken im Design «ASCII Brutal».

Reines Schwarz, Arch-Blau (#1793D1), Papierweiß (#F2F2EC) und Monospace.
Alle Dateien werden deterministisch aus diesem Skript erzeugt:

    python3 scripts/generate-branding.py

Benötigt Pillow. Die Schrift ist die im Control Center gebündelte
Geist Mono, damit die Assets unabhängig vom Host reproduzierbar bleiben.
"""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
FONT_PATH = ROOT / "control-center/assets/fonts/GeistMono-Variable.ttf"

BLACK = (0, 0, 0)
INK = (5, 5, 5)
EDGE = (28, 28, 28)
EDGE_SOFT = (16, 16, 16)
TEXT = (242, 242, 236)
MUTED = (150, 150, 142)
FAINT = (86, 86, 80)
BLUE = (23, 147, 209)
BLUE_DIM = (12, 74, 105)

ART = [
    "        /\\",
    "       /  \\",
    "      / /\\ \\",
    "     / /  \\ \\",
    "    / /    \\ \\",
    "   /_/      \\_\\",
]
LINE_FACTOR = 0.76


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    face = ImageFont.truetype(str(FONT_PATH), size)
    if bold:
        for name in ("Bold", "SemiBold", "Medium"):
            try:
                face.set_variation_by_name(name)
                break
            except Exception:
                continue
    return face


def track(draw: ImageDraw.ImageDraw, xy: tuple[float, float], text: str,
          face: ImageFont.FreeTypeFont, fill, spacing: float = 0.0) -> float:
    """Zeichnet Text mit zusätzlichem Buchstabenabstand, gibt die Breite zurück."""
    x, y = xy
    for char in text:
        draw.text((x, y), char, font=face, fill=fill)
        x += face.getlength(char) + spacing
    return x - xy[0]


def tracked_width(text: str, face: ImageFont.FreeTypeFont, spacing: float) -> float:
    width = sum(face.getlength(char) for char in text)
    return width + spacing * max(0, len(text) - 1)


def grid(img: Image.Image) -> None:
    """Feines Raster plus Scanlines – nur angedeutet, das Schwarz bleibt tief."""
    w, h = img.size
    scale = h / 1080
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    step = max(24, int(56 * scale))
    for x in range(0, w, step):
        d.line([(x, 0), (x, h)], fill=(255, 255, 255, 6))
    for y in range(0, h, step):
        d.line([(0, y), (w, y)], fill=(255, 255, 255, 6))
    for x in range(0, w, step * 4):
        for y in range(0, h, step * 4):
            d.line([(x, y), (x + step, y)], fill=(*BLUE, 26))
            d.line([(x, y), (x, y + step)], fill=(*BLUE, 26))
    merged = Image.alpha_composite(img.convert("RGBA"), layer)
    img.paste(merged.convert(img.mode), (0, 0))


def scanlines(img: Image.Image, alpha: int = 12) -> None:
    w, h = img.size
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    for y in range(0, h, 3):
        d.line([(0, y), (w, y)], fill=(255, 255, 255, alpha))
    base = img.convert("RGBA")
    img.paste(Image.alpha_composite(base, layer).convert(img.mode), (0, 0))


def wallpaper(w: int, h: int, hero: bool = True) -> Image.Image:
    img = Image.new("RGB", (w, h), BLACK)
    grid(img)
    scanlines(img, 10)
    d = ImageDraw.Draw(img)
    scale = h / 1080
    margin = int(w * 0.055)

    small = font(int(17 * scale))
    label = font(int(20 * scale), bold=True)
    kicker = font(int(18 * scale))
    mark_size = int(h * 0.128)
    mark = font(mark_size, bold=True)
    sub = font(int(19 * scale))
    art_size = max(14, int(h * 0.062))
    art = font(art_size)

    # Kopfzeile
    top = int(h * 0.068)
    d.rectangle([margin, top, margin + int(13 * scale), top + int(13 * scale)], fill=BLUE)
    x = margin + int(27 * scale)
    x += track(d, (x, top - int(5 * scale)), "PLAYFORGE", label, TEXT, 1.5 * scale)
    d.text((x + int(16 * scale), top - int(2 * scale)),
           "// ARCH LINUX GAMING", font=small, fill=FAINT)
    d.line([(margin, int(h * 0.115)), (w - margin, int(h * 0.115))], fill=EDGE)

    # ASCII-Wappen rechts: das Arch-Zeichen als reines Textbild
    art_line = art_size * LINE_FACTOR
    art_w = max(art.getlength(line) for line in ART)
    ax = int(w * 0.94) - art_w
    ay = int(h * 0.26)
    for index, line in enumerate(ART):
        d.text((ax, ay + index * art_line), line, font=art, fill=BLUE)

    # Wortmarke links
    if hero:
        y = int(h * 0.33)
        d.text((margin, y), "> GAMING.FORGED", font=kicker, fill=BLUE)
        y += int(38 * scale)
        d.text((margin, y - int(mark_size * 0.18)), "PLAYFORGE", font=mark, fill=TEXT)
        y = y - int(mark_size * 0.18) + int(mark_size * 1.05)
        d.rectangle([margin, y, margin + int(w * 0.14), y + max(4, int(6 * scale))], fill=BLUE)
        y += int(30 * scale)
        track(d, (margin, y), "ARCH LINUX   /   KDE PLASMA   /   x86_64", sub, MUTED, 1.2 * scale)

    # Fußzeile mit ASCII-Marken
    foot = int(h * 0.88)
    d.line([(margin, foot - int(24 * scale)), (w - margin, foot - int(24 * scale))], fill=EDGE)
    track(d, (margin, foot), "[ ARCH ]   [ GAMING ]   [ ASCII BRUTAL ]", small, FAINT,
          1.0 * scale)
    right = "PLAYFORGE // ASCII BRUTAL"
    d.text((w - margin - small.getlength(right), foot), right, font=small, fill=FAINT)
    return img


def boot_frame(w: int, h: int, glow: bool, compact: bool = False) -> Image.Image:
    img = Image.new("RGB", (w, h), BLACK)
    grid(img)
    scanlines(img, 8)
    d = ImageDraw.Draw(img)
    scale = h / 1080

    art_size = max(12, int(h * (0.082 if compact else 0.052)))
    art = font(art_size)
    art_w = max(art.getlength(line) for line in ART)
    art_h = art_size * LINE_FACTOR * len(ART)
    ax = (w - art_w) / 2
    ay = int(h * (0.17 if compact else 0.33))
    for index, line in enumerate(ART):
        d.text((ax, ay + index * art_size * LINE_FACTOR), line, font=art, fill=BLUE)

    mark_size = int(h * (0.098 if compact else 0.062))
    mark = font(mark_size, bold=True)
    mark_w = mark.getlength("PLAYFORGE")
    gap = int(h * (0.028 if compact else 0.035))
    d.text(((w - mark_w) / 2, ay + art_h + gap), "PLAYFORGE", font=mark, fill=TEXT)

    small = font(max(8, int(h * (0.024 if compact else 0.0139))))
    sub = "ARCH LINUX   /   GAMING   /   ASCII BRUTAL"
    sub_w = tracked_width(sub, small, 1.4 * scale)
    track(d, ((w - sub_w) / 2, ay + art_h + gap + int(mark_size * 1.15)), sub,
          small, MUTED, 1.4 * scale)

    bar_y = int(h * 0.86)
    line_w = int(w * 0.34)
    d.line([((w - line_w) / 2, bar_y), ((w + line_w) / 2, bar_y)], fill=EDGE)

    if glow:
        # Rahmen und Cursor sind die einzigen bewegten Elemente.
        frame = 2 * scale
        pad = int(w * 0.06)
        top = int(h * 0.20)
        bottom = int(h * 0.80)
        length = int(w * 0.035)
        f = max(1, int(frame))
        for x0, y0, dx, dy in (
            (pad, top, 1, 1), (w - pad, top, -1, 1),
            (pad, bottom, 1, -1), (w - pad, bottom, -1, -1),
        ):
            d.line([(x0, y0), (x0 + dx * length, y0)], fill=BLUE, width=f)
            d.line([(x0, y0), (x0, y0 + dy * length)], fill=BLUE, width=f)

        cursor = int(22 * scale)
        cx = int((w - cursor) / 2)
        cy = bar_y + int(30 * scale)
        d.rectangle([cx, cy, cx + cursor, cy + cursor], fill=TEXT)

    return img


def logo(size: int = 512) -> Image.Image:
    img = Image.new("RGB", (size, size), BLACK)
    d = ImageDraw.Draw(img)
    pad = max(3, size // 26)
    d.rectangle([pad, pad, size - pad, size - pad], outline=TEXT, width=max(3, size // 42))
    # Arch-Wappen als harte Vektorkanten: zwei verschachtelte Sparren.
    cx = size * 0.5
    stroke = max(4, int(size * 0.068))
    for apex, half, bottom in ((0.26, 0.29, 0.74), (0.42, 0.155, 0.66)):
        top = size * apex
        d.line([(cx, top), (cx - half * size, size * bottom)], fill=BLUE, width=stroke)
        d.line([(cx, top), (cx + half * size, size * bottom)], fill=BLUE, width=stroke)
    # ASCII-Blockcursor als Prompt in der Öffnung
    cursor = int(size * 0.095)
    x = int(cx - cursor / 2)
    y = int(size * 0.77)
    d.rectangle([x, y, x + cursor, y + cursor], fill=TEXT)
    return img


def plasma_preview(w: int, h: int) -> Image.Image:
    img = wallpaper(w, h, hero=False).convert("RGBA")
    veil = Image.new("RGBA", (w, h), (0, 0, 0, 120))
    img = Image.alpha_composite(img, veil)
    d = ImageDraw.Draw(img)
    scale = h / 1080

    def px(value: float) -> int:
        return int(value * scale)

    mono = font(px(17))
    mono_small = font(px(13))
    mono_big = font(px(38), bold=True)

    # Fenster
    left, top, right, bottom = px(250), px(200), px(1670), px(880)
    d.rectangle([left, top, right, bottom], fill=(5, 5, 5, 250), outline=TEXT, width=px(2))
    d.rectangle([left, top, right, top + px(46)], fill=BLACK, outline=TEXT, width=px(2))
    d.line([(left, top + px(46)), (right, top + px(46))], fill=BLUE, width=px(2))
    d.text((left + px(18), top + px(14)), "PLAYFORGE CONTROL CENTER", font=mono, fill=TEXT)
    for index in range(3):
        bx = right - px(30) - index * px(26)
        d.rectangle([bx, top + px(18), bx + px(10), top + px(28)], outline=MUTED, width=px(1))

    # Seitenleiste
    rail = px(320)
    d.rectangle([left, top + px(46), left + rail, bottom], fill=BLACK)
    d.line([(left + rail, top + px(46)), (left + rail, bottom)], fill=EDGE, width=px(1))
    rows = [
        ("01", "ÜBERSICHT", True),
        ("02", "LEISTUNG", False),
        ("03", "GAMING", False),
        ("04", "GRAFIK", False),
        ("05", "DIENSTE", False),
    ]
    y = top + px(70)
    for index, label, active in rows:
        if active:
            d.rectangle([left + px(10), y, left + rail - px(10), y + px(38)], fill=BLUE)
            color = BLACK
        else:
            color = MUTED
        d.text((left + px(22), y + px(10)), index, font=mono_small, fill=color)
        d.text((left + px(60), y + px(9)), label, font=mono, fill=color)
        y += px(46)

    # Inhalt
    cx = left + rail + px(32)
    cy = top + px(78)
    d.text((cx, cy), "PLAYFORGE :: 01 / OVERVIEW", font=mono_small, fill=BLUE)
    cy += px(26)
    d.text((cx, cy), "ÜBERSICHT", font=mono_big, fill=TEXT)
    cy += px(60)
    d.text((cx, cy), "Systemstatus, Spielbereitschaft und Schnellaktionen", font=mono_small,
           fill=MUTED)
    cy += px(38)
    d.line([(cx, cy), (right - px(32), cy)], fill=EDGE, width=px(1))
    cy += px(24)

    cards = [
        ("PROZESSOR", "16C/32T", "4.8 GHZ · SCHEDUTIL"),
        ("SPEICHER", "11.2 GB", "31.9 GB · 35 %"),
        ("GRAFIK", "AMD", "RADV · MESA 26.1"),
        ("PROFIL", "PERFORMANCE", "GOV PERFORMANCE"),
    ]
    card_w = (right - px(32) - cx - px(3 * 12)) / 4
    for index, (label, value, note) in enumerate(cards):
        x0 = cx + index * (card_w + px(12))
        x1 = x0 + card_w
        d.rectangle([x0, cy, x1, cy + px(96)], outline=EDGE, width=px(1))
        d.rectangle([x0, cy, x0 + px(12), cy + px(3)], fill=BLUE)
        d.text((x0 + px(12), cy + px(12)), label, font=mono_small, fill=MUTED)
        d.text((x0 + px(12), cy + px(38)), value, font=font(px(24), bold=True), fill=TEXT)
        d.text((x0 + px(12), cy + px(72)), note, font=font(px(11)), fill=FAINT)
    cy += px(122)

    for label, value in (
        ("DISTRIBUTION", "Playforge 2026.11"),
        ("KERNEL", "6.17.8-zen1-1-zen"),
        ("SITZUNG", "Wayland · KDE Plasma 6.5"),
        ("PAKETE", "1247 installiert"),
    ):
        d.rectangle([cx, cy, right - px(32), cy + px(34)], outline=EDGE_SOFT, width=px(1))
        d.text((cx + px(12), cy + px(9)), label, font=mono_small, fill=FAINT)
        d.text((cx + px(230), cy + px(9)), value, font=mono, fill=TEXT)
        cy += px(40)

    # Panel
    panel_h = px(54)
    d.rectangle([0, h - panel_h, w, h], fill=BLACK)
    d.line([(0, h - panel_h), (w, h - panel_h)], fill=BLUE, width=px(2))
    d.rectangle([px(44), h - panel_h + px(18), px(58), h - panel_h + px(32)], fill=BLUE)
    d.text((px(72), h - panel_h + px(18)), "PLAYFORGE", font=mono, fill=TEXT)
    d.text((w - px(160), h - panel_h + px(18)), "23:41", font=mono, fill=TEXT)
    return img.convert("RGB")


def splash_preview(w: int, h: int) -> Image.Image:
    img = Image.new("RGB", (w, h), BLACK)
    grid(img)
    scanlines(img, 6)
    d = ImageDraw.Draw(img)
    scale = h / 1080

    art = font(int(h * 0.030))
    art_w = max(art.getlength(line) for line in ART)
    art_h = h * 0.030 * LINE_FACTOR * len(ART)
    for index, line in enumerate(ART):
        d.text(((w - art_w) / 2, int(h * 0.36) + index * h * 0.030 * LINE_FACTOR), line,
               font=art, fill=BLUE)

    mark = font(int(h * 0.052), bold=True)
    mark_w = mark.getlength("PLAYFORGE")
    d.text(((w - mark_w) / 2, int(h * 0.36) + art_h + int(30 * scale)), "PLAYFORGE",
           font=mark, fill=TEXT)

    small = font(int(13 * scale))
    sub = "ARCH LINUX   /   GAMING   /   ASCII BRUTAL"
    sub_w = tracked_width(sub, small, 1.2 * scale)
    track(d, ((w - sub_w) / 2, int(h * 0.36) + art_h + int(30 * scale) + h * 0.078), sub,
          small, MUTED, 1.2 * scale)

    # Segmentierter Ladebalken wie im Splash
    segments = 32
    total_w = int(w * 0.30)
    gap = max(2, int(2 * scale))
    seg_w = (total_w - gap * (segments - 1)) / segments
    x = (w - total_w) / 2
    y = int(h * 0.78)
    for index in range(segments):
        filled = index < 13
        d.rectangle([x, y, x + seg_w, y + max(3, int(5 * scale))],
                    fill=BLUE if filled else EDGE)
        x += seg_w + gap
    return img


def main() -> None:
    sizes = [(1920, 1080), (2560, 1440), (3840, 2160)]
    for w, h in sizes:
        target = ROOT / "overlay/usr/share/wallpapers/Playforge/contents/images" / f"{w}x{h}.png"
        wallpaper(w, h).save(target)
        print(target.relative_to(ROOT))

    preview_dir = ROOT / "overlay/usr/share/plasma/look-and-feel/org.playforge.desktop/contents/previews"
    plasma_preview(1920, 1080).save(preview_dir / "preview.png")
    plasma_preview(1920, 1080).save(preview_dir / "fullscreenpreview.jpg", quality=90)
    splash_preview(1920, 1080).save(preview_dir / "splash.png")
    print("previews")

    plymouth = ROOT / "overlay/usr/share/plymouth/themes/playforge"
    boot_frame(1920, 1080, glow=False).save(plymouth / "background.png", optimize=True)
    boot_frame(1920, 1080, glow=True).save(plymouth / "background-glow.png", optimize=True)
    print("plymouth")

    logo(512).save(ROOT / "overlay/usr/share/pixmaps/playforge-logo.png", optimize=True)
    print("logo")

    # syslinux zeigt das Bild bei 640x480
    small = boot_frame(640, 480, glow=False, compact=True)
    small.save(ROOT / "overlay/usr/local/share/playforge/branding/syslinux-splash.png",
               optimize=True)
    print("syslinux")


if __name__ == "__main__":
    main()
