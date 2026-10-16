#!/usr/bin/env python3
"""Check the real client smoke output. Requires Pillow; never starts a fake server."""
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image


def verify(output):
    proof_path = output / "verification.json"
    proof_path.unlink(missing_ok=True)
    report_path = output / "report.json"
    report = json.loads(report_path.read_bytes())
    assert report["schema"] == 1 and report["completed"] and report["offline"]
    world = report.get("mode", "movement") == "world"
    expected = ([(59, [0., .9, 0.]), (143, [-7., .9, 0.]), (263, [-7., .9, -2.54])]
                if world else [(59, [0., .9, 0.]), (119, [0., .9, -5.]), (179, [5., .9, -5.])])
    assert len(report["checkpoints"]) == len(expected)
    for actual, (frame, position) in zip(report["checkpoints"], expected):
        assert actual["frame"] == frame
        assert len(actual["position"]) == 3
        assert all(abs(a - b) < .001 for a, b in zip(actual["position"], position)), actual
    if world:
        blocked, clear = report["camera_checks"]
        assert blocked["frame"] == 279 and 0 < blocked["distance"] < 1
        assert clear["frame"] == 298 and abs(clear["distance"] - 10) < .001
        assert abs(clear["yaw"] - .8) < .001
    image_path = output / "scene.png"
    image = Image.open(image_path).convert("RGB")
    assert list(image.size) == report["extent"] == [1280, 720]
    colours = image.getcolors(image.width * image.height)
    assert colours and len(colours) >= 256, "Expected a rendered scene, not a clear image"
    green = sum(count for count, (r, g, b) in colours if g > r * 1.1 and g > b * 1.1)
    assert green > 10_000, "Ground is not visible"
    proof = dict(passed=True, checkpoints=len(expected), extent=report["extent"],
                 distinct_colours=len(colours), green_pixels=green, gpu=report["gpu"],
                 report_sha256=hashlib.sha256(report_path.read_bytes()).hexdigest(),
                 image_sha256=hashlib.sha256(image_path.read_bytes()).hexdigest(),
                 mode="world" if world else "movement",
                 scope="scripted CPU poses/camera and rendered-image presence; no network authority, imported geometry or pixel-exact oracle")
    proof_path.write_text(json.dumps(proof, indent=2) + "\n")
    return proof


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.output), indent=2))
