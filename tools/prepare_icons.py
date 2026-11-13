#!/usr/bin/env python3
"""Crop Sabbatia's section icons from the 1910 Rider–Waite–Smith cards.

All sources are public-domain scans of the "Pam-A" printing (Pamela Colman
Smith, c. 1910) on Wikimedia Commons:
  https://commons.wikimedia.org/wiki/File:RWS_Tarot_19_Sun.jpg
  https://commons.wikimedia.org/wiki/File:RWS_Tarot_18_Moon.jpg
  https://commons.wikimedia.org/wiki/File:RWS_Tarot_10_Wheel_of_Fortune.jpg

Usage:
  python3 tools/prepare_icons.py path/to/cards/ assets/icons
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 96  # output px: sharp up to 48 logical px at 2× scaling

# name: (card file, centre x, centre y, radius) in full-resolution scan pixels
ICONS = {
    "season": ("RWS_Tarot_19_Sun.jpg", 250, 860, 170),  # sunflowers
    "moon": ("RWS_Tarot_18_Moon.jpg", 595, 377, 265),  # the moon's face
    "wheel": ("RWS_Tarot_10_Wheel_of_Fortune.jpg", 552, 966, 380),  # the wheel
    "sabbat": ("RWS_Tarot_19_Sun.jpg", 549, 378, 235),  # the sun's face
}


def circle_crop(path: Path, cx: int, cy: int, r: int) -> Image.Image:
    ss = SIZE * 4
    im = Image.open(path).convert("RGB").crop((cx - r, cy - r, cx + r, cy + r)).resize((ss, ss), Image.LANCZOS)
    mask = Image.new("L", (ss, ss), 0)
    ImageDraw.Draw(mask).ellipse([0, 0, ss - 1, ss - 1], fill=255)
    im.putalpha(mask)
    return im.convert("RGBa").resize((SIZE, SIZE), Image.LANCZOS).convert("RGBA")


def main(cards: Path, out: Path) -> None:
    out.mkdir(parents=True, exist_ok=True)
    for name, (card, cx, cy, r) in ICONS.items():
        circle_crop(cards / card, cx, cy, r).save(out / f"{name}.png", optimize=True)
    print(f"wrote {len(ICONS)} icons to {out}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(Path(sys.argv[1]), Path(sys.argv[2]))
