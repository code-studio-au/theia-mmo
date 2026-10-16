#!/usr/bin/env python3
"""Check actual Bevy asset inspection output. Requires Pillow; no mocked loader."""
import argparse
import hashlib
import json
import math
from pathlib import Path

from PIL import Image


def verify(directory, asset):
    proof = directory / "verification.json"
    proof.unlink(missing_ok=True)
    report = json.loads((directory / "asset.json").read_text())
    assert report["schema"] == 1 and report["completed"]
    assert report["file"] == asset.name
    stats = report["stats"]
    assert stats["mesh_instances"] > 0 and stats["triangles"] > 0
    assert all(math.isfinite(v) for vector in stats["bounds"].values() for v in vector)
    assert all(v > 0 for v in stats["bounds"]["size"])
    png = directory / "scene.png"
    image = Image.open(png).convert("RGB")
    assert image.size == (1280, 720)
    colours = image.getcolors(image.width * image.height)
    assert colours and len(colours) > 256
    # White/neutral model pixels, excluding the blue background and green floor.
    # This is an image-presence check, supplemented by visual inspection.
    neutral = sum(count for count, (r, g, b) in colours if max(r, g, b)-min(r, g, b) < 5 and r > 60)
    assert neutral > 10_000, "Expected a substantial rendered reference model"
    result = {"passed": True, "file": asset.name, "stats": stats,
              "asset_sha256": hashlib.sha256(asset.read_bytes()).hexdigest(),
              "image_sha256": hashlib.sha256(png.read_bytes()).hexdigest(),
              "gpu": report["gpu"], "neutral_pixels": neutral,
              "scope": "scene import/bounds and reference image presence; not texture fidelity or gameplay collision"}
    proof.write_text(json.dumps(result, indent=2) + "\n")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("asset", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.directory, args.asset), indent=2))
