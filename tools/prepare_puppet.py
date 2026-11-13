#!/usr/bin/env python3
"""Cut a companion out of a 1910 Rider–Waite–Smith card and split it into
puppet parts.

Sources (public domain, Pamela Colman Smith, "Pam-A" printing c. 1910):
  hermit  https://commons.wikimedia.org/wiki/File:RWS_Tarot_09_Hermit.jpg
  fool    https://commons.wikimedia.org/wiki/File:RWS_Tarot_00_Fool.jpg

Usage:
  python3 tools/prepare_puppet.py hermit path/to/RWS_Tarot_09_Hermit.jpg assets/hermit
  python3 tools/prepare_puppet.py fool   path/to/RWS_Tarot_00_Fool.jpg   assets/fool

Needs only Pillow. Writes one PNG per part plus rig.toml. Parts are listed in
drawing order; a part with `parent` follows its parent's movement; a
`hang = true` part keeps its own world angle (a pendulum: lantern, bag).
All coordinates below are full-resolution scan pixels.
"""

import sys
from collections import deque
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

# ── shared helpers ────────────────────────────────────────────────────────


def poly_mask(size, *polys) -> Image.Image:
    m = Image.new("L", size, 0)
    d = ImageDraw.Draw(m)
    for p in polys:
        d.polygon(p, fill=255)
    return m


def box(x0, y0, x1, y1):
    return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]


def far_from(size, center, radius) -> Image.Image:
    m = Image.new("L", size, 255)
    cx, cy = center
    ImageDraw.Draw(m).ellipse([cx - radius, cy - radius, cx + radius, cy + radius], fill=0)
    return m


def component(mask: Image.Image, seed) -> Image.Image:
    work = mask.copy()
    ImageDraw.floodfill(work, seed, 128)
    return work.point(lambda v: 255 if v == 128 else 0)


def cloth_colour(img: Image.Image, mask: Image.Image, light_only: bool = True):
    """Median colour under `mask`: of the light (cloth, not ink) pixels for
    plain cloth such as the Hermit's hood, or of all pixels for patterned
    cloth such as the Fool's tunic."""
    px = [p for p, m in zip(img.getdata(), mask.getdata()) if m > 200 and (not light_only or sum(p) > 330)]
    if not px:
        return (160, 150, 142)
    return tuple(sorted(c[i] for c in px)[len(px) // 2] for i in range(3))


# ── figure masks ──────────────────────────────────────────────────────────


def hermit_mask(img: Image.Image) -> Image.Image:
    """The Hermit stands against a flat teal sky: key it out by hue."""
    r, g, b = img.split()
    rg = ImageChops.subtract(g, r)
    rb = ImageChops.subtract(b, r)
    gb = ImageChops.difference(g, b)
    lum = img.convert("L")
    is_sky = ImageChops.multiply(
        ImageChops.multiply(rg.point(lambda v: 255 if v > 8 else 0), rb.point(lambda v: 255 if v > 8 else 0)),
        ImageChops.multiply(gb.point(lambda v: 255 if v < 22 else 0), lum.point(lambda v: 255 if 70 < v < 200 else 0)),
    )
    raw = ImageChops.invert(is_sky).filter(ImageFilter.MedianFilter(5))
    raw = raw.filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MinFilter(3))
    # Scene: inside the card frame, above the snow.
    raw = ImageChops.multiply(raw, poly_mask(img.size, box(60, 150, 1060, 1595)))
    lantern_box = poly_mask(img.size, box(60, 245, 275, 565))
    fig = ImageChops.lighter(component(raw, (660, 950)), ImageChops.multiply(raw, lantern_box))
    # Drop the drawn rays beside the lantern (the app draws its own glow).
    r, g, b = img.split()
    yellow = ImageChops.multiply(
        ImageChops.multiply(r.point(lambda v: 255 if v > 130 else 0), g.point(lambda v: 255 if v > 130 else 0)),
        ImageChops.subtract(r, b).point(lambda v: 255 if v > 35 else 0),
    )
    lantern_body = [(130, 250), (195, 250), (240, 290), (252, 310), (252, 550), (82, 550), (82, 310), (100, 285)]
    rays = ImageChops.multiply(yellow.filter(ImageFilter.MaxFilter(5)),
                               ImageChops.subtract(lantern_box, poly_mask(img.size, lantern_body)))
    fig = component(ImageChops.subtract(fig, rays), (660, 950))
    return fig.filter(ImageFilter.MinFilter(3))


FOOL_OUTLINE = [(40, 650), (470, 385), (515, 280), (690, 245), (765, 195), (895, 200), (905, 470), (1035, 470),
                (1045, 650), (1015, 905), (900, 915), (765, 905), (745, 1040), (805, 1165), (800, 1285), (700, 1335),
                (600, 1305), (585, 1425), (445, 1425), (430, 1260), (415, 1060), (355, 1060), (335, 900), (150, 885),
                (40, 790)]
DOG_OUTLINE = [(700, 1245), (780, 1235), (838, 1160), (968, 1160), (980, 1225), (1002, 1262), (1048, 1288),
               (1052, 1400), (1030, 1580), (880, 1580), (855, 1448), (760, 1428), (700, 1335)]


def fool_mask(img: Image.Image) -> Image.Image:
    """The Fool stands before yellow sky, snow and a rocky cliff. Grow the
    background inwards from outside a generous outline, through sky-, snow-
    and rock-coloured pixels only, so Smith's ink outlines stop it."""
    W, H = img.size
    thick = img.filter(ImageFilter.MinFilter(5))  # ink grows ~2 px, closing small gaps
    px = thick.load()
    # Where white is part of the figure: the dog, the boots, the shirt, the rose.
    white_ok = poly_mask(img.size, DOG_OUTLINE, box(462, 1215, 578, 1415), box(670, 1130, 800, 1290),
                         box(90, 600, 470, 800), box(460, 480, 580, 580), box(900, 470, 1030, 580)).load()
    boots = poly_mask(img.size, box(462, 1215, 578, 1415), box(670, 1130, 800, 1290)).load()
    outline = poly_mask(img.size, FOOL_OUTLINE, DOG_OUTLINE).load()
    bg = bytearray(W * H)
    for y in range(H):
        for x in range(W):
            r, g, b = px[x, y]
            sky = r > 150 and g > 120 and b < 150 and r - b > 45 and not boots[x, y]
            white = r > 150 and g > 150 and b > 145 and not white_ok[x, y]
            blue = b > r + 10 and b > 100
            rock = y > 1250 and r > 80 and r - b > 25 and g < r - 5 and not boots[x, y]
            bg[y * W + x] = 1 if (sky or white or blue or rock) else 0
    seen = bytearray(W * H)
    q = deque()
    for y in range(H):
        for x in range(W):
            i = y * W + x
            if bg[i] and not outline[x, y]:
                seen[i] = 1
                q.append(i)
    # Background pockets enclosed by the figure (between the legs, under the staff).
    for x, y in [(600, 1200), (640, 1260), (400, 500), (300, 840), (860, 940), (700, 1100)]:
        i = y * W + x
        if bg[i] and not seen[i]:
            seen[i] = 1
            q.append(i)
    while q:
        i = q.popleft()
        x = i % W
        for j in ((i - 1) if x > 0 else -1, (i + 1) if x < W - 1 else -1, i - W, i + W):
            if 0 <= j < W * H and bg[j] and not seen[j]:
                seen[j] = 1
                q.append(j)
    fig = Image.frombytes("L", (W, H), bytes(0 if s else 255 for s in seen))
    ground = poly_mask(img.size, box(46, 46, W - 46, 1408), DOG_OUTLINE)
    fig = ImageChops.multiply(ImageChops.multiply(fig, ground), poly_mask(img.size, FOOL_OUTLINE, DOG_OUTLINE))
    # A strand of mountain outline caught between the legs.
    fig = ImageChops.subtract(fig, poly_mask(img.size, box(585, 1205, 650, 1250)))
    fig = fig.filter(ImageFilter.MinFilter(5))  # undo the ink thickening at the outer edge
    fig = fig.filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MinFilter(3))
    out = component(fig, (560, 700))
    for seed in [(930, 1330), (790, 380)]:  # the dog; the bag (joined to the staff by a thin knot)
        if fig.getpixel(seed) == 255 and out.getpixel(seed) == 0:
            out = ImageChops.lighter(out, component(fig, seed))
    return out


# ── characters ────────────────────────────────────────────────────────────
# Each part: polygons, optional pivot, parent, hang, overlap (pixels kept in
# the body around the pivot so joints never gap) and back (paint the body
# flat cloth colour behind the part, as paper puppets are backed).

HERMIT = dict(
    size=(1123, 1920),
    crop=(60, 150, 1060, 1595),
    mask=hermit_mask,
    out_height=320,
    light=("lantern", (160, 415)),
    parts=[
        dict(name="body"),
        dict(name="head", polys=[[(415, 180), (625, 175), (705, 260), (700, 400), (660, 490), (560, 500), (490, 485),
                                  (432, 420), (415, 300)]], pivot=(540, 480), overlap=70, back="light"),
        dict(name="arm", polys=[[(110, 180), (265, 185), (325, 220), (385, 290), (385, 365), (340, 385), (290, 350),
                                 (255, 300), (210, 268), (195, 254), (130, 254), (110, 250)]], pivot=(350, 350),
             overlap=90, exclude="lantern"),
        dict(name="staff", polys=[[(378, 180), (422, 180), (515, 1595), (472, 1595)],
                                  [(395, 365), (515, 370), (520, 485), (460, 500), (395, 480)]]),
        dict(name="lantern", polys=[[(130, 254), (195, 254), (195, 268), (260, 290), (260, 562), (60, 562), (60, 290),
                                     (130, 268)]], pivot=(160, 253), parent="arm", hang=True),
    ],
)

FOOL = dict(
    size=(1144, 1919),
    crop=(40, 190, 1075, 1585),
    mask=fool_mask,
    out_height=360,
    light=None,
    parts=[
        dict(name="staff_arm", polys=[[(40, 650), (440, 470), (480, 540), (470, 600), (420, 640), (400, 880),
                                       (150, 885), (40, 790)], [(430, 420), (775, 220), (800, 250), (470, 470)]],
             pivot=(445, 565), overlap=115),
        dict(name="leg", polys=[[(565, 1000), (690, 985), (730, 1090), (805, 1150), (805, 1295), (690, 1295),
                                 (648, 1205), (585, 1105)]], pivot=(615, 1015), overlap=90),
        dict(name="body"),
        dict(name="head", polys=[[(500, 330), (545, 270), (625, 250), (770, 238), (778, 268), (700, 320),
                                  (695, 470), (650, 522), (570, 528), (518, 445), (498, 380)]],
             pivot=(600, 525), overlap=65),
        dict(name="rose_arm", polys=[[(705, 560), (760, 515), (905, 515), (925, 465), (1040, 465), (1048, 655),
                                      (1005, 700), (1008, 910), (870, 912), (780, 785), (715, 700)]],
             pivot=(725, 600), overlap=100, back="all", back_radius=230),
        dict(name="bag", polys=[box(700, 222, 900, 475)], pivot=(788, 248), parent="staff_arm", hang=True),
        dict(name="dog", polys=[DOG_OUTLINE], pivot=(955, 1560)),
    ],
)

CHARACTERS = {"hermit": HERMIT, "fool": FOOL}


def build(spec, scan: Path, out: Path) -> None:
    full = Image.open(scan).convert("RGB")
    if full.size != spec["size"]:
        sys.exit(f"unexpected scan size {full.size}; coordinates assume {spec['size']}")
    size = full.size
    figure = spec["mask"](full).filter(ImageFilter.GaussianBlur(0.8))

    masks = {p["name"]: poly_mask(size, *p["polys"]) for p in spec["parts"] if "polys" in p}
    for p in spec["parts"]:
        if p.get("exclude"):
            masks[p["name"]] = ImageChops.subtract(masks[p["name"]], masks[p["exclude"]])

    # The body is everything not taken by a moving part (static parts such
    # as the Hermit's staff are copies drawn on top, so stay in the body).
    removed = Image.new("L", size, 0)
    body_rgb = full
    solid = figure.point(lambda v: 255 if v > 200 else 0)
    for p in spec["parts"]:
        if "pivot" not in p:
            continue
        m = masks[p["name"]]
        if p.get("overlap"):
            m = ImageChops.multiply(m, far_from(size, p["pivot"], p["overlap"]))
        if p.get("back"):
            # Only for parts drawn in front of the body. Near the joint, keep
            # the silhouette in the body but paint it flat cloth colour, so a
            # moving part never uncovers a hole; farther out (hands, flowers)
            # remove it, or a flat ghost of the limb would show.
            behind = ImageChops.multiply(m, solid)
            if p.get("back_radius"):
                behind = ImageChops.subtract(behind, far_from(size, p["pivot"], p["back_radius"]))
            fill = cloth_colour(full, behind, light_only=p["back"] == "light")
            body_rgb = Image.composite(Image.new("RGB", size, fill), body_rgb, behind)
            removed = ImageChops.lighter(removed, ImageChops.subtract(m, behind))
        else:
            removed = ImageChops.lighter(removed, m)

    x0, y0, x1, y1 = spec["crop"]
    s = spec["out_height"] / (y1 - y0)
    out_size = (round((x1 - x0) * s), spec["out_height"])
    to_out = lambda p: ((p[0] - x0) * s, (p[1] - y0) * s)

    out.mkdir(parents=True, exist_ok=True)
    rig = [
        "# Generated by tools/prepare_puppet.py — do not edit by hand.",
        "# Source: Pamela Colman Smith, Rider–Waite–Smith tarot (c. 1910), public domain.",
        f"width = {out_size[0]}",
        f"height = {out_size[1]}",
    ]
    if spec["light"]:
        part, point = spec["light"]
        lx, ly = to_out(point)
        rig += [f'light_part = "{part}"', f"light = [{lx:.1f}, {ly:.1f}]"]
    rig.append("")
    for p in spec["parts"]:
        name = p["name"]
        if name == "body":
            alpha = ImageChops.subtract(figure, removed)
            src = body_rgb
        else:
            alpha = ImageChops.multiply(figure, masks[name])
            src = full
        rgba = src.copy()
        rgba.putalpha(alpha)
        rgba = rgba.crop(spec["crop"])
        small = rgba.convert("RGBa").resize(out_size, Image.LANCZOS).convert("RGBA")
        bbox = small.getchannel("A").point(lambda v: 255 if v > 3 else 0).getbbox()
        if bbox is None:
            sys.exit(f"part {name} is empty")
        small.crop(bbox).save(out / f"{name}.png", optimize=True)
        rig += ["[[part]]", f'name = "{name}"', f"x = {bbox[0]}", f"y = {bbox[1]}"]
        if "pivot" in p:
            px, py = to_out(p["pivot"])
            rig.append(f"pivot = [{px:.1f}, {py:.1f}]")
        if p.get("parent"):
            rig.append(f'parent = "{p["parent"]}"')
        if p.get("hang"):
            rig.append("hang = true")
        rig.append("")
    for stale in out.glob("*.png"):
        if stale.stem not in {p["name"] for p in spec["parts"]}:
            stale.unlink()
    (out / "rig.toml").write_text("\n".join(rig))
    print(f"wrote {len(spec['parts'])} parts at {out_size[0]}×{out_size[1]} to {out}")


if __name__ == "__main__":
    if len(sys.argv) != 4 or sys.argv[1] not in CHARACTERS:
        sys.exit(__doc__)
    build(CHARACTERS[sys.argv[1]], Path(sys.argv[2]), Path(sys.argv[3]))
