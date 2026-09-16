#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""T1's two headless readback receipts.

  depth  — a `preserve-3d` context with two overlapping faces paints them in
           transformed depth order, in both orderings, the second obtained by
           flipping the context's rotation rather than the DOM order.
  yaw    — an individual `rotate` turns one part's yaw across frames. The
           paint-list half of this receipt is the genet-livery unit test
           `an_individual_rotate_moves_alone_while_the_parent_matrix_is_unchanged`;
           this is the rendered half.

    css3d_receipts.py emit <out-dir>
    css3d_receipts.py check <out-dir> <renders-dir>

`check` reads the PNGs Ortet wrote for each emitted document and prints one
line per assertion. It exits non-zero if any assertion fails.
"""

import json
import os
import sys

SIZE = (400, 300)

DEPTH_NEAR = "rgb(220, 60, 60)"
DEPTH_FAR = "rgb(60, 90, 220)"

# The two faces are concentric, so the context's half turn keeps them on the
# same pixels and only their depth order changes. This is the sample point.
OVERLAP = (120, 120)

DEPTH_TEMPLATE = """<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>preserve-3d depth order</title><style>
html, body {{ margin: 0; padding: 0; background: #ffffff; }}
#context {{ position: absolute; left: 20px; top: 20px; width: 200px; height: 200px;
  transform-origin: 100px 100px; transform-style: preserve-3d; transform: {context}; }}
.face {{ position: absolute; transform-origin: 100px 100px; }}
/* DOM order is near-then-far in both documents, so a correct order here is the
   depth sort and not the tree order. The far face is the larger of the two, so
   the losing face is still visible as a border rather than vanishing. */
#near {{ left: 40px; top: 40px; width: 120px; height: 120px; background: {near};
  transform-origin: 60px 60px; transform: translate3d(0, 0, 60px); }}
#far {{ left: 0; top: 0; width: 200px; height: 200px; background: {far};
  transform: translate3d(0, 0, -60px); }}
</style></head><body>
<div id="context"><div class="face" id="near"></div><div class="face" id="far"></div></div>
</body></html>
"""

YAW_TEMPLATE = """<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>individual rotate yaw {angle}</title><style>
html, body {{ margin: 0; padding: 0; background: #ffffff; }}
#body {{ position: absolute; left: 0; top: 0; width: 0; height: 0;
  transform-origin: 0 0; transform-style: preserve-3d;
  transform: matrix3d(0.5, 0.25, 0.0625, 0, 0, 0.5, -0.0625, 0, -0.5, 0.25, 0.0625, 0,
                      100, 100, 0, 1); }}
.part {{ position: absolute; left: 0; top: 0; width: 0; height: 0;
  transform-origin: 0 0; transform-style: preserve-3d; }}
/* The trunk's matrix is a fixed translate3d and never changes. Only the arm's
   individual `rotate` differs between frames, and the two never overlap on
   screen, so the trunk's pixels are a direct check that it did not move. */
#trunk {{ transform: translate3d(0, 0, 0); }}
#arm {{ transform: translate3d(320px, 0, 0); rotate: y {angle}deg; }}
.face {{ position: absolute; left: 0; top: 0; width: 128px; height: 128px;
  transform-origin: 0 0; backface-visibility: hidden; }}
.trunkface {{ background: rgb(120, 130, 150); }}
.armface {{ background: rgb(210, 140, 60); }}
</style></head><body>
<div id="body">
  <div class="part" id="trunk">
    <div class="face trunkface" style="transform: rotate3d(1, 0, 0, 90deg)"></div>
    <div class="face trunkface" style="transform: translate3d(0, 0, 128px)"></div>
    <div class="face trunkface" style="transform: rotate3d(0, 1, 0, -90deg)"></div>
  </div>
  <div class="part" id="arm">
    <div class="face armface" style="transform: rotate3d(1, 0, 0, 90deg)"></div>
    <div class="face armface" style="transform: translate3d(0, 0, 128px)"></div>
    <div class="face armface" style="transform: rotate3d(0, 1, 0, -90deg)"></div>
  </div>
</div>
</body></html>
"""

YAW_ANGLES = [0, 30, 60, 90]


def emit(out_dir):
    os.makedirs(out_dir, exist_ok=True)
    documents = []
    for name, context in [
        ("depth_front", "none"),
        ("depth_flipped", "rotate3d(0, 1, 0, 180deg)"),
    ]:
        path = os.path.join(out_dir, name + ".html")
        write(path, DEPTH_TEMPLATE.format(context=context, near=DEPTH_NEAR, far=DEPTH_FAR))
        documents.append({"name": name, "path": path, "size": "%dx%d" % SIZE})
    for angle in YAW_ANGLES:
        name = "yaw_%03d" % angle
        path = os.path.join(out_dir, name + ".html")
        write(path, YAW_TEMPLATE.format(angle=angle))
        documents.append({"name": name, "path": path, "size": "%dx%d" % SIZE})
    index = os.path.join(out_dir, "documents.json")
    with open(index, "w", encoding="utf-8", newline="\n") as handle:
        json.dump(documents, handle, indent=2)
    for document in documents:
        print(document["path"])


def write(path, text):
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(text)


def sample(image, point):
    """One CSS pixel, sampled from a readback at the display scale."""
    import numpy

    scale = max(1, image.shape[1] // SIZE[0])
    pixel = image[point[1] * scale, point[0] * scale]
    return tuple(int(channel) for channel in pixel[:3])


def near(colour, expected, tolerance=12):
    return all(abs(a - b) <= tolerance for a, b in zip(colour, expected))


def check(out_dir, renders):
    import numpy
    from PIL import Image

    failures = 0

    def load(name):
        return numpy.asarray(
            Image.open(os.path.join(renders, name + ".png")).convert("RGB")
        )

    def report(ok, text):
        nonlocal failures
        if not ok:
            failures += 1
        print("%s  %s" % ("ok  " if ok else "FAIL", text))

    front = sample(load("depth_front"), OVERLAP)
    flipped = sample(load("depth_flipped"), OVERLAP)
    report(
        near(front, (220, 60, 60)),
        "depth_front overlap is the near face %s (expected the red face)" % (front,),
    )
    report(
        near(flipped, (60, 90, 220)),
        "depth_flipped overlap is the far face %s (expected the blue face)" % (flipped,),
    )
    report(front != flipped, "flipping the context reversed the paint order")

    masks = []
    for angle in YAW_ANGLES:
        image = load("yaw_%03d" % angle)
        arm = numpy.all(numpy.abs(image.astype(int) - (210, 140, 60)) <= 12, axis=2)
        trunk = numpy.all(numpy.abs(image.astype(int) - (120, 130, 150)) <= 12, axis=2)
        masks.append((angle, int(arm.sum()), int(trunk.sum()), arm, trunk))
    for angle, arm_px, trunk_px, _, _ in masks:
        report(arm_px > 0, "yaw %3d: the arm is on screen (%d px)" % (angle, arm_px))
    trunk_counts = {entry[2] for entry in masks}
    report(
        len(trunk_counts) == 1,
        "the trunk covers the same pixel count in every yaw frame %s"
        % sorted(trunk_counts),
    )
    first_trunk = masks[0][4]
    for angle, _, _, _, trunk in masks[1:]:
        report(
            bool(numpy.array_equal(first_trunk, trunk)),
            "the trunk is pixel-identical at yaw %d" % angle,
        )
    for (angle, _, _, arm, _), (next_angle, _, _, next_arm, _) in zip(masks, masks[1:]):
        changed = int(numpy.count_nonzero(arm != next_arm))
        report(
            changed > 0,
            "the arm moved between yaw %d and yaw %d (%d px changed)"
            % (angle, next_angle, changed),
        )
    print("%d assertion(s) failed" % failures)
    return failures


def main(argv):
    if len(argv) < 3:
        raise SystemExit(__doc__)
    if argv[1] == "emit":
        emit(argv[2])
    elif argv[1] == "check":
        sys.exit(1 if check(argv[2], argv[3]) else 0)
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main(sys.argv)
