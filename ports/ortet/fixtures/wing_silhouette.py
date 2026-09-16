#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""T1's wing fixture: a rigid-part voxel body rendered as CSS 3D boxes.

Emits an HTML document whose faces are `preserve-3d` boxes placed with
`translate3d`/`rotate3d` under an orthographic `matrix3d` camera, then
compares Ortet's headless readback silhouette against the appearance crate's
bake silhouette at the same facing and scale.

The bake side is produced by the separate read-only driver recorded under
`Code/testing/genet/t1_css_3d_20260915/bake_driver`; this script never imports
the appearance crate, it only reads that driver's `body.json` and `bake.pgm`.

    wing_silhouette.py emit  <body.json> <out.html>
    wing_silhouette.py compare <body.json> <bake.pgm> <ortet.png> [mask.png]

The camera is the bake's own projection written as a matrix3d. For a voxel at
(x, y, z), the bake computes

    (xp, zp) = rot(x - cx, z - cz, facing)
    sx = (xp - zp) * half_w
    sy = (xp + zp) * half_w / 2 - y * cube_h

which is linear in (xp, y, zp) and therefore an orthographic 4x4. The model
unit below is the CSS size of one voxel cube; the camera carries the scale.
"""

import json
import sys

# CSS px per voxel in model space. The camera scales this down to the bake's
# half_w, so the face boxes stay large enough to be authored in whole pixels.
UNIT = 64.0


def camera_matrix3d(body):
    """The bake's orthographic projection as a CSS matrix3d()."""
    s = float(body["half_w"])
    e = float(body["cube_h"])
    ox = float(body["sheet"]["ox"])
    oy = float(body["sheet"]["oy"])
    # Columns are the screen images of the model's unit axes. CSS Y points
    # down, and the bake's y points up, so the model Y column is +cube_h.
    col_x = (s / UNIT, (s / 2.0) / UNIT, 1.0 / UNIT, 0.0)
    col_y = (0.0, e / UNIT, -1.0 / UNIT, 0.0)
    col_z = (-s / UNIT, (s / 2.0) / UNIT, 1.0 / UNIT, 0.0)
    col_w = (ox, oy, 0.0, 1.0)
    cells = col_x + col_y + col_z + col_w
    return "matrix3d(%s)" % ", ".join(fmt(v) for v in cells)


def fmt(value):
    text = "%.6f" % value
    return text.rstrip("0").rstrip(".") if "." in text else text


# One cube's six faces: (class, transform). Each face box is UNIT square with
# its origin at the cube's own corner, so the cube fills [0, UNIT]^3.
def faces():
    u = fmt(UNIT)
    return [
        ("front", "translate3d(0, 0, %spx)" % u),
        ("back", "translate3d(%spx, 0, 0) rotate3d(0, 1, 0, 180deg)" % u),
        ("right", "translate3d(%spx, 0, %spx) rotate3d(0, 1, 0, 90deg)" % (u, u)),
        ("left", "rotate3d(0, 1, 0, -90deg)"),
        ("top", "rotate3d(1, 0, 0, 90deg)"),
        ("bottom", "translate3d(0, %spx, %spx) rotate3d(1, 0, 0, -90deg)" % (u, u)),
    ]


PALETTE = ["#c84646", "#46a0c8", "#5abe6e"]

# Where the rotated unit box has to move so it covers its own grid cell again,
# in voxel units of the rotated frame, indexed by facing.
FACING_BOX_ORIGIN = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]


def emit(body, path):
    width = body["sheet"]["w"]
    height = body["sheet"]["h"]
    cx, cz = body["centre"]
    facing = body["facing"]
    by_part = {}
    for cell in body["cells"]:
        by_part.setdefault(cell["part"], []).append(cell)

    out = []
    out.append("<!DOCTYPE html>")
    out.append('<html><head><meta charset="utf-8"><title>wing body</title><style>')
    out.append("html, body { margin: 0; padding: 0; background: #ffffff; }")
    out.append(
        "#camera, .part, .voxel { position: absolute; left: 0; top: 0; "
        "width: 0; height: 0; transform-origin: 0 0; transform-style: preserve-3d; }"
    )
    out.append(
        ".face { position: absolute; left: 0; top: 0; width: %spx; height: %spx; "
        "transform-origin: 0 0; backface-visibility: hidden; }" % (fmt(UNIT), fmt(UNIT))
    )
    for index, color in enumerate(PALETTE):
        out.append(".p%d { background: %s; }" % (index, color))
    out.append("</style></head><body>")
    out.append('<div id="camera" style="transform: %s">' % camera_matrix3d(body))
    for name, cells in by_part.items():
        # Each rigid part carries the facing rotation and the model centring.
        # `rotate3d` here is the body's facing; a part's own yaw would be the
        # individual `rotate` property on this same element.
        # The bake rotates voxel *positions* and stamps an axis-aligned cube,
        # so a real rotation of the unit box lands one cell off for every
        # quarter turn. FACING_BOX_ORIGIN puts the rotated box back on its
        # grid cell; at facing 0 it is the identity.
        box_x, box_z = FACING_BOX_ORIGIN[facing & 3]
        transform = (
            "translate3d(%spx, 0, %spx) rotate3d(0, 1, 0, %sdeg) "
            "translate3d(%spx, 0, %spx)"
        ) % (
            fmt(box_x * UNIT),
            fmt(box_z * UNIT),
            fmt(-90.0 * facing),
            fmt(-cx * UNIT),
            fmt(-cz * UNIT),
        )
        out.append('<div class="part" data-part="%s" style="transform: %s">'
                   % (name, transform))
        for cell in cells:
            place = "translate3d(%spx, %spx, %spx)" % (
                fmt(cell["x"] * UNIT),
                fmt(-cell["y"] * UNIT),
                fmt(cell["z"] * UNIT),
            )
            out.append('<div class="voxel" style="transform: %s">' % place)
            for face, transform in faces():
                out.append('<div class="face p%d %s" style="transform: %s"></div>'
                           % (cell["pal"], face, transform))
            out.append("</div>")
        out.append("</div>")
    out.append("</div>")
    out.append("</body></html>")
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\n".join(out) + "\n")
    print("wrote %s (%dx%d, %d voxels)" % (path, width, height, len(body["cells"])))


def read_pgm(path):
    import numpy

    with open(path, "rb") as handle:
        data = handle.read()
    fields = []
    index = 0
    while len(fields) < 4:
        end = index
        while data[end : end + 1] not in (b" ", b"\n", b"\t"):
            end += 1
        fields.append(data[index:end])
        index = end + 1
    width, height = int(fields[1]), int(fields[2])
    pixels = numpy.frombuffer(data[index : index + width * height], dtype=numpy.uint8)
    return pixels.reshape(height, width) > 0


def compare(body, bake_path, render_path, mask_path=None):
    import numpy
    from PIL import Image

    bake = read_pgm(bake_path)
    height, width = bake.shape
    image = numpy.asarray(Image.open(render_path).convert("RGB"))
    # Ortet presents at the display scale, so the readback is an integer
    # multiple of the CSS sheet. Sample one device pixel per CSS pixel rather
    # than averaging, which would blur the edges the comparison is measuring.
    scale = max(1, image.shape[1] // width)
    if image.shape[1] < width * scale or image.shape[0] < height * scale:
        raise SystemExit(
            "readback %dx%d cannot cover the %dx%d sheet at scale %d"
            % (image.shape[1], image.shape[0], width, height, scale)
        )
    sampled = image[: height * scale : scale, : width * scale : scale]
    # The document paints on white; anything the body drew is darker.
    render = sampled.max(axis=2) < 250

    both = int(numpy.count_nonzero(bake & render))
    only_bake = int(numpy.count_nonzero(bake & ~render))
    only_render = int(numpy.count_nonzero(~bake & render))
    union = both + only_bake + only_render

    # A one-pixel tolerance: the bake stamps hard-edged integer triangles whose
    # grid is a pixel wider than the quad, while the CSS rasterizer antialiases
    # the same quads. Anything further apart than one pixel is a real
    # disagreement about geometry.
    outside_render = int(numpy.count_nonzero(bake & ~dilate(render)))
    outside_bake = int(numpy.count_nonzero(render & ~dilate(bake)))

    print("sheet            %dx%d at device scale %d" % (width, height, scale))
    print("bake pixels      %d" % int(numpy.count_nonzero(bake)))
    print("render pixels    %d" % int(numpy.count_nonzero(render)))
    print("agree            %d" % both)
    print("bake only        %d" % only_bake)
    print("render only      %d" % only_render)
    print("union            %d" % union)
    print("agreement        %.4f%%" % (100.0 * both / union if union else 0.0))
    print("bake px further than 1px from the render    %d" % outside_render)
    print("render px further than 1px from the bake    %d" % outside_bake)
    if mask_path:
        overlay = numpy.zeros((height, width, 3), dtype=numpy.uint8)
        overlay[bake & render] = (255, 255, 255)
        overlay[bake & ~render] = (255, 0, 0)
        overlay[~bake & render] = (0, 80, 255)
        Image.fromarray(overlay).save(mask_path)
        print("mask             %s" % mask_path)
    return outside_render + outside_bake


def dilate(mask):
    import numpy

    out = mask.copy()
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            out |= numpy.roll(numpy.roll(mask, dy, axis=0), dx, axis=1)
    return out


def main(argv):
    if len(argv) < 2:
        raise SystemExit(__doc__)
    command = argv[1]
    if command == "emit":
        body = json.load(open(argv[2], encoding="utf-8"))
        emit(body, argv[3])
    elif command == "compare":
        body = json.load(open(argv[2], encoding="utf-8"))
        compare(body, argv[3], argv[4], argv[5] if len(argv) > 5 else None)
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main(sys.argv)
